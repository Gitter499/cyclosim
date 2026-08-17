import SwiftUI
import VeloSimSupport

@MainActor
struct AppleMusicConnectionWizard: View {
    @ObservedObject var model: VeloSimModel
    @Binding var isPresented: Bool

    @State private var step: ConnectionWizardStep = .intro
    @State private var testMessage: String = ""
    @State private var searchSummary: String?
    @State private var isTesting = false

    var body: some View {
        ConnectionWizardChrome(step: $step, title: "Apple Music", onClose: { isPresented = false }) {
            switch step {
            case .intro:
                introStep
            case .action:
                authorizeStep
            case .test:
                testStep
            case .done:
                doneStep
            }
        }
        .onAppear { syncStepFromAuth() }
    }

    private var introStep: some View {
        VStack(alignment: .leading, spacing: 10) {
            Label("Workout interval music", systemImage: "music.note.list")
                .font(.subheadline.weight(.semibold))
            Text("MusicKit queues catalog searches at workout segment boundaries. Playback control only — not BPM-locked.")
                .font(.caption)
                .foregroundStyle(.secondary)
            SettingsStatusBadge(label: model.musicStatus, kind: musicStatusKind)
        }
    }

    private var authorizeStep: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Authorize VeloSim to access your Apple Music catalog.")
                .font(.caption)
                .foregroundStyle(.secondary)

            Button("Authorize Apple Music") {
                model.connectAppleMusic()
                Task {
                    try? await Task.sleep(nanoseconds: 500_000_000)
                    await model.musicDirector.refreshAuthorizationStatus()
                    model.musicStatus = model.musicDirector.status
                    if model.musicDirector.authorized {
                        step = .test
                    }
                }
            }
            .buttonStyle(VeloGlassPrimaryButtonStyle())

            Text(model.musicStatus)
                .font(.caption)
                .foregroundStyle(.secondary)
                .lineLimit(3)
        }
    }

    private var testStep: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Search the catalog for \"cycling warmup\" to confirm access.")
                .font(.caption)
                .foregroundStyle(.secondary)

            Button(isTesting ? "Searching…" : "Run test") {
                runTest()
            }
            .buttonStyle(VeloGlassPrimaryButtonStyle())
            .disabled(isTesting)

            if let searchSummary {
                Label(searchSummary, systemImage: "checkmark.circle.fill")
                    .font(.caption)
                    .foregroundStyle(.green)
            } else if !testMessage.isEmpty {
                Text(testMessage)
                    .font(.caption)
                    .foregroundStyle(.red)
            }

            if searchSummary != nil {
                Button("Continue") { step = .done }
                    .buttonStyle(VeloGlassSecondaryButtonStyle())
            }
        }
    }

    private var doneStep: some View {
        VStack(alignment: .leading, spacing: 10) {
            Label("Ready", systemImage: "checkmark.seal.fill")
                .font(.title3.weight(.semibold))
                .foregroundStyle(.green)
            if let searchSummary {
                Text(searchSummary)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Toggle("Enable segment music", isOn: Binding(
                get: { model.segmentMusicEnabled },
                set: { model.setSegmentMusicEnabled($0) }
            ))
        }
    }

    private var musicStatusKind: SettingsStatusKind {
        model.musicDirector.authorized ? .ok : .missing
    }

    private func syncStepFromAuth() {
        if model.musicDirector.authorized {
            step = .test
        }
    }

    private func runTest() {
        isTesting = true
        testMessage = ""
        searchSummary = nil
        Task {
            await model.musicDirector.refreshAuthorizationStatus()
            let outcome = await model.musicDirector.testCatalogSearch()
            isTesting = false
            switch outcome {
            case let .success(count, first):
                if count > 0, let first {
                    searchSummary = "Found \(count) tracks — e.g. \"\(first)\""
                } else if count > 0 {
                    searchSummary = "Found \(count) tracks"
                } else {
                    testMessage = "No catalog matches — check subscription."
                }
            case let .failure(message):
                testMessage = message
            }
        }
    }
}
