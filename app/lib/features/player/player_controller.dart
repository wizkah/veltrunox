import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'player_source.dart';
import 'remote_change_detector.dart';
import 'player_state.dart';
import 'position_display.dart';
import 'target_tempo.dart';

/// Supplies the [PlayerSource]; overridden at the application root.
final playerSourceProvider = Provider<PlayerSource>(
  (ref) => throw UnimplementedError('playerSourceProvider must be overridden'),
);

/// The shared playback state; absent until the server has sent it.
final playerStateProvider = StreamProvider<PlayerState>(
  (ref) => ref.watch(playerSourceProvider).watchState(),
);

/// The playback position, read again exactly when the displayed tenth changes
/// while playing (a fixed poll would show the tenths unevenly); while paused it
/// is read at intervals to follow a jump.
final playerPositionProvider = StreamProvider<int?>((ref) {
  final source = ref.watch(playerSourceProvider);
  // Only what changes the timing; a changed volume must not restart the reading.
  final (recording, running, speed) = ref.watch(
    playerStateProvider.select(
      (state) => (
        state.value?.recordingId,
        state.value?.running ?? false,
        state.value?.speed ?? 1.0,
      ),
    ),
  );
  final controller = StreamController<int?>();
  if (recording == null) {
    controller.add(null);
    ref.onDispose(controller.close);
    return controller.stream;
  }
  Timer? timer;
  var disposed = false;
  Future<void> read() async {
    final position = await source.position();
    if (disposed) return;
    controller.add(position);
    final wait = running && position != null
        ? untilNextTenth(position, speed)
        : const Duration(milliseconds: 100);
    timer = Timer(wait, read);
  }

  unawaited(read());
  ref.onDispose(() {
    disposed = true;
    timer?.cancel();
    unawaited(controller.close());
  });
  return controller.stream;
});

/// Refusals of the server, for display.
final playerRejectionProvider = StreamProvider<String>(
  (ref) => ref.watch(playerSourceProvider).rejections(),
);

/// Whether the server can play sound; `false` shows a notice.
final audioOutputProvider = StreamProvider<bool>(
  (ref) => ref.watch(playerSourceProvider).audioOutput(),
);

/// Turns user actions into commands for the shared player.
final playerControllerProvider = Provider<PlayerController>(
  (ref) => PlayerController(
    ref.watch(playerSourceProvider),
    ref.watch(remoteChangeDetectorProvider),
  ),
);

class PlayerController {
  PlayerController(this._source, this._changes);

  final PlayerSource _source;

  /// Told about every change of position or recording made here, so it is not
  /// reported as a change by another client.
  final RemoteChangeDetector _changes;

  Future<void> select(int recordingId) {
    _changes.ownAction();
    return _source.select(recordingId);
  }

  /// Loads the recording and plays it from the start.
  Future<void> selectAndPlay(int recordingId) async {
    await select(recordingId);
    await _source.play();
  }

  Future<void> play() => _source.play();

  Future<void> togglePlay({required bool running}) =>
      running ? _source.pause() : _source.play();

  Future<void> restart() {
    _changes.ownAction();
    return _source.seek(0);
  }

  /// Moves the position without starting playback.
  Future<void> seek(int positionMs) {
    _changes.ownAction();
    return _source.seek(positionMs);
  }

  /// Jumps to `positionMs` and plays on to the end of the recording.
  Future<void> playFrom(int positionMs) async {
    _changes.ownAction();
    await _source.seek(positionMs);
    await _source.play();
  }

  /// Plays the region from its start; the play mode decides what follows its end.
  Future<void> playRegion({required int startMs, required int endMs}) {
    _changes.ownAction();
    return _source.playRegion(startMs: startMs, endMs: endMs);
  }

  Future<void> setPlayMode(PlayMode mode) => _source.setPlayMode(mode);
  Future<void> setSpeed(double speed) => _source.setSpeed(speed);

  Future<void> scrub(int positionMs) {
    _changes.ownAction();
    return _source.scrub(positionMs);
  }

  Future<void> endScrub(int positionMs) {
    _changes.ownAction();
    return _source.endScrub(positionMs);
  }

  /// Whether the recording was stopped and plays only while the handle rests.
  bool _playsOnlyWhileHeld = false;

  /// The handle rests at `positionMs`: the recording plays on from there, also
  /// if it was stopped; letting go ([endHold]) stops it again.
  Future<void> hold(int positionMs, {required bool running}) async {
    _playsOnlyWhileHeld = !running;
    _changes.ownAction();
    await _source.endScrub(positionMs);
    if (_playsOnlyWhileHeld) await _source.play();
  }

  /// The handle is let go after [hold].
  Future<void> endHold() async {
    if (!_playsOnlyWhileHeld) return;
    _playsOnlyWhileHeld = false;
    await _source.pause();
  }

  /// Sets the tuning. `false` if it is outside the supported range.
  Future<bool> setTuning(double hertz) async {
    if (!await _source.tuningSupported(hertz)) return false;
    await _source.setTuning(hertz);
    return true;
  }

  Future<void> setKey(int semitones) => _source.setKey(semitones);
  Future<void> setVolume(int percent) => _source.setVolume(percent);

  /// Sets the speed that plays the recording at `targetBpm`. `false` if that
  /// speed is outside the supported range or the tempo is not usable.
  Future<bool> setTargetTempo({
    required double targetBpm,
    required double recordingBpm,
  }) async {
    final TargetTempo target;
    try {
      target = await _source.speedForTargetTempo(
        targetBpm: targetBpm,
        recordingBpm: recordingBpm,
      );
    } on Object {
      return false;
    }
    if (!target.supported) return false;
    await _source.setSpeed(target.speed);
    return true;
  }
}
