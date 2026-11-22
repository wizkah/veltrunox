import '../design/tokens/app_geometry.dart';

/// How the shell arranges itself, by the width it gets.
enum ShellLayout {
  mobile,
  tablet,
  desktop;

  static ShellLayout forWidth(double width) {
    if (width >= AppBreakpoint.desktopMin) return desktop;
    if (width >= AppBreakpoint.tabletMin) return tablet;
    return mobile;
  }
}
