import SwiftUI
import VeloSimSupport

// MARK: - Workout library stub (§7.4)

@MainActor
struct WorkoutLibraryView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        VStack(alignment: .leading, spacing: Tok.s4) {
            Text("FTP Tests")
                .font(.headline)

            workoutRow(
                name: "2x20 Threshold",
                duration: "60 min",
                tss: "~65",
                blocks: [0.55, 0.75, 1.0, 0.55, 1.0, 0.55]
            ) {
                model.startSampleWorkout()
            }

            Text("Custom workouts")
                .font(.headline)
                .padding(.top, Tok.s2)

            WorkoutBuilderView(model: model)
        }
    }

    private func workoutRow(
        name: String,
        duration: String,
        tss: String,
        blocks: [Double],
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            HStack(spacing: Tok.s3) {
                IntervalGraphPreview(blocks: blocks)
                    .frame(width: 96, height: 32)

                VStack(alignment: .leading, spacing: 2) {
                    Text(name)
                        .font(.subheadline.weight(.semibold))
                        .foregroundStyle(.primary)
                    Text("\(duration) · TSS \(tss)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Image(systemName: "play.circle.fill")
                    .foregroundStyle(.orange)
            }
            .padding(Tok.s3)
            .background(.quaternary, in: RoundedRectangle(cornerRadius: Tok.rTile))
        }
        .buttonStyle(.plain)
    }
}

struct IntervalGraphPreview: View {
    let blocks: [Double]

    var body: some View {
        GeometryReader { geo in
            HStack(spacing: 1) {
                ForEach(Array(blocks.enumerated()), id: \.offset) { _, pct in
                    RoundedRectangle(cornerRadius: 2)
                        .fill(PowerZone.of(watts: Int(pct * 250), ftp: 250).color.opacity(0.85))
                        .frame(height: geo.size.height * CGFloat(min(1, pct)))
                        .frame(maxHeight: .infinity, alignment: .bottom)
                }
            }
        }
    }
}
