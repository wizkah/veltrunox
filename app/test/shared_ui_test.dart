import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/design/theme/app_theme.dart';
import 'package:veltrunox_app/design/tokens/app_geometry.dart';
import 'package:veltrunox_app/shared_ui/action_button.dart';
import 'package:veltrunox_app/shared_ui/activity_status.dart';
import 'package:veltrunox_app/shared_ui/app_icon_button.dart';
import 'package:veltrunox_app/shared_ui/confirm_dialog.dart';
import 'package:veltrunox_app/shared_ui/toast_notice.dart';

Widget _host(Widget child) => MaterialApp(
  theme: buildAppTheme(),
  home: Scaffold(body: Center(child: child)),
);

void main() {
  group('ActionButton', () {
    testWidgets('triggers when pressed', (tester) async {
      var pressed = 0;
      await tester.pumpWidget(
        _host(ActionButton(label: 'Speichern', onPressed: () => pressed++)),
      );
      await tester.tap(find.text('Speichern'));
      expect(pressed, 1);
    });

    testWidgets('cannot be pressed while loading or without a handler', (
      tester,
    ) async {
      var pressed = 0;
      await tester.pumpWidget(
        _host(
          ActionButton(
            label: 'Speichern',
            loading: true,
            onPressed: () => pressed++,
          ),
        ),
      );
      await tester.tap(find.text('Speichern'));
      expect(pressed, 0);
      expect(find.byType(CircularProgressIndicator), findsOneWidget);

      await tester.pumpWidget(
        _host(const ActionButton(label: 'Speichern', onPressed: null)),
      );
      expect(
        tester.widget<FilledButton>(find.byType(FilledButton)).onPressed,
        isNull,
      );
    });

    testWidgets('is at least a touch target large in every kind', (
      tester,
    ) async {
      for (final kind in ActionKind.values) {
        await tester.pumpWidget(
          _host(ActionButton(label: 'X', kind: kind, onPressed: () {})),
        );
        final size = tester.getSize(find.byType(ActionButton));
        expect(size.height, greaterThanOrEqualTo(AppSize.touchTarget));
        expect(size.width, greaterThanOrEqualTo(AppSize.touchTarget));
      }
    });
  });

  group('AppIconButton', () {
    testWidgets('has an accessible name and a touch-sized hit area', (
      tester,
    ) async {
      await tester.pumpWidget(
        _host(
          AppIconButton(tooltip: 'Zurück', icon: Icons.add, onPressed: () {}),
        ),
      );
      expect(find.byTooltip('Zurück'), findsOneWidget);
      final size = tester.getSize(find.byType(IconButton));
      expect(size.width, greaterThanOrEqualTo(AppSize.touchTarget));
      expect(size.height, greaterThanOrEqualTo(AppSize.touchTarget));
    });

    testWidgets('the prominent button is the large primary control', (
      tester,
    ) async {
      await tester.pumpWidget(
        _host(
          AppIconButton(
            tooltip: 'Abspielen',
            icon: Icons.play_arrow,
            prominent: true,
            onPressed: () {},
          ),
        ),
      );
      final size = tester.getSize(find.byType(IconButton));
      expect(size, const Size(AppSize.primaryPlay, AppSize.primaryPlay));
    });

    testWidgets('is disabled without a handler', (tester) async {
      await tester.pumpWidget(
        _host(
          const AppIconButton(
            tooltip: 'Zurück',
            icon: Icons.add,
            onPressed: null,
          ),
        ),
      );
      expect(
        tester.widget<IconButton>(find.byType(IconButton)).onPressed,
        isNull,
      );
    });
  });

  group('ToastNotice', () {
    testWidgets('shows message and kind icon without blocking', (tester) async {
      await tester.pumpWidget(
        _host(
          Builder(
            builder: (context) => TextButton(
              onPressed: () => ToastNotice.show(
                context,
                kind: ToastKind.error,
                message: 'Nicht gespeichert',
              ),
              child: const Text('Los'),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Los'));
      await tester.pump();

      expect(find.text('Nicht gespeichert'), findsOneWidget);
      expect(find.byIcon(Icons.error_outline), findsOneWidget);
    });

    testWidgets('each kind has its own icon', (tester) async {
      final icons = <IconData>{};
      for (final kind in ToastKind.values) {
        await tester.pumpWidget(_host(ToastNotice(kind: kind, message: 'm')));
        icons.add(tester.widget<Icon>(find.byType(Icon)).icon!);
      }
      expect(icons.length, ToastKind.values.length);
    });
  });

  group('ConfirmDialog', () {
    Future<bool?> ask(WidgetTester tester, String answer) async {
      bool? result;
      await tester.pumpWidget(
        _host(
          Builder(
            builder: (context) => TextButton(
              onPressed: () async => result = await ConfirmDialog.ask(
                context,
                title: 'Löschen?',
                message: 'Das kann nicht rückgängig gemacht werden.',
                confirmLabel: 'Löschen',
                cancelLabel: 'Abbrechen',
              ),
              child: const Text('Öffnen'),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Öffnen'));
      await tester.pumpAndSettle();
      expect(find.text('Löschen?'), findsOneWidget);
      await tester.tap(find.text(answer));
      await tester.pumpAndSettle();
      return result;
    }

    testWidgets('confirms only on the confirm button', (tester) async {
      expect(await ask(tester, 'Löschen'), isTrue);
    });

    testWidgets('cancelling answers no', (tester) async {
      expect(await ask(tester, 'Abbrechen'), isFalse);
    });
  });

  group('ActivityStatus', () {
    testWidgets('progress without a value is indeterminate', (tester) async {
      await tester.pumpWidget(
        _host(
          const ActivityStatus(state: ActivityState.progress, label: 'Lädt'),
        ),
      );
      final indicator = tester.widget<CircularProgressIndicator>(
        find.byType(CircularProgressIndicator),
      );
      expect(indicator.value, isNull);
      expect(find.text('Lädt'), findsOneWidget);
    });

    testWidgets('progress with a value shows it', (tester) async {
      await tester.pumpWidget(
        _host(
          const ActivityStatus(
            state: ActivityState.progress,
            label: 'Lädt',
            progress: 0.4,
          ),
        ),
      );
      final indicator = tester.widget<CircularProgressIndicator>(
        find.byType(CircularProgressIndicator),
      );
      expect(indicator.value, 0.4);
    });

    testWidgets('success and error show icons, not colour alone', (
      tester,
    ) async {
      await tester.pumpWidget(
        _host(const ActivityStatus(state: ActivityState.success, label: 'OK')),
      );
      expect(find.byIcon(Icons.check_circle_outline), findsOneWidget);
      await tester.pumpWidget(
        _host(
          const ActivityStatus(state: ActivityState.error, label: 'Fehler'),
        ),
      );
      expect(find.byIcon(Icons.error_outline), findsOneWidget);
    });
  });
}
