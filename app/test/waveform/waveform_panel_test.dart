import 'package:flutter/gestures.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/features/waveform/region_handle.dart';
import 'package:veltrunox_app/features/waveform/region_item.dart';
import 'package:veltrunox_app/features/waveform/timeline_ruler.dart';
import 'package:veltrunox_app/features/waveform/waveform_renderer.dart';
import 'package:veltrunox_app/features/waveform/waveform_view_controller.dart';

import 'waveform_scene.dart';

Offset waveformOrigin(WidgetTester tester) =>
    tester.getTopLeft(find.byType(WaveformRenderer));

WaveformViewState viewState(WidgetTester tester) =>
    ProviderScope.containerOf(tester.element(find.byType(WaveformRenderer)))
        .read(waveformViewProvider(1));

double top(WidgetTester tester, String text) =>
    tester.getTopLeft(find.text(text)).dy;

void main() {
  testWidgets('the region being played is highlighted in its lane', (
    tester,
  ) async {
    await pumpScene(tester, activeRange: (startMs: 25000, endMs: 50000));
    final items = tester.widgetList<RegionItem>(find.byType(RegionItem));
    expect(
      {for (final item in items) item.region.title: item.active},
      {'Verse': false, 'Chorus': false, 'Solo': true},
    );
  });

  testWidgets('tapping a label plays from its position (UI-AC-01)', (
    tester,
  ) async {
    final intents = await pumpScene(tester);
    await tester.tap(find.text('Intro'));
    expect(intents.played, [2000]);
  });

  testWidgets('tapping a region name plays the region (UI-AC-02)', (
    tester,
  ) async {
    final intents = await pumpScene(tester);
    await tester.tap(find.text('Chorus'));
    expect(intents.playedRegions, [11]);
  });

  testWidgets('overlapping regions use separate lanes (UI-AC-03)', (
    tester,
  ) async {
    await pumpScene(tester);
    final tops = {
      top(tester, 'Verse'),
      top(tester, 'Chorus'),
      top(tester, 'Solo'),
    };
    expect(tops.length, 3);
  });

  testWidgets('only the selected region has handles; lanes hold while '
      'dragging (UI-AC-04)', (tester) async {
    final intents = await pumpScene(tester);
    expect(find.byType(RegionHandle), findsNothing);
    final before = [
      top(tester, 'Verse'),
      top(tester, 'Chorus'),
      top(tester, 'Solo'),
    ];

    await tester.tap(
      find.descendant(
        of: find
            .ancestor(of: find.text('Chorus'), matching: find.byType(Row))
            .first,
        matching: find.byIcon(Icons.edit_outlined),
      ),
    );
    await tester.pump();
    expect(find.byType(RegionHandle), findsNWidgets(2));

    final gesture = await tester.startGesture(
      tester.getCenter(find.byType(RegionHandle).last),
    );
    await gesture.moveBy(const Offset(40, 0));
    await tester.pump();
    await gesture.moveBy(const Offset(40, 0));
    await tester.pump();
    expect([
      top(tester, 'Verse'),
      top(tester, 'Chorus'),
      top(tester, 'Solo'),
    ], before);
    await gesture.up();
    await tester.pump();

    expect(intents.resized.length, 1);
    final (id, start, end) = intents.resized.single;
    expect(id, 11);
    expect(start, 20000);
    expect(end, greaterThan(45000));
  });

  testWidgets('a tap on the free waveform sets the position without '
      'playing (UI-AC-06)', (tester) async {
    final intents = await pumpScene(tester);
    await tester.tapAt(waveformOrigin(tester) + const Offset(600, 60));
    expect(intents.seeks.length, 1);
    expect(intents.seeks.single, closeTo(43900, 300));
    expect(intents.played, isEmpty);
    expect(intents.playedRegions, isEmpty);
  });

  testWidgets('dragging free pans, dragging the playhead scrubs (UI-AC-07)', (
    tester,
  ) async {
    final intents = await pumpScene(tester);
    await tester.tap(find.byTooltip('Vergrößern'));
    await tester.pump();
    final zoomed = viewState(tester).viewport!;

    await tester.dragFrom(
      waveformOrigin(tester) + const Offset(700, 60),
      const Offset(-120, 0),
    );
    await tester.pump();
    expect(viewState(tester).viewport!.startMs, greaterThan(zoomed.startMs));
    expect(intents.scrubs, isEmpty);

    final playheadX = viewState(tester).viewport!.timeToX(10000);
    await tester.dragFrom(
      waveformOrigin(tester) + Offset(playheadX, 60),
      const Offset(40, 0),
    );
    await tester.pump();
    expect(intents.scrubs, isNotEmpty);
    expect(intents.scrubEnds.length, 1);
    expect(intents.scrubEnds.single, greaterThan(10000));
  });

  testWidgets('the toolbar creates a label or a region at the playhead', (
    tester,
  ) async {
    final intents = await pumpScene(tester);
    await tester.tap(find.byTooltip('Label hier setzen'));
    await tester.tap(find.byTooltip('Region hier anlegen'));
    await tester.pump();
    expect(intents.labelsSet, 1);
    expect(intents.regionsSet, 1);
  });

  testWidgets('a narrow screen shows two lanes and "all regions" (UI-AC-09)', (
    tester,
  ) async {
    await pumpScene(tester, width: 400);
    int shown() => [
      'Verse',
      'Chorus',
      'Solo',
    ].where((name) => find.text(name).evaluate().isNotEmpty).length;
    expect(shown(), 2);
    await tester.tap(find.text('Alle Regionen'));
    await tester.pump();
    expect(shown(), 3);
  });

  testWidgets('close labels form a group playable in two taps (UI-AC-10)', (
    tester,
  ) async {
    final intents = await pumpScene(tester);
    expect(find.text('+2'), findsOneWidget);
    expect(find.text('Fill B'), findsNothing);
    await tester.tap(find.text('+2'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Fill B'));
    await tester.pumpAndSettle();
    expect(intents.played, [60300]);
  });

  testWidgets('without data the waveform says it is loading', (tester) async {
    await pumpScene(tester, withData: false);
    expect(find.text('Wellenform wird geladen …'), findsOneWidget);
    expect(find.bySemanticsLabel('Wellenform'), findsNothing);
  });

  testWidgets('keyboard-reachable zoom buttons change only the local view', (
    tester,
  ) async {
    final intents = await pumpScene(tester);
    await tester.tap(find.byTooltip('Vergrößern'));
    await tester.pump();
    expect(viewState(tester).viewport!.isFitted, isFalse);
    await tester.tap(find.byTooltip('Ganze Aufnahme zeigen'));
    await tester.pump();
    expect(viewState(tester).viewport!.isFitted, isTrue);
    expect(intents.seeks, isEmpty);
    expect(intents.played, isEmpty);
  });

  group('position', () {
    testWidgets('a click that wobbles a little still sets the position', (
      tester,
    ) async {
      final intents = await pumpScene(tester);
      final gesture = await tester.startGesture(
        waveformOrigin(tester) + const Offset(600, 60),
        kind: PointerDeviceKind.mouse,
      );
      await gesture.moveBy(const Offset(3, 1));
      await gesture.up();
      await tester.pump();

      expect(intents.seeks.length, 1);
      expect(intents.seeks.single, closeTo(43800, 400));
    });

    testWidgets('a press on the ruler puts the playhead there (UI-AC-06)', (
      tester,
    ) async {
      final intents = await pumpScene(tester);
      final ruler = tester.getTopLeft(find.byType(TimelineRuler));
      await tester.tapAt(ruler + const Offset(500, 10));
      await tester.pump();

      expect(intents.scrubs.single, closeTo(36468, 400));
      expect(intents.scrubEnds.single, intents.scrubs.single);
    });

    testWidgets('dragging along the ruler goes on from where it was pressed', (
      tester,
    ) async {
      final intents = await pumpScene(tester);
      final ruler = tester.getTopLeft(find.byType(TimelineRuler));
      final gesture = await tester.startGesture(ruler + const Offset(300, 10));
      await gesture.moveBy(const Offset(100, 0));
      await tester.pump();
      await gesture.moveBy(const Offset(100, 0));
      await tester.pump();
      await gesture.up();
      await tester.pump();

      expect(intents.scrubs.length, greaterThanOrEqualTo(2));
      expect(intents.scrubs.last, closeTo(500 / 1000 * 72936, 800));
      expect(intents.scrubEnds.single, intents.scrubs.last);
    });

    testWidgets('the exact time stands above the playhead and follows it', (
      tester,
    ) async {
      await pumpScene(tester);
      expect(find.text('0:10,0'), findsOneWidget);

      final ruler = tester.getTopLeft(find.byType(TimelineRuler));
      final gesture = await tester.startGesture(ruler + const Offset(500, 10));
      await gesture.moveBy(const Offset(100, 0));
      await tester.pump();
      expect(find.text('0:10,0'), findsNothing);
      expect(find.textContaining(RegExp(r'^0:4\d,\d$')), findsOneWidget);
      expect(
        tester.getBottomLeft(find.textContaining(RegExp(r'^0:4\d,\d$'))).dy,
        lessThan(tester.getTopLeft(find.byType(TimelineRuler)).dy),
      );
      await gesture.up();
    });
  });

  group('two fingers and the trackpad', () {
    Future<(TestGesture, TestGesture)> twoFingers(
      WidgetTester tester,
      Offset a,
      Offset b,
    ) async {
      final first = await tester.startGesture(a, pointer: 1);
      final second = await tester.startGesture(b, pointer: 2);
      return (first, second);
    }

    testWidgets('spreading two fingers zooms in', (tester) async {
      await pumpScene(tester);
      final origin = waveformOrigin(tester);
      final (first, second) = await twoFingers(
        tester,
        origin + const Offset(450, 60),
        origin + const Offset(550, 60),
      );
      await first.moveBy(const Offset(-60, 0));
      await second.moveBy(const Offset(60, 0));
      await tester.pump();
      await first.up();
      await second.up();

      final viewport = viewState(tester).viewport!;
      expect(viewport.isFitted, isFalse);
      expect(viewport.msPerPx, lessThan(72936 / 1000 / 2));
    });

    testWidgets('pinching works after the first finger already moved', (
      tester,
    ) async {
      await pumpScene(tester);
      final origin = waveformOrigin(tester);
      final first = await tester.startGesture(
        origin + const Offset(450, 60),
        pointer: 1,
      );
      await first.moveBy(const Offset(-40, 0));
      // The second finger lands above the waveform, on the ruler.
      final second = await tester.startGesture(
        origin + const Offset(550, -60),
        pointer: 2,
      );
      await first.moveBy(const Offset(-100, 0));
      await second.moveBy(const Offset(100, 0));
      await tester.pump();
      await first.up();
      await second.up();

      expect(viewState(tester).viewport!.isFitted, isFalse);
    });

    testWidgets('moving two fingers together pans the waveform', (
      tester,
    ) async {
      await pumpScene(tester);
      await tester.tap(find.byTooltip('Vergrößern'));
      await tester.pump();
      final before = viewState(tester).viewport!;
      final origin = waveformOrigin(tester);
      final (first, second) = await twoFingers(
        tester,
        origin + const Offset(450, 60),
        origin + const Offset(550, 60),
      );
      await first.moveBy(const Offset(-80, 0));
      await second.moveBy(const Offset(-80, 0));
      await tester.pump();
      await first.up();
      await second.up();

      final after = viewState(tester).viewport!;
      expect(after.startMs, greaterThan(before.startMs));
      expect(after.msPerPx, closeTo(before.msPerPx, before.msPerPx * 0.05));
    });

    testWidgets('the mouse wheel zooms around the pointer, not the page', (
      tester,
    ) async {
      await pumpScene(tester);
      final at = waveformOrigin(tester) + const Offset(250, 60);
      final before = viewState(tester).viewport!;
      final pointer = TestPointer(1, PointerDeviceKind.mouse);

      await tester.sendEventToBinding(pointer.hover(at));
      await tester.sendEventToBinding(pointer.scroll(const Offset(0, -100)));
      await tester.pump();
      final zoomed = viewState(tester).viewport!;
      expect(zoomed.msPerPx, lessThan(before.msPerPx));
      expect(zoomed.xToTime(250), closeTo(before.xToTime(250), 1));
      expect(waveformOrigin(tester) + const Offset(250, 60), at);

      await tester.sendEventToBinding(pointer.scroll(const Offset(0, 100)));
      await tester.pump();
      expect(
        viewState(tester).viewport!.msPerPx,
        closeTo(before.msPerPx, 0.01),
      );
    });

    testWidgets('a sideways trackpad swipe pans, a pinch zooms', (
      tester,
    ) async {
      await pumpScene(tester);
      await tester.tap(find.byTooltip('Vergrößern'));
      await tester.pump();
      final before = viewState(tester).viewport!;
      final centre = waveformOrigin(tester) + const Offset(500, 60);
      final pointer = TestPointer(1, PointerDeviceKind.trackpad);

      await tester.sendEventToBinding(pointer.hover(centre));
      await tester.sendEventToBinding(pointer.scroll(const Offset(60, 0)));
      await tester.pump();
      expect(viewState(tester).viewport!.startMs, greaterThan(before.startMs));

      final panned = viewState(tester).viewport!;
      await tester.sendEventToBinding(
        PointerScaleEvent(position: centre, scale: 2),
      );
      await tester.pump();
      expect(
        viewState(tester).viewport!.msPerPx,
        closeTo(panned.msPerPx / 2, 0.01),
      );
    });
  });

  group('leaving the edit mode of a region', () {
    Future<void> select(WidgetTester tester) async {
      await tester.tap(
        find.descendant(
          of: find
              .ancestor(of: find.text('Chorus'), matching: find.byType(Row))
              .first,
          matching: find.byIcon(Icons.edit_outlined),
        ),
      );
      await tester.pump();
      expect(find.byType(RegionHandle), findsNWidgets(2));
    }

    testWidgets('a tap on the waveform leaves it', (tester) async {
      await pumpScene(tester);
      await select(tester);
      await tester.tapAt(waveformOrigin(tester) + const Offset(700, 60));
      await tester.pump();
      expect(find.byType(RegionHandle), findsNothing);
    });

    testWidgets('a tap elsewhere on the page leaves it', (tester) async {
      await pumpScene(tester);
      await select(tester);
      await tester.tap(find.byTooltip('Vergrößern'));
      await tester.pump();
      expect(find.byType(RegionHandle), findsNothing);
    });

    testWidgets('a tap on another region leaves it', (tester) async {
      await pumpScene(tester);
      await select(tester);
      await tester.tap(find.text('Verse'));
      await tester.pump();
      expect(find.byType(RegionHandle), findsNothing);
    });

    testWidgets('a tap on empty lane space leaves it', (tester) async {
      await pumpScene(tester);
      await select(tester);
      final lane = tester.getTopLeft(find.text('Verse'));
      await tester.tapAt(Offset(950, lane.dy));
      await tester.pump();
      expect(find.byType(RegionHandle), findsNothing);
    });
  });
}
