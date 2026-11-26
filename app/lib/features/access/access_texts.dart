import '../../shared_ui/app_language.dart';
import '../../shared_ui/server_reason.dart';
import 'access_models.dart';

/// User-visible texts of the sign-in.
abstract final class AccessTexts {
  static String get checking => tr(
    'Verbindung zum Server wird geprüft …',
    'Checking the connection to the server …',
  );
  static String get unreachable =>
      tr('Der Server ist nicht erreichbar.', 'The server cannot be reached.');
  static String get retry => tr('Erneut versuchen', 'Try again');
  static String get signInTitle => tr('Anmelden', 'Sign in');
  static String get userName => tr('Benutzername', 'User name');
  static String get password => tr('Passwort', 'Password');
  static String get passwordAgain =>
      tr('Passwort wiederholen', 'Repeat password');
  static String get showPassword => tr('Passwort anzeigen', 'Show password');
  static String get hidePassword => tr('Passwort verbergen', 'Hide password');
  static String get signIn => tr('Anmelden', 'Sign in');
  static String get guest => tr('Als Gast fortfahren', 'Continue as guest');
  static String get setupTitle =>
      tr('Ersten Admin einrichten', 'Set up the first admin');
  static String get setupHint => tr(
    'Dieser Server ist noch nicht eingerichtet. Lege das Konto an, das '
        'später Benutzer und Aufnahmen verwaltet.',
    'This server is not set up yet. Create the account that will manage '
        'users and recordings.',
  );
  static String get setup => tr('Einrichten', 'Set up');
  static String get required => tr('Bitte ausfüllen.', 'Please fill in.');
  static String get mismatch => tr(
    'Die Passwörter stimmen nicht überein.',
    'The passwords do not match.',
  );
  static String get accountMenu => tr('Konto', 'Account');
  static String get signOut => tr('Abmelden', 'Sign out');
  static String get users => tr('Benutzer verwalten', 'Manage users');
  static String get apiTokens => tr('API-Tokens', 'API tokens');
  static String get guestName => tr('Gast', 'Guest');

  /// What a password must be like, as a hint under the field.
  static String passwordRule(PasswordPolicy policy) => switch (policy) {
    PasswordPolicy.standard => tr(
      'Mindestens ${policy.minLength} Zeichen.',
      'At least ${policy.minLength} characters.',
    ),
    PasswordPolicy.strict => tr(
      'Mindestens ${policy.minLength} Zeichen mit Klein- und Großbuchstabe, '
          'Ziffer und Sonderzeichen.',
      'At least ${policy.minLength} characters with a lower and an upper case '
          'letter, a digit and a special character.',
    ),
  };

  /// What is missing from a password the person typed.
  static String passwordProblem(
    PasswordPolicy policy,
    PasswordProblem problem,
  ) => switch (problem) {
    PasswordProblem.tooShort => tr(
      'Mindestens ${policy.minLength} Zeichen.',
      'At least ${policy.minLength} characters.',
    ),
    PasswordProblem.noLowercase => tr(
      'Es fehlt ein Kleinbuchstabe.',
      'A lower case letter is missing.',
    ),
    PasswordProblem.noUppercase => tr(
      'Es fehlt ein Großbuchstabe.',
      'An upper case letter is missing.',
    ),
    PasswordProblem.noDigit => tr(
      'Es fehlt eine Ziffer.',
      'A digit is missing.',
    ),
    PasswordProblem.noSpecial => tr(
      'Es fehlt ein Sonderzeichen.',
      'A special character is missing.',
    ),
  };

  static String get changePassword => tr('Passwort ändern', 'Change password');
  static String get passwordChangedSignIn => tr(
    'Das Passwort wurde geändert. Bitte mit dem neuen Passwort anmelden.',
    'The password was changed. Please sign in with the new password.',
  );

  static String get localOwner =>
      tr('An diesem Mac (alle Rechte)', 'On this Mac (all rights)');
  static String signedInAs(String name) =>
      tr('Angemeldet als $name', 'Signed in as $name');

  /// A short, understandable explanation of a failure.
  static String failure(AccessFailure failure) => switch (failure) {
    WrongCredentials() => tr(
      'Benutzername oder Passwort stimmt nicht.',
      'Wrong user name or password.',
    ),
    NotAllowed() => tr(
      'Dafür fehlt die Berechtigung.',
      'You are not allowed to do this.',
    ),
    TooManyAttempts(:final retryAfterSeconds) => tr(
      'Zu viele Fehlversuche. Bitte in $retryAfterSeconds s erneut versuchen.',
      'Too many failed attempts. Please try again in $retryAfterSeconds s.',
    ),
    Refused(:final reason) => serverReasonText(reason),
    SessionExpired() => tr(
      'Die Anmeldung ist abgelaufen. Bitte erneut anmelden.',
      'The sign-in has expired. Please sign in again.',
    ),
    Unavailable() => tr(
      'Der Server ist nicht erreichbar.',
      'The server cannot be reached.',
    ),
  };
}
