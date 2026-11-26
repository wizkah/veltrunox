import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/features/server_sharing/sharing_button.dart';
import 'package:veltrunox_app/features/server_sharing/sharing_controller.dart';
import 'package:veltrunox_app/features/server_sharing/sharing_dialog.dart';
import 'package:veltrunox_app/features/server_sharing/sharing_models.dart';
import 'package:veltrunox_app/features/server_sharing/sharing_source.dart';
import 'package:veltrunox_app/features/server_sharing/sharing_texts.dart';

class FakeSharingSource implements SharingSource {
  SharingStatus current = const SharingStatus(
    settings: SharingSettings(enabled: false, port: 8080, https: false),
  );
  SharingProblem? refusal;
  final applied = <SharingSettings>[];

  @override
  Future<SharingStatus> status() async => current;

  @override
  Future<SharingStatus> apply(SharingSettings settings) async {
    applied.add(settings);
    final problem = refusal;
    return current = SharingStatus(
      settings: SharingSettings(
        enabled: settings.enabled && problem == null,
        port: settings.port,
        https: settings.https,
      ),
      addresses: settings.enabled && problem == null
          ? ['http://mac.local:${settings.port}']
          : const [],
      problem: problem,
    );
  }
}

Future<void> pump(
  WidgetTester tester,
  Widget child, {
  SharingSource? source,
}) async {
  await tester.pumpWidget(
    ProviderScope(
      overrides: [sharingSourceProvider.overrideWithValue(source)],
      child: MaterialApp(home: Scaffold(body: child)),
    ),
  );
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('the settings are offered only with a server', (tester) async {
    await pump(tester, SharingButton(onPressed: () {}));
    expect(find.byIcon(Icons.settings_outlined), findsNothing);

    await tester.pumpWidget(const SizedBox());
    await pump(
      tester,
      SharingButton(onPressed: () {}),
      source: FakeSharingSource(),
    );
    expect(find.byIcon(Icons.settings_outlined), findsOneWidget);
  });

  testWidgets('switching on uses the entered port and shows the address', (
    tester,
  ) async {
    final source = FakeSharingSource();
    await pump(tester, SharingDialog(onManageUsers: () {}), source: source);

    await tester.enterText(find.byType(TextField), '9000');
    await tester.pump();
    await tester.tap(find.text(SharingTexts.enable));
    await tester.pumpAndSettle();

    expect(source.applied.single.enabled, isTrue);
    expect(source.applied.single.port, 9000);
    expect(find.text('http://mac.local:9000'), findsOneWidget);
  });

  testWidgets('a port out of range cannot be applied', (tester) async {
    final source = FakeSharingSource();
    await pump(tester, SharingDialog(onManageUsers: () {}), source: source);

    await tester.enterText(find.byType(TextField), '80');
    await tester.pump();
    await tester.tap(find.text(SharingTexts.enable));
    await tester.pumpAndSettle();

    expect(find.text(SharingTexts.portInvalid), findsOneWidget);
    expect(source.applied, isEmpty);
  });

  testWidgets('without a user account the dialog leads to the users', (
    tester,
  ) async {
    var asked = false;
    final source = FakeSharingSource()..refusal = const NoAccountYet();
    await pump(
      tester,
      SharingDialog(onManageUsers: () => asked = true),
      source: source,
    );

    await tester.tap(find.text(SharingTexts.enable));
    await tester.pumpAndSettle();
    await tester.tap(find.text(SharingTexts.manageUsers));

    expect(asked, isTrue);
  });
}
