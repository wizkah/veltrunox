import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../app_shell/shell_destination.dart';
import '../features/library/library_screen.dart';
import '../features/player/player_controller.dart';
import 'bookmark_exchange.dart';
import 'recording_preview.dart';

/// The library with its recordings wired to the shared player: opening or
/// playing one loads it and leads to the player.
class LibraryPage extends ConsumerWidget {
  const LibraryPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final loadedId = ref.watch(
      playerStateProvider.select((state) => state.value?.recordingId),
    );
    final player = ref.read(playerControllerProvider);
    final shell = ref.read(shellDestinationProvider.notifier);
    return LibraryScreen(
      loadedId: loadedId,
      exchange: const BookmarkExchangeMenu(),
      previewBuilder: (recording) => recording.lengthMs == null
          ? const SizedBox.shrink()
          : RecordingPreview(
              recordingId: recording.id,
              lengthMs: recording.lengthMs!,
            ),
      onOpenInPlayer: (id) {
        if (id != loadedId) player.select(id);
        shell.select(ShellDestination.player);
      },
      onPlay: (id) {
        if (id == loadedId) {
          player.play();
        } else {
          player.selectAndPlay(id);
        }
        shell.select(ShellDestination.player);
      },
    );
  }
}
