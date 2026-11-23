import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/app.dart';
import 'package:veltrunox_app/features/connection/connection_controller.dart';
import 'package:veltrunox_app/features/connection/connection_source.dart';
import 'package:veltrunox_app/features/connection/connection_status.dart';
import 'package:veltrunox_app/features/library/analysis_info.dart';
import 'package:veltrunox_app/features/library/library_controller.dart';
import 'package:veltrunox_app/features/library/recording_source.dart';
import 'package:veltrunox_app/features/library/recording_summary.dart';
import 'package:veltrunox_app/features/player/player_controller.dart';

import 'fake_access_source.dart';
import 'fake_player_source.dart';
import 'no_recording_actions.dart';

class _FakeSource with NoRecordingActions implements RecordingSource {
  _FakeSource(this.result);

  final Future<List<RecordingSummary>> Function() result;
  final changes = StreamController<int>.broadcast();
  final libraryChanged = StreamController<void>.broadcast();
  final reanalysed = <int>[];

  @override
  Future<List<RecordingSummary>> listRecordings() => result();

  @override
  Stream<int> analysisChanges() => changes.stream;

  @override
  Stream<void> libraryChanges() => libraryChanged.stream;

  @override
  Future<void> reanalyse(int recordingId) async => reanalysed.add(recordingId);
}

class _FakeConnection implements ConnectionSource {
  _FakeConnection([this.status = ConnectionStatus.connected]);

  final ConnectionStatus status;
  int reconnects = 0;

  @override
  Stream<ConnectionStatus> watch() => Stream.value(status);

  @override
  Future<void> reconnect() async => reconnects++;
}

Widget _app(
  RecordingSource source, [
  FakePlayerSource? player,
  _FakeConnection? connection,
]) => ProviderScope(
  overrides: [
    playerSourceProvider.overrideWithValue(player ?? FakePlayerSource()),
    recordingSourceProvider.overrideWithValue(source),
    ...signedInAs(),
    connectionSourceProvider.overrideWithValue(connection ?? _FakeConnection()),
  ],
  child: const VeltrunoxApp(),
);

/// Pending analyses show a spinner that never settles, so settle by frames.
Future<void> _pumpFrames(WidgetTester tester) async {
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 100));
}

void main() {
  testWidgets('shows the connection status', (tester) async {
    await tester.pumpWidget(_app(_FakeSource(() async => [])));
    await tester.pumpAndSettle();

    expect(find.text('Verbunden'), findsOneWidget);
  });

  testWidgets('shows the recordings from the source', (tester) async {
    await tester.pumpWidget(
      _app(
        _FakeSource(
          () async => [const RecordingSummary(id: 1, title: 'Soundcheck')],
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('Soundcheck'), findsOneWidget);
  });

  testWidgets('shows a retry action when loading fails', (tester) async {
    await tester.pumpWidget(
      _app(_FakeSource(() async => throw Exception('down'))),
    );
    await tester.pumpAndSettle();

    expect(find.text('Erneut versuchen'), findsOneWidget);
  });

  testWidgets('the play button loads the recording, plays it and leads to the '
      'player', (tester) async {
    final player = FakePlayerSource();
    await tester.pumpWidget(
      _app(
        _FakeSource(
          () async => [const RecordingSummary(id: 4, title: 'Soundcheck')],
        ),
        player,
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('Abspielen: Soundcheck'));
    await tester.pump();

    expect(player.calls, ['select 4', 'play']);
    expect(find.text('Wiedergabe wird geladen …'), findsOneWidget);
  });

  testWidgets('tapping a row shows its details, which open it in the player', (
    tester,
  ) async {
    final player = FakePlayerSource();
    await tester.pumpWidget(
      _app(
        _FakeSource(
          () async => [const RecordingSummary(id: 4, title: 'Soundcheck')],
        ),
        player,
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Soundcheck'));
    await tester.pumpAndSettle();
    expect(player.calls, isEmpty);
    expect(find.text('Zurück zur Liste'), findsOneWidget);

    await tester.tap(find.text('Im Player öffnen'));
    await tester.pump();
    expect(player.calls, ['select 4']);
  });

  testWidgets('the details stand beside the list on a wide screen', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1400, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    await tester.pumpWidget(
      _app(
        _FakeSource(
          () async => [
            const RecordingSummary(id: 4, title: 'Soundcheck'),
            const RecordingSummary(id: 5, title: 'Zugabe'),
          ],
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.tap(find.text('Zugabe'));
    await tester.pumpAndSettle();

    expect(find.text('Soundcheck'), findsOneWidget);
    expect(find.text('Im Player öffnen'), findsOneWidget);
    expect(find.text('Zurück zur Liste'), findsNothing);
  });

  testWidgets('search, filter and order narrow down the list', (tester) async {
    await tester.pumpWidget(
      _app(
        _FakeSource(
          () async => [
            const RecordingSummary(id: 1, title: 'Blues'),
            const RecordingSummary(
              id: 2,
              title: 'Ballade',
              analysis: AnalysisInfo(status: AnalysisStatus.running),
            ),
            const RecordingSummary(id: 3, title: 'Zugabe'),
          ],
        ),
      ),
    );
    await _pumpFrames(tester);

    await tester.enterText(find.byType(TextField), 'bl');
    await _pumpFrames(tester);
    expect(find.text('Blues'), findsOneWidget);
    expect(find.text('Zugabe'), findsNothing);

    await tester.enterText(find.byType(TextField), 'xyz');
    await _pumpFrames(tester);
    expect(find.text('Keine Aufnahme passt zur Suche.'), findsOneWidget);

    await tester.enterText(find.byType(TextField), '');
    await tester.tap(find.text('Wird analysiert'));
    await _pumpFrames(tester);
    expect(find.text('Ballade'), findsOneWidget);
    expect(find.text('Blues'), findsNothing);
  });

  testWidgets('a lost connection is announced and can be reopened', (
    tester,
  ) async {
    final connection = _FakeConnection(ConnectionStatus.disconnected);
    await tester.pumpWidget(
      _app(_FakeSource(() async => []), null, connection),
    );
    await tester.pumpAndSettle();

    expect(
      find.text('Die Verbindung zum Server ist verloren.'),
      findsOneWidget,
    );
    await tester.tap(find.text('Neu verbinden'));
    expect(connection.reconnects, 1);
  });

  testWidgets('shows key, tempo and tuning of an analysed recording', (
    tester,
  ) async {
    await tester.pumpWidget(
      _app(
        _FakeSource(
          () async => [
            const RecordingSummary(
              id: 1,
              title: 'Soundcheck',
              analysis: AnalysisInfo(
                status: AnalysisStatus.done,
                values: AnalysisValues(
                  tuningHz: 441.2,
                  keyTonic: 1,
                  keyMinor: true,
                  tempoBpm: 148,
                ),
              ),
            ),
          ],
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.text('C#-Moll · 148,0 bpm · 441,2 Hz'), findsOneWidget);
  });

  testWidgets('shows the state of a pending or failed analysis', (
    tester,
  ) async {
    await tester.pumpWidget(
      _app(
        _FakeSource(
          () async => [
            const RecordingSummary(
              id: 1,
              title: 'A',
              analysis: AnalysisInfo(status: AnalysisStatus.running),
            ),
            const RecordingSummary(
              id: 2,
              title: 'B',
              analysis: AnalysisInfo(
                status: AnalysisStatus.failed,
                failure: 'analysis_too_short',
              ),
            ),
          ],
        ),
      ),
    );
    await _pumpFrames(tester);

    expect(find.text('Analyse läuft … 0 %'), findsOneWidget);
    expect(find.text('Analyse fehlgeschlagen: Die Aufnahme ist zu kurz für die Analyse.'), findsOneWidget);
  });

  testWidgets('a finished analysis shows up without reloading by hand', (
    tester,
  ) async {
    var analysed = false;
    final source = _FakeSource(
      () async => [
        RecordingSummary(
          id: 1,
          title: 'Soundcheck',
          analysis: analysed
              ? const AnalysisInfo(
                  status: AnalysisStatus.done,
                  values: AnalysisValues(
                    tuningHz: 440,
                    keyTonic: 9,
                    keyMinor: true,
                    tempoBpm: 90,
                  ),
                )
              : const AnalysisInfo(status: AnalysisStatus.running),
        ),
      ],
    );
    await tester.pumpWidget(_app(source));
    await _pumpFrames(tester);
    expect(find.text('Analyse läuft … 0 %'), findsOneWidget);

    analysed = true;
    source.changes.add(1);
    await tester.pumpAndSettle();

    expect(find.text('A-Moll · 90,0 bpm · 440,0 Hz'), findsOneWidget);
  });

  testWidgets('a running analysis shows how far it has come', (tester) async {
    final source = _FakeSource(
      () async => [
        const RecordingSummary(
          id: 1,
          title: 'Soundcheck',
          lengthMs: 95000,
          analysis: AnalysisInfo(status: AnalysisStatus.running, percent: 40),
        ),
      ],
    );
    await tester.pumpWidget(_app(source));
    await _pumpFrames(tester);

    expect(find.text('Analyse läuft … 40 %'), findsOneWidget);

    await tester.tap(find.text('Soundcheck'));
    await _pumpFrames(tester);
    expect(
      find.textContaining('Länge: 1:35', findRichText: true),
      findsOneWidget,
    );
  });

  testWidgets('a change of the library by another client reloads the list', (
    tester,
  ) async {
    var titles = ['Soundcheck'];
    final source = _FakeSource(
      () async => [
        for (final (index, title) in titles.indexed)
          RecordingSummary(id: index + 1, title: title),
      ],
    );
    await tester.pumpWidget(_app(source));
    await _pumpFrames(tester);
    expect(find.text('Soundcheck'), findsOneWidget);

    titles = ['Soundcheck', 'Zugabe'];
    source.libraryChanged.add(null);
    await tester.pumpAndSettle();

    expect(find.text('Zugabe'), findsOneWidget);
  });

  testWidgets('the details ask for a new analysis', (tester) async {
    final source = _FakeSource(
      () async => [const RecordingSummary(id: 5, title: 'Soundcheck')],
    );
    await tester.pumpWidget(_app(source));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Soundcheck'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Erneut analysieren'));

    expect(source.reanalysed, [5]);
  });

  testWidgets('no new analysis can be asked for while one is pending', (
    tester,
  ) async {
    final source = _FakeSource(
      () async => [
        const RecordingSummary(
          id: 5,
          title: 'Soundcheck',
          analysis: AnalysisInfo(status: AnalysisStatus.queued),
        ),
      ],
    );
    await tester.pumpWidget(_app(source));
    await _pumpFrames(tester);

    await tester.tap(find.text('Soundcheck'));
    await _pumpFrames(tester);
    await tester.tap(find.text('Erneut analysieren'));

    expect(source.reanalysed, isEmpty);
  });
}
