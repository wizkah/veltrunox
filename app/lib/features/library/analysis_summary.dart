import 'package:material_ui/material_ui.dart';

import 'analysis_format.dart';
import 'analysis_info.dart';

/// Key, tempo and tuning of a recording, or the state of its analysis.
class AnalysisSummary extends StatelessWidget {
  const AnalysisSummary({required this.analysis, super.key});

  final AnalysisInfo analysis;

  @override
  Widget build(BuildContext context) => Text(analysisLine(analysis));
}
