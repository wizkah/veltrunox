/// User-visible texts of the waveform feature (German for now).
abstract final class WaveformTexts {
  static const loading = 'Wellenform wird geladen …';
  static const unavailable =
      'Für diese Aufnahme gibt es noch keine Wellenform.';
  static const zoomIn = 'Vergrößern';
  static const zoomOut = 'Verkleinern';
  static const fit = 'Ganze Aufnahme zeigen';
  static const snap = 'Einrasten an Markierungen';
  static const newRegion = 'Region aufziehen';
  static const allRegions = 'Alle Regionen';
  static const fewerRegions = 'Weniger Regionen';
  static const playhead = 'Abspielposition';
  static const regionStart = 'Regionsanfang verschieben';
  static const regionEnd = 'Regionsende verschieben';
  static const groupTitle = 'Labels an dieser Stelle';

  static String fallbackLabel(int id) => 'Label $id';
  static String fallbackRegion(int id) => 'Region $id';
  static String more(int count) => '+$count';
  static String playMarker(String name) => 'Abspielen: $name';
  static String moreRegions(int count) => '$count weitere Regionen';
}
