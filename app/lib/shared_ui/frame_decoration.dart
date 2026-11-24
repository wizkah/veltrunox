import 'package:material_ui/material_ui.dart';

import '../design/tokens/app_colors.dart';
import '../design/tokens/app_geometry.dart';

/// The lower end of the panel fill: the surface sinking a little towards the
/// canvas, so a panel reads as a lit plate rather than a flat box.
final _surfaceLow = Color.lerp(AppColors.surface, AppColors.canvas, 0.45)!;

/// The panel border: the border token drawn a little towards the accent.
final _frameBorder = Color.lerp(AppColors.border, AppColors.accent, 0.25)!;

/// The filled, bordered frame with a teal glow that panels and framed buttons
/// share.
BoxDecoration frameDecoration() => BoxDecoration(
  gradient: LinearGradient(
    begin: Alignment.topCenter,
    end: Alignment.bottomCenter,
    colors: [AppColors.surface, _surfaceLow],
  ),
  border: Border.all(color: _frameBorder, width: AppSize.borderWidth),
  borderRadius: BorderRadius.circular(AppRadius.medium),
  boxShadow: [
    BoxShadow(
      color: AppColors.accent.withValues(alpha: 0.14),
      blurRadius: AppSpace.s4,
    ),
  ],
);
