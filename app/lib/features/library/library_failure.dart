import 'package:material_ui/material_ui.dart';

import 'library_texts.dart';

class LibraryFailure extends StatelessWidget {
  const LibraryFailure({required this.onRetry, super.key});

  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) => Center(
    child: Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        const Text(LibraryTexts.failure),
        TextButton(onPressed: onRetry, child: const Text(LibraryTexts.retry)),
      ],
    ),
  );
}
