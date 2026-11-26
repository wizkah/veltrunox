import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import 'sharing_controller.dart';
import 'sharing_texts.dart';

/// Opens the settings; nothing where the application carries no server.
class SharingButton extends ConsumerWidget {
  const SharingButton({required this.onPressed, super.key});

  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    if (ref.watch(sharingSourceProvider) == null) {
      return const SizedBox.shrink();
    }
    return IconButton(
      tooltip: SharingTexts.open,
      icon: const Icon(Icons.settings_outlined),
      onPressed: onPressed,
    );
  }
}
