import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/app_icon_button.dart';
import 'player_texts.dart';

class TransportControls extends StatelessWidget {
  const TransportControls({
    required this.running,
    required this.onTogglePlay,
    required this.onRestart,
    super.key,
  });

  final bool running;
  final VoidCallback onTogglePlay;
  final VoidCallback onRestart;

  @override
  Widget build(BuildContext context) => Row(
    mainAxisSize: MainAxisSize.min,
    crossAxisAlignment: CrossAxisAlignment.center,
    children: [
      AppIconButton(
        tooltip: PlayerTexts.restart,
        onPressed: onRestart,
        icon: Icons.skip_previous,
        framed: true,
        extent: AppSize.touchTarget,
        iconSize: 44,
      ),
      const SizedBox(width: AppSpace.s6),
      AppIconButton(
        tooltip: running ? PlayerTexts.pause : PlayerTexts.play,
        onPressed: onTogglePlay,
        icon: running ? Icons.pause : Icons.play_arrow,
        prominent: true,
        extent: AppSize.touchTarget * 1.5,
        iconSize: 72,
      ),
    ],
  );
}
