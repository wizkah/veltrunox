import '../shared_ui/app_language.dart';

/// User-visible texts of the app shell.
abstract final class ShellTexts {
  static const appName = 'Veltrunox';
  static String get player => tr('Player', 'Player');
  static String get library => tr('Bibliothek', 'Library');
}
