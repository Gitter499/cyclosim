import SwiftUI
import VeloFFI
import VeloSimSupport

/// In-ride metrics overlay on the Metal viewport (guide §5.2).
///
/// SwiftUI owns all live ride readouts. Rust glyphon HUD is disabled at renderer init;
/// throttled values come from `HUDModel` (~8 Hz via `HUDCoordinator`).
@MainActor
struct RideHUDOverlay: View {
    @ObservedObject var model: VeloSimModel
    @Environment(\.accessibilityReduceTransparency) private var reduceTransparency
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    private var hud: HUDModel { model.hudModel }

    var body: some View {
        ZStack {
            VStack(spacing: 0) {
                topPill
                    .padding(.top, Tok.s4)

                Spacer(minLength: 0)

                if model.hudMinimalMode {
                    powerCard
                        .frame(maxWidth: .infinity, alignment: .leading)
                } else {
                    HStack(alignment: .bottom, spacing: Tok.s4) {
                        primaryCluster
                        Spacer(minLength: 0)
                        RideControlCluster(model: model)
                    }

                    if let workout = hud.workout {
                        WorkoutBarView(workout: workout)
                            .padding(.top, Tok.s3)
                    }
                }

                if !model.tilesAttribution.isEmpty, !model.hudMinimalMode {
                    tilesAttributionRow
                        .padding(.top, Tok.s2)
                        .padding(.bottom, Tok.s3)
                }
            }
            .padding(.horizontal, Tok.s6)
            .padding(.bottom, Tok.s4)
        }
        .allowsHitTesting(!model.hudMinimalMode)
    }

    // MARK: - Top pill: time · dist · grade

    private var topPill: some View {
        VeloHUDGlassContainer(spacing: Tok.glassGap) {
            HStack(spacing: Tok.glassGap) {
                hudStat(label: "TIME", value: HUDDurationFormat.hms(seconds: hud.elapsedS))
                hudStat(label: "DIST", value: String(format: "%.1f km", hud.distanceKm))
                hudStat(label: "GRADE", value: String(format: "%+.1f%%", hud.gradientPercent))
            }
        }
        .allowsHitTesting(false)
    }

    // MARK: - Power card + secondary stats

    private var primaryCluster: some View {
        VeloHUDGlassContainer(spacing: Tok.glassGap) {
            VStack(alignment: .leading, spacing: Tok.glassGap) {
                powerCard
                HStack(spacing: Tok.glassGap) {
                    hudStat(label: "CAD", value: "\(hud.cadence)")
                    hudStat(label: "HR", value: "\(hud.heartRate)")
                    hudStat(label: "W/KG", value: String(format: "%.1f", hud.wattsPerKg))
                }
            }
        }
        .allowsHitTesting(false)
    }

    private var powerCard: some View {
        let zone = PowerZone.of(watts: hud.power, ftp: hud.ftp)
        return HStack(alignment: .firstTextBaseline, spacing: Tok.s2) {
            Text("\(hud.power)")
                .font(Typo.bigMetric())
                .monospacedDigit()
                .contentTransition(.numericText())
                .foregroundStyle(.white)
                .accessibilityLabel("Power, \(hud.power) watts")
            Text("W")
                .font(Typo.unit())
                .foregroundStyle(.secondary)
        }
        .padding(.horizontal, Tok.s4)
        .padding(.vertical, Tok.s3)
        .hudPowerSurface(zone: zone, reduceTransparency: reduceTransparency, reduceMotion: reduceMotion)
    }

    private func hudStat(label: String, value: String) -> some View {
        VStack(spacing: Tok.s1) {
            Text(label)
                .font(Typo.label())
                .foregroundStyle(.secondary)
            Text(value)
                .font(Typo.metric())
                .monospacedDigit()
                .contentTransition(.numericText())
                .foregroundStyle(.white)
        }
        .padding(.horizontal, Tok.s3)
        .padding(.vertical, Tok.s2)
        .hudSurface(RoundedRectangle(cornerRadius: Tok.rTile), reduceTransparency: reduceTransparency)
        .accessibilityLabel("\(label), \(value)")
    }

    private var tilesAttributionRow: some View {
        Text(model.tilesAttribution)
            .font(.caption2)
            .foregroundStyle(.white.opacity(0.7))
            .lineLimit(2)
            .frame(maxWidth: .infinity, alignment: .leading)
    }
}
