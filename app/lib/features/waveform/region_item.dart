import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import '../../design/tokens/app_typography.dart';
import '../markers/marker.dart';
import 'waveform_geometry.dart';
import 'waveform_texts.dart';

/// One region in its lane. The title is the play action; a long press opens
/// the actions, a plain tap on the body selects it for editing. The selected
/// one is orange, the others use a secondary tone chosen by [toneIndex].
class RegionItem extends StatelessWidget {
  const RegionItem({
    required this.region,
    required this.selected,
    required this.toneIndex,
    required this.onPlay,
    required this.onSelect,
    required this.onContext,
    this.showSelectButton = true,
    super.key,
  });

  final RegionMarker region;
  final bool selected;
  final int toneIndex;
  final VoidCallback onPlay;
  final VoidCallback onSelect;
  final VoidCallback onContext;

  /// Hidden on regions too narrow for it; they are selected from the list.
  final bool showSelectButton;

  static const _tones = [AppColors.accent, AppColors.regionOther];

  @override
  Widget build(BuildContext context) {
    final name = region.title.isEmpty
        ? WaveformTexts.fallbackRegion(region.id)
        : region.title;
    final tone = selected
        ? AppColors.action
        : _tones[toneIndex % _tones.length];
    return Semantics(
      container: true,
      selected: selected,
      label: name,
      child: Container(
        height: WaveformGeometry.laneHeight,
        decoration: BoxDecoration(
          color: AppColors.surface,
          border: Border.all(color: tone, width: selected ? 2 : 1),
          borderRadius: BorderRadius.circular(AppRadius.small),
        ),
        child: Row(
          children: [
            Expanded(
              child: InkWell(
                onTap: onPlay,
                onLongPress: onContext,
                borderRadius: BorderRadius.circular(AppRadius.small),
                child: Padding(
                  padding: const EdgeInsets.symmetric(horizontal: AppSpace.s2),
                  child: Align(
                    alignment: Alignment.centerLeft,
                    child: Text(
                      name,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(
                        color: AppColors.textPrimary,
                        fontSize: AppTypography.sizeSmall,
                      ),
                    ),
                  ),
                ),
              ),
            ),
            if (showSelectButton)
              InkWell(
                onTap: onSelect,
                child: SizedBox(
                  width: AppSpace.s6,
                  height: WaveformGeometry.laneHeight,
                  child: Icon(
                    selected ? Icons.edit : Icons.edit_outlined,
                    size: AppSpace.s4,
                    color: tone,
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}
