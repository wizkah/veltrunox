/// The shared playback state as the interface shows it.
class PlayerState {
  const PlayerState({
    required this.recordingId,
    required this.running,
    required this.speed,
    required this.tuningHz,
    required this.keyShiftSemitones,
    required this.volumePercent,
  });

  final int? recordingId;
  final bool running;
  final double speed;
  final double tuningHz;
  final int keyShiftSemitones;

  /// Output volume of the server, 0 to 100.
  final int volumePercent;
}
