import 'package:material_ui/material_ui.dart';

import 'analysis_format.dart';
import 'analysis_summary.dart';
import 'reanalyse_button.dart';
import 'recording_summary.dart';

class RecordingTile extends StatelessWidget {
  const RecordingTile({
    required this.recording,
    required this.selected,
    required this.onTap,
    required this.onReanalyse,
    super.key,
  });

  final RecordingSummary recording;
  final bool selected;
  final VoidCallback onTap;
  final VoidCallback onReanalyse;

  @override
  Widget build(BuildContext context) => ListTile(
    leading: const Icon(Icons.music_note),
    title: Text(recording.title),
    subtitle: analysisLine(recording.analysis).isEmpty
        ? null
        : AnalysisSummary(analysis: recording.analysis),
    trailing: ReanalyseButton(
      status: recording.analysis.status,
      onPressed: onReanalyse,
    ),
    selected: selected,
    onTap: onTap,
  );
}
