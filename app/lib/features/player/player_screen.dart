import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/panel_surface.dart';
import 'no_recording_notice.dart';
import 'player_analysis.dart';
import 'player_controller.dart';
import 'player_controls.dart';
import 'player_texts.dart';
import 'recording_header.dart';

/// The player page: the loaded recording with its controls and its markers.
/// What the markers look like is passed in, so the player knows no marker
/// feature: [markers] stands beside the controls on a wide screen, [markerOpener]
/// replaces it on a narrow one.
class PlayerScreen extends ConsumerWidget {
  const PlayerScreen({
    required this.onChooseRecording,
    this.title,
    this.analysis,
    this.lengthMs,
    this.connected = true,
    this.markers,
    this.markerOpener,
    this.waveform,
    super.key,
  });

  /// Leads to the place where a recording is chosen (the library).
  final VoidCallback onChooseRecording;

  /// Name of the loaded recording, once the library knows it.
  final String? title;
  final PlayerAnalysis? analysis;

  /// Length of the loaded recording, once the server has scanned it.
  final int? lengthMs;

  /// Whether the server is connected; without it the controls are disabled.
  final bool connected;
  final Widget? markers;
  final Widget? markerOpener;

  /// Shown between the title and the controls; moves the position, so the
  /// plain position slider is left out when it is there.
  final Widget? waveform;

  /// From this width the markers stand beside the controls.
  static const _markersBesideMin = AppBreakpoint.desktopMin;
  static const _markerColumnWidth = 380.0;

  /// From this height (and when the markers stand beside) the page fits the
  /// screen: the markers list and the regions scroll on their own. Below it
  /// even the smallest waveform and one region lane do not fit, and the whole
  /// page scrolls.
  static const _fitHeightMin = 720.0;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(playerStateProvider);
    final loaded = state.value;
    if (loaded == null) {
      return Center(
        child: Padding(
          padding: EdgeInsets.all(AppSpace.s6),
          child: Text(PlayerTexts.waiting),
        ),
      );
    }
    if (loaded.recordingId == null) {
      return NoRecordingNotice(onChoose: onChooseRecording);
    }
    return LayoutBuilder(
      builder: (context, constraints) {
        final beside = constraints.maxWidth >= _markersBesideMin;
        final controls = PlayerControls(
          state: loaded,
          analysis: analysis,
          lengthMs: lengthMs,
          connected: connected,
          above: _aboveTransport(showMarkerOpener: !beside),
          showPositionSlider: waveform == null,
        );
        final header = _header();
        final fits =
            beside &&
            constraints.hasBoundedHeight &&
            constraints.maxHeight >= _fitHeightMin;
        final main = Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            ?header,
            if (fits) Expanded(child: controls) else controls,
          ],
        );
        if (!beside) {
          return SingleChildScrollView(
            padding: const EdgeInsets.all(AppSpace.s4),
            child: main,
          );
        }
        final row = Row(
          crossAxisAlignment: fits
              ? CrossAxisAlignment.stretch
              : CrossAxisAlignment.start,
          children: [
            if (markers != null) ...[
              SizedBox(
                width: _markerColumnWidth,
                child: PanelSurface(child: markers!),
              ),
              const SizedBox(width: AppSpace.s6),
            ],
            Expanded(child: main),
          ],
        );
        return fits
            ? Padding(padding: const EdgeInsets.all(AppSpace.s4), child: row)
            : SingleChildScrollView(
                padding: const EdgeInsets.all(AppSpace.s4),
                child: row,
              );
      },
    );
  }

  /// What stands above the transport: on a narrow screen the way to the
  /// markers, then the waveform.
  Widget? _aboveTransport({required bool showMarkerOpener}) {
    final opener = showMarkerOpener ? markerOpener : null;
    if (opener == null) return waveform;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Align(alignment: Alignment.centerLeft, child: opener),
        if (waveform != null) const SizedBox(height: AppSpace.s3),
        ?waveform,
      ],
    );
  }

  Widget? _header() {
    final name = title;
    if (name == null) return null;
    return Padding(
      padding: const EdgeInsets.only(bottom: AppSpace.s3),
      child: RecordingHeader(
        title: name,
        analysis: analysis,
        lengthMs: lengthMs,
      ),
    );
  }
}
