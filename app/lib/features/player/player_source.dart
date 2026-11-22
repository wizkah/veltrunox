import 'player_state.dart';
import 'target_tempo.dart';

/// Port: the shared player. The server plays; this only controls and shows it.
abstract interface class PlayerSource {
  Stream<PlayerState> watchState();

  /// Reasons why the server refused a command.
  Stream<String> rejections();

  /// The playback position now in milliseconds; `null` before the first state.
  Future<int?> position();

  Future<void> select(int recordingId);
  Future<void> play();
  Future<void> pause();
  Future<void> seek(int positionMs);
  Future<void> setSpeed(double speed);
  Future<void> setTuning(double hertz);
  Future<void> setKey(int semitones);
  Future<void> setVolume(int percent);

  /// The speed that plays a recording of `recordingBpm` at `targetBpm`.
  Future<TargetTempo> speedForTargetTempo({
    required double targetBpm,
    required double recordingBpm,
  });
}
