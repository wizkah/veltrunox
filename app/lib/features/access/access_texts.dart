import 'access_models.dart';

/// User-visible texts of the sign-in (German for now).
abstract final class AccessTexts {
  static const checking = 'Verbindung zum Server wird geprüft …';
  static const unreachable = 'Der Server ist nicht erreichbar.';
  static const retry = 'Erneut versuchen';
  static const signInTitle = 'Anmelden';
  static const userName = 'Benutzername';
  static const password = 'Passwort';
  static const passwordAgain = 'Passwort wiederholen';
  static const showPassword = 'Passwort anzeigen';
  static const hidePassword = 'Passwort verbergen';
  static const signIn = 'Anmelden';
  static const guest = 'Als Gast fortfahren';
  static const setupTitle = 'Ersten Admin einrichten';
  static const setupHint =
      'Dieser Server ist noch nicht eingerichtet. Lege das Konto an, das '
      'später Benutzer und Aufnahmen verwaltet.';
  static const setup = 'Einrichten';
  static const required = 'Bitte ausfüllen.';
  static const tooShort = 'Mindestens 10 Zeichen.';
  static const mismatch = 'Die Passwörter stimmen nicht überein.';
  static const accountMenu = 'Konto';
  static const signOut = 'Abmelden';
  static const users = 'Benutzer verwalten';
  static const guestName = 'Gast';

  static String signedInAs(String name) => 'Angemeldet als $name';

  /// A short, understandable explanation of a failure.
  static String failure(AccessFailure failure) => switch (failure) {
    WrongCredentials() => 'Benutzername oder Passwort stimmt nicht.',
    NotAllowed() => 'Dafür fehlt die Berechtigung.',
    TooManyAttempts(:final retryAfterSeconds) =>
      'Zu viele Fehlversuche. Bitte in $retryAfterSeconds s erneut versuchen.',
    Refused(:final reason) => reason,
    Unavailable() => 'Der Server ist nicht erreichbar.',
  };
}
