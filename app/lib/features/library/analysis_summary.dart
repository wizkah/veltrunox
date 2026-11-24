import 'package:material_ui/material_ui.dart';

import '../../shared_ui/activity_status.dart';
import 'analysis_chips.dart';
import 'analysis_info.dart';
import 'analysis_progress.dart';
import 'library_texts.dart';

/// The state of a recording's analysis while it is pending or failed, else
/// its key, tempo and tuning. A recording analysed again shows the progress,
/// not the values that are about to be replaced.
class AnalysisSummary extends StatelessWidget {
  const AnalysisSummary({required this.analysis, super.key});

  final AnalysisInfo analysis;

  @override
  Widget build(BuildContext context) {
    final values = analysis.values;
    return switch (analysis.status) {
      AnalysisStatus.queued => AnalysisProgress(
        label: LibraryTexts.analysisQueued,
      ),
      AnalysisStatus.running => AnalysisProgress(
        label: LibraryTexts.analysisRunning(analysis.percent),
        share: analysis.percent / 100,
      ),
      AnalysisStatus.failed => ActivityStatus(
        state: ActivityState.error,
        label: LibraryTexts.analysisFailed(analysis.failure),
      ),
      _ when values != null => AnalysisChips(values: values),
      _ => const SizedBox.shrink(),
    };
  }
}
