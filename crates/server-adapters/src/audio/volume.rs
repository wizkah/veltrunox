//! The output volume of the server. The system mixer of the sound card is used
//! when the device has one (ALSA on Linux); otherwise the render engine applies a gain.

use veltrunox_core::Volume;

/// The decibel span a mixer control is mapped to, so that the percentages of
/// the slider feel even: a card with a range of 100 dB would otherwise be
/// inaudible below about 60 %.
#[cfg(any(target_os = "linux", test))]
const MIXER_SPAN_MILLIBEL: i64 = 6_000;

/// The mixer value (in millibel) for a percentage, linear in decibel between
/// `bottom` and `top`.
#[cfg(any(target_os = "linux", test))]
fn percent_to_millibel(percent: u8, bottom: i64, top: i64) -> i64 {
    bottom + (top - bottom) * i64::from(percent) / 100
}

/// The percentage of a mixer value in millibel; values outside the span are clamped.
#[cfg(any(target_os = "linux", test))]
fn millibel_to_percent(value: i64, bottom: i64, top: i64) -> u8 {
    if top <= bottom {
        return 100;
    }
    let percent = (value - bottom) * 100 / (top - bottom);
    percent.clamp(0, 100) as u8
}

/// The lower end of the span for a control with the given decibel range.
#[cfg(any(target_os = "linux", test))]
fn span_bottom(min: i64, max: i64) -> i64 {
    min.max(max - MIXER_SPAN_MILLIBEL)
}

/// The amplitude factor the engine applies when there is no mixer. Squared,
/// because loudness is perceived roughly logarithmically.
pub fn engine_gain(volume: Volume) -> f32 {
    let fraction = f32::from(volume.percent()) / 100.0;
    fraction * fraction
}

/// The ALSA card behind a configured device name such as
/// `alsa:plughw:CARD=Headphones,DEV=0`; `None` means the default mixer.
#[cfg(any(target_os = "linux", test))]
fn mixer_name(device: Option<&str>) -> String {
    let card = device
        .and_then(|name| name.split_once("CARD="))
        .map(|(_, rest)| rest.split(',').next().unwrap_or(rest))
        .filter(|card| !card.is_empty());
    match card {
        Some(card) => format!("hw:{card}"),
        None => "default".to_owned(),
    }
}

#[cfg(target_os = "linux")]
mod system {
    use super::{millibel_to_percent, mixer_name, percent_to_millibel, span_bottom};
    use alsa::Round;
    use alsa::mixer::{MilliBel, Mixer, Selem, SelemChannelId, SelemId};
    use veltrunox_core::Volume;

    /// Controls that usually set the loudness of the output, best first.
    const PREFERRED: [&str; 5] = ["Master", "PCM", "Headphone", "Speaker", "Digital"];

    /// The playback volume control of the sound card.
    pub struct SystemMixer {
        mixer: Mixer,
        control: SelemId,
    }

    impl SystemMixer {
        /// `None` if the card has no mixer or no playback volume control.
        pub fn open(device: Option<&str>) -> Option<Self> {
            let mixer = Mixer::new(&mixer_name(device), false).ok()?;
            let control = pick_control(&mixer)?;
            Some(Self { mixer, control })
        }

        fn selem(&self) -> Option<Selem<'_>> {
            self.mixer.find_selem(&self.control)
        }

        /// The volume now; `None` if the control cannot be read.
        pub fn volume(&self) -> Option<Volume> {
            let selem = self.selem()?;
            let (min, max) = selem.get_playback_volume_range();
            let channel = SelemChannelId::FrontLeft;
            let percent = match selem.get_playback_db_range() {
                (MilliBel(low), MilliBel(high)) if high > low => {
                    let db = selem.get_playback_vol_db(channel).ok()?.0;
                    millibel_to_percent(db, span_bottom(low, high), high)
                }
                _ => millibel_to_percent(selem.get_playback_volume(channel).ok()?, min, max),
            };
            Volume::new(percent).ok()
        }

        pub fn set(&self, volume: Volume) -> Result<(), alsa::Error> {
            let Some(selem) = self.selem() else {
                return Ok(());
            };
            let percent = volume.percent();
            match selem.get_playback_db_range() {
                (MilliBel(low), MilliBel(high)) if high > low => {
                    let db = percent_to_millibel(percent, span_bottom(low, high), high);
                    if percent == 0 {
                        selem.set_playback_volume_all(selem.get_playback_volume_range().0)?;
                    } else {
                        selem.set_playback_db_all(MilliBel(db), Round::Floor)?;
                    }
                }
                _ => {
                    let (min, max) = selem.get_playback_volume_range();
                    selem.set_playback_volume_all(percent_to_millibel(percent, min, max))?;
                }
            }
            if percent > 0 && selem.has_playback_switch() {
                selem.set_playback_switch_all(1)?;
            }
            Ok(())
        }
    }

    fn pick_control(mixer: &Mixer) -> Option<SelemId> {
        let controls: Vec<(String, u32)> = mixer
            .iter()
            .filter_map(Selem::new)
            .filter(Selem::has_playback_volume)
            .filter_map(|selem| {
                let id = selem.get_id();
                Some((id.get_name().ok()?.to_owned(), id.get_index()))
            })
            .collect();
        let (name, index) = PREFERRED
            .iter()
            .find_map(|wanted| controls.iter().find(|(name, _)| name == wanted))
            .or_else(|| controls.first())?;
        Some(SelemId::new(name, *index))
    }
}

#[cfg(not(target_os = "linux"))]
mod system {
    use veltrunox_core::Volume;

    /// There is no system mixer to control on this platform; the engine's gain is used.
    pub struct SystemMixer;

    impl SystemMixer {
        pub fn open(_device: Option<&str>) -> Option<Self> {
            None
        }

        pub fn volume(&self) -> Option<Volume> {
            None
        }

        pub fn set(&self, _volume: Volume) -> Result<(), String> {
            Ok(())
        }
    }
}

pub use system::SystemMixer;

/// The volume the system mixer of the device has now; full volume without a
/// mixer, where the engine's gain starts at unity.
pub fn system_volume(device: Option<&str>) -> Volume {
    SystemMixer::open(device)
        .and_then(|mixer| mixer.volume())
        .unwrap_or(Volume::FULL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentages_are_even_in_decibel_between_the_ends() {
        assert_eq!(percent_to_millibel(0, -6_000, 400), -6_000);
        assert_eq!(percent_to_millibel(100, -6_000, 400), 400);
        assert_eq!(percent_to_millibel(50, -6_000, 400), -2_800);
    }

    #[test]
    fn a_mixer_value_gives_back_its_percentage() {
        for percent in [0, 1, 25, 50, 99, 100] {
            let value = percent_to_millibel(percent, -6_000, 400);
            assert_eq!(millibel_to_percent(value, -6_000, 400), percent);
        }
        assert_eq!(millibel_to_percent(-20_000, -6_000, 400), 0);
        assert_eq!(millibel_to_percent(900, -6_000, 400), 100);
    }

    #[test]
    fn a_wide_range_is_cut_to_the_audible_span() {
        assert_eq!(span_bottom(-10_239, 400), -5_600);
        assert_eq!(span_bottom(-3_000, 0), -3_000);
    }

    #[test]
    fn the_engine_gain_is_silent_at_zero_and_unity_at_full() {
        assert_eq!(engine_gain(Volume::SILENT), 0.0);
        assert_eq!(engine_gain(Volume::FULL), 1.0);
        assert!(engine_gain(Volume::new(50).unwrap()) < 0.5);
    }

    #[test]
    fn the_card_comes_from_the_device_name() {
        assert_eq!(
            mixer_name(Some("alsa:plughw:CARD=Headphones,DEV=0")),
            "hw:Headphones"
        );
        assert_eq!(
            mixer_name(Some("alsa:sysdefault:CARD=Loopback")),
            "hw:Loopback"
        );
        assert_eq!(mixer_name(Some("alsa:default")), "default");
        assert_eq!(mixer_name(None), "default");
    }
}
