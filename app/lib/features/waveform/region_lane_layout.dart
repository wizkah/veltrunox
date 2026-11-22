import '../markers/marker.dart';

/// The lane of every region and the number of lanes in use.
final class LaneAssignment {
  const LaneAssignment({required this.laneById, required this.laneCount});

  static const empty = LaneAssignment(laneById: {}, laneCount: 0);

  final Map<int, int> laneById;
  final int laneCount;

  int? laneOf(int regionId) => laneById[regionId];

  /// The lanes to show when at most [maxLanes] fit: the first ones, with the
  /// last slot given to the selected region's lane if it lies beyond them.
  List<int> visibleLanes(int maxLanes, {int? selectedId}) {
    final shown = laneCount < maxLanes ? laneCount : maxLanes;
    final lanes = [for (var lane = 0; lane < shown; lane++) lane];
    final selectedLane = selectedId == null ? null : laneById[selectedId];
    if (selectedLane != null && selectedLane >= shown && shown > 0) {
      lanes[shown - 1] = selectedLane;
    }
    return lanes;
  }
}

/// Interval partitioning for regions as half-open intervals `[start, end)`:
/// regions that overlap never share a lane, regions that only touch may.
///
/// Deterministic for the same input, and as compact as the order allows: a
/// region takes the lowest lane that is free for it.
LaneAssignment layoutRegionLanes(List<RegionMarker> regions) {
  final ordered = [...regions]..sort(_byStartEndId);
  final lanes = <List<RegionMarker>>[];
  final laneById = <int, int>{};

  for (final region in ordered) {
    final lane = _lowestFreeLane(lanes, region);
    if (lane == lanes.length) lanes.add([]);
    lanes[lane].add(region);
    laneById[region.id] = lane;
  }

  return LaneAssignment(laneById: laneById, laneCount: lanes.length);
}

int _byStartEndId(RegionMarker a, RegionMarker b) {
  final byStart = a.startMs.compareTo(b.startMs);
  if (byStart != 0) return byStart;
  final byEnd = a.endMs.compareTo(b.endMs);
  return byEnd != 0 ? byEnd : a.id.compareTo(b.id);
}

bool _overlaps(RegionMarker a, RegionMarker b) =>
    a.startMs < b.endMs && b.startMs < a.endMs;

bool _fits(List<List<RegionMarker>> lanes, int lane, RegionMarker region) =>
    lane >= lanes.length ||
    !lanes[lane].any((other) => _overlaps(other, region));

int _lowestFreeLane(List<List<RegionMarker>> lanes, RegionMarker region) {
  var lane = 0;
  while (!_fits(lanes, lane, region)) {
    lane++;
  }
  return lane;
}
