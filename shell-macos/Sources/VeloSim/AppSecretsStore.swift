import Foundation
import Security
import VeloFFI

/// Keychain-backed API keys for tile and bikegen providers.
public enum AppSecretsStore {
    public enum Account: String, CaseIterable {
        case googleMapTilesApiKey
        case cesiumIonAccessToken
        case meshyApiKey
    }

    private static let service = "com.velosim.secrets"
    static let keychainService = service

    /// Injectable backend for unit tests (defaults to Security framework).
    public static var keychain: SecretsKeychainBacking = SystemSecretsKeychain()

    public static func save(_ value: String, account: Account) throws {
        let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
        if trimmed.isEmpty {
            keychain.delete(account: account.rawValue)
            return
        }
        try keychain.save(account: account.rawValue, value: trimmed)
    }

    public static func load(account: Account) -> String? {
        keychain.load(account: account.rawValue)?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .nilIfEmpty
    }

    public static func clear(account: Account) {
        keychain.delete(account: account.rawValue)
    }

    public static func runtimeSecretsDto(preferHostedBikeGeneration: Bool) -> RuntimeSecretsDto {
        RuntimeSecretsDto(
            googleMapTilesApiKey: load(account: .googleMapTilesApiKey),
            cesiumIonAccessToken: load(account: .cesiumIonAccessToken),
            meshyApiKey: load(account: .meshyApiKey),
            preferHostedBikeGeneration: preferHostedBikeGeneration
        )
    }
}

public protocol SecretsKeychainBacking {
    func save(account: String, value: String) throws
    func load(account: String) -> String?
    func delete(account: String)
}

public enum AppSecretsStoreError: Error {
    case keychain(OSStatus)
}

struct SystemSecretsKeychain: SecretsKeychainBacking {
    private let service = AppSecretsStore.keychainService

    func save(account: String, value: String) throws {
        let data = Data(value.utf8)
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
            throw AppSecretsStoreError.keychain(status)
        }
    }

    func load(account: String) -> String? {
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
        return String(data: data, encoding: .utf8)
    }

    func delete(account: String) {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
        ]
        SecItemDelete(query as CFDictionary)
    }
}

private extension String {
    var nilIfEmpty: String? {
        isEmpty ? nil : self
    }
}
