import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

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
    theme: ThemeData(colorSchemeSeed: Colors.indigo),
    home: const HomePage(),
  );
}

/// Composes library and player; the two features do not know each other.
class HomePage extends ConsumerWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final selectedId = ref.watch(
      playerStateProvider.select((state) => state.value?.recordingId),
    );
    return LibraryScreen(
      selectedId: selectedId,
      onSelect: ref.read(playerControllerProvider).select,
      appBarActions: const [ConnectionIndicator()],
      bottomBar: PlayerBar(analysis: _analysisOf(ref, selectedId)),
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
    return PlayerAnalysis(tuningHz: values.tuningHz, tempoBpm: values.tempoBpm);
  }
}
