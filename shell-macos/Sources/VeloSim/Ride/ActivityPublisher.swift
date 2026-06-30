import Foundation
import VeloFFI

/// Implements UniFFI `ActivityPublisherCallback` — Strava upload or local-only marker.
/// Artifact persistence is handled by Rust (`finish_ride_and_publish` + `velo-rides`).
public final class VeloActivityPublisher: ActivityPublisherCallback, @unchecked Sendable {
  private let config: StravaConfig
  private var tokens: StravaTokens?
  private let session: URLSession

  public init(config: StravaConfig = .load(), session: URLSession = .shared) {
    self.config = config
    self.tokens = StravaTokenStore.load()
    self.session = session
  }

  public var isStravaConfigured: Bool { config.isConfigured }
  public var isAuthenticated: Bool { tokens != nil }

  public func setTokens(_ tokens: StravaTokens) throws {
    try StravaTokenStore.save(tokens)
    self.tokens = tokens
  }

  public func clearAuth() {
    StravaTokenStore.clear()
    tokens = nil
  }

  public func publishRide(
    fitBytes: Data,
    screenshotPng: Data?,
    summary: RideSummaryDto
  ) -> PublishResultDto {
    if config.isConfigured, let tokens = tokens {
      return publishToStrava(fitBytes: fitBytes, screenshotPng: screenshotPng, tokens: tokens, summary: summary)
    }
    return localOnlyMarker()
  }

  private func publishToStrava(
    fitBytes: Data,
    screenshotPng: Data?,
    tokens: StravaTokens,
    summary: RideSummaryDto
  ) -> PublishResultDto {
    let sem = DispatchSemaphore(value: 0)
    var result: PublishResultDto?
    Task {
      defer { sem.signal() }
      do {
        var active = tokens
        if Date().timeIntervalSince1970 >= active.expiresAt - 60 {
          active = try await StravaOAuth.refresh(tokens: active, config: config, session: session)
          try StravaTokenStore.save(active)
          self.tokens = active
        }
        let upload = try await StravaUploader.upload(
          fitBytes: fitBytes,
          screenshotPng: screenshotPng,
          accessToken: active.accessToken,
          session: session
        )
        let url = upload.activityId.map { "https://www.strava.com/activities/\($0)" }
          ?? "https://www.strava.com/uploads/\(upload.id)"
        result = PublishResultDto(activityUrl: url, savedLocally: false, rideId: "")
      } catch {
        result = PublishResultDto(
          activityUrl: "error:\(error.localizedDescription)",
          savedLocally: true,
          rideId: ""
        )
      }
    }
    sem.wait()
    return result!
  }

  private func localOnlyMarker() -> PublishResultDto {
    PublishResultDto(activityUrl: "", savedLocally: true, rideId: "")
  }
}

/// Implements UniFFI `MediaCaptureCallback` — RGBA → PNG in Swift.
public final class VeloMediaCapture: MediaCaptureCallback, @unchecked Sendable {
  public init() {}

  public func encodePngRgba(width: UInt32, height: UInt32, rgbaPixels: Data) -> Data {
    (try? PngEncoder.encode(
      width: Int(width),
      height: Int(height),
      rgba: rgbaPixels
    )) ?? Data()
  }
}
