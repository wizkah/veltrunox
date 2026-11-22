import 'dart:async';

import 'package:veltrunox_app/features/markers/marker.dart';
import 'package:veltrunox_app/features/markers/marker_source.dart';

/// Holds markers in memory, applies writes at once and announces them like the
/// server does; a test can make writes fail.
class FakeMarkerSource implements MarkerSource {
  FakeMarkerSource(this.markers);

  final List<Marker> markers;
  final announced = StreamController<int>.broadcast();
  final calls = <String>[];
  Object? failure;
  Future<List<Marker>> Function()? listOverride;
  int listings = 0;
  var _nextId = 100;

  @override
  Future<List<Marker>> list(int recordingId) {
    listings++;
    return listOverride?.call() ?? Future.value(List.of(markers));
  }

  @override
  Stream<int> changes() => announced.stream;

  void _write(String call, void Function() change) {
    calls.add(call);
    final error = failure;
    if (error != null) throw error;
    change();
    announced.add(1);
  }

  @override
  Future<void> addLabel(
    int recordingId, {
    required String title,
    required int positionMs,
  }) async => _write(
    'addLabel $title $positionMs',
    () => markers.add(
      LabelMarker(
        id: _nextId++,
        version: 1,
        title: title,
        positionMs: positionMs,
      ),
    ),
  );

  @override
  Future<void> addRegion(
    int recordingId, {
    required String title,
    required int startMs,
    required int endMs,
  }) async => _write(
    'addRegion $title $startMs $endMs',
    () => markers.add(
      RegionMarker(
        id: _nextId++,
        version: 1,
        title: title,
        startMs: startMs,
        endMs: endMs,
      ),
    ),
  );

  @override
  Future<void> updateRegion(
    RegionMarker region, {
    required String title,
    required int startMs,
    required int endMs,
  }) async => _write('updateRegion ${region.id} v${region.version} $title '
      '$startMs $endMs', () {
    markers
      ..removeWhere((marker) => marker.id == region.id)
      ..add(
        RegionMarker(
          id: region.id,
          version: region.version + 1,
          title: title,
          startMs: startMs,
          endMs: endMs,
        ),
      );
  });

  @override
  Future<void> updateLabel(
    LabelMarker label, {
    required String title,
    required int positionMs,
  }) async => _write('updateLabel ${label.id} $title $positionMs', () {
    markers
      ..removeWhere((marker) => marker.id == label.id)
      ..add(
        LabelMarker(
          id: label.id,
          version: label.version + 1,
          title: title,
          positionMs: positionMs,
        ),
      );
  });

  @override
  Future<void> deleteLabel(LabelMarker label) async => _write(
    'deleteLabel ${label.id}',
    () => markers.removeWhere((marker) => marker.id == label.id),
  );

  @override
  Future<void> deleteRegion(RegionMarker region) async => _write(
    'deleteRegion ${region.id}',
    () => markers.removeWhere((marker) => marker.id == region.id),
  );
}
