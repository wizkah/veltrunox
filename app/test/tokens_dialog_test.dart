import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/design/theme/app_theme.dart';
import 'package:veltrunox_app/features/access/access_models.dart';
import 'package:veltrunox_app/features/tokens/token_models.dart';
import 'package:veltrunox_app/features/tokens/token_source.dart';
import 'package:veltrunox_app/features/tokens/tokens_dialog.dart';
import 'package:veltrunox_app/shared_ui/server_reason.dart';

import 'fake_tokens.dart';

const _used = ApiToken(
  id: 1,
  name: 'Claude',
  scope: TokenScope.editBookmarks,
  createdAt: 1700000000,
  lastUsed: 1700003600,
);
const _unused = ApiToken(
  id: 2,
  name: 'Skript',
  scope: TokenScope.readOnly,
  createdAt: 1700000000,
);

Future<(FakeTokenSource, FakeClipboard)> _open(
  WidgetTester tester, {
  List<ApiToken> tokens = const [_used, _unused],
}) async {
  tester.view.physicalSize = const Size(900, 1600);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final source = FakeTokenSource(tokens);
  final clipboard = FakeClipboard();
  await tester.pumpWidget(
    ProviderScope(
      overrides: [
        tokenSourceProvider.overrideWithValue(source),
        textClipboardProvider.overrideWithValue(clipboard),
      ],
      child: MaterialApp(
        theme: buildAppTheme(),
        home: Builder(
          builder: (context) => Scaffold(
            body: TextButton(
              onPressed: () => TokensDialog.show(context),
              child: const Text('öffnen'),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('öffnen'));
  await tester.pumpAndSettle();
  return (source, clipboard);
}

Future<void> _make(WidgetTester tester, String name) async {
  await tester.enterText(find.widgetWithText(TextField, 'Name'), name);
  await tester.tap(find.widgetWithText(FilledButton, 'Anlegen'));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('lists the tokens with reach and use but never the text', (
    tester,
  ) async {
    await _open(tester);

    expect(find.text('API-Tokens'), findsOneWidget);
    expect(find.text('Claude'), findsOneWidget);
    expect(find.text('Lesen und Bookmarks bearbeiten'), findsWidgets);
    expect(find.text('Nur lesen'), findsWidgets);
    expect(find.textContaining('Zuletzt benutzt: nie'), findsOneWidget);
    expect(find.textContaining('vxt_'), findsNothing);
  });

  testWidgets('says so when there are no tokens', (tester) async {
    await _open(tester, tokens: const []);

    expect(find.text('Noch keine Tokens.'), findsOneWidget);
  });

  testWidgets('a new token shows its text once, with a copy action', (
    tester,
  ) async {
    final (source, clipboard) = await _open(tester, tokens: const []);

    await _make(tester, 'Mein LLM');

    expect(source.calls, contains('create Mein LLM editBookmarks'));
    expect(find.text('Dein neues Token'), findsOneWidget);
    expect(find.text('vxt_secret100'), findsOneWidget);
    expect(find.textContaining('nicht noch einmal angezeigt'), findsOneWidget);
    // The list shows the token without its text.
    expect(find.text('Mein LLM'), findsOneWidget);

    await tester.tap(find.text('Kopieren'));
    await tester.pumpAndSettle();
    expect(clipboard.copied, ['vxt_secret100']);
    expect(find.text('Kopiert'), findsOneWidget);

    await tester.tap(find.text('Ausblenden'));
    await tester.pumpAndSettle();
    expect(find.textContaining('vxt_'), findsNothing);
    expect(find.text('Mein LLM'), findsOneWidget);
  });

  testWidgets('the reach can be read only', (tester) async {
    final (source, _) = await _open(tester, tokens: const []);

    await tester.tap(find.byType(DropdownButtonFormField<TokenScope>));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Nur lesen').last);
    await tester.pumpAndSettle();
    await _make(tester, 'Lesender');

    expect(source.calls, contains('create Lesender readOnly'));
  });

  testWidgets('a token needs a name', (tester) async {
    final (source, _) = await _open(tester, tokens: const []);

    await _make(tester, '  ');

    expect(find.text('Bitte einen Namen eingeben'), findsOneWidget);
    expect(source.calls.where((call) => call.startsWith('create')), isEmpty);
  });

  testWidgets('a refusal shows where the person acted and no text', (
    tester,
  ) async {
    final (source, _) = await _open(tester, tokens: const []);
    source.failure = const Refused(ServerReason('empty_name'));

    await _make(tester, 'x');

    expect(find.text('Der Name darf nicht leer sein.'), findsOneWidget);
    expect(find.text('Dein neues Token'), findsNothing);
  });

  testWidgets('revoking asks first, naming the token', (tester) async {
    final (source, _) = await _open(tester);

    await tester.tap(find.byTooltip('Widerrufen: Claude'));
    await tester.pumpAndSettle();
    expect(find.text('Token „Claude“ widerrufen?'), findsOneWidget);
    await tester.tap(find.text('Abbrechen'));
    await tester.pumpAndSettle();
    expect(source.calls.where((call) => call.startsWith('revoke')), isEmpty);

    await tester.tap(find.byTooltip('Widerrufen: Claude'));
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilledButton, 'Widerrufen'));
    await tester.pumpAndSettle();

    expect(source.calls, contains('revoke 1'));
    expect(find.text('Claude'), findsNothing);
    expect(find.text('Skript'), findsOneWidget);
  });

  testWidgets('a token that is already gone is reported', (tester) async {
    final (source, _) = await _open(tester);
    source.failure = const Refused(ServerReason('token_not_found'));

    await tester.tap(find.byTooltip('Widerrufen: Claude'));
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(FilledButton, 'Widerrufen'));
    await tester.pumpAndSettle();

    expect(find.text('Das Token gibt es nicht (mehr).'), findsOneWidget);
  });
}
