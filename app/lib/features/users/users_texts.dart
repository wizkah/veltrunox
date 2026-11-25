import '../../shared_ui/app_language.dart';

/// User-visible texts of the user administration.
abstract final class UsersTexts {
  static String get title => tr('Benutzer', 'Users');
  static String get close => tr('Schließen', 'Close');
  static String get loading =>
      tr('Benutzer werden geladen …', 'Loading users …');
  static String get failure => tr(
    'Die Benutzer konnten nicht geladen werden.',
    'The users could not be loaded.',
  );
  static String get retry => tr('Erneut versuchen', 'Try again');
  static String get guestAccess => tr('Gastzugang', 'Guest access');
  static String get guestAccessHint => tr(
    'Gäste dürfen ohne Anmeldung abspielen, Markierungen bearbeiten und '
        'hochladen.',
    'Guests may play, edit markers and upload without signing in.',
  );
  static String get strictPasswords =>
      tr('Strenge Passwortregeln', 'Strict password rules');
  static String get strictPasswordsHint => tr(
    'Neue Passwörter brauchen mindestens 10 Zeichen mit Klein- und '
        'Großbuchstabe, Ziffer und Sonderzeichen. Sonst genügen 6 Zeichen. '
        'Bestehende Passwörter bleiben gültig.',
    'New passwords need at least 10 characters with a lower and an upper '
        'case letter, a digit and a special character. Otherwise 6 '
        'characters are enough. Existing passwords stay valid.',
  );
  static String get newUser => tr('Neuer Benutzer', 'New user');
  static String get userName => tr('Benutzername', 'User name');
  static String get password => tr('Passwort', 'Password');
  static String get role => tr('Rolle', 'Role');
  static String get roleAdmin => tr('Admin', 'Admin');
  static String get roleUser => tr('Benutzer', 'User');
  static String get create => tr('Anlegen', 'Create');
  static String get active => tr('Aktiv', 'Active');
  static String get inactive => tr('Deaktiviert', 'Deactivated');
  static String get setPassword => tr('Passwort ändern', 'Change password');
  static String get newPassword => tr('Neues Passwort', 'New password');
  static String get repeatPassword =>
      tr('Passwort wiederholen', 'Repeat password');
  static String get mismatch => tr(
    'Die beiden Passwörter sind verschieden.',
    'The two passwords differ.',
  );
  static String get save => tr('Speichern', 'Save');
  static String get cancel => tr('Abbrechen', 'Cancel');
  static String get delete => tr('Löschen', 'Delete');
  static String get required => tr('Bitte ausfüllen.', 'Please fill in.');
  static String get passwordChanged => tr(
    'Passwort geändert. Der Benutzer ist überall abgemeldet.',
    'Password changed. The user is signed out everywhere.',
  );

  static String deleteTitle(String name) =>
      tr('Benutzer „$name“ löschen?', 'Delete user “$name”?');
  static String get deleteMessage => tr(
    'Das Konto und seine Anmeldungen werden entfernt. Das lässt sich nicht '
        'rückgängig machen.',
    'The account and its sign-ins are removed. This cannot be undone.',
  );
}
