import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'marker.dart';
import 'marker_order.dart';
import 'marker_source.dart';

/// Supplies the [MarkerSource]; overridden at the application root.
final markerSourceProvider = Provider<MarkerSource>(
  (ref) => throw UnimplementedError('markerSourceProvider must be overridden'),
);

/// The markers of one recording. They are loaded again whenever the server
/// reports a change (also one made by this client); a failed reload keeps
/// what is shown, a failed first load is an error.
final markersProvider = StreamProvider.family<List<Marker>, int>((
  ref,
  recordingId,
) async* {
  final source = ref.watch(markerSourceProvider);
  yield await source.list(recordingId);
  await for (final changed in source.changes()) {
    if (changed != recordingId) continue;
    try {
      yield await source.list(recordingId);
    } on Object {
      continue;
    }
  }
});

class MarkerFilterNotifier extends Notifier<MarkerFilter> {
  @override
  MarkerFilter build() => MarkerFilter.all;

  void select(MarkerFilter filter) => state = filter;
}

/// Which kind of marker the list shows; local to this client.
final markerFilterProvider =
    NotifierProvider<MarkerFilterNotifier, MarkerFilter>(
      MarkerFilterNotifier.new,
    );

/// Writes the markers of one recording. Every method answers whether the
/// server accepted the change; a refused one makes the list load again.
final markerControllerProvider = Provider.family<MarkerController, int>(
  (ref, recordingId) => MarkerController(
    ref.watch(markerSourceProvider),
    recordingId,
    reload: () => ref.invalidate(markersProvider(recordingId)),
  ),
);

class MarkerController {
  MarkerController(this._source, this._recordingId, {required this.reload});

  final MarkerSource _source;
  final int _recordingId;
  final void Function() reload;

  Future<bool> addRegion({
    required String title,
    required int startMs,
    required int endMs,
  }) => _attempt(
    () => _source.addRegion(
      _recordingId,
      title: title,
      startMs: startMs,
      endMs: endMs,
    ),
  );

  Future<bool> updateRegion(
    RegionMarker region, {
    required String title,
    required int startMs,
    required int endMs,
  }) => _attempt(
    () => _source.updateRegion(
      region,
      title: title,
      startMs: startMs,
      endMs: endMs,
    ),
  );

  Future<bool> updateLabel(
    LabelMarker label, {
    required String title,
    required int positionMs,
  }) => _attempt(
    () => _source.updateLabel(label, title: title, positionMs: positionMs),
  );

  Future<bool> delete(Marker marker) => _attempt(
    () => switch (marker) {
      LabelMarker() => _source.deleteLabel(marker),
      RegionMarker() => _source.deleteRegion(marker),
    },
  );

  Future<bool> _attempt(Future<void> Function() write) async {
    try {
      await write();
      return true;
    } on Object {
      reload();
      return false;
    }
  }
}
