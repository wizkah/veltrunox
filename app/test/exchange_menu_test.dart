import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/design/theme/app_theme.dart';
import 'package:veltrunox_app/features/access/access_models.dart';
import 'package:veltrunox_app/features/exchange/exchange_menu.dart';
import 'package:veltrunox_app/features/exchange/exchange_models.dart';
import 'package:veltrunox_app/features/exchange/exchange_source.dart';
import 'package:veltrunox_app/features/exchange/export_controller.dart';
import 'package:veltrunox_app/features/exchange/file_exchange.dart';
import 'package:veltrunox_app/features/markers/marker_controller.dart';
import 'package:veltrunox_app/features/markers/marker_panel.dart';
import 'package:veltrunox_app/shared_ui/server_reason.dart';

import 'fake_exchange.dart';
import 'fake_marker_source.dart';

const _ready = RecordingImport(
  recordingId: 1,
  status: ImportStatus.ready,
  added: 2,
  changed: 1,
  unchanged: 3,
  removed: 4,
);
const _conflict = RecordingImport(
  recordingId: 2,
  status: ImportStatus.conflict,
  issues: [
    ImportIssue(
      kind: ImportEntryKind.label,
      index: 2,
      reason: ServerReason('bookmark_conflict', args: [3]),
    ),
  ],
);
const _invalid = RecordingImport(
  recordingId: 3,
  status: ImportStatus.invalid,
  issues: [
    ImportIssue(
      kind: ImportEntryKind.region,
      index: 0,
      reason: ServerReason('outside_recording'),
    ),
  ],
);
const _unknown = RecordingImport(
  recordingId: 9,
  status: ImportStatus.unknownRecording,
);

class _Harness {
  _Harness({
    FakeExchangeSource? source,
    this.canImport = true,
    PickedText? picked,
  }) : source = source ?? FakeExchangeSource(),
       saver = FakeFileSaver(),
       picker = FakeTextFilePicker(
         picked ?? const PickedText(name: 'marks.json', content: '{}'),
       );

  final FakeExchangeSource source;
  final FakeFileSaver saver;
  final FakeTextFilePicker picker;
  final bool canImport;
}

Future<void> _pump(WidgetTester tester, _Harness harness) async {
  tester.view.physicalSize = const Size(900, 1400);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  await tester.pumpWidget(
    ProviderScope(
      overrides: [
        exchangeSourceProvider.overrideWithValue(harness.source),
        fileSaverProvider.overrideWithValue(harness.saver),
        textFilePickerProvider.overrideWithValue(harness.picker),
      ],
      child: MaterialApp(
        theme: buildAppTheme(),
        home: Scaffold(
          body: Center(
            child: ExchangeMenu(
              scope: const ExportScope.recording(id: 1, title: 'Song'),
              canImport: harness.canImport,
              titleOf: (id) => id == 1 ? 'Soundcheck' : null,
            ),
          ),
        ),
      ),
    ),
  );
}

Future<void> _openMenu(WidgetTester tester) async {
  await tester.tap(find.byTooltip('Export und Import'));
  await tester.pumpAndSettle();
}

Future<void> _startImport(WidgetTester tester) async {
  await _openMenu(tester);
  await tester.tap(find.text('Importieren …'));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('export as JSON saves a file and says so', (tester) async {
    final harness = _Harness(source: FakeExchangeSource(exported: 'data'));
    await _pump(tester, harness);

    await _openMenu(tester);
    await tester.tap(find.text('Exportieren als JSON'));
    await tester.pumpAndSettle();

    expect(harness.saver.saved.single.fileName, 'Song.bookmarks.json');
    expect(harness.saver.saved.single.content, 'data');
    expect(find.text('Gespeichert: Song.bookmarks.json'), findsOneWidget);
  });

  testWidgets('export as CSV asks for the table', (tester) async {
    final harness = _Harness();
    await _pump(tester, harness);

    await _openMenu(tester);
    await tester.tap(find.text('Exportieren als CSV'));
    await tester.pumpAndSettle();

    expect(harness.source.calls, ['export 1 csv']);
    expect(harness.saver.saved.single.mimeType, 'text/csv');
  });

  testWidgets('a failed export is reported', (tester) async {
    final harness = _Harness();
    harness.saver.failure = 'no space';
    await _pump(tester, harness);

    await _openMenu(tester);
    await tester.tap(find.text('Exportieren als JSON'));
    await tester.pumpAndSettle();

    expect(
      find.textContaining('Der Export ist fehlgeschlagen'),
      findsOneWidget,
    );
  });

  testWidgets('without the right to edit there is export but no import', (
    tester,
  ) async {
    await _pump(tester, _Harness(canImport: false));

    await _openMenu(tester);

    expect(find.text('Exportieren als JSON'), findsOneWidget);
    expect(find.text('Importieren …'), findsNothing);
  });

  testWidgets('a chosen file shows a preview and writes nothing', (
    tester,
  ) async {
    final harness = _Harness(
      source: FakeExchangeSource()
        ..report = const ImportReport(
          dryRun: true,
          recordings: [_ready, _conflict, _invalid, _unknown],
        ),
    );
    await _pump(tester, harness);

    await _startImport(tester);

    expect(find.text('Import-Vorschau'), findsOneWidget);
    // The recording's title, or its number when it is not known.
    expect(find.text('Soundcheck'), findsOneWidget);
    expect(find.text('Aufnahme 2'), findsOneWidget);
    expect(find.text('Neu 2 · Geändert 1 · Unverändert 3'), findsOneWidget);
    // What the import removes is said before anything is applied.
    expect(find.textContaining('Entfernt 4'), findsOneWidget);
    expect(
      find.text(
        'Label 3: Die Markierung, aus der dieser Eintrag stammt, '
        'wurde inzwischen geändert.',
      ),
      findsOneWidget,
    );
    expect(
      find.text('Region 1: Der Eintrag liegt hinter dem Ende der Aufnahme.'),
      findsOneWidget,
    );
    expect(
      find.text('Diese Aufnahme gibt es auf dem Server nicht.'),
      findsOneWidget,
    );
    expect(harness.source.calls, ['import dryRun=true overwrite=false json']);
  });

  testWidgets('apply writes only the ready recordings and reports', (
    tester,
  ) async {
    final harness = _Harness(
      source: FakeExchangeSource()
        ..report = const ImportReport(
          dryRun: true,
          recordings: [_ready, _conflict],
        ),
    );
    await _pump(tester, harness);
    await _startImport(tester);

    await tester.tap(find.text('Anwenden'));
    await tester.pumpAndSettle();

    expect(
      harness.source.calls.last,
      'import dryRun=false overwrite=false json',
    );
    expect(
      find.text(
        '1 von 2 Aufnahmen wurden aktualisiert, 1 blieben unverändert.',
      ),
      findsOneWidget,
    );
    await tester.tap(find.text('Schließen'));
    await tester.pumpAndSettle();
    expect(find.text('Import-Vorschau'), findsNothing);
  });

  testWidgets('cancelling leaves everything as it was', (tester) async {
    final harness = _Harness(
      source: FakeExchangeSource()
        ..report = const ImportReport(dryRun: true, recordings: [_ready]),
    );
    await _pump(tester, harness);
    await _startImport(tester);

    await tester.tap(find.text('Abbrechen'));
    await tester.pumpAndSettle();

    expect(find.text('Import-Vorschau'), findsNothing);
    expect(harness.source.calls, ['import dryRun=true overwrite=false json']);
  });

  testWidgets('overwriting offers itself with a conflict and refreshes', (
    tester,
  ) async {
    final harness = _Harness(
      source: FakeExchangeSource()
        ..report = const ImportReport(dryRun: true, recordings: [_conflict])
        ..reportWithOverwrite = const ImportReport(
          dryRun: true,
          recordings: [
            RecordingImport(
              recordingId: 2,
              status: ImportStatus.ready,
              changed: 1,
            ),
          ],
        ),
    );
    await _pump(tester, harness);
    await _startImport(tester);
    expect(find.text('Keine Aufnahme kann übernommen werden.'), findsOneWidget);
    final apply = find.widgetWithText(FilledButton, 'Anwenden');
    expect(tester.widget<FilledButton>(apply).onPressed, isNull);

    await tester.tap(find.byType(Switch));
    await tester.pumpAndSettle();

    expect(harness.source.calls.last, 'import dryRun=true overwrite=true json');
    expect(find.text('Bereit'), findsOneWidget);
    expect(tester.widget<FilledButton>(apply).onPressed, isNotNull);
  });

  testWidgets('a file that cannot be read shows one error and no preview', (
    tester,
  ) async {
    final harness = _Harness(
      source: FakeExchangeSource()
        ..failure = const Refused(ServerReason('csv_invalid', args: [5])),
    );
    await _pump(tester, harness);

    await _startImport(tester);

    expect(find.textContaining('Zeile 5'), findsOneWidget);
    expect(find.text('Anwenden'), findsNothing);
  });

  testWidgets('cancelling the file dialog opens nothing', (tester) async {
    final harness = _Harness();
    harness.picker.file = null;
    await _pump(tester, harness);

    await _startImport(tester);

    expect(find.text('Import-Vorschau'), findsNothing);
    expect(harness.source.calls, isEmpty);
  });

  testWidgets('the marker panel carries what the caller offers', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(800, 1400);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          markerSourceProvider.overrideWithValue(FakeMarkerSource([])),
        ],
        child: MaterialApp(
          theme: buildAppTheme(),
          home: Scaffold(
            body: MarkerPanel(
              recordingId: 1,
              lengthMs: 1000,
              readPlayhead: () async => 0,
              onPlay: (_) {},
              exchange: const Text('exchange slot'),
            ),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('exchange slot'), findsOneWidget);
    expect(find.text('Region anlegen'), findsOneWidget);
  });
}
