import 'marker.dart';

/// A write was refused because someone else changed the marker first (D-19).
class MarkerConflictException implements Exception {
  const MarkerConflictException();
}

/// Port: where the markers of a recording come from and are written to. Every
/// write names the version the user saw; a stale one fails.
abstract interface class MarkerSource {
  Future<List<Marker>> list(int recordingId);

  /// Ids of recordings whose markers changed on the server.
  Stream<int> changes();

  Future<void> addLabel(
    int recordingId, {
    required String title,
    required int positionMs,
  });
  Future<void> updateLabel(
    LabelMarker label, {
    required String title,
    required int positionMs,
  });
  Future<void> deleteLabel(LabelMarker label);

  Future<void> addRegion(
    int recordingId, {
    required String title,
    required int startMs,
    required int endMs,
  });
  Future<void> updateRegion(
    RegionMarker region, {
    required String title,
    required int startMs,
    required int endMs,
  });
  Future<void> deleteRegion(RegionMarker region);

  /// Replaces a label by a region in one step; a failure leaves the label.
  Future<void> labelToRegion(
    LabelMarker label, {
    required String title,
    required int startMs,
    required int endMs,
  });

  /// Replaces a region by a label in one step; a failure leaves the region.
  Future<void> regionToLabel(
    RegionMarker region, {
    required String title,
    required int positionMs,
  });
}
