//! Strongly typed physical quantities — no bare `f64` in core APIs.

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Watts(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Meters(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct MetersPerSecond(pub f64);

/// Grade as rise/run ratio (e.g. 0.05 = 5%).
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Grade(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Kilograms(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Rpm(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Bpm(pub f64);

impl Watts {
    pub fn new(v: f64) -> Self {
        Self(v)
    }
}

impl Meters {
    pub fn new(v: f64) -> Self {
        Self(v)
    }
}

impl MetersPerSecond {
    pub fn new(v: f64) -> Self {
        Self(v)
    }
}

impl Grade {
    pub fn new(v: f64) -> Self {
        Self(v)
    }
}

impl Kilograms {
    pub fn new(v: f64) -> Self {
        Self(v)
    }
}

impl Rpm {
    pub fn new(v: f64) -> Self {
        Self(v)
    }
}

impl Bpm {
    pub fn new(v: f64) -> Self {
        Self(v)
    }
}
