import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/features/player/player_bar.dart';
import 'package:veltrunox_app/features/player/player_controller.dart';
import 'package:veltrunox_app/features/player/player_state.dart';
import 'package:veltrunox_app/features/player/position_display.dart';

import 'fake_player_source.dart';

const _loaded = PlayerState(
  recordingId: 1,
  running: false,
  speed: 1,
  tuningHz: 440,
  keyShiftSemitones: 0,
);

Future<FakePlayerSource> _pumpBar(
  WidgetTester tester, {
  PlayerState? state,
}) async {
  final player = FakePlayerSource();
  await tester.pumpWidget(
    ProviderScope(
      overrides: [playerSourceProvider.overrideWithValue(player)],
      child: const MaterialApp(home: Scaffold(body: PlayerBar())),
    ),
  );
  if (state != null) player.states.add(state);
  await tester.pump();
  await tester.pump();
  return player;
}

void main() {
  testWidgets('invites to choose a recording while none is loaded', (
    tester,
  ) async {
    await _pumpBar(tester);

    expect(find.text('Keine Aufnahme gewählt'), findsOneWidget);
    expect(find.byIcon(Icons.play_arrow), findsNothing);
  });

  testWidgets('play and pause follow the shared state', (tester) async {
    final player = await _pumpBar(tester, state: _loaded);

    await tester.tap(find.byIcon(Icons.play_arrow));
    expect(player.calls, ['play']);

    player.states.add(
      const PlayerState(
        recordingId: 1,
        running: true,
        speed: 1,
        tuningHz: 440,
        keyShiftSemitones: 0,
      ),
    );
    await tester.pump();
    await tester.pump();
    await tester.tap(find.byIcon(Icons.pause));
    expect(player.calls, ['play', 'pause']);
  });

  testWidgets('restart seeks to the beginning', (tester) async {
    final player = await _pumpBar(tester, state: _loaded);

    await tester.tap(find.byIcon(Icons.skip_previous));

    expect(player.calls, ['seek 0']);
  });

  testWidgets('the key buttons shift by one semitone', (tester) async {
    final player = await _pumpBar(tester, state: _loaded);

    await tester.tap(find.byIcon(Icons.add));
    await tester.tap(find.byIcon(Icons.remove));

    expect(player.calls, ['key 1', 'key -1']);
  });

  testWidgets('shows the settings of the shared state', (tester) async {
    await _pumpBar(
      tester,
      state: const PlayerState(
        recordingId: 1,
        running: false,
        speed: 0.75,
        tuningHz: 432,
        keyShiftSemitones: -2,
      ),
    );

    expect(find.text('0.75×'), findsOneWidget);
    expect(find.text('432 Hz'), findsOneWidget);
    expect(find.text('-2'), findsOneWidget);
  });

  testWidgets('shows why the server refused a command', (tester) async {
    final player = await _pumpBar(tester, state: _loaded);

    player.refusals.add('speed must be between 0.5 and 2.0');
    await tester.pump();
    await tester.pump();

    expect(find.text('speed must be between 0.5 and 2.0'), findsOneWidget);
  });

  test('positions are shown as minutes, seconds and tenths', () {
    expect(formatPosition(0), '0:00.0');
    expect(formatPosition(83_456), '1:23.4');
  });
}
