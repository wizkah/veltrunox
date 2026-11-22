import '../../shared_ui/clock_format.dart';
import 'time_viewport.dart';

/// A labelled mark on the time ruler.
final class RulerTick {
  const RulerTick({required this.ms, required this.label});

  final int ms;
  final String label;
}

const _stepsMs = [
  100, 200, 500, //
  1000, 2000, 5000, 10000, 15000, 30000, //
  60000, 120000, 300000, 600000, 1800000, 3600000,
];

/// The marks of the ruler for [viewport]: the finest regular step that leaves
/// at least [minSpacingPx] between two labels.
List<RulerTick> computeRulerTicks(
  TimeViewport viewport, {
  double minSpacingPx = 72,
}) {
  final step = _stepsMs.firstWhere(
    (candidate) => candidate / viewport.msPerPx >= minSpacingPx,
    orElse: () => _stepsMs.last,
  );
  final first = (viewport.startMs / step).ceil() * step;
  return [
    for (var ms = first; ms <= viewport.endMs; ms += step)
      RulerTick(
        ms: ms,
        label: step < 1000 ? formatClockExact(ms) : formatClock(ms),
      ),
  ];
}
