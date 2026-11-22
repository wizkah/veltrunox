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
  Stream<bool> audioOutput() => _session.audioOutput();

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
  Future<void> scrub(int positionMs) =>
      rust.scrubTo(positionMs: BigInt.from(positionMs));

  @override
  Future<void> endScrub(int positionMs) =>
      rust.endScrub(positionMs: BigInt.from(positionMs));

  @override
  Future<int?> recordingLength(int recordingId) async =>
      (await rust.recordingLengthMs(
        serverUrl: _session.serverUrl,
        recordingId: BigInt.from(recordingId),
      ))?.toInt();

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
  Future<void> setVolume(int percent) =>
      _send(rust.PlayerCommand.setVolume(volumePercent: percent));

  @override
  Future<void> setPlayMode(PlayMode mode) => _send(
    rust.PlayerCommand.setPlayMode(
      playMode: switch (mode) {
        PlayMode.toEnd => rust.PlayModeChoice.toEnd,
        PlayMode.toRegionEnd => rust.PlayModeChoice.toRegionEnd,
        PlayMode.loop => rust.PlayModeChoice.loop,
      },
    ),
  );

  @override
  Future<void> playRegion({required int startMs, required int endMs}) => _send(
    rust.PlayerCommand.playRegion(
      startMs: BigInt.from(startMs),
      endMs: BigInt.from(endMs),
    ),
  );

  @override
  Future<bool> tuningSupported(double hertz) =>
      rust.tuningSupported(tuningHz: hertz);

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
