import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/design/theme/app_theme.dart';
import 'package:veltrunox_app/features/access/access_controller.dart';
import 'package:veltrunox_app/features/access/access_gate.dart';
import 'package:veltrunox_app/features/access/access_models.dart';
import 'package:veltrunox_app/features/access/account_menu.dart';

import 'fake_access_source.dart';

class _Setup {
  _Setup(this.source, this.tokens);

  final FakeAccessSource source;
  final MemoryTokenStore tokens;
}

Future<_Setup> _pump(
  WidgetTester tester, {
  FakeAccessSource? source,
  String? savedToken,
}) async {
  tester.view.physicalSize = const Size(800, 1000);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final fake = source ?? FakeAccessSource();
  final tokens = MemoryTokenStore(savedToken);
  await tester.pumpWidget(
    ProviderScope(
      overrides: [
        accessSourceProvider.overrideWithValue(fake),
        tokenStoreProvider.overrideWithValue(tokens),
      ],
      child: MaterialApp(
        theme: buildAppTheme(),
        home: AccessGate(
          builder: (_) => Scaffold(
            appBar: AppBar(
              title: const Text('Drinnen'),
              actions: [AccountMenu(onManageUsers: () {})],
            ),
          ),
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return _Setup(fake, tokens);
}

Future<void> _type(WidgetTester tester, String label, String text) async {
  await tester.enterText(find.widgetWithText(TextField, label), text);
}

void main() {
  testWidgets('a saved sign-in resumes without asking', (tester) async {
    final setup = await _pump(tester, savedToken: 'saved');

    expect(find.text('Drinnen'), findsOneWidget);
    expect(setup.source.calls, ['resume:saved']);
    expect(setup.source.sessionsStarted, 1);
  });

  testWidgets('a saved sign-in the server refuses is forgotten', (
    tester,
  ) async {
    final source = FakeAccessSource(failure: const WrongCredentials());
    final setup = await _pump(tester, source: source, savedToken: 'old');

    expect(find.text('Anmelden'), findsWidgets);
    expect(find.text('Drinnen'), findsNothing);
    expect(setup.tokens.token, isNull);
  });

  testWidgets('signing in with the right password opens the app and keeps the '
      'token', (tester) async {
    final setup = await _pump(tester);

    await _type(tester, 'Benutzername', 'ann');
    await _type(tester, 'Passwort', 'correct horse');
    await tester.tap(find.widgetWithText(FilledButton, 'Anmelden'));
    await tester.pumpAndSettle();

    expect(find.text('Drinnen'), findsOneWidget);
    expect(setup.source.calls, ['signIn:ann:correct horse']);
    expect(setup.tokens.token, 'token');
    expect(setup.source.sessionsStarted, 1);
  });

  testWidgets('a wrong password is explained and the form stays', (
    tester,
  ) async {
    final source = FakeAccessSource();
    await _pump(tester, source: source);
    source.failure = const WrongCredentials();

    await _type(tester, 'Benutzername', 'ann');
    await _type(tester, 'Passwort', 'nope');
    await tester.tap(find.widgetWithText(FilledButton, 'Anmelden'));
    await tester.pumpAndSettle();

    expect(find.text('Benutzername oder Passwort stimmt nicht.'), findsOne);
    expect(find.text('Drinnen'), findsNothing);
  });

  testWidgets('too many attempts tell how long to wait', (tester) async {
    final source = FakeAccessSource();
    await _pump(tester, source: source);
    source.failure = const TooManyAttempts(8);

    await tester.tap(find.widgetWithText(FilledButton, 'Anmelden'));
    await tester.pumpAndSettle();

    expect(
      find.text('Zu viele Fehlversuche. Bitte in 8 s erneut versuchen.'),
      findsOne,
    );
  });

  testWidgets('guest access is offered only while the server allows it', (
    tester,
  ) async {
    await _pump(tester);
    expect(find.text('Als Gast fortfahren'), findsNothing);
  });

  testWidgets('a guest enters without a password', (tester) async {
    final source = FakeAccessSource(guestEnabled: true);
    final setup = await _pump(tester, source: source);

    await tester.tap(find.text('Als Gast fortfahren'));
    await tester.pumpAndSettle();

    expect(find.text('Drinnen'), findsOneWidget);
    expect(setup.source.calls, ['guest']);
    // A guest has no token to keep.
    expect(setup.tokens.token, isNull);
  });

  testWidgets('the first start sets up the admin and checks the password', (
    tester,
  ) async {
    final source = FakeAccessSource(setupNeeded: true);
    final setup = await _pump(tester, source: source);
    expect(find.text('Ersten Admin einrichten'), findsOneWidget);

    await _type(tester, 'Benutzername', 'root');
    await _type(tester, 'Passwort', 'short');
    await _type(tester, 'Passwort wiederholen', 'other');
    await tester.tap(find.widgetWithText(FilledButton, 'Einrichten'));
    await tester.pumpAndSettle();
    expect(find.text('Mindestens 6 Zeichen.'), findsOneWidget);
    expect(find.text('Die Passwörter stimmen nicht überein.'), findsOneWidget);
    expect(setup.source.calls, isEmpty);

    await _type(tester, 'Passwort', 'a long enough password');
    await _type(tester, 'Passwort wiederholen', 'a long enough password');
    await tester.tap(find.widgetWithText(FilledButton, 'Einrichten'));
    await tester.pumpAndSettle();

    expect(find.text('Drinnen'), findsOneWidget);
    expect(setup.source.calls, ['setUp:root']);
    expect(setup.tokens.token, 'new-token');
  });

  testWidgets('an unreachable server can be tried again', (tester) async {
    final source = FakeAccessSource(unreachable: true);
    await _pump(tester, source: source);
    expect(find.text('Der Server ist nicht erreichbar.'), findsOneWidget);

    source.unreachable = false;
    await tester.tap(find.text('Erneut versuchen'));
    await tester.pumpAndSettle();

    expect(find.text('Benutzername'), findsOneWidget);
  });

  testWidgets('the password is hidden until the person asks to see it', (
    tester,
  ) async {
    await _pump(tester);
    EditableText field() => tester.widget<EditableText>(
      find.descendant(
        of: find.widgetWithText(TextField, 'Passwort'),
        matching: find.byType(EditableText),
      ),
    );
    expect(field().obscureText, isTrue);

    await tester.tap(find.byTooltip('Passwort anzeigen'));
    await tester.pump();
    expect(field().obscureText, isFalse);
  });

  group('account menu', () {
    testWidgets('signing out returns to the sign-in and forgets the token', (
      tester,
    ) async {
      final setup = await _pump(tester, savedToken: 'saved');

      await tester.tap(find.byTooltip('Konto'));
      await tester.pumpAndSettle();
      expect(find.text('Angemeldet als ann'), findsOneWidget);
      await tester.tap(find.text('Abmelden'));
      await tester.pumpAndSettle();

      expect(find.text('Benutzername'), findsOneWidget);
      expect(setup.tokens.token, isNull);
      expect(setup.source.calls, contains('signOut'));
    });

    testWidgets('only the admin sees the user administration', (tester) async {
      await _pump(tester, savedToken: 'saved');
      await tester.tap(find.byTooltip('Konto'));
      await tester.pumpAndSettle();
      expect(find.text('Benutzer verwalten'), findsNothing);
    });

    testWidgets('the admin opens the user administration', (tester) async {
      final source = FakeAccessSource(
        rights: const Rights(
          role: UserRole.admin,
          userName: 'root',
          canManageUsers: true,
        ),
      );
      await _pump(tester, source: source, savedToken: 'saved');
      await tester.tap(find.byTooltip('Konto'));
      await tester.pumpAndSettle();
      expect(find.text('Benutzer verwalten'), findsOneWidget);
    });
  });
}
