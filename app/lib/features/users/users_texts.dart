/// User-visible texts of the user administration (German for now).
abstract final class UsersTexts {
  static const title = 'Benutzer';
  static const close = 'Schließen';
  static const loading = 'Benutzer werden geladen …';
  static const failure = 'Die Benutzer konnten nicht geladen werden.';
  static const retry = 'Erneut versuchen';
  static const guestAccess = 'Gastzugang';
  static const guestAccessHint =
      'Gäste dürfen ohne Anmeldung abspielen, Markierungen bearbeiten und '
      'hochladen.';
  static const newUser = 'Neuer Benutzer';
  static const userName = 'Benutzername';
  static const password = 'Passwort';
  static const role = 'Rolle';
  static const roleAdmin = 'Admin';
  static const roleUser = 'Benutzer';
  static const create = 'Anlegen';
  static const active = 'Aktiv';
  static const inactive = 'Deaktiviert';
  static const setPassword = 'Passwort ändern';
  static const newPassword = 'Neues Passwort';
  static const save = 'Speichern';
  static const cancel = 'Abbrechen';
  static const delete = 'Löschen';
  static const required = 'Bitte ausfüllen.';
  static const tooShort = 'Mindestens 10 Zeichen.';
  static const passwordChanged = 'Passwort geändert.';

  static String deleteTitle(String name) => 'Benutzer „$name“ löschen?';
  static const deleteMessage =
      'Das Konto und seine Anmeldungen werden entfernt. Das lässt sich nicht '
      'rückgängig machen.';
}
