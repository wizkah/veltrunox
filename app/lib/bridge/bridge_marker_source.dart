import '../features/markers/marker.dart';
import '../features/markers/marker_source.dart';
import 'bridge_session.dart';
import 'generated/api.dart' as rust;

/// Adapter: reads and writes markers through the Rust client core.
class BridgeMarkerSource implements MarkerSource {
  BridgeMarkerSource({required this.serverUrl, required this.session});

  final String serverUrl;
  final BridgeSession session;

  @override
  Future<List<Marker>> list(int recordingId) async {
    final found = await rust.listBookmarks(
      serverUrl: serverUrl,
      recordingId: BigInt.from(recordingId),
    );
    return [
      for (final label in found.labels)
        LabelMarker(
          id: label.id.toInt(),
          version: label.version.toInt(),
          title: label.name,
          positionMs: label.positionMs.toInt(),
        ),
      for (final region in found.regions)
        RegionMarker(
          id: region.id.toInt(),
          version: region.version.toInt(),
          title: region.name,
          startMs: region.startMs.toInt(),
          endMs: region.endMs.toInt(),
        ),
    ];
  }

  @override
  Stream<int> changes() => session.bookmarkChanges();

  @override
  Future<void> addLabel(
    int recordingId, {
    required String title,
    required int positionMs,
  }) => rust.createLabel(
    serverUrl: serverUrl,
    recordingId: BigInt.from(recordingId),
    name: title,
    positionMs: BigInt.from(positionMs),
  );

  @override
  Future<void> updateLabel(
    LabelMarker label, {
    required String title,
    required int positionMs,
  }) => rust.changeLabel(
    serverUrl: serverUrl,
    id: BigInt.from(label.id),
    expectedVersion: BigInt.from(label.version),
    name: title,
    positionMs: BigInt.from(positionMs),
  );

  @override
  Future<void> deleteLabel(LabelMarker label) => rust.removeLabel(
    serverUrl: serverUrl,
    id: BigInt.from(label.id),
    expectedVersion: BigInt.from(label.version),
  );

  @override
  Future<void> addRegion(
    int recordingId, {
    required String title,
    required int startMs,
    required int endMs,
  }) => rust.createRegion(
    serverUrl: serverUrl,
    recordingId: BigInt.from(recordingId),
    name: title,
    startMs: BigInt.from(startMs),
    endMs: BigInt.from(endMs),
  );

  @override
  Future<void> updateRegion(
    RegionMarker region, {
    required String title,
    required int startMs,
    required int endMs,
  }) => rust.changeRegion(
    serverUrl: serverUrl,
    id: BigInt.from(region.id),
    expectedVersion: BigInt.from(region.version),
    name: title,
    startMs: BigInt.from(startMs),
    endMs: BigInt.from(endMs),
  );

  @override
  Future<void> deleteRegion(RegionMarker region) => rust.removeRegion(
    serverUrl: serverUrl,
    id: BigInt.from(region.id),
    expectedVersion: BigInt.from(region.version),
  );

  @override
  Future<void> labelToRegion(
    LabelMarker label, {
    required String title,
    required int startMs,
    required int endMs,
  }) => rust.labelToRegion(
    serverUrl: serverUrl,
    id: BigInt.from(label.id),
    expectedVersion: BigInt.from(label.version),
    name: title,
    startMs: BigInt.from(startMs),
    endMs: BigInt.from(endMs),
  );

  @override
  Future<void> regionToLabel(
    RegionMarker region, {
    required String title,
    required int positionMs,
  }) => rust.regionToLabel(
    serverUrl: serverUrl,
    id: BigInt.from(region.id),
    expectedVersion: BigInt.from(region.version),
    name: title,
    positionMs: BigInt.from(positionMs),
  );
}
