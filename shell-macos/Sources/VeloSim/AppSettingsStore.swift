import Foundation

/// Non-secret app preferences (UserDefaults).
public enum AppSettingsStore {
    private static let preferHostedBikeGenerationKey = "velosim.preferHostedBikeGeneration"
    private static let defaultSteeringModeKey = "velosim.defaultSteeringMode"
    private static let segmentMusicEnabledKey = "velosim.segmentMusicEnabled"

    public static var preferHostedBikeGeneration: Bool {
        get { UserDefaults.standard.bool(forKey: preferHostedBikeGenerationKey) }
        set { UserDefaults.standard.set(newValue, forKey: preferHostedBikeGenerationKey) }
    }

    public static var defaultSteeringMode: SteeringInputMode {
        get {
            guard let raw = UserDefaults.standard.string(forKey: defaultSteeringModeKey),
                  let mode = SteeringInputMode(rawValue: raw) else { return .off }
            return mode
        }
        set { UserDefaults.standard.set(newValue.rawValue, forKey: defaultSteeringModeKey) }
    }

    public static var segmentMusicEnabled: Bool {
        get { UserDefaults.standard.bool(forKey: segmentMusicEnabledKey) }
        set { UserDefaults.standard.set(newValue, forKey: segmentMusicEnabledKey) }
    }
}
