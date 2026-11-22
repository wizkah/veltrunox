import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import 'access_models.dart';
import 'access_texts.dart';

/// Says in a sentence why signing in did not work; nothing without a failure.
class FailureNotice extends StatelessWidget {
  const FailureNotice({required this.failure, super.key});

  final AccessFailure? failure;

  @override
  Widget build(BuildContext context) {
    final failure = this.failure;
    if (failure == null) return const SizedBox.shrink();
    return Semantics(
      liveRegion: true,
      child: Padding(
        padding: const EdgeInsets.only(bottom: AppSpace.s3),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Icon(Icons.error_outline, color: AppColors.error),
            const SizedBox(width: AppSpace.s2),
            Expanded(child: Text(AccessTexts.failure(failure))),
          ],
        ),
      ),
    );
  }
}
