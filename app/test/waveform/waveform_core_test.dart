import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:veltrunox_app/features/markers/marker.dart';
import 'package:veltrunox_app/features/waveform/label_clusters.dart';
import 'package:veltrunox_app/features/waveform/region_lane_layout.dart';
import 'package:veltrunox_app/features/waveform/snap_targets.dart';
import 'package:veltrunox_app/features/waveform/time_viewport.dart';
import 'package:veltrunox_app/features/waveform/waveform_data.dart';

RegionMarker region(int id, int start, int end) => RegionMarker(
  id: id,
  version: 1,
  title: 'Region $id',
  startMs: start,
  endMs: end,
);

LabelMarker label(int id, int at) =>
    LabelMarker(id: id, version: 1, title: 'Label $id', positionMs: at);

/// Real peaks of a 73-second recording, exported from the development server.
({WaveformData data, int durationMs}) loadCleanFixture() {
  final json = jsonDecode(
    File('test/fixtures/clean_waveform.json').readAsStringSync(),
  ) as Map<String, dynamic>;
  final levels = [
    for (final level in json['levels'] as List)
      WaveformLevel(
        framesPerBlock: level['framesPerBlock'] as int,
        peaks: Int8List.fromList(base64Decode(level['peaks'] as String)),
      ),
  ];
  return (
    data: WaveformData(sampleRate: json['sampleRate'] as int, levels: levels),
    durationMs: json['durationMs'] as int,
  );
}

void main() {
  group('TimeViewport', () {
    final fit = TimeViewport.fit(durationMs: 60000, widthPx: 600);

    test('fit shows the whole recording', () {
      expect(fit.startMs, 0);
      expect(fit.endMs, 60000);
      expect(fit.isFitted, isTrue);
      expect(fit.timeToX(30000), 300);
      expect(fit.xToTime(300), 30000);
    });

    test('zooming keeps the time under the anchor in place', () {
      final zoomed = fit.zoomedAt(4, 150);
      expect(zoomed.msPerPx, 25);
      expect(zoomed.xToTime(150), closeTo(15000, 1e-6));
      expect(zoomed.isFitted, isFalse);
    });

    test('zoom stays within fit and the finest zoom', () {
      expect(fit.zoomedAt(0.1, 0).msPerPx, fit.msPerPx);
      expect(fit.zoomedAt(1e6, 0).msPerPx, TimeViewport.minMsPerPx);
    });

    test('panning is clamped to the recording', () {
      final zoomed = fit.zoomedAt(4, 0);
      expect(zoomed.pannedByPixels(100).startMs, 0);
      final far = zoomed.pannedByPixels(-1e6);
      expect(far.endMs, closeTo(60000, 1e-6));
      final moved = zoomed.pannedByPixels(-40);
      expect(moved.startMs, 40 * 25);
    });

    test('button zoom centres on a visible playhead, else on the middle', () {
      final zoomed = fit.zoomedForButton(2, playheadMs: 6000);
      expect(zoomed.timeToX(6000), closeTo(60, 1e-6));
      final away = fit.zoomedAt(4, 0).zoomedForButton(2, playheadMs: 50000);
      final base = fit.zoomedAt(4, 0);
      expect(away.xToTime(300), closeTo(base.xToTime(300), 1e-6));
    });

    test('fitted returns to the whole recording', () {
      expect(fit.zoomedAt(8, 300).fitted(), fit);
    });

    test('revealing scrolls minimally and zooms out for long ranges', () {
      final zoomed = fit.zoomedAt(4, 0);
      expect(zoomed.revealing(1000, 2000), zoomed);
      final right = zoomed.revealing(20000, 21000);
      expect(right.endMs, closeTo(21000, 1e-6));
      final wide = zoomed.revealing(0, 40000);
      expect(wide.visibleSpanMs, closeTo(40000, 1e-6));
    });

    test('resizing keeps the visible range valid', () {
      final zoomed = fit.zoomedAt(4, 0).pannedByPixels(-100);
      final wider = zoomed.withWidth(1200);
      expect(wider.startMs, zoomed.startMs);
      expect(wider.widthPx, 1200);
    });
  });

  group('layoutRegionLanes', () {
    test('three overlapping regions get three lanes (UI-AC-03)', () {
      final result = layoutRegionLanes([
        region(1, 0, 10000),
        region(2, 5000, 15000),
        region(3, 8000, 20000),
      ]);
      expect(result.laneCount, 3);
      expect({...result.laneById.values}, {0, 1, 2});
    });

    test('regions that only touch share a lane', () {
      final result = layoutRegionLanes([
        region(1, 0, 5000),
        region(2, 5000, 9000),
      ]);
      expect(result.laneCount, 1);
    });

    test('non-overlapping regions reuse lanes', () {
      final result = layoutRegionLanes([
        region(1, 0, 10000),
        region(2, 5000, 15000),
        region(3, 12000, 20000),
      ]);
      expect(result.laneCount, 2);
      expect(result.laneOf(3), result.laneOf(1));
    });

    test('no lane ever holds overlapping regions', () {
      final regions = [
        for (var i = 0; i < 40; i++)
          region(i, (i * 37) % 900, (i * 37) % 900 + 50 + i),
      ];
      final result = layoutRegionLanes(regions);
      for (final a in regions) {
        for (final b in regions) {
          if (a.id == b.id || result.laneOf(a.id) != result.laneOf(b.id)) {
            continue;
          }
          expect(a.startMs < b.endMs && b.startMs < a.endMs, isFalse);
        }
      }
    });

    test('the same input gives the same output, in any input order', () {
      final regions = [
        region(1, 0, 10000),
        region(2, 5000, 15000),
        region(3, 8000, 20000),
      ];
      final a = layoutRegionLanes(regions);
      final b = layoutRegionLanes(regions.reversed.toList());
      expect(b.laneById, a.laneById);
    });

    test('a previous layout is kept while it causes no overlap (UI-AC-04)', () {
      final before = layoutRegionLanes([
        region(1, 0, 10000),
        region(2, 5000, 15000),
      ]);
      // Region 1 shrinks away from region 2: it could drop to lane 0 of its
      // own, but the lanes must not change during the drag.
      final after = layoutRegionLanes([
        region(1, 0, 4000),
        region(2, 5000, 15000),
      ], previous: before);
      expect(after.laneById, before.laneById);
    });

    test('a conflicting kept lane is replaced', () {
      final before = layoutRegionLanes([
        region(1, 0, 4000),
        region(2, 5000, 15000),
      ]);
      expect(before.laneCount, 1);
      final after = layoutRegionLanes([
        region(1, 0, 10000),
        region(2, 5000, 15000),
      ], previous: before);
      expect(after.laneCount, 2);
    });

    test(
      'compact view gives the last slot to the selected region (UI-AC-09)',
      () {
        final result = layoutRegionLanes([
          region(1, 0, 10000),
          region(2, 1000, 11000),
          region(3, 2000, 12000),
          region(4, 3000, 13000),
        ]);
        expect(result.visibleLanes(2), [0, 1]);
        expect(result.visibleLanes(2, selectedId: 4), [0, result.laneOf(4)]);
        expect(result.visibleLanes(8), [0, 1, 2, 3]);
      },
    );

    test('no regions give no lanes', () {
      expect(layoutRegionLanes([]).laneCount, 0);
    });
  });

  group('clusterLabels', () {
    final viewport = TimeViewport.fit(durationMs: 100000, widthPx: 1000);
    double chip(LabelMarker label) => 60;

    test('distant labels stay single', () {
      final clusters = clusterLabels(
        [label(1, 10000), label(2, 50000)],
        viewport,
        chipWidthPx: chip,
      );
      expect(clusters.map((c) => c.isGroup), [false, false]);
      expect(clusters.first.xPx, 100);
    });

    test('colliding chips become one group with a count (UI-AC-10)', () {
      final clusters = clusterLabels(
        [label(1, 10000), label(2, 12000), label(3, 14000), label(4, 60000)],
        viewport,
        chipWidthPx: chip,
      );
      expect(clusters.length, 2);
      expect(clusters.first.hiddenCount, 2);
      expect(clusters.first.labels.map((l) => l.id), [1, 2, 3]);
    });

    test('zooming in separates a group', () {
      final zoomed = viewport.zoomedAt(10, 100);
      final clusters = clusterLabels(
        [label(1, 10000), label(2, 12000)],
        zoomed,
        chipWidthPx: chip,
      );
      expect(clusters.length, 2);
    });

    test('labels outside the viewport are left out', () {
      final zoomed = viewport.zoomedAt(10, 0);
      final clusters = clusterLabels(
        [label(1, 1000), label(2, 90000)],
        zoomed,
        chipWidthPx: chip,
      );
      expect(clusters.map((c) => c.first.id), [1]);
    });
  });

  group('SnapTargets', () {
    final viewport = TimeViewport.fit(durationMs: 100000, widthPx: 1000);
    final targets = SnapTargets.from([
      label(1, 20000),
      region(2, 40000, 60000),
    ]);

    test('snaps to labels and to both region ends', () {
      expect(targets.nearest(20500, viewport), 20000);
      expect(targets.nearest(39000, viewport), 40000);
      expect(targets.nearest(60800, viewport), 60000);
    });

    test('does not snap beyond the pixel threshold', () {
      expect(targets.nearest(30000, viewport), isNull);
      expect(targets.nearest(21500, viewport), isNull);
    });

    test('the threshold follows the zoom', () {
      final zoomed = viewport.zoomedAt(10, 200);
      expect(targets.nearest(20000 + 12.0 * zoomed.msPerPx / 2, zoomed), 20000);
    });

    test('the dragged marker does not attract itself', () {
      final others = SnapTargets.from([
        label(1, 20000),
        region(2, 40000, 60000),
      ], excludeId: 2);
      expect(others.nearest(40000, viewport), isNull);
      expect(others.nearest(20000, viewport), 20000);
    });
  });

  group('WaveformData with real peaks', () {
    final fixture = loadCleanFixture();
    final data = fixture.data;

    test('levels run from coarse to fine', () {
      final sizes = data.levels.map((l) => l.framesPerBlock).toList();
      expect(sizes, [...sizes]..sort((a, b) => b - a));
      expect(data.levels.last.framesPerBlock, 1024);
    });

    test('the level follows the zoom', () {
      expect(data.levelFor(1000).framesPerBlock, 16384);
      expect(data.levelFor(30).framesPerBlock, 1024);
      expect(data.levelFor(1).framesPerBlock, 1024);
    });

    test('columns cover the whole viewport with valid amplitudes', () {
      final viewport = TimeViewport.fit(
        durationMs: fixture.durationMs,
        widthPx: 800,
      );
      final columns = computeColumns(data, viewport);
      expect(columns.length, 800);
      var loud = 0;
      for (var i = 0; i < columns.length; i++) {
        expect(columns.min[i], inInclusiveRange(-1, 0.0001));
        expect(columns.max[i], inInclusiveRange(-0.0001, 1));
        if (columns.max[i] - columns.min[i] > 0.05) loud++;
      }
      expect(loud, greaterThan(400));
    });

    test('zoomed columns come from the finer level and stay in range', () {
      final viewport = TimeViewport.fit(
        durationMs: fixture.durationMs,
        widthPx: 800,
      ).zoomedAt(20, 400);
      final columns = computeColumns(data, viewport);
      expect(columns.length, 800);
      expect(columns.max.reduce((a, b) => a > b ? a : b), lessThanOrEqualTo(1));
    });

    test('synthetic peaks map to the expected columns', () {
      final level = WaveformLevel(
        framesPerBlock: 1000,
        peaks: Int8List.fromList([-10, 20, -127, 127, 0, 0, -5, 5]),
      );
      final synthetic = WaveformData(sampleRate: 1000, levels: [level]);
      final viewport = TimeViewport.fit(durationMs: 4000, widthPx: 4);
      final columns = computeColumns(synthetic, viewport);
      expect(columns.min[1], -1);
      expect(columns.max[1], 1);
      expect(columns.max[3], closeTo(5 / 127, 1e-6));
    });

    test('columns beyond the recording stay empty', () {
      final level = WaveformLevel(
        framesPerBlock: 1000,
        peaks: Int8List.fromList([-127, 127, -127, 127]),
      );
      final synthetic = WaveformData(sampleRate: 1000, levels: [level]);
      final viewport = TimeViewport.fit(durationMs: 4000, widthPx: 4);
      final columns = computeColumns(synthetic, viewport);
      expect(columns.max[2], 0);
      expect(columns.min[3], 0);
    });
  });
}
