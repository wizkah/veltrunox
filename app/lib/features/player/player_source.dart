import 'player_state.dart';
import 'target_tempo.dart';

/// Port: the shared player. The server plays; this only controls and shows it.
abstract interface class PlayerSource {
  Stream<PlayerState> watchState();

  /// Reasons why the server refused a command.
  Stream<String> rejections();

  /// Whether the server has an audio device; without one it plays nothing.
  /// Emits when the server tells, and again when that changes.
  Stream<bool> audioOutput();

  /// The playback position now in milliseconds; `null` before the first state.
  Future<int?> position();

  Future<void> select(int recordingId);
  Future<void> play();
  Future<void> pause();
  Future<void> seek(int positionMs);

  /// The position handle is dragged to `positionMs`; the source limits how
  /// often this reaches the server and the server mutes meanwhile.
  Future<void> scrub(int positionMs);

  /// The position handle is let go at `positionMs`; always reaches the server.
  Future<void> endScrub(int positionMs);

  /// The length of the recording in milliseconds; `null` while it has no scan.
  Future<int?> recordingLength(int recordingId);
  Future<void> setSpeed(double speed);
  Future<void> setTuning(double hertz);
  Future<void> setKey(int semitones);
  Future<void> setVolume(int percent);
  Future<void> setPlayMode(PlayMode mode);

  /// Plays the region from its start; what happens at its end depends on the
  /// play mode.
  Future<void> playRegion({required int startMs, required int endMs});

  /// Whether `hertz` lies in the range the server accepts for the tuning.
  Future<bool> tuningSupported(double hertz);

  /// The speed that plays a recording of `recordingBpm` at `targetBpm`.
  Future<TargetTempo> speedForTargetTempo({
    required double targetBpm,
    required double recordingBpm,
  });
}
