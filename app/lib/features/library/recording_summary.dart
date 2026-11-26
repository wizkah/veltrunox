import 'analysis_info.dart';

/// A recording as listed in the library.
class RecordingSummary {
  const RecordingSummary({
    required this.id,
    required this.title,
    this.lengthMs,
    this.analysis = const AnalysisInfo(),
    this.hasStems = false,
  });

  final int id;
  final String title;

  /// Length of the audio; `null` until the server has scanned the recording.
  final int? lengthMs;
  final AnalysisInfo analysis;

  /// Whether it has been split into stems.
  final bool hasStems;
}
