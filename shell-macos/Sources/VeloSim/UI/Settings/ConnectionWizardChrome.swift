import SwiftUI
import VeloSimSupport

/// Shared step indicator and navigation for Settings connection wizards.
@MainActor
struct ConnectionWizardChrome<Content: View>: View {
    @Binding var step: ConnectionWizardStep
    let title: String
    let onClose: () -> Void
    @ViewBuilder var content: () -> Content

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                Text(title)
                    .font(.headline)
                Spacer()
                Button("Close", action: onClose)
                    .buttonStyle(.plain)
                    .foregroundStyle(.secondary)
            }

            stepIndicator

            content()
                .frame(maxWidth: .infinity, alignment: .leading)

            wizardNavigation
        }
        .padding(20)
        .frame(width: 420)
    }

    private var stepIndicator: some View {
        HStack(spacing: 6) {
            ForEach(ConnectionWizardStep.allCases, id: \.rawValue) { candidate in
                Capsule()
                    .fill(candidate.rawValue <= step.rawValue ? Color.accentColor : Color.secondary.opacity(0.25))
                    .frame(height: 4)
            }
        }
        .accessibilityElement(children: .ignore)
        .accessibilityLabel("Step \(step.rawValue + 1) of \(ConnectionWizardStep.allCases.count)")
    }

    private var wizardNavigation: some View {
        HStack {
            if step != .intro {
                Button("Back") {
                    step = ConnectionWizardStep.retreat(from: step)
                }
            }

            Spacer()

            if step == .done {
                Button("Done") { onClose() }
                    .buttonStyle(VeloGlassPrimaryButtonStyle())
            } else if step != .test {
                Button(step == .intro ? "Continue" : "Next") {
                    step = ConnectionWizardStep.advance(from: step)
                }
                .buttonStyle(VeloGlassPrimaryButtonStyle())
            }
        }
    }
}

struct ConnectionWizardChrome_Previews: PreviewProvider {
    struct PreviewHost: View {
        @State private var step: ConnectionWizardStep = .intro

        var body: some View {
            ConnectionWizardChrome(step: $step, title: "Strava", onClose: {}) {
                Text("Step: \(step.title)")
            }
        }
    }

    static var previews: some View {
        PreviewHost()
    }
}
