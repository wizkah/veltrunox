import 'package:material_ui/material_ui.dart';

import '../tokens/app_colors.dart';
import '../tokens/app_geometry.dart';
import '../tokens/app_typography.dart';

/// The dark theme of the app, built only from the design tokens.
ThemeData buildAppTheme() {
  const scheme = ColorScheme.dark(
    primary: AppColors.action,
    onPrimary: AppColors.onAction,
    secondary: AppColors.accent,
    onSecondary: AppColors.onAction,
    tertiary: AppColors.accentSoft,
    error: AppColors.error,
    onError: AppColors.onAction,
    surface: AppColors.surface,
    onSurface: AppColors.textPrimary,
    onSurfaceVariant: AppColors.textSecondary,
    surfaceContainer: AppColors.surface,
    surfaceContainerHigh: AppColors.surfaceRaised,
    surfaceContainerHighest: AppColors.surfaceRaised,
    outline: AppColors.border,
    outlineVariant: AppColors.border,
    scrim: AppColors.scrim,
  );
  final text = _textTheme();
  final shape = RoundedRectangleBorder(
    borderRadius: BorderRadius.circular(AppRadius.medium),
  );
  const touchSize = Size(AppSize.touchTarget, AppSize.touchTarget);

  return ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    colorScheme: scheme,
    scaffoldBackgroundColor: AppColors.canvas,
    canvasColor: AppColors.canvas,
    fontFamily: AppTypography.family,
    fontFamilyFallback: AppTypography.familyFallback,
    textTheme: text,
    focusColor: AppColors.focus.withValues(alpha: 0.24),
    hoverColor: AppColors.textPrimary.withValues(alpha: 0.08),
    dividerColor: AppColors.border,
    disabledColor: AppColors.textSecondary.withValues(alpha: 0.5),
    appBarTheme: AppBarTheme(
      backgroundColor: AppColors.canvas,
      foregroundColor: AppColors.textPrimary,
      elevation: 0,
      scrolledUnderElevation: 0,
      surfaceTintColor: Colors.transparent,
      shape: const Border(
        bottom: BorderSide(color: AppColors.border, width: AppSize.borderWidth),
      ),
      centerTitle: false,
      titleTextStyle: text.titleLarge,
    ),
    iconButtonTheme: IconButtonThemeData(
      style: ButtonStyle(
        minimumSize: const WidgetStatePropertyAll(touchSize),
        side: _focusRing(),
        // A disabled icon button stays recognisable (for example a reset
        // button of a setting that is already as analysed).
        foregroundColor: WidgetStateProperty.resolveWith(
          (states) => states.contains(WidgetState.disabled)
              ? AppColors.textSecondary.withValues(alpha: 0.65)
              : null,
        ),
      ),
    ),
    filledButtonTheme: FilledButtonThemeData(
      style: ButtonStyle(
        minimumSize: const WidgetStatePropertyAll(touchSize),
        shape: WidgetStatePropertyAll(shape),
        side: _focusRing(),
      ),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style: ButtonStyle(
        minimumSize: const WidgetStatePropertyAll(touchSize),
        shape: WidgetStatePropertyAll(shape),
        side: _focusRing(
          rest: const BorderSide(
            color: AppColors.border,
            width: AppSize.borderWidth,
          ),
        ),
      ),
    ),
    textButtonTheme: TextButtonThemeData(
      style: ButtonStyle(
        minimumSize: const WidgetStatePropertyAll(touchSize),
        shape: WidgetStatePropertyAll(shape),
        side: _focusRing(),
      ),
    ),
    listTileTheme: ListTileThemeData(
      minVerticalPadding: AppSpace.s2,
      selectedColor: AppColors.accentSoft,
      selectedTileColor: AppColors.surfaceRaised,
      shape: shape,
    ),
    sliderTheme: SliderThemeData(
      activeTrackColor: AppColors.accent,
      inactiveTrackColor: AppColors.border,
      thumbColor: AppColors.textPrimary,
      overlayColor: AppColors.textPrimary.withValues(alpha: 0.16),
      trackHeight: 6,
    ),
    inputDecorationTheme: InputDecorationThemeData(
      border: _inputBorder(AppColors.border, AppSize.borderWidth),
      enabledBorder: _inputBorder(AppColors.border, AppSize.borderWidth),
      focusedBorder: _inputBorder(AppColors.focus, AppSize.focusRingWidth),
      errorBorder: _inputBorder(AppColors.error, AppSize.borderWidth),
      focusedErrorBorder: _inputBorder(AppColors.focus, AppSize.focusRingWidth),
    ),
    snackBarTheme: SnackBarThemeData(
      behavior: SnackBarBehavior.floating,
      backgroundColor: AppColors.surfaceRaised,
      contentTextStyle: text.bodyMedium,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(AppRadius.medium),
        side: const BorderSide(color: AppColors.border),
      ),
    ),
    dialogTheme: DialogThemeData(
      backgroundColor: AppColors.surfaceRaised,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(AppRadius.large),
      ),
    ),
    navigationBarTheme: NavigationBarThemeData(
      backgroundColor: AppColors.canvas,
      surfaceTintColor: Colors.transparent,
      indicatorColor: AppColors.surfaceRaised,
      height: 72,
      iconTheme: WidgetStateProperty.resolveWith(
        (states) => IconThemeData(color: _navigationColor(states)),
      ),
      labelTextStyle: WidgetStateProperty.resolveWith(
        (states) => text.labelMedium!.copyWith(color: _navigationColor(states)),
      ),
    ),
    navigationRailTheme: NavigationRailThemeData(
      backgroundColor: AppColors.canvas,
      indicatorColor: AppColors.surfaceRaised,
      selectedIconTheme: const IconThemeData(color: AppColors.action),
      unselectedIconTheme: const IconThemeData(color: AppColors.textPrimary),
      selectedLabelTextStyle: text.labelMedium!.copyWith(
        color: AppColors.action,
      ),
      unselectedLabelTextStyle: text.labelMedium!.copyWith(
        color: AppColors.textPrimary,
      ),
    ),
  );
}

/// The selected navigation entry is orange, all others are plain text colour.
Color _navigationColor(Set<WidgetState> states) =>
    states.contains(WidgetState.selected)
    ? AppColors.action
    : AppColors.textPrimary;

/// The focus ring: a visible outline on keyboard focus.
WidgetStateProperty<BorderSide?> _focusRing({BorderSide? rest}) =>
    WidgetStateProperty.resolveWith(
      (states) => states.contains(WidgetState.focused)
          ? const BorderSide(
              color: AppColors.focus,
              width: AppSize.focusRingWidth,
            )
          : rest,
    );

OutlineInputBorder _inputBorder(Color color, double width) =>
    OutlineInputBorder(
      borderRadius: BorderRadius.circular(AppRadius.small),
      borderSide: BorderSide(color: color, width: width),
    );

TextTheme _textTheme() {
  TextStyle style(double size, double height, FontWeight weight) => TextStyle(
    fontSize: size,
    height: height,
    fontWeight: weight,
    color: AppColors.textPrimary,
  );
  final body = style(
    AppTypography.sizeBody,
    AppTypography.lineHeightBody,
    AppTypography.regular,
  );
  final small = style(
    AppTypography.sizeSmall,
    AppTypography.lineHeightSmall,
    AppTypography.regular,
  );
  final section = style(
    AppTypography.sizeSection,
    AppTypography.lineHeightSection,
    AppTypography.semibold,
  );
  final page = style(
    AppTypography.sizePage,
    AppTypography.lineHeightPage,
    AppTypography.semibold,
  );
  return TextTheme(
    headlineMedium: page,
    headlineSmall: page,
    titleLarge: section,
    titleMedium: section,
    titleSmall: body.copyWith(fontWeight: AppTypography.medium),
    bodyLarge: body,
    bodyMedium: body,
    bodySmall: small.copyWith(color: AppColors.textSecondary),
    labelLarge: body.copyWith(fontWeight: AppTypography.medium),
    labelMedium: small,
    labelSmall: small,
  );
}
