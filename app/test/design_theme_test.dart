import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';
import 'package:veltrunox_app/design/theme/app_theme.dart';
import 'package:veltrunox_app/design/tokens/app_colors.dart';
import 'package:veltrunox_app/design/tokens/app_geometry.dart';

double _contrast(Color a, Color b) {
  final la = a.computeLuminance();
  final lb = b.computeLuminance();
  final (hi, lo) = la > lb ? (la, lb) : (lb, la);
  return (hi + 0.05) / (lo + 0.05);
}

void main() {
  final theme = buildAppTheme();

  test('is dark and takes its colours from the tokens', () {
    expect(theme.brightness, Brightness.dark);
    expect(theme.scaffoldBackgroundColor, AppColors.canvas);
    expect(theme.colorScheme.primary, AppColors.action);
  });

  test('text and focus meet the contrast of WCAG AA', () {
    expect(
      _contrast(AppColors.textPrimary, AppColors.canvas),
      greaterThan(4.5),
    );
    expect(
      _contrast(AppColors.textSecondary, AppColors.surface),
      greaterThan(4.5),
    );
    expect(_contrast(AppColors.onAction, AppColors.action), greaterThan(4.5));
    expect(_contrast(AppColors.focus, AppColors.canvas), greaterThan(3));
    expect(_contrast(AppColors.border, AppColors.canvas), greaterThan(1.5));
  });

  test('buttons are at least a touch target large', () {
    for (final style in [
      theme.iconButtonTheme.style,
      theme.filledButtonTheme.style,
      theme.outlinedButtonTheme.style,
      theme.textButtonTheme.style,
    ]) {
      final size = style!.minimumSize!.resolve({});
      expect(size, const Size(AppSize.touchTarget, AppSize.touchTarget));
    }
  });

  test('a focused button shows the focus ring', () {
    final side = theme.filledButtonTheme.style!.side!.resolve({
      WidgetState.focused,
    });
    expect(side!.color, AppColors.focus);
    expect(side.width, AppSize.focusRingWidth);
  });
}
