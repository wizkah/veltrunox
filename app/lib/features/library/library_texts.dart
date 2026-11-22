/// User-visible texts of the library feature (German for now).
abstract final class LibraryTexts {
  static const title = 'Aufnahmen';
  static const empty = 'Noch keine Aufnahmen.';
  static const noMatch = 'Keine Aufnahme passt zur Suche.';
  static const loading = 'Aufnahmen werden geladen …';
  static const failure = 'Die Aufnahmen konnten nicht geladen werden.';
  static const retry = 'Erneut versuchen';
  static const major = 'Dur';
  static const minor = 'Moll';
  static const analysisQueued = 'Analyse wartet …';
  static const analysisRunning = 'Analyse läuft …';
  static const reanalyse = 'Erneut analysieren';
  static const search = 'Suchen: Titel, Tonart, Tempo';
  static const clearSearch = 'Suche löschen';
  static const sort = 'Sortieren';
  static const sortTitle = 'Titel';
  static const sortTempo = 'Tempo';
  static const sortKey = 'Tonart';
  static const filterAll = 'Alle';
  static const filterAnalysed = 'Analysiert';
  static const filterAnalysing = 'Wird analysiert';
  static const filterFailed = 'Fehlgeschlagen';
  static const loaded = 'Im Player geladen';
  static const openInPlayer = 'Im Player öffnen';
  static const play = 'Abspielen';
  static const back = 'Zurück zur Liste';
  static const keyLabel = 'Tonart';
  static const tempoLabel = 'Tempo';
  static const tuningLabel = 'Stimmung';
  static const notAnalysed = 'Noch nicht analysiert';

  static const upload = 'Aufnahme hochladen';
  static const uploadDismiss = 'Meldung schließen';
  static const rename = 'Umbenennen';
  static const renameTitle = 'Aufnahme umbenennen';
  static const renameField = 'Titel';
  static const renameEmpty = 'Der Titel darf nicht leer sein.';
  static const save = 'Speichern';
  static const cancel = 'Abbrechen';
  static const delete = 'Löschen';
  static const deleteTitle = 'Aufnahme löschen?';
  static const actionFailed = 'Das ist nicht gelungen';

  static String uploading(String fileName) => 'Wird hochgeladen: $fileName';
  static String uploaded(String title) => 'Hochgeladen: $title';
  static String uploadedAlready(String title) =>
      'Schon vorhanden: $title (nicht noch einmal angelegt)';
  static String uploadFailed(String fileName, String reason) =>
      fileName.isEmpty ? reason : '$fileName: $reason';
  static String deleteMessage(String title) =>
      '„$title“ wird mit Markierungen und Analyse für alle gelöscht. '
      'Das lässt sich nicht rückgängig machen.';

  static String playRecording(String title) => 'Abspielen: $title';
  static String analysisFailed(String? reason) => reason == null
      ? 'Analyse fehlgeschlagen'
      : 'Analyse fehlgeschlagen: $reason';
}
