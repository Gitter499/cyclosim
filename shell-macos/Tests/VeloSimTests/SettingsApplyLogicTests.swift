import XCTest
@testable import VeloSimSupport

final class SettingsApplyLogicTests: XCTestCase {
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

    func testLoadFormStateReadsKeychainAndPreferences() throws {
        try AppSecretsStore.save("google-1", account: .googleMapTilesApiKey)
        try AppSecretsStore.save("meshy-1", account: .meshyApiKey)
        let prior = AppSettingsStore.preferHostedBikeGeneration
        defer { AppSettingsStore.preferHostedBikeGeneration = prior }
        AppSettingsStore.preferHostedBikeGeneration = true

        let form = SettingsApplyLogic.loadFormState()
        XCTAssertEqual(form.googleKey, "google-1")
        XCTAssertEqual(form.meshyKey, "meshy-1")
        XCTAssertTrue(form.preferHostedBikegen)
    }

    func testApplyPersistsSecretsAndReturnsWarningWhenMeshyMissing() {
        let prior = AppSettingsStore.preferHostedBikeGeneration
        defer { AppSettingsStore.preferHostedBikeGeneration = prior }

        let outcome = SettingsApplyLogic.apply(
            SettingsApplyLogic.FormState(
                googleKey: "g",
                cesiumToken: "c",
                meshyKey: "",
                preferHostedBikegen: true
            ),
            tilesProviderStatus: "Google Photorealistic"
        )

        guard case let .success(message, warning) = outcome else {
            return XCTFail("expected success")
        }
        XCTAssertTrue(message.contains("Google Photorealistic"))
        XCTAssertEqual(warning, "Hosted bike generation needs a Meshy API key.")
        XCTAssertEqual(AppSecretsStore.load(account: .googleMapTilesApiKey), "g")
        XCTAssertEqual(AppSecretsStore.load(account: .cesiumIonAccessToken), "c")
        XCTAssertTrue(AppSettingsStore.preferHostedBikeGeneration)
    }

    func testApplyClearsEmptyKeys() {
        try? AppSecretsStore.save("old", account: .googleMapTilesApiKey)

        let outcome = SettingsApplyLogic.apply(
            SettingsApplyLogic.FormState(googleKey: "   "),
            tilesProviderStatus: "ion dev"
        )

        guard case .success = outcome else {
            return XCTFail("expected success")
        }
        XCTAssertNil(AppSecretsStore.load(account: .googleMapTilesApiKey))
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
