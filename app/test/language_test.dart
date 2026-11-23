import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/app.dart';
import 'package:veltrunox_app/features/connection/connection_controller.dart';
import 'package:veltrunox_app/features/connection/connection_source.dart';
import 'package:veltrunox_app/features/connection/connection_status.dart';
import 'package:veltrunox_app/features/language/language_controller.dart';
import 'package:veltrunox_app/features/language/language_store.dart';
import 'package:veltrunox_app/features/library/library_controller.dart';
import 'package:veltrunox_app/features/library/recording_summary.dart';
import 'package:veltrunox_app/features/player/player_controller.dart';
import 'package:veltrunox_app/shared_ui/app_language.dart';
import 'package:veltrunox_app/shared_ui/server_reason.dart';

import 'fake_access_source.dart';
import 'fake_player_source.dart';
import 'no_recording_actions.dart';

import 'package:veltrunox_app/features/library/recording_source.dart';

class _Recordings with NoRecordingActions implements RecordingSource {
  @override
  Future<List<RecordingSummary>> listRecordings() async => [
    const RecordingSummary(id: 1, title: 'Soundcheck'),
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

class _Store implements LanguageStore {
  AppLanguage? saved;

  @override
  Future<AppLanguage?> read() async => saved;

  @override
  Future<void> write(AppLanguage language) async => saved = language;
}

Widget _app(_Store store, {AppLanguage start = AppLanguage.de}) =>
    ProviderScope(
      overrides: [
        playerSourceProvider.overrideWithValue(FakePlayerSource()),
        recordingSourceProvider.overrideWithValue(_Recordings()),
        languageStoreProvider.overrideWithValue(store),
        initialLanguageProvider.overrideWithValue(start),
        ...signedInAs(),
        connectionSourceProvider.overrideWithValue(_Connection()),
      ],
      child: const VeltrunoxApp(),
    );

void main() {
  tearDown(() => AppLanguage.current = AppLanguage.de);

  testWidgets('the interface starts in the language it is given', (
    tester,
  ) async {
    await tester.pumpWidget(_app(_Store(), start: AppLanguage.en));
    await tester.pumpAndSettle();

    expect(find.text('Recordings'), findsWidgets);
    expect(find.text('Aufnahmen'), findsNothing);
    expect(find.text('Connected'), findsOneWidget);
  });

  testWidgets('switching the language rewrites every text and keeps it', (
    tester,
  ) async {
    final store = _Store();
    await tester.pumpWidget(_app(store));
    await tester.pumpAndSettle();
    expect(find.text('Aufnahmen'), findsWidgets);

    await tester.tap(find.byTooltip('Sprache'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('English'));
    await tester.pumpAndSettle();

    expect(find.text('Recordings'), findsWidgets);
    expect(find.text('Aufnahmen'), findsNothing);
    expect(find.text('Soundcheck'), findsOneWidget);
    expect(store.saved, AppLanguage.en);

    await tester.tap(find.byTooltip('Language'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Deutsch'));
    await tester.pumpAndSettle();
    expect(find.text('Aufnahmen'), findsWidgets);
    expect(store.saved, AppLanguage.de);
  });

  test('a language is found by the code a browser reports', () {
    expect(AppLanguage.fromCode('de-AT'), AppLanguage.de);
    expect(AppLanguage.fromCode('en_GB'), AppLanguage.en);
    expect(AppLanguage.fromCode('fr'), isNull);
    expect(AppLanguage.fromCode(null), isNull);
  });

  test('reasons of the server come in the language of the interface', () {
    AppLanguage.current = AppLanguage.de;
    expect(
      serverReasonText(const ServerReason('bad_credentials')),
      'Benutzername oder Passwort stimmt nicht.',
    );
    AppLanguage.current = AppLanguage.en;
    expect(
      serverReasonText(const ServerReason('bad_credentials')),
      'Wrong user name or password.',
    );
    expect(
      serverReasonText(const ServerReason('new_code', message: 'It broke')),
      'It broke.',
    );
  });

  test('numbers use the decimal separator of the language', () {
    AppLanguage.current = AppLanguage.de;
    expect(decimalText(96.5), '96,5');
    AppLanguage.current = AppLanguage.en;
    expect(decimalText(96.5), '96.5');
  });
}
