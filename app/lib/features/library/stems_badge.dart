import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import '../../design/tokens/app_typography.dart';
import 'library_texts.dart';

/// Says that a recording has stems.
class StemsBadge extends StatelessWidget {
  const StemsBadge({super.key});

  @override
  Widget build(BuildContext context) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      const Icon(
        Icons.call_split,
        size: AppTypography.sizeSection,
        color: AppColors.accentSoft,
      ),
      const SizedBox(width: AppSpace.s1),
      Text(
        LibraryTexts.stems,
        style: const TextStyle(
          color: AppColors.textSecondary,
          fontSize: AppTypography.sizeSmall,
        ),
      ),
    ],
  );
}
