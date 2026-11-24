import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/panel_surface.dart';

/// A titled panel that groups related settings, with an optional note below
/// (for example the value the analysis found).
class SettingsPanel extends StatelessWidget {
  const SettingsPanel({
    required this.icon,
    required this.title,
    required this.child,
    this.note,
    super.key,
  });

  /// Drawn in the accent colour.
  final Widget icon;
  final String title;
  final Widget child;
  final String? note;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context).textTheme;
    return PanelSurface(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Row(
            children: [
              IconTheme(
                data: const IconThemeData(color: AppColors.accentSoft),
                child: icon,
              ),
              const SizedBox(width: AppSpace.s2),
              Text(title, style: theme.titleMedium),
            ],
          ),
          const SizedBox(height: AppSpace.s2),
          child,
          if (note != null) ...[
            const SizedBox(height: AppSpace.s2),
            Text(note!, style: theme.bodySmall),
          ],
        ],
      ),
    );
  }
}
