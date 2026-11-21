/// User-visible texts of the library feature (German for now).
abstract final class LibraryTexts {
  static const title = 'Aufnahmen';
  static const empty = 'Noch keine Aufnahmen.';
  static const failure = 'Die Aufnahmen konnten nicht geladen werden.';
  static const retry = 'Erneut versuchen';
  static const major = 'Dur';
  static const minor = 'Moll';
  static const analysisQueued = 'Analyse wartet …';
  static const analysisRunning = 'Analyse läuft …';
  static const reanalyse = 'Erneut analysieren';

  static String analysisFailed(String? reason) => reason == null
      ? 'Analyse fehlgeschlagen'
      : 'Analyse fehlgeschlagen: $reason';
}
