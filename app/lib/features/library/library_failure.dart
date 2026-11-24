import 'package:material_ui/material_ui.dart';

import '../../shared_ui/state_notice.dart';
import 'library_texts.dart';

/// The library could not be loaded: what happened and how to try again.
class LibraryFailure extends StatelessWidget {
  const LibraryFailure({required this.onRetry, super.key});

  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) => StateNotice(
    message: LibraryTexts.failure,
    icon: Icons.error_outline,
    tone: NoticeTone.error,
    actionLabel: LibraryTexts.retry,
    actionIcon: Icons.refresh,
    onAction: onRetry,
  );
}
