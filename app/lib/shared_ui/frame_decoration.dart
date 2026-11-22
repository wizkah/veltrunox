import 'package:material_ui/material_ui.dart';

import '../design/tokens/app_colors.dart';
import '../design/tokens/app_geometry.dart';

/// The bordered, transparent frame with a faint teal glow that panels and
/// framed buttons share.
BoxDecoration frameDecoration() => BoxDecoration(
  border: Border.all(color: AppColors.border, width: AppSize.borderWidth),
  borderRadius: BorderRadius.circular(AppRadius.medium),
  boxShadow: [
    BoxShadow(
      color: AppColors.accent.withValues(alpha: 0.05),
      blurRadius: AppSpace.s2,
    ),
  ],
);
