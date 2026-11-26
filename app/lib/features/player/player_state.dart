/// What happens at the end of the region being played.
enum PlayMode {
  /// Playback runs on to the end of the recording.
  toEnd,

  /// Playback stops at the end of the region.
  toRegionEnd,

  /// Playback wraps from the end of the region back to its start.
  loop;

  /// The mode a tap on the toggle switches to: the three modes in a circle.
  PlayMode get next => PlayMode.values[(index + 1) % PlayMode.values.length];
}

/// How one stem sounds in the mix.
class StemChannel {
  const StemChannel({
    this.volumePercent = 100,
    this.muted = false,
    this.solo = false,
    this.audible = true,
  });

  final int volumePercent;
  final bool muted;
  final bool solo;

  /// Whether the stem is heard, with mute and solo of all of them counted.
  final bool audible;
}

/// The shared playback state as the interface shows it.
class PlayerState {
  const PlayerState({
    required this.recordingId,
    required this.running,
    required this.speed,
    required this.tuningHz,
    required this.keyShiftSemitones,
    required this.volumePercent,
    this.positionMs = 0,
    this.playMode = PlayMode.toEnd,
    this.playRange,
    this.stemsEnabled = false,
    this.stemChannels = const [],
  });

  final int? recordingId;
  final bool running;
  final double speed;
  final double tuningHz;
  final int keyShiftSemitones;

  /// Output volume of the server, 0 to 100.
  final int volumePercent;

  /// Position at the moment the server made this state; the player reads the
  /// position now from the source.
  final int positionMs;
  final PlayMode playMode;

  /// Whether the stems play instead of the original recording.
  final bool stemsEnabled;

  /// The mix, one entry per possible stem.
  final List<StemChannel> stemChannels;

  /// The region being played, if any.
  final ({int startMs, int endMs})? playRange;
}
