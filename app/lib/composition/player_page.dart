import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../app_shell/shell_destination.dart';
import '../features/markers/marker.dart';
import '../features/markers/marker_panel.dart';
import '../features/markers/marker_sheet_button.dart';
import '../features/player/player_controller.dart';
import '../features/player/player_screen.dart';
import 'loaded_recording.dart';
import 'player_waveform.dart';

/// The player with the markers of its recording wired to it: playing a marker
/// starts the shared player.
class PlayerPage extends ConsumerWidget {
  const PlayerPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final recordingId = ref.watch(
      playerStateProvider.select((state) => state.value?.recordingId),
    );
    final recording = ref.watch(loadedRecordingProvider);
    final analysis = ref.watch(loadedAnalysisProvider);
    final lengthMs = recordingId == null
        ? null
        : ref
              .watch(
                playerLengthProvider((
                  recordingId: recordingId,
                  scanned: analysis != null,
                )),
              )
              .value;
    final player = ref.read(playerControllerProvider);
    Widget panel() => MarkerPanel(
      recordingId: recordingId!,
      lengthMs: lengthMs,
      readPlayhead: ref.read(playerSourceProvider).position,
      onPlay: (marker) => switch (marker) {
        LabelMarker() => player.playFrom(marker.positionMs),
        RegionMarker() => player.playRegion(
          startMs: marker.startMs,
          endMs: marker.endMs,
        ),
      },
    );
    return PlayerScreen(
      onChooseRecording: () => ref
          .read(shellDestinationProvider.notifier)
          .select(ShellDestination.library),
      title: recording?.title,
      waveform: recordingId == null
          ? null
          : PlayerWaveform(recordingId: recordingId, scanned: analysis != null),
      analysis: analysis,
      markers: recordingId == null ? null : panel(),
      markerOpener: recordingId == null
          ? null
          : MarkerSheetButton(
              recordingId: recordingId,
              panelBuilder: (_) => panel(),
            ),
    );
  }
}
