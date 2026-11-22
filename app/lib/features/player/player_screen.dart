import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import 'no_recording_notice.dart';
import 'player_analysis.dart';
import 'player_controller.dart';
import 'player_controls.dart';
import 'player_texts.dart';

/// The player page: the loaded recording with its controls and its markers.
/// What the markers look like is passed in, so the player knows no marker
/// feature: [markers] stands beside the controls on a wide screen, [markerOpener]
/// replaces it on a narrow one.
class PlayerScreen extends ConsumerWidget {
  const PlayerScreen({
    required this.onChooseRecording,
    this.title,
    this.analysis,
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
  final Widget? markers;
  final Widget? markerOpener;

  /// Shown between the title and the controls; moves the position, so the
  /// plain position slider is left out when it is there.
  final Widget? waveform;

  /// From this width the markers stand beside the controls.
  static const _markersBesideMin = AppBreakpoint.desktopMin;
  static const _markerColumnWidth = 380.0;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(playerStateProvider);
    final loaded = state.value;
    if (loaded == null) {
      return const Center(
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
          above: _aboveTransport(showMarkerOpener: !beside),
          showPositionSlider: waveform == null,
        );
        final header = _title(context);
        final main = Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [?header, controls],
        );
        return SingleChildScrollView(
          padding: const EdgeInsets.all(AppSpace.s4),
          child: beside
              ? Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    if (markers != null) ...[
                      SizedBox(width: _markerColumnWidth, child: markers),
                      const SizedBox(width: AppSpace.s6),
                    ],
                    Expanded(child: main),
                  ],
                )
              : main,
        );
      },
    );
  }

  /// What stands above the transport: on a narrow screen the way to the
  /// markers, then the waveform.
  Widget? _aboveTransport({required bool showMarkerOpener}) {
    final opener = showMarkerOpener ? markerOpener : null;
    if (opener == null && waveform == null) return null;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (opener != null)
          Align(alignment: Alignment.centerLeft, child: opener),
        if (opener != null && waveform != null)
          const SizedBox(height: AppSpace.s3),
        ?waveform,
      ],
    );
  }

  Widget? _title(BuildContext context) {
    final name = title;
    if (name == null) return null;
    return Padding(
      padding: const EdgeInsets.only(bottom: AppSpace.s3),
      child: Text(name, style: Theme.of(context).textTheme.headlineMedium),
    );
  }
}
