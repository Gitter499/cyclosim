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

                if !hud.elevationProfile.isEmpty, !model.hudMinimalMode {
                    ElevationProfileBar(
                        profile: hud.elevationProfile,
                        totalM: hud.routeTotalM,
                        riderDistanceM: hud.distanceM
                    )
                    .padding(.top, Tok.s2)
                }

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
                        WorkoutBarView(
                            workout: workout,
                            ergBiasPct: hud.ergBiasPct,
                            onBiasDown: { model.adjustErgBias(by: -5) },
                            onBiasUp: { model.adjustErgBias(by: 5) },
                            onSkip: { model.skipWorkoutInterval() }
                        )
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
        .onAppear { model.refreshElevationProfile() }
    }

    // MARK: - Top pill: time · dist · grade

    private var topPill: some View {
        VeloHUDGlassContainer(spacing: Tok.glassGap) {
            HStack(spacing: Tok.glassGap) {
                hudStat(label: "TIME", value: HUDDurationFormat.hms(seconds: hud.elapsedS))
                hudStat(label: "SPEED", value: String(format: "%.1f km/h", hud.speedKph))
                hudStat(label: "DIST", value: String(format: "%.1f km", hud.distanceKm))
                hudStat(label: "GRADE", value: String(format: "%+.1f%%", hud.gradientPercent))
                if hud.lapCount > 0 {
                    hudStat(
                        label: "LAP \(hud.lapCount + 1)",
                        value: HUDDurationFormat.mmss(seconds: hud.currentLapElapsedS)
                    )
                }
            }
        }
        .allowsHitTesting(false)
    }

    // MARK: - Power card + secondary stats

    private var primaryCluster: some View {
        VeloHUDGlassContainer(spacing: Tok.glassGap) {
            VStack(alignment: .leading, spacing: Tok.glassGap) {
                powerCard
                if !hud.rollingPower.isEmpty {
                    RollingPowerGraph(series: hud.rollingPower, ftp: Double(hud.ftp))
                }
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


// MARK: - Rolling power graph (P2-B 2.3)

/// ~60 s power sparkline inside the primary cluster, colored by FTP fraction.
@MainActor
private struct RollingPowerGraph: View {
    let series: [Double]
    let ftp: Double

    var body: some View {
        Canvas { context, size in
            guard series.count > 1 else { return }
            let maxW = max(series.max() ?? 1, ftp * 1.2, 1)
            let stepX = size.width / CGFloat(series.count - 1)
            var path = Path()
            for (i, w) in series.enumerated() {
                let x = CGFloat(i) * stepX
                let y = size.height * (1 - CGFloat(w / maxW))
                if i == 0 { path.move(to: CGPoint(x: x, y: y)) }
                else { path.addLine(to: CGPoint(x: x, y: y)) }
            }
            // FTP reference line
            let ftpY = size.height * (1 - CGFloat(ftp / maxW))
            var ftpLine = Path()
            ftpLine.move(to: CGPoint(x: 0, y: ftpY))
            ftpLine.addLine(to: CGPoint(x: size.width, y: ftpY))
            context.stroke(
                ftpLine,
                with: .color(.white.opacity(0.25)),
                style: StrokeStyle(lineWidth: 1, dash: [3, 3])
            )
            context.stroke(path, with: .color(.orange), lineWidth: 2)
        }
        .frame(width: 180, height: 44)
        .padding(Tok.s2)
        .background(.black.opacity(0.25), in: RoundedRectangle(cornerRadius: Tok.rTile))
        .accessibilityLabel("Rolling one minute power graph")
    }
}

// MARK: - Elevation profile bar (P2-B 2.2)

/// Route elevation silhouette with the rider's position dot.
@MainActor
private struct ElevationProfileBar: View {
    let profile: [Double]
    let totalM: Double
    let riderDistanceM: Double

    var body: some View {
        Canvas { context, size in
            guard profile.count > 1, totalM > 0 else { return }
            let minE = profile.min() ?? 0
            let maxE = max(profile.max() ?? 1, minE + 1)
            let stepX = size.width / CGFloat(profile.count - 1)
            func y(_ elev: Double) -> CGFloat {
                let frac = (elev - minE) / (maxE - minE)
                return size.height * (1 - CGFloat(frac) * 0.85) - size.height * 0.05
            }
            var fill = Path()
            fill.move(to: CGPoint(x: 0, y: size.height))
            for (i, e) in profile.enumerated() {
                fill.addLine(to: CGPoint(x: CGFloat(i) * stepX, y: y(e)))
            }
            fill.addLine(to: CGPoint(x: size.width, y: size.height))
            fill.closeSubpath()
            context.fill(fill, with: .color(.white.opacity(0.22)))

            // Rider position dot on the silhouette.
            let frac = min(max(riderDistanceM / totalM, 0), 1)
            let idx = min(Int(frac * Double(profile.count - 1)), profile.count - 1)
            let dot = CGPoint(x: size.width * CGFloat(frac), y: y(profile[idx]))
            context.fill(
                Path(ellipseIn: CGRect(x: dot.x - 4, y: dot.y - 4, width: 8, height: 8)),
                with: .color(.orange)
            )
        }
        .frame(height: 36)
        .frame(maxWidth: 420)
        .background(.black.opacity(0.2), in: RoundedRectangle(cornerRadius: Tok.rTile))
        .allowsHitTesting(false)
        .accessibilityLabel("Route elevation profile with rider position")
    }
}
