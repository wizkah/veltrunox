import '../../shared_ui/app_language.dart';

/// User-visible texts of changing one's own password.
abstract final class OwnPasswordTexts {
  static String get title => tr('Passwort ändern', 'Change password');
  static String get currentPassword =>
      tr('Aktuelles Passwort', 'Current password');
  static String get newPassword => tr('Neues Passwort', 'New password');
  static String get repeatPassword =>
      tr('Neues Passwort wiederholen', 'Repeat new password');
  static String get signOutNotice => tr(
    'Nach dem Ändern wirst du überall abgemeldet, auch auf diesem Gerät, '
        'und meldest dich mit dem neuen Passwort wieder an.',
    'After the change you are signed out everywhere, on this device too, '
        'and sign in again with the new password.',
  );
  static String get save => tr('Passwort ändern', 'Change password');
  static String get cancel => tr('Abbrechen', 'Cancel');
  static String get required => tr('Bitte ausfüllen.', 'Please fill in.');
  static String get mismatch => tr(
    'Die beiden neuen Passwörter sind verschieden.',
    'The two new passwords differ.',
  );
  static String get loading => tr('Wird geladen …', 'Loading …');
}
