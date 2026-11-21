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

  static String speedValue(double factor) => '${factor.toStringAsFixed(2)}×';
  static String tuningValue(double hertz) => '${hertz.round()} Hz';
  static String keyValue(int semitones) =>
      semitones > 0 ? '+$semitones' : '$semitones';
}
