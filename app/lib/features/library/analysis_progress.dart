import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';

/// A pending analysis: its state in words and a bar below. The bar shows the
/// reported share while the analysis runs, and runs indeterminately while it
/// waits in the queue, so no progress is pretended.
class AnalysisProgress extends StatelessWidget {
  const AnalysisProgress({required this.label, this.share, super.key});

  final String label;

  /// 0 to 1; `null` while no progress is known.
  final double? share;

  static const _barWidth = 200.0;
  static const _barHeight = 6.0;

  @override
  Widget build(BuildContext context) => Semantics(
    liveRegion: true,
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(
              Icons.graphic_eq,
              size: AppSpace.s4,
              color: AppColors.accent,
            ),
            const SizedBox(width: AppSpace.s2),
            Flexible(child: Text(label)),
          ],
        ),
        const SizedBox(height: AppSpace.s1),
        ExcludeSemantics(
          child: SizedBox(
            width: _barWidth,
            child: LinearProgressIndicator(
              value: share?.clamp(0.0, 1.0),
              minHeight: _barHeight,
              borderRadius: BorderRadius.circular(AppRadius.pill),
              color: AppColors.accent,
              backgroundColor: AppColors.border,
            ),
          ),
        ),
      ],
    ),
  );
}
