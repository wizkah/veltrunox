import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/panel_surface.dart';
import 'player_analysis.dart';
import 'player_controller.dart';
import 'player_state.dart';
import 'position_slider.dart';
import 'settings_controls.dart';
import 'transport_clock.dart';
import 'transport_controls.dart';
import 'volume_control.dart';

/// Transport, position, volume and the tempo and pitch panels of the loaded
/// recording; the shared state is passed in, actions go to the controller.
/// Given a bounded height on a wide screen, it fills it: [above] takes what the
/// transport and the panels leave. Otherwise it is as high as its content.
class PlayerControls extends ConsumerWidget {
  const PlayerControls({
    required this.state,
    required this.analysis,
    this.lengthMs,
    this.connected = true,
    this.above,
    this.showPositionSlider = true,
    super.key,
  });

  final PlayerState state;
  final PlayerAnalysis? analysis;

  /// Length of the recording; `null` until the server knows it.
  final int? lengthMs;

  /// Without a connection to the server nothing is processed (D-22): the
  /// controls keep the shared state, cannot be used, and what was changed
  /// locally and not sent yet is dropped.
  final bool connected;

  /// Shown above the transport, in the same column (the waveform, and on a
  /// narrow screen the way to the markers). On a wide screen the volume stands
  /// beside it, the transport and the panels, over their whole height.
  final Widget? above;

  /// Hidden where another control moves the position (the waveform).
  final bool showPositionSlider;

  /// From this width the volume stands beside the transport and the panels
  /// beside each other.
  static const _wideMin = 900.0;

  /// Width of the upright volume panel on a wide screen.
  static const _volumeWidth = 128.0;

  /// The least height the upright volume panel needs for its content.
  static const _volumeMinHeight = 280.0;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final controller = ref.watch(playerControllerProvider);
    final position = ref.watch(playerPositionProvider).value ?? 0;
    final length = lengthMs;
    final transport = Column(
      mainAxisSize: MainAxisSize.min,
      children: [
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
        // The clock goes below the buttons where both do not fit in a row.
        Wrap(
          alignment: WrapAlignment.center,
          crossAxisAlignment: WrapCrossAlignment.center,
          spacing: AppSpace.s6,
          runSpacing: AppSpace.s2,
          children: [
            TransportControls(
              running: state.running,
              playMode: state.playMode,
              onTogglePlay: () => controller.togglePlay(running: state.running),
              onRestart: controller.restart,
              onPlayMode: () => controller.setPlayMode(state.playMode.next),
            ),
            TransportClock(
              positionMs: length == null ? position : position.clamp(0, length),
              lengthMs: length,
            ),
          ],
        ),
      ],
    );
    Widget main({required bool fill}) => Column(
      mainAxisSize: fill ? MainAxisSize.max : MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (above != null) ...[
          if (fill) Expanded(child: above!) else above!,
          const SizedBox(height: AppSpace.s3),
        ],
        transport,
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
    return _Offline(
      connected: connected,
      child: LayoutBuilder(
        builder: (context, constraints) {
          final wide = constraints.maxWidth >= _wideMin;
          if (wide) {
            final fill = constraints.hasBoundedHeight;
            // The volume panel is laid over the right edge and stretched down to
            // the panels; the waveform inside `main` rules out measuring that
            // height beforehand.
            final top = ConstrainedBox(
              constraints: const BoxConstraints(minHeight: _volumeMinHeight),
              child: Stack(
                fit: fill ? StackFit.expand : StackFit.loose,
                children: [
                  // Waveform and transport share one panel, the stage of
                  // the player.
                  Padding(
                    padding: const EdgeInsets.only(
                      right: _volumeWidth + AppSpace.s4,
                    ),
                    child: PanelSurface(child: main(fill: fill)),
                  ),
                  Positioned(
                    top: 0,
                    bottom: 0,
                    right: 0,
                    width: _volumeWidth,
                    child: volume(vertical: true),
                  ),
                ],
              ),
            );
            return Column(
              mainAxisSize: fill ? MainAxisSize.max : MainAxisSize.min,
              children: [
                if (fill) Expanded(child: top) else top,
                const SizedBox(height: AppSpace.s3),
                settings(sideBySide: true),
              ],
            );
          }
          final stacked = Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              main(fill: false),
              const SizedBox(height: AppSpace.s3),
              volume(vertical: false),
              const SizedBox(height: AppSpace.s3),
              settings(sideBySide: false),
            ],
          );
          return constraints.hasBoundedHeight
              ? SingleChildScrollView(child: stacked)
              : stacked;
        },
      ),
    );
  }
}

/// Shows its child as it is while [connected]. Otherwise the child is dimmed
/// and cannot be used, and it is built anew, so local entries (a dragged
/// handle, typed text) give way to the shared state.
class _Offline extends StatelessWidget {
  const _Offline({required this.connected, required this.child});

  final bool connected;
  final Widget child;

  @override
  Widget build(BuildContext context) => KeyedSubtree(
    key: ValueKey(connected),
    child: connected
        ? child
        : ExcludeFocus(
            child: IgnorePointer(child: Opacity(opacity: 0.5, child: child)),
          ),
  );
}
