import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../shared_ui/app_icon_button.dart';
import 'analysis_format.dart';
import 'analysis_summary.dart';
import 'library_texts.dart';
import 'recording_summary.dart';

/// One recording in the list: a direct play button, its title and the state
/// or values of its analysis. Tapping the row asks for its details.
class RecordingTile extends StatelessWidget {
  const RecordingTile({
    required this.recording,
    required this.selected,
    required this.loaded,
    required this.onTap,
    required this.onPlay,
    super.key,
  });

  final RecordingSummary recording;

  /// Its details are shown.
  final bool selected;

  /// The player has it loaded.
  final bool loaded;
  final VoidCallback onTap;
  final VoidCallback onPlay;

  @override
  Widget build(BuildContext context) => ListTile(
    leading: AppIconButton(
      tooltip: LibraryTexts.playRecording(recording.title),
      icon: Icons.play_arrow,
      framed: true,
      onPressed: onPlay,
    ),
    title: Text(recording.title),
    subtitle: analysisLine(recording.analysis).isEmpty
        ? null
        : AnalysisSummary(analysis: recording.analysis),
    trailing: loaded
        ? Icon(
            Icons.graphic_eq,
            color: AppColors.accentSoft,
            semanticLabel: LibraryTexts.loaded,
          )
        : null,
    selected: selected,
    onTap: onTap,
  );
}
