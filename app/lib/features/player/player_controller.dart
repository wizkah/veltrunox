import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'player_source.dart';
import 'player_state.dart';

/// Supplies the [PlayerSource]; overridden at the application root.
final playerSourceProvider = Provider<PlayerSource>(
  (ref) => throw UnimplementedError('playerSourceProvider must be overridden'),
);

/// The shared playback state; absent until the server has sent it.
final playerStateProvider = StreamProvider<PlayerState>(
  (ref) => ref.watch(playerSourceProvider).watchState(),
);

/// The playback position, refreshed while a recording is loaded.
final playerPositionProvider = StreamProvider<int?>((ref) async* {
  final source = ref.watch(playerSourceProvider);
  final recording = ref.watch(
    playerStateProvider.select((state) => state.value?.recordingId),
  );
  if (recording == null) {
    yield null;
    return;
  }
  yield await source.position();
  // `yield*` hands cancellation straight to the timer when the provider is disposed.
  yield* Stream<void>.periodic(const Duration(milliseconds: 100))
      .asyncMap((_) => source.position());
});

/// Refusals of the server, for display.
final playerRejectionProvider = StreamProvider<String>(
  (ref) => ref.watch(playerSourceProvider).rejections(),
);

/// Turns user actions into commands for the shared player.
final playerControllerProvider = Provider<PlayerController>(
  (ref) => PlayerController(ref.watch(playerSourceProvider)),
);

class PlayerController {
  PlayerController(this._source);

  final PlayerSource _source;

  Future<void> select(int recordingId) => _source.select(recordingId);
  Future<void> togglePlay({required bool running}) =>
      running ? _source.pause() : _source.play();
  Future<void> restart() => _source.seek(0);
  Future<void> setSpeed(double speed) => _source.setSpeed(speed);
  Future<void> setTuning(double hertz) => _source.setTuning(hertz);
  Future<void> setKey(int semitones) => _source.setKey(semitones);
}
