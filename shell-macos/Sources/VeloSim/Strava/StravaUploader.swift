import Foundation

/// Strava `/uploads` multipart upload (shell HTTP — no live network in unit tests).
public enum StravaUploader {
  public static let uploadsURL = URL(string: "https://www.strava.com/api/v3/uploads")!

  public struct UploadResponse: Decodable, Equatable {
    public let id: Int64
    public let activityId: Int64?

    enum CodingKeys: String, CodingKey {
      case id
      case activityId = "activity_id"
    }
  }

  public static func upload(
    fitBytes: Data,
    screenshotPng: Data?,
    accessToken: String,
    session: URLSession = .shared
  ) async throws -> UploadResponse {
    let boundary = "VeloSim-\(UUID().uuidString)"
    var request = URLRequest(url: uploadsURL)
    request.httpMethod = "POST"
    request.setValue("Bearer \(accessToken)", forHTTPHeaderField: "Authorization")
    request.setValue(
      "multipart/form-data; boundary=\(boundary)",
      forHTTPHeaderField: "Content-Type"
    )
    request.httpBody = buildMultipart(
      boundary: boundary,
      fitBytes: fitBytes,
      screenshotPng: screenshotPng
    )
    let (data, response) = try await session.data(for: request)
    guard let http = response as? HTTPURLResponse, (200 ... 299).contains(http.statusCode) else {
      throw StravaUploadError.httpFailed
    }
    return try JSONDecoder().decode(UploadResponse.self, from: data)
  }

  public static func buildMultipart(
    boundary: String,
    fitBytes: Data,
    screenshotPng: Data?
  ) -> Data {
    var body = Data()
    func append(_ s: String) {
      body.append(Data(s.utf8))
    }

    append("--\(boundary)\r\n")
    append("Content-Disposition: form-data; name=\"data_type\"\r\n\r\n")
    append("fit\r\n")

    append("--\(boundary)\r\n")
    append("Content-Disposition: form-data; name=\"file\"; filename=\"ride.fit\"\r\n")
    append("Content-Type: application/octet-stream\r\n\r\n")
    body.append(fitBytes)
    append("\r\n")

    if let png = screenshotPng {
      append("--\(boundary)\r\n")
      append("Content-Disposition: form-data; name=\"photo\"; filename=\"screenshot.png\"\r\n")
      append("Content-Type: image/png\r\n\r\n")
      body.append(png)
      append("\r\n")
    }

    append("--\(boundary)--\r\n")
    return body
  }
}

public enum StravaUploadError: Error, Equatable {
  case httpFailed
  case notConfigured
}
