import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import 'snap_targets.dart';
import 'time_viewport.dart';

/// Turns touches on the free waveform into view and position intents: a tap
/// sets the position, a one-finger drag pans, two fingers or the mouse wheel
/// zoom. In "draw region" mode a drag draws a new interval instead. Handles,
/// the playhead, labels and regions sit above and take their own touches.
class WaveformGestureLayer extends StatefulWidget {
  const WaveformGestureLayer({
    required this.viewport,
    required this.drawingRegion,
    required this.snapTargets,
    required this.onTapAt,
    required this.onPan,
    required this.onZoom,
    required this.onRegionDrawn,
    required this.child,
    super.key,
  });

  final TimeViewport viewport;
  final bool drawingRegion;

  /// `null` while snapping is off.
  final SnapTargets? snapTargets;

  /// A tap at this time: set the position, do not play.
  final ValueChanged<int> onTapAt;
  final ValueChanged<double> onPan;

  /// Zoom by a factor (above 1 zooms in) around a pixel position.
  final void Function(double factor, double anchorX) onZoom;
  final void Function(int startMs, int endMs) onRegionDrawn;
  final Widget child;

  @override
  State<WaveformGestureLayer> createState() => _WaveformGestureLayerState();
}

class _WaveformGestureLayerState extends State<WaveformGestureLayer> {
  /// Drawing a region: where it began and where the finger is, in pixels.
  ({double from, double to})? _drawing;
  double _lastScale = 1;

  /// A pixel position as time, snapped to a marker if snapping is on.
  int _timeAt(double x) {
    final ms = widget.viewport
        .xToTime(x)
        .clamp(0.0, widget.viewport.durationMs.toDouble());
    return widget.snapTargets?.nearest(ms, widget.viewport) ?? ms.round();
  }

  void _start(ScaleStartDetails details) {
    _lastScale = 1;
    if (widget.drawingRegion && details.pointerCount == 1) {
      final x = details.localFocalPoint.dx;
      setState(() => _drawing = (from: x, to: x));
    }
  }

  void _update(ScaleUpdateDetails details) {
    if (details.pointerCount >= 2) {
      _drawing = null;
      widget.onZoom(details.scale / _lastScale, details.localFocalPoint.dx);
      _lastScale = details.scale;
    } else if (_drawing case final drawing?) {
      setState(
        () => _drawing = (from: drawing.from, to: details.localFocalPoint.dx),
      );
    } else {
      widget.onPan(details.focalPointDelta.dx);
    }
  }

  void _end(ScaleEndDetails details) {
    final drawing = _drawing;
    if (drawing == null) return;
    setState(() => _drawing = null);
    if ((drawing.to - drawing.from).abs() < 4) return;
    final a = _timeAt(drawing.from);
    final b = _timeAt(drawing.to);
    widget.onRegionDrawn(math.min(a, b), math.max(a, b));
  }

  void _scrolled(PointerSignalEvent event) {
    if (event is! PointerScrollEvent) return;
    widget.onZoom(
      math.pow(1.002, -event.scrollDelta.dy).toDouble(),
      event.localPosition.dx,
    );
  }

  @override
  Widget build(BuildContext context) => Listener(
    onPointerSignal: _scrolled,
    child: GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTapUp: (details) => widget.onTapAt(
        widget.viewport.xToTime(details.localPosition.dx).round(),
      ),
      onScaleStart: _start,
      onScaleUpdate: _update,
      onScaleEnd: _end,
      child: CustomPaint(
        foregroundPainter: _DrawingPainter(_drawing),
        child: widget.child,
      ),
    ),
  );
}

/// The two edges of the interval being drawn: lines only, nothing that covers
/// the waveform.
class _DrawingPainter extends CustomPainter {
  _DrawingPainter(this.drawing);

  final ({double from, double to})? drawing;

  @override
  void paint(Canvas canvas, Size size) {
    final current = drawing;
    if (current == null) return;
    final paint = Paint()
      ..color = AppColors.action
      ..strokeWidth = 2;
    for (final x in [current.from, current.to]) {
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), paint);
    }
    final y = size.height - 2;
    canvas.drawLine(Offset(current.from, y), Offset(current.to, y), paint);
  }

  @override
  bool shouldRepaint(_DrawingPainter old) => old.drawing != drawing;
}
