import Foundation

/// Strava API credentials — never commit real values.
public struct StravaConfig: Equatable {
  public let clientId: String
  public let clientSecret: String
  public let redirectURI: String

  public static let defaultRedirectURI = "velosim://oauth"

  public var isConfigured: Bool {
    !clientId.isEmpty && !clientSecret.isEmpty
  }

  public init(clientId: String, clientSecret: String, redirectURI: String) {
    self.clientId = clientId
    self.clientSecret = clientSecret
    self.redirectURI = redirectURI
  }

  /// Load from environment (`STRAVA_CLIENT_ID`, `STRAVA_CLIENT_SECRET`) or `StravaConfig.plist` in the app bundle.
  public static func load() -> StravaConfig {
    let envId = ProcessInfo.processInfo.environment["STRAVA_CLIENT_ID"] ?? ""
    let envSecret = ProcessInfo.processInfo.environment["STRAVA_CLIENT_SECRET"] ?? ""
    if !envId.isEmpty, !envSecret.isEmpty {
      return StravaConfig(clientId: envId, clientSecret: envSecret, redirectURI: defaultRedirectURI)
    }
    if let url = Bundle.main.url(forResource: "StravaConfig", withExtension: "plist"),
       let dict = NSDictionary(contentsOf: url) as? [String: String],
       let id = dict["client_id"], let secret = dict["client_secret"]
    {
      return StravaConfig(
        clientId: id,
        clientSecret: secret,
        redirectURI: dict["redirect_uri"] ?? defaultRedirectURI
      )
    }
    return StravaConfig(clientId: "", clientSecret: "", redirectURI: defaultRedirectURI)
  }
}
