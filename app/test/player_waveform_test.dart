import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/app.dart';
import 'package:veltrunox_app/app_shell/shell_destination.dart';
import 'package:veltrunox_app/features/connection/connection_controller.dart';
import 'package:veltrunox_app/features/connection/connection_source.dart';
import 'package:veltrunox_app/features/connection/connection_status.dart';
import 'package:veltrunox_app/features/library/analysis_info.dart';
import 'package:veltrunox_app/features/library/library_controller.dart';
import 'package:veltrunox_app/features/library/recording_source.dart';
import 'package:veltrunox_app/features/library/recording_summary.dart';
import 'package:veltrunox_app/features/markers/marker.dart';
import 'package:veltrunox_app/features/markers/marker_controller.dart';
import 'package:veltrunox_app/features/player/panel_surface.dart';
import 'package:veltrunox_app/features/player/player_controller.dart';
import 'package:veltrunox_app/features/player/player_state.dart';
import 'package:veltrunox_app/features/player/position_slider.dart';
import 'package:veltrunox_app/features/player/settings_panel.dart';
import 'package:veltrunox_app/features/player/volume_control.dart';
import 'package:veltrunox_app/features/waveform/label_layer.dart';
import 'package:veltrunox_app/features/waveform/region_handle.dart';
import 'package:veltrunox_app/features/waveform/region_lanes.dart';
import 'package:veltrunox_app/features/waveform/waveform_renderer.dart';
import 'package:veltrunox_app/features/waveform/waveform_source.dart';

import 'no_recording_actions.dart';
import 'fake_marker_source.dart';
import 'fake_access_source.dart';
import 'fake_player_source.dart';
import 'fake_waveform_source.dart';

class _Recordings with NoRecordingActions implements RecordingSource {
  _Recordings({required this.scanned});

  final bool scanned;

  @override
  Future<List<RecordingSummary>> listRecordings() async => [
    RecordingSummary(
      id: 1,
      title: 'Soundcheck',
      analysis: scanned
          ? const AnalysisInfo(
              status: AnalysisStatus.done,
              values: AnalysisValues(
                tuningHz: 440,
                keyTonic: 0,
                keyMinor: false,
                tempoBpm: 120,
              ),
            )
          : const AnalysisInfo(),
    ),
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

const _state = PlayerState(
  recordingId: 1,
  running: false,
  speed: 1,
  tuningHz: 440,
  keyShiftSemitones: 0,
  volumePercent: 100,
  positionMs: 10000,
  playMode: PlayMode.toEnd,
);

class _App {
  _App(this.player, this.markers, this.waveforms, this.container);

  final FakePlayerSource player;
  final FakeMarkerSource markers;
  final FakeWaveformSource waveforms;
  final ProviderContainer container;
}

Future<_App> _pump(
  WidgetTester tester, {
  bool scanned = true,
  double width = 1400,
  double height = 1200,
  int extraRegions = 0,
}) async {
  tester.view.physicalSize = Size(width, height);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final player = FakePlayerSource()..currentPosition = 10000;
  final waveforms = FakeWaveformSource();
  final markers = FakeMarkerSource([
    const LabelMarker(id: 1, version: 1, title: 'Einsatz', positionMs: 25000),
    const RegionMarker(
      id: 2,
      version: 1,
      title: 'Strophe',
      startMs: 10000,
      endMs: 40000,
    ),
    for (var i = 0; i < extraRegions; i++)
      RegionMarker(
        id: 100 + i,
        version: 1,
        title: 'Extra $i',
        startMs: 10000,
        endMs: 40000,
      ),
  ]);
  final container = ProviderContainer(
    overrides: [
      playerSourceProvider.overrideWithValue(player),
      recordingSourceProvider.overrideWithValue(_Recordings(scanned: scanned)),
      markerSourceProvider.overrideWithValue(markers),
      waveformSourceProvider.overrideWithValue(waveforms),
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
  player.states.add(_state);
  await tester.pumpAndSettle();
  return _App(player, markers, waveforms, container);
}

void main() {
  testWidgets('a scanned recording shows the waveform instead of the slider', (
    tester,
  ) async {
    final app = await _pump(tester);

    expect(find.byType(WaveformRenderer), findsOneWidget);
    expect(find.byKey(positionSliderKey), findsNothing);
    expect(app.waveforms.requests, [1]);
    expect(find.text('Einsatz'), findsWidgets);
  });

  testWidgets('a recording without a scan says there is no waveform yet', (
    tester,
  ) async {
    final app = await _pump(tester, scanned: false);

    expect(find.byType(WaveformRenderer), findsNothing);
    expect(
      find.text('Für diese Aufnahme gibt es noch keine Wellenform.'),
      findsOneWidget,
    );
    expect(app.waveforms.requests, isEmpty);
  });

  testWidgets('tapping a label in the waveform plays from there', (
    tester,
  ) async {
    final app = await _pump(tester);

    await tester.tap(
      find.descendant(
        of: find.byType(LabelLayer),
        matching: find.text('Einsatz'),
      ),
    );
    await tester.pump();

    expect(app.player.calls, ['seek 25000', 'play']);
  });

  testWidgets('tapping a region in the waveform plays it', (tester) async {
    final app = await _pump(tester);

    await tester.tap(
      find.descendant(
        of: find.byType(RegionLanes),
        matching: find.text('Strophe'),
      ),
    );
    await tester.pump();

    expect(app.player.calls, ['playRegion 10000 40000']);
  });

  testWidgets('"Label hier setzen" adds a label at the current position', (
    tester,
  ) async {
    final app = await _pump(tester);

    await tester.tap(find.byTooltip('Label hier setzen'));
    await tester.pumpAndSettle();

    expect(app.markers.calls, ['addLabel Label 2 10000']);
    expect(
      app.markers.markers.whereType<LabelMarker>().map((l) => l.positionMs),
      contains(10000),
    );
  });

  testWidgets('a tap on the free waveform moves the position, no playing', (
    tester,
  ) async {
    final app = await _pump(tester);

    final origin = tester.getTopLeft(find.byType(WaveformRenderer));
    await tester.tapAt(origin + const Offset(300, 60));
    await tester.pump();

    expect(app.player.calls.single, startsWith('seek '));
    expect(app.player.calls, isNot(contains('play')));
  });

  testWidgets('dragging the playhead scrubs the shared player', (tester) async {
    final app = await _pump(tester);
    final origin = tester.getTopLeft(find.byType(WaveformRenderer));
    final width = tester.getSize(find.byType(WaveformRenderer)).width;
    final playheadX = 10000 / 72936 * width;

    final gesture = await tester.startGesture(origin + Offset(playheadX, 60));
    await gesture.moveBy(const Offset(40, 0));
    await tester.pump();
    await gesture.moveBy(const Offset(40, 0));
    await tester.pump();
    await gesture.up();
    await tester.pump(const Duration(milliseconds: 300));

    expect(app.player.calls.any((c) => c.startsWith('scrub ')), isTrue);
    expect(app.player.calls.last, startsWith('endScrub '));
  });

  testWidgets('moving a region handle writes the new end to the server', (
    tester,
  ) async {
    final app = await _pump(tester);

    await tester.tap(find.byIcon(Icons.edit_outlined).first);
    await tester.pump();
    final gesture = await tester.startGesture(
      tester.getCenter(find.byType(RegionHandle).last),
    );
    await gesture.moveBy(const Offset(40, 0));
    await tester.pump();
    await gesture.moveBy(const Offset(40, 0));
    await tester.pump();
    await gesture.up();
    await tester.pumpAndSettle();

    final call = app.markers.calls.single;
    expect(call, startsWith('updateRegion 2 v1 Strophe 10000 '));
    final region = app.markers.markers.whereType<RegionMarker>().single;
    expect(region.endMs, greaterThan(40000));
    expect(region.version, 2);
  });

  testWidgets('a marker changed by someone else shows up in the waveform', (
    tester,
  ) async {
    final app = await _pump(tester);
    expect(find.text('Chor'), findsNothing);

    app.markers.markers.add(
      const LabelMarker(id: 9, version: 1, title: 'Chor', positionMs: 55000),
    );
    app.markers.announced.add(1);
    await tester.pumpAndSettle();

    expect(find.text('Chor'), findsWidgets);
  });

  group('layout', () {
    testWidgets('on a wide screen the markers stand left and the volume right '
        'of the waveform and the transport', (tester) async {
      await _pump(tester);
      final waveform = tester.getRect(find.byType(WaveformRenderer));
      final markers = tester.getTopLeft(find.text('Markierungen'));
      final volume = tester.getTopLeft(find.text('Lautstärke'));
      final play = tester.getCenter(find.byIcon(Icons.play_arrow).last);

      expect(markers.dx, lessThan(waveform.left));
      expect(volume.dx, greaterThan(waveform.right));
      expect(volume.dx, greaterThan(play.dx));
      // The volume reaches from the waveform's level down to the transport.
      expect(volume.dy, lessThan(play.dy));
    });

    testWidgets('on a wide screen the volume panel reaches down to the pitch '
        'panel, which reaches the right edge', (tester) async {
      await _pump(tester);
      final volume = tester.getRect(
        find.ancestor(
          of: find.byKey(volumeSliderKey),
          matching: find.byType(PanelSurface),
        ),
      );
      final pitch = tester.getRect(
        find.ancestor(
          of: find.text('Tonhöhe'),
          matching: find.byType(SettingsPanel),
        ),
      );
      // The volume ends where the pitch panel begins, which reaches the edge.
      expect(volume.bottom, lessThanOrEqualTo(pitch.top));
      expect(pitch.top - volume.bottom, lessThan(24));
      expect(pitch.right, greaterThanOrEqualTo(volume.right - 0.5));
    });

    testWidgets('on a narrow screen the way to the markers is above the '
        'waveform', (tester) async {
      await _pump(tester, width: 500);
      final opener = tester.getTopLeft(find.text('Markierungen (2)'));
      final waveform = tester.getTopLeft(find.byType(WaveformRenderer));
      expect(opener.dy, lessThan(waveform.dy));
    });
  });

  testWidgets('a region is created at the playhead from the toolbar', (
    tester,
  ) async {
    final app = await _pump(tester);
    expect(find.text('Label hier setzen'), findsNothing);
    expect(find.text('Region hier anlegen'), findsNothing);

    await tester.tap(find.byTooltip('Region hier anlegen'));
    await tester.pumpAndSettle();

    final call = app.markers.calls.single;
    expect(call, startsWith('addRegion Region 2 10000 '));
    expect(find.byType(RegionHandle), findsNWidgets(2));
  });

  group('on a tall screen', () {
    testWidgets('the page fits and only the regions scroll', (tester) async {
      await _pump(tester, height: 800, extraRegions: 12);
      expect(tester.takeException(), isNull);
      final lanes = find.byType(RegionLanes);
      final scrollable = find.ancestor(
        of: lanes,
        matching: find.byType(SingleChildScrollView),
      );
      final regionScroll = tester.widget<SingleChildScrollView>(
        scrollable.first,
      );
      expect(regionScroll.scrollDirection, Axis.vertical);
      // The regions overflow their window, the controls below stay on screen.
      final screen = tester.view.physicalSize.height;
      expect(tester.getBottomLeft(find.text('Tonhöhe')).dy, lessThan(screen));
      expect(
        tester.getSize(scrollable.first).height,
        lessThan(tester.getSize(lanes).height),
      );
    });

    testWidgets('with few regions the waveform grows and the panels sit at '
        'the bottom', (tester) async {
      await _pump(tester);
      expect(tester.takeException(), isNull);
      expect(
        tester.getSize(find.byType(WaveformRenderer)).height,
        greaterThan(200),
      );
      final pitch = tester.getRect(
        find.ancestor(
          of: find.text('Tonhöhe'),
          matching: find.byType(SettingsPanel),
        ),
      );
      expect(pitch.bottom, closeTo(tester.view.physicalSize.height - 16, 1));
    });

    testWidgets('on a low screen the whole page scrolls instead', (
      tester,
    ) async {
      await _pump(tester, height: 700, extraRegions: 12);
      expect(tester.takeException(), isNull);
      final lanes = find.byType(RegionLanes);
      final scrollable = find.ancestor(
        of: lanes,
        matching: find.byType(SingleChildScrollView),
      );
      expect(
        tester.getSize(scrollable.first).height,
        greaterThanOrEqualTo(tester.getSize(lanes).height),
      );
    });
  });
}
