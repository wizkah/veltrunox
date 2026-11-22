import 'dart:math' as math;

/// The shortest region a button creates.
const _minSpanMs = 1000;

/// The span of a region created from the playhead: it starts there and ends at
/// the end of the visible range, kept inside the recording and at least a
/// second long, so it can be adjusted with its handles right away.
({int startMs, int endMs}) defaultRegionRange({
  required int positionMs,
  required double visibleEndMs,
  required int lengthMs,
}) {
  final end = math.min(
    lengthMs,
    math.max(visibleEndMs.round(), positionMs + _minSpanMs),
  );
  final start = math.max(0, math.min(positionMs, end - _minSpanMs));
  return (startMs: start, endMs: end);
}
