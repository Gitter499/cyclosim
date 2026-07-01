import Foundation
import MusicKit
import VeloFFI

public final class NoopAudioDirector: AudioDirectorCallback, @unchecked Sendable {
    public init() {}
    public func onSegment(energy: SegmentEnergyDto, intent: PlaybackIntentDto) {}
}

@MainActor
public final class VeloMusicDirector: AudioDirectorCallback, @unchecked Sendable {
    public var onStatusChange: ((String) -> Void)?
    public private(set) var status: String = "Music off"
    public private(set) var authorized = false
    public private(set) var canPlayCatalog = false
    public private(set) var lastAction: String = ""
    public private(set) var nowPlayingTitle: String?
    private var enabled = false

    public init() {}

    public func setEnabled(_ on: Bool) {
        enabled = on
        status = on ? readyStatus() : "Music off"
        if !on { nowPlayingTitle = nil }
        publishStatus()
    }

    public func refreshAuthorizationStatus() async {
        let auth = MusicAuthorization.currentStatus
        authorized = auth == .authorized
        guard authorized else {
            canPlayCatalog = false
            status = authorizationStatusMessage(auth)
            publishStatus()
            return
        }
        do {
            let subscription = try await MusicSubscription.current
            canPlayCatalog = subscription.canPlayCatalogContent
            status = canPlayCatalog ? readyStatus() : "Apple Music subscription required"
        } catch {
            canPlayCatalog = true
            status = readyStatus()
        }
        publishStatus()
    }

    public func requestAuthorization() async {
        await refreshAuthorizationStatus()
        guard !authorized else { return }
        authorized = await MusicAuthorization.request() == .authorized
        await refreshAuthorizationStatus()
    }

    nonisolated public func onSegment(energy: SegmentEnergyDto, intent: PlaybackIntentDto) {
        Task { @MainActor in await handleSegment(energy: energy, intent: intent) }
    }

    private func handleSegment(energy: SegmentEnergyDto, intent: PlaybackIntentDto) async {
        guard enabled else {
            status = "Skipped \(SegmentMusicSearch.energyLabel(for: energy)) — music off"
            publishStatus(); return
        }
        guard authorized else {
            status = "Skipped \(SegmentMusicSearch.energyLabel(for: energy)) — connect Apple Music"
            publishStatus(); return
        }
        guard canPlayCatalog else {
            status = "Skipped \(SegmentMusicSearch.energyLabel(for: energy)) — subscription required"
            publishStatus(); return
        }

        let term = SegmentMusicSearch.searchTerm(for: energy)
        let label = SegmentMusicSearch.energyLabel(for: energy)
        lastAction = intent == .start ? "Started \(label)" : "Switched to \(label)"
        status = "\(lastAction) — searching \"\(term)\""
        publishStatus()

        let player = ApplicationMusicPlayer.shared
        do {
            if intent == .transition { player.stop() }
            if let playlist = try await searchPlaylist(term: term) {
                player.queue = ApplicationMusicPlayer.Queue(for: [playlist])
                try await player.play()
                status = "Now playing: \(playlist.name)"
                publishStatus(); return
            }
            if let song = try await searchSong(term: term) {
                player.queue = ApplicationMusicPlayer.Queue(for: [song])
                try await player.play()
                status = "Now playing: \(song.title)"
                publishStatus(); return
            }
            status = "No catalog match for \"\(term)\""
            publishStatus()
        } catch {
            status = "Music error: \(error.localizedDescription)"
            publishStatus()
        }
    }

    private func searchPlaylist(term: String) async throws -> Playlist? {
        var request = MusicCatalogSearchRequest(term: term, types: [Playlist.self])
        request.limit = 5
        return try await request.response().playlists.first
    }

    private func searchSong(term: String) async throws -> Song? {
        var request = MusicCatalogSearchRequest(term: term, types: [Song.self])
        request.limit = 5
        return try await request.response().songs.first
    }

    /// Lightweight catalog probe for Settings connection wizard.
    public enum CatalogTestOutcome: Equatable {
        case success(count: Int, firstTitle: String?)
        case failure(String)
    }

    public func testCatalogSearch(term: String = "cycling warmup") async -> CatalogTestOutcome {
        guard authorized else {
            return .failure("Apple Music not authorized.")
        }
        do {
            var request = MusicCatalogSearchRequest(term: term, types: [Song.self])
            request.limit = 5
            let response = try await request.response()
            let songs = response.songs
            let first = songs.first?.title
            return .success(count: songs.count, firstTitle: first)
        } catch {
            return .failure(error.localizedDescription)
        }
    }

    private func readyStatus() -> String {
        enabled ? "Ready — segment music on" : "Apple Music authorized"
    }

    private func authorizationStatusMessage(_ auth: MusicAuthorization.Status) -> String {
        switch auth {
        case .authorized: return readyStatus()
        case .denied: return "Apple Music denied — enable in System Settings"
        case .restricted: return "Apple Music restricted on this device"
        case .notDetermined: return "Apple Music not connected"
        @unknown default: return "Apple Music unavailable"
        }
    }

    private func publishStatus() { onStatusChange?(status) }
}
