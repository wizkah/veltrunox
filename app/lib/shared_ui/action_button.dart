import 'package:material_ui/material_ui.dart';

import '../design/tokens/app_colors.dart';
import '../design/tokens/app_geometry.dart';

enum ActionKind { primary, secondary, danger }

/// A labelled button for a simple action. Without [onPressed], or while
/// [loading], it cannot be pressed.
class ActionButton extends StatelessWidget {
  const ActionButton({
    required this.label,
    required this.onPressed,
    this.kind = ActionKind.primary,
    this.icon,
    this.loading = false,
    super.key,
  });

  final String label;
  final VoidCallback? onPressed;
  final ActionKind kind;
  final IconData? icon;
  final bool loading;

  @override
  Widget build(BuildContext context) {
    final onTap = loading ? null : onPressed;
    final child = _content();
    return switch (kind) {
      ActionKind.primary => FilledButton(onPressed: onTap, child: child),
      ActionKind.secondary => OutlinedButton(onPressed: onTap, child: child),
      ActionKind.danger => FilledButton(
        onPressed: onTap,
        style: FilledButton.styleFrom(
          backgroundColor: AppColors.error,
          foregroundColor: AppColors.onAction,
        ),
        child: child,
      ),
    };
  }

  Widget _content() => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      if (loading)
        const SizedBox.square(
          dimension: AppSpace.s4,
          child: CircularProgressIndicator(strokeWidth: 2),
        )
      else if (icon != null)
        Icon(icon, size: AppSpace.s4 + AppSpace.s1),
      if (loading || icon != null) const SizedBox(width: AppSpace.s2),
      Flexible(child: Text(label)),
    ],
  );
}
