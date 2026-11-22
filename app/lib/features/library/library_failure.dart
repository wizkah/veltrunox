import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import 'library_texts.dart';

/// The library could not be loaded: what happened and how to try again.
class LibraryFailure extends StatelessWidget {
  const LibraryFailure({required this.onRetry, super.key});

  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) => Center(
    child: Padding(
      padding: const EdgeInsets.all(AppSpace.s6),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.error_outline, color: AppColors.error),
          const SizedBox(height: AppSpace.s2),
          const Text(LibraryTexts.failure, textAlign: TextAlign.center),
          const SizedBox(height: AppSpace.s3),
          ActionButton(
            label: LibraryTexts.retry,
            kind: ActionKind.secondary,
            onPressed: onRetry,
          ),
        ],
      ),
    ),
  );
}
