import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import 'library_controller.dart';
import 'library_failure.dart';
import 'library_texts.dart';
import 'recording_list.dart';

class LibraryScreen extends ConsumerWidget {
  const LibraryScreen({
    required this.selectedId,
    required this.onSelect,
    super.key,
  });

  final int? selectedId;
  final ValueChanged<int> onSelect;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final library = ref.watch(libraryProvider);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.all(AppSpace.s4),
          child: Text(
            LibraryTexts.title,
            style: Theme.of(context).textTheme.headlineMedium,
          ),
        ),
        Expanded(
          child: library.when(
            data: (recordings) => RecordingList(
              recordings: recordings,
              selectedId: selectedId,
              onSelect: onSelect,
              onReanalyse: ref.read(libraryProvider.notifier).reanalyse,
            ),
            loading: () => const Center(child: CircularProgressIndicator()),
            error: (_, _) => LibraryFailure(
              onRetry: () => ref.read(libraryProvider.notifier).reload(),
            ),
          ),
        ),
      ],
    );
  }
}
