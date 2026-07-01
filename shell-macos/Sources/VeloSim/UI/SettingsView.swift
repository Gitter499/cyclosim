import SwiftUI
import VeloFFI
import VeloSimSupport

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

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            headerChrome

            ScrollView {
                VStack(alignment: .leading, spacing: 12) {
                    googleTilesSection
                    cesiumSection
                    bikegenSection
                    statusSection
                }
            }

            HStack {
                Spacer()
                footerActions
            }
        }
        .padding(16)
        .frame(minWidth: 440, minHeight: embeddedInTab ? 480 : 560)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(embeddedInTab ? Color(nsColor: .windowBackgroundColor) : Color.clear)
        .onAppear { loadFromStore() }
    }

    @ViewBuilder
    private var footerActions: some View {
        if !embeddedInTab {
            HStack {
                Spacer()
                Button("Done") { dismiss() }
                    .buttonStyle(VeloGlassPrimaryButtonStyle())
            }
        }
    }

    private var headerChrome: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Settings")
                .font(.title2.bold())
            Text("API keys stay in Keychain on this Mac.")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .veloGlassRoundedRect(cornerRadius: 14)
    }

    private var googleTilesSection: some View {
        VeloGlassSection("Google Photorealistic 3D Tiles") {
            secretField("Map Tiles API key", text: $googleKey)
            Text("Enable Map Tiles API in Google Cloud and restrict the key. Photorealistic tiles stream online during rides only — never cached to disk.")
                .font(.caption)
                .foregroundStyle(.secondary)
            Text("Attribution © Google is shown in the HUD when this provider is active.")
                .font(.caption2)
                .foregroundStyle(.tertiary)
        }
    }

    private var cesiumSection: some View {
        VeloGlassSection("Cesium ion") {
            secretField("Access token (optional)", text: $cesiumToken)
            Text("Used when no Google key is set. Without a token, VeloSim uses the public ion dev tileset (OSM Buildings sample).")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
    }

    private var bikegenSection: some View {
        VeloGlassSection("Bike generation") {
            Toggle("Prefer hosted generation (Meshy)", isOn: $preferHostedBikegen)
            secretField("Meshy API key", text: $meshyKey)
            Text("Offline placeholder import works without a key. Hosted Meshy/Tripo HTTP wiring is deferred — enabling hosted mode requires a key and shows a clear error until the API lands.")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
    }

    private var statusSection: some View {
        VeloGlassSection("Apply") {
            Button("Save & apply") {
                saveAndApply()
            }
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

    @ViewBuilder
    private func secretField(_ label: String, text: Binding<String>) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(label)
                .font(.caption.weight(.medium))
            SecureField("Paste key…", text: text)
                .textFieldStyle(.roundedBorder)
        }
    }

    private func loadFromStore() {
        googleKey = AppSecretsStore.load(account: .googleMapTilesApiKey) ?? ""
        cesiumToken = AppSecretsStore.load(account: .cesiumIonAccessToken) ?? ""
        meshyKey = AppSecretsStore.load(account: .meshyApiKey) ?? ""
        preferHostedBikegen = AppSettingsStore.preferHostedBikeGeneration
        saveStatus = model.tilesProviderStatus
    }

    private func saveAndApply() {
        saveError = nil
        do {
            try AppSecretsStore.save(googleKey, account: .googleMapTilesApiKey)
            try AppSecretsStore.save(cesiumToken, account: .cesiumIonAccessToken)
            try AppSecretsStore.save(meshyKey, account: .meshyApiKey)
            AppSettingsStore.preferHostedBikeGeneration = preferHostedBikegen
            model.applyRuntimeSecrets()
            saveStatus = "Saved. \(model.tilesProviderStatus)"
            if preferHostedBikegen && AppSecretsStore.load(account: .meshyApiKey) == nil {
                saveError = "Hosted bike generation needs a Meshy API key."
            }
        } catch {
            saveError = "Keychain save failed: \(error)"
        }
    }
}

struct SettingsView_Previews: PreviewProvider {
    static var previews: some View {
        SettingsView(model: VeloSimModel())
    }
}
