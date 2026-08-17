import SwiftUI
import VeloFFI
import VeloSimSupport

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

@MainActor
struct SettingsView: View {
    @ObservedObject var model: VeloSimModel
    var embeddedInTab: Bool = false
    @Environment(\.dismiss) private var dismiss

    @State private var showStravaWizard = false
    @State private var showMusicWizard = false
    @State private var showIntegrationsWizard = false
    @State private var advancedExpanded = false

    var body: some View {
        NavigationStack {
            List {
                profileSection
                connectionsSection
                integrationsSection
                rideDefaultsSection
                advancedSection
            }
            .listStyle(.inset)
            .navigationTitle("Settings")
            .toolbar {
                if !embeddedInTab {
                    ToolbarItem(placement: .confirmationAction) {
                        Button("Done") { dismiss() }
                    }
                }
            }
        }
        .frame(minWidth: 440, minHeight: embeddedInTab ? 480 : 560)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(nsColor: .windowBackgroundColor))
        .popover(isPresented: $showStravaWizard) {
            StravaConnectionWizard(model: model, isPresented: $showStravaWizard)
        }
        .popover(isPresented: $showMusicWizard) {
            AppleMusicConnectionWizard(model: model, isPresented: $showMusicWizard)
        }
        .popover(isPresented: $showIntegrationsWizard) {
            IntegrationsKeysWizard(model: model, isPresented: $showIntegrationsWizard)
        }
    }

    private var profileSection: some View {
        Section("Profile") {
            TextField("Name", text: Binding(
                get: { model.riderName },
                set: {
                    model.riderName = $0
                    AppSettingsStore.riderName = $0
                }
            ))
            HStack {
                Text("Weight")
                Slider(value: Binding(
                    get: { model.riderWeightKg },
                    set: {
                        model.riderWeightKg = $0
                        AppSettingsStore.riderWeightKg = $0
                    }
                ), in: 45...120, step: 0.5)
                Text(String(format: "%.1f kg", model.riderWeightKg))
                    .monospacedDigit()
                    .frame(width: 64, alignment: .trailing)
            }
            HStack {
                Text("FTP")
                Slider(value: Binding(
                    get: { model.ftp },
                    set: { model.applyFtp($0) }
                ), in: 100...400, step: 5)
                Text("\(Int(model.ftp)) W")
                    .monospacedDigit()
                    .frame(width: 56, alignment: .trailing)
            }
        }
    }

    private var connectionsSection: some View {
        Section("Connections") {
            connectionRow(
                title: "Strava",
                systemImage: "figure.outdoor.cycle",
                status: stravaStatusLabel,
                kind: stravaStatusKind
            ) {
                showStravaWizard = true
            }

            connectionRow(
                title: "Apple Music",
                systemImage: "music.note",
                status: model.musicStatus,
                kind: model.musicDirector.authorized ? .ok : .missing
            ) {
                showMusicWizard = true
            }

            connectionRow(
                title: "Trainer",
                systemImage: "dot.radiowaves.left.and.right",
                status: bleBadgeLabel,
                kind: bleBadgeKind
            ) {
                model.shellDestination = .activities
            }
        }
    }

    private var integrationsSection: some View {
        Section("Integrations") {
            connectionRow(
                title: "3D Tiles & bikegen keys",
                systemImage: "key",
                status: model.tilesProviderStatus.isEmpty ? "Configure" : model.tilesProviderStatus,
                kind: SettingsApplyLogic.googleKeyConfigured() ? .ok : .neutral
            ) {
                showIntegrationsWizard = true
            }
        }
    }

    private var rideDefaultsSection: some View {
        Section("Ride defaults") {
            Toggle("Shift music at workout intervals", isOn: Binding(
                get: { model.segmentMusicEnabled },
                set: { model.setSegmentMusicEnabled($0) }
            ))

            Picker("Default steering", selection: Binding(
                get: { AppSettingsStore.defaultSteeringMode },
                set: { AppSettingsStore.defaultSteeringMode = $0 }
            )) {
                ForEach(SteeringInputMode.allCases) { mode in
                    Text(mode.label).tag(mode)
                }
            }

            Text("Current ride uses \(model.steeringMode.label).")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
    }

    private var advancedSection: some View {
        Section("Advanced") {
            DisclosureGroup("Developer tools", isExpanded: $advancedExpanded) {
                Toggle("Draw Rust HUD (velo-render)", isOn: Binding(
                    get: { model.rustHudDrawEnabled },
                    set: { model.setRustHudDrawEnabled($0) }
                ))

                LabeledContent("Toggle count") {
                    Text("\(model.toggleCount)")
                        .monospacedDigit()
                        .foregroundStyle(.secondary)
                }

                if model.logs.isEmpty {
                    Text("No log lines yet")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                } else {
                    ForEach(Array(model.logs.suffix(12).enumerated()), id: \.offset) { _, line in
                        Text(line)
                            .font(.caption.monospaced())
                            .lineLimit(2)
                    }
                }
            }
        }
    }

    private func connectionRow(
        title: String,
        systemImage: String,
        status: String,
        kind: SettingsStatusKind,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            HStack {
                Label(title, systemImage: systemImage)
                Spacer()
                SettingsStatusBadge(label: status, kind: kind)
                Image(systemName: "chevron.right")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.tertiary)
            }
        }
        .buttonStyle(.plain)
    }

    private var stravaStatusLabel: String {
        if StravaTokenStore.load() != nil { return "Connected" }
        return model.stravaAuth.status
    }

    private var stravaStatusKind: SettingsStatusKind {
        StravaTokenStore.load() != nil ? .ok : .missing
    }

    private var bleBadgeLabel: String {
        switch model.sensorMode {
        case .bluetooth:
            return model.bleState
        case .fake, .replay:
            return model.sensorMode.label
        }
    }

    private var bleBadgeKind: SettingsStatusKind {
        guard model.sensorMode == .bluetooth else { return .neutral }
        if model.bleControlError != nil { return .warning }
        if model.bleState.lowercased().contains("connect") { return .ok }
        return .neutral
    }
}

struct SettingsView_Previews: PreviewProvider {
    static var previews: some View {
        SettingsView(model: VeloSimModel())
    }
}
