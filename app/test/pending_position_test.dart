import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:veltrunox_app/features/player/pending_position.dart';
import 'package:veltrunox_app/features/player/player_controller.dart';
import 'package:veltrunox_app/features/player/player_state.dart';

import 'fake_player_source.dart';

void main() {
  group('PendingPosition', () {
    late Duration now;
    late PendingPosition pending;

    setUp(() {
      now = Duration.zero;
      pending = PendingPosition(clock: () => now);
    });

    test('passes the position of the server on when nothing was asked', () {
      expect(pending.shown(1200, running: true, speed: 1), 1200);
      expect(pending.shown(null, running: false, speed: 1), isNull);
    });

    test('shows the asked position while the server still reports the old', () {
      pending.expect(60000);
      expect(pending.shown(1200, running: false, speed: 1), 60000);
      now += const Duration(milliseconds: 300);
      expect(pending.shown(1200, running: false, speed: 1), 60000);
    });

    test('moves the asked position on with the playing recording', () {
      pending.expect(60000);
      now += const Duration(milliseconds: 400);
      expect(pending.shown(1200, running: true, speed: 1.5), 60600);
    });

    test('hands over to the server once it has caught up', () {
      pending.expect(60000);
      expect(pending.shown(60120, running: true, speed: 1), 60120);
      // Asked is done: the server's value is passed on from here on.
      expect(pending.shown(1200, running: true, speed: 1), 1200);
    });

    test('gives up after the timeout', () {
      pending.expect(60000);
      now += const Duration(milliseconds: 1501);
      expect(pending.shown(1200, running: false, speed: 1), 1200);
      expect(pending.shown(1200, running: false, speed: 1), 1200);
    });

    test('a new action replaces the earlier one', () {
      pending.expect(60000);
      now += const Duration(milliseconds: 100);
      pending.expect(90000);
      expect(pending.shown(1200, running: false, speed: 1), 90000);
    });
  });

  test(
    'the position reading shows a seek at once, before the server answers',
    () async {
      final source = FakePlayerSource()..currentPosition = 1200;
      final container = ProviderContainer(
        overrides: [playerSourceProvider.overrideWithValue(source)],
      );
      addTearDown(container.dispose);
      addTearDown(source.states.close);
      container.listen(playerStateProvider, (_, _) {});
      source.states.add(
        const PlayerState(
          recordingId: 1,
          running: false,
          speed: 1,
          tuningHz: 440,
          keyShiftSemitones: 0,
          volumePercent: 100,
        ),
      );
      final readings = <int?>[];
      container.listen(
        playerPositionProvider,
        (_, next) => readings.add(next.value),
      );
      await Future<void>.delayed(const Duration(milliseconds: 20));
      expect(readings.last, 1200);

      await container.read(playerControllerProvider).seek(60000);
      await Future<void>.delayed(const Duration(milliseconds: 250));
      expect(readings.last, 60000);

      source.currentPosition = 60000;
      await Future<void>.delayed(const Duration(milliseconds: 250));
      expect(readings.last, 60000);
      source.currentPosition = 61000;
      await Future<void>.delayed(const Duration(milliseconds: 250));
      expect(readings.last, 61000);
    },
  );
}
