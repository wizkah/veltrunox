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
/// Deterministic for the same input. With a [previous] assignment, regions
/// keep their lane whenever that causes no overlap, so the layout does not
/// jump while a region is edited.
LaneAssignment layoutRegionLanes(
  List<RegionMarker> regions, {
  LaneAssignment previous = LaneAssignment.empty,
}) {
  final ordered = [...regions]..sort(_byStartEndId);
  final lanes = <List<RegionMarker>>[];
  final laneById = <int, int>{};

  void place(RegionMarker region, int lane) {
    while (lanes.length <= lane) {
      lanes.add([]);
    }
    lanes[lane].add(region);
    laneById[region.id] = lane;
  }

  final unplaced = <RegionMarker>[];
  for (final region in ordered) {
    final wanted = previous.laneOf(region.id);
    if (wanted != null && _fits(lanes, wanted, region)) {
      place(region, wanted);
    } else {
      unplaced.add(region);
    }
  }
  for (final region in unplaced) {
    place(region, _lowestFreeLane(lanes, region));
  }

  return LaneAssignment(laneById: laneById, laneCount: _usedLaneCount(lanes));
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

/// Lanes can stay empty when a kept lane is far up; count up to the last used.
int _usedLaneCount(List<List<RegionMarker>> lanes) {
  var count = lanes.length;
  while (count > 0 && lanes[count - 1].isEmpty) {
    count--;
  }
  return count;
}
