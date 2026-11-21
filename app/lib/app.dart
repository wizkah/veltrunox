import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import 'features/connection/connection_indicator.dart';
import 'features/library/library_screen.dart';
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
      bottomBar: const PlayerBar(),
    );
  }
}
