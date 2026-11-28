import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../app_shell/shell_destination.dart';
import '../design/tokens/app_colors.dart';
import '../design/tokens/app_geometry.dart';
import '../features/access/access_controller.dart';
import '../features/connection/connection_controller.dart';
import '../features/connection/connection_status.dart';
import '../features/library/library_controller.dart';
import '../features/mixer/mixer_source.dart';
import '../features/mixer/mixer_texts.dart';
import '../features/mixer/stem_actions_bar.dart';
import '../features/mixer/stem_set_picker.dart';
import '../features/player/player_controller.dart';
import '../features/player/player_screen.dart';
import '../features/waveform/waveform_view_controller.dart';
import '../shared_ui/app_icon_button.dart';
import '../shared_ui/server_reason.dart';
import '../shared_ui/state_notice.dart';
import 'loaded_recording.dart';
import 'mixer_board.dart';
import 'stem_set_actions.dart';

/// The second view of the loaded recording: the player's transport and panels
/// with the stem lanes in place of the waveform and the markers.
class MixerPage extends ConsumerWidget {
  const MixerPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final recordingId = ref.watch(
      playerStateProvider.select((state) => state.value?.recordingId),
    );
    final recording = ref.watch(loadedRecordingProvider);
    final connected =
        ref.watch(connectionStatusProvider).value == ConnectionStatus.connected;
    // The library is read again when the server reports a change, which may be
    // new stems from another client.
    if (recordingId != null) {
      ref.listen(libraryProvider, (_, _) {
        ref.invalidate(stemsProvider(recordingId));
      });
    }
    return PlayerScreen(
      onChooseRecording: () => ref
          .read(shellDestinationProvider.notifier)
          .select(ShellDestination.library),
      title: recording?.title,
      analysis: ref.watch(loadedAnalysisProvider),
      lengthMs: recording?.lengthMs,
      connected: connected,
      waveform: recordingId == null
          ? null
          : _MixerArea(
              recordingId: recordingId,
              title: recording?.title ?? 'recording-$recordingId',
              lengthMs: recording?.lengthMs,
              connected: connected,
            ),
    );
  }
}

class _MixerArea extends ConsumerWidget {
  const _MixerArea({
    required this.recordingId,
    required this.title,
    required this.lengthMs,
    required this.connected,
  });

  final int recordingId;
  final String title;
  final int? lengthMs;
  final bool connected;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(playerStateProvider).value;
    final overview = ref.watch(stemsProvider(recordingId)).value;
    final canManage = ref.watch(rightsProvider)?.canManageRecordings ?? false;
    final canControl = ref.watch(rightsProvider)?.canControlPlayback ?? false;
    final busy = ref.watch(stemSetBusyProvider);
    if (state == null || overview == null) {
      return StateNotice(message: MixerTexts.loading, busy: true);
    }
    final actions = StemSetActions(context, ref, recordingId);
    final player = ref.read(playerControllerProvider);
    final stems = overview.stems;
    final hasStems = stems.isNotEmpty;
    final percent = overview.analysingPercent;
    final failure = overview.failureCode;
    final view = ref.read(waveformViewProvider(recordingId).notifier);
    final length = lengthMs;
    Future<void> zoom(double factor) async {
      final position = await ref.read(playerSourceProvider).position();
      view.zoomByButton(factor, playheadMs: position ?? 0);
    }

    final canAnalyse =
        canManage && connected && overview.analysisAvailable && percent == null;
    final bar = StemActionsBar(
      stemsEnabled: state.stemsEnabled,
      onEnabled: hasStems && connected ? player.setStemsEnabled : null,
      onSplit: canAnalyse ? actions.split : null,
      existingSets: overview.sets.toSet(),
      setPicker: switch (overview.selectedSet) {
        final selected? => StemSetPicker(
          sets: overview.sets,
          selected: selected,
          onSelect: canControl && connected && !busy ? actions.select : null,
          onDelete: canManage && connected && !busy && percent == null
              ? () => actions.delete(selected)
              : null,
        ),
        null => null,
      },
      onCreateRegions: canAnalyse && hasStems ? actions.createRegions : null,
      onImport: canManage && connected
          ? () => actions.import(replacing: hasStems)
          : null,
      onExport: hasStems && connected ? () => actions.export(title) : null,
      busy: busy,
      zoom: hasStems
          ? [
              AppIconButton(
                icon: Icons.zoom_in,
                tooltip: MixerTexts.zoomIn,
                onPressed: () => zoom(2),
              ),
              AppIconButton(
                icon: Icons.zoom_out,
                tooltip: MixerTexts.zoomOut,
                onPressed: () => zoom(0.5),
              ),
              AppIconButton(
                icon: Icons.fit_screen,
                tooltip: MixerTexts.fit,
                onPressed: view.fit,
              ),
            ]
          : const [],
    );
    return LayoutBuilder(
      builder: (context, bounds) {
        final board = !hasStems || length == null
            ? StateNotice(message: MixerTexts.none, icon: Icons.graphic_eq)
            : MixerBoard(
                recordingId: recordingId,
                durationMs: length,
                stems: stems,
                state: state,
                connected: connected,
              );
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          mainAxisSize: MainAxisSize.min,
          children: [
            bar,
            if (hasStems && !state.stemsEnabled) Text(MixerTexts.originalPlays),
            if (percent != null)
              Semantics(
                liveRegion: true,
                child: Text(
                  overview.creatingRegions
                      ? MixerTexts.creatingRegions(percent)
                      : MixerTexts.splitting(percent),
                ),
              ),
            if (failure != null)
              Text(
                serverReasonText(ServerReason(failure)),
                style: const TextStyle(color: AppColors.error),
              ),
            const SizedBox(height: AppSpace.s2),
            if (bounds.hasBoundedHeight) Flexible(child: board) else board,
          ],
        );
      },
    );
  }
}
