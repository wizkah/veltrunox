import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import 'time_viewport.dart';
import 'waveform_geometry.dart';
import 'waveform_texts.dart';

/// The shared playback position, drawn through all layers ([height]). Only the
/// part from [hitTop] for [hitHeight] takes touches, so labels and regions
/// stay tappable. Dragging it scrubs: it reports positions along the way and
/// the last one when it is let go. A drag that starts beside it belongs to the
/// waveform behind (panning).
class Playhead extends StatefulWidget {
  const Playhead({
    required this.positionMs,
    required this.viewport,
    required this.height,
    required this.hitTop,
    required this.hitHeight,
    required this.onScrub,
    required this.onScrubEnd,
    super.key,
  });

  final int positionMs;
  final TimeViewport viewport;
  final double height;
  final double hitTop;
  final double hitHeight;
  final ValueChanged<int> onScrub;
  final ValueChanged<int> onScrubEnd;

  @override
  State<Playhead> createState() => _PlayheadState();
}

class _PlayheadState extends State<Playhead> {
  double? _dragMs;

  int _clamped(double ms) => ms.round().clamp(0, widget.viewport.durationMs);

  @override
  Widget build(BuildContext context) {
    final viewport = widget.viewport;
    final shownMs = _dragMs ?? widget.positionMs.toDouble();
    final x = viewport.timeToX(shownMs);
    if (x < -WaveformGeometry.playheadHitWidth ||
        x > viewport.widthPx + WaveformGeometry.playheadHitWidth) {
      return const SizedBox.shrink();
    }
    final dragging = _dragMs != null;
    final hitWidth = WaveformGeometry.playheadHitWidth;
    return Positioned(
      left: x - hitWidth / 2,
      top: 0,
      width: hitWidth,
      height: widget.height,
      child: Stack(
        children: [
          Positioned.fill(
            child: IgnorePointer(
              child: Center(
                child: Container(
                  width: dragging ? 4 : 2,
                  color: AppColors.action,
                ),
              ),
            ),
          ),
          Positioned(
            top: widget.hitTop,
            height: widget.hitHeight,
            left: 0,
            right: 0,
            child: Semantics(
              label: WaveformTexts.playhead,
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onHorizontalDragStart: (_) =>
                    setState(() => _dragMs = widget.positionMs.toDouble()),
                onHorizontalDragUpdate: _update,
                onHorizontalDragEnd: (_) => _end(),
                onHorizontalDragCancel: _end,
              ),
            ),
          ),
        ],
      ),
    );
  }

  void _update(DragUpdateDetails details) {
    final viewport = widget.viewport;
    final next =
        ((_dragMs ?? widget.positionMs.toDouble()) +
                details.delta.dx * viewport.msPerPx)
            .clamp(0.0, viewport.durationMs.toDouble());
    setState(() => _dragMs = next);
    widget.onScrub(_clamped(next));
  }

  void _end() {
    final last = _clamped(_dragMs ?? widget.positionMs.toDouble());
    setState(() => _dragMs = null);
    widget.onScrubEnd(last);
  }
}
