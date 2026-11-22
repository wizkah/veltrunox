import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/app_shell/app_shell.dart';
import 'package:veltrunox_app/app_shell/shell_destination.dart';
import 'package:veltrunox_app/app_shell/shell_layout.dart';
import 'package:veltrunox_app/design/theme/app_theme.dart';
import 'package:veltrunox_app/design/tokens/app_colors.dart';
import 'package:veltrunox_app/design/tokens/app_geometry.dart';

Future<ValueNotifier<ShellDestination>> _pump(
  WidgetTester tester,
  double width,
) async {
  tester.view.physicalSize = Size(width, 800);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final selected = ValueNotifier(ShellDestination.library);
  await tester.pumpWidget(
    MaterialApp(
      theme: buildAppTheme(),
      home: ValueListenableBuilder(
        valueListenable: selected,
        builder: (_, destination, _) => AppShell(
          destination: destination,
          onSelect: (next) => selected.value = next,
          status: const Text('Status'),
          body: Text('Seite ${destination.label}'),
          persistentBar: const Text('Leiste'),
        ),
      ),
    ),
  );
  return selected;
}

Color? _iconColor(WidgetTester tester, IconData icon) =>
    tester.widget<Icon>(find.byIcon(icon)).color ??
    IconTheme.of(tester.element(find.byIcon(icon))).color;

void main() {
  test('the layout follows the breakpoints', () {
    expect(ShellLayout.forWidth(599), ShellLayout.mobile);
    expect(ShellLayout.forWidth(AppBreakpoint.tabletMin), ShellLayout.tablet);
    expect(ShellLayout.forWidth(1023), ShellLayout.tablet);
    expect(ShellLayout.forWidth(AppBreakpoint.desktopMin), ShellLayout.desktop);
  });

  testWidgets('mobile navigates along the bottom edge', (tester) async {
    final selected = await _pump(tester, 400);

    expect(find.byType(NavigationBar), findsOneWidget);
    expect(find.text('Leiste'), findsOneWidget);
    expect(find.text('Status'), findsOneWidget);
    await tester.tap(find.text('Player'));
    await tester.pump();
    expect(selected.value, ShellDestination.player);
    expect(find.text('Seite Player'), findsOneWidget);
  });

  testWidgets('tablet navigates along the side', (tester) async {
    await _pump(tester, 800);

    expect(find.byType(NavigationRail), findsOneWidget);
    expect(find.byType(NavigationBar), findsNothing);
    expect(find.text('Leiste'), findsOneWidget);
  });

  testWidgets('desktop navigates along the top and keeps the status', (
    tester,
  ) async {
    final selected = await _pump(tester, 1200);

    expect(find.byType(NavigationRail), findsNothing);
    expect(find.byType(NavigationBar), findsNothing);
    expect(find.text('Status'), findsOneWidget);
    await tester.tap(find.text('Player'));
    await tester.pump();
    expect(selected.value, ShellDestination.player);
  });

  testWidgets('navigation targets are touch sized', (tester) async {
    await _pump(tester, 400);
    final size = tester.getSize(find.byType(NavigationDestination).first);
    expect(size.height, greaterThanOrEqualTo(AppSize.touchTarget));
    expect(size.width, greaterThanOrEqualTo(AppSize.touchTarget));
  });

  test('the library is where the app starts', () {
    final container = ProviderContainer();
    addTearDown(container.dispose);
    expect(container.read(shellDestinationProvider), ShellDestination.library);
    container
        .read(shellDestinationProvider.notifier)
        .select(ShellDestination.player);
    expect(container.read(shellDestinationProvider), ShellDestination.player);
  });

  for (final (width, name) in [
    (400.0, 'mobile'),
    (800.0, 'tablet'),
    (1200.0, 'desktop'),
  ]) {
    testWidgets('the selected entry is orange and the other plain on $name', (
      tester,
    ) async {
      await _pump(tester, width);

      expect(
        _iconColor(tester, ShellDestination.library.selectedIcon),
        AppColors.action,
      );
      expect(
        _iconColor(tester, ShellDestination.player.icon),
        AppColors.textPrimary,
      );
    });
  }

  testWidgets('shows the logo', (tester) async {
    await _pump(tester, 400);
    expect(find.bySemanticsLabel('Veltrunox'), findsOneWidget);
    expect(find.byType(Image), findsNWidgets(2));
  });
}
