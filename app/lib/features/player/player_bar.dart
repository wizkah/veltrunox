import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import 'player_analysis.dart';
import 'player_controller.dart';
import 'player_state.dart';
import 'player_texts.dart';
import 'position_display.dart';
import 'settings_controls.dart';
import 'transport_controls.dart';

/// The shared player: what is playing, transport and settings.
class PlayerBar extends ConsumerWidget {
  const PlayerBar({this.analysis, super.key});

  /// Analysis of the loaded recording, if it has one.
  final PlayerAnalysis? analysis;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    ref.listen(playerRejectionProvider, (_, rejection) {
      final reason = rejection.value;
      if (reason != null) {
        ScaffoldMessenger.of(context)
            .showSnackBar(SnackBar(content: Text(reason)));
      }
    });
    final state = ref.watch(playerStateProvider).value;
    return Material(
      elevation: 8,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: state?.recordingId == null
            ? const Text(PlayerTexts.nothingSelected)
            : _LoadedPlayer(state: state!, analysis: analysis),
      ),
    );
  }
}

class _LoadedPlayer extends ConsumerWidget {
  const _LoadedPlayer({required this.state, required this.analysis});

  final PlayerState state;
  final PlayerAnalysis? analysis;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final controller = ref.watch(playerControllerProvider);
    final position = ref.watch(playerPositionProvider).value ?? 0;
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        PositionDisplay(positionMs: position),
        TransportControls(
          running: state.running,
          onTogglePlay: () => controller.togglePlay(running: state.running),
          onRestart: controller.restart,
        ),
        SettingsControls(
          state: state,
          analysis: analysis,
          onSpeed: controller.setSpeed,
          onTuning: controller.setTuning,
          onKey: controller.setKey,
          onTargetTempo: (bpm) => controller.setTargetTempo(
            targetBpm: bpm,
            recordingBpm: analysis?.tempoBpm ?? bpm,
          ),
        ),
      ],
    );
  }
}
