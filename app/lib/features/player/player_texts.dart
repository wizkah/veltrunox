/// User-visible texts of the player feature (German for now).
abstract final class PlayerTexts {
  static const nothingSelected = 'Keine Aufnahme gewählt';
  static const play = 'Abspielen';
  static const pause = 'Pause';
  static const restart = 'Zum Anfang';
  static const speed = 'Geschwindigkeit';
  static const tuning = 'Stimmung';
  static const key = 'Tonart';
  static const keyLower = 'Tonart tiefer';
  static const keyHigher = 'Tonart höher';
  static const tempo = 'Tempo';
  static const targetTempo = 'Soll-bpm';
  static const tempoInvalid = 'Keine Zahl';
  static const tempoOutOfRange = 'Außerhalb von 0,5× bis 2×';
  static const position = 'Position';
  static const tuningTarget = 'Soll-Hz';
  static const tuningInvalid = 'Keine Zahl';
  static const tuningOutOfRange = 'Nur 300–500 Hz erlaubt';
  static const volume = 'Lautstärke';
  static const resetToAnalysis = 'Auf Analysewert zurücksetzen';

  static String speedValue(double factor) => '${factor.toStringAsFixed(2)}×';
  static String volumeValue(double percent) => '${percent.round()} %';
  static String tuningValue(double hertz) => '${hertz.round()} Hz';
  static String tempoValue(double bpm) =>
      '${bpm.toStringAsFixed(1).replaceAll('.', ',')} bpm';
  static String keyValue(int semitones) =>
      semitones > 0 ? '+$semitones' : '$semitones';
}
