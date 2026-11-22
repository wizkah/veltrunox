import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/features/waveform/region_handle.dart';
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

  testWidgets('region mode draws an interval in two steps (UI-AC-08)', (
    tester,
  ) async {
    final intents = await pumpScene(tester);
    await tester.tap(find.byTooltip('Region aufziehen'));
    await tester.pump();
    await tester.dragFrom(
      waveformOrigin(tester) + const Offset(300, 60),
      const Offset(200, 0),
    );
    await tester.pump();
    expect(intents.drawn.length, 1);
    final (start, end) = intents.drawn.single;
    expect(start, lessThan(end));
    expect(intents.seeks, isEmpty);
    expect(viewState(tester).drawingRegion, isFalse);
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
}
