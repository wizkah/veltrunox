import 'marker.dart';

/// Which markers the list shows.
enum MarkerFilter { all, labels, regions }

/// The markers the [filter] lets through, in time order. Markers at the same
/// time stay in a fixed order: labels before regions, then by id.
List<Marker> orderMarkers(Iterable<Marker> markers, MarkerFilter filter) {
  final shown = markers.where(
    (marker) => switch (filter) {
      MarkerFilter.all => true,
      MarkerFilter.labels => marker is LabelMarker,
      MarkerFilter.regions => marker is RegionMarker,
    },
  );
  return shown.toList()..sort(_compare);
}

int _compare(Marker a, Marker b) {
  final byTime = a.startMs.compareTo(b.startMs);
  if (byTime != 0) return byTime;
  final byType = (a is LabelMarker ? 0 : 1).compareTo(b is LabelMarker ? 0 : 1);
  return byType != 0 ? byType : a.id.compareTo(b.id);
}
