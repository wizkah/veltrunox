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
    super.key,
  });

  final int recordingId;

  /// The recording's length, if known.
  final int? lengthMs;

  /// The current playback position, for the start of a new region.
  final Future<int?> Function() readPlayhead;
  final ValueChanged<Marker> onPlay;

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
      data: (all) => _list(all, filter, flows),
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
        Align(
          alignment: Alignment.centerLeft,
          child: ActionButton(
            label: MarkerTexts.addRegion,
            icon: Icons.add,
            kind: ActionKind.secondary,
            onPressed: () async =>
                flows().addRegion(playheadMs: await readPlayhead() ?? 0),
          ),
        ),
        const SizedBox(height: AppSpace.s3),
        MarkerFilterBar(
          selected: filter,
          onSelected: ref.read(markerFilterProvider.notifier).select,
        ),
        const SizedBox(height: AppSpace.s2),
        if (scrolling)
          Expanded(child: SingleChildScrollView(child: content))
        else
          content,
      ],
    );
  }

  Widget _list(
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
    return Column(
      children: [
        for (final marker in shown)
          MarkerRow(
            key: ValueKey('${marker.runtimeType}-${marker.id}'),
            marker: marker,
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
