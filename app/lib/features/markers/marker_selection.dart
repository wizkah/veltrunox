import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'marker.dart';

/// The regions ticked in the list, by id; local to this client. Ids of regions
/// that no longer exist are ignored by the readers, not removed here.
class MarkerSelection extends Notifier<Set<int>> {
  @override
  Set<int> build() => const {};

  void toggle(int regionId) => state = state.contains(regionId)
      ? ({...state}..remove(regionId))
      : {...state, regionId};

  void clear() => state = const {};
}

final markerSelectionProvider =
    NotifierProvider.family<MarkerSelection, Set<int>, int>(
      (recordingId) => MarkerSelection(),
    );

/// Whether the list offers to tick regions; local to this client.
class MarkerSelecting extends Notifier<bool> {
  MarkerSelecting(this.recordingId);

  final int recordingId;

  @override
  bool build() => false;

  /// Switches ticking on or off; switching it off drops what was ticked.
  void toggle() {
    state = !state;
    if (!state) ref.read(markerSelectionProvider(recordingId).notifier).clear();
  }
}

final markerSelectingProvider =
    NotifierProvider.family<MarkerSelecting, bool, int>(MarkerSelecting.new);

/// The ticked regions that still exist, in time order.
List<RegionMarker> selectedRegions(List<Marker> all, Set<int> ids) =>
    (all.whereType<RegionMarker>().where((r) => ids.contains(r.id)).toList()
      ..sort(
        (a, b) => a.startMs != b.startMs
            ? a.startMs.compareTo(b.startMs)
            : a.id.compareTo(b.id),
      ));
