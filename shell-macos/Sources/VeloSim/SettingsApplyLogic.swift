import Foundation
import VeloFFI

/// Persists Settings form fields to Keychain / UserDefaults (testable without VeloSimModel).
public enum SettingsApplyLogic {
    public struct FormState: Equatable {
        public var googleKey: String
        public var cesiumToken: String
        public var meshyKey: String
        public var preferHostedBikegen: Bool

        public init(
            googleKey: String = "",
            cesiumToken: String = "",
            meshyKey: String = "",
            preferHostedBikegen: Bool = false
        ) {
            self.googleKey = googleKey
            self.cesiumToken = cesiumToken
            self.meshyKey = meshyKey
            self.preferHostedBikegen = preferHostedBikegen
        }
    }

    public enum ApplyOutcome: Equatable {
        case success(statusMessage: String, warning: String?)
        case keychainFailed(String)
    }

    public static func loadFormState() -> FormState {
        FormState(
            googleKey: AppSecretsStore.load(account: .googleMapTilesApiKey) ?? "",
            cesiumToken: AppSecretsStore.load(account: .cesiumIonAccessToken) ?? "",
            meshyKey: AppSecretsStore.load(account: .meshyApiKey) ?? "",
            preferHostedBikegen: AppSettingsStore.preferHostedBikeGeneration
        )
    }

    public static func apply(_ form: FormState, tilesProviderStatus: String) -> ApplyOutcome {
        do {
            try AppSecretsStore.save(form.googleKey, account: .googleMapTilesApiKey)
            try AppSecretsStore.save(form.cesiumToken, account: .cesiumIonAccessToken)
            try AppSecretsStore.save(form.meshyKey, account: .meshyApiKey)
            AppSettingsStore.preferHostedBikeGeneration = form.preferHostedBikegen

            var warning: String?
            if form.preferHostedBikegen, AppSecretsStore.load(account: .meshyApiKey) == nil {
                warning = "Hosted bike generation needs a Meshy API key."
            }

            return .success(statusMessage: "Saved. \(tilesProviderStatus)", warning: warning)
        } catch {
            return .keychainFailed("Keychain save failed: \(error)")
        }
    }

    public static func googleKeyConfigured() -> Bool {
        AppSecretsStore.load(account: .googleMapTilesApiKey) != nil
    }

    public static func cesiumTokenConfigured() -> Bool {
        AppSecretsStore.load(account: .cesiumIonAccessToken) != nil
    }

    public static func meshyKeyConfigured() -> Bool {
        AppSecretsStore.load(account: .meshyApiKey) != nil
    }
}
