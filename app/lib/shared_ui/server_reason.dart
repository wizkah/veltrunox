import 'app_language.dart';

/// Why the server refused something, as it reaches the interface: a stable
/// [code], the numbers its text needs ([args], in a fixed order per code) and
/// an English [message] for codes this version does not know.
class ServerReason {
  const ServerReason(this.code, {this.args = const [], this.message = ''});

  /// A text that is already worded, not a code of the server (for example a
  /// failure of this app itself).
  const ServerReason.plain(String text)
    : code = plainCode,
      args = const [],
      message = text;

  static const plainCode = 'plain';

  /// The server needs a valid sign-in (the code of a channel without one).
  static const signInRequiredCode = 'sign_in_required';

  final String code;
  final List<int> args;
  final String message;

  bool get isSignInRequired => code == signInRequiredCode;

  @override
  bool operator ==(Object other) =>
      other is ServerReason &&
      other.code == code &&
      other.message == message &&
      _sameArgs(other.args);

  bool _sameArgs(List<int> other) {
    if (other.length != args.length) return false;
    for (var i = 0; i < args.length; i++) {
      if (other[i] != args[i]) return false;
    }
    return true;
  }

  @override
  int get hashCode => Object.hash(code, message, Object.hashAll(args));

  @override
  String toString() => message.isEmpty ? code : message;
}

/// The words for a reason whose code this version knows; `null` otherwise.
String? knownReasonText(ServerReason reason) =>
    reason.code == ServerReason.plainCode ? reason.message : _text(reason);

/// The words for a reason in the language of the interface. A code this
/// version does not know shows the server's own English message, so nothing is
/// lost.
String serverReasonText(ServerReason reason) {
  if (reason.code == ServerReason.plainCode) return reason.message;
  final text = _text(reason);
  if (text != null) return text;
  if (reason.message.isNotEmpty) return _sentence(reason.message);
  return tr('Der Server hat die Anfrage abgelehnt.', 'The server refused.');
}

/// The reason as a sentence: capital first letter, full stop at the end.
String _sentence(String text) {
  final trimmed = text.trim();
  if (trimmed.isEmpty) return trimmed;
  final capital = trimmed[0].toUpperCase() + trimmed.substring(1);
  return RegExp(r'[.!?]$').hasMatch(capital) ? capital : '$capital.';
}

String _number(ServerReason reason, int index) =>
    index < reason.args.length ? '${reason.args[index]}' : '';

String? _text(ServerReason reason) => switch (reason.code) {
  'user_name_empty' => tr(
    'Der Benutzername darf nicht leer sein.',
    'The user name must not be empty.',
  ),
  'user_name_too_long' => tr(
    'Der Benutzername darf höchstens ${_number(reason, 0)} Zeichen haben.',
    'The user name may have at most ${_number(reason, 0)} characters.',
  ),
  'user_name_control_character' => tr(
    'Der Benutzername enthält unzulässige Zeichen.',
    'The user name contains characters that are not allowed.',
  ),
  'password_too_short' => tr(
    'Das Passwort braucht mindestens ${_number(reason, 0)} Zeichen.',
    'The password needs at least ${_number(reason, 0)} characters.',
  ),
  'password_needs_lowercase' => tr(
    'Das Passwort braucht einen Kleinbuchstaben.',
    'The password needs a lowercase letter.',
  ),
  'password_needs_uppercase' => tr(
    'Das Passwort braucht einen Großbuchstaben.',
    'The password needs an uppercase letter.',
  ),
  'password_needs_digit' => tr(
    'Das Passwort braucht eine Ziffer.',
    'The password needs a digit.',
  ),
  'password_needs_special' => tr(
    'Das Passwort braucht ein Sonderzeichen.',
    'The password needs a special character.',
  ),
  'user_name_taken' => tr(
    'Ein Benutzer mit diesem Namen gibt es schon.',
    'A user with this name exists already.',
  ),
  'user_not_found' => tr(
    'Der Benutzer wurde nicht gefunden.',
    'The user was not found.',
  ),
  'setup_done' => tr(
    'Der erste Admin ist schon eingerichtet.',
    'The first admin has been set up already.',
  ),
  'bad_credentials' => tr(
    'Benutzername oder Passwort stimmt nicht.',
    'Wrong user name or password.',
  ),
  'wrong_current_password' => tr(
    'Das aktuelle Passwort stimmt nicht.',
    'The current password is wrong.',
  ),
  'too_many_attempts' => tr(
    'Zu viele Fehlversuche. Bitte in ${_number(reason, 0)} s erneut versuchen.',
    'Too many failed attempts. Please try again in ${_number(reason, 0)} s.',
  ),
  'last_admin' => tr(
    'Mindestens ein aktiver Admin muss bleiben.',
    'At least one active admin must remain.',
  ),
  'guest_account' => tr(
    'Gast ist keine Rolle für ein Konto.',
    'Guest is not a role for an account.',
  ),
  'sign_in_required' => tr('Bitte melde dich an.', 'Please sign in.'),
  'auth_expected' => tr(
    'Die Verbindung braucht zuerst eine Anmeldung.',
    'The connection needs a sign-in first.',
  ),
  'not_allowed' => tr(
    'Dafür fehlt die Berechtigung.',
    'You are not allowed to do this.',
  ),
  'guest_access_off' => tr(
    'Der Gastzugang ist ausgeschaltet.',
    'Guest access is switched off.',
  ),
  'message_unrecognised' => tr(
    'Die Nachricht wurde nicht verstanden.',
    'The message was not understood.',
  ),
  'binary_unsupported' => tr(
    'Binäre Nachrichten werden nicht unterstützt.',
    'Binary messages are not supported.',
  ),
  'recording_unknown' => tr(
    'Die Aufnahme gibt es nicht.',
    'The recording does not exist.',
  ),
  'position_beyond_end' => tr(
    'Die Position liegt hinter dem Ende der Aufnahme.',
    'The position is beyond the end of the recording.',
  ),
  'stems_package_invalid' => tr(
    'Die Datei ist kein gültiges Segmentpaket.',
    'The file is not a valid stem package.',
  ),
  'stems_unknown_recording' => tr(
    'Die Aufnahme zu diesen Segmenten ist nicht in der Bibliothek.',
    'The recording of these stems is not in the library.',
  ),
  'stems_length_mismatch' => tr(
    'Ein Segment ist nicht so lang wie die Aufnahme.',
    'A stem is not as long as the recording.',
  ),
  'stems_rate_mismatch' => tr(
    'Die Segmente haben eine andere Abtastrate als die Aufnahme.',
    'The stems do not have the sample rate of the recording.',
  ),
  'stems_unavailable' => tr(
    'Dieser Server nimmt keine Segmente an.',
    'This server takes no stems.',
  ),
  'stems_analysis_unavailable' => tr(
    'Auf diesem Server ist kein Analyseprogramm eingerichtet.',
    'No analysis program is set up on this server.',
  ),
  'stems_analysis_failed' => tr(
    'Das Analyseprogramm ist fehlgeschlagen.',
    'The analysis program failed.',
  ),
  'stems_model_unknown' => tr(
    'Dieses Trennmodell gibt es nicht.',
    'There is no such separation model.',
  ),
  'bookmark_not_found' => tr(
    'Die Markierung oder Aufnahme wurde nicht gefunden.',
    'The marker or recording was not found.',
  ),
  'bookmark_conflict' => tr(
    'Die Markierung wurde inzwischen geändert.',
    'The marker was changed in the meantime.',
  ),
  'storage_failed' || 'internal_error' => tr(
    'Der Server konnte das nicht speichern.',
    'The server could not store that.',
  ),
  'no_recording_selected' => tr(
    'Es ist keine Aufnahme gewählt.',
    'No recording is selected.',
  ),
  'speed_out_of_range' => tr(
    'Die Geschwindigkeit muss zwischen 0,5 und 2,0 liegen.',
    'The speed must be between 0.5 and 2.0.',
  ),
  'tuning_out_of_range' => tr(
    'Die Stimmung muss zwischen 300 und 500 Hz liegen.',
    'The tuning must be between 300 and 500 Hz.',
  ),
  'key_shift_out_of_range' => tr(
    'Die Tonart kann um höchstens 12 Halbtöne verschoben werden.',
    'The key can be shifted by 12 semitones at most.',
  ),
  'volume_out_of_range' => tr(
    'Die Lautstärke muss zwischen 0 und 100 Prozent liegen.',
    'The volume must be between 0 and 100 percent.',
  ),
  'position_out_of_range' => tr(
    'Die Position liegt außerhalb der Aufnahme.',
    'The position is outside the recording.',
  ),
  'empty_range' => tr(
    'Das Ende muss nach dem Start liegen.',
    'The end must come after the start.',
  ),
  'tempo_not_positive' => tr(
    'Das Tempo muss größer als 0 sein.',
    'The tempo must be greater than 0.',
  ),
  'key_tonic_out_of_range' => tr(
    'Der Grundton der Tonart ist ungültig.',
    'The key’s tonic is not valid.',
  ),
  'empty_name' => tr(
    'Der Name darf nicht leer sein.',
    'The name must not be empty.',
  ),
  'name_too_long' => tr('Der Name ist zu lang.', 'The name is too long.'),
  'outside_recording' => tr(
    'Der Eintrag liegt hinter dem Ende der Aufnahme.',
    'The entry lies beyond the end of the recording.',
  ),
  'import_malformed' => tr(
    'Die Datei ließ sich nicht lesen.',
    'The file could not be read.',
  ),
  'import_format_unsupported' => tr(
    'Diese Version des Dateiformats wird nicht unterstützt.',
    'This version of the file format is not supported.',
  ),
  'csv_invalid' => tr(
    'Die Tabelle ist in Zeile ${_number(reason, 0)} fehlerhaft.',
    'The table has an error in line ${_number(reason, 0)}.',
  ),
  'export_format_unknown' => tr(
    'Dieses Exportformat gibt es nicht.',
    'There is no such export format.',
  ),
  'token_not_found' => tr(
    'Das Token gibt es nicht (mehr).',
    'The token does not exist (any more).',
  ),
  'empty_title' => tr(
    'Der Titel darf nicht leer sein.',
    'The title must not be empty.',
  ),
  'upload_unavailable' => tr(
    'Der Upload ist auf diesem Server nicht eingerichtet.',
    'Uploading is not set up on this server.',
  ),
  'upload_interrupted' => tr(
    'Der Upload wurde unterbrochen.',
    'The upload was interrupted.',
  ),
  'file_empty' => tr('Die Datei ist leer.', 'The file is empty.'),
  'file_too_large' => tr(
    'Die Datei ist größer als das Limit von ${_number(reason, 0)} MB.',
    'The file is larger than the limit of ${_number(reason, 0)} MB.',
  ),
  'file_name_missing' => tr(
    'Der Dateiname fehlt.',
    'The file name is missing.',
  ),
  'file_name_invalid' => tr(
    'Der Dateiname ist ungültig.',
    'The file name is not valid.',
  ),
  'file_name_empty' => tr('Der Dateiname ist leer.', 'The file name is empty.'),
  'file_unreadable' => tr(
    'Die Datei lässt sich nicht lesen.',
    'The file cannot be read.',
  ),
  'audio_unsupported' => tr(
    'Das Audioformat wird nicht unterstützt oder die Datei ist beschädigt.',
    'The audio format is not supported or the file is damaged.',
  ),
  'audio_file_missing' => tr(
    'Zur Aufnahme gibt es keine Audiodatei.',
    'The recording has no audio file.',
  ),
  'analysis_too_short' => tr(
    'Die Aufnahme ist zu kurz für die Analyse.',
    'The recording is too short to analyse.',
  ),
  'analysis_no_tonal_content' => tr(
    'Die Aufnahme hat keinen Klanginhalt, in dem sich eine Tonart finden ließe.',
    'The recording has no tonal content to find a key in.',
  ),
  'analysis_no_rhythm' => tr(
    'Die Aufnahme hat keinen Rhythmus, in dem sich ein Tempo finden ließe.',
    'The recording has no rhythm to find a tempo in.',
  ),
  _ => null,
};
