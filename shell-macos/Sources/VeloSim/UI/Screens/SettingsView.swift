import SwiftUI
import VeloFFI
import VeloSimSupport

enum SettingsStatusKind {
    case ok
    case warning
    case missing
    case neutral
}

struct SettingsStatusBadge: View {
    let label: String
    let kind: SettingsStatusKind

    var body: some View {
        Text(label)
            .font(.caption2.weight(.medium))
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(tint.opacity(0.22))
            .foregroundStyle(tint)
            .clipShape(Capsule())
    }

    private var tint: Color {
        switch kind {
        case .ok: return .green
        case .warning: return .orange
        case .missing: return .secondary
        case .neutral: return .blue
        }
    }
}

struct SettingsBlockedView<Content: View>: View {
    let isBlocked: Bool
    @ViewBuilder var content: () -> Content

    var body: some View {
        if isBlocked {
            VStack(spacing: 14) {
                Image(systemName: "lock.fill")
                    .font(.largeTitle)
                    .foregroundStyle(.secondary)
                Text("Settings unavailable during a ride")
                    .font(.headline)
                Text("Stop your ride to change API keys, trainer setup, or other preferences.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
                    .frame(maxWidth: 320)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .padding(24)
        } else {
            content()
        }
    }
}

@MainActor
struct SettingsView: View {
    @ObservedObject var model: VeloSimModel
    var embeddedInTab: Bool = false
    @Environment(\.dismiss) private var dismiss

    @State private var googleKey: String = ""
    @State private var cesiumToken: String = ""
    @State private var meshyKey: String = ""
    @State private var preferHostedBikegen: Bool = false
    @State private var saveStatus: String = ""
    @State private var saveError: String?
    @State private var advancedExpanded: Bool = false

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            headerChrome

            ScrollView {
                VStack(alignment: .leading, spacing: 12) {
                    connectedServicesSection
                    trainerSection
                    audioSection
                    steeringSection
                    advancedSection
                }
            }

            if !embeddedInTab {
                HStack {
                    Spacer()
                    Button("Done") { dismiss() }
                        .buttonStyle(VeloGlassPrimaryButtonStyle())
                }
            }
        }
        .padding(16)
        .frame(minWidth: 440, minHeight: embeddedInTab ? 480 : 560)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(embeddedInTab ? Color(nsColor: .windowBackgroundColor) : Color.clear)
        .onAppear { loadFromStore() }
    }

    private var headerChrome: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Settings")
                .font(.title2.bold())
            Text("API keys stay in Keychain on this Mac. Tile and bikegen providers apply on Save.")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .veloBrowseHeader(cornerRadius: 14)
    }

    private var connectedServicesSection: some View {
        VeloBrowseSection("Connected services") {
            serviceRow(
                title: "Google Photorealistic 3D Tiles",
                linkTitle: "Google Cloud credentials",
                linkURL: "https://console.cloud.google.com/apis/credentials",
                configured: SettingsApplyLogic.googleKeyConfigured() || !googleKey.isEmpty,
                statusLabel: model.tilesProviderStatus
            ) {
                secretField("Map Tiles API key", text: $googleKey)
                Text("Enable Map Tiles API and restrict the key. Tiles stream online during rides only — never cached to disk.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Text("Attribution © Google appears in the HUD when this provider is active.")
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }

            Divider()

            serviceRow(
                title: "Cesium ion",
                linkTitle: "ion.cesium.com/tokens",
                linkURL: "https://ion.cesium.com/tokens",
                configured: SettingsApplyLogic.cesiumTokenConfigured() || !cesiumToken.isEmpty,
                statusLabel: cesiumStatusLabel
            ) {
                secretField("Access token (optional)", text: $cesiumToken)
                Text("Used when no Google key is set. Without a token, VeloSim uses the public ion dev tileset.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Divider()

            VStack(alignment: .leading, spacing: 8) {
                HStack {
                    Text("Meshy (bike generation)")
                        .font(.subheadline.weight(.medium))
                    Spacer()
                    SettingsStatusBadge(
                        label: meshyBadgeLabel,
                        kind: meshyBadgeKind
                    )
                }

                Link("docs.meshy.ai", destination: URL(string: "https://docs.meshy.ai")!)
                    .font(.caption)

                Toggle("Prefer hosted generation (Meshy)", isOn: $preferHostedBikegen)

                secretField("Meshy API key", text: $meshyKey)

                Text(model.bikegenModeStatus)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)

                Text("Offline placeholder import works without a key. Hosted Meshy HTTP wiring is deferred — enabling hosted mode requires a key and shows a clear error until the API lands.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Button("Save & apply") { saveAndApply() }
                .buttonStyle(VeloGlassPrimaryButtonStyle())

            if let saveError {
                Text(saveError)
                    .font(.caption)
                    .foregroundStyle(.red)
            } else if !saveStatus.isEmpty {
                Text(saveStatus)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
    }

    private var trainerSection: some View {
        VeloBrowseSection("Trainer & sensors") {
            HStack {
                Text("Bluetooth")
                Spacer()
                SettingsStatusBadge(label: bleBadgeLabel, kind: bleBadgeKind)
            }

            if model.sensorMode == .bluetooth {
                VStack(alignment: .leading, spacing: 4) {
                    Text("State: \(model.bleState)")
                    Text("Capabilities: \(model.bleCapabilities)")
                    Text("Trainer: \(model.bleTrainerStatus)")
                    if let err = model.bleControlError {
                        Text("Control error: \(err)")
                            .foregroundStyle(.red)
                    }
                }
                .font(.caption)
                .foregroundStyle(.secondary)
            } else {
                Text("Sensor input is \(model.sensorMode.label). Switch to BLE and pair your trainer in pre-ride setup.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Button("Open Activities setup") {
                model.shellDestination = .activities
            }
            .buttonStyle(VeloGlassSecondaryButtonStyle())
        }
    }

    private var audioSection: some View {
        VeloBrowseSection("Audio") {
            Toggle("Shift music at workout intervals", isOn: Binding(
                get: { model.segmentMusicEnabled },
                set: { enabled in
                    AppSettingsStore.segmentMusicEnabled = enabled
                    model.setSegmentMusicEnabled(enabled)
                }
            ))

            Button("Connect Apple Music") { model.connectAppleMusic() }
                .buttonStyle(VeloGlassSecondaryButtonStyle())

            Text(model.musicStatus)
                .font(.caption)
                .foregroundStyle(.secondary)
                .lineLimit(2)

            Text("Playback control only — MusicKit queues catalog searches by workout interval energy. VeloSim does not mix raw PCM; selection is best-effort, not BPM-locked.")
                .font(.caption2)
                .foregroundStyle(.tertiary)
        }
    }

    private var steeringSection: some View {
        VeloBrowseSection("Steering") {
            Picker("Default mode", selection: Binding(
                get: { AppSettingsStore.defaultSteeringMode },
                set: { AppSettingsStore.defaultSteeringMode = $0 }
            )) {
                ForEach(SteeringInputMode.allCases) { mode in
                    Text(mode.label).tag(mode)
                }
            }
            .pickerStyle(.segmented)

            Text("Applied on launch and from pre-ride setup. Current ride uses \(model.steeringMode.label).")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
    }

    private var advancedSection: some View {
        VeloBrowseSection("Advanced") {
            DisclosureGroup("Developer tools", isExpanded: $advancedExpanded) {
                VStack(alignment: .leading, spacing: 10) {
                    Toggle("Draw Rust HUD (velo-render)", isOn: Binding(
                        get: { model.rustHudDrawEnabled },
                        set: { model.setRustHudDrawEnabled($0) }
                    ))

                    HStack {
                        Text("Toggle count")
                        Spacer()
                        Text("\(model.toggleCount)")
                            .monospacedDigit()
                            .foregroundStyle(.secondary)
                    }
                    .font(.caption)

                    Text("Rust log tail")
                        .font(.caption.weight(.medium))

                    ScrollView {
                        LazyVStack(alignment: .leading, spacing: 2) {
                            if model.logs.isEmpty {
                                Text("No log lines yet")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            } else {
                                ForEach(Array(model.logs.enumerated()), id: \.offset) { _, line in
                                    Text(line)
                                        .font(.caption.monospaced())
                                        .frame(maxWidth: .infinity, alignment: .leading)
                                }
                            }
                        }
                    }
                    .frame(maxHeight: 160)
                }
                .padding(.top, 6)
            }
        }
    }

    @ViewBuilder
    private func serviceRow<Fields: View>(
        title: String,
        linkTitle: String,
        linkURL: String,
        configured: Bool,
        statusLabel: String,
        @ViewBuilder fields: () -> Fields
    ) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text(title)
                    .font(.subheadline.weight(.medium))
                Spacer()
                SettingsStatusBadge(
                    label: configured ? "Key set" : "No key",
                    kind: configured ? .ok : .missing
                )
            }

            if !statusLabel.isEmpty {
                SettingsStatusBadge(label: statusLabel, kind: .neutral)
            }

            Link(linkTitle, destination: URL(string: linkURL)!)
                .font(.caption)

            fields()
        }
    }

    @ViewBuilder
    private func secretField(_ label: String, text: Binding<String>) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(label)
                .font(.caption.weight(.medium))
            SecureField("Paste key…", text: text)
                .textFieldStyle(.roundedBorder)
        }
    }

    private var cesiumStatusLabel: String {
        if SettingsApplyLogic.googleKeyConfigured() || !googleKey.isEmpty {
            return "Fallback (Google active)"
        }
        if SettingsApplyLogic.cesiumTokenConfigured() || !cesiumToken.isEmpty {
            return "ion token configured"
        }
        return "ion dev tileset"
    }

    private var meshyBadgeLabel: String {
        if preferHostedBikegen {
            return SettingsApplyLogic.meshyKeyConfigured() || !meshyKey.isEmpty ? "Hosted · key set" : "Hosted · no key"
        }
        return "Offline import"
    }

    private var meshyBadgeKind: SettingsStatusKind {
        if preferHostedBikegen {
            return SettingsApplyLogic.meshyKeyConfigured() || !meshyKey.isEmpty ? .ok : .warning
        }
        return .neutral
    }

    private var bleBadgeLabel: String {
        switch model.sensorMode {
        case .bluetooth:
            return model.bleState
        case .fake, .replay:
            return model.sensorMode.label
        }
    }

    private var bleBadgeKind: SettingsStatusKind {
        guard model.sensorMode == .bluetooth else { return .neutral }
        if model.bleControlError != nil { return .warning }
        if model.bleState.lowercased().contains("connect") { return .ok }
        return .neutral
    }

    private func loadFromStore() {
        let form = SettingsApplyLogic.loadFormState()
        googleKey = form.googleKey
        cesiumToken = form.cesiumToken
        meshyKey = form.meshyKey
        preferHostedBikegen = form.preferHostedBikegen
        saveStatus = model.tilesProviderStatus
    }

    private func saveAndApply() {
        saveError = nil
        let form = SettingsApplyLogic.FormState(
            googleKey: googleKey,
            cesiumToken: cesiumToken,
            meshyKey: meshyKey,
            preferHostedBikegen: preferHostedBikegen
        )
        switch SettingsApplyLogic.apply(form, tilesProviderStatus: model.tilesProviderStatus) {
        case let .success(message, warning):
            model.applyRuntimeSecrets()
            saveStatus = message
            saveError = warning
        case let .keychainFailed(message):
            saveError = message
        }
    }
}

struct SettingsView_Previews: PreviewProvider {
    static var previews: some View {
        SettingsView(model: VeloSimModel())
    }
}
