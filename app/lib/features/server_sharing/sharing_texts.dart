import '../../shared_ui/app_language.dart';
import 'sharing_models.dart';

/// User-visible texts of the server settings.
abstract final class SharingTexts {
  static String get open => tr('Einstellungen', 'Settings');
  static String get title => tr('Server', 'Server');
  static String get close => tr('Schließen', 'Close');
  static String get loading =>
      tr('Einstellungen werden geladen …', 'Loading settings …');
  static String get failure => tr(
    'Die Einstellungen konnten nicht geladen werden.',
    'The settings could not be loaded.',
  );
  static String get enable => tr('Server im Netzwerk', 'Server on the network');
  static String get enableHint => tr(
    'Andere Geräte öffnen Veltrunox im Browser. Die Wiedergabe läuft an '
        'diesem Mac.',
    'Other devices open Veltrunox in a browser. The audio plays on this Mac.',
  );
  static String get port => tr('Port', 'Port');
  static String get portInvalid =>
      tr('Eine Zahl von 1024 bis 65535', 'A number from 1024 to 65535');
  static String get https => tr('Verschlüsselt (HTTPS)', 'Encrypted (HTTPS)');
  static String get httpsHint => tr(
    'Mit eigener Zertifizierungsstelle; jedes Gerät installiert einmal '
        '/ca.crt. Ohne HTTPS nur in einem vertrauten Netz verwenden.',
    'With a certificate authority of its own; each device installs /ca.crt '
        'once. Without HTTPS use it in a trusted network only.',
  );
  static String get apply => tr('Übernehmen', 'Apply');
  static String get addresses => tr('Erreichbar unter', 'Reachable at');
  static String get noWebApp => tr(
    'Diese App enthält keine Web-Oberfläche; Browser sehen nichts.',
    'This app holds no web interface; browsers get nothing.',
  );
  static String get manageUsers => tr('Benutzer anlegen', 'Add users');

  static String problem(SharingProblem problem) => switch (problem) {
    NoAccountYet() => tr(
      'Lege zuerst einen Benutzer an, damit sich Browser anmelden können.',
      'Add a user first, so that browsers can sign in.',
    ),
    PortInUse() => tr(
      'Der Port ist belegt. Wähle einen anderen.',
      'The port is in use. Choose another one.',
    ),
    SharingFailed(:final detail) => tr(
      'Der Server konnte nicht starten: $detail',
      'The server could not start: $detail',
    ),
  };
}
