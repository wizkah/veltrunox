import 'package:material_ui/material_ui.dart';

import '../../shared_ui/app_icon_button.dart';
import 'waveform_texts.dart';

/// Local view controls: zoom, whole recording, snapping, and creating a label or a
/// region at the playhead.
class WaveformToolbar extends StatelessWidget {
  const WaveformToolbar({
    required this.snapEnabled,
    required this.onZoomIn,
    required this.onZoomOut,
    required this.onFit,
    required this.onToggleSnap,
    required this.onSetLabel,
    required this.onSetRegion,
    super.key,
  });

  final bool snapEnabled;
  final VoidCallback onZoomIn;
  final VoidCallback onZoomOut;
  final VoidCallback onFit;
  final VoidCallback onToggleSnap;
  final VoidCallback onSetLabel;
  final VoidCallback onSetRegion;

  @override
  Widget build(BuildContext context) => Wrap(
    crossAxisAlignment: WrapCrossAlignment.center,
    children: [
      AppIconButton(
        tooltip: WaveformTexts.zoomOut,
        icon: Icons.zoom_out,
        onPressed: onZoomOut,
      ),
      AppIconButton(
        tooltip: WaveformTexts.zoomIn,
        icon: Icons.zoom_in,
        onPressed: onZoomIn,
      ),
      AppIconButton(
        tooltip: WaveformTexts.fit,
        icon: Icons.fit_screen,
        onPressed: onFit,
      ),
      AppIconButton(
        tooltip: WaveformTexts.snap,
        icon: Icons.align_horizontal_center,
        selected: snapEnabled,
        onPressed: onToggleSnap,
      ),
      AppIconButton(
        tooltip: WaveformTexts.setLabelHere,
        icon: Icons.add_location_alt_outlined,
        onPressed: onSetLabel,
      ),
      AppIconButton(
        tooltip: WaveformTexts.setRegionHere,
        icon: Icons.add_road,
        onPressed: onSetRegion,
      ),
    ],
  );
}
