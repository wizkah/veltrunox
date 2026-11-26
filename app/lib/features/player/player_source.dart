import 'player_state.dart';
import 'target_tempo.dart';
import '../../shared_ui/server_reason.dart';

/// Port: the shared player. The server plays; this only controls and shows it.
abstract interface class PlayerSource {
  Stream<PlayerState> watchState();

  /// Reasons why the server refused a command.
  Stream<ServerReason> rejections();

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

  Future<void> setSpeed(double speed);
  Future<void> setTuning(double hertz);
  Future<void> setKey(int semitones);
  Future<void> setVolume(int percent);
  Future<void> setPlayMode(PlayMode mode);

  /// Plays the stems of the recording, or the original.
  Future<void> setStemsEnabled(bool enabled);
  Future<void> setStemVolume(int stem, int percent);
  Future<void> setStemMuted(int stem, bool muted);
  Future<void> setStemSolo(int stem, bool solo);

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
