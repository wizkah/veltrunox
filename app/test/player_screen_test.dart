import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/features/player/player_analysis.dart';
import 'package:veltrunox_app/features/player/player_notices.dart';
import 'package:veltrunox_app/features/player/player_screen.dart';
import 'package:veltrunox_app/features/player/player_controller.dart';
import 'package:veltrunox_app/features/player/player_state.dart';
import 'package:veltrunox_app/features/player/position_slider.dart';
import 'package:veltrunox_app/features/player/tempo_controls.dart';
import 'package:veltrunox_app/features/player/volume_control.dart';
import 'package:veltrunox_app/features/player/number_input.dart';
import 'package:veltrunox_app/features/player/position_display.dart';

import 'fake_player_source.dart';

const _loaded = PlayerState(
  recordingId: 1,
  running: false,
  speed: 1,
  tuningHz: 440,
  keyShiftSemitones: 0,
  volumePercent: 100,
);

/// The player scrolls on a small screen; the tests need all controls in view.
void _useTallScreen(WidgetTester tester) {
  tester.view.physicalSize = const Size(800, 1600);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
}

Future<FakePlayerSource> _pumpBar(
  WidgetTester tester, {
  PlayerState? state,
  PlayerAnalysis? analysis,
}) async {
  _useTallScreen(tester);
  final player = FakePlayerSource();
  await tester.pumpWidget(
    ProviderScope(
      overrides: [playerSourceProvider.overrideWithValue(player)],
      child: MaterialApp(
        home: Scaffold(
          body: PlayerNotices(
            child: PlayerScreen(onChooseRecording: () {}, analysis: analysis),
          ),
        ),
      ),
    ),
  );
  if (state != null) player.states.add(state);
  await tester.pump();
  await tester.pump();
  return player;
}

/// The slider known by the given name ('Position', 'Geschwindigkeit' or
/// 'Lautstärke'); the sliders show no label of their own.
Finder _sliderOf(String name) => find.descendant(
  of: find.byKey(switch (name) {
    'Position' => positionSliderKey,
    'Geschwindigkeit' => speedSliderKey,
    _ => volumeSliderKey,
  }),
  matching: find.byType(Slider),
);

Future<void> _enter(WidgetTester tester, Finder field, String text) async {
  await tester.enterText(field, text);
  await tester.testTextInput.receiveAction(TextInputAction.done);
  await tester.pump();
  await tester.pump();
}

const _analysis = PlayerAnalysis(tuningHz: 443, tempoBpm: 120);

void main() {
  testWidgets('invites to choose a recording while none is loaded', (
    tester,
  ) async {
    await _pumpBar(
      tester,
      state: const PlayerState(
        recordingId: null,
        running: false,
        speed: 1,
        tuningHz: 440,
        keyShiftSemitones: 0,
        volumePercent: 100,
      ),
    );

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
        volumePercent: 100,
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

  testWidgets('the key is chosen from -12 to +12 semitones', (tester) async {
    final player = await _pumpBar(tester, state: _loaded);

    await tester.tap(find.byType(DropdownButton<int>));
    await tester.pumpAndSettle();
    expect(find.text('+12'), findsOneWidget);
    expect(find.text('-12'), findsOneWidget);
    await tester.tap(find.text('+1').last);
    await tester.pumpAndSettle();

    expect(player.calls, ['key 1']);
  });

  testWidgets('the key steps are named after the analysed key', (tester) async {
    await _pumpBar(
      tester,
      state: _loaded,
      analysis: const PlayerAnalysis(tuningHz: 440, tempoBpm: 120, keyTonic: 4),
    );

    expect(find.text('E-Dur (Original)'), findsOneWidget);
    expect(find.text('Original: 440,0 Hz · E-Dur'), findsOneWidget);
    await tester.tap(find.byType(DropdownButton<int>));
    await tester.pumpAndSettle();
    expect(find.text('F-Dur (+1)'), findsOneWidget);
    expect(find.text('E-Moll (+0)'), findsNothing);
    expect(find.text('E-Dur (-12)'), findsOneWidget);
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
        volumePercent: 100,
      ),
    );

    expect(find.text('0.75×'), findsOneWidget);
    expect(find.text('432,0'), findsOneWidget);
    expect(find.text('-2'), findsOneWidget);
  });

  testWidgets('shows why the server refused a command', (tester) async {
    final player = await _pumpBar(tester, state: _loaded);

    player.refusals.add('speed must be between 0.5 and 2.0');
    await tester.pump();
    await tester.pump();

    expect(
      find.text('Die Geschwindigkeit muss zwischen 0,5 und 2,0 liegen.'),
      findsOneWidget,
    );
  });

  group('with an analysis', () {
    const analysis = PlayerAnalysis(tuningHz: 443, tempoBpm: 120);
    const changed = PlayerState(
      recordingId: 1,
      running: false,
      speed: 0.75,
      tuningHz: 440,
      keyShiftSemitones: 2,
      volumePercent: 100,
    );

    testWidgets('shows the tempo the recording plays at', (tester) async {
      await _pumpBar(tester, state: changed, analysis: analysis);

      expect(find.text('90,0'), findsOneWidget);
    });

    testWidgets(
      'without an analysis only the analysed values cannot be reset',
      (tester) async {
        final player = await _pumpBar(tester, state: changed);

        final resets = find.widgetWithIcon(IconButton, Icons.restart_alt);
        expect(resets, findsNWidgets(3));
        final enabled = tester
            .widgetList<IconButton>(resets)
            .map((button) => button.onPressed != null)
            .toList();
        // Speed and key go back to 1× and 0; the tuning has no analysed value.
        expect(enabled, [true, false, true]);
        // The wanted tempo is shown but cannot be typed without a tempo.
        expect(find.text('Ziel-bpm'), findsOneWidget);
        expect(find.text('–'), findsWidgets);
        await tester.tap(find.byIcon(Icons.restart_alt).first);
        await tester.tap(find.byIcon(Icons.restart_alt).last);
        expect(player.calls, ['speed 1.0', 'key 0']);
      },
    );

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
          volumePercent: 100,
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

      await tester.enterText(find.byType(TextField).first, '96');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pump();
      await tester.pump();

      expect(player.calls, ['speed 0.8']);
    });

    testWidgets('a tempo outside the speed range is refused', (tester) async {
      final player = await _pumpBar(tester, state: changed, analysis: analysis);

      await tester.enterText(find.byType(TextField).first, '300');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pump();
      await tester.pump();

      expect(player.calls, isEmpty);
      expect(find.text('Außerhalb von 0,5× bis 2×'), findsOneWidget);
    });

    testWidgets('text that is no tempo is not sent', (tester) async {
      final player = await _pumpBar(tester, state: changed, analysis: analysis);

      await tester.enterText(find.byType(TextField).first, 'schnell');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pump();

      expect(player.calls, isEmpty);
      expect(find.text('Keine Zahl'), findsOneWidget);
    });
  });

  test('tempo input accepts a decimal comma and rejects the rest', () {
    expect(parsePositiveNumber('96'), 96);
    expect(parsePositiveNumber(' 96,5 '), 96.5);
    expect(parsePositiveNumber('96.5'), 96.5);
    expect(parsePositiveNumber('0'), isNull);
    expect(parsePositiveNumber('-3'), isNull);
    expect(parsePositiveNumber('abc'), isNull);
    expect(parsePositiveNumber(''), isNull);
  });

  test('positions are shown as minutes, seconds and tenths', () {
    expect(formatPosition(0), '0:00.0');
    expect(formatPosition(83_456), '1:23.4');
  });

  testWidgets('the volume slider shows the shared volume and sends changes', (
    tester,
  ) async {
    final player = await _pumpBar(tester, state: _loaded);

    expect(find.text('Lautstärke'), findsOneWidget);
    expect(find.text('100 %'), findsOneWidget);

    player.states.add(
      const PlayerState(
        recordingId: 1,
        running: false,
        speed: 1,
        tuningHz: 440,
        keyShiftSemitones: 0,
        volumePercent: 40,
      ),
    );
    await tester.pump();
    await tester.pump();
    expect(find.text('40 %'), findsOneWidget);

    await tester.drag(_sliderOf('Lautstärke'), const Offset(-60, 0));
    await tester.pump();
    expect(
      player.calls.where((call) => call.startsWith('volume ')),
      isNotEmpty,
    );
  });

  group('position slider', () {
    testWidgets('follows the playback position and spans the length', (
      tester,
    ) async {
      _useTallScreen(tester);
      final player = FakePlayerSource()..currentPosition = 30000;
      await tester.pumpWidget(
        ProviderScope(
          overrides: [playerSourceProvider.overrideWithValue(player)],
          child: MaterialApp(
            home: Scaffold(
              body: PlayerNotices(
                child: PlayerScreen(
                  onChooseRecording: () {},
                  analysis: _analysis,
                ),
              ),
            ),
          ),
        ),
      );
      player.states.add(_loaded);
      await tester.pump();
      await tester.pump();

      final slider = tester.widget<Slider>(_sliderOf('Position'));
      expect(slider.value, 30000);
      expect(slider.max, 180000);
      expect(find.text('0:30.0'), findsWidgets);
    });

    testWidgets('is shown disabled for a recording without a scan', (
      tester,
    ) async {
      final player = await _pumpBar(tester, state: _loaded);

      expect(find.byKey(positionSliderKey), findsOneWidget);
      expect(tester.widget<Slider>(_sliderOf('Position')).onChanged, isNull);
      expect(player.lengthRequests, isEmpty);
    });

    testWidgets('scrubs while dragged and ends with the final position', (
      tester,
    ) async {
      final player = await _pumpBar(
        tester,
        state: _loaded,
        analysis: _analysis,
      );

      final gesture = await tester.startGesture(
        tester.getCenter(_sliderOf('Position')),
      );
      await gesture.moveBy(const Offset(40, 0));
      await tester.pump();
      expect(
        player.calls.where((call) => call.startsWith('scrub ')),
        isNotEmpty,
      );
      expect(
        player.calls.where((call) => call.startsWith('endScrub')),
        isEmpty,
      );

      await gesture.up();
      await tester.pump();
      expect(player.calls.last, startsWith('endScrub '));
    });
  });

  group('position slider holding still', () {
    testWidgets(
      'lets a running recording play on and reports nothing more on release',
      (tester) async {
        final player = await _pumpBar(
          tester,
          state: const PlayerState(
            recordingId: 1,
            running: true,
            speed: 1,
            tuningHz: 440,
            keyShiftSemitones: 0,
            volumePercent: 100,
          ),
          analysis: _analysis,
        );
        final gesture = await tester.startGesture(
          tester.getCenter(_sliderOf('Position')),
        );
        await gesture.moveBy(const Offset(40, 0));
        await tester.pump();
        expect(player.calls.where((c) => c.startsWith('endScrub')), isEmpty);

        await tester.pump(const Duration(milliseconds: 200));
        expect(
          player.calls.where((c) => c.startsWith('endScrub')),
          hasLength(1),
        );

        await gesture.up();
        await tester.pump();
        expect(
          player.calls.where((c) => c.startsWith('endScrub')),
          hasLength(1),
        );
        expect(player.calls, isNot(contains('play')));
        expect(player.calls, isNot(contains('pause')));
      },
    );

    testWidgets(
      'plays a stopped recording while held and stops it on release',
      (tester) async {
        final player = await _pumpBar(
          tester,
          state: _loaded,
          analysis: _analysis,
        );
        final gesture = await tester.startGesture(
          tester.getCenter(_sliderOf('Position')),
        );
        await gesture.moveBy(const Offset(40, 0));
        await tester.pump(const Duration(milliseconds: 200));
        expect(
          player.calls.where((c) => c.startsWith('endScrub')),
          hasLength(1),
        );
        expect(player.calls.last, 'play');

        await gesture.up();
        await tester.pump();
        expect(player.calls.last, 'pause');
      },
    );

    testWidgets('moving again after the rest scrubs again', (tester) async {
      final player = await _pumpBar(
        tester,
        state: _loaded,
        analysis: _analysis,
      );
      final gesture = await tester.startGesture(
        tester.getCenter(_sliderOf('Position')),
      );
      await gesture.moveBy(const Offset(40, 0));
      await tester.pump(const Duration(milliseconds: 200));
      final before = player.calls.where((c) => c.startsWith('scrub ')).length;
      await gesture.moveBy(const Offset(40, 0));
      await tester.pump();
      expect(
        player.calls.where((c) => c.startsWith('scrub ')).length,
        greaterThan(before),
      );
      await gesture.up();
      await tester.pump();
      expect(player.calls.last, startsWith('endScrub '));
    });
  });

  group('speed slider', () {
    const analysis = PlayerAnalysis(tuningHz: 443, tempoBpm: 120);

    testWidgets('sends the speed while dragged and shows the wanted tempo', (
      tester,
    ) async {
      final player = await _pumpBar(tester, state: _loaded, analysis: analysis);
      expect(find.text('120,0'), findsOneWidget);

      final gesture = await tester.startGesture(
        tester.getCenter(_sliderOf('Geschwindigkeit')),
      );
      await gesture.moveBy(const Offset(60, 0));
      await tester.pump();

      expect(
        player.calls.where((call) => call.startsWith('speed ')),
        isNotEmpty,
      );
      // The state still says 1×, yet the tempo already follows the handle.
      expect(find.text('120,0'), findsNothing);
      final shown = tester
          .widget<TextField>(find.byType(TextField).first)
          .controller!
          .text;
      expect(shown, isNot('120,0'));

      await gesture.up();
      await tester.pump(const Duration(milliseconds: 100));
      final sent = player.calls.where((call) => call.startsWith('speed '));
      expect(sent.last, isNot(sent.first));
    });
  });

  group('tuning field', () {
    const analysis = PlayerAnalysis(tuningHz: 443, tempoBpm: 120);

    testWidgets('has no slider and shows the shared tuning', (tester) async {
      await _pumpBar(tester, state: _loaded);

      // Position, volume and speed are sliders; the tuning is typed in.
      expect(find.byType(Slider), findsNWidgets(3));
      expect(find.text('440,0'), findsOneWidget);
    });

    testWidgets('the one field follows the shared tuning', (tester) async {
      final player = await _pumpBar(tester, state: _loaded);
      expect(find.text('440,0'), findsOneWidget);

      player.states.add(
        const PlayerState(
          recordingId: 1,
          running: false,
          speed: 1,
          tuningHz: 432.5,
          keyShiftSemitones: 0,
          volumePercent: 100,
        ),
      );
      await tester.pump();
      await tester.pump();

      expect(find.text('432,5'), findsOneWidget);
    });

    testWidgets('typed text that is not handed on is put back', (tester) async {
      await _pumpBar(tester, state: _loaded);

      await tester.enterText(find.byType(TextField).last, '4');
      FocusManager.instance.primaryFocus?.unfocus();
      await tester.pump();

      expect(find.text('440,0'), findsOneWidget);
    });

    testWidgets('a confirmed value is sent', (tester) async {
      final player = await _pumpBar(tester, state: _loaded, analysis: analysis);

      await _enter(tester, find.byType(TextField).last, '432,5');

      expect(player.calls, ['tuning 432.5']);
    });

    testWidgets('a value the core refuses shows a message', (tester) async {
      final player = await _pumpBar(tester, state: _loaded, analysis: analysis);

      await _enter(tester, find.byType(TextField).last, '600');

      expect(player.calls, isEmpty);
      expect(find.text('Nur 300–500 Hz erlaubt'), findsOneWidget);
    });

    testWidgets('text that is no number is not sent', (tester) async {
      final player = await _pumpBar(tester, state: _loaded, analysis: analysis);

      await _enter(tester, find.byType(TextField).last, 'tief');

      expect(player.calls, isEmpty);
      expect(find.text('Keine Zahl'), findsOneWidget);
    });
  });

  group('layout by width', () {
    const analysis = PlayerAnalysis(tuningHz: 443, tempoBpm: 120, keyTonic: 4);

    Future<void> pumpAt(WidgetTester tester, double width) async {
      await _pumpBar(tester, state: _loaded, analysis: analysis);
      tester.view.physicalSize = Size(width, 1600);
      await tester.pump();
    }

    for (final width in [360.0, 800.0, 1000.0, 1400.0]) {
      testWidgets('shows all controls and the resets at $width px', (
        tester,
      ) async {
        await pumpAt(tester, width);

        expect(find.byIcon(Icons.restart_alt), findsNWidgets(3));
        for (var i = 0; i < 3; i++) {
          final right = tester
              .getTopRight(find.byIcon(Icons.restart_alt).at(i))
              .dx;
          expect(right, lessThanOrEqualTo(width));
        }
        expect(find.text('Lautstärke'), findsOneWidget);
      });
    }

    testWidgets('the volume stands upright beside the transport when wide', (
      tester,
    ) async {
      await pumpAt(tester, 1400);
      expect(find.byType(RotatedBox), findsWidgets);

      await pumpAt(tester, 600);
      expect(
        find.descendant(
          of: find.byType(PlayerScreen),
          matching: find.byType(RotatedBox),
        ),
        findsNothing,
      );
    });
  });
}
