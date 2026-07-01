import SwiftUI
import VeloSimSupport

// MARK: - Route select with sparkline stub (§7.3)

@MainActor
struct RouteSelectView: View {
    @ObservedObject var model: VeloSimModel

    var body: some View {
        VStack(alignment: .leading, spacing: Tok.s3) {
            if model.availableRoutes.isEmpty {
                Text("Import a GPX route to ride with elevation.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            } else {
                ForEach(model.availableRoutes, id: \.routeId) { route in
                    Button {
                        model.selectRoute(route.routeId)
                    } label: {
                        HStack(spacing: Tok.s3) {
                            RouteElevationSparkline(routeId: route.routeId)
                                .frame(width: 72, height: 28)

                            VStack(alignment: .leading, spacing: 2) {
                                Text(route.name)
                                    .font(.subheadline.weight(.semibold))
                                    .foregroundStyle(.primary)
                                Text("\(Int(route.totalDistanceM / 1000)) km")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                                    .monospacedDigit()
                            }

                            Spacer()

                            if model.activeRouteId == route.routeId {
                                Image(systemName: "checkmark.circle.fill")
                                    .foregroundStyle(.green)
                            }
                        }
                        .padding(.vertical, Tok.s2)
                    }
                    .buttonStyle(.plain)
                }
            }
        }
    }
}

struct RouteElevationSparkline: View {
    let routeId: String

    private var samples: [CGFloat] {
        var hash: UInt64 = 14695981039346656037
        for byte in routeId.utf8 {
            hash = (hash ^ UInt64(byte)) &* 1099511628211
        }
        return (0 ..< 24).map { i in
            let seed = Double((hash &+ UInt64(i * 17)) % 1000) / 1000.0
            return CGFloat(0.2 + seed * 0.6)
        }
    }

    var body: some View {
        GeometryReader { geo in
            Path { path in
                let w = geo.size.width
                let h = geo.size.height
                let step = w / CGFloat(max(samples.count - 1, 1))
                for (i, y) in samples.enumerated() {
                    let x = CGFloat(i) * step
                    let py = h * (1 - y)
                    if i == 0 { path.move(to: CGPoint(x: x, y: py)) }
                    else { path.addLine(to: CGPoint(x: x, y: py)) }
                }
            }
            .stroke(Color.accentColor, lineWidth: 1.5)
        }
    }
}
