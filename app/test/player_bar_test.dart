import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/features/player/player_analysis.dart';
import 'package:veltrunox_app/features/player/player_bar.dart';
import 'package:veltrunox_app/features/player/player_controller.dart';
import 'package:veltrunox_app/features/player/player_state.dart';
import 'package:veltrunox_app/features/player/position_display.dart';
import 'package:veltrunox_app/features/player/tempo_input.dart';

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
  PlayerAnalysis? analysis,
}) async {
  final player = FakePlayerSource();
  await tester.pumpWidget(
    ProviderScope(
      overrides: [playerSourceProvider.overrideWithValue(player)],
      child: MaterialApp(
        home: Scaffold(body: PlayerBar(analysis: analysis)),
      ),
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

  group('with an analysis', () {
    const analysis = PlayerAnalysis(tuningHz: 443, tempoBpm: 120);
    const changed = PlayerState(
      recordingId: 1,
      running: false,
      speed: 0.75,
      tuningHz: 440,
      keyShiftSemitones: 2,
    );

    testWidgets('shows the tempo the recording plays at', (tester) async {
      await _pumpBar(tester, state: changed, analysis: analysis);

      expect(find.text('90,0 bpm'), findsOneWidget);
    });

    testWidgets('has nothing to reset without an analysis', (tester) async {
      await _pumpBar(tester, state: changed);

      expect(find.byIcon(Icons.restart_alt), findsNothing);
      expect(find.text('Soll-bpm'), findsNothing);
    });

    testWidgets('the reset buttons restore the analysed values', (
      tester,
    ) async {
      final player = await _pumpBar(tester, state: changed, analysis: analysis);
      final resets = find.byIcon(Icons.restart_alt);
      expect(resets, findsNWidgets(3));

      await tester.tap(resets.at(0));
      await tester.tap(resets.at(1));
      await tester.tap(resets.at(2));

      expect(player.calls, ['speed 1.0', 'tuning 443.0', 'key 0']);
    });

    testWidgets('a setting that already is as analysed cannot be reset', (
      tester,
    ) async {
      await _pumpBar(
        tester,
        state: const PlayerState(
          recordingId: 1,
          running: false,
          speed: 1,
          tuningHz: 443,
          keyShiftSemitones: 0,
        ),
        analysis: analysis,
      );

      final buttons = tester.widgetList<IconButton>(
        find.widgetWithIcon(IconButton, Icons.restart_alt),
      );
      expect(buttons.every((button) => button.onPressed == null), isTrue);
    });

    testWidgets('a wanted tempo sets the speed', (tester) async {
      final player = await _pumpBar(tester, state: changed, analysis: analysis);

      await tester.enterText(find.byType(TextField), '96');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pump();
      await tester.pump();

      expect(player.calls, ['speed 0.8']);
    });

    testWidgets('a tempo outside the speed range is refused', (tester) async {
      final player = await _pumpBar(tester, state: changed, analysis: analysis);

      await tester.enterText(find.byType(TextField), '300');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pump();
      await tester.pump();

      expect(player.calls, isEmpty);
      expect(find.text('Außerhalb von 0,5× bis 2×'), findsOneWidget);
    });

    testWidgets('text that is no tempo is not sent', (tester) async {
      final player = await _pumpBar(tester, state: changed, analysis: analysis);

      await tester.enterText(find.byType(TextField), 'schnell');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pump();

      expect(player.calls, isEmpty);
      expect(find.text('Keine Zahl'), findsOneWidget);
    });
  });

  test('tempo input accepts a decimal comma and rejects the rest', () {
    expect(parseBpm('96'), 96);
    expect(parseBpm(' 96,5 '), 96.5);
    expect(parseBpm('96.5'), 96.5);
    expect(parseBpm('0'), isNull);
    expect(parseBpm('-3'), isNull);
    expect(parseBpm('abc'), isNull);
    expect(parseBpm(''), isNull);
  });

  test('positions are shown as minutes, seconds and tenths', () {
    expect(formatPosition(0), '0:00.0');
    expect(formatPosition(83_456), '1:23.4');
  });
}
