import '../../shared_ui/app_language.dart';
import '../../shared_ui/server_reason.dart';

/// User-visible texts of the library feature.
abstract final class LibraryTexts {
  static String get title => tr('Aufnahmen', 'Recordings');
  static String get empty => tr('Noch keine Aufnahmen.', 'No recordings yet.');
  static String get noMatch =>
      tr('Keine Aufnahme passt zur Suche.', 'No recording matches the search.');
  static String get loading =>
      tr('Aufnahmen werden geladen …', 'Loading recordings …');
  static String get failure => tr(
    'Die Aufnahmen konnten nicht geladen werden.',
    'The recordings could not be loaded.',
  );
  static String get retry => tr('Erneut versuchen', 'Try again');
  static String get major => tr('Dur', 'major');
  static String get minor => tr('Moll', 'minor');
  static String get analysisQueued =>
      tr('Analyse wartet …', 'Analysis waiting …');
  static String analysisRunning(int percent) =>
      tr('Analyse läuft … $percent %', 'Analysing … $percent %');
  static String get stems => tr('Segmente vorhanden', 'Has stems');
  static String get search =>
      tr('Suchen: Titel, Tonart, Tempo', 'Search: title, key, tempo');
  static String get clearSearch => tr('Suche löschen', 'Clear search');
  static String get sort => tr('Sortieren', 'Sort');
  static String get sortTitle => tr('Titel', 'Title');
  static String get sortTempo => tr('Tempo', 'Tempo');
  static String get sortKey => tr('Tonart', 'Key');
  static String get loaded => tr('Im Player geladen', 'Loaded in the player');
  static String get openInPlayer =>
      tr('Im Player öffnen', 'Open in the player');
  static String get play => tr('Abspielen', 'Play');
  static String get chooseForDetails => tr(
    'Wähle eine Aufnahme, um ihre Details zu sehen.',
    'Choose a recording to see its details.',
  );
  static String get back => tr('Zurück zur Liste', 'Back to the list');
  static String get lengthLabel => tr('Länge', 'Length');
  static String get notAnalysed =>
      tr('Noch nicht analysiert', 'Not analysed yet');

  /// [local]: the files are on the machine of the application's own server,
  /// so the person opens them rather than uploading them.
  static String upload({bool local = false}) => local
      ? tr('Aufnahmen öffnen', 'Open recordings')
      : tr('Aufnahmen hochladen', 'Upload recordings');
  static String get uploadDismiss => tr('Meldung schließen', 'Dismiss message');
  static String get rename => tr('Umbenennen', 'Rename');
  static String get renameTitle =>
      tr('Aufnahme umbenennen', 'Rename recording');
  static String get renameField => tr('Titel', 'Title');
  static String get renameEmpty =>
      tr('Der Titel darf nicht leer sein.', 'The title must not be empty.');
  static String get save => tr('Speichern', 'Save');
  static String get cancel => tr('Abbrechen', 'Cancel');
  static String get delete => tr('Löschen', 'Delete');
  static String get deleteTitle => tr('Aufnahme löschen?', 'Delete recording?');
  static String get actionFailed =>
      tr('Das ist nicht gelungen', 'That did not work');

  /// [position] of [total] files, counted from 1; no count for a single file.
  /// [percent] of the file the server has received; `null` before its first report.
  static String uploading(
    String fileName, {
    int position = 1,
    int total = 1,
    int? percent,
    bool local = false,
  }) {
    final count = total > 1 ? ' ($position/$total)' : '';
    final done = percent == null ? '' : ' – $percent\u00a0%';
    return local
        ? tr(
            'Wird geöffnet: $fileName$count$done',
            'Opening: $fileName$count$done',
          )
        : tr(
            'Wird hochgeladen: $fileName$count$done',
            'Uploading: $fileName$count$done',
          );
  }

  static String uploaded(String title, {bool local = false}) => local
      ? tr('Geöffnet: $title', 'Opened: $title')
      : tr('Hochgeladen: $title', 'Uploaded: $title');
  static String uploadedAlready(String title) => tr(
    'Schon vorhanden: $title (nicht noch einmal angelegt)',
    'Already there: $title (not added again)',
  );
  static String uploadFailed(String fileName, String reason) =>
      fileName.isEmpty ? reason : '$fileName: $reason';

  /// The result of uploading several files.
  static String uploadSummary({
    required int uploaded,
    required int existing,
    required int failed,
    bool local = false,
  }) {
    final de = [
      if (uploaded > 0) '$uploaded ${local ? 'geöffnet' : 'hochgeladen'}',
      if (existing > 0) '$existing schon vorhanden',
      if (failed > 0) '$failed fehlgeschlagen',
    ];
    final en = [
      if (uploaded > 0) '$uploaded ${local ? 'opened' : 'uploaded'}',
      if (existing > 0) '$existing already there',
      if (failed > 0) '$failed failed',
    ];
    return tr(de.join(', '), en.join(', '));
  }

  static String deleteMessage(String title) => tr(
    '„$title“ wird mit Markierungen und Analyse für alle gelöscht. '
        'Das lässt sich nicht rückgängig machen.',
    '“$title” is deleted for everyone, with its markers and analysis. '
        'This cannot be undone.',
  );

  static String playRecording(String title) =>
      tr('Abspielen: $title', 'Play: $title');
  static String analysisFailed(String? code) {
    final shown = code == null ? null : knownReasonText(ServerReason(code));
    if (shown == null) return tr('Analyse fehlgeschlagen', 'Analysis failed');
    return tr('Analyse fehlgeschlagen: $shown', 'Analysis failed: $shown');
  }
}
