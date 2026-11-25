import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'marker.dart';
import 'marker_order.dart';
import 'marker_source.dart';
import 'marker_texts.dart';

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

  bool _conflict = false;

  /// What to tell the user after a write that was not accepted: another
  /// person's change replaced it (D-19), or it failed for another reason.
  String get failureMessage =>
      _conflict ? MarkerTexts.replacedByOther : MarkerTexts.saveFailed;

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

  /// Adds a label, or a region when an end is given.
  Future<bool> add({required String title, required int startMs, int? endMs}) =>
      endMs == null
      ? addLabel(title: title, positionMs: startMs)
      : addRegion(title: title, startMs: startMs, endMs: endMs);

  /// Changes a marker. A label that gets an end becomes a region and a region
  /// that loses its end becomes a label; the server replaces the marker in one
  /// step, so a failure leaves the old marker untouched.
  Future<bool> change(
    Marker marker, {
    required String title,
    required int startMs,
    int? endMs,
  }) => _attempt(() async {
    switch ((marker, endMs)) {
      case (final LabelMarker label, null):
        await _source.updateLabel(label, title: title, positionMs: startMs);
      case (final RegionMarker region, final int end):
        await _source.updateRegion(
          region,
          title: title,
          startMs: startMs,
          endMs: end,
        );
      case (final LabelMarker label, final int end):
        await _source.labelToRegion(
          label,
          title: title,
          startMs: startMs,
          endMs: end,
        );
      case (final RegionMarker region, null):
        await _source.regionToLabel(region, title: title, positionMs: startMs);
    }
  });

  Future<bool> addLabel({required String title, required int positionMs}) =>
      _attempt(
        () => _source.addLabel(
          _recordingId,
          title: title,
          positionMs: positionMs,
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

  /// Replaces [regions] by one region named [title] over [startMs]–[endMs]
  /// (destructive). The server does it in one step: all or nothing.
  Future<bool> combineRegions(
    List<RegionMarker> regions, {
    required String title,
    required int startMs,
    required int endMs,
  }) => _attempt(
    () => _source.mergeRegions(
      _recordingId,
      regions,
      title: title,
      startMs: startMs,
      endMs: endMs,
    ),
  );

  /// Replaces [region] by two regions that meet at [positionMs], in one step
  /// on the server.
  Future<bool> splitRegion(RegionMarker region, {required int positionMs}) =>
      _attempt(
        () => _source.splitRegion(
          region,
          positionMs: positionMs,
          firstTitle: '${region.title} (1)',
          secondTitle: '${region.title} (2)',
        ),
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
      _conflict = false;
      return true;
    } on MarkerConflictException {
      _conflict = true;
      reload();
      return false;
    } on Object {
      _conflict = false;
      reload();
      return false;
    }
  }
}
