import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import '../../design/tokens/app_typography.dart';
import '../markers/marker.dart';
import 'label_clusters.dart';
import 'time_viewport.dart';
import 'waveform_geometry.dart';
import 'waveform_texts.dart';

const _charWidthPx = 8.0;
const _chipPaddingPx = 16.0;
const _maxChipWidthPx = 120.0;
const _badgeWidthPx = 28.0;

/// The width a label's chip takes; the layout of the clusters depends on it.
double labelChipWidth(LabelMarker label) =>
    (_displayName(label).length * _charWidthPx + _chipPaddingPx).clamp(
      _chipPaddingPx * 2,
      _maxChipWidthPx,
    );

String _displayName(LabelMarker label) =>
    label.title.isEmpty ? WaveformTexts.fallbackLabel(label.id) : label.title;

/// The point markers above the waveform: a marker line and a name chip each.
/// Chips that would collide are shown as one chip with a `+N` count.
class LabelLayer extends StatelessWidget {
  const LabelLayer({
    required this.labels,
    required this.viewport,
    required this.onPlay,
    required this.onOpenGroup,
    required this.onContext,
    super.key,
  });

  final List<LabelMarker> labels;
  final TimeViewport viewport;

  /// A single label was tapped: play from its position.
  final ValueChanged<LabelMarker> onPlay;

  /// A group was tapped: let the user pick one of its labels.
  final ValueChanged<LabelCluster> onOpenGroup;
  final ValueChanged<LabelMarker> onContext;

  @override
  Widget build(BuildContext context) {
    final clusters = clusterLabels(
      labels,
      viewport,
      chipWidthPx: (label) => labelChipWidth(label) + _badgeWidthPx,
    );
    return SizedBox(
      width: viewport.widthPx,
      height: WaveformGeometry.labelBandHeight,
      child: Stack(
        clipBehavior: Clip.hardEdge,
        children: [
          for (final cluster in clusters)
            Positioned(
              left: cluster.xPx,
              top: 0,
              child: _LabelChip(
                cluster: cluster,
                onTap: () => cluster.isGroup
                    ? onOpenGroup(cluster)
                    : onPlay(cluster.first),
                onLongPress: () => onContext(cluster.first),
              ),
            ),
        ],
      ),
    );
  }
}

class _LabelChip extends StatelessWidget {
  const _LabelChip({
    required this.cluster,
    required this.onTap,
    required this.onLongPress,
  });

  final LabelCluster cluster;
  final VoidCallback onTap;
  final VoidCallback onLongPress;

  @override
  Widget build(BuildContext context) {
    final name = _displayName(cluster.first);
    return Semantics(
      button: true,
      label: cluster.isGroup
          ? '${WaveformTexts.groupTitle}: $name ${WaveformTexts.more(cluster.hiddenCount)}'
          : WaveformTexts.playMarker(name),
      child: ExcludeSemantics(
        child: InkWell(
          onTap: onTap,
          onLongPress: onLongPress,
          child: SizedBox(
            height: WaveformGeometry.labelBandHeight,
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Container(
                  width: 2,
                  height: WaveformGeometry.labelBandHeight,
                  color: AppColors.accentSoft,
                ),
                Container(
                  constraints: const BoxConstraints(maxWidth: _maxChipWidthPx),
                  padding: const EdgeInsets.symmetric(horizontal: AppSpace.s2),
                  decoration: BoxDecoration(
                    color: AppColors.surfaceRaised,
                    border: Border.all(color: AppColors.border),
                    borderRadius: BorderRadius.circular(AppRadius.small),
                  ),
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
                if (cluster.isGroup)
                  Padding(
                    padding: const EdgeInsets.only(left: AppSpace.s1),
                    child: Text(
                      WaveformTexts.more(cluster.hiddenCount),
                      style: const TextStyle(
                        color: AppColors.action,
                        fontSize: AppTypography.sizeSmall,
                        fontWeight: AppTypography.semibold,
                      ),
                    ),
                  ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
