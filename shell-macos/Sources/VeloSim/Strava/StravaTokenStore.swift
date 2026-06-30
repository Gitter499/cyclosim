import Foundation
import Security

public struct StravaTokens: Codable, Equatable {
  public var accessToken: String
  public var refreshToken: String
  public var expiresAt: TimeInterval

  public init(accessToken: String, refreshToken: String, expiresAt: TimeInterval) {
    self.accessToken = accessToken
    self.refreshToken = refreshToken
    self.expiresAt = expiresAt
  }
}

public enum StravaTokenStore {
  private static let service = "com.velosim.strava"
  private static let account = "oauth"

  public static func save(_ tokens: StravaTokens) throws {
    let data = try JSONEncoder().encode(tokens)
    let query: [String: Any] = [
      kSecClass as String: kSecClassGenericPassword,
      kSecAttrService as String: service,
      kSecAttrAccount as String: account,
    ]
    SecItemDelete(query as CFDictionary)
    var add = query
    add[kSecValueData as String] = data
    let status = SecItemAdd(add as CFDictionary, nil)
    guard status == errSecSuccess else {
      throw StravaTokenStoreError.keychain(status)
    }
  }

  public static func load() -> StravaTokens? {
    let query: [String: Any] = [
      kSecClass as String: kSecClassGenericPassword,
      kSecAttrService as String: service,
      kSecAttrAccount as String: account,
      kSecReturnData as String: true,
      kSecMatchLimit as String: kSecMatchLimitOne,
    ]
    var item: CFTypeRef?
    let status = SecItemCopyMatching(query as CFDictionary, &item)
    guard status == errSecSuccess, let data = item as? Data else { return nil }
    return try? JSONDecoder().decode(StravaTokens.self, from: data)
  }

  public static func clear() {
    let query: [String: Any] = [
      kSecClass as String: kSecClassGenericPassword,
      kSecAttrService as String: service,
      kSecAttrAccount as String: account,
    ]
    SecItemDelete(query as CFDictionary)
  }
}

public enum StravaTokenStoreError: Error {
  case keychain(OSStatus)
}
