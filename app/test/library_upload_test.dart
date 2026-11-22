import 'dart:async';
import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/app.dart';
import 'package:veltrunox_app/features/access/access_models.dart';
import 'package:veltrunox_app/features/connection/connection_controller.dart';
import 'package:veltrunox_app/features/connection/connection_source.dart';
import 'package:veltrunox_app/features/connection/connection_status.dart';
import 'package:veltrunox_app/features/library/audio_file_picker.dart';
import 'package:veltrunox_app/features/library/library_controller.dart';
import 'package:veltrunox_app/features/library/recording_source.dart';
import 'package:veltrunox_app/features/library/recording_summary.dart';
import 'package:veltrunox_app/features/player/player_controller.dart';

import 'fake_access_source.dart';
import 'fake_player_source.dart';

class _Recordings implements RecordingSource {
  final titles = <int, String>{1: 'Soundcheck'};
  final calls = <String>[];
  Completer<UploadedRecording>? pendingUpload;
  RecordingActionException? failure;

  /// Refuses the first upload only.
  bool failFirst = false;

  @override
  Future<List<RecordingSummary>> listRecordings() async => [
    for (final entry in titles.entries)
      RecordingSummary(id: entry.key, title: entry.value),
  ];

  @override
  Stream<int> analysisChanges() => const Stream.empty();

  @override
  Stream<void> libraryChanges() => const Stream.empty();

  @override
  Future<void> reanalyse(int recordingId) async {}

  @override
  Future<UploadedRecording> upload(String fileName, Uint8List content) {
    calls.add('upload $fileName ${content.length}');
    final error = failure;
    if (error != null) return Future.error(error);
    if (failFirst) {
      failFirst = false;
      return Future.error(const RecordingActionException('this is not audio'));
    }
    final pending = pendingUpload;
    if (pending != null) return pending.future;
    titles[2] = 'New song';
    return Future.value(
      const UploadedRecording(id: 2, title: 'New song', existing: false),
    );
  }

  @override
  Future<void> rename(int recordingId, String title) async {
    calls.add('rename $recordingId $title');
    final error = failure;
    if (error != null) throw error;
    titles[recordingId] = title;
  }

  @override
  Future<void> delete(int recordingId) async {
    calls.add('delete $recordingId');
    titles.remove(recordingId);
  }
}

class _Picker implements AudioFilePicker {
  /// Further files chosen together with [next].
  final more = <PickedAudio>[];

  PickedAudio? next = PickedAudio(
    name: 'New song.wav',
    bytes: Uint8List.fromList([1, 2, 3]),
  );

  @override
  Future<List<PickedAudio>> pick() async => [?next, ...more];
}

class _Connection implements ConnectionSource {
  @override
  Stream<ConnectionStatus> watch() => Stream.value(ConnectionStatus.connected);

  @override
  Future<void> reconnect() async {}
}

Widget _app(_Recordings source, _Picker picker, Rights rights) => ProviderScope(
  overrides: [
    playerSourceProvider.overrideWithValue(FakePlayerSource()),
    recordingSourceProvider.overrideWithValue(source),
    audioFilePickerProvider.overrideWithValue(picker),
    ...signedInAs(rights),
    connectionSourceProvider.overrideWithValue(_Connection()),
  ],
  child: const VeltrunoxApp(),
);

const _user = Rights(role: UserRole.user, userName: 'ann');
const _admin = Rights(
  role: UserRole.admin,
  userName: 'root',
  canManageRecordings: true,
  canManageUsers: true,
);
const _noUpload = Rights(role: UserRole.guest, canUpload: false);

Future<void> _openDetails(WidgetTester tester) async {
  await tester.tap(find.text('Soundcheck'));
  await tester.pumpAndSettle();
}

void main() {
  setUp(() {});

  testWidgets('a user uploads a file and sees it in the library', (
    tester,
  ) async {
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(1400, 900);
    addTearDown(tester.view.reset);
    final source = _Recordings();
    await tester.pumpWidget(_app(source, _Picker(), _user));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Aufnahmen hochladen'));
    await tester.pumpAndSettle();

    expect(source.calls, ['upload New song.wav 3']);
    expect(find.text('Hochgeladen: New song'), findsOneWidget);
    expect(find.text('New song'), findsOneWidget);
  });

  testWidgets('while the file is on its way the button waits', (tester) async {
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(1400, 900);
    addTearDown(tester.view.reset);
    final source = _Recordings()..pendingUpload = Completer();
    await tester.pumpWidget(_app(source, _Picker(), _user));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Aufnahmen hochladen'));
    await tester.pump();

    expect(find.text('Wird hochgeladen: New song.wav'), findsOneWidget);
    await tester.tap(find.text('Aufnahmen hochladen'), warnIfMissed: false);
    await tester.pump();
    expect(source.calls, hasLength(1));
  });

  testWidgets('a refused file shows the reason of the server', (tester) async {
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(1400, 900);
    addTearDown(tester.view.reset);
    final source = _Recordings()
      ..failure = const RecordingActionException('this is not audio');
    await tester.pumpWidget(_app(source, _Picker(), _user));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Aufnahmen hochladen'));
    await tester.pumpAndSettle();

    expect(find.text('New song.wav: this is not audio'), findsOneWidget);
    await tester.tap(find.byTooltip('Meldung schließen'));
    await tester.pump();
    expect(find.text('New song.wav: this is not audio'), findsNothing);
  });

  testWidgets('several files are sent one after the other and summed up', (
    tester,
  ) async {
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(1400, 900);
    addTearDown(tester.view.reset);
    final source = _Recordings();
    final picker = _Picker()
      ..more.add(PickedAudio(name: 'Second.wav', bytes: Uint8List(5)));
    await tester.pumpWidget(_app(source, picker, _user));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Aufnahmen hochladen'));
    await tester.pumpAndSettle();

    expect(source.calls, ['upload New song.wav 3', 'upload Second.wav 5']);
    expect(find.text('2 hochgeladen'), findsOneWidget);
  });

  testWidgets('a refused file does not stop the others', (tester) async {
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(1400, 900);
    addTearDown(tester.view.reset);
    final source = _Recordings()..failFirst = true;
    final picker = _Picker()
      ..more.add(PickedAudio(name: 'Second.wav', bytes: Uint8List(5)));
    await tester.pumpWidget(_app(source, picker, _user));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Aufnahmen hochladen'));
    await tester.pumpAndSettle();

    expect(source.calls, hasLength(2));
    expect(
      find.textContaining('1 hochgeladen, 1 fehlgeschlagen'),
      findsOneWidget,
    );
    expect(find.textContaining('New song.wav: this is not audio'), findsOne);
  });

  testWidgets('cancelling the file dialog sends nothing', (tester) async {
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(1400, 900);
    addTearDown(tester.view.reset);
    final source = _Recordings();
    await tester.pumpWidget(_app(source, _Picker()..next = null, _user));
    await tester.pumpAndSettle();

    await tester.tap(find.text('Aufnahmen hochladen'));
    await tester.pumpAndSettle();

    expect(source.calls, isEmpty);
  });

  testWidgets('without the right to upload there is no upload button', (
    tester,
  ) async {
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(1400, 900);
    addTearDown(tester.view.reset);
    await tester.pumpWidget(_app(_Recordings(), _Picker(), _noUpload));
    await tester.pumpAndSettle();

    expect(find.text('Aufnahmen hochladen'), findsNothing);
  });

  testWidgets('only those who manage recordings see rename and delete', (
    tester,
  ) async {
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(1400, 900);
    addTearDown(tester.view.reset);
    await tester.pumpWidget(_app(_Recordings(), _Picker(), _user));
    await tester.pumpAndSettle();
    await _openDetails(tester);

    expect(find.text('Abspielen'), findsWidgets);
    expect(find.text('Umbenennen'), findsNothing);
    expect(find.text('Löschen'), findsNothing);
  });

  testWidgets('the admin renames a recording', (tester) async {
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(1400, 900);
    addTearDown(tester.view.reset);
    final source = _Recordings();
    await tester.pumpWidget(_app(source, _Picker(), _admin));
    await tester.pumpAndSettle();
    await _openDetails(tester);

    await tester.tap(find.text('Umbenennen'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField).last, '  Intro ');
    await tester.tap(find.text('Speichern'));
    await tester.pumpAndSettle();

    expect(source.calls, ['rename 1 Intro']);
    expect(find.text('Intro'), findsWidgets);
    expect(find.text('Soundcheck'), findsNothing);
  });

  testWidgets('an empty title is not sent', (tester) async {
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(1400, 900);
    addTearDown(tester.view.reset);
    final source = _Recordings();
    await tester.pumpWidget(_app(source, _Picker(), _admin));
    await tester.pumpAndSettle();
    await _openDetails(tester);

    await tester.tap(find.text('Umbenennen'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField).last, '   ');
    await tester.tap(find.text('Speichern'));
    await tester.pump();

    expect(find.text('Der Titel darf nicht leer sein.'), findsOneWidget);
    expect(source.calls, isEmpty);
  });

  testWidgets('deleting needs a confirmation that names the recording', (
    tester,
  ) async {
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(1400, 900);
    addTearDown(tester.view.reset);
    final source = _Recordings();
    await tester.pumpWidget(_app(source, _Picker(), _admin));
    await tester.pumpAndSettle();
    await _openDetails(tester);

    await tester.tap(find.text('Löschen'));
    await tester.pumpAndSettle();
    expect(find.textContaining('„Soundcheck“ wird'), findsOneWidget);
    await tester.tap(find.text('Abbrechen'));
    await tester.pumpAndSettle();
    expect(source.calls, isEmpty);

    await tester.tap(find.text('Löschen'));
    await tester.pumpAndSettle();
    await tester.tap(
      find.descendant(
        of: find.byType(AlertDialog),
        matching: find.text('Löschen'),
      ),
    );
    await tester.pumpAndSettle();

    expect(source.calls, ['delete 1']);
    expect(find.text('Soundcheck'), findsNothing);
    expect(find.text('Noch keine Aufnahmen.'), findsOneWidget);
  });

  testWidgets('a refused rename is shown and changes nothing', (tester) async {
    tester.view.devicePixelRatio = 1;
    tester.view.physicalSize = const Size(1400, 900);
    addTearDown(tester.view.reset);
    final source = _Recordings();
    await tester.pumpWidget(_app(source, _Picker(), _admin));
    await tester.pumpAndSettle();
    await _openDetails(tester);
    source.failure = const RecordingActionException('not allowed');

    await tester.tap(find.text('Umbenennen'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(TextField).last, 'Other');
    await tester.tap(find.text('Speichern'));
    await tester.pumpAndSettle();

    expect(
      find.text('Das ist nicht gelungen: Dafür fehlt die Berechtigung.'),
      findsOneWidget,
    );
    expect(source.titles[1], 'Soundcheck');
  });
}
