import XCTest
@testable import VeloSimSupport

final class AppSecretsStoreTests: XCTestCase {
    private var memory = MemorySecretsKeychain()

    override func setUp() {
        super.setUp()
        memory = MemorySecretsKeychain()
        AppSecretsStore.keychain = memory
    }

    override func tearDown() {
        AppSecretsStore.keychain = SystemSecretsKeychain()
        super.tearDown()
    }

    func testSaveLoadRoundTrip() throws {
        try AppSecretsStore.save("google-secret", account: .googleMapTilesApiKey)
        XCTAssertEqual(AppSecretsStore.load(account: .googleMapTilesApiKey), "google-secret")
    }

    func testEmptySaveClearsKey() throws {
        try AppSecretsStore.save("temp", account: .meshyApiKey)
        try AppSecretsStore.save("   ", account: .meshyApiKey)
        XCTAssertNil(AppSecretsStore.load(account: .meshyApiKey))
    }

    func testRuntimeSecretsDtoAssembly() throws {
        try AppSecretsStore.save("g", account: .googleMapTilesApiKey)
        try AppSecretsStore.save("m", account: .meshyApiKey)
        let dto = AppSecretsStore.runtimeSecretsDto(preferHostedBikeGeneration: true)
        XCTAssertEqual(dto.googleMapTilesApiKey, "g")
        XCTAssertEqual(dto.meshyApiKey, "m")
        XCTAssertTrue(dto.preferHostedBikeGeneration)
    }
}

private final class MemorySecretsKeychain: SecretsKeychainBacking {
    private var values: [String: String] = [:]

    func save(account: String, value: String) throws {
        values[account] = value
    }

    func load(account: String) -> String? {
        values[account]
    }

    func delete(account: String) {
        values.removeValue(forKey: account)
    }
}
