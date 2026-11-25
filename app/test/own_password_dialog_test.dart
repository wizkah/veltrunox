import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/design/theme/app_theme.dart';
import 'package:veltrunox_app/features/access/access_models.dart';
import 'package:veltrunox_app/features/own_password/own_password_dialog.dart';
import 'package:veltrunox_app/features/own_password/own_password_source.dart';
import 'package:veltrunox_app/shared_ui/server_reason.dart';

/// Changing the own password, in memory; a test can make it fail.
class _FakeOwnPassword implements OwnPasswordSource {
  _FakeOwnPassword({this.policyValue = PasswordPolicy.standard});

  final PasswordPolicy policyValue;
  final calls = <String>[];
  AccessFailure? failure;

  @override
  Future<PasswordPolicy> policy() async => policyValue;

  @override
  Future<void> change({required String current, required String next}) async {
    calls.add('change:$current:$next');
    final error = failure;
    if (error != null) throw AccessException(error);
  }
}

Future<(_FakeOwnPassword, List<bool>)> _open(
  WidgetTester tester, {
  PasswordPolicy policy = PasswordPolicy.standard,
}) async {
  tester.view.physicalSize = const Size(900, 1600);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final source = _FakeOwnPassword(policyValue: policy);
  final results = <bool>[];
  await tester.pumpWidget(
    ProviderScope(
      overrides: [ownPasswordSourceProvider.overrideWithValue(source)],
      child: MaterialApp(
        theme: buildAppTheme(),
        home: Builder(
          builder: (context) => Scaffold(
            body: TextButton(
              onPressed: () async =>
                  results.add(await OwnPasswordDialog.show(context)),
              child: const Text('öffnen'),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('öffnen'));
  await tester.pumpAndSettle();
  return (source, results);
}

Future<void> _fill(
  WidgetTester tester, {
  String current = 'old secret',
  String next = 'new secret!',
  String? repeat,
}) async {
  await tester.enterText(
    find.widgetWithText(TextField, 'Aktuelles Passwort'),
    current,
  );
  await tester.enterText(
    find.widgetWithText(TextField, 'Neues Passwort'),
    next,
  );
  await tester.enterText(
    find.widgetWithText(TextField, 'Neues Passwort wiederholen'),
    repeat ?? next,
  );
}

Future<void> _save(WidgetTester tester) async {
  await tester.tap(find.widgetWithText(FilledButton, 'Passwort ändern'));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('asks for the current and the new password and warns about '
      'the sign-out', (tester) async {
    await _open(tester);

    expect(find.text('Aktuelles Passwort'), findsOneWidget);
    expect(find.text('Neues Passwort'), findsOneWidget);
    expect(find.text('Neues Passwort wiederholen'), findsOneWidget);
    expect(find.text('Mindestens 6 Zeichen.'), findsOneWidget);
    expect(find.textContaining('überall abgemeldet'), findsOneWidget);
  });

  testWidgets('problems are named at their field and nothing is sent', (
    tester,
  ) async {
    final (source, results) = await _open(tester);

    await _save(tester);
    expect(find.text('Bitte ausfüllen.'), findsOneWidget);

    await _fill(tester, current: 'old secret', next: 'short', repeat: 'other');
    await _save(tester);
    expect(find.text('Bitte ausfüllen.'), findsNothing);
    expect(
      find.text('Die beiden neuen Passwörter sind verschieden.'),
      findsOneWidget,
    );
    expect(source.calls, isEmpty);
    expect(results, isEmpty);
  });

  testWidgets('a wrong current password is named at its field and the dialog '
      'stays', (tester) async {
    final (source, results) = await _open(tester);
    source.failure = const Refused(ServerReason('wrong_current_password'));

    await _fill(tester);
    await _save(tester);

    expect(find.text('Das aktuelle Passwort stimmt nicht.'), findsOneWidget);
    expect(find.widgetWithText(TextField, 'Neues Passwort'), findsOneWidget);
    expect(source.calls, ['change:old secret:new secret!']);
    expect(results, isEmpty);
  });

  testWidgets('a refusal of the new password shows at the new field', (
    tester,
  ) async {
    final (source, _) = await _open(tester);
    source.failure = const Refused(ServerReason('password_needs_digit'));

    await _fill(tester);
    await _save(tester);

    expect(find.text('Das Passwort braucht eine Ziffer.'), findsOneWidget);
  });

  testWidgets('another failure shows above the fields', (tester) async {
    final (source, _) = await _open(tester);
    source.failure = const Unavailable();

    await _fill(tester);
    await _save(tester);

    expect(find.text('Der Server ist nicht erreichbar.'), findsOneWidget);
  });

  testWidgets('a success closes the dialog and reports the change', (
    tester,
  ) async {
    final (source, results) = await _open(tester);

    await _fill(tester);
    await _save(tester);

    expect(source.calls, ['change:old secret:new secret!']);
    expect(results, [true]);
    expect(find.byType(AlertDialog), findsNothing);
  });

  testWidgets('cancelling changes nothing', (tester) async {
    final (source, results) = await _open(tester);

    await tester.tap(find.widgetWithText(OutlinedButton, 'Abbrechen'));
    await tester.pumpAndSettle();

    expect(source.calls, isEmpty);
    expect(results, [false]);
  });

  testWidgets('strict rules are asked for before anything is sent', (
    tester,
  ) async {
    final (source, _) = await _open(tester, policy: PasswordPolicy.strict);

    await _fill(tester, next: 'longenough1!');
    await _save(tester);

    expect(find.text('Es fehlt ein Großbuchstabe.'), findsOneWidget);
    expect(source.calls, isEmpty);
  });
}
