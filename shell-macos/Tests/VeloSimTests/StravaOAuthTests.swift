import XCTest
import VeloSimSupport

final class StravaOAuthTests: XCTestCase {

    func testPKCEChallengeIsS256Compatible() {
        let pkce = StravaOAuth.generatePKCE()
        XCTAssertFalse(pkce.verifier.isEmpty)
        XCTAssertFalse(pkce.challenge.isEmpty)
        XCTAssertNotEqual(pkce.verifier, pkce.challenge)
        XCTAssertFalse(pkce.challenge.contains("+"))
        XCTAssertFalse(pkce.challenge.contains("/"))
        XCTAssertFalse(pkce.challenge.contains("="))
    }

    func testAuthorizationURLContainsPKCEParams() {
        let config = StravaConfig(clientId: "12345", clientSecret: "secret", redirectURI: "velosim://oauth")
        let pkce = StravaOAuth.PKCEChallenge(verifier: "verifier", challenge: "challenge")
        let url = StravaOAuth.authorizationURL(config: config, pkce: pkce)
        let items = URLComponents(url: url, resolvingAgainstBaseURL: false)!.queryItems!
        XCTAssertEqual(items.first { $0.name == "code_challenge" }?.value, "challenge")
        XCTAssertEqual(items.first { $0.name == "code_challenge_method" }?.value, "S256")
        XCTAssertEqual(items.first { $0.name == "client_id" }?.value, "12345")
    }

    func testParseCallbackExtractsCode() {
        let url = URL(string: "velosim://oauth?code=abc123&scope=activity:write")!
        XCTAssertEqual(StravaOAuth.parseCallback(url: url), "abc123")
    }

    func testParseCallbackRejectsMissingCode() {
        let url = URL(string: "velosim://oauth?error=access_denied")!
        XCTAssertNil(StravaOAuth.parseCallback(url: url))
    }

    func testTokenExchangeParsesFixtureJSON() async throws {
        let fixture = """
        {"access_token":"at","refresh_token":"rt","expires_at":9999999999,"token_type":"Bearer"}
        """
        MockURLProtocol.requestHandler = { request in
            XCTAssertEqual(request.url?.absoluteString, StravaOAuth.tokenURL.absoluteString)
            let response = HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!
            return (response, Data(fixture.utf8))
        }
        let session = MockURLProtocol.makeSession()
        let config = StravaConfig(clientId: "id", clientSecret: "sec", redirectURI: "velosim://oauth")
        let tokens = try await StravaOAuth.exchangeCode(code: "code", config: config, verifier: "v", session: session)
        XCTAssertEqual(tokens.accessToken, "at")
        XCTAssertEqual(tokens.refreshToken, "rt")
        XCTAssertEqual(tokens.expiresAt, 9_999_999_999)
    }
}
