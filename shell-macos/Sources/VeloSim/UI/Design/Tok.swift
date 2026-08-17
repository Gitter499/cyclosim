import CoreGraphics

/// Design tokens — spacing and radii per `VeloSim-Roadmap.md` Part II §2–4.
public enum Tok {
    public static let s1: CGFloat = 4
    public static let s2: CGFloat = 8
    public static let s3: CGFloat = 12
    public static let s4: CGFloat = 16
    public static let s6: CGFloat = 24
    public static let s8: CGFloat = 32

    public static let rCard: CGFloat = 22
    public static let rTile: CGFloat = 16

    public static let glassGap: CGFloat = 12

    // HUD graph tiles + scrim floor (hud-design skill §5: never below ~0.35).
    public static let sparkW: CGFloat = 180
    public static let sparkH: CGFloat = 44
    public static let elevBarH: CGFloat = 36
    public static let elevBarMaxW: CGFloat = 420
    public static let hudScrimAlpha: CGFloat = 0.45
}
