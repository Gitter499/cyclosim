import SwiftUI
import VeloSimSupport

// MARK: - FTP test picker (§6.3)

@MainActor
struct FTPTestPickerView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        NavigationStack {
            List(RampTestEngine.ProtocolKind.allCases) { kind in
                Button {
                    model.startFTPTest(kind)
                    model.showFTPTestPicker = false
                } label: {
                    VStack(alignment: .leading, spacing: 4) {
                        Text(kind.rawValue)
                            .font(.headline)
                        Text(kind.subtitle)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .navigationTitle("FTP Test")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { model.showFTPTestPicker = false }
                }
            }
        }
        .frame(minWidth: 440, minHeight: 320)
    }
}

@MainActor
struct FTPAnnouncementSheet: View {
    @ObservedObject var model: VeloSimModel
    let oldFTP: Int
    let newFTP: Int

    var body: some View {
        VStack(spacing: Tok.s4) {
            Text("New FTP set!")
                .font(.title.bold())
            Text("\(oldFTP) → \(newFTP) W")
                .font(Typo.metric())
                .monospacedDigit()
            Button("Done") { model.pendingFTPAnnouncement = nil }
                .buttonStyle(VeloGlassPrimaryButtonStyle())
        }
        .padding(Tok.s6)
        .frame(minWidth: 320)
    }
}
