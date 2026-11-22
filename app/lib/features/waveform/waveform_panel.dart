import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../markers/marker.dart';
import 'label_clusters.dart';
import 'label_layer.dart';
import 'label_group_dialog.dart';
import 'playhead.dart';
import 'region_lane_layout.dart';
import 'region_lanes.dart';
import 'snap_targets.dart';
import 'time_viewport.dart';
import 'timeline_ruler.dart';
import 'waveform_data.dart';
import 'waveform_geometry.dart';
import 'waveform_gesture_layer.dart';
import 'waveform_renderer.dart';
import 'waveform_texts.dart';
import 'waveform_toolbar.dart';
import 'waveform_view_controller.dart';

/// The waveform with ruler, labels, playhead and region lanes. It composes the
/// layers and turns what the user does into intents; it neither knows where
/// the data comes from nor how an intent is carried out.
class WaveformPanel extends ConsumerWidget {
  const WaveformPanel({
    required this.recordingId,
    required this.durationMs,
    required this.data,
    required this.positionMs,
    required this.markers,
    required this.onPlayFrom,
    required this.onPlayRegion,
    required this.onSeek,
    required this.onScrub,
    required this.onScrubEnd,
    required this.onRegionDrawn,
    required this.onRegionResized,
    required this.onMarkerContext,
    super.key,
  });

  final int recordingId;
  final int durationMs;

  /// `null` while the waveform is not available.
  final WaveformData? data;
  final int positionMs;
  final List<Marker> markers;

  /// Play from this position (a label was chosen).
  final ValueChanged<int> onPlayFrom;
  final ValueChanged<RegionMarker> onPlayRegion;

  /// Move the position without playing.
  final ValueChanged<int> onSeek;
  final ValueChanged<int> onScrub;
  final ValueChanged<int> onScrubEnd;
  final void Function(int startMs, int endMs) onRegionDrawn;
  final void Function(RegionMarker region, int startMs, int endMs)
  onRegionResized;
  final ValueChanged<Marker> onMarkerContext;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final view = ref.watch(waveformViewProvider(recordingId));
    final controller = ref.read(waveformViewProvider(recordingId).notifier);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        WaveformToolbar(
          snapEnabled: view.snapEnabled,
          drawingRegion: view.drawingRegion,
          onZoomIn: () => controller.zoomByButton(2, playheadMs: positionMs),
          onZoomOut: () => controller.zoomByButton(0.5, playheadMs: positionMs),
          onFit: controller.fit,
          onToggleSnap: controller.toggleSnap,
          onToggleDrawRegion: controller.toggleDrawingRegion,
        ),
        LayoutBuilder(
          builder: (context, constraints) {
            final width = constraints.maxWidth;
            WidgetsBinding.instance.addPostFrameCallback(
              (_) => controller.resize(durationMs: durationMs, widthPx: width),
            );
            final viewport =
                view.viewport ??
                TimeViewport.fit(durationMs: durationMs, widthPx: width);
            return _Layers(
              viewport: viewport,
              view: view,
              narrow: width < AppBreakpoint.tabletMin,
              panel: this,
              controller: controller,
            );
          },
        ),
      ],
    );
  }
}

class _Layers extends StatelessWidget {
  const _Layers({
    required this.viewport,
    required this.view,
    required this.narrow,
    required this.panel,
    required this.controller,
  });

  final TimeViewport viewport;
  final WaveformViewState view;
  final bool narrow;
  final WaveformPanel panel;
  final WaveformViewController controller;

  List<LabelMarker> get _labels =>
      panel.markers.whereType<LabelMarker>().toList();
  List<RegionMarker> get _regions =>
      panel.markers.whereType<RegionMarker>().toList();

  Future<void> _openGroup(BuildContext context, LabelCluster cluster) async {
    final chosen = await showLabelGroupDialog(context, cluster);
    if (chosen != null) panel.onPlayFrom(chosen.positionMs);
  }

  @override
  Widget build(BuildContext context) {
    final regions = _regions;
    final laneCount = layoutRegionLanes(regions).laneCount;
    final collapsible = narrow && laneCount > WaveformGeometry.compactLanes;
    final maxLanes = collapsible && !view.showAllLanes
        ? WaveformGeometry.compactLanes
        : null;
    final shownLanes = maxLanes == null || laneCount < maxLanes
        ? laneCount
        : maxLanes;
    final waveformTop =
        WaveformGeometry.rulerHeight + WaveformGeometry.labelBandHeight;
    final topHeight = waveformTop + WaveformGeometry.waveformHeight;
    final totalHeight = topHeight + WaveformGeometry.lanesHeight(shownLanes);
    final snapTargets = view.snapEnabled
        ? SnapTargets.from(panel.markers)
        : null;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        SizedBox(
          width: viewport.widthPx,
          height: totalHeight,
          child: Stack(
            children: [
              Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  TimelineRuler(viewport: viewport),
                  LabelLayer(
                    labels: _labels,
                    viewport: viewport,
                    onPlay: (label) => panel.onPlayFrom(label.positionMs),
                    onOpenGroup: (cluster) => _openGroup(context, cluster),
                    onContext: panel.onMarkerContext,
                  ),
                  WaveformGestureLayer(
                    viewport: viewport,
                    drawingRegion: view.drawingRegion,
                    snapTargets: snapTargets,
                    onTapAt: (ms) =>
                        panel.onSeek(ms.clamp(0, panel.durationMs)),
                    onPan: controller.panByPixels,
                    onZoom: controller.zoomAt,
                    onRegionDrawn: (start, end) {
                      controller.endDrawingRegion();
                      panel.onRegionDrawn(start, end);
                    },
                    child: WaveformRenderer(
                      data: panel.data,
                      viewport: viewport,
                    ),
                  ),
                  RegionLanes(
                    regions: regions,
                    markers: panel.markers,
                    viewport: viewport,
                    selectedId: view.selectedRegionId,
                    maxLanes: maxLanes,
                    snapEnabled: view.snapEnabled,
                    onPlay: panel.onPlayRegion,
                    onSelect: (region) => controller.selectRegion(
                      region.id == view.selectedRegionId ? null : region.id,
                    ),
                    onContext: panel.onMarkerContext,
                    onResized: panel.onRegionResized,
                  ),
                ],
              ),
              Playhead(
                positionMs: panel.positionMs,
                viewport: viewport,
                height: totalHeight,
                hitTop: waveformTop,
                hitHeight: WaveformGeometry.waveformHeight,
                onScrub: panel.onScrub,
                onScrubEnd: panel.onScrubEnd,
              ),
            ],
          ),
        ),
        if (collapsible)
          Align(
            alignment: Alignment.centerLeft,
            child: TextButton.icon(
              onPressed: controller.toggleAllLanes,
              icon: Icon(
                view.showAllLanes ? Icons.expand_less : Icons.expand_more,
              ),
              label: Text(
                view.showAllLanes
                    ? WaveformTexts.fewerRegions
                    : WaveformTexts.allRegions,
              ),
            ),
          ),
      ],
    );
  }
}
