import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import 'app_shell/app_shell.dart';
import 'app_shell/shell_destination.dart';
import 'composition/library_page.dart';
import 'composition/player_page.dart';
import 'design/theme/app_theme.dart';
import 'features/connection/connection_banner.dart';
import 'features/language/language_controller.dart';
import 'composition/shell_status.dart';
import 'features/access/access_controller.dart';
import 'features/access/access_gate.dart';
import 'features/player/audio_output_banner.dart';
import 'features/player/player_controller.dart';
import 'features/player/player_notices.dart';

class VeltrunoxApp extends ConsumerWidget {
  const VeltrunoxApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final language = ref.watch(appLanguageProvider);
    // The key builds the whole interface anew after a change of language, so
    // every text is read again; what the screens know sits in providers.
    return MaterialApp(
      key: ValueKey(language),
      title: 'Veltrunox',
      locale: Locale(language.code),
      theme: buildAppTheme(),
      home: AccessGate(builder: (_) => const HomePage()),
    );
  }
}

/// Composes shell, library and player; the features do not know each other.
class HomePage extends ConsumerWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final destination = ref.watch(shellDestinationProvider);
    ref.listen(playerRejectionProvider, (_, rejection) {
      if (rejection.value?.isSignInRequired ?? false) {
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
