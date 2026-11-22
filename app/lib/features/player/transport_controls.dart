import 'package:material_ui/material_ui.dart';

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
        extent: 64,
        iconSize: 40,
      ),
      AppIconButton(
        tooltip: running ? PlayerTexts.pause : PlayerTexts.play,
        onPressed: onTogglePlay,
        icon: running ? Icons.pause : Icons.play_arrow,
        prominent: true,
        extent: 88,
        iconSize: 56,
      ),
    ],
  );
}
