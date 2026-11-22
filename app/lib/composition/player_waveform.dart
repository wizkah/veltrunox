import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../design/tokens/app_geometry.dart';
import '../features/markers/marker.dart';
import '../features/markers/marker_actions_sheet.dart';
import '../features/markers/marker_controller.dart';
import '../features/markers/marker_flows.dart';
import '../features/markers/marker_texts.dart';
import '../features/player/player_controller.dart';
import '../features/player/scrub_hold.dart';
import '../features/waveform/new_region_range.dart';
import '../features/waveform/waveform_panel.dart';
import '../features/waveform/waveform_placeholder.dart';
import '../features/waveform/waveform_source.dart';
import '../features/waveform/waveform_texts.dart';
import '../features/waveform/waveform_view_controller.dart';
import '../shared_ui/action_button.dart';
import '../shared_ui/toast_notice.dart';

/// The waveform of the loaded recording wired to the shared player and to the
/// recording's markers: what the user does in the waveform becomes player
/// commands and marker writes, and what the server reports shows up in it.
class PlayerWaveform extends ConsumerStatefulWidget {
  const PlayerWaveform({
    required this.recordingId,
    required this.scanned,
    super.key,
  });

  final int recordingId;

  /// Whether the recording has been scanned; only then it has a waveform.
  final bool scanned;

  @override
  ConsumerState<PlayerWaveform> createState() => _PlayerWaveformState();
}

class _PlayerWaveformState extends ConsumerState<PlayerWaveform> {
  /// Moving the playhead behaves like moving the position slider.
  late final ScrubHold _scrub = ScrubHold(
    onScrub: (position) => _player.scrub(position),
    onScrubEnd: (position) => _player.endScrub(position),
    onHold: (position) => _player.hold(
      position,
      running: ref.read(playerStateProvider).value?.running ?? false,
    ),
    onHoldEnd: () => _player.endHold(),
  );

  /// The region just drawn, to be selected once it shows up in the list.
  ({int startMs, int endMs})? _awaitedRegion;

  PlayerController get _player => ref.read(playerControllerProvider);
  MarkerController get _markerWriter =>
      ref.read(markerControllerProvider(widget.recordingId));

  @override
  void dispose() {
    _scrub.dispose();
    super.dispose();
  }

  void _selectAwaitedRegion(List<Marker> markers) {
    final awaited = _awaitedRegion;
    if (awaited == null) return;
    final found = markers.whereType<RegionMarker>().where(
      (region) =>
          region.startMs == awaited.startMs && region.endMs == awaited.endMs,
    );
    if (found.isEmpty) return;
    _awaitedRegion = null;
    ref
        .read(waveformViewProvider(widget.recordingId).notifier)
        .selectRegion(found.first.id);
  }

  Future<void> _drawRegion(List<Marker> markers, int startMs, int endMs) async {
    final number = markers.whereType<RegionMarker>().length + 1;
    _awaitedRegion = (startMs: startMs, endMs: endMs);
    final accepted = await _markerWriter.addRegion(
      title: MarkerTexts.defaultRegionName(number),
      startMs: startMs,
      endMs: endMs,
    );
    if (!accepted) {
      _awaitedRegion = null;
      _reportFailure();
    }
  }

  Future<void> _resizeRegion(RegionMarker region, int start, int end) async {
    final accepted = await _markerWriter.updateRegion(
      region,
      title: region.title,
      startMs: start,
      endMs: end,
    );
    if (!accepted) _reportFailure();
  }

  Future<void> _setRegionHere(List<Marker> markers, int lengthMs) async {
    final position = await ref.read(playerSourceProvider).position();
    if (position == null) return;
    final visibleEnd = ref
        .read(waveformViewProvider(widget.recordingId))
        .viewport
        ?.endMs;
    final range = defaultRegionRange(
      positionMs: position,
      visibleEndMs: visibleEnd ?? lengthMs.toDouble(),
      lengthMs: lengthMs,
    );
    await _drawRegion(markers, range.startMs, range.endMs);
  }

  Future<void> _setLabelHere(List<Marker> markers) async {
    final position = await ref.read(playerSourceProvider).position();
    if (position == null) return;
    final number = markers.whereType<LabelMarker>().length + 1;
    final accepted = await _markerWriter.addLabel(
      title: MarkerTexts.defaultLabelName(number),
      positionMs: position,
    );
    if (!accepted) _reportFailure();
  }

  Future<void> _openActions(Marker marker, int lengthMs) async {
    final action = await showMarkerActions(context, marker);
    if (action == null || !mounted) return;
    final flows = MarkerFlows(
      context: context,
      controller: _markerWriter,
      lengthMs: lengthMs,
    );
    switch (action) {
      case MarkerAction.edit:
        await flows.edit(marker);
      case MarkerAction.delete:
        await flows.delete(marker);
    }
  }

  void _reportFailure() {
    if (!mounted) return;
    ToastNotice.show(
      context,
      kind: ToastKind.error,
      message: MarkerTexts.saveFailed,
    );
  }

  @override
  Widget build(BuildContext context) {
    final recordingId = widget.recordingId;
    ref.listen(
      markersProvider(recordingId),
      (_, next) => _selectAwaitedRegion(next.value ?? const []),
    );
    final loaded = ref
        .watch(
          recordingWaveformProvider((
            recordingId: recordingId,
            scanned: widget.scanned,
          )),
        )
        .value;
    if (loaded == null) {
      return WaveformPlaceholder(
        text: widget.scanned
            ? WaveformTexts.loading
            : WaveformTexts.unavailable,
      );
    }
    final markers = ref.watch(markersProvider(recordingId)).value ?? const [];
    final position = (ref.watch(playerPositionProvider).value ?? 0).clamp(
      0,
      loaded.lengthMs,
    );
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        WaveformPanel(
          recordingId: recordingId,
          durationMs: loaded.lengthMs,
          data: loaded.data,
          positionMs: position,
          markers: markers,
          onPlayFrom: _player.playFrom,
          onPlayRegion: (region) =>
              _player.playRegion(startMs: region.startMs, endMs: region.endMs),
          onSeek: _player.seek,
          onScrub: _scrub.moved,
          onScrubEnd: _scrub.released,
          onRegionDrawn: (start, end) => _drawRegion(markers, start, end),
          onRegionResized: _resizeRegion,
          onMarkerContext: (marker) => _openActions(marker, loaded.lengthMs),
        ),
        const SizedBox(height: AppSpace.s2),
        Wrap(
          spacing: AppSpace.s2,
          runSpacing: AppSpace.s2,
          children: [
            ActionButton(
              label: MarkerTexts.setLabelHere,
              icon: Icons.add_location_alt_outlined,
              kind: ActionKind.secondary,
              onPressed: () => _setLabelHere(markers),
            ),
            ActionButton(
              label: MarkerTexts.setRegionHere,
              icon: Icons.add_road,
              kind: ActionKind.secondary,
              onPressed: () => _setRegionHere(markers, loaded.lengthMs),
            ),
          ],
        ),
      ],
    );
  }
}
