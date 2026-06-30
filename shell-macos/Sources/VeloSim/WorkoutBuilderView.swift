import SwiftUI
import VeloFFI

enum WorkoutTargetKind: String, CaseIterable, Identifiable {
    case ergWatts = "ERG (W)"
    case ftpPercent = "FTP %"
    case freeRide = "Free ride"

    var id: String { rawValue }
}

struct WorkoutBuilderInterval: Identifiable, Equatable {
    let id = UUID()
    var name: String
    var durationMinutes: Int
    var durationSeconds: Int
    var targetKind: WorkoutTargetKind
    var ergWatts: Double
    var ftpPercent: Double

    var durationS: Double {
        Double(durationMinutes * 60 + durationSeconds)
    }

    static func warmupDefault() -> Self {
        Self(
            name: "Warmup",
            durationMinutes: 10,
            durationSeconds: 0,
            targetKind: .ftpPercent,
            ergWatts: 150,
            ftpPercent: 55
        )
    }

    func toDto() -> WorkoutIntervalDto {
        let target: WorkoutTargetDto
        switch targetKind {
        case .ergWatts:
            target = .ergWatts(watts: ergWatts)
        case .ftpPercent:
            target = .ftpPercent(percent: ftpPercent)
        case .freeRide:
            target = .freeRide
        }
        return WorkoutIntervalDto(name: name, durationS: durationS, target: target)
    }
}

struct WorkoutBuilderView: View {
    @ObservedObject var model: VeloSimModel
    @State private var workoutName = "Custom Workout"
    @State private var intervals: [WorkoutBuilderInterval] = [
        .warmupDefault(),
        WorkoutBuilderInterval(
            name: "Interval",
            durationMinutes: 5,
            durationSeconds: 0,
            targetKind: .ftpPercent,
            ergWatts: 200,
            ftpPercent: 95
        ),
        WorkoutBuilderInterval(
            name: "Recovery",
            durationMinutes: 5,
            durationSeconds: 0,
            targetKind: .ftpPercent,
            ergWatts: 150,
            ftpPercent: 50
        ),
    ]
    @State private var builderError: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            TextField("Workout name", text: $workoutName)
                .textFieldStyle(.roundedBorder)

            ForEach(Array(intervals.enumerated()), id: \.element.id) { index, _ in
                intervalRow(index: index)
            }

            HStack {
                Button("Add interval") {
                    intervals.append(.warmupDefault())
                }
                Spacer()
            }

            if let builderError {
                Text(builderError)
                    .font(.caption)
                    .foregroundStyle(.red)
            }

            HStack {
                Button("Start workout") {
                    startCustomWorkout()
                }
                .disabled(model.workoutLive.active || intervals.isEmpty)

                Button("Start 2×20 Threshold") {
                    model.startSampleWorkout()
                }
                .disabled(model.workoutLive.active)

                if model.workoutLive.active {
                    Button("Clear workout") {
                        model.clearWorkout()
                    }
                }
            }

            liveWorkoutPanel
        }
    }

    @ViewBuilder
    private func intervalRow(index: Int) -> some View {
        let binding = Binding(
            get: { intervals[index] },
            set: { intervals[index] = $0 }
        )

        VStack(alignment: .leading, spacing: 6) {
            HStack {
                TextField("Name", text: binding.name)
                    .textFieldStyle(.roundedBorder)
                Button("Remove", role: .destructive) {
                    intervals.remove(at: index)
                }
                .disabled(intervals.count <= 1)
            }

            HStack(spacing: 8) {
                Stepper("Min: \(binding.wrappedValue.durationMinutes)", value: binding.durationMinutes, in: 0...180)
                Stepper("Sec: \(binding.wrappedValue.durationSeconds)", value: binding.durationSeconds, in: 0...59)
            }
            .font(.caption)

            Picker("Target", selection: binding.targetKind) {
                ForEach(WorkoutTargetKind.allCases) { kind in
                    Text(kind.rawValue).tag(kind)
                }
            }
            .pickerStyle(.segmented)

            switch binding.wrappedValue.targetKind {
            case .ergWatts:
                HStack {
                    Text("Watts")
                    Slider(value: binding.ergWatts, in: 50...500, step: 5)
                    Text("\(Int(binding.wrappedValue.ergWatts)) W")
                        .monospacedDigit()
                        .frame(width: 56, alignment: .trailing)
                }
            case .ftpPercent:
                HStack {
                    Text("FTP %")
                    Slider(value: binding.ftpPercent, in: 30...150, step: 1)
                    Text("\(Int(binding.wrappedValue.ftpPercent))%")
                        .monospacedDigit()
                        .frame(width: 56, alignment: .trailing)
                }
            case .freeRide:
                Text("Trainer follows SIM grade")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            HStack {
                Button("Move up") {
                    guard index > 0 else { return }
                    intervals.swapAt(index, index - 1)
                }
                .disabled(index == 0)
                Button("Move down") {
                    guard index < intervals.count - 1 else { return }
                    intervals.swapAt(index, index + 1)
                }
                .disabled(index >= intervals.count - 1)
            }
            .font(.caption)
        }
        .padding(8)
        .background(.quaternary.opacity(0.35), in: RoundedRectangle(cornerRadius: 8))
    }

    @ViewBuilder
    private var liveWorkoutPanel: some View {
        Text(model.workoutStatus)
            .font(.caption)
            .foregroundStyle(.secondary)

        if model.workoutLive.active {
            VStack(alignment: .leading, spacing: 4) {
                Text(model.workoutLive.intervalName)
                    .font(.headline)
                if let target = model.workoutLive.targetWatts {
                    Text("Target: \(Int(target)) W")
                } else {
                    Text("Target: Free ride")
                        .foregroundStyle(.secondary)
                }
                Text(String(
                    format: "Interval: %.0f s · Workout: %.0f s",
                    model.workoutLive.intervalElapsedS,
                    model.workoutLive.workoutElapsedS
                ))
                .font(.caption)
                .monospacedDigit()
            }
        }
    }

    private func startCustomWorkout() {
        builderError = nil
        let dto = WorkoutDto(
            name: workoutName.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                ? "Custom Workout"
                : workoutName,
            intervals: intervals.map { $0.toDto() }
        )
        do {
            try model.startCustomWorkout(dto)
        } catch let error as VeloError {
            switch error {
            case .RideError(let message):
                builderError = message
            default:
                builderError = "Failed to start workout"
            }
        } catch {
            builderError = error.localizedDescription
        }
    }
}
