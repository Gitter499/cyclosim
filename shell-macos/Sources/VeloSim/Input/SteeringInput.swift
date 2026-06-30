import AppKit
import CoreMotion
import Foundation
import VeloFFI

/// How the rider steers the chase camera on supported routes.
public enum SteeringInputMode: String, CaseIterable, Identifiable, Sendable {
    case off
    case keyboard
    case airpods

    public var id: String { rawValue }

    public var label: String {
        switch self {
        case .off: return "Off"
        case .keyboard: return "Keyboard"
        case .airpods: return "AirPods"
        }
    }
}

/// Returns zero axis — used when steering is disabled.
public final class NoopSteeringInput: SteeringInputCallback, @unchecked Sendable {
    public init() {}

    public func poll() -> SteerStateDto {
        SteerStateDto(axis: 0, recenter: false)
    }
}

/// Arrow keys / A-D steering with Space to recenter.
@MainActor
public final class KeyboardSteeringInput: SteeringInputCallback, @unchecked Sendable {
    private var axis: Float = 0
    private var recenterPending = false
    private var monitor: Any?

    public init() {
        installMonitor()
    }

    deinit {
        if let monitor {
            NSEvent.removeMonitor(monitor)
        }
    }

    private func installMonitor() {
        monitor = NSEvent.addLocalMonitorForEvents(matching: [.keyDown, .keyUp, .flagsChanged]) { [weak self] event in
            guard let self else { return event }
            self.handle(event: event)
            return event
        }
    }

    private func handle(event: NSEvent) {
        let down = event.type == .keyDown
        switch event.keyCode {
        case 123, 0: // left arrow, A
            axis = down ? -1 : (axis < 0 ? 0 : axis)
        case 124, 2: // right arrow, D
            axis = down ? 1 : (axis > 0 ? 0 : axis)
        case 49 where down: // space — recenter
            recenterPending = true
            axis = 0
        default:
            break
        }
    }

    public func poll() -> SteerStateDto {
        let state = SteerStateDto(axis: axis, recenter: recenterPending)
        recenterPending = false
        return state
    }
}

/// AirPods head yaw → steering axis. Low-pass lives in Rust core; yaw delta mapped here.
@MainActor
public final class AirPodsSteeringInput: NSObject, SteeringInputCallback, @unchecked Sendable {
    private let motion = CMHeadphoneMotionManager()
    private var lastYaw: Double?
    private var axis: Float = 0
    private var recenterPending = false
    private var started = false

    public override init() {
        super.init()
    }

    public func start() {
        guard !started, motion.isDeviceMotionAvailable else { return }
        started = true
        lastYaw = nil
        motion.startDeviceMotionUpdates(to: .main) { [weak self] motion, _ in
            guard let self, let yaw = motion?.attitude.yaw else { return }
            if let prev = self.lastYaw {
                let delta = Float(yaw - prev)
                // Scale head turn to axis; core applies deadzone + smoothing.
                self.axis = (self.axis + delta * 2.5).clamped(to: -1 ... 1)
            }
            self.lastYaw = yaw
        }
    }

    public func stop() {
        guard started else { return }
        motion.stopDeviceMotionUpdates()
        started = false
        lastYaw = nil
        axis = 0
    }

    public func requestRecenter() {
        recenterPending = true
        axis = 0
        lastYaw = nil
    }

    public var isAvailable: Bool {
        motion.isDeviceMotionAvailable
    }

    public func poll() -> SteerStateDto {
        let state = SteerStateDto(axis: axis, recenter: recenterPending)
        recenterPending = false
        return state
    }
}

private extension Comparable {
    func clamped(to range: ClosedRange<Self>) -> Self {
        min(max(self, range.lowerBound), range.upperBound)
    }
}
