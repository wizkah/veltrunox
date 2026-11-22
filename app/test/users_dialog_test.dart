import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/design/theme/app_theme.dart';
import 'package:veltrunox_app/features/access/access_models.dart';
import 'package:veltrunox_app/features/users/user_models.dart';
import 'package:veltrunox_app/features/users/user_source.dart';
import 'package:veltrunox_app/features/users/users_controller.dart';
import 'package:veltrunox_app/features/users/users_dialog.dart';

class _FakeUsers implements UserSource {
  final users = [
    const ManagedUser(
      id: 1,
      userName: 'root',
      role: UserRole.admin,
      active: true,
    ),
    const ManagedUser(
      id: 2,
      userName: 'ann',
      role: UserRole.user,
      active: true,
    ),
  ];
  bool guest = false;
  var policy = PasswordPolicy.standard;
  AccessFailure? failure;
  final calls = <String>[];

  void _maybeFail() {
    final failure = this.failure;
    if (failure != null) throw AccessException(failure);
  }

  @override
  Future<UserAdministration> load() async => UserAdministration(
    users: List.of(users),
    guestEnabled: guest,
    passwordPolicy: policy,
  );

  @override
  Future<void> create({
    required String userName,
    required String password,
    required UserRole role,
  }) async {
    _maybeFail();
    calls.add('create:$userName:${role.name}');
    users.add(
      ManagedUser(
        id: users.length + 1,
        userName: userName,
        role: role,
        active: true,
      ),
    );
  }

  @override
  Future<void> change(
    int id, {
    UserRole? role,
    bool? active,
    String? password,
  }) async {
    _maybeFail();
    calls.add('change:$id:${role?.name}:$active:${password != null}');
    final index = users.indexWhere((user) => user.id == id);
    final user = users[index];
    users[index] = ManagedUser(
      id: id,
      userName: user.userName,
      role: role ?? user.role,
      active: active ?? user.active,
    );
  }

  @override
  Future<void> delete(int id) async {
    _maybeFail();
    calls.add('delete:$id');
    users.removeWhere((user) => user.id == id);
  }

  @override
  Future<void> setGuestAccess({required bool enabled}) async {
    _maybeFail();
    calls.add('guest:$enabled');
    guest = enabled;
  }

  @override
  Future<void> setPasswordPolicy(PasswordPolicy policy) async {
    _maybeFail();
    calls.add('policy:${policy.name}');
    this.policy = policy;
  }
}

Future<_FakeUsers> _open(WidgetTester tester) async {
  tester.view.physicalSize = const Size(900, 1200);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final source = _FakeUsers();
  await tester.pumpWidget(
    ProviderScope(
      overrides: [userSourceProvider.overrideWithValue(source)],
      child: MaterialApp(
        theme: buildAppTheme(),
        home: Builder(
          builder: (context) => Scaffold(
            body: TextButton(
              onPressed: () => UsersDialog.show(context),
              child: const Text('öffnen'),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('öffnen'));
  await tester.pumpAndSettle();
  return source;
}

/// The form sits below the list; scrolls the dialog until it is built.
Future<void> _toForm(WidgetTester tester) async {
  await tester.scrollUntilVisible(
    find.widgetWithText(FilledButton, 'Anlegen'),
    200,
    scrollable: find.byType(Scrollable).last,
  );
}

void main() {
  testWidgets('lists the users and the guest switch', (tester) async {
    await _open(tester);

    expect(find.text('root'), findsOneWidget);
    expect(find.text('ann'), findsOneWidget);
    expect(find.text('Gastzugang'), findsOneWidget);
  });

  testWidgets('switching guest access goes to the server', (tester) async {
    final source = await _open(tester);

    await tester.tap(find.byType(Switch).first);
    await tester.pumpAndSettle();

    expect(source.calls, ['guest:true']);
    expect(source.guest, isTrue);
  });

  testWidgets('deactivating a user goes to the server', (tester) async {
    final source = await _open(tester);

    await tester.tap(find.byType(Switch).at(3));
    await tester.pumpAndSettle();

    expect(source.calls, ['change:2:null:false:false']);
    expect(find.text('Deaktiviert'), findsOneWidget);
  });

  testWidgets('a new user is created and appears in the list', (tester) async {
    final source = await _open(tester);

    await _toForm(tester);
    await tester.enterText(
      find.widgetWithText(TextField, 'Benutzername'),
      'bo',
    );
    await tester.enterText(
      find.widgetWithText(TextField, 'Passwort'),
      'a long enough password',
    );
    await tester.tap(find.widgetWithText(FilledButton, 'Anlegen'));
    await tester.pumpAndSettle();

    expect(source.calls, ['create:bo:user']);
    expect(find.text('bo'), findsOneWidget);
  });

  testWidgets('a short password is refused at the field', (tester) async {
    final source = await _open(tester);

    await _toForm(tester);
    await tester.enterText(
      find.widgetWithText(TextField, 'Benutzername'),
      'bo',
    );
    await tester.enterText(find.widgetWithText(TextField, 'Passwort'), 'short');
    await tester.tap(find.widgetWithText(FilledButton, 'Anlegen'));
    await tester.pumpAndSettle();

    expect(find.text('Mindestens 6 Zeichen.'), findsOneWidget);
    expect(source.calls, isEmpty);
  });

  testWidgets(
    'the admin switches to strict rules, which the forms then ask for',
    (tester) async {
      final source = await _open(tester);

      await tester.tap(find.text('Strenge Passwortregeln'));
      await tester.pumpAndSettle();
      expect(source.calls, ['policy:strict']);

      await _toForm(tester);
      await tester.enterText(
        find.widgetWithText(TextField, 'Benutzername'),
        'bo',
      );
      await tester.enterText(
        find.widgetWithText(TextField, 'Passwort'),
        'longenough1!',
      );
      await tester.tap(find.widgetWithText(FilledButton, 'Anlegen'));
      await tester.pumpAndSettle();
      expect(find.text('Es fehlt ein Großbuchstabe.'), findsOneWidget);
      expect(source.calls, ['policy:strict']);

      await tester.enterText(
        find.widgetWithText(TextField, 'Passwort'),
        'Longenough1!',
      );
      await tester.tap(find.widgetWithText(FilledButton, 'Anlegen'));
      await tester.pumpAndSettle();
      expect(source.calls, ['policy:strict', 'create:bo:user']);
    },
  );

  testWidgets('a refusal by the server is shown where the person acted', (
    tester,
  ) async {
    final source = await _open(tester);
    source.failure = const Refused('a user with this name exists already');

    await _toForm(tester);
    await tester.enterText(
      find.widgetWithText(TextField, 'Benutzername'),
      'ann',
    );
    await tester.enterText(
      find.widgetWithText(TextField, 'Passwort'),
      'a long enough password',
    );
    await tester.tap(find.widgetWithText(FilledButton, 'Anlegen'));
    await tester.pumpAndSettle();

    expect(
      find.text('Ein Benutzer mit diesem Namen gibt es schon.'),
      findsOneWidget,
    );
  });

  testWidgets('deleting asks first and names the user', (tester) async {
    final source = await _open(tester);

    await tester.tap(find.byTooltip('Löschen').last);
    await tester.pumpAndSettle();
    expect(find.text('Benutzer „ann“ löschen?'), findsOneWidget);
    expect(source.calls, isEmpty);

    await tester.tap(find.widgetWithText(FilledButton, 'Löschen'));
    await tester.pumpAndSettle();

    expect(source.calls, ['delete:2']);
    expect(find.text('ann'), findsNothing);
  });

  testWidgets('a refused change says so in the dialog', (tester) async {
    final source = await _open(tester);
    source.failure = const Refused('at least one active admin must remain');

    await tester.tap(find.byType(Switch).at(1));
    await tester.pumpAndSettle();

    expect(
      find.text('Mindestens ein aktiver Admin muss bleiben.'),
      findsOneWidget,
    );
  });
}
