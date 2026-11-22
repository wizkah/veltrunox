import 'app_language.dart';

/// Turns the English reasons the server gives for a refusal into the language
/// of the interface. In English the server's own wording stands (with a full
/// stop); a reason without a translation comes back as it is, so nothing is
/// lost.
String serverReasonText(String reason) {
  final text = reason.trim();
  final german = AppLanguage.current == AppLanguage.de;
  if (!german) return _sentence(text);
  final known = _exact[text];
  if (known != null) return known;
  for (final rule in _patterns) {
    final match = rule.pattern.firstMatch(text);
    if (match != null) return rule.build(match);
  }
  return text;
}

/// The reason as a sentence: capital first letter, full stop at the end.
String _sentence(String text) {
  if (text.isEmpty) return text;
  final capital = text[0].toUpperCase() + text.substring(1);
  return RegExp(r'[.!?]$').hasMatch(capital) ? capital : '$capital.';
}

const _exact = <String, String>{
  'user name must not be empty': 'Der Benutzername darf nicht leer sein.',
  'user name must not contain control characters':
      'Der Benutzername enthält unzulässige Zeichen.',
  'password must contain a lowercase letter':
      'Das Passwort braucht einen Kleinbuchstaben.',
  'password must contain an uppercase letter':
      'Das Passwort braucht einen Großbuchstaben.',
  'password must contain a digit': 'Das Passwort braucht eine Ziffer.',
  'password must contain a special character':
      'Das Passwort braucht ein Sonderzeichen.',
  'a user with this name exists already':
      'Ein Benutzer mit diesem Namen gibt es schon.',
  'user not found': 'Der Benutzer wurde nicht gefunden.',
  'the first admin exists already': 'Der erste Admin ist schon eingerichtet.',
  'wrong user name or password': 'Benutzername oder Passwort stimmt nicht.',
  'at least one active admin must remain':
      'Mindestens ein aktiver Admin muss bleiben.',
  'guests are not a role of an account': 'Gast ist keine Rolle für ein Konto.',
  'sign-in required': 'Bitte melde dich an.',
  'not allowed': 'Dafür fehlt die Berechtigung.',
  'guest access is off': 'Der Gastzugang ist ausgeschaltet.',
  'not found': 'Nicht gefunden.',
  'recording does not exist': 'Die Aufnahme gibt es nicht.',
  'recording title must not be empty': 'Der Titel darf nicht leer sein.',
  'a name must not be empty': 'Der Name darf nicht leer sein.',
  'bookmark or recording not found':
      'Die Markierung oder Aufnahme wurde nicht gefunden.',
  'no recording is selected': 'Es ist keine Aufnahme gewählt.',
  'speed must be between 0.5 and 2.0':
      'Die Geschwindigkeit muss zwischen 0,5 und 2,0 liegen.',
  'tuning must be between 300 and 500 Hz':
      'Die Stimmung muss zwischen 300 und 500 Hz liegen.',
  'key shift must be between -12 and 12 semitones':
      'Die Tonart kann um höchstens 12 Halbtöne verschoben werden.',
  'volume must be between 0 and 100 percent':
      'Die Lautstärke muss zwischen 0 und 100 Prozent liegen.',
  'position is out of range': 'Die Position liegt außerhalb der Aufnahme.',
  'a time range must end after it starts':
      'Das Ende muss nach dem Start liegen.',
  'tempo must be a positive number': 'Das Tempo muss größer als 0 sein.',
  "a key's tonic must be a pitch class from 0 to 11":
      'Der Grundton der Tonart ist ungültig.',
  'the upload was interrupted': 'Der Upload wurde unterbrochen.',
  'the file is empty': 'Die Datei ist leer.',
  'the file name is missing': 'Der Dateiname fehlt.',
  'the file name is not valid': 'Der Dateiname ist ungültig.',
  'the file name is empty': 'Der Dateiname ist leer.',
  'uploading is not set up on this server':
      'Der Upload ist auf diesem Server nicht eingerichtet.',
  'the recording is too short to analyse':
      'Die Aufnahme ist zu kurz für die Analyse.',
  'the recording has no tonal content to find a key in': 'Die Aufnahme hat keinen Klanginhalt, in dem sich eine Tonart finden ließe.',
  'the recording has no rhythmic structure to find a tempo in':
      'Die Aufnahme hat keinen Rhythmus, in dem sich ein Tempo finden ließe.',
  'the recording has no audio file': 'Zur Aufnahme gibt es keine Audiodatei.',
  'unrecognised message': 'Die Nachricht wurde nicht verstanden.',
  'binary messages are not supported':
      'Binäre Nachrichten werden nicht unterstützt.',
};

typedef _Rule = ({RegExp pattern, String Function(RegExpMatch) build});

final _patterns = <_Rule>[
  (
    pattern: RegExp(r'^user name must have at most (\d+) characters$'),
    build: (m) => 'Der Benutzername darf höchstens ${m[1]} Zeichen haben.',
  ),
  (
    pattern: RegExp(r'^password must have at least (\d+) characters$'),
    build: (m) => 'Das Passwort braucht mindestens ${m[1]} Zeichen.',
  ),
  (
    pattern: RegExp(r'^too many failed logins; try again in (\d+) seconds$'),
    build: (m) => 'Zu viele Fehlversuche. Bitte in ${m[1]} s erneut versuchen.',
  ),
  (
    pattern: RegExp(r'^the file is larger than the limit of (\d+) MB$'),
    build: (m) => 'Die Datei ist größer als das Limit von ${m[1]} MB.',
  ),
  (
    pattern: RegExp(r'^cannot read the file'),
    build: (_) => 'Die Datei lässt sich nicht lesen.',
  ),
  (
    pattern: RegExp(r'^unsupported or damaged audio'),
    build: (_) =>
        'Das Audioformat wird nicht unterstützt oder die Datei ist beschädigt.',
  ),
  (
    pattern: RegExp(r'^this is not audio the server can read'),
    build: (_) => 'Das ist keine Audiodatei, die der Server lesen kann.',
  ),
  (
    pattern: RegExp(
      r'^bookmark was changed meanwhile; current version is (\d+)$',
    ),
    build: (_) => 'Die Markierung wurde inzwischen geändert.',
  ),
  (
    pattern: RegExp(
      r'^position (\d+) ms is beyond the end of the recording \((\d+) ms\)$',
    ),
    build: (_) => 'Die Position liegt hinter dem Ende der Aufnahme.',
  ),
];
