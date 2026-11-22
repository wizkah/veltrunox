import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'time_viewport.dart';

/// The local view of one recording's waveform: what is visible, whether
/// dragged markers snap, and which region is
/// selected for editing. None of it is shared with other clients.
@immutable
class WaveformViewState {
  const WaveformViewState({
    this.viewport,
    this.snapEnabled = true,
    this.selectedRegionId,
    this.showAllLanes = false,
  });

  /// `null` until the width of the waveform is known.
  final TimeViewport? viewport;
  final bool snapEnabled;
  final int? selectedRegionId;
  final bool showAllLanes;

  WaveformViewState copyWith({
    TimeViewport? viewport,
    bool? snapEnabled,
    Object? selectedRegionId = _keep,
    bool? showAllLanes,
  }) => WaveformViewState(
    viewport: viewport ?? this.viewport,
    snapEnabled: snapEnabled ?? this.snapEnabled,
    selectedRegionId: identical(selectedRegionId, _keep)
        ? this.selectedRegionId
        : selectedRegionId as int?,
    showAllLanes: showAllLanes ?? this.showAllLanes,
  );
}

const Object _keep = Object();

class WaveformViewController extends Notifier<WaveformViewState> {
  WaveformViewController(this.recordingId);

  /// The recording this view belongs to; a new recording has a new view.
  final int recordingId;

  @override
  WaveformViewState build() => const WaveformViewState();

  /// Takes the size of the waveform: the first call shows the whole
  /// recording, later ones keep the visible range.
  void resize({required int durationMs, required double widthPx}) {
    final current = state.viewport;
    if (current != null &&
        current.widthPx == widthPx &&
        current.durationMs == durationMs) {
      return;
    }
    state = state.copyWith(
      viewport: current == null || current.durationMs != durationMs
          ? TimeViewport.fit(durationMs: durationMs, widthPx: widthPx)
          : current.withWidth(widthPx),
    );
  }

  void _update(TimeViewport Function(TimeViewport viewport) change) {
    final current = state.viewport;
    if (current != null) state = state.copyWith(viewport: change(current));
  }

  void zoomAt(double factor, double anchorX) =>
      _update((v) => v.zoomedAt(factor, anchorX));

  void zoomByButton(double factor, {required int playheadMs}) => _update(
    (v) => v.zoomedForButton(factor, playheadMs: playheadMs.toDouble()),
  );

  void panByPixels(double deltaPx) => _update((v) => v.pannedByPixels(deltaPx));

  void fit() => _update((v) => v.fitted());

  void reveal(int startMs, int endMs) =>
      _update((v) => v.revealing(startMs.toDouble(), endMs.toDouble()));

  void toggleSnap() => state = state.copyWith(snapEnabled: !state.snapEnabled);

  void selectRegion(int? id) => state = state.copyWith(selectedRegionId: id);

  void toggleAllLanes() =>
      state = state.copyWith(showAllLanes: !state.showAllLanes);
}

final waveformViewProvider =
    NotifierProvider.family<WaveformViewController, WaveformViewState, int>(
      WaveformViewController.new,
    );
