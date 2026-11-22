/// User-visible texts of the marker feature (German for now).
abstract final class MarkerTexts {
  static const title = 'Markierungen';
  static const empty = 'Noch keine Markierungen.';
  static const emptyFiltered = 'Keine Markierungen in dieser Auswahl.';
  static const loading = 'Markierungen werden geladen …';
  static const failure = 'Die Markierungen konnten nicht geladen werden.';
  static const retry = 'Erneut versuchen';
  static const filterAll = 'Alle';
  static const filterLabels = 'Labels';
  static const filterRegions = 'Regionen';
  static const addRegion = 'Region anlegen';
  static const editRegion = 'Region bearbeiten';
  static const editLabel = 'Label bearbeiten';
  static const edit = 'Bearbeiten';
  static const delete = 'Löschen';
  static const more = 'Weitere Aktionen';
  static const cancel = 'Abbrechen';
  static const save = 'Speichern';
  static const name = 'Name';
  static const start = 'Start';
  static const end = 'Ende';
  static const position = 'Position';
  static const deleteTitle = 'Markierung löschen?';
  static const nameMissing = 'Bitte einen Namen eingeben';
  static const timeUnreadable = 'Zeit nicht lesbar, zum Beispiel 1:07';
  static const endNotAfterStart = 'Das Ende muss nach dem Start liegen';
  static const saveFailed =
      'Die Änderung wurde nicht übernommen. Die Liste wird neu geladen.';

  static String outsideRecording(String length) =>
      'Außerhalb der Aufnahme (Länge $length)';
  static String deleteMessage(String name) =>
      '„$name“ wird gelöscht. Das lässt sich nicht rückgängig machen.';
  static String openMarkers(int count) => 'Markierungen ($count)';
  static String play(String name) => 'Abspielen: $name';
  static String label(String time) => 'Label bei $time';
  static String region(String start, String end) => 'Region $start–$end';
  static String moreFor(String name) => '$more: $name';
}
