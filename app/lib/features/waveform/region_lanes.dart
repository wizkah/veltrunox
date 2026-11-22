import 'dart:math' as math;

import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_typography.dart';
import '../../shared_ui/clock_format.dart';
import '../markers/marker.dart';
import 'region_handle.dart';
import 'region_item.dart';
import 'region_lane_layout.dart';
import 'snap_targets.dart';
import 'time_viewport.dart';
import 'waveform_geometry.dart';
import 'waveform_texts.dart';

enum RegionEdge { start, end }

/// The regions below the waveform, in lanes. While a handle is dragged only the
/// drawn region changes; the lane assignment stays as it was until the drag
/// ends and the new times arrive as [regions].
class RegionLanes extends StatefulWidget {
  const RegionLanes({
    required this.regions,
    required this.markers,
    required this.viewport,
    required this.selectedId,
    required this.maxLanes,
    required this.snapEnabled,
    required this.onPlay,
    required this.onSelect,
    required this.onContext,
    required this.onResized,
    super.key,
  });

  final List<RegionMarker> regions;

  /// All markers of the recording; the snap targets come from them.
  final List<Marker> markers;
  final TimeViewport viewport;
  final int? selectedId;

  /// How many lanes to show; `null` shows all of them.
  final int? maxLanes;
  final bool snapEnabled;
  final ValueChanged<RegionMarker> onPlay;
  final ValueChanged<RegionMarker> onSelect;
  final ValueChanged<RegionMarker> onContext;

  /// A handle was let go: the region with its new start and end.
  final void Function(RegionMarker region, int startMs, int endMs) onResized;

  @override
  State<RegionLanes> createState() => _RegionLanesState();
}

class _RegionLanesState extends State<RegionLanes> {
  LaneAssignment _assignment = LaneAssignment.empty;
  _HandleDrag? _drag;

  @override
  void initState() {
    super.initState();
    _assignment = layoutRegionLanes(widget.regions);
  }

  @override
  void didUpdateWidget(RegionLanes old) {
    super.didUpdateWidget(old);
    if (_drag == null) {
      _assignment = layoutRegionLanes(widget.regions, previous: _assignment);
    }
  }

  void _startDrag(RegionMarker region, RegionEdge edge) {
    setState(() {
      _drag = _HandleDrag(
        region: region,
        edge: edge,
        rawMs: (edge == RegionEdge.start ? region.startMs : region.endMs)
            .toDouble(),
        targets: widget.snapEnabled
            ? SnapTargets.from(widget.markers, excludeId: region.id)
            : null,
      );
    });
  }

  void _updateDrag(double deltaPx) {
    final drag = _drag;
    if (drag == null) return;
    setState(() => _drag = drag.moved(deltaPx, widget.viewport));
  }

  void _endDrag() {
    final drag = _drag;
    if (drag == null) return;
    setState(() => _drag = null);
    final (start, end) = (drag.startMs, drag.endMs);
    if (start != drag.region.startMs || end != drag.region.endMs) {
      widget.onResized(drag.region, start, end);
    }
  }

  @override
  Widget build(BuildContext context) {
    final viewport = widget.viewport;
    final shown = _assignment.visibleLanes(
      widget.maxLanes ?? _assignment.laneCount,
      selectedId: widget.selectedId,
    );
    return SizedBox(
      width: viewport.widthPx,
      height: WaveformGeometry.lanesHeight(shown.length),
      child: Stack(
        clipBehavior: Clip.hardEdge,
        children: [
          for (final region in widget.regions)
            if (_rowOf(region, shown) case final row?)
              ..._regionWidgets(region, row),
        ],
      ),
    );
  }

  int? _rowOf(RegionMarker region, List<int> shown) {
    final lane = _assignment.laneOf(region.id);
    final row = lane == null ? -1 : shown.indexOf(lane);
    return row < 0 ? null : row;
  }

  List<Widget> _regionWidgets(RegionMarker region, int row) {
    final viewport = widget.viewport;
    final drag = _drag;
    final shownRegion = drag != null && drag.region.id == region.id
        ? drag.preview
        : region;
    final left = viewport.timeToX(shownRegion.startMs.toDouble());
    final right = viewport.timeToX(shownRegion.endMs.toDouble());
    if (right < 0 || left > viewport.widthPx) return const [];
    final width = math.max(right - left, WaveformGeometry.regionMinWidth);
    final top = row * (WaveformGeometry.laneHeight + WaveformGeometry.laneGap);
    final selected = region.id == widget.selectedId;
    return [
      Positioned(
        left: left,
        top: top,
        width: width,
        child: RegionItem(
          region: shownRegion,
          selected: selected,
          toneIndex: _assignment.laneOf(region.id) ?? 0,
          showSelectButton: width >= 72,
          onPlay: () => widget.onPlay(region),
          onSelect: () => widget.onSelect(region),
          onContext: () => widget.onContext(region),
        ),
      ),
      if (selected) ...[
        _handle(region, RegionEdge.start, left, top),
        _handle(region, RegionEdge.end, left + width, top),
      ],
      if (drag != null && drag.region.id == region.id)
        Positioned(
          left: (drag.edge == RegionEdge.start ? left : left + width) - 24,
          top: top,
          child: _TimeHint(
            ms: drag.edge == RegionEdge.start ? drag.startMs : drag.endMs,
          ),
        ),
    ];
  }

  Widget _handle(RegionMarker region, RegionEdge edge, double x, double top) =>
      Positioned(
        left: x - WaveformGeometry.handleHitWidth / 2,
        top: top,
        child: RegionHandle(
          label: edge == RegionEdge.start
              ? WaveformTexts.regionStart
              : WaveformTexts.regionEnd,
          onDragStart: () => _startDrag(region, edge),
          onDragUpdate: _updateDrag,
          onDragEnd: _endDrag,
        ),
      );
}

/// The state of one handle drag: the raw position under the finger and the
/// edge it settles on (snapped to a marker if snapping is on).
final class _HandleDrag {
  _HandleDrag({
    required this.region,
    required this.edge,
    required this.rawMs,
    required this.targets,
    this.snappedMs,
  });

  final RegionMarker region;
  final RegionEdge edge;
  final double rawMs;
  final SnapTargets? targets;
  final int? snappedMs;

  int get startMs =>
      edge == RegionEdge.start ? _edgeMs(region.endMs - 1) : region.startMs;
  int get endMs =>
      edge == RegionEdge.end ? _edgeMs(region.startMs + 1) : region.endMs;

  /// The dragged edge, kept one millisecond away from the other one.
  int _edgeMs(int limit) {
    final value = snappedMs ?? rawMs.round();
    return edge == RegionEdge.start
        ? math.max(0, math.min(value, limit))
        : math.max(value, limit);
  }

  RegionMarker get preview => RegionMarker(
    id: region.id,
    version: region.version,
    title: region.title,
    startMs: startMs,
    endMs: endMs,
  );

  _HandleDrag moved(double deltaPx, TimeViewport viewport) {
    final raw = rawMs + deltaPx * viewport.msPerPx;
    final bounded = raw.clamp(0.0, viewport.durationMs.toDouble());
    return _HandleDrag(
      region: region,
      edge: edge,
      rawMs: bounded,
      targets: targets,
      snappedMs: targets?.nearest(bounded, viewport),
    );
  }
}

class _TimeHint extends StatelessWidget {
  const _TimeHint({required this.ms});

  final int ms;

  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.symmetric(horizontal: 4),
    color: AppColors.canvas,
    child: Text(
      formatClockExact(ms),
      style: const TextStyle(
        color: AppColors.textPrimary,
        fontSize: AppTypography.sizeSmall,
      ),
    ),
  );
}
