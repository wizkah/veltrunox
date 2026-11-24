import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import 'waveform_geometry.dart';

/// Stands where the waveform will be while it is not there (loading, or the
/// recording has no scan yet): the empty stage with an icon and a message,
/// never a plausible empty waveform.
class WaveformPlaceholder extends StatelessWidget {
  const WaveformPlaceholder({required this.text, super.key});

  final String text;

  @override
  Widget build(BuildContext context) => Container(
    height: WaveformGeometry.waveformHeight,
    decoration: BoxDecoration(
      color: AppColors.canvas,
      border: Border.all(
        color: Color.lerp(AppColors.border, AppColors.accent, 0.25)!,
      ),
      borderRadius: BorderRadius.circular(AppRadius.small),
    ),
    child: Center(
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.graphic_eq, color: AppColors.accent),
          const SizedBox(width: AppSpace.s2),
          Flexible(
            child: Text(
              text,
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodyMedium
                  ?.copyWith(color: AppColors.textSecondary),
            ),
          ),
        ],
      ),
    ),
  );
}
