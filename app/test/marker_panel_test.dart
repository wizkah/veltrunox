import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/design/theme/app_theme.dart';
import 'package:veltrunox_app/features/markers/marker.dart';
import 'package:veltrunox_app/features/markers/marker_controller.dart';
import 'package:veltrunox_app/features/markers/marker_panel.dart';
import 'package:veltrunox_app/features/markers/marker_source.dart';
import 'package:veltrunox_app/features/markers/marker_row.dart';

import 'fake_marker_source.dart';

const _label = LabelMarker(
  id: 1,
  version: 1,
  title: 'Einsatz',
  positionMs: 25000,
);
const _region = RegionMarker(
  id: 2,
  version: 3,
  title: 'Strophe',
  startMs: 10000,
  endMs: 67000,
);

class _Harness {
  _Harness(this.source);

  final FakeMarkerSource source;
  final played = <Marker>[];
  int? playhead = 42000;
}

Future<_Harness> _pump(
  WidgetTester tester,
  FakeMarkerSource source, {
  int? lengthMs = 180000,
  TimeSpan? activeRange,
}) async {
  tester.view.physicalSize = const Size(800, 1400);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final harness = _Harness(source);
  await tester.pumpWidget(
    ProviderScope(
      overrides: [markerSourceProvider.overrideWithValue(source)],
      child: MaterialApp(
        theme: buildAppTheme(),
        home: Scaffold(
          body: SingleChildScrollView(
            child: MarkerPanel(
              recordingId: 1,
              lengthMs: lengthMs,
              activeRange: activeRange,
              readPlayhead: () async => harness.playhead,
              onPlay: harness.played.add,
            ),
          ),
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return harness;
}

Finder _field(String label) => find.widgetWithText(TextField, label);

void main() {
  testWidgets('lists labels and regions in time order with their times', (
    tester,
  ) async {
    await _pump(tester, FakeMarkerSource([_label, _region]));

    expect(find.text('Label bei 0:25'), findsOneWidget);
    expect(find.text('Region 0:10–1:07'), findsOneWidget);
    final strophe = tester.getTopLeft(find.text('Strophe')).dy;
    final einsatz = tester.getTopLeft(find.text('Einsatz')).dy;
    expect(strophe, lessThan(einsatz));
  });

  testWidgets('the filter shows one kind of marker', (tester) async {
    await _pump(tester, FakeMarkerSource([_label, _region]));

    await tester.tap(find.text('Regionen'));
    await tester.pumpAndSettle();
    expect(find.text('Strophe'), findsOneWidget);
    expect(find.text('Einsatz'), findsNothing);

    await tester.tap(find.text('Labels'));
    await tester.pumpAndSettle();
    expect(find.text('Einsatz'), findsOneWidget);
    expect(find.text('Strophe'), findsNothing);
  });

  testWidgets('the play button and the row hand the marker over', (
    tester,
  ) async {
    final harness = await _pump(tester, FakeMarkerSource([_label, _region]));

    await tester.tap(find.byTooltip('Abspielen: Einsatz'));
    await tester.tap(find.text('Strophe'));

    expect(harness.played, [_label, _region]);
  });

  testWidgets('says why the list is empty', (tester) async {
    await _pump(tester, FakeMarkerSource([_region]));
    await tester.tap(find.text('Labels'));
    await tester.pumpAndSettle();
    expect(find.text('Keine Markierungen in dieser Auswahl.'), findsOneWidget);

    await _pump(tester, FakeMarkerSource([]));
    expect(find.text('Noch keine Markierungen.'), findsOneWidget);
  });

  testWidgets('shows a failed load with a way to try again', (tester) async {
    final source = FakeMarkerSource([_label])
      ..listOverride = () async => throw Exception('down');
    await _pump(tester, source);
    expect(
      find.text('Die Markierungen konnten nicht geladen werden.'),
      findsOneWidget,
    );

    source.listOverride = null;
    await tester.tap(find.text('Erneut versuchen'));
    await tester.pumpAndSettle();
    expect(find.text('Einsatz'), findsOneWidget);
  });

  group('region form', () {
    testWidgets('starts at the playhead, ends with the recording', (
      tester,
    ) async {
      await _pump(tester, FakeMarkerSource([]));

      await tester.tap(find.text('Region anlegen'));
      await tester.pumpAndSettle();

      expect(find.widgetWithText(TextField, '0:42'), findsOneWidget);
      expect(find.widgetWithText(TextField, '3:00'), findsOneWidget);
    });

    testWidgets('creates a region and shows it in the list', (tester) async {
      final source = FakeMarkerSource([]);
      await _pump(tester, source);
      await tester.tap(find.text('Region anlegen'));
      await tester.pumpAndSettle();

      await tester.enterText(_field('Name'), ' Refrain ');
      await tester.enterText(_field('Start'), '0:40');
      await tester.enterText(_field('Ende'), '1:32');
      await tester.tap(find.text('Speichern'));
      await tester.pumpAndSettle();

      expect(source.calls, ['addRegion Refrain 40000 92000']);
      expect(find.text('Refrain'), findsOneWidget);
      expect(find.text('Region 0:40–1:32'), findsOneWidget);
    });

    testWidgets('refuses invalid input at the field and sends nothing', (
      tester,
    ) async {
      final source = FakeMarkerSource([]);
      await _pump(tester, source);
      await tester.tap(find.text('Region anlegen'));
      await tester.pumpAndSettle();

      await tester.enterText(_field('Name'), '');
      await tester.enterText(_field('Start'), '2:00');
      await tester.enterText(_field('Ende'), '1:00');
      await tester.tap(find.text('Speichern'));
      await tester.pumpAndSettle();
      expect(find.text('Bitte einen Namen eingeben'), findsOneWidget);
      expect(find.text('Das Ende muss nach dem Start liegen'), findsOneWidget);

      await tester.enterText(_field('Name'), 'X');
      await tester.enterText(_field('Ende'), '9:00');
      await tester.tap(find.text('Speichern'));
      await tester.pumpAndSettle();
      expect(find.text('Außerhalb der Aufnahme (Länge 3:00)'), findsOneWidget);
      expect(source.calls, isEmpty);
    });

    testWidgets('edits name and span of a region, naming the version seen', (
      tester,
    ) async {
      final source = FakeMarkerSource([_region]);
      await _pump(tester, source);
      await tester.tap(find.byTooltip('Weitere Aktionen: Strophe'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Bearbeiten'));
      await tester.pumpAndSettle();

      await tester.enterText(_field('Name'), 'Strophe 1');
      await tester.enterText(_field('Ende'), '1:10');
      await tester.tap(find.text('Speichern'));
      await tester.pumpAndSettle();

      expect(source.calls, ['updateRegion 2 v3 Strophe 1 10000 70000']);
      expect(find.text('Strophe 1'), findsOneWidget);
    });

    testWidgets('a time stays exact when a region is edited', (tester) async {
      final source = FakeMarkerSource([
        const RegionMarker(
          id: 2,
          version: 1,
          title: 'Fein',
          startMs: 10500,
          endMs: 20250,
        ),
      ]);
      await _pump(tester, source);
      await tester.tap(find.byTooltip('Weitere Aktionen: Fein'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Bearbeiten'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Speichern'));
      await tester.pumpAndSettle();

      expect(source.calls, ['updateRegion 2 v1 Fein 10500 20250']);
    });
  });

  testWidgets('a label is renamed and moved in its own form', (tester) async {
    final source = FakeMarkerSource([_label]);
    await _pump(tester, source);
    await tester.tap(find.byTooltip('Weitere Aktionen: Einsatz'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Bearbeiten'));
    await tester.pumpAndSettle();

    expect(find.text('Markierung bearbeiten'), findsOneWidget);
    await tester.enterText(_field('Start'), '0:30');
    await tester.tap(find.text('Speichern'));
    await tester.pumpAndSettle();

    expect(source.calls, ['updateLabel 1 Einsatz 30000']);
    expect(find.text('Label bei 0:30'), findsOneWidget);
  });

  group('a label and a region differ only in the end', () {
    Future<void> openEdit(WidgetTester tester, String name) async {
      await tester.tap(find.byTooltip('Weitere Aktionen: $name'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Bearbeiten'));
      await tester.pumpAndSettle();
    }

    testWidgets('an end makes a label a region', (tester) async {
      final source = FakeMarkerSource([_label]);
      await _pump(tester, source);
      await openEdit(tester, 'Einsatz');
      await tester.enterText(_field('Ende'), '1:00');
      await tester.tap(find.text('Speichern'));
      await tester.pumpAndSettle();

      expect(source.calls, ['labelToRegion 1 v1 Einsatz 25000 60000']);
      expect(find.text('Region 0:25–1:00'), findsOneWidget);
      expect(find.textContaining('Label bei'), findsNothing);
    });

    testWidgets('no end makes a region a label', (tester) async {
      final source = FakeMarkerSource([_region]);
      await _pump(tester, source);
      await openEdit(tester, 'Strophe');
      await tester.enterText(_field('Ende'), '');
      await tester.tap(find.text('Speichern'));
      await tester.pumpAndSettle();

      expect(source.calls, ['regionToLabel 2 v3 Strophe 10000']);
      expect(find.text('Label bei 0:10'), findsOneWidget);
    });

    testWidgets('an end that is not after the start is refused', (
      tester,
    ) async {
      final source = FakeMarkerSource([_label]);
      await _pump(tester, source);
      await openEdit(tester, 'Einsatz');
      await tester.enterText(_field('Ende'), '0:10');
      await tester.tap(find.text('Speichern'));
      await tester.pumpAndSettle();

      expect(find.text('Das Ende muss nach dem Start liegen'), findsOneWidget);
      expect(source.calls, isEmpty);
    });
  });

  group('delete', () {
    Future<void> openDelete(WidgetTester tester, String name) async {
      await tester.tap(find.byTooltip('Weitere Aktionen: $name'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Löschen'));
      await tester.pumpAndSettle();
    }

    testWidgets('asks first, naming the marker, and keeps it on cancel', (
      tester,
    ) async {
      final source = FakeMarkerSource([_label]);
      await _pump(tester, source);
      await openDelete(tester, 'Einsatz');

      expect(find.textContaining('„Einsatz“ wird gelöscht'), findsOneWidget);
      await tester.tap(find.text('Abbrechen'));
      await tester.pumpAndSettle();

      expect(source.calls, isEmpty);
      expect(find.text('Einsatz'), findsOneWidget);
    });

    testWidgets('deletes after the confirmation', (tester) async {
      final source = FakeMarkerSource([_label, _region]);
      await _pump(tester, source);
      await openDelete(tester, 'Strophe');

      await tester.tap(find.widgetWithText(FilledButton, 'Löschen'));
      await tester.pumpAndSettle();

      expect(source.calls, ['deleteRegion 2']);
      expect(find.text('Strophe'), findsNothing);
    });

    testWidgets('tells the user when the server refuses, and reloads', (
      tester,
    ) async {
      final source = FakeMarkerSource([_label])..failure = Exception('409');
      await _pump(tester, source);
      final listings = source.listings;
      await openDelete(tester, 'Einsatz');

      await tester.tap(find.widgetWithText(FilledButton, 'Löschen'));
      await tester.pumpAndSettle();

      expect(find.textContaining('nicht übernommen'), findsOneWidget);
      expect(source.listings, greaterThan(listings));
    });

    testWidgets('says so when someone else changed the marker first', (
      tester,
    ) async {
      final source = FakeMarkerSource([_label])
        ..failure = const MarkerConflictException();
      await _pump(tester, source);
      final listings = source.listings;
      await openDelete(tester, 'Einsatz');

      await tester.tap(find.widgetWithText(FilledButton, 'Löschen'));
      await tester.pumpAndSettle();

      expect(find.textContaining('gleichzeitig geändert'), findsOneWidget);
      expect(find.textContaining('nicht übernommen'), findsNothing);
      expect(source.listings, greaterThan(listings));
    });
  });

  testWidgets('another client changing the markers updates the list', (
    tester,
  ) async {
    final source = FakeMarkerSource([_label]);
    await _pump(tester, source);

    source.markers.add(_region);
    source.announced.add(1);
    await tester.pumpAndSettle();

    expect(find.text('Strophe'), findsOneWidget);
  });

  group('active region', () {
    const other = RegionMarker(
      id: 3,
      version: 1,
      title: 'Refrain',
      startMs: 70000,
      endMs: 90000,
    );

    testWidgets('highlights the row of the region being played', (
      tester,
    ) async {
      await _pump(
        tester,
        FakeMarkerSource([_region, other]),
        activeRange: (startMs: 70000, endMs: 90000),
      );

      final rows = tester.widgetList<MarkerRow>(find.byType(MarkerRow));
      expect(
        {for (final row in rows) row.marker.title: row.active},
        {'Strophe': false, 'Refrain': true},
      );
    });

    testWidgets('highlights nothing without a play range', (tester) async {
      await _pump(tester, FakeMarkerSource([_region, other]));

      expect(
        tester
            .widgetList<MarkerRow>(find.byType(MarkerRow))
            .any((r) => r.active),
        isFalse,
      );
    });
  });

  group('regions from a selection', () {
    const other = RegionMarker(
      id: 3,
      version: 1,
      title: 'Refrain',
      startMs: 50000,
      endMs: 90000,
    );

    Future<FakeMarkerSource> pumpTicked(WidgetTester tester) async {
      final source = FakeMarkerSource([_label, _region, other]);
      await _pump(tester, source);
      await tester.tap(find.byType(Checkbox).first);
      await tester.tap(find.byType(Checkbox).last);
      await tester.pumpAndSettle();
      return source;
    }

    testWidgets('labels cannot be ticked, the bar needs two regions', (
      tester,
    ) async {
      final source = FakeMarkerSource([_label, _region, other]);
      await _pump(tester, source);
      expect(find.byType(Checkbox), findsNWidgets(2));
      expect(find.text('Regionen kombinieren'), findsNothing);

      await tester.tap(find.byType(Checkbox).first);
      await tester.pumpAndSettle();
      expect(find.text('1 ausgewählt'), findsOneWidget);
      final combine = tester.widget<OutlinedButton>(
        find.widgetWithText(OutlinedButton, 'Regionen kombinieren'),
      );
      expect(combine.onPressed, isNull);
    });

    testWidgets('combining replaces them by one region over their span', (
      tester,
    ) async {
      final source = await pumpTicked(tester);

      await tester.tap(find.text('Regionen kombinieren'));
      await tester.pumpAndSettle();
      expect(find.widgetWithText(TextField, '0:10'), findsOneWidget);
      expect(find.widgetWithText(TextField, '1:30'), findsOneWidget);
      await tester.tap(find.text('Speichern'));
      await tester.pumpAndSettle();

      expect(source.calls, [
        'updateRegion 2 v3 Strophe 10000 90000',
        'deleteRegion 3',
      ]);
      expect(source.markers.whereType<RegionMarker>().map((r) => r.id), [2]);
      expect(find.text('Refrain'), findsNothing);
      expect(find.text('2 ausgewählt'), findsNothing);
    });

    testWidgets('a new region from the selection keeps the originals', (
      tester,
    ) async {
      final source = await pumpTicked(tester);

      await tester.tap(find.text('Neue Region aus Auswahl'));
      await tester.pumpAndSettle();
      await tester.enterText(_field('Name'), 'Alles');
      await tester.tap(find.text('Speichern'));
      await tester.pumpAndSettle();

      expect(source.calls, ['addRegion Alles 10000 90000']);
      expect(source.markers.whereType<RegionMarker>().length, 3);
    });

    testWidgets('cancelling the form writes nothing and keeps the ticks', (
      tester,
    ) async {
      final source = await pumpTicked(tester);

      await tester.tap(find.text('Regionen kombinieren'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Abbrechen'));
      await tester.pumpAndSettle();

      expect(source.calls, isEmpty);
      expect(find.text('2 ausgewählt'), findsOneWidget);
    });
  });
}
