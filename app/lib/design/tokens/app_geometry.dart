/// Spacing, radius, size, motion and breakpoint tokens; the values come from
/// `docs/design/design-tokens.json`.
abstract final class AppSpace {
  static const s1 = 4.0;
  static const s2 = 8.0;
  static const s3 = 12.0;
  static const s4 = 16.0;
  static const s6 = 24.0;
  static const s8 = 32.0;
}

abstract final class AppRadius {
  static const small = 8.0;
  static const medium = 12.0;
  static const large = 16.0;
  static const pill = 999.0;
}

abstract final class AppSize {
  static const touchTarget = 48.0;
  static const primaryPlay = 64.0;
  static const regionLaneHeight = 36.0;
  static const regionHandleVisualWidth = 12.0;
  static const borderWidth = 1.0;
  static const focusRingWidth = 3.0;
}

abstract final class AppMotion {
  static const quick = Duration(milliseconds: 120);
  static const normal = Duration(milliseconds: 180);
}

abstract final class AppBreakpoint {
  static const tabletMin = 600.0;
  static const desktopMin = 1024.0;
}
