import XCTest
@testable import VeloSimSupport

final class AppSettingsStoreTests: XCTestCase {
    func testPreferHostedBikeGenerationRoundTrip() {
        let prior = AppSettingsStore.preferHostedBikeGeneration
        defer { AppSettingsStore.preferHostedBikeGeneration = prior }
        AppSettingsStore.preferHostedBikeGeneration = true
        XCTAssertTrue(AppSettingsStore.preferHostedBikeGeneration)
    }
}
