import CoreBluetooth
import Foundation
import VeloFFI
import VeloSimBLE
import VeloSimSupport

/// CoreBluetooth FTMS client — sensor polling + trainer control for Rust core.
final class FTMSBridge: NSObject, SensorSourceCallback, TrainerControlCallback, @unchecked Sendable {
    private enum SetupPhase {
        case idle
        case discoveringCharacteristics
        case readingCapabilities
        case subscribing
        case requestingControl
        case starting
        case ready
    }

    private struct PendingControlCommand {
        let opcode: FTMS.ControlOpcode
        let payload: Data
    }

    private let queue = DispatchQueue(label: "velosim.ftms", qos: .userInteractive)
    private var central: CBCentralManager!

    private var trainerPeripheral: CBPeripheral?
    private var hrPeripheral: CBPeripheral?
    private var controlPoint: CBCharacteristic?
    private var featureCharacteristic: CBCharacteristic?
    private var powerRangeCharacteristic: CBCharacteristic?

    private var setupPhase: SetupPhase = .idle
    private var featureReadDone = false
    private var powerRangeReadDone = false
    private var hasControl = false
    private var controlCommandQueue: [PendingControlCommand] = []
    private var awaitingControlResponse = false
    private var pendingTargetPower: Double?
    private var pendingSimGrade: Double?
    private var pendingSimCrr: Double = 0.004
    private var pendingSimCwa: Double = 0.5

    private var latestSample = TelemetrySampleDto(
        elapsedMs: 0,
        powerW: nil,
        cadenceRpm: nil,
        heartRateBpm: nil,
        wheelSpeedMps: nil
    )
    private var startTime = Date()
    private var pendingSamples: [TelemetrySampleDto] = []
    private let lock = NSLock()

    private(set) var connectionState: String = "idle"
    private(set) var trainerStatus: String = "—"
    private(set) var capabilities = FitnessMachineCapabilities()
    private(set) var lastControlError: String?
    private(set) var lastTargetPower: Double = 0
    private(set) var lastSimGrade: Double = 0

    var onStateChange: ((String) -> Void)?
    var onCapabilitiesChange: ((FitnessMachineCapabilities) -> Void)?
    var onTrainerStatusChange: ((String) -> Void)?
    var onControlErrorChange: ((String?) -> Void)?

    override init() {
        super.init()
        central = CBCentralManager(
            delegate: self,
            queue: queue,
            options: [CBCentralManagerOptionRestoreIdentifierKey: FTMS.stateRestoreIdentifier]
        )
    }

    func startScanning() {
        queue.async { [weak self] in
            guard let self, self.central.state == .poweredOn else { return }
            self.connectionState = "scanning"
            self.notifyState()
            self.central.scanForPeripherals(
                withServices: [FTMS.service, FTMS.heartRateService],
                options: [CBCentralManagerScanOptionAllowDuplicatesKey: false]
            )
        }
    }

    func stopScanning() {
        queue.async { [weak self] in
            self?.central.stopScan()
        }
    }

    func disconnect() {
        queue.async { [weak self] in
            guard let self else { return }
            if let p = self.trainerPeripheral {
                self.central.cancelPeripheralConnection(p)
            }
            if let p = self.hrPeripheral {
                self.central.cancelPeripheralConnection(p)
            }
            self.resetConnectionState()
            self.connectionState = "disconnected"
            self.notifyState()
        }
    }

    // MARK: - SensorSourceCallback

    func pollSamples() -> [TelemetrySampleDto] {
        lock.lock()
        defer { lock.unlock() }
        let elapsedMs = UInt64(Date().timeIntervalSince(startTime) * 1000)
        return TelemetrySamplePoll.drain(
            latest: latestSample,
            pending: &pendingSamples,
            elapsedMs: elapsedMs
        )
    }

    // MARK: - TrainerControlCallback

    func setTargetPower(watts: Double) {
        queue.async { [weak self] in
            guard let self else { return }
            self.lastTargetPower = watts
            guard self.capabilities.supportsErg else { return }
            self.pendingTargetPower = watts
            self.flushPendingTrainerCommands()
        }
    }

    func setSimulation(grade: Double, crr: Double, cwa: Double) {
        queue.async { [weak self] in
            guard let self else { return }
            self.lastSimGrade = grade
            guard self.capabilities.supportsSimulation else { return }
            self.pendingSimGrade = grade
            self.pendingSimCrr = crr
            self.pendingSimCwa = cwa
            self.flushPendingTrainerCommands()
        }
    }

    func stop() {
        queue.async { [weak self] in
            guard let self else { return }
            self.enqueueControl(opcode: .stopOrPause, payload: Data())
        }
    }

    // MARK: - Connection setup

    private func resetConnectionState() {
        trainerPeripheral = nil
        hrPeripheral = nil
        controlPoint = nil
        featureCharacteristic = nil
        powerRangeCharacteristic = nil
        hasControl = false
        setupPhase = .idle
        featureReadDone = false
        powerRangeReadDone = false
        controlCommandQueue.removeAll()
        awaitingControlResponse = false
        capabilities = FitnessMachineCapabilities()
        trainerStatus = "—"
        lastControlError = nil
        notifyCapabilities()
        notifyTrainerStatus()
        notifyControlError()
    }

    private func beginSetupIfReady(peripheral: CBPeripheral) {
        guard peripheral === trainerPeripheral,
              controlPoint != nil,
              featureCharacteristic != nil else { return }
        setupPhase = .readingCapabilities
        featureReadDone = false
        powerRangeReadDone = powerRangeCharacteristic == nil
        connectionState = "reading capabilities"
        notifyState()
        peripheral.readValue(for: featureCharacteristic!)
        if let pr = powerRangeCharacteristic {
            peripheral.readValue(for: pr)
        }
    }

    private func capabilitiesReadsComplete() -> Bool {
        featureReadDone && powerRangeReadDone
    }

    private func tryFinishCapabilityReads() {
        guard setupPhase == .readingCapabilities, capabilitiesReadsComplete() else { return }
        onCapabilitiesRead()
    }

    private func onCapabilitiesRead() {
        guard setupPhase == .readingCapabilities else { return }
        setupPhase = .subscribing
        connectionState = "subscribing"
        notifyState()
        notifyCapabilities()

        guard let peripheral = trainerPeripheral else { return }
        for service in peripheral.services ?? [] where service.uuid == FTMS.service {
            for ch in service.characteristics ?? [] {
                switch ch.uuid {
                case FTMS.indoorBikeData, FTMS.fitnessMachineStatus, FTMS.fitnessMachineControlPoint:
                    peripheral.setNotifyValue(true, for: ch)
                case FTMS.trainingStatus where ch.properties.contains(.notify):
                    peripheral.setNotifyValue(true, for: ch)
                default:
                    break
                }
            }
        }
        setupPhase = .requestingControl
        connectionState = "requesting control"
        notifyState()
        enqueueControl(opcode: .requestControl, payload: Data())
    }

    private func onControlAcquired() {
        guard setupPhase == .requestingControl || setupPhase == .starting else { return }
        setupPhase = .starting
        connectionState = "starting"
        notifyState()
        enqueueControl(opcode: .startOrResume, payload: Data())
    }

    private func onTrainerReady() {
        setupPhase = .ready
        hasControl = true
        connectionState = "trainer ready"
        trainerStatus = "Ready"
        notifyState()
        notifyTrainerStatus()
        flushPendingTrainerCommands()
    }

    // MARK: - Control point queue

    private func enqueueControl(opcode: FTMS.ControlOpcode, payload: Data) {
        var data = Data([opcode.rawValue])
        data.append(payload)
        controlCommandQueue.append(PendingControlCommand(opcode: opcode, payload: data))
        processControlQueue()
    }

    private func processControlQueue() {
        guard !awaitingControlResponse,
              let cmd = controlCommandQueue.first,
              let peripheral = trainerPeripheral,
              let cp = controlPoint else { return }
        awaitingControlResponse = true
        peripheral.writeValue(cmd.payload, for: cp, type: .withResponse)
    }

    private func handleControlResponse(_ data: Data) {
        awaitingControlResponse = false
        if !controlCommandQueue.isEmpty {
            controlCommandQueue.removeFirst()
        }

        guard let response = FTMSParser.parseControlPointResponse(data) else {
            logControlFailure("unparseable response: \(FTMSParser.hexString(data))")
            processControlQueue()
            return
        }

        if response.result != .success {
            let msg = "\(response.requestOpcode): \(response.result.description) [\(FTMSParser.hexString(response.raw))]"
            logControlFailure(msg)
            if response.result == .controlNotPermitted {
                hasControl = false
                setupPhase = .requestingControl
                enqueueControl(opcode: .requestControl, payload: Data())
            }
            processControlQueue()
            return
        }

        lastControlError = nil
        notifyControlError()

        switch response.requestOpcode {
        case .requestControl:
            hasControl = true
            onControlAcquired()
        case .startOrResume:
            onTrainerReady()
        case .setTargetPower, .setIndoorBikeSimulation, .reset, .stopOrPause:
            break
        }

        processControlQueue()
    }

    private func logControlFailure(_ message: String) {
        print("[FTMS] control point error: \(message)")
        lastControlError = message
        notifyControlError()
    }

    private func flushPendingTrainerCommands() {
        guard setupPhase == .ready, hasControl else { return }

        if let watts = pendingTargetPower, capabilities.supportsErg {
            pendingTargetPower = nil
            let clamped = capabilities.powerRange?.clamp(watts) ?? Int16(max(0, min(watts, 2000)))
            lastTargetPower = Double(clamped)
            var payload = Data()
            payload.append(contentsOf: withUnsafeBytes(of: clamped.littleEndian) { Data($0) })
            enqueueControl(opcode: .setTargetPower, payload: payload)
        }

        if let grade = pendingSimGrade, capabilities.supportsSimulation {
            pendingSimGrade = nil
            lastSimGrade = grade
            let gradePct = Int16((grade * 100.0 * 100.0).rounded())
            let wind: Int16 = 0
            let crrRaw = UInt8(min(255, max(0, Int((pendingSimCrr / 0.0001).rounded()))))
            let cwaRaw = UInt8(min(255, max(0, Int((pendingSimCwa / 0.01).rounded()))))
            var payload = Data()
            payload.append(contentsOf: withUnsafeBytes(of: wind.littleEndian) { Data($0) })
            payload.append(contentsOf: withUnsafeBytes(of: gradePct.littleEndian) { Data($0) })
            payload.append(crrRaw)
            payload.append(cwaRaw)
            enqueueControl(opcode: .setIndoorBikeSimulation, payload: payload)
        }
    }

    // MARK: - Telemetry

    private func pushSample(_ sample: TelemetrySampleDto) {
        lock.lock()
        latestSample = sample
        pendingSamples.append(sample)
        lock.unlock()
    }

    private func notifyState() {
        let state = connectionState
        DispatchQueue.main.async { [weak self] in
            self?.onStateChange?(state)
        }
    }

    private func notifyCapabilities() {
        let caps = capabilities
        DispatchQueue.main.async { [weak self] in
            self?.onCapabilitiesChange?(caps)
        }
    }

    private func notifyTrainerStatus() {
        let status = trainerStatus
        DispatchQueue.main.async { [weak self] in
            self?.onTrainerStatusChange?(status)
        }
    }

    private func notifyControlError() {
        let err = lastControlError
        DispatchQueue.main.async { [weak self] in
            self?.onControlErrorChange?(err)
        }
    }
}

// MARK: - CBCentralManagerDelegate

extension FTMSBridge: CBCentralManagerDelegate {
    func centralManagerDidUpdateState(_ central: CBCentralManager) {
        switch central.state {
        case .poweredOn:
            connectionState = "ready"
            notifyState()
        case .unauthorized:
            connectionState = "bluetooth unauthorized"
            notifyState()
        case .poweredOff:
            connectionState = "bluetooth off"
            notifyState()
        default:
            connectionState = "bluetooth unavailable"
            notifyState()
        }
    }

    func centralManager(_ central: CBCentralManager, willRestoreState dict: [String: Any]) {
        if let peripherals = dict[CBCentralManagerRestoredStatePeripheralsKey] as? [CBPeripheral] {
            for peripheral in peripherals {
                if peripheral.services?.contains(where: { $0.uuid == FTMS.service }) == true
                    || peripheral.name?.localizedCaseInsensitiveContains("KICKR") == true {
                    trainerPeripheral = peripheral
                    peripheral.delegate = self
                    connectionState = "restoring \(peripheral.name ?? "trainer")"
                    notifyState()
                    central.connect(peripheral, options: nil)
                } else if peripheral.services?.contains(where: { $0.uuid == FTMS.heartRateService }) == true {
                    hrPeripheral = peripheral
                    peripheral.delegate = self
                    central.connect(peripheral, options: nil)
                }
            }
        }
    }

    func centralManager(_ central: CBCentralManager, didDiscover peripheral: CBPeripheral,
                        advertisementData: [String: Any], rssi RSSI: NSNumber) {
        let name = peripheral.name ?? advertisementData[CBAdvertisementDataLocalNameKey] as? String ?? "Unknown"
        let services = advertisementData[CBAdvertisementDataServiceUUIDsKey] as? [CBUUID] ?? []
        let isFTMS = services.contains(FTMS.service) || name.localizedCaseInsensitiveContains("KICKR")
            || name.localizedCaseInsensitiveContains("Wahoo")
        let isHR = services.contains(FTMS.heartRateService)

        if isFTMS, trainerPeripheral == nil {
            trainerPeripheral = peripheral
            peripheral.delegate = self
            connectionState = "connecting \(name)"
            notifyState()
            central.connect(peripheral, options: nil)
        } else if isHR, hrPeripheral == nil {
            hrPeripheral = peripheral
            peripheral.delegate = self
            central.connect(peripheral, options: nil)
        }
    }

    func centralManager(_ central: CBCentralManager, didConnect peripheral: CBPeripheral) {
        startTime = Date()
        if peripheral === trainerPeripheral {
            setupPhase = .discoveringCharacteristics
            connectionState = "connected \(peripheral.name ?? "device")"
            notifyState()
            peripheral.discoverServices([FTMS.service])
        } else if peripheral === hrPeripheral {
            peripheral.discoverServices([FTMS.heartRateService])
        }
    }

    func centralManager(_ central: CBCentralManager, didFailToConnect peripheral: CBPeripheral, error: Error?) {
        connectionState = "connect failed"
        notifyState()
        if peripheral === trainerPeripheral {
            trainerPeripheral = nil
            setupPhase = .idle
        }
    }

    func centralManager(_ central: CBCentralManager, didDisconnectPeripheral peripheral: CBPeripheral, error: Error?) {
        if peripheral === trainerPeripheral {
            resetConnectionState()
        }
        if peripheral === hrPeripheral { hrPeripheral = nil }
        connectionState = "disconnected"
        notifyState()
    }
}

// MARK: - CBPeripheralDelegate

extension FTMSBridge: CBPeripheralDelegate {
    func peripheral(_ peripheral: CBPeripheral, didDiscoverServices error: Error?) {
        guard error == nil else { return }
        for service in peripheral.services ?? [] {
            if service.uuid == FTMS.service {
                peripheral.discoverCharacteristics(
                    [
                        FTMS.fitnessMachineFeature,
                        FTMS.supportedPowerRange,
                        FTMS.trainingStatus,
                        FTMS.indoorBikeData,
                        FTMS.fitnessMachineControlPoint,
                        FTMS.fitnessMachineStatus,
                    ],
                    for: service
                )
            } else if service.uuid == FTMS.heartRateService {
                peripheral.discoverCharacteristics([FTMS.heartRateMeasurement], for: service)
            }
        }
    }

    func peripheral(_ peripheral: CBPeripheral, didDiscoverCharacteristicsFor service: CBService, error: Error?) {
        guard error == nil, peripheral === trainerPeripheral, service.uuid == FTMS.service else {
            if service.uuid == FTMS.heartRateService {
                for ch in service.characteristics ?? [] where ch.uuid == FTMS.heartRateMeasurement {
                    peripheral.setNotifyValue(true, for: ch)
                }
            }
            return
        }

        for ch in service.characteristics ?? [] {
            switch ch.uuid {
            case FTMS.fitnessMachineFeature:
                featureCharacteristic = ch
            case FTMS.supportedPowerRange:
                powerRangeCharacteristic = ch
            case FTMS.fitnessMachineControlPoint:
                controlPoint = ch
            default:
                break
            }
        }
        beginSetupIfReady(peripheral: peripheral)
    }

    func peripheral(_ peripheral: CBPeripheral, didUpdateValueFor characteristic: CBCharacteristic, error: Error?) {
        guard error == nil, let data = characteristic.value else { return }

        switch characteristic.uuid {
        case FTMS.fitnessMachineFeature:
            var caps = FTMSParser.parseFitnessMachineFeature(data)
            if let existing = capabilities.powerRange {
                caps.powerRange = existing
            }
            capabilities = caps
            featureReadDone = true
            tryFinishCapabilityReads()

        case FTMS.supportedPowerRange:
            if let range = FTMSParser.parseSupportedPowerRange(data) {
                capabilities.powerRange = range
            }
            powerRangeReadDone = true
            tryFinishCapabilityReads()
            notifyCapabilities()

        case FTMS.indoorBikeData:
            let parsed = FTMSParser.parseIndoorBikeData(data)
            let elapsedMs = UInt64(Date().timeIntervalSince(startTime) * 1000)
            var sample = latestSample
            sample.elapsedMs = elapsedMs
            if let p = parsed.instantaneousPowerW { sample.powerW = p }
            if let c = parsed.instantaneousCadenceRpm { sample.cadenceRpm = c }
            if let s = parsed.instantaneousSpeedKmh { sample.wheelSpeedMps = s / 3.6 }
            if let hr = parsed.heartRateBpm {
                sample.heartRateBpm = Double(hr)
            }
            pushSample(sample)

        case FTMS.fitnessMachineStatus:
            if let status = FTMSParser.parseFitnessMachineStatus(data) {
                trainerStatus = status.description
                notifyTrainerStatus()
                if status.opcode == .controlPermissionLost {
                    hasControl = false
                    setupPhase = .requestingControl
                    enqueueControl(opcode: .requestControl, payload: Data())
                }
            }

        case FTMS.fitnessMachineControlPoint:
            handleControlResponse(data)

        case FTMS.heartRateMeasurement:
            if let hr = FTMSParser.parseHeartRate(data) {
                let elapsedMs = UInt64(Date().timeIntervalSince(startTime) * 1000)
                lock.lock()
                latestSample.heartRateBpm = hr
                latestSample.elapsedMs = elapsedMs
                let sample = latestSample
                lock.unlock()
                pushSample(sample)
            }

        default:
            break
        }
    }

    func peripheral(_ peripheral: CBPeripheral, didWriteValueFor characteristic: CBCharacteristic, error: Error?) {
        if let error {
            let msg = "write failed: \(error.localizedDescription)"
            logControlFailure(msg)
            awaitingControlResponse = false
            if !controlCommandQueue.isEmpty {
                controlCommandQueue.removeFirst()
            }
            processControlQueue()
        }
    }
}
