import Foundation

/// Shared step progression for connection wizards (Settings → Strava / Apple Music / Integrations).
public enum ConnectionWizardStep: Int, CaseIterable, Equatable {
    case intro = 0
    case action = 1
    case test = 2
    case done = 3

    public var title: String {
        switch self {
        case .intro: return "Intro"
        case .action: return "Connect"
        case .test: return "Test"
        case .done: return "Done"
        }
    }

    public var next: ConnectionWizardStep? {
        ConnectionWizardStep(rawValue: rawValue + 1)
    }

    public var previous: ConnectionWizardStep? {
        guard rawValue > 0 else { return nil }
        return ConnectionWizardStep(rawValue: rawValue - 1)
    }

    public static func advance(from step: ConnectionWizardStep) -> ConnectionWizardStep {
        step.next ?? .done
    }

    public static func retreat(from step: ConnectionWizardStep) -> ConnectionWizardStep {
        step.previous ?? .intro
    }
}
