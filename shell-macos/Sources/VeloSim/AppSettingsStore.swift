import Foundation

/// Non-secret app preferences (UserDefaults).
public enum AppSettingsStore {
    private static let preferHostedBikeGenerationKey = "velosim.preferHostedBikeGeneration"
    private static let defaultSteeringModeKey = "velosim.defaultSteeringMode"
    private static let segmentMusicEnabledKey = "velosim.segmentMusicEnabled"
    private static let hudMinimalModeKey = "velosim.hudMinimalMode"
    private static let pinnedRouteIdKey = "velosim.pinnedRouteId"
    private static let pinnedWorkoutNameKey = "velosim.pinnedWorkoutName"
    private static let riderNameKey = "velosim.riderName"
    private static let riderWeightKgKey = "velosim.riderWeightKg"

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

    public static var hudMinimalMode: Bool {
        get { UserDefaults.standard.bool(forKey: hudMinimalModeKey) }
        set { UserDefaults.standard.set(newValue, forKey: hudMinimalModeKey) }
    }

    public static var pinnedRouteId: String? {
        get { UserDefaults.standard.string(forKey: pinnedRouteIdKey) }
        set {
            if let newValue {
                UserDefaults.standard.set(newValue, forKey: pinnedRouteIdKey)
            } else {
                UserDefaults.standard.removeObject(forKey: pinnedRouteIdKey)
            }
        }
    }

    public static var pinnedWorkoutName: String? {
        get { UserDefaults.standard.string(forKey: pinnedWorkoutNameKey) }
        set {
            if let newValue { UserDefaults.standard.set(newValue, forKey: pinnedWorkoutNameKey) }
            else { UserDefaults.standard.removeObject(forKey: pinnedWorkoutNameKey) }
        }
    }

    public static var riderName: String {
        get { UserDefaults.standard.string(forKey: riderNameKey) ?? "Rider" }
        set { UserDefaults.standard.set(newValue, forKey: riderNameKey) }
    }

    public static var riderWeightKg: Double {
        get { let v = UserDefaults.standard.double(forKey: riderWeightKgKey); return v > 0 ? v : 75 }
        set { UserDefaults.standard.set(newValue, forKey: riderWeightKgKey) }
    }
}
