import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';

/// The bordered, transparent surface the player's panels (tempo, pitch, volume) stand on.
class PanelSurface extends StatelessWidget {
  const PanelSurface({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) => DecoratedBox(
    decoration: BoxDecoration(
      border: Border.all(color: AppColors.border, width: AppSize.borderWidth),
      borderRadius: BorderRadius.circular(AppRadius.medium),
      // A faint teal glow instead of a filled background.
      boxShadow: [
        BoxShadow(
          color: AppColors.accent.withValues(alpha: 0.10),
          blurRadius: AppSpace.s3,
        ),
      ],
    ),
    child: Padding(padding: const EdgeInsets.all(AppSpace.s4), child: child),
  );
}
