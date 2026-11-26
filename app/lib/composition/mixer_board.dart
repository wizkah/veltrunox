import 'dart:math' as math;

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../design/tokens/app_geometry.dart';
import '../features/mixer/mixer_source.dart';
import '../features/mixer/mixer_texts.dart';
import '../features/mixer/stem_controls.dart';
import '../features/mixer/stem_models.dart';
import '../features/player/player_controller.dart';
import '../features/player/player_state.dart';
import '../features/player/scrub_hold.dart';
import '../features/waveform/pinch_zoom_layer.dart';
import '../features/waveform/playhead.dart';
import '../features/waveform/time_viewport.dart';
import '../features/waveform/timeline_ruler.dart';
import '../features/waveform/waveform_geometry.dart';
import '../features/waveform/waveform_gesture_layer.dart';
import '../features/waveform/waveform_renderer.dart';
import '../features/waveform/waveform_view_controller.dart';

/// The stem lanes of a recording on one time axis with one playhead: controls
/// on the left of each waveform. Zoom and visible range are the player's.
class MixerBoard extends ConsumerStatefulWidget {
  const MixerBoard({
    required this.recordingId,
    required this.durationMs,
    required this.stems,
    required this.state,
    required this.connected,
    super.key,
  });

  final int recordingId;
  final int durationMs;
  final List<Stem> stems;
  final PlayerState state;
  final bool connected;

  static const laneHeight = 72.0;

  @override
  ConsumerState<MixerBoard> createState() => _MixerBoardState();
}

class _MixerBoardState extends ConsumerState<MixerBoard> {
  static const _controlsWide = 260.0;
  static const _controlsNarrow = 168.0;

  /// Where the playhead is while the user moves it; `null` shows the shared
  /// position.
  double? _scrubMs;
  bool _multiTouch = false;

  late final ScrubHold _scrub = ScrubHold(
    onScrub: (position) => _player.scrub(position),
    onScrubEnd: (position) => _player.endScrub(position),
    onHold: (position) => _player.hold(
      position,
      running: ref.read(playerStateProvider).value?.running ?? false,
    ),
    onHoldEnd: (position) => _player.endHold(position),
  );

  PlayerController get _player => ref.read(playerControllerProvider);

  @override
  void dispose() {
    _scrub.dispose();
    super.dispose();
  }

  void _scrubTo(double ms) {
    if (!widget.connected) return;
    final bounded = ms.clamp(0.0, widget.durationMs.toDouble());
    setState(() => _scrubMs = bounded);
    _scrub.moved(bounded.round());
  }

  void _scrubEnd() {
    final last = _scrubMs;
    if (last == null) return;
    setState(() => _scrubMs = null);
    _scrub.released(last.round());
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, bounds) {
      final controlsWidth = bounds.maxWidth < AppBreakpoint.tabletMin
          ? _controlsNarrow
          : _controlsWide;
      final width = math.max(1.0, bounds.maxWidth - controlsWidth);
      final recordingId = widget.recordingId;
      final view = ref.watch(waveformViewProvider(recordingId));
      final controller = ref.read(waveformViewProvider(recordingId).notifier);
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) {
          controller.resize(durationMs: widget.durationMs, widthPx: width);
        }
      });
      final viewport =
          view.viewport ??
          TimeViewport.fit(durationMs: widget.durationMs, widthPx: width);
      final position = (ref.watch(playerPositionProvider).value ?? 0).clamp(
        0,
        widget.durationMs,
      );
      final shownMs = _scrubMs ?? position.toDouble();
      const top = WaveformGeometry.timeRowHeight + WaveformGeometry.rulerHeight;
      final lanesHeight = widget.stems.length * MixerBoard.laneHeight;

      final board = Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: controlsWidth,
            child: Column(
              children: [
                const SizedBox(height: top),
                for (final (index, stem) in widget.stems.indexed)
                  _controls(index, stem),
              ],
            ),
          ),
          SizedBox(
            width: width,
            height: top + lanesHeight,
            child: PinchZoomLayer(
              onZoom: controller.zoomAt,
              onPan: controller.panByPixels,
              onMultiTouch: (value) => _multiTouch = value,
              child: Stack(
                children: [
                  Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const SizedBox(height: WaveformGeometry.timeRowHeight),
                      TimelineRuler(
                        viewport: viewport,
                        onScrub: _scrubTo,
                        onScrubEnd: _scrubEnd,
                      ),
                      for (final (index, stem) in widget.stems.indexed)
                        _lane(index, stem, viewport, shownMs, controller),
                    ],
                  ),
                  Playhead(
                    shownMs: shownMs,
                    viewport: viewport,
                    height: top + lanesHeight,
                    hitTop: top,
                    hitHeight: lanesHeight,
                    dragging: _scrubMs != null,
                    onDragStart: () =>
                        setState(() => _scrubMs = position.toDouble()),
                    onDragUpdate: (dx) =>
                        _scrubTo((_scrubMs ?? shownMs) + dx * viewport.msPerPx),
                    onDragEnd: _scrubEnd,
                  ),
                  PlayheadTime(shownMs: shownMs, viewport: viewport),
                ],
              ),
            ),
          ),
        ],
      );
      // In a given height the lanes scroll.
      return bounds.hasBoundedHeight
          ? SingleChildScrollView(child: board)
          : board;
    },
  );

  Widget _controls(int index, Stem stem) {
    final channel = index < widget.state.stemChannels.length
        ? widget.state.stemChannels[index]
        : const StemChannel();
    return SizedBox(
      height: MixerBoard.laneHeight,
      child: Padding(
        padding: const EdgeInsets.only(right: AppSpace.s2),
        child: StemControls(
          name: stem.name,
          volumePercent: channel.volumePercent,
          muted: channel.muted,
          solo: channel.solo,
          audible: channel.audible,
          enabled: widget.connected && widget.state.stemsEnabled,
          onVolume: (percent) => _player.setStemVolume(index, percent),
          onMuted: (muted) => _player.setStemMuted(index, muted),
          onSolo: (solo) => _player.setStemSolo(index, solo),
        ),
      ),
    );
  }

  Widget _lane(
    int index,
    Stem stem,
    TimeViewport viewport,
    double shownMs,
    WaveformViewController controller,
  ) {
    final loaded = ref
        .watch(
          stemWaveformProvider((
            recordingId: widget.recordingId,
            stem: index,
            key: stem.key,
          )),
        )
        .value;
    final channel = index < widget.state.stemChannels.length
        ? widget.state.stemChannels[index]
        : const StemChannel();
    final silent = widget.state.stemsEnabled && !channel.audible;
    return Padding(
      padding: const EdgeInsets.only(bottom: AppSpace.s1),
      child: Opacity(
        opacity: silent ? 0.35 : 1,
        child: WaveformGestureLayer(
          viewport: viewport,
          onZoom: controller.zoomAt,
          onTapAt: (ms) {
            if (_multiTouch || !widget.connected) return;
            _player.seek(ms.clamp(0, widget.durationMs));
          },
          onPan: (dx) {
            if (!_multiTouch) controller.panByPixels(dx);
          },
          child: WaveformRenderer(
            data: loaded?.data,
            viewport: viewport,
            height: MixerBoard.laneHeight - AppSpace.s1,
            playedMs: shownMs,
            missingText: MixerTexts.loading,
          ),
        ),
      ),
    );
  }
}
