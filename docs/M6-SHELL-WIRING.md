# M6 + replay-camera shell wiring (needs a Mac)

The Rust core and FFI for M6 (audio events, steering) and the M5 cinematic
replay camera are shipped and tested. This file is the exact Swift wiring
left, written against the current `VeloSimModel` / `HighlightClipEncoder`
structure. Regenerate bindings first (`just bindgen`) so the new
`VeloHandle` methods appear: `drainAudioEvents()`, `setSteering(axis:recenter:)`,
`steeringOffsetM()`, `replayCameraPoses(clip:fps:)`, `setReplayCameraPose(pose:)`.

## 1. Segment-aware music (MusicKit)

Core queues an `AudioEventDto` at workout start, every interval boundary, and
workout finish. Drain in the sim tick and map to MusicKit:

```swift
// VeloSimModel.simTick(), after handle.tick(...):
for event in handle.drainAudioEvents() {
    audioDirector.handle(event)
}
```

```swift
// New file: Sources/VeloSim/Audio/MusicDirector.swift
import MusicKit

@MainActor final class MusicDirector {
    func handle(_ event: AudioEventDto) {
        switch event.intent {
        case .start:      startQueue(for: event.energy)
        case .transition: transition(to: event.energy)
        case .duck:       duckBriefly()
        }
    }

    private func playlistName(for energy: SegmentEnergyDto) -> String {
        switch energy {
        case .warmup:    return "VeloSim Warmup"
        case .build:     return "VeloSim Build"
        case .threshold: return "VeloSim Threshold"
        case .recovery:  return "VeloSim Recovery"
        case .cooldown:  return "VeloSim Cooldown"
        }
    }

    private func startQueue(for energy: SegmentEnergyDto) { /* fetch playlist by name, set queue, play */ }
    private func transition(to energy: SegmentEnergyDto) { /* swap queue at next track boundary; volume dip-and-restore */ }
    private func duckBriefly() { /* temporary volume duck */ }
}
```

Playback control only — no raw audio access (§13); ship as "smart
segment-aware playback".

## 2. AirPods steering (CMHeadphoneMotionManager)

Core handles deadzone, slew (2.5 m/s), and ±3.5 m clamp; the shell only maps
yaw → axis and forwards it:

```swift
// New file: Sources/VeloSim/Input/HeadSteering.swift
import CoreMotion

final class HeadSteering {
    private let manager = CMHeadphoneMotionManager()
    private var neutralYaw: Double?

    func start(handle: VeloHandle) {
        guard manager.isDeviceMotionAvailable else { return }
        manager.startDeviceMotionUpdates(to: .main) { motion, _ in
            guard let yaw = motion?.attitude.yaw else { return }
            let neutral = self.neutralYaw ?? { self.neutralYaw = yaw; return yaw }()
            // ±30° of head turn maps to the full axis.
            let axis = ((yaw - neutral) / (.pi / 6)).clamped(to: -1...1)
            handle.setSteering(axis: axis, recenter: false)
        }
    }

    func recenter(handle: VeloHandle) {
        neutralYaw = nil
        handle.setSteering(axis: 0, recenter: true)
    }

    func stop() { manager.stopDeviceMotionUpdates() }
}
```

Bind a recenter key/menu item to `recenter(handle:)` — yaw drift is real.
Keyboard fallback: left/right arrows call `setSteering(axis: ∓1, recenter: false)`
on keyDown and `setSteering(axis: 0, recenter: false)` on keyUp.

## 3. Cinematic replay clips (VideoToolbox)

Replace the ring-buffer-only clip path with a replay render at encode time
(`finishRideAndPublish` flow). For each planned clip:

```swift
func renderReplayClip(clip: HighlightClipRequestDto, fps: Double = 30) throws -> [HighlightClipEncoder.Frame] {
    let poses = try handle.replayCameraPoses(clip: clip, fps: fps)
    var frames: [HighlightClipEncoder.Frame] = []
    for pose in poses {
        try handle.setReplayCameraPose(pose: pose)
        let fb = try handle.captureFramebufferRgba()
        frames.append(.init(width: Int(fb.width), height: Int(fb.height), rgba: fb.rgbaPixels))
    }
    try handle.setReplayCameraPose(pose: nil)   // restore chase camera
    return frames
}
```

Feed the collected frames to the existing
`HighlightClipEncoder.encode(frames:fps:outputURL:)`. Requires a loaded route
and the retained post-ride samples (both already in core). Shot styles are
chosen per clip label in core: Start → drone rise, Power surge → orbit,
Mid-ride → low flyby, Finish → chase pull.

## Verification without hardware

`velo-eval-mcp` previews all of this headlessly: `replay_camera_preview`
(camera paths), `sim_scenario` with `steer_axis` (lateral offset), and
workout scenarios emit the same audio events the shell drains.
