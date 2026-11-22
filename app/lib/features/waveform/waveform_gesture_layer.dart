import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import 'snap_targets.dart';
import 'time_viewport.dart';
import 'two_finger_recognizer.dart';

/// Moving less than this still counts as a tap: a mouse or finger rarely stays
/// perfectly still during a click.
const _tapTravelPx = 10.0;

/// Turns touches on the free waveform into view and position intents: a tap
/// sets the position, one finger pans, two fingers pinch (zoom) and pan, and
/// on a trackpad a sideways swipe pans and a pinch zooms. In "draw region" mode
/// a one-finger drag draws a new interval instead. Handles, the playhead,
/// labels and regions sit above and take their own touches.
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

  /// The content moved sideways by this many pixels.
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
  double _travel = 0;

  /// Where the touch began; a drag is recognised only after it moved a little,
  /// so the recogniser's own start point is already past it.
  Offset _downPoint = Offset.zero;

  /// A pixel position as time, snapped to a marker if snapping is on.
  int _timeAt(double x) {
    final ms = widget.viewport
        .xToTime(x)
        .clamp(0.0, widget.viewport.durationMs.toDouble());
    return widget.snapTargets?.nearest(ms, widget.viewport) ?? ms.round();
  }

  void _start(ScaleStartDetails details) {
    _travel = 0;
    if (widget.drawingRegion) {
      final x = _downPoint.dx;
      setState(() => _drawing = (from: x, to: details.localFocalPoint.dx));
    }
  }

  void _update(ScaleUpdateDetails details) {
    _travel += details.focalPointDelta.distance;
    if (_drawing case final drawing?) {
      setState(
        () => _drawing = (from: drawing.from, to: details.localFocalPoint.dx),
      );
    } else if (_travel >= _tapTravelPx) {
      widget.onPan(details.focalPointDelta.dx);
    }
  }

  void _end(ScaleEndDetails details) {
    final drawing = _drawing;
    if (drawing == null) {
      if (_travel < _tapTravelPx) widget.onTapAt(_ms(_downPoint.dx));
      return;
    }
    setState(() => _drawing = null);
    if ((drawing.to - drawing.from).abs() < 4) return;
    final a = _timeAt(drawing.from);
    final b = _timeAt(drawing.to);
    widget.onRegionDrawn(math.min(a, b), math.max(a, b));
  }

  int _ms(double x) => widget.viewport.xToTime(x).round();

  /// A trackpad or wheel: a mostly sideways swipe pans, a pinch (the browser
  /// reports it as a scale) zooms. Mostly vertical scrolling is left to the
  /// page.
  void _signalled(PointerSignalEvent event) {
    final resolver = GestureBinding.instance.pointerSignalResolver;
    if (event is PointerScrollEvent &&
        event.scrollDelta.dx.abs() > event.scrollDelta.dy.abs()) {
      resolver.register(event, (_) => widget.onPan(-event.scrollDelta.dx));
    } else if (event is PointerScaleEvent) {
      resolver.register(
        event,
        (_) => widget.onZoom(event.scale, event.localPosition.dx),
      );
    }
  }

  @override
  Widget build(BuildContext context) => Listener(
    onPointerDown: (event) => _downPoint = event.localPosition,
    onPointerSignal: _signalled,
    child: RawGestureDetector(
      behavior: HitTestBehavior.opaque,
      gestures: {
        TapGestureRecognizer:
            GestureRecognizerFactoryWithHandlers<TapGestureRecognizer>(
              TapGestureRecognizer.new,
              (tap) =>
                  tap.onTapUp = (details) =>
                      widget.onTapAt(_ms(details.localPosition.dx)),
            ),
        ScaleGestureRecognizer:
            GestureRecognizerFactoryWithHandlers<ScaleGestureRecognizer>(
              ScaleGestureRecognizer.new,
              (scale) => scale
                ..onStart = _start
                ..onUpdate = _update
                ..onEnd = _end,
            ),
        TwoFingerRecognizer:
            GestureRecognizerFactoryWithHandlers<TwoFingerRecognizer>(
              TwoFingerRecognizer.new,
              (twoFingers) => twoFingers.onUpdate = (factor, focalX, panDx) {
                widget.onZoom(factor, focalX);
                widget.onPan(panDx);
              },
            ),
      },
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
