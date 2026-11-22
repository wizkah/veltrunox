/// Formats a playback position as minutes, seconds and tenths.
String formatPosition(int positionMs) {
  final tenths = positionMs ~/ 100 % 10;
  final seconds = positionMs ~/ 1000 % 60;
  final minutes = positionMs ~/ 60000;
  return '$minutes:${seconds.toString().padLeft(2, '0')}.$tenths';
}

/// How long to wait until the displayed tenth changes, at the given speed. A
/// little later than the change, so the next reading is already past it.
Duration untilNextTenth(int positionMs, double speed) {
  final remaining = 100 - positionMs % 100;
  final milliseconds = remaining / (speed > 0 ? speed : 1);
  return Duration(microseconds: (milliseconds * 1000).ceil() + 3000);
}
