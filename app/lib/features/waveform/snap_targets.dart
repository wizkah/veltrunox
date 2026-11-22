import '../markers/marker.dart';
import 'time_viewport.dart';

/// Where a drag may snap to: label positions and region starts and ends.
final class SnapTargets {
  SnapTargets._(this._sortedMs);

  /// Targets of all [markers]; the marker with [excludeId] (the one being
  /// dragged) does not attract itself.
  factory SnapTargets.from(List<Marker> markers, {int? excludeId}) {
    final times = <int>{};
    for (final marker in markers) {
      if (marker.id == excludeId) continue;
      switch (marker) {
        case LabelMarker(:final positionMs):
          times.add(positionMs);
        case RegionMarker(:final startMs, :final endMs):
          times
            ..add(startMs)
            ..add(endMs);
      }
    }
    return SnapTargets._(times.toList()..sort());
  }

  final List<int> _sortedMs;

  /// The nearest target within [thresholdPx] of [ms] at the zoom of
  /// [viewport], or null when snapping is off or nothing is close.
  int? nearest(double ms, TimeViewport viewport, {double thresholdPx = 12}) {
    final limitMs = thresholdPx * viewport.msPerPx;
    int? best;
    var bestDistance = double.infinity;
    for (final target in _sortedMs) {
      final distance = (target - ms).abs();
      if (distance <= limitMs && distance < bestDistance) {
        best = target;
        bestDistance = distance;
      }
    }
    return best;
  }
}
