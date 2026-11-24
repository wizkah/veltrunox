import 'package:material_ui/material_ui.dart';

import '../design/tokens/app_geometry.dart';
import 'frame_decoration.dart';

/// The framed surface panels stand on (tempo, pitch, volume, markers).
class PanelSurface extends StatelessWidget {
  const PanelSurface({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) => DecoratedBox(
    decoration: frameDecoration(),
    child: Padding(padding: const EdgeInsets.all(AppSpace.s4), child: child),
  );
}
