//! Structured workout timeline and interval engine (M5).

use serde::{Deserialize, Serialize};
use velo_units::Watts;

/// How an interval specifies trainer resistance.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WorkoutTarget {
    /// Fixed ERG wattage.
    ErgWatts(f64),
    /// Percent of rider FTP (0–100+).
    FtpPercent(f64),
    /// Free ride; trainer follows SIM grade (no ERG hold).
    FreeRide,
}

/// One workout step bounded by elapsed time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkoutInterval {
    pub name: String,
    pub duration_s: f64,
    pub target: WorkoutTarget,
}

/// Ordered workout definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workout {
    pub name: String,
    pub intervals: Vec<WorkoutInterval>,
}

impl Workout {
    pub fn total_duration_s(&self) -> f64 {
        self.intervals.iter().map(|i| i.duration_s).sum()
    }

    /// Simple 2×20-style template for tests and defaults.
    pub fn sample_threshold() -> Self {
        Self {
            name: "2x20 Threshold".into(),
            intervals: vec![
                WorkoutInterval {
                    name: "Warmup".into(),
                    duration_s: 600.0,
                    target: WorkoutTarget::FtpPercent(55.0),
                },
                WorkoutInterval {
                    name: "Interval 1".into(),
                    duration_s: 1200.0,
                    target: WorkoutTarget::FtpPercent(95.0),
                },
                WorkoutInterval {
                    name: "Recovery".into(),
                    duration_s: 600.0,
                    target: WorkoutTarget::FtpPercent(50.0),
                },
                WorkoutInterval {
                    name: "Interval 2".into(),
                    duration_s: 1200.0,
                    target: WorkoutTarget::FtpPercent(95.0),
                },
                WorkoutInterval {
                    name: "Cooldown".into(),
                    duration_s: 600.0,
                    target: WorkoutTarget::FtpPercent(45.0),
                },
            ],
        }
    }
}

/// Live workout playback state.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkoutState {
    pub interval_index: usize,
    pub interval_elapsed_s: f64,
    pub workout_elapsed_s: f64,
    pub finished: bool,
}

/// Advances a workout timeline and resolves ERG targets.
#[derive(Debug, Clone)]
pub struct WorkoutEngine {
    workout: Workout,
    ftp_w: f64,
    state: WorkoutState,
}

impl WorkoutEngine {
    pub fn new(workout: Workout, ftp_w: f64) -> Self {
        Self {
            workout,
            ftp_w,
            state: WorkoutState {
                interval_index: 0,
                interval_elapsed_s: 0.0,
                workout_elapsed_s: 0.0,
                finished: false,
            },
        }
    }

    pub fn workout(&self) -> &Workout {
        &self.workout
    }

    pub fn state(&self) -> &WorkoutState {
        &self.state
    }

    pub fn current_interval(&self) -> Option<&WorkoutInterval> {
        if self.state.finished {
            return None;
        }
        self.workout.intervals.get(self.state.interval_index)
    }

    /// Resolve the active interval's target to watts (None for free ride).
    pub fn target_watts(&self) -> Option<Watts> {
        let interval = self.current_interval()?;
        match interval.target {
            WorkoutTarget::ErgWatts(w) => Some(Watts::new(w)),
            WorkoutTarget::FtpPercent(pct) => Some(Watts::new(self.ftp_w * pct / 100.0)),
            WorkoutTarget::FreeRide => None,
        }
    }

    pub fn is_free_ride_interval(&self) -> bool {
        self.current_interval()
            .is_some_and(|i| matches!(i.target, WorkoutTarget::FreeRide))
    }

    /// Advance by fixed sim dt; returns true if interval or workout changed.
    pub fn tick(&mut self, dt_s: f64) -> bool {
        if self.state.finished {
            return false;
        }
        let Some(interval) = self.workout.intervals.get(self.state.interval_index) else {
            self.state.finished = true;
            return true;
        };

        let prev_index = self.state.interval_index;
        self.state.interval_elapsed_s += dt_s;
        self.state.workout_elapsed_s += dt_s;

        if self.state.interval_elapsed_s >= interval.duration_s {
            self.state.interval_index += 1;
            self.state.interval_elapsed_s = 0.0;
            if self.state.interval_index >= self.workout.intervals.len() {
                self.state.finished = true;
            }
        }

        prev_index != self.state.interval_index || self.state.finished
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advances_through_intervals() {
        let mut engine = WorkoutEngine::new(
            Workout {
                name: "test".into(),
                intervals: vec![
                    WorkoutInterval {
                        name: "A".into(),
                        duration_s: 10.0,
                        target: WorkoutTarget::ErgWatts(100.0),
                    },
                    WorkoutInterval {
                        name: "B".into(),
                        duration_s: 10.0,
                        target: WorkoutTarget::ErgWatts(200.0),
                    },
                ],
            },
            250.0,
        );

        assert_eq!(engine.target_watts(), Some(Watts::new(100.0)));
        for _ in 0..1001 {
            engine.tick(0.01);
        }
        assert_eq!(engine.state().interval_index, 1);
        assert_eq!(engine.target_watts(), Some(Watts::new(200.0)));
        for _ in 0..1001 {
            engine.tick(0.01);
        }
        assert!(engine.state().finished);
        assert!(engine.current_interval().is_none());
    }

    #[test]
    fn ftp_percent_resolves_from_ftp() {
        let engine = WorkoutEngine::new(
            Workout {
                name: "ftp".into(),
                intervals: vec![WorkoutInterval {
                    name: "on".into(),
                    duration_s: 60.0,
                    target: WorkoutTarget::FtpPercent(80.0),
                }],
            },
            250.0,
        );
        assert_eq!(engine.target_watts(), Some(Watts::new(200.0)));
    }

    #[test]
    fn sample_threshold_duration() {
        let w = Workout::sample_threshold();
        assert!((w.total_duration_s() - 4200.0).abs() < 0.1);
    }
}
