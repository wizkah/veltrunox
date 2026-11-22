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

  static String playRecording(String title) => 'Abspielen: $title';
  static String analysisFailed(String? reason) => reason == null
      ? 'Analyse fehlgeschlagen'
      : 'Analyse fehlgeschlagen: $reason';
}
