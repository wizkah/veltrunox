import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../app_shell/shell_destination.dart';
import '../features/connection/connection_controller.dart';
import '../features/connection/connection_status.dart';
import '../features/markers/marker.dart';
import '../features/markers/marker_panel.dart';
import '../features/markers/marker_sheet_button.dart';
import '../features/player/player_controller.dart';
import '../features/player/player_screen.dart';
import '../features/waveform/waveform_view_controller.dart';
import 'bookmark_exchange.dart';
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
    final lengthMs = recording?.lengthMs;
    final connected =
        ref.watch(connectionStatusProvider).value == ConnectionStatus.connected;
    final player = ref.read(playerControllerProvider);
    final activeRange = ref.watch(
      playerStateProvider.select((state) => state.value?.playRange),
    );
    Widget panel() => MarkerPanel(
      recordingId: recordingId!,
      exchange: BookmarkExchangeMenu(recordingId: recordingId),
      lengthMs: lengthMs,
      activeRange: activeRange,
      readPlayhead: ref.read(playerSourceProvider).position,
      onPlay: (marker) {
        final view = ref.read(waveformViewProvider(recordingId).notifier);
        switch (marker) {
          case LabelMarker():
            view.reveal(marker.positionMs, marker.positionMs);
            player.playFrom(marker.positionMs);
          case RegionMarker():
            view.reveal(marker.startMs, marker.endMs);
            player.playRegion(startMs: marker.startMs, endMs: marker.endMs);
        }
      },
    );
    return PlayerScreen(
      onChooseRecording: () => ref
          .read(shellDestinationProvider.notifier)
          .select(ShellDestination.library),
      title: recording?.title,
      waveform: recordingId == null
          ? null
          : PlayerWaveform(recordingId: recordingId, scanned: lengthMs != null),
      analysis: analysis,
      lengthMs: lengthMs,
      connected: connected,
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
