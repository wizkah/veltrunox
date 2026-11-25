import 'package:flutter/gestures.dart';
import 'package:material_ui/material_ui.dart';

import 'two_finger_recognizer.dart';

/// Zooms and pans with two fingers or a trackpad pinch anywhere on the
/// timeline. It reads the pointers themselves, so it also works when a layer
/// below (playhead, region, ruler) has already taken the first finger.
class PinchZoomLayer extends StatefulWidget {
  const PinchZoomLayer({
    required this.onZoom,
    required this.onPan,
    required this.onMultiTouch,
    required this.child,
    super.key,
  });

  /// Zoom by a factor (above 1 zooms in) around a pixel position.
  final void Function(double factor, double anchorX) onZoom;

  /// The content moved sideways by this many pixels.
  final ValueChanged<double> onPan;

  /// `true` when a second finger lands, `false` when the next touch begins.
  /// One-finger intents of the same touch are to be ignored meanwhile.
  final ValueChanged<bool> onMultiTouch;
  final Widget child;

  @override
  State<PinchZoomLayer> createState() => _PinchZoomLayerState();
}

class _PinchZoomLayerState extends State<PinchZoomLayer> {
  final Map<int, Offset> _fingers = {};
  double _trackpadScale = 1;

  List<Offset> get _pair => _fingers.values.take(2).toList();
  double get _span => (_pair[0] - _pair[1]).distance;
  double get _focalX => (_pair[0].dx + _pair[1].dx) / 2;

  void _down(PointerDownEvent event) {
    if (event.kind != PointerDeviceKind.touch) return;
    if (_fingers.isEmpty) widget.onMultiTouch(false);
    _fingers[event.pointer] = event.localPosition;
    if (_fingers.length >= 2) widget.onMultiTouch(true);
  }

  void _move(PointerMoveEvent event) {
    if (!_fingers.containsKey(event.pointer)) return;
    if (_fingers.length < 2) {
      _fingers[event.pointer] = event.localPosition;
      return;
    }
    final span = _span;
    final focal = _focalX;
    _fingers[event.pointer] = event.localPosition;
    if (span > 0 && _span > 0) widget.onZoom(_span / span, focal);
    widget.onPan(_focalX - focal);
  }

  void _lift(PointerEvent event) => _fingers.remove(event.pointer);

  /// A pinch the browser reports as a scale signal.
  void _signalled(PointerSignalEvent event) {
    if (event is! PointerScaleEvent) return;
    GestureBinding.instance.pointerSignalResolver.register(
      event,
      (_) => widget.onZoom(event.scale, event.localPosition.dx),
    );
  }

  /// A trackpad pinch of a native desktop; its scale counts from the start.
  void _trackpad(PointerPanZoomUpdateEvent event) {
    if (event.scale <= 0) return;
    widget.onZoom(event.scale / _trackpadScale, event.localPosition.dx);
    _trackpadScale = event.scale;
  }

  @override
  Widget build(BuildContext context) => Listener(
    onPointerDown: _down,
    onPointerMove: _move,
    onPointerUp: _lift,
    onPointerCancel: _lift,
    onPointerSignal: _signalled,
    onPointerPanZoomStart: (_) => _trackpadScale = 1,
    onPointerPanZoomUpdate: _trackpad,
    child: RawGestureDetector(
      gestures: {
        TwoFingerRecognizer:
            GestureRecognizerFactoryWithHandlers<TwoFingerRecognizer>(
              TwoFingerRecognizer.new,
              (_) {},
            ),
      },
      child: widget.child,
    ),
  );
}
