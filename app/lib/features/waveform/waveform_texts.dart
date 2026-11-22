import '../../shared_ui/app_language.dart';

/// User-visible texts of the waveform feature.
abstract final class WaveformTexts {
  static String get title => tr('Wellenform', 'Waveform');
  static String get loading =>
      tr('Wellenform wird geladen …', 'Loading waveform …');
  static String get unavailable => tr(
    'Für diese Aufnahme gibt es noch keine Wellenform.',
    'This recording has no waveform yet.',
  );
  static String get zoomIn => tr('Vergrößern', 'Zoom in');
  static String get zoomOut => tr('Verkleinern', 'Zoom out');
  static String get fit => tr('Ganze Aufnahme zeigen', 'Show whole recording');
  static String get snap => tr('Einrasten an Markierungen', 'Snap to markers');
  static String get setLabelHere => tr('Label hier setzen', 'Set label here');
  static String get setRegionHere =>
      tr('Region hier anlegen', 'Add region here');
  static String get allRegions => tr('Alle Regionen', 'All regions');
  static String get fewerRegions => tr('Weniger Regionen', 'Fewer regions');
  static String get playhead => tr('Abspielposition', 'Playback position');
  static String get regionStart =>
      tr('Regionsanfang verschieben', 'Move region start');
  static String get regionEnd =>
      tr('Regionsende verschieben', 'Move region end');
  static String get groupTitle =>
      tr('Labels an dieser Stelle', 'Labels at this point');

  static String fallbackLabel(int id) => 'Label $id';
  static String fallbackRegion(int id) => tr('Region $id', 'Region $id');
  static String more(int count) => '+$count';
  static String playMarker(String name) =>
      tr('Abspielen: $name', 'Play: $name');
  static String moreRegions(int count) =>
      tr('$count weitere Regionen', '$count more regions');
}
