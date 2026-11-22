import 'marker.dart';

/// Port: where the markers of a recording come from and are written to. Every
/// write names the version the user saw; a stale one fails.
abstract interface class MarkerSource {
  Future<List<Marker>> list(int recordingId);

  /// Ids of recordings whose markers changed on the server.
  Stream<int> changes();

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
}
