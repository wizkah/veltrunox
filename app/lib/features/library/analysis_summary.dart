import 'package:material_ui/material_ui.dart';

import '../../shared_ui/activity_status.dart';
import 'analysis_format.dart';
import 'analysis_info.dart';

/// Key, tempo and tuning of a recording, or the state of its analysis.
class AnalysisSummary extends StatelessWidget {
  const AnalysisSummary({required this.analysis, super.key});

  final AnalysisInfo analysis;

  @override
  Widget build(BuildContext context) {
    final line = analysisLine(analysis);
    return switch (analysis.status) {
      AnalysisStatus.queued || AnalysisStatus.running => ActivityStatus(
        state: ActivityState.loading,
        label: line,
      ),
      AnalysisStatus.failed => ActivityStatus(
        state: ActivityState.error,
        label: line,
      ),
      _ => Text(line),
    };
  }
}
