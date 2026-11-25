import '../../shared_ui/app_language.dart';

/// User-visible texts of the marker feature.
abstract final class MarkerTexts {
  static String get title => tr('Markierungen', 'Markers');
  static String get empty => tr('Noch keine Markierungen.', 'No markers yet.');
  static String get emptyFiltered => tr(
    'Keine Markierungen in dieser Auswahl.',
    'No markers in this selection.',
  );
  static String get loading =>
      tr('Markierungen werden geladen …', 'Loading markers …');
  static String get failure => tr(
    'Die Markierungen konnten nicht geladen werden.',
    'The markers could not be loaded.',
  );
  static String get retry => tr('Erneut versuchen', 'Try again');
  static String get filterAll => tr('Alle', 'All');
  static String get filterLabels => tr('Labels', 'Labels');
  static String get filterRegions => tr('Regionen', 'Regions');
  static String get addRegion => tr('Region anlegen', 'Add region');
  static String get editMarker => tr('Markierung bearbeiten', 'Edit marker');
  static String get edit => tr('Bearbeiten', 'Edit');
  static String get delete => tr('Löschen', 'Delete');
  static String get more => tr('Weitere Aktionen', 'More actions');
  static String get cancel => tr('Abbrechen', 'Cancel');
  static String get save => tr('Speichern', 'Save');
  static String get name => tr('Name', 'Name');
  static String get start => tr('Start', 'Start');
  static String get end => tr('Ende', 'End');
  static String get endOptional =>
      tr('Leer lassen für ein Label', 'Leave empty for a label');
  static String get deleteTitle => tr('Markierung löschen?', 'Delete marker?');
  static String get nameMissing =>
      tr('Bitte einen Namen eingeben', 'Please enter a name');
  static String get timeUnreadable => tr(
    'Zeit nicht lesbar, zum Beispiel 1:07',
    'Time not readable, for example 1:07',
  );
  static String get endNotAfterStart => tr(
    'Das Ende muss nach dem Start liegen',
    'The end must be after the start',
  );
  static String get saveFailed => tr(
    'Die Änderung wurde nicht übernommen. Die Liste wird neu geladen.',
    'The change was not applied. The list is reloaded.',
  );

  /// Another person changed the marker first (D-19): the server's state stands.
  static String get replacedByOther => tr(
    'Jemand anderes hat diese Markierung gleichzeitig geändert. '
        'Es gilt der Stand des Servers.',
    'Someone else changed this marker at the same time. '
        'The server\'s state stands.',
  );

  static String get combineRegions =>
      tr('Regionen kombinieren', 'Combine regions');
  static String get regionFromSelection =>
      tr('Neue Region aus Auswahl', 'New region from selection');
  static String get clearSelection => tr('Auswahl aufheben', 'Clear selection');
  static String selectedCount(int count) =>
      tr('$count ausgewählt', '$count selected');
  static String select(String name) => tr('Auswählen: $name', 'Select: $name');
  static String get activeRegion => tr('Wird abgespielt', 'Playing');

  static String defaultLabelName(int number) => 'Label $number';
  static String defaultRegionName(int number) =>
      tr('Region $number', 'Region $number');
  static String outsideRecording(String length) => tr(
    'Außerhalb der Aufnahme (Länge $length)',
    'Outside the recording (length $length)',
  );
  static String deleteMessage(String name) => tr(
    '„$name“ wird gelöscht. Das lässt sich nicht rückgängig machen.',
    '“$name” is deleted. This cannot be undone.',
  );
  static String openMarkers(int count) =>
      tr('Markierungen ($count)', 'Markers ($count)');
  static String play(String name) => tr('Abspielen: $name', 'Play: $name');
  static String label(String time) => tr('Label bei $time', 'Label at $time');
  static String region(String start, String end) =>
      tr('Region $start–$end', 'Region $start–$end');
  static String moreFor(String name) => '$more: $name';
}
