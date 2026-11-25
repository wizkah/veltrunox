import 'package:flutter/gestures.dart';

/// Claims the touch as soon as a second finger lands, so a scrolling parent or
/// a one-finger gesture that has not started yet does not take it. One finger
/// never triggers it.
class TwoFingerRecognizer extends OneSequenceGestureRecognizer {
  TwoFingerRecognizer({super.debugOwner});

  int _fingers = 0;

  @override
  void addAllowedPointer(PointerDownEvent event) {
    startTrackingPointer(event.pointer, event.transform);
    _fingers++;
    if (_fingers >= 2) resolve(GestureDisposition.accepted);
  }

  @override
  void handleEvent(PointerEvent event) {
    if (event is PointerUpEvent || event is PointerCancelEvent) {
      stopTrackingPointer(event.pointer);
    }
  }

  @override
  void acceptGesture(int pointer) {}

  // The pointer stays tracked: it still counts as a finger on the screen.
  @override
  void rejectGesture(int pointer) {}

  @override
  void didStopTrackingLastPointer(int pointer) => _fingers = 0;

  @override
  String get debugDescription => 'two fingers';
}
