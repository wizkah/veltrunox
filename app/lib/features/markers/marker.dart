/// A marker of a recording: a label (one point in time) or a region (a span).
sealed class Marker {
  const Marker({required this.id, required this.version, required this.title});

  final int id;

  /// The version this client saw; a change or deletion names it.
  final int version;
  final String title;

  /// Where the marker begins: the point of a label, the start of a region.
  int get startMs;
}

final class LabelMarker extends Marker {
  const LabelMarker({
    required super.id,
    required super.version,
    required super.title,
    required this.positionMs,
  });

  final int positionMs;

  @override
  int get startMs => positionMs;
}

final class RegionMarker extends Marker {
  const RegionMarker({
    required super.id,
    required super.version,
    required super.title,
    required this.startMs,
    required this.endMs,
  });

  @override
  final int startMs;
  final int endMs;
}

/// A span of time, as the shared player reports the region being played.
typedef TimeSpan = ({int startMs, int endMs});

extension RegionActivity on RegionMarker {
  /// Whether this region is the one being played: the player knows the range,
  /// not the region, so a region counts as active when its times match.
  bool isActiveIn(TimeSpan? playRange) =>
      playRange != null &&
      playRange.startMs == startMs &&
      playRange.endMs == endMs;
}

/// The one region that covers all of [regions]: from the earliest start to
/// the latest end. Gaps between them are covered too.
TimeSpan spanOfRegions(Iterable<RegionMarker> regions) => (
  startMs: regions.map((r) => r.startMs).reduce((a, b) => a < b ? a : b),
  endMs: regions.map((r) => r.endMs).reduce((a, b) => a > b ? a : b),
);
