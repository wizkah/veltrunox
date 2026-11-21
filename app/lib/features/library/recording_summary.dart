import 'analysis_info.dart';

/// A recording as listed in the library.
class RecordingSummary {
  const RecordingSummary({
    required this.id,
    required this.title,
    this.analysis = const AnalysisInfo(),
  });

  final int id;
  final String title;
  final AnalysisInfo analysis;
}
