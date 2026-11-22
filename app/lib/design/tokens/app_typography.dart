import 'package:material_ui/material_ui.dart';

/// Typography tokens; the values come from `docs/design/design-tokens.json`.
abstract final class AppTypography {
  static const family = 'Inter';
  static const familyFallback = [
    'system-ui',
    '-apple-system',
    'BlinkMacSystemFont',
    'Segoe UI',
    'sans-serif',
  ];

  static const sizeSmall = 14.0;
  static const sizeBody = 16.0;
  static const sizeSection = 20.0;
  static const sizePage = 24.0;

  static const lineHeightSmall = 1.4286;
  static const lineHeightBody = 1.5;
  static const lineHeightSection = 1.4;
  static const lineHeightPage = 1.3333;

  static const regular = FontWeight.w400;
  static const medium = FontWeight.w500;
  static const semibold = FontWeight.w600;
  static const bold = FontWeight.w700;
}
