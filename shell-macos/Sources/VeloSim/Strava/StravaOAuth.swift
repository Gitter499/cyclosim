import Foundation
import CryptoKit

/// OAuth2 PKCE helpers for Strava (shell-only).
public enum StravaOAuth {
  public static let authorizeURL = URL(string: "https://www.strava.com/oauth/authorize")!
  public static let tokenURL = URL(string: "https://www.strava.com/oauth/token")!

  public struct PKCEChallenge: Equatable {
    public let verifier: String
    public let challenge: String

    public init(verifier: String, challenge: String) {
      self.verifier = verifier
      self.challenge = challenge
    }
  }

  public static func generatePKCE() -> PKCEChallenge {
    var bytes = [UInt8](repeating: 0, count: 32)
    _ = SecRandomCopyBytes(kSecRandomDefault, bytes.count, &bytes)
    let verifier = Data(bytes).base64URLEncoded()
    let hash = SHA256.hash(data: Data(verifier.utf8))
    let challenge = Data(hash).base64URLEncoded()
    return PKCEChallenge(verifier: verifier, challenge: challenge)
  }

  public static func authorizationURL(config: StravaConfig, pkce: PKCEChallenge) -> URL {
    var components = URLComponents(url: authorizeURL, resolvingAgainstBaseURL: false)!
    components.queryItems = [
      URLQueryItem(name: "client_id", value: config.clientId),
      URLQueryItem(name: "redirect_uri", value: config.redirectURI),
      URLQueryItem(name: "response_type", value: "code"),
      URLQueryItem(name: "approval_prompt", value: "auto"),
      URLQueryItem(name: "scope", value: "activity:write"),
      URLQueryItem(name: "code_challenge", value: pkce.challenge),
      URLQueryItem(name: "code_challenge_method", value: "S256"),
    ]
    return components.url!
  }

  public static func parseCallback(url: URL) -> String? {
    guard let components = URLComponents(url: url, resolvingAgainstBaseURL: false),
          let code = components.queryItems?.first(where: { $0.name == "code" })?.value
    else { return nil }
    return code
  }

  public struct TokenResponse: Decodable, Equatable {
    public let accessToken: String
    public let refreshToken: String
    public let expiresAt: TimeInterval

    enum CodingKeys: String, CodingKey {
      case accessToken = "access_token"
      case refreshToken = "refresh_token"
      case expiresAt = "expires_at"
    }
  }

  public static func exchangeCode(
    code: String,
    config: StravaConfig,
    verifier: String,
    session: URLSession = .shared
  ) async throws -> StravaTokens {
    var request = URLRequest(url: tokenURL)
    request.httpMethod = "POST"
    request.setValue("application/json", forHTTPHeaderField: "Content-Type")
    let body: [String: Any] = [
      "client_id": config.clientId,
      "client_secret": config.clientSecret,
      "code": code,
      "grant_type": "authorization_code",
      "code_verifier": verifier,
    ]
    request.httpBody = try JSONSerialization.data(withJSONObject: body)
    let (data, response) = try await session.data(for: request)
    try validateHTTP(response)
    let decoded = try JSONDecoder().decode(TokenResponse.self, from: data)
    return StravaTokens(
      accessToken: decoded.accessToken,
      refreshToken: decoded.refreshToken,
      expiresAt: decoded.expiresAt
    )
  }

  public static func refresh(
    tokens: StravaTokens,
    config: StravaConfig,
    session: URLSession = .shared
  ) async throws -> StravaTokens {
    var request = URLRequest(url: tokenURL)
    request.httpMethod = "POST"
    request.setValue("application/json", forHTTPHeaderField: "Content-Type")
    let body: [String: Any] = [
      "client_id": config.clientId,
      "client_secret": config.clientSecret,
      "grant_type": "refresh_token",
      "refresh_token": tokens.refreshToken,
    ]
    request.httpBody = try JSONSerialization.data(withJSONObject: body)
    let (data, response) = try await session.data(for: request)
    try validateHTTP(response)
    let decoded = try JSONDecoder().decode(TokenResponse.self, from: data)
    return StravaTokens(
      accessToken: decoded.accessToken,
      refreshToken: decoded.refreshToken,
      expiresAt: decoded.expiresAt
    )
  }

  private static func validateHTTP(_ response: URLResponse) throws {
    guard let http = response as? HTTPURLResponse, (200 ... 299).contains(http.statusCode) else {
      throw StravaOAuthError.httpFailed
    }
  }
}

public enum StravaOAuthError: Error, Equatable {
  case httpFailed
  case notAuthenticated
}

private extension Data {
  func base64URLEncoded() -> String {
    base64EncodedString()
      .replacingOccurrences(of: "+", with: "-")
      .replacingOccurrences(of: "/", with: "_")
      .replacingOccurrences(of: "=", with: "")
  }
}
