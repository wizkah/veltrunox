import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/app.dart';
import 'package:veltrunox_app/app_shell/shell_destination.dart';
import 'package:veltrunox_app/features/connection/connection_controller.dart';
import 'package:veltrunox_app/features/connection/connection_source.dart';
import 'package:veltrunox_app/features/connection/connection_status.dart';
import 'package:veltrunox_app/features/library/library_controller.dart';
import 'package:veltrunox_app/features/library/recording_source.dart';
import 'package:veltrunox_app/features/library/recording_summary.dart';
import 'package:veltrunox_app/features/markers/marker.dart';
import 'package:veltrunox_app/features/markers/marker_controller.dart';
import 'package:veltrunox_app/features/player/player_controller.dart';
import 'package:veltrunox_app/features/player/player_state.dart';

import 'fake_marker_source.dart';
import 'fake_access_source.dart';
import 'fake_player_source.dart';

class _Recordings implements RecordingSource {
  @override
  Future<List<RecordingSummary>> listRecordings() async => const [
    RecordingSummary(id: 1, title: 'Soundcheck'),
  ];

  @override
  Stream<int> analysisChanges() => const Stream.empty();

  @override
  Future<void> reanalyse(int recordingId) async {}
}

class _Connection implements ConnectionSource {
  @override
  Stream<ConnectionStatus> watch() => Stream.value(ConnectionStatus.connected);

  @override
  Future<void> reconnect() async {}
}

PlayerState _state({
  PlayMode mode = PlayMode.toEnd,
  int position = 0,
  bool running = false,
  int? recording = 1,
}) => PlayerState(
  recordingId: recording,
  running: running,
  speed: 1,
  tuningHz: 440,
  keyShiftSemitones: 0,
  volumePercent: 100,
  positionMs: position,
  playMode: mode,
);

class _App {
  _App(this.player, this.markers);

  final FakePlayerSource player;
  final FakeMarkerSource markers;
}

Future<_App> _pump(WidgetTester tester, {double width = 1400}) async {
  tester.view.physicalSize = Size(width, 1000);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final player = FakePlayerSource();
  final markers = FakeMarkerSource([
    const LabelMarker(id: 1, version: 1, title: 'Einsatz', positionMs: 25000),
    const RegionMarker(
      id: 2,
      version: 1,
      title: 'Strophe',
      startMs: 10000,
      endMs: 67000,
    ),
  ]);
  final container = ProviderContainer(
    overrides: [
      playerSourceProvider.overrideWithValue(player),
      recordingSourceProvider.overrideWithValue(_Recordings()),
      markerSourceProvider.overrideWithValue(markers),
      ...signedInAs(),
      connectionSourceProvider.overrideWithValue(_Connection()),
    ],
  );
  addTearDown(container.dispose);
  container
      .read(shellDestinationProvider.notifier)
      .select(ShellDestination.player);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: const VeltrunoxApp(),
    ),
  );
  // The sign-in passes first; the app listens to the player only after it.
  await tester.pumpAndSettle();
  player.states.add(_state());
  await tester.pumpAndSettle();
  return _App(player, markers);
}

void main() {
  testWidgets('without a recording the player says what to do', (tester) async {
    tester.view.physicalSize = const Size(800, 1000);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    final player = FakePlayerSource();
    final container = ProviderContainer(
      overrides: [
        playerSourceProvider.overrideWithValue(player),
        recordingSourceProvider.overrideWithValue(_Recordings()),
        markerSourceProvider.overrideWithValue(FakeMarkerSource([])),
        ...signedInAs(),
        connectionSourceProvider.overrideWithValue(_Connection()),
      ],
    );
    addTearDown(container.dispose);
    container
        .read(shellDestinationProvider.notifier)
        .select(ShellDestination.player);
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: const VeltrunoxApp(),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Wiedergabe wird geladen …'), findsOneWidget);

    player.states.add(_state(recording: null));
    await tester.pumpAndSettle();
    expect(find.text('Keine Aufnahme gewählt'), findsOneWidget);

    await tester.tap(find.text('Aufnahme wählen'));
    await tester.pumpAndSettle();
    expect(find.text('Aufnahmen'), findsWidgets);
    expect(container.read(shellDestinationProvider), ShellDestination.library);
  });

  testWidgets('shows the recording and its markers beside the controls', (
    tester,
  ) async {
    await _pump(tester);

    expect(find.text('Soundcheck'), findsOneWidget);
    expect(find.text('Markierungen'), findsOneWidget);
    expect(find.text('Einsatz'), findsOneWidget);
  });

  testWidgets('on a narrow screen the markers open in a sheet', (tester) async {
    await _pump(tester, width: 500);

    expect(find.text('Einsatz'), findsNothing);
    await tester.tap(find.text('Markierungen (2)'));
    await tester.pumpAndSettle();
    expect(find.text('Einsatz'), findsOneWidget);
  });

  testWidgets('playing a label jumps there and plays on to the end', (
    tester,
  ) async {
    final app = await _pump(tester);

    await tester.tap(find.byTooltip('Abspielen: Einsatz'));
    await tester.pump();

    expect(app.player.calls, ['seek 25000', 'play']);
  });

  testWidgets('playing a region plays it from its start', (tester) async {
    final app = await _pump(tester);

    await tester.tap(find.byTooltip('Abspielen: Strophe'));
    await tester.pump();

    expect(app.player.calls, ['playRegion 10000 67000']);
  });

  group('play mode', () {
    testWidgets('shows the shared mode as text and cycles through the three', (
      tester,
    ) async {
      final app = await _pump(tester);
      expect(find.text('Bis Ende'), findsOneWidget);

      await tester.tap(find.text('Bis Ende'));
      app.player.states.add(_state(mode: PlayMode.toRegionEnd));
      await tester.pumpAndSettle();
      expect(find.text('Regionsende'), findsOneWidget);
      await tester.tap(find.text('Regionsende'));
      app.player.states.add(_state(mode: PlayMode.loop));
      await tester.pumpAndSettle();
      expect(find.text('Schleife'), findsOneWidget);
      await tester.tap(find.text('Schleife'));

      expect(app.player.calls, [
        'playMode toRegionEnd',
        'playMode loop',
        'playMode toEnd',
      ]);
    });

    testWidgets('follows a change made by another client', (tester) async {
      final app = await _pump(tester);

      app.player.states.add(_state(mode: PlayMode.loop));
      await tester.pumpAndSettle();

      expect(find.text('Schleife'), findsOneWidget);
      expect(find.byIcon(Icons.repeat), findsOneWidget);
    });
  });

  group('changes by others', () {
    testWidgets('a moved position is announced without blocking', (
      tester,
    ) async {
      final app = await _pump(tester);

      app.player.states.add(_state(position: 72000));
      await tester.pump();
      await tester.pump();

      expect(
        find.text('Die Wiedergabeposition wurde geändert.'),
        findsOneWidget,
      );
    });

    testWidgets('what this client does itself is not announced', (
      tester,
    ) async {
      final app = await _pump(tester);

      await tester.tap(find.byTooltip('Abspielen: Einsatz'));
      app.player.states.add(_state(position: 25000, running: true));
      await tester.pump();
      await tester.pump();

      expect(find.text('Die Wiedergabeposition wurde geändert.'), findsNothing);
    });
  });
}
