import Foundation

/// Non-secret app preferences (UserDefaults).
public enum AppSettingsStore {
    private static let preferHostedBikeGenerationKey = "velosim.preferHostedBikeGeneration"

    public static var preferHostedBikeGeneration: Bool {
        get { UserDefaults.standard.bool(forKey: preferHostedBikeGenerationKey) }
        set { UserDefaults.standard.set(newValue, forKey: preferHostedBikeGenerationKey) }
    }
}
