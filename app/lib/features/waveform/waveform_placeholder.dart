import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import 'waveform_geometry.dart';

/// Stands where the waveform will be while it is not there (loading, or the
/// recording has no scan yet): a message, never a plausible empty waveform.
class WaveformPlaceholder extends StatelessWidget {
  const WaveformPlaceholder({required this.text, super.key});

  final String text;

  @override
  Widget build(BuildContext context) => SizedBox(
    height: WaveformGeometry.waveformHeight,
    child: Center(
      child: Text(
        text,
        style: Theme.of(context).textTheme.bodyMedium
            ?.copyWith(color: AppColors.textSecondary),
      ),
    ),
  );
}
