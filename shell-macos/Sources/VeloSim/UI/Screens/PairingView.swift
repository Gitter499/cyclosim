import SwiftUI
import VeloSimSupport

// MARK: - Pairing (§7.2)

@MainActor
struct PairingView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        NavigationStack {
            List {
                pairRow(role: "Power / Trainer", connected: model.sensorMode == .bluetooth && model.bleState.contains("connected"))
                pairRow(role: "Cadence", connected: false)
                pairRow(role: "Heart Rate", connected: false)
            }
            .navigationTitle("Pair devices")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Close") { model.showPairingSheet = false }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Ride") {
                        model.showPairingSheet = false
                        model.beginJustRide()
                    }
                    .disabled(model.sensorMode != .bluetooth)
                }
            }
        }
        .frame(minWidth: 420, minHeight: 360)
    }

    private func pairRow(role: String, connected: Bool) -> some View {
        HStack {
            Label(role, systemImage: connected ? "checkmark.circle.fill" : "dot.radiowaves.left.and.right")
                .foregroundStyle(connected ? .green : .secondary)
            Spacer()
            Text(connected ? "Connected" : "Searching…")
                .foregroundStyle(.secondary)
                .monospacedDigit()
            Button(connected ? "Change" : "Connect") {
                model.setSensorMode(.bluetooth)
            }
            .buttonStyle(VeloGlassSecondaryButtonStyle())
            .controlSize(.small)
        }
        .padding(.vertical, Tok.s1)
    }
}
