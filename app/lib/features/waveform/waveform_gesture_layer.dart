import 'package:flutter/gestures.dart';
import 'package:material_ui/material_ui.dart';

import 'time_viewport.dart';

/// Moving less than this still counts as a tap: a mouse or finger rarely stays
/// perfectly still during a click.
const _tapTravelPx = 10.0;

/// Turns touches on the free waveform into view and position intents: a tap
/// sets the position, one finger pans, and on a trackpad a sideways swipe
/// pans. Pinching is handled around the whole timeline. Handles, the playhead,
/// labels and regions sit above and take their own touches.
class WaveformGestureLayer extends StatefulWidget {
  const WaveformGestureLayer({
    required this.viewport,
    required this.onTapAt,
    required this.onPan,
    required this.child,
    super.key,
  });

  final TimeViewport viewport;

  /// A tap at this time: set the position, do not play.
  final ValueChanged<int> onTapAt;

  /// The content moved sideways by this many pixels.
  final ValueChanged<double> onPan;
  final Widget child;

  @override
  State<WaveformGestureLayer> createState() => _WaveformGestureLayerState();
}

class _WaveformGestureLayerState extends State<WaveformGestureLayer> {
  double _travel = 0;

  /// Where the touch began; a drag is recognised only after it moved a little,
  /// so the recogniser's own start point is already past it.
  Offset _downPoint = Offset.zero;

  void _start(ScaleStartDetails details) => _travel = 0;

  void _update(ScaleUpdateDetails details) {
    _travel += details.focalPointDelta.distance;
    if (_travel >= _tapTravelPx) widget.onPan(details.focalPointDelta.dx);
  }

  void _end(ScaleEndDetails details) {
    if (_travel < _tapTravelPx) widget.onTapAt(_ms(_downPoint.dx));
  }

  int _ms(double x) => widget.viewport.xToTime(x).round();

  /// A trackpad or wheel: a mostly sideways swipe pans. Mostly vertical
  /// scrolling is left to the page.
  void _signalled(PointerSignalEvent event) {
    if (event is PointerScrollEvent &&
        event.scrollDelta.dx.abs() > event.scrollDelta.dy.abs()) {
      GestureBinding.instance.pointerSignalResolver.register(
        event,
        (_) => widget.onPan(-event.scrollDelta.dx),
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
      },
      child: widget.child,
    ),
  );
}
