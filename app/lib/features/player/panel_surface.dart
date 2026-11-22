import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/frame_decoration.dart';

/// The bordered, transparent surface the player's panels (tempo, pitch, volume) stand on.
class PanelSurface extends StatelessWidget {
  const PanelSurface({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) => DecoratedBox(
    decoration: frameDecoration(),
    child: Padding(padding: const EdgeInsets.all(AppSpace.s4), child: child),
  );
}
