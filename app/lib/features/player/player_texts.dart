import '../../shared_ui/app_language.dart';

/// User-visible texts of the player feature.
abstract final class PlayerTexts {
  static String get noAudioDevice => tr(
    'Der Server hat kein Audiogerät. Es ist nichts zu hören.',
    'The server has no audio device. Nothing can be heard.',
  );
  static String get nothingSelected =>
      tr('Keine Aufnahme gewählt', 'No recording chosen');
  static String get play => tr('Abspielen', 'Play');
  static String get pause => tr('Pause', 'Pause');
  static String get restart => tr('Zum Anfang', 'To the start');
  static String get speed => tr('Geschwindigkeit', 'Speed');
  static String get tuning => tr('Stimmung', 'Tuning');
  static String get key => tr('Tonart', 'Key');
  static String get keyOriginal => tr('Original', 'Original');
  static String get tempo => tr('Tempo', 'Tempo');
  static String get pitch => tr('Tonhöhe', 'Pitch');
  static String get targetTempoRow => tr('Ziel-bpm', 'Target bpm');
  static String get tempoInvalid => tr('Keine Zahl', 'Not a number');
  static String get tempoOutOfRange =>
      tr('Außerhalb von 0,5× bis 2×', 'Outside 0.5× to 2×');
  static String get position => tr('Position', 'Position');
  static String get tuningInvalid => tr('Keine Zahl', 'Not a number');
  static String get tuningOutOfRange =>
      tr('Nur 300–500 Hz erlaubt', 'Only 300–500 Hz allowed');
  static String get volume => tr('Lautstärke', 'Volume');
  static String get keyChoice => tr('Tonart wählen', 'Choose key');
  static String get choose => tr('Aufnahme wählen', 'Choose a recording');
  static String get nothingSelectedHint => tr(
    'Wähle in der Bibliothek eine Aufnahme, um sie abzuspielen.',
    'Choose a recording in the library to play it.',
  );
  static String get waiting =>
      tr('Wiedergabe wird geladen …', 'Loading playback …');
  static String get positionChanged => tr(
    'Die Wiedergabeposition wurde geändert.',
    'The playback position was changed.',
  );
  static String get playModeToEnd => tr('Bis Ende', 'To the end');
  static String get playModeToRegionEnd => tr('Regionsende', 'Region end');
  static String get playModeLoop => tr('Schleife', 'Loop');
  static String get resetToAnalysis =>
      tr('Auf Analysewert zurücksetzen', 'Reset to the analysed value');

  static String playModeTooltip(String mode) => tr(
    'Abspielmodus: $mode. Tippen zum Wechseln.',
    'Play mode: $mode. Tap to change.',
  );
  static String original(String value) =>
      tr('Original: $value', 'Original: $value');
  static String speedValue(double factor) => '${factor.toStringAsFixed(2)}×';
  static String volumeValue(double percent) => '${percent.round()} %';
  static String tuningValue(double hertz) => '${decimal(hertz)} Hz';
  static String tempoValue(double bpm) => '${decimal(bpm)} bpm';

  /// One decimal with the decimal separator of the language, like "440,0".
  static String decimal(double value) => decimalText(value);
  static String keyValue(int semitones) =>
      semitones > 0 ? '+$semitones' : '$semitones';
}
