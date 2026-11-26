import '../../shared_ui/app_language.dart';

/// User-visible texts of the mixer.
abstract final class MixerTexts {
  static String get stems => tr('Segmente', 'Stems');
  static String get originalPlays => tr(
    'Es spielt die Originalaufnahme.',
    'The original recording is playing.',
  );
  static String get none => tr(
    'Diese Aufnahme hat noch keine Segmente.',
    'This recording has no stems yet.',
  );
  static String get loading =>
      tr('Segmente werden geladen …', 'Loading stems …');
  static String get split => tr('Aufteilen', 'Split');
  static String splitting(int percent) => percent == 0
      ? tr('Wird aufgeteilt …', 'Splitting …')
      : tr('Wird aufgeteilt ($percent %)', 'Splitting ($percent %)');
  static String get createRegions => tr('Regionen erstellen', 'Create regions');
  static String creatingRegions(int percent) => percent == 0
      ? tr('Regionen werden erstellt …', 'Creating regions …')
      : tr(
          'Regionen werden erstellt ($percent %)',
          'Creating regions ($percent %)',
        );
  static String get regionsCreated =>
      tr('Die Regionen stehen im Player.', 'The regions are in the player.');
  static String get import => tr('Importieren', 'Import');
  static String get export => tr('Exportieren', 'Export');
  static String get cancel => tr('Abbrechen', 'Cancel');
  static String get replaceTitle =>
      tr('Segmente ersetzen?', 'Replace the stems?');
  static String get replaceMessage => tr(
    'Die vorhandenen Segmente der Aufnahme im Paket werden ersetzt, '
        'ihre Mischung wird zurückgesetzt.',
    'The existing stems of the recording in the package are replaced and '
        'their mix is reset.',
  );
  static String get replace => tr('Ersetzen', 'Replace');
  static String imported(String title, int stems, int regions) => tr(
    '„$title“: $stems Segmente importiert, $regions Regionen hinzugefügt.',
    '"$title": $stems stems imported, $regions regions added.',
  );
  static String get exported => tr('Gespeichert', 'Saved');
  static String volumeOf(String name) =>
      tr('Lautstärke $name', 'Volume of $name');
  static String soloOf(String name) => tr('Solo $name', 'Solo $name');
  static String muteOf(String name) => tr('Stumm $name', 'Mute $name');
  static String get muted => tr('stumm', 'muted');
  static String get notInSolo => tr('nicht im Solo', 'not soloed');
  static String get zoomIn => tr('Hineinzoomen', 'Zoom in');
  static String get zoomOut => tr('Herauszoomen', 'Zoom out');
  static String get fit => tr('Ganze Aufnahme zeigen', 'Fit whole recording');
}
