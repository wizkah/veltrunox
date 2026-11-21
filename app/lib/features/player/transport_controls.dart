import 'package:material_ui/material_ui.dart';

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
    children: [
      IconButton(
        tooltip: PlayerTexts.restart,
        onPressed: onRestart,
        icon: const Icon(Icons.skip_previous),
      ),
      IconButton.filled(
        tooltip: running ? PlayerTexts.pause : PlayerTexts.play,
        onPressed: onTogglePlay,
        icon: Icon(running ? Icons.pause : Icons.play_arrow),
      ),
    ],
  );
}
