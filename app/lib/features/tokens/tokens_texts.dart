import '../../shared_ui/app_language.dart';
import 'token_models.dart';

/// User-visible texts of the API tokens.
abstract final class TokensTexts {
  static String get title => tr('API-Tokens', 'API tokens');
  static String get close => tr('Schließen', 'Close');
  static String get intro => tr(
    'Ein Token lässt ein Skript oder ein LLM Bookmarks lesen und, wenn '
        'erlaubt, bearbeiten, ohne dein Passwort.',
    'A token lets a script or an LLM read and, if allowed, edit bookmarks '
        'without your password.',
  );
  static String get loading =>
      tr('Tokens werden geladen …', 'Loading tokens …');
  static String get failure => tr(
    'Die Tokens konnten nicht geladen werden.',
    'The tokens could not be loaded.',
  );
  static String get retry => tr('Erneut versuchen', 'Try again');
  static String get none => tr('Noch keine Tokens.', 'No tokens yet.');

  static String scope(TokenScope scope) => switch (scope) {
    TokenScope.readOnly => tr('Nur lesen', 'Read only'),
    TokenScope.editBookmarks => tr(
      'Lesen und Bookmarks bearbeiten',
      'Read and edit bookmarks',
    ),
  };

  static String created(String when) => tr('Angelegt: $when', 'Created: $when');
  static String lastUsed(String? when) => when == null
      ? tr('Zuletzt benutzt: nie', 'Last used: never')
      : tr('Zuletzt benutzt: $when', 'Last used: $when');

  static String get newToken => tr('Neues Token', 'New token');
  static String get name => tr('Name', 'Name');
  static String get nameHint =>
      tr('Wofür ist das Token?', 'What is the token for?');
  static String get reach => tr('Reichweite', 'Reach');
  static String get reachHint => tr(
    'Ein Token kann nie mehr als du selbst und nie mehr als Bookmarks lesen '
        'und bearbeiten: kein Abspielen, kein Hochladen, keine Verwaltung.',
    'A token can never do more than you can, and never more than read and '
        'edit bookmarks: no playing, no uploading, no administration.',
  );
  static String get create => tr('Anlegen', 'Create');
  static String get required =>
      tr('Bitte einen Namen eingeben', 'Please enter a name');

  static String get secretTitle => tr('Dein neues Token', 'Your new token');
  static String get secretWarning => tr(
    'Kopiere es jetzt. Es wird nicht noch einmal angezeigt.',
    'Copy it now. It cannot be shown again.',
  );
  static String get copy => tr('Kopieren', 'Copy');
  static String get copied => tr('Kopiert', 'Copied');
  static String get hideSecret => tr('Ausblenden', 'Hide');

  static String get revoke => tr('Widerrufen', 'Revoke');
  static String revokeTitle(String name) =>
      tr('Token „$name“ widerrufen?', 'Revoke the token “$name”?');
  static String get revokeMessage => tr(
    'Wer das Token benutzt, wird abgewiesen. Das lässt sich nicht rückgängig '
        'machen.',
    'Whoever uses the token is turned away. This cannot be undone.',
  );
  static String get cancel => tr('Abbrechen', 'Cancel');

  /// A moment in local time, to the minute.
  static String moment(int unixSeconds) {
    final time = DateTime.fromMillisecondsSinceEpoch(unixSeconds * 1000)
        .toLocal();
    String two(int value) => value.toString().padLeft(2, '0');
    return '${time.year}-${two(time.month)}-${two(time.day)} '
        '${two(time.hour)}:${two(time.minute)}';
  }
}
