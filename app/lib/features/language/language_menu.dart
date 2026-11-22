import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/app_language.dart';
import 'language_controller.dart';
import 'language_texts.dart';

/// Lets the person switch between the languages of the interface.
class LanguageMenu extends ConsumerWidget {
  const LanguageMenu({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final current = ref.watch(appLanguageProvider);
    return PopupMenuButton<AppLanguage>(
      tooltip: LanguageTexts.menu,
      icon: const Icon(Icons.language),
      initialValue: current,
      onSelected: ref.read(appLanguageProvider.notifier).select,
      itemBuilder: (_) => [
        for (final language in AppLanguage.values)
          PopupMenuItem<AppLanguage>(
            value: language,
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                SizedBox(
                  width: AppSpace.s6,
                  child: language == current
                      ? const Icon(Icons.check, size: AppSpace.s4)
                      : null,
                ),
                Text(language.nativeName),
              ],
            ),
          ),
      ],
    );
  }
}
