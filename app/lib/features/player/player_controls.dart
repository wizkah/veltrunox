import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import 'player_analysis.dart';
import 'player_controller.dart';
import 'player_state.dart';
import 'position_display.dart';
import 'position_slider.dart';
import 'settings_controls.dart';
import 'transport_controls.dart';
import 'volume_control.dart';

/// Transport, position, volume and the tempo and pitch panels of the loaded
/// recording; the shared state is passed in, actions go to the controller.
class PlayerControls extends ConsumerWidget {
  const PlayerControls({
    required this.state,
    required this.analysis,
    this.beforeSettings,
    this.showPositionSlider = true,
    super.key,
  });

  final PlayerState state;
  final PlayerAnalysis? analysis;

  /// Shown between the transport and the settings, for what belongs close to
  /// the transport (for example the way to the markers on a narrow screen).
  final Widget? beforeSettings;

  /// Hidden where another control moves the position (the waveform).
  final bool showPositionSlider;

  /// From this width the volume stands beside the transport and the panels
  /// beside each other.
  static const _wideMin = 720.0;

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
        if (showPositionSlider)
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
          playMode: state.playMode,
          onTogglePlay: () => controller.togglePlay(running: state.running),
          onRestart: controller.restart,
          onPlayMode: () => controller.setPlayMode(state.playMode.next),
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
            if (beforeSettings != null) ...[
              const SizedBox(height: AppSpace.s3),
              beforeSettings!,
            ],
            const SizedBox(height: AppSpace.s3),
            settings(sideBySide: wide),
          ],
        );
      },
    );
  }
}
