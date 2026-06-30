import AppKit
import Foundation
import VeloSimSupport

/// Handles Strava OAuth browser flow on macOS.
@MainActor
public final class StravaAuthCoordinator: ObservableObject {
  @Published public var status: String = "not connected"
  @Published public var pkce: StravaOAuth.PKCEChallenge?

  private let config: StravaConfig

  public init(config: StravaConfig = .load()) {
    self.config = config
    if StravaTokenStore.load() != nil {
      status = "connected"
    } else if config.isConfigured {
      status = "ready"
    } else {
      status = "not configured"
    }
  }

  public func beginAuth() {
    guard config.isConfigured else {
      status = "set STRAVA_CLIENT_ID / STRAVA_CLIENT_SECRET"
      return
    }
    let challenge = StravaOAuth.generatePKCE()
    pkce = challenge
    let url = StravaOAuth.authorizationURL(config: config, pkce: challenge)
    NSWorkspace.shared.open(url)
    status = "waiting for callback…"
  }

  public func handleCallback(url: URL, publisher: VeloActivityPublisher) async {
    guard let code = StravaOAuth.parseCallback(url: url),
          let verifier = pkce?.verifier
    else {
      status = "oauth callback failed"
      return
    }
    do {
      let tokens = try await StravaOAuth.exchangeCode(
        code: code,
        config: config,
        verifier: verifier
      )
      try publisher.setTokens(tokens)
      status = "connected"
      pkce = nil
    } catch {
      status = "token exchange failed"
    }
  }
}
