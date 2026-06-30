import XCTest
import VeloSimSupport

final class StravaUploadTests: XCTestCase {

    override func tearDown() {
        MockURLProtocol.requestHandler = nil
        super.tearDown()
    }

    func testMultipartContainsFitAndPhoto() {
        let fit = Data("FITDATA".utf8)
        let png = Data([0x89, 0x50, 0x4E, 0x47])
        let body = StravaUploader.buildMultipart(boundary: "test", fitBytes: fit, screenshotPng: png)
        let text = String(decoding: body, as: UTF8.self)
        XCTAssertTrue(text.contains("name=\"data_type\""))
        XCTAssertTrue(text.contains("fit"))
        XCTAssertTrue(text.contains("filename=\"ride.fit\""))
        XCTAssertTrue(text.contains("filename=\"screenshot.png\""))
        XCTAssertTrue(text.contains("image/png"))
        XCTAssertTrue(body.contains(fit))
        XCTAssertTrue(body.contains(png))
    }

    func testUploadPostsBearerAndParsesResponse() async throws {
        let fixture = #"{"id":42,"activity_id":1001}"#
        MockURLProtocol.requestHandler = { request in
            XCTAssertEqual(request.httpMethod, "POST")
            XCTAssertEqual(request.value(forHTTPHeaderField: "Authorization"), "Bearer test-token")
            XCTAssertTrue(request.value(forHTTPHeaderField: "Content-Type")?.contains("multipart/form-data") == true)
            let response = HTTPURLResponse(url: request.url!, statusCode: 201, httpVersion: nil, headerFields: nil)!
            return (response, Data(fixture.utf8))
        }
        let session = MockURLProtocol.makeSession()
        let result = try await StravaUploader.upload(
            fitBytes: Data([0x0E, 0x10]),
            screenshotPng: nil,
            accessToken: "test-token",
            session: session
        )
        XCTAssertEqual(result.id, 42)
        XCTAssertEqual(result.activityId, 1001)
    }

    func testTokenRefreshOnPublish() async throws {
        let refreshFixture = """
        {"access_token":"new_at","refresh_token":"new_rt","expires_at":9999999999}
        """
        let uploadFixture = #"{"id":1,"activity_id":55}"#
        var call = 0
        MockURLProtocol.requestHandler = { request in
            call += 1
            if request.url?.absoluteString.contains("oauth/token") == true {
                let response = HTTPURLResponse(url: request.url!, statusCode: 200, httpVersion: nil, headerFields: nil)!
                return (response, Data(refreshFixture.utf8))
            }
            let response = HTTPURLResponse(url: request.url!, statusCode: 201, httpVersion: nil, headerFields: nil)!
            return (response, Data(uploadFixture.utf8))
        }
        let session = MockURLProtocol.makeSession()
        let config = StravaConfig(clientId: "id", clientSecret: "sec", redirectURI: "velosim://oauth")
        let expired = StravaTokens(accessToken: "old", refreshToken: "rt", expiresAt: 1)
        let refreshed = try await StravaOAuth.refresh(tokens: expired, config: config, session: session)
        XCTAssertEqual(refreshed.accessToken, "new_at")
        XCTAssertGreaterThanOrEqual(call, 1)
    }
}
