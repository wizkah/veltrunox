import '../features/player/player_source.dart';
import '../features/player/player_state.dart';
import '../features/player/target_tempo.dart';
import 'bridge_session.dart';
import 'generated/api.dart' as rust;

/// Adapter: the shared player of the Rust client core.
class BridgePlayerSource implements PlayerSource {
  BridgePlayerSource(this._session);

  final BridgeSession _session;

  @override
  Stream<PlayerState> watchState() => _session.player();

  @override
  Stream<String> rejections() => _session.rejections();

  @override
  Future<int?> position() async => (await rust.playbackPositionMs())?.toInt();

  @override
  Future<void> select(int recordingId) => _send(
    rust.PlayerCommand.selectRecording(recordingId: BigInt.from(recordingId)),
  );

  @override
  Future<void> play() => _send(const rust.PlayerCommand.play());

  @override
  Future<void> pause() => _send(const rust.PlayerCommand.pause());

  @override
  Future<void> seek(int positionMs) =>
      _send(rust.PlayerCommand.seek(positionMs: BigInt.from(positionMs)));

  @override
  Future<void> setSpeed(double speed) =>
      _send(rust.PlayerCommand.setSpeed(speed: speed));

  @override
  Future<void> setTuning(double hertz) =>
      _send(rust.PlayerCommand.setTuning(tuningHz: hertz));

  @override
  Future<void> setKey(int semitones) =>
      _send(rust.PlayerCommand.setKey(keyShiftSemitones: semitones));

  @override
  Future<TargetTempo> speedForTargetTempo({
    required double targetBpm,
    required double recordingBpm,
  }) async {
    final target = await rust.speedForTargetBpm(
      targetBpm: targetBpm,
      recordingBpm: recordingBpm,
    );
    return TargetTempo(speed: target.speed, supported: target.supported);
  }

  Future<void> _send(rust.PlayerCommand command) => _session.send(command);
}
