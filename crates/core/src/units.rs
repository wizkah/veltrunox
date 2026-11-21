//! Validated values with units: time, speed, tuning and key shift.

/// A point in time or a duration in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Millis(u64);

impl Millis {
    pub const ZERO: Self = Self(0);

    /// Upper bound of a position in a recording (about 35 years); larger input is invalid.
    pub const MAX_POSITION: u64 = 1 << 40;

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// A position received from outside, checked against `MAX_POSITION`.
    pub fn position(value: u64) -> Result<Self, InvalidValue> {
        if value <= Self::MAX_POSITION {
            Ok(Self(value))
        } else {
            Err(InvalidValue::PositionOutOfRange)
        }
    }

    pub fn value(self) -> u64 {
        self.0
    }

    pub fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    pub fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidValue {
    #[error("speed must be between 0.5 and 2.0")]
    SpeedOutOfRange,
    #[error("tuning must be between 300 and 500 Hz")]
    TuningOutOfRange,
    #[error("key shift must be between -12 and 12 semitones")]
    KeyShiftOutOfRange,
    #[error("position is out of range")]
    PositionOutOfRange,
    #[error("a time range must end after it starts")]
    EmptyRange,
    #[error("tempo must be a positive number")]
    TempoNotPositive,
    #[error("a name must not be empty")]
    EmptyName,
}

/// Playback speed as a factor of the original; pitch is not affected.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Speed(f64);

impl Speed {
    pub const NORMAL: Self = Self(1.0);
    const RANGE: std::ops::RangeInclusive<f64> = 0.5..=2.0;

    pub fn new(factor: f64) -> Result<Self, InvalidValue> {
        if Self::RANGE.contains(&factor) {
            Ok(Self(factor))
        } else {
            Err(InvalidValue::SpeedOutOfRange)
        }
    }

    pub fn factor(self) -> f64 {
        self.0
    }
}

/// Reference tone (concert pitch) in hertz.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TuningHz(f64);

impl TuningHz {
    pub const CONCERT_PITCH: Self = Self(440.0);
    const RANGE: std::ops::RangeInclusive<f64> = 300.0..=500.0;

    pub fn new(hertz: f64) -> Result<Self, InvalidValue> {
        if Self::RANGE.contains(&hertz) {
            Ok(Self(hertz))
        } else {
            Err(InvalidValue::TuningOutOfRange)
        }
    }

    pub fn hertz(self) -> f64 {
        self.0
    }
}

/// Pitch shift of the key in semitones relative to the recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyShift(i8);

impl KeyShift {
    pub const NONE: Self = Self(0);
    const LIMIT: i8 = 12;

    pub fn new(semitones: i8) -> Result<Self, InvalidValue> {
        if (-Self::LIMIT..=Self::LIMIT).contains(&semitones) {
            Ok(Self(semitones))
        } else {
            Err(InvalidValue::KeyShiftOutOfRange)
        }
    }

    pub fn semitones(self) -> i8 {
        self.0
    }
}

/// A span of time with `start < end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeRange {
    start: Millis,
    end: Millis,
}

impl TimeRange {
    pub fn new(start: Millis, end: Millis) -> Result<Self, InvalidValue> {
        if end > start {
            Ok(Self { start, end })
        } else {
            Err(InvalidValue::EmptyRange)
        }
    }

    pub fn start(self) -> Millis {
        self.start
    }

    pub fn end(self) -> Millis {
        self.end
    }

    pub fn length(self) -> Millis {
        self.end.saturating_sub(self.start)
    }

    pub fn contains(self, position: Millis) -> bool {
        self.start <= position && position < self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speed_accepts_only_the_supported_range() {
        assert!(Speed::new(0.5).is_ok());
        assert!(Speed::new(2.0).is_ok());
        assert_eq!(Speed::new(0.49), Err(InvalidValue::SpeedOutOfRange));
        assert_eq!(Speed::new(2.01), Err(InvalidValue::SpeedOutOfRange));
        assert_eq!(Speed::new(f64::NAN), Err(InvalidValue::SpeedOutOfRange));
    }

    #[test]
    fn tuning_accepts_only_the_supported_range() {
        assert!(TuningHz::new(412.0).is_ok());
        assert_eq!(TuningHz::new(299.0), Err(InvalidValue::TuningOutOfRange));
        assert_eq!(TuningHz::new(f64::NAN), Err(InvalidValue::TuningOutOfRange));
    }

    #[test]
    fn key_shift_is_limited_to_an_octave() {
        assert!(KeyShift::new(-12).is_ok());
        assert!(KeyShift::new(12).is_ok());
        assert_eq!(KeyShift::new(13), Err(InvalidValue::KeyShiftOutOfRange));
    }

    #[test]
    fn positions_beyond_the_limit_are_invalid() {
        assert!(Millis::position(Millis::MAX_POSITION).is_ok());
        assert_eq!(
            Millis::position(Millis::MAX_POSITION + 1),
            Err(InvalidValue::PositionOutOfRange)
        );
    }

    #[test]
    fn range_must_end_after_it_starts() {
        let (a, b) = (Millis::new(1_000), Millis::new(2_000));
        assert_eq!(TimeRange::new(b, a), Err(InvalidValue::EmptyRange));
        assert_eq!(TimeRange::new(a, a), Err(InvalidValue::EmptyRange));
        let range = TimeRange::new(a, b).unwrap();
        assert_eq!(range.length(), Millis::new(1_000));
        assert!(range.contains(a));
        assert!(!range.contains(b));
    }
}
