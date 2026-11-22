import '../../shared_ui/app_language.dart';

/// User-visible texts of the connection feature.
abstract final class ConnectionTexts {
  static String get connecting => tr('Verbinde …', 'Connecting …');
  static String get connected => tr('Verbunden', 'Connected');
  static String get disconnected => tr('Getrennt', 'Disconnected');
  static String get lost => tr(
    'Die Verbindung zum Server ist verloren.',
    'The connection to the server is lost.',
  );
  static String get lostHint => tr(
    'Es wird nichts verarbeitet, Eingaben werden zurückgesetzt.',
    'Nothing is processed; changes are reset.',
  );
  static String get reconnect => tr('Neu verbinden', 'Reconnect');
}
