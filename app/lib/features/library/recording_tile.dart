import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/app_icon_button.dart';
import 'analysis_format.dart';
import 'analysis_summary.dart';
import 'library_texts.dart';
import 'recording_summary.dart';
import 'stems_badge.dart';

/// One recording in the list, as a card: a direct play button, its title and
/// the state or values of its analysis. Tapping the card asks for its
/// details; the card whose details are shown is outlined in orange.
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

  static final _restBorder = AppColors.border.withValues(alpha: 0.6);

  Widget? _subtitle() {
    final analysed = analysisLine(recording.analysis).isNotEmpty;
    if (!analysed && !recording.hasStems) return null;
    return Padding(
      padding: const EdgeInsets.only(top: AppSpace.s1),
      child: Wrap(
        spacing: AppSpace.s3,
        runSpacing: AppSpace.s1,
        crossAxisAlignment: WrapCrossAlignment.center,
        children: [
          if (analysed) AnalysisSummary(analysis: recording.analysis),
          if (recording.hasStems) const StemsBadge(),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final radius = BorderRadius.circular(AppRadius.medium);
    return Padding(
      padding: const EdgeInsets.symmetric(
        horizontal: AppSpace.s4,
        vertical: AppSpace.s1,
      ),
      child: AnimatedContainer(
        duration: AppMotion.quick,
        decoration: BoxDecoration(
          color: selected
              ? Color.alphaBlend(
                  AppColors.action.withValues(alpha: 0.08),
                  AppColors.surface,
                )
              : AppColors.surface.withValues(alpha: 0.5),
          border: Border.all(
            color: selected ? AppColors.action : _restBorder,
            width: AppSize.borderWidth,
          ),
          borderRadius: radius,
          boxShadow: selected
              ? [
                  BoxShadow(
                    color: AppColors.action.withValues(alpha: 0.25),
                    blurRadius: AppSpace.s3,
                  ),
                ]
              : null,
        ),
        // Its own transparent material, so the ink of a tap shows on the card.
        child: Material(
          type: MaterialType.transparency,
          child: ListTile(
            shape: RoundedRectangleBorder(borderRadius: radius),
            selectedColor: AppColors.textPrimary,
            selectedTileColor: Colors.transparent,
            leading: AppIconButton(
              tooltip: LibraryTexts.playRecording(recording.title),
              icon: Icons.play_arrow,
              framed: true,
              onPressed: onPlay,
            ),
            title: Text(recording.title),
            subtitle: _subtitle(),
            trailing: loaded
                ? Icon(
                    Icons.graphic_eq,
                    color: AppColors.accentSoft,
                    semanticLabel: LibraryTexts.loaded,
                  )
                : null,
            selected: selected,
            onTap: onTap,
          ),
        ),
      ),
    );
  }
}
