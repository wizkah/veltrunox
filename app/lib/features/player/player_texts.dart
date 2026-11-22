/// User-visible texts of the player feature (German for now).
abstract final class PlayerTexts {
  static const noAudioDevice =
      'Der Server hat kein Audiogerät. Es ist nichts zu hören.';
  static const nothingSelected = 'Keine Aufnahme gewählt';
  static const play = 'Abspielen';
  static const pause = 'Pause';
  static const restart = 'Zum Anfang';
  static const speed = 'Geschwindigkeit';
  static const tuning = 'Stimmung';
  static const key = 'Tonart';
  static const keyOriginal = 'Original';
  static const tempo = 'Tempo';
  static const pitch = 'Tonhöhe';
  static const targetTempoRow = 'Ziel-bpm';
  static const tempoInvalid = 'Keine Zahl';
  static const tempoOutOfRange = 'Außerhalb von 0,5× bis 2×';
  static const position = 'Position';
  static const tuningInvalid = 'Keine Zahl';
  static const tuningOutOfRange = 'Nur 300–500 Hz erlaubt';
  static const volume = 'Lautstärke';
  static const keyChoice = 'Tonart wählen';
  static const choose = 'Aufnahme wählen';
  static const nothingSelectedHint =
      'Wähle in der Bibliothek eine Aufnahme, um sie abzuspielen.';
  static const waiting = 'Wiedergabe wird geladen …';
  static const positionChanged = 'Die Wiedergabeposition wurde geändert.';
  static const playModeToEnd = 'Bis Ende';
  static const playModeToRegionEnd = 'Regionsende';
  static const playModeLoop = 'Schleife';
  static const resetToAnalysis = 'Auf Analysewert zurücksetzen';

  static String playModeTooltip(String mode) =>
      'Abspielmodus: $mode. Tippen zum Wechseln.';
  static String original(String value) => 'Original: $value';
  static String speedValue(double factor) => '${factor.toStringAsFixed(2)}×';
  static String volumeValue(double percent) => '${percent.round()} %';
  static String tuningValue(double hertz) => '${decimal(hertz)} Hz';
  static String tempoValue(double bpm) => '${decimal(bpm)} bpm';

  /// One decimal with a decimal comma, like "440,0".
  static String decimal(double value) =>
      value.toStringAsFixed(1).replaceAll('.', ',');
  static String keyValue(int semitones) =>
      semitones > 0 ? '+$semitones' : '$semitones';
}
