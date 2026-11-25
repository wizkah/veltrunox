import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import '../../shared_ui/state_notice.dart';
import 'marker.dart';
import 'marker_controller.dart';
import 'marker_filter_bar.dart';
import 'marker_flows.dart';
import 'marker_order.dart';
import 'marker_row.dart';
import 'marker_selection.dart';
import 'marker_texts.dart';

/// The markers of a recording as one chronological list with a filter, a play
/// button per row, and the forms to add a region and to change or delete a
/// marker. Playing is done by the caller ([onPlay]); the panel knows no player.
/// Given a bounded height, the list scrolls inside it and the rest stays in
/// place.
class MarkerPanel extends ConsumerWidget {
  const MarkerPanel({
    required this.recordingId,
    required this.lengthMs,
    required this.readPlayhead,
    required this.onPlay,
    this.exchange,
    this.activeRange,
    super.key,
  });

  final int recordingId;

  /// The recording's length, if known.
  final int? lengthMs;

  /// The current playback position, for the start of a new region.
  final Future<int?> Function() readPlayhead;
  final ValueChanged<Marker> onPlay;

  /// What the caller offers beside "Add region", for example the exchange menu.
  final Widget? exchange;

  /// The range of the region being played, if any; that region is highlighted.
  final TimeSpan? activeRange;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final markers = ref.watch(markersProvider(recordingId));
    final filter = ref.watch(markerFilterProvider);
    MarkerFlows flows() => MarkerFlows(
      context: context,
      controller: ref.read(markerControllerProvider(recordingId)),
      lengthMs: lengthMs,
    );
    return LayoutBuilder(
      builder: (context, constraints) => _panel(
        context,
        ref,
        flows,
        markers,
        filter,
        scrolling: constraints.hasBoundedHeight,
      ),
    );
  }

  Widget _panel(
    BuildContext context,
    WidgetRef ref,
    MarkerFlows Function() flows,
    AsyncValue<List<Marker>> markers,
    MarkerFilter filter, {
    required bool scrolling,
  }) {
    final content = markers.when(
      data: (all) => _list(ref, all, filter, flows),
      loading: _loading,
      error: (_, _) =>
          _failure(() => ref.invalidate(markersProvider(recordingId))),
    );
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: scrolling ? MainAxisSize.max : MainAxisSize.min,
      children: [
        Text(MarkerTexts.title, style: Theme.of(context).textTheme.titleLarge),
        const SizedBox(height: AppSpace.s3),
        // A narrow panel puts the exchange menu under the button.
        Wrap(
          spacing: AppSpace.s2,
          runSpacing: AppSpace.s2,
          crossAxisAlignment: WrapCrossAlignment.center,
          children: [
            ActionButton(
              label: MarkerTexts.addRegion,
              icon: Icons.add,
              kind: ActionKind.secondary,
              onPressed: () async =>
                  flows().addRegion(playheadMs: await readPlayhead() ?? 0),
            ),
            ?exchange,
          ],
        ),
        const SizedBox(height: AppSpace.s3),
        MarkerFilterBar(
          selected: filter,
          onSelected: ref.read(markerFilterProvider.notifier).select,
        ),
        const SizedBox(height: AppSpace.s2),
        if (markers.value case final all?) _selectionBar(ref, all, flows),
        if (scrolling)
          Expanded(child: SingleChildScrollView(child: content))
        else
          content,
      ],
    );
  }

  /// Shown while regions are ticked: what can be done with them.
  Widget _selectionBar(
    WidgetRef ref,
    List<Marker> all,
    MarkerFlows Function() flows,
  ) {
    final selection = ref.read(markerSelectionProvider(recordingId).notifier);
    final regions = selectedRegions(
      all,
      ref.watch(markerSelectionProvider(recordingId)),
    );
    if (regions.isEmpty) return const SizedBox.shrink();
    final several = regions.length >= 2;
    return Padding(
      padding: const EdgeInsets.only(bottom: AppSpace.s2),
      child: Wrap(
        spacing: AppSpace.s2,
        runSpacing: AppSpace.s2,
        crossAxisAlignment: WrapCrossAlignment.center,
        children: [
          Text(MarkerTexts.selectedCount(regions.length)),
          ActionButton(
            label: MarkerTexts.combineRegions,
            icon: Icons.merge_type,
            kind: ActionKind.secondary,
            onPressed: several
                ? () async {
                    if (await flows().combine(regions)) selection.clear();
                  }
                : null,
          ),
          ActionButton(
            label: MarkerTexts.regionFromSelection,
            icon: Icons.add,
            kind: ActionKind.secondary,
            onPressed: several
                ? () async {
                    final number = all.whereType<RegionMarker>().length + 1;
                    if (await flows().addCovering(
                      regions,
                      title: MarkerTexts.defaultRegionName(number),
                    )) {
                      selection.clear();
                    }
                  }
                : null,
          ),
          ActionButton(
            label: MarkerTexts.clearSelection,
            kind: ActionKind.secondary,
            onPressed: selection.clear,
          ),
        ],
      ),
    );
  }

  Widget _list(
    WidgetRef ref,
    List<Marker> all,
    MarkerFilter filter,
    MarkerFlows Function() flows,
  ) {
    final shown = orderMarkers(all, filter);
    if (shown.isEmpty) {
      return StateNotice(
        message: all.isEmpty ? MarkerTexts.empty : MarkerTexts.emptyFiltered,
        icon: Icons.bookmark_border,
        framed: false,
      );
    }
    final ticked = ref.watch(markerSelectionProvider(recordingId));
    return Column(
      children: [
        for (final marker in shown)
          MarkerRow(
            key: ValueKey('${marker.runtimeType}-${marker.id}'),
            marker: marker,
            active: marker is RegionMarker && marker.isActiveIn(activeRange),
            selected: marker is RegionMarker
                ? ticked.contains(marker.id)
                : null,
            onToggleSelected: () => ref
                .read(markerSelectionProvider(recordingId).notifier)
                .toggle(marker.id),
            onPlay: () => onPlay(marker),
            onEdit: () => flows().edit(marker),
            onDelete: () => flows().delete(marker),
          ),
      ],
    );
  }

  Widget _loading() => const Padding(
    padding: EdgeInsets.all(AppSpace.s4),
    child: Center(child: CircularProgressIndicator()),
  );

  Widget _failure(VoidCallback onRetry) => StateNotice(
    message: MarkerTexts.failure,
    icon: Icons.error_outline,
    tone: NoticeTone.error,
    actionLabel: MarkerTexts.retry,
    actionIcon: Icons.refresh,
    onAction: onRetry,
    framed: false,
  );
}
