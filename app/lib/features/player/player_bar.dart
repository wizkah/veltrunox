import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/toast_notice.dart';
import 'player_analysis.dart';
import 'player_controller.dart';
import 'player_state.dart';
import 'player_texts.dart';
import 'position_display.dart';
import 'position_slider.dart';
import 'settings_controls.dart';
import 'transport_controls.dart';
import 'volume_control.dart';

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
        ToastNotice.show(context, kind: ToastKind.error, message: reason);
      }
    });
    final state = ref.watch(playerStateProvider).value;
    return Material(
      color: AppColors.canvas,
      shape: const Border(
        top: BorderSide(color: AppColors.border, width: AppSize.borderWidth),
      ),
      child: SizedBox(
        width: double.infinity,
        child: Padding(
          padding: const EdgeInsets.all(AppSpace.s4),
          child: state?.recordingId == null
              ? const Text(PlayerTexts.nothingSelected)
              : ConstrainedBox(
                  // On a small screen the controls scroll instead of overflowing.
                  constraints: BoxConstraints(
                    maxHeight: MediaQuery.sizeOf(context).height * 0.6,
                  ),
                  child: SingleChildScrollView(
                    child: _LoadedPlayer(state: state!, analysis: analysis),
                  ),
                ),
        ),
      ),
    );
  }
}

class _LoadedPlayer extends ConsumerWidget {
  const _LoadedPlayer({required this.state, required this.analysis});

  final PlayerState state;
  final PlayerAnalysis? analysis;

  /// From this width the volume stands beside the transport and the panels
  /// beside each other.
  static const _wideMin = 1000.0;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final controller = ref.watch(playerControllerProvider);
    final position = ref.watch(playerPositionProvider).value ?? 0;
    final length = ref
        .watch(
          playerLengthProvider((
            recordingId: state.recordingId!,
            scanned: analysis != null,
          )),
        )
        .value;
    final transport = Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        PositionDisplay(positionMs: position),
        PositionSlider(
          positionMs: position.clamp(0, length ?? 0),
          lengthMs: length,
          onScrub: controller.scrub,
          onScrubEnd: controller.endScrub,
          onHold: (position) =>
              controller.hold(position, running: state.running),
          onHoldEnd: controller.endHold,
        ),
        TransportControls(
          running: state.running,
          onTogglePlay: () => controller.togglePlay(running: state.running),
          onRestart: controller.restart,
        ),
      ],
    );
    Widget volume({required bool vertical}) => VolumeControl(
      percent: state.volumePercent,
      onChanged: controller.setVolume,
      vertical: vertical,
    );
    Widget settings({required bool sideBySide}) => SettingsControls(
      state: state,
      analysis: analysis,
      sideBySide: sideBySide,
      onSpeed: controller.setSpeed,
      onTuning: controller.setTuning,
      onKey: controller.setKey,
      onTargetTempo: (bpm) => controller.setTargetTempo(
        targetBpm: bpm,
        recordingBpm: analysis?.tempoBpm ?? bpm,
      ),
    );
    return LayoutBuilder(
      builder: (context, constraints) {
        final wide = constraints.maxWidth >= _wideMin;
        return Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (wide)
              Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Expanded(child: transport),
                  const SizedBox(width: AppSpace.s4),
                  volume(vertical: true),
                ],
              )
            else ...[
              transport,
              const SizedBox(height: AppSpace.s3),
              volume(vertical: false),
            ],
            const SizedBox(height: AppSpace.s3),
            settings(sideBySide: wide),
          ],
        );
      },
    );
  }
}
