import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/app_icon_button.dart';
import 'play_mode_toggle.dart';
import 'player_state.dart';
import 'player_texts.dart';

class TransportControls extends StatelessWidget {
  const TransportControls({
    required this.running,
    required this.playMode,
    required this.onTogglePlay,
    required this.onRestart,
    required this.onPlayMode,
    super.key,
  });

  final bool running;
  final PlayMode playMode;
  final VoidCallback onTogglePlay;
  final VoidCallback onRestart;

  /// The play mode toggle was tapped; it switches to the next mode.
  final VoidCallback onPlayMode;

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
      const SizedBox(width: AppSpace.s3),
      AppIconButton(
        tooltip: running ? PlayerTexts.pause : PlayerTexts.play,
        onPressed: onTogglePlay,
        icon: running ? Icons.pause : Icons.play_arrow,
        prominent: true,
        extent: AppSize.touchTarget * 1.5,
        iconSize: 72,
      ),
      const SizedBox(width: AppSpace.s3),
      PlayModeToggle(mode: playMode, onPressed: onPlayMode),
    ],
  );
}
