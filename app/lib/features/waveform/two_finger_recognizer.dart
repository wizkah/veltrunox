import 'package:flutter/gestures.dart';

/// Recognises two fingers on the screen: it claims the touch as soon as the
/// second finger lands, so a scrolling parent does not take it, and then
/// reports the pinch (change of distance) and the pan (movement of the middle).
/// One finger never triggers it.
class TwoFingerRecognizer extends OneSequenceGestureRecognizer {
  TwoFingerRecognizer({super.debugOwner});

  /// [factor] is the change of the distance since the last update (above 1
  /// when the fingers move apart), [focalX] the middle of the fingers and
  /// [panDx] how far the middle moved sideways.
  void Function(double factor, double focalX, double panDx)? onUpdate;

  final Map<int, Offset> _positions = {};
  bool _accepted = false;
  double _lastSpan = 0;
  Offset _lastFocal = Offset.zero;

  double get _span {
    final points = _positions.values.take(2).toList();
    return (points[0] - points[1]).distance;
  }

  Offset get _focal {
    final points = _positions.values.take(2).toList();
    return (points[0] + points[1]) / 2;
  }

  @override
  void addAllowedPointer(PointerDownEvent event) {
    startTrackingPointer(event.pointer, event.transform);
    _positions[event.pointer] = event.localPosition;
    if (_positions.length >= 2) {
      _lastSpan = _span;
      _lastFocal = _focal;
      resolve(GestureDisposition.accepted);
    }
  }

  @override
  void handleEvent(PointerEvent event) {
    if (event is PointerMoveEvent) {
      _positions[event.pointer] = event.localPosition;
      if (_accepted && _positions.length >= 2) _report();
    } else if (event is PointerUpEvent || event is PointerCancelEvent) {
      _positions.remove(event.pointer);
      stopTrackingPointer(event.pointer);
      if (_positions.length >= 2) {
        _lastSpan = _span;
        _lastFocal = _focal;
      }
    }
  }

  void _report() {
    final span = _span;
    final focal = _focal;
    final factor = _lastSpan > 0 && span > 0 ? span / _lastSpan : 1.0;
    onUpdate?.call(factor, focal.dx, focal.dx - _lastFocal.dx);
    _lastSpan = span;
    _lastFocal = focal;
  }

  @override
  void acceptGesture(int pointer) => _accepted = true;

  @override
  void rejectGesture(int pointer) {
    stopTrackingPointer(pointer);
    _positions.remove(pointer);
  }

  @override
  void didStopTrackingLastPointer(int pointer) {
    _accepted = false;
    _positions.clear();
  }

  @override
  String get debugDescription => 'two fingers';
}
