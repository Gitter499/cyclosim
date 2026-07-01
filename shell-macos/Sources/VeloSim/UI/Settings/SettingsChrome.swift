import SwiftUI

enum SettingsStatusKind {
    case ok
    case warning
    case missing
    case neutral
}

struct SettingsStatusBadge: View {
    let label: String
    let kind: SettingsStatusKind

    var body: some View {
        Text(label)
            .font(.caption2.weight(.medium))
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(tint.opacity(0.22))
            .foregroundStyle(tint)
            .clipShape(Capsule())
    }

    private var tint: Color {
        switch kind {
        case .ok: return .green
        case .warning: return .orange
        case .missing: return .secondary
        case .neutral: return .blue
        }
    }
}

struct SettingsBlockedView<Content: View>: View {
    let isBlocked: Bool
    @ViewBuilder var content: () -> Content

    var body: some View {
        if isBlocked {
            VStack(spacing: 14) {
                Image(systemName: "lock.fill")
                    .font(.largeTitle)
                    .foregroundStyle(.secondary)
                Text("Settings unavailable during a ride")
                    .font(.headline)
                Text("Stop your ride to change API keys, trainer setup, or other preferences.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
                    .frame(maxWidth: 320)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .padding(24)
        } else {
            content()
        }
    }
}
