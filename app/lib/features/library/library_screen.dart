import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import 'library_controller.dart';
import 'library_failure.dart';
import 'library_texts.dart';
import 'recording_list.dart';

class LibraryScreen extends ConsumerWidget {
  const LibraryScreen({this.appBarActions = const [], super.key});

  final List<Widget> appBarActions;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final library = ref.watch(libraryProvider);
    return Scaffold(
      appBar: AppBar(
        title: const Text(LibraryTexts.title),
        actions: appBarActions,
      ),
      body: library.when(
        data: (recordings) => RecordingList(recordings: recordings),
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (_, _) => LibraryFailure(
          onRetry: () => ref.read(libraryProvider.notifier).reload(),
        ),
      ),
    );
  }
}
