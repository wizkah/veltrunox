import 'package:material_ui/material_ui.dart';

import '../../shared_ui/app_icon_button.dart';
import 'waveform_texts.dart';

/// Local view controls: zoom, whole recording, snapping, drawing a region.
class WaveformToolbar extends StatelessWidget {
  const WaveformToolbar({
    required this.snapEnabled,
    required this.drawingRegion,
    required this.onZoomIn,
    required this.onZoomOut,
    required this.onFit,
    required this.onToggleSnap,
    required this.onToggleDrawRegion,
    super.key,
  });

  final bool snapEnabled;
  final bool drawingRegion;
  final VoidCallback onZoomIn;
  final VoidCallback onZoomOut;
  final VoidCallback onFit;
  final VoidCallback onToggleSnap;
  final VoidCallback onToggleDrawRegion;

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
        tooltip: WaveformTexts.newRegion,
        icon: Icons.add_road,
        selected: drawingRegion,
        onPressed: onToggleDrawRegion,
      ),
    ],
  );
}
