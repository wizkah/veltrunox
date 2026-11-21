//! Tempo and pitch arithmetic for playback: target tempo to speed, tuning and key to a pitch shift.

use crate::playback::PlaybackSettings;
use crate::units::{InvalidValue, Speed, TuningHz};

/// Tempo in beats per minute; positive and finite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bpm(f64);

impl Bpm {
    pub fn new(value: f64) -> Result<Self, InvalidValue> {
        if value.is_finite() && value > 0.0 {
            Ok(Self(value))
        } else {
            Err(InvalidValue::TempoNotPositive)
        }
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

/// The speed a target tempo asks for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TargetSpeed {
    /// Within the supported range; ready to be set.
    Supported(Speed),
    /// Outside the supported range; the application points this out and keeps the old speed.
    OutOfRange { factor: f64 },
}

/// Speed that plays a recording of tempo `recording` at tempo `target`.
pub fn speed_for_target_tempo(target: Bpm, recording: Bpm) -> TargetSpeed {
    let factor = target.value() / recording.value();
    match Speed::new(factor) {
        Ok(speed) => TargetSpeed::Supported(speed),
        Err(_) => TargetSpeed::OutOfRange { factor },
    }
}

/// Pitch shift in semitones that applies the settings to a recording whose
/// own tuning is `recording_tuning`: key shift plus the tuning difference.
pub fn pitch_shift_semitones(settings: &PlaybackSettings, recording_tuning: TuningHz) -> f64 {
    let tuning = 12.0 * (settings.tuning.hertz() / recording_tuning.hertz()).log2();
    f64::from(settings.key.semitones()) + tuning
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::units::KeyShift;

    fn bpm(value: f64) -> Bpm {
        Bpm::new(value).unwrap()
    }

    #[test]
    fn tempo_must_be_positive_and_finite() {
        assert!(Bpm::new(0.0).is_err());
        assert!(Bpm::new(-90.0).is_err());
        assert!(Bpm::new(f64::NAN).is_err());
        assert!(Bpm::new(f64::INFINITY).is_err());
    }

    #[test]
    fn target_tempo_gives_the_ratio() {
        let TargetSpeed::Supported(speed) = speed_for_target_tempo(bpm(90.0), bpm(120.0)) else {
            panic!("expected a supported speed");
        };
        assert!((speed.factor() - 0.75).abs() < 1e-12);
    }

    #[test]
    fn target_tempo_outside_the_range_is_reported() {
        assert_eq!(
            speed_for_target_tempo(bpm(300.0), bpm(100.0)),
            TargetSpeed::OutOfRange { factor: 3.0 }
        );
        assert_eq!(
            speed_for_target_tempo(bpm(40.0), bpm(100.0)),
            TargetSpeed::OutOfRange { factor: 0.4 }
        );
    }

    #[test]
    fn range_limits_are_supported() {
        assert!(matches!(
            speed_for_target_tempo(bpm(50.0), bpm(100.0)),
            TargetSpeed::Supported(_)
        ));
        assert!(matches!(
            speed_for_target_tempo(bpm(200.0), bpm(100.0)),
            TargetSpeed::Supported(_)
        ));
    }

    #[test]
    fn no_change_needs_no_shift() {
        let settings = PlaybackSettings::default();
        assert_eq!(
            pitch_shift_semitones(&settings, TuningHz::CONCERT_PITCH),
            0.0
        );
    }

    #[test]
    fn key_shift_adds_whole_semitones() {
        let settings = PlaybackSettings {
            key: KeyShift::new(-3).unwrap(),
            ..PlaybackSettings::default()
        };
        assert_eq!(
            pitch_shift_semitones(&settings, TuningHz::CONCERT_PITCH),
            -3.0
        );
    }

    #[test]
    fn tuning_difference_is_twelve_times_the_log_ratio() {
        let settings = PlaybackSettings {
            tuning: TuningHz::new(480.0).unwrap(),
            ..PlaybackSettings::default()
        };
        let shift = pitch_shift_semitones(&settings, TuningHz::new(400.0).unwrap());
        assert!((shift - 12.0 * (1.2_f64).log2()).abs() < 1e-12);
    }

    #[test]
    fn tuning_below_the_recording_shifts_down_and_adds_to_the_key() {
        let settings = PlaybackSettings {
            tuning: TuningHz::new(432.0).unwrap(),
            key: KeyShift::new(2).unwrap(),
            ..PlaybackSettings::default()
        };
        let shift = pitch_shift_semitones(&settings, TuningHz::CONCERT_PITCH);
        assert!(shift < 2.0 && shift > 1.6);
    }
}
