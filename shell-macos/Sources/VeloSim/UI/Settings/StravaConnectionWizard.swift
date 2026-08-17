import SwiftUI
import VeloSimSupport

@MainActor
struct StravaConnectionWizard: View {
    @ObservedObject var model: VeloSimModel
    @Binding var isPresented: Bool

    @State private var step: ConnectionWizardStep = .intro
    @State private var testMessage: String = ""
    @State private var athleteName: String?
    @State private var isTesting = false

    var body: some View {
        ConnectionWizardChrome(step: $step, title: "Strava", onClose: { isPresented = false }) {
            switch step {
            case .intro:
                introStep
            case .action:
                connectStep
            case .test:
                testStep
            case .done:
                doneStep
            }
        }
        .onAppear { resetIfConnected() }
        .onChange(of: model.stravaAuth.status) { _, _ in
            if StravaTokenStore.load() != nil, step == .action {
                step = .test
            }
        }
    }

    private var introStep: some View {
        VStack(alignment: .leading, spacing: 10) {
            Label("Publish rides to Strava", systemImage: "figure.outdoor.cycle")
                .font(.subheadline.weight(.semibold))
            Text("VeloSim uploads FIT files after each ride. OAuth tokens stay in Keychain on this Mac.")
                .font(.caption)
                .foregroundStyle(.secondary)
            SettingsStatusBadge(label: statusLabel, kind: statusKind)
        }
    }

    private var connectStep: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Sign in with Strava in your browser. Return here when the callback completes.")
                .font(.caption)
                .foregroundStyle(.secondary)

            Button("Connect with Strava") {
                model.stravaAuth.beginAuth()
            }
            .buttonStyle(VeloGlassPrimaryButtonStyle())

            Text(model.stravaAuth.status)
                .font(.caption)
                .foregroundStyle(.secondary)
        }
    }

    private var testStep: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Verify your token by fetching your athlete profile.")
                .font(.caption)
                .foregroundStyle(.secondary)

            Button(isTesting ? "Testing…" : "Run test") {
                runTest()
            }
            .buttonStyle(VeloGlassPrimaryButtonStyle())
            .disabled(isTesting)

            if let athleteName {
                Label(athleteName, systemImage: "checkmark.circle.fill")
                    .font(.caption)
                    .foregroundStyle(.green)
            } else if !testMessage.isEmpty {
                Text(testMessage)
                    .font(.caption)
                    .foregroundStyle(.red)
            }

            if athleteName != nil {
                Button("Continue") { step = .done }
                    .buttonStyle(VeloGlassSecondaryButtonStyle())
            }
        }
    }

    private var doneStep: some View {
        VStack(alignment: .leading, spacing: 10) {
            Label("Connected", systemImage: "checkmark.seal.fill")
                .font(.title3.weight(.semibold))
                .foregroundStyle(.green)
            if let athleteName {
                Text("Signed in as \(athleteName)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Text("Rides can be published from the post-ride summary.")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
    }

    private var statusLabel: String {
        if StravaTokenStore.load() != nil { return "Connected" }
        return model.stravaAuth.status
    }

    private var statusKind: SettingsStatusKind {
        StravaTokenStore.load() != nil ? .ok : .missing
    }

    private func resetIfConnected() {
        if StravaTokenStore.load() != nil {
            step = .test
        }
    }

    private func runTest() {
        isTesting = true
        testMessage = ""
        athleteName = nil
        Task {
            let outcome = await StravaOAuth.validateStoredConnection()
            isTesting = false
            switch outcome {
            case let .success(athlete):
                athleteName = athlete.displayName
                model.stravaAuth.status = "connected"
            case let .failure(message):
                testMessage = message
            }
        }
    }
}
