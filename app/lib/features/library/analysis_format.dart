import 'analysis_info.dart';
import 'library_texts.dart';

const _tonics = [
  'C',
  'C#',
  'D',
  'D#',
  'E',
  'F',
  'F#',
  'G',
  'G#',
  'A',
  'A#',
  'B',
];

/// "A-Moll", "C#-Dur".
String keyName(int tonic, {required bool minor}) =>
    '${_tonics[tonic % 12]}-${minor ? LibraryTexts.minor : LibraryTexts.major}';

/// One decimal with a decimal comma, like "96,5".
String decimal(double value) => value.toStringAsFixed(1).replaceAll('.', ',');

/// The one line under a recording's title.
String analysisLine(AnalysisInfo analysis) {
  final values = analysis.values;
  if (values != null) {
    final key = keyName(values.keyTonic, minor: values.keyMinor);
    return '$key · ${decimal(values.tempoBpm)} bpm · '
        '${decimal(values.tuningHz)} Hz';
  }
  return switch (analysis.status) {
    AnalysisStatus.queued => LibraryTexts.analysisQueued,
    AnalysisStatus.running => LibraryTexts.analysisRunning,
    AnalysisStatus.failed => LibraryTexts.analysisFailed(analysis.failure),
    AnalysisStatus.none || AnalysisStatus.done => '',
  };
}
