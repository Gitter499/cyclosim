import SwiftUI
import VeloSimSupport

@MainActor
struct IntegrationsKeysWizard: View {
    @ObservedObject var model: VeloSimModel
    @Binding var isPresented: Bool

    @State private var step: ConnectionWizardStep = .intro
    @State private var googleKey: String = ""
    @State private var cesiumToken: String = ""
    @State private var meshyKey: String = ""
    @State private var preferHostedBikegen: Bool = false
    @State private var saveStatus: String = ""
    @State private var saveError: String?
    @State private var testResults: [IntegrationTile: String] = [:]

    private enum IntegrationTile: String, CaseIterable {
        case google = "Google 3D Tiles"
        case cesium = "Cesium ion"
        case meshy = "Meshy bikegen"
    }

    var body: some View {
        ConnectionWizardChrome(step: $step, title: "Integrations", onClose: { isPresented = false }) {
            switch step {
            case .intro:
                introStep
            case .action:
                keysStep
            case .test:
                testStep
            case .done:
                doneStep
            }
        }
        .onAppear { loadFromStore() }
    }

    private var introStep: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("API keys for photorealistic 3D tiles and optional hosted bike generation. Stored in Keychain on this Mac.")
                .font(.caption)
                .foregroundStyle(.secondary)
            tileStatusRow(.google)
            tileStatusRow(.cesium)
            tileStatusRow(.meshy)
        }
    }

    private var keysStep: some View {
        VStack(alignment: .leading, spacing: 14) {
            keyField("Google Map Tiles API key", text: $googleKey, link: "https://console.cloud.google.com/apis/credentials")
            keyField("Cesium ion token (optional)", text: $cesiumToken, link: "https://ion.cesium.com/tokens")
            Toggle("Prefer hosted bikegen (Meshy)", isOn: $preferHostedBikegen)
            keyField("Meshy API key", text: $meshyKey, link: "https://docs.meshy.ai")

            Button("Save & apply") { saveAndApply() }
                .buttonStyle(VeloGlassPrimaryButtonStyle())

            if let saveError {
                Text(saveError).font(.caption).foregroundStyle(.red)
            } else if !saveStatus.isEmpty {
                Text(saveStatus).font(.caption).foregroundStyle(.secondary)
            }
        }
    }

    private var testStep: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Confirm each provider status after saving.")
                .font(.caption)
                .foregroundStyle(.secondary)

            ForEach(IntegrationTile.allCases, id: \.rawValue) { tile in
                HStack {
                    Text(tile.rawValue)
                    Spacer()
                    SettingsStatusBadge(
                        label: testResults[tile] ?? tileConfiguredLabel(tile),
                        kind: tileBadgeKind(tile)
                    )
                }
            }

            Button("Refresh status") { refreshTestResults() }
                .buttonStyle(VeloGlassSecondaryButtonStyle())

            Button("Continue") { step = .done }
                .buttonStyle(VeloGlassPrimaryButtonStyle())
        }
    }

    private var doneStep: some View {
        VStack(alignment: .leading, spacing: 10) {
            Label("Integrations saved", systemImage: "checkmark.seal.fill")
                .font(.title3.weight(.semibold))
                .foregroundStyle(.green)
            Text(model.tilesProviderStatus)
                .font(.caption)
                .foregroundStyle(.secondary)
            Text(model.bikegenModeStatus)
                .font(.caption2)
                .foregroundStyle(.tertiary)
        }
    }

    @ViewBuilder
    private func keyField(_ label: String, text: Binding<String>, link: String) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Link(label, destination: URL(string: link)!)
                .font(.caption.weight(.medium))
            SecureField("Paste key…", text: text)
                .textFieldStyle(.roundedBorder)
        }
    }

    private func tileStatusRow(_ tile: IntegrationTile) -> some View {
        HStack {
            Text(tile.rawValue)
                .font(.caption)
            Spacer()
            SettingsStatusBadge(label: tileConfiguredLabel(tile), kind: tileBadgeKind(tile))
        }
    }

    private func tileConfiguredLabel(_ tile: IntegrationTile) -> String {
        switch tile {
        case .google:
            return SettingsApplyLogic.googleKeyConfigured() || !googleKey.isEmpty ? "Key set" : "No key"
        case .cesium:
            return SettingsApplyLogic.cesiumTokenConfigured() || !cesiumToken.isEmpty ? "Token set" : "Optional"
        case .meshy:
            if preferHostedBikegen {
                return SettingsApplyLogic.meshyKeyConfigured() || !meshyKey.isEmpty ? "Key set" : "No key"
            }
            return "Offline import"
        }
    }

    private func tileBadgeKind(_ tile: IntegrationTile) -> SettingsStatusKind {
        switch tile {
        case .google:
            return SettingsApplyLogic.googleKeyConfigured() || !googleKey.isEmpty ? .ok : .missing
        case .cesium:
            return SettingsApplyLogic.cesiumTokenConfigured() || !cesiumToken.isEmpty ? .ok : .neutral
        case .meshy:
            if preferHostedBikegen {
                return SettingsApplyLogic.meshyKeyConfigured() || !meshyKey.isEmpty ? .ok : .warning
            }
            return .neutral
        }
    }

    private func loadFromStore() {
        let form = SettingsApplyLogic.loadFormState()
        googleKey = form.googleKey
        cesiumToken = form.cesiumToken
        meshyKey = form.meshyKey
        preferHostedBikegen = form.preferHostedBikegen
        saveStatus = model.tilesProviderStatus
        refreshTestResults()
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
            refreshTestResults()
            step = .test
        case let .keychainFailed(message):
            saveError = message
        }
    }

    private func refreshTestResults() {
        testResults[.google] = SettingsApplyLogic.googleKeyConfigured() ? "Ready" : "Missing"
        testResults[.cesium] = SettingsApplyLogic.cesiumTokenConfigured() ? "Configured" : "Dev tileset"
        if preferHostedBikegen {
            testResults[.meshy] = SettingsApplyLogic.meshyKeyConfigured() ? "Hosted ready" : "Needs key"
        } else {
            testResults[.meshy] = "Offline mode"
        }
    }
}
