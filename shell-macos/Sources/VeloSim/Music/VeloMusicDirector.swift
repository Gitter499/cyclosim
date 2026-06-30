import Foundation
import MusicKit
import VeloFFI

/// No-op segment music for tests and when MusicKit is unavailable.
public final class NoopAudioDirector: AudioDirectorCallback, @unchecked Sendable {
    public init() {}

    public func onSegment(energy: SegmentEnergyDto, intent: PlaybackIntentDto) {}
}

/// Maps workout segment energy to Apple Music playback (control only — no raw PCM).
@MainActor
public final class VeloMusicDirector: AudioDirectorCallback, @unchecked Sendable {
    public private(set) var status: String = "Music off"
    public private(set) var authorized = false

    private var enabled = false

    public init() {}

    public func setEnabled(_ on: Bool) {
        enabled = on
        status = on ? "Segment music enabled" : "Music off"
    }

    /// Request Apple Music authorization (minimal setup flow).
    public func requestAuthorization() async {
        let current = await MusicAuthorization.currentStatus
        if current == .authorized {
            authorized = true
            status = "Apple Music authorized"
            return
        }
        let result = await MusicAuthorization.request()
        authorized = result == .authorized
        status = authorized ? "Apple Music authorized" : "Apple Music denied"
    }

    public func onSegment(energy: SegmentEnergyDto, intent: PlaybackIntentDto) {
        guard enabled, authorized else { return }
        Task { @MainActor in
            await applySegment(energy: energy, intent: intent)
        }
    }

    private func applySegment(energy: SegmentEnergyDto, intent: PlaybackIntentDto) async {
        let term = searchTerm(for: energy)
        status = "Playing: \(term)"

        do {
            var request = MusicCatalogSearchRequest(term: term, types: [Playlist.self])
            request.limit = 1
            let response = try await request.response()
            if let playlist = response.playlists.first {
                let queue = ApplicationMusicPlayer.Queue(for: [playlist])
                ApplicationMusicPlayer.shared.queue = queue
                if intent == .start || intent == .transition {
                    try await ApplicationMusicPlayer.shared.play()
                }
                return
            }

            var songRequest = MusicCatalogSearchRequest(term: term, types: [Song.self])
            songRequest.limit = 1
            let songResponse = try await songRequest.response()
            if let song = songResponse.songs.first {
                ApplicationMusicPlayer.shared.queue = ApplicationMusicPlayer.Queue(for: [song])
                try await ApplicationMusicPlayer.shared.play()
            }
        } catch {
            status = "Music error: \(error.localizedDescription)"
        }
    }

    private func searchTerm(for energy: SegmentEnergyDto) -> String {
        switch energy {
        case .warmup: return "warm up cycling"
        case .build: return "workout build"
        case .threshold: return "high energy cycling"
        case .recovery: return "recovery chill"
        case .cooldown: return "cool down ambient"
        }
    }
}
