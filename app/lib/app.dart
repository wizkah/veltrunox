import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import 'app_shell/app_shell.dart';
import 'app_shell/shell_destination.dart';
import 'composition/library_page.dart';
import 'composition/player_page.dart';
import 'design/theme/app_theme.dart';
import 'features/connection/connection_banner.dart';
import 'composition/shell_status.dart';
import 'features/access/access_controller.dart';
import 'features/access/access_gate.dart';
import 'features/access/access_texts.dart';
import 'features/player/audio_output_banner.dart';
import 'features/player/player_controller.dart';
import 'features/player/player_notices.dart';

class VeltrunoxApp extends StatelessWidget {
  const VeltrunoxApp({super.key});

  @override
  Widget build(BuildContext context) => MaterialApp(
    title: 'Veltrunox',
    theme: buildAppTheme(),
    home: AccessGate(builder: (_) => const HomePage()),
  );
}

/// Composes shell, library and player; the features do not know each other.
class HomePage extends ConsumerWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final destination = ref.watch(shellDestinationProvider);
    ref.listen(playerRejectionProvider, (_, rejection) {
      if (rejection.value == AccessTexts.signInRequiredReason) {
        ref.read(accessControllerProvider.notifier).sessionExpired();
      }
    });
    return PlayerNotices(
      child: AppShell(
        destination: destination,
        onSelect: ref.read(shellDestinationProvider.notifier).select,
        status: const ShellStatus(),
        body: Column(
          children: [
            const ConnectionBanner(),
            const AudioOutputBanner(),
            Expanded(
              child: switch (destination) {
                ShellDestination.library => const LibraryPage(),
                ShellDestination.player => const PlayerPage(),
              },
            ),
          ],
        ),
      ),
    );
  }
}
