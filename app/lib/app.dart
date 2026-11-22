import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import 'app_shell/app_shell.dart';
import 'app_shell/shell_destination.dart';
import 'design/theme/app_theme.dart';
import 'features/connection/connection_indicator.dart';
import 'features/library/library_controller.dart';
import 'features/library/library_screen.dart';
import 'features/library/recording_summary.dart';
import 'features/player/player_analysis.dart';
import 'features/player/player_bar.dart';
import 'features/player/player_controller.dart';

class VeltrunoxApp extends StatelessWidget {
  const VeltrunoxApp({super.key});

  @override
  Widget build(BuildContext context) => MaterialApp(
    title: 'Veltrunox',
    theme: buildAppTheme(),
    home: const HomePage(),
  );
}

/// Composes shell, library and player; the features do not know each other.
class HomePage extends ConsumerWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final destination = ref.watch(shellDestinationProvider);
    final selectedId = ref.watch(
      playerStateProvider.select((state) => state.value?.recordingId),
    );
    final player = PlayerBar(analysis: _analysisOf(ref, selectedId));
    final inLibrary = destination == ShellDestination.library;
    return AppShell(
      destination: destination,
      onSelect: ref.read(shellDestinationProvider.notifier).select,
      status: const ConnectionIndicator(),
      body: inLibrary
          ? LibraryScreen(
              selectedId: selectedId,
              onSelect: ref.read(playerControllerProvider).select,
            )
          : SingleChildScrollView(child: player),
      persistentBar: inLibrary ? player : null,
    );
  }

  /// The analysis values of the selected recording, once the library has them.
  PlayerAnalysis? _analysisOf(WidgetRef ref, int? selectedId) {
    final recordings = ref.watch(libraryProvider).value;
    final values = recordings
        ?.where((RecordingSummary recording) => recording.id == selectedId)
        .firstOrNull
        ?.analysis
        .values;
    if (values == null) return null;
    return PlayerAnalysis(
      tuningHz: values.tuningHz,
      tempoBpm: values.tempoBpm,
      keyTonic: values.keyTonic,
      keyMinor: values.keyMinor,
    );
  }
}
