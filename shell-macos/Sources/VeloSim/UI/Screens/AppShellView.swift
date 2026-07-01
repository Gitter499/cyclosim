import SwiftUI
import VeloSimSupport

@MainActor
struct AppShellView: View {
    @ObservedObject var model: VeloSimModel
    @Namespace private var navNamespace

    var body: some View {
        VStack(spacing: 0) {
            topNav
                .padding(.horizontal, 20)
                .padding(.vertical, 12)

            destinationContent
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .background(Color(nsColor: .windowBackgroundColor))
        .animation(.easeInOut(duration: 0.22), value: model.shellDestination)
    }

    private var topNav: some View {
        HStack(spacing: 0) {
            ForEach(Array(ShellDestination.allCases.enumerated()), id: \.element.id) { index, destination in
                if index > 0 {
                    Text("·")
                        .foregroundStyle(.tertiary)
                        .padding(.horizontal, 10)
                }
                Button {
                    model.shellDestination = destination
                } label: {
                    Text(destination.title)
                        .font(.subheadline.weight(model.shellDestination == destination ? .semibold : .regular))
                        .foregroundStyle(model.shellDestination == destination ? Color.accentColor : Color.secondary)
                        .padding(.horizontal, 10)
                        .padding(.vertical, 6)
                        .background {
                            if model.shellDestination == destination {
                                RoundedRectangle(cornerRadius: 8)
                                    .fill(Color.accentColor.opacity(0.12))
                                    .matchedGeometryEffect(id: "navSelection", in: navNamespace)
                            }
                        }
                }
                .buttonStyle(.plain)
            }
            Spacer()
        }
    }

    @ViewBuilder
    private var destinationContent: some View {
        switch model.shellDestination {
        case .home:
            DashboardView(model: model)
        case .activities:
            ActivitiesCatalogView(model: model)
        case .history:
            RideHistoryView(model: model)
        case .settings:
            SettingsBlockedView(isBlocked: model.shellPhase == .riding) {
                SettingsView(model: model, embeddedInTab: true)
            }
        }
    }
}

struct AppShellView_Previews: PreviewProvider {
    static var previews: some View {
        AppShellView(model: VeloSimModel())
            .frame(width: 900, height: 640)
    }
}
