import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:veltrunox_app/design/theme/app_theme.dart';
import 'package:veltrunox_app/features/markers/marker.dart';
import 'package:veltrunox_app/features/waveform/waveform_panel.dart';
import 'package:material_ui/material_ui.dart';

import 'waveform_core_test.dart' show loadCleanFixture;

/// The test scene of the design specification: three overlapping regions, two
/// labels apart and a group of three close ones, on real peaks.
const sceneLabels = [
  LabelMarker(id: 1, version: 1, title: 'Intro', positionMs: 2000),
  LabelMarker(id: 2, version: 1, title: 'Break', positionMs: 40000),
  LabelMarker(id: 3, version: 1, title: 'Fill A', positionMs: 60000),
  LabelMarker(id: 4, version: 1, title: 'Fill B', positionMs: 60300),
  LabelMarker(id: 5, version: 1, title: 'Fill C', positionMs: 60600),
];
const sceneRegions = [
  RegionMarker(id: 10, version: 1, title: 'Verse', startMs: 5000, endMs: 30000),
  RegionMarker(
    id: 11,
    version: 1,
    title: 'Chorus',
    startMs: 20000,
    endMs: 45000,
  ),
  RegionMarker(id: 12, version: 1, title: 'Solo', startMs: 25000, endMs: 50000),
];

/// What the panel reported, for the tests to check.
class Intents {
  final played = <int>[];
  final playedRegions = <int>[];
  final seeks = <int>[];
  final scrubs = <int>[];
  final scrubEnds = <int>[];
  final drawn = <(int, int)>[];
  final resized = <(int, int, int)>[];
}

Future<Intents> pumpScene(
  WidgetTester tester, {
  double width = 1000,
  List<Marker> markers = const [...sceneLabels, ...sceneRegions],
  int positionMs = 10000,
  bool withData = true,
}) async {
  tester.view.physicalSize = Size(width, 900);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final fixture = loadCleanFixture();
  final intents = Intents();
  await tester.pumpWidget(
    ProviderScope(
      child: MaterialApp(
        theme: buildAppTheme(),
        home: Scaffold(
          body: SingleChildScrollView(
            child: WaveformPanel(
              recordingId: 1,
              durationMs: fixture.durationMs,
              data: withData ? fixture.data : null,
              positionMs: positionMs,
              markers: markers,
              onPlayFrom: intents.played.add,
              onPlayRegion: (region) => intents.playedRegions.add(region.id),
              onSeek: intents.seeks.add,
              onScrub: intents.scrubs.add,
              onScrubEnd: intents.scrubEnds.add,
              onRegionDrawn: (start, end) => intents.drawn.add((start, end)),
              onRegionResized: (region, start, end) =>
                  intents.resized.add((region.id, start, end)),
              onMarkerContext: (_) {},
            ),
          ),
        ),
      ),
    ),
  );
  await tester.pump();
  await tester.pump();
  return intents;
}
