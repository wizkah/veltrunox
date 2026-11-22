import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import 'waveform_geometry.dart';

/// A drag handle at one end of the selected region. Reports the horizontal
/// movement in pixels; the owner turns it into time. The touch area is wider
/// than the drawn handle.
class RegionHandle extends StatelessWidget {
  const RegionHandle({
    required this.label,
    required this.onDragStart,
    required this.onDragUpdate,
    required this.onDragEnd,
    super.key,
  });

  final String label;
  final VoidCallback onDragStart;
  final ValueChanged<double> onDragUpdate;
  final VoidCallback onDragEnd;

  @override
  Widget build(BuildContext context) => Semantics(
    label: label,
    child: GestureDetector(
      behavior: HitTestBehavior.opaque,
      onHorizontalDragStart: (_) => onDragStart(),
      onHorizontalDragUpdate: (details) => onDragUpdate(details.delta.dx),
      onHorizontalDragEnd: (_) => onDragEnd(),
      onHorizontalDragCancel: onDragEnd,
      child: SizedBox(
        width: WaveformGeometry.handleHitWidth,
        height: WaveformGeometry.laneHeight,
        child: Center(
          child: Container(
            width: AppSize.regionHandleVisualWidth,
            height: WaveformGeometry.laneHeight - AppSpace.s2,
            decoration: BoxDecoration(
              color: AppColors.action,
              borderRadius: BorderRadius.circular(AppRadius.small),
            ),
          ),
        ),
      ),
    ),
  );
}
