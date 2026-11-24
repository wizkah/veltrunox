import 'package:material_ui/material_ui.dart';

import '../design/tokens/app_colors.dart';
import '../design/tokens/app_geometry.dart';
import 'frame_decoration.dart';

/// An icon button with an accessible name. [prominent] makes it the large
/// primary control (play).
class AppIconButton extends StatelessWidget {
  const AppIconButton({
    required this.tooltip,
    required this.icon,
    required this.onPressed,
    this.selected = false,
    this.prominent = false,
    this.framed = false,
    this.extent,
    this.iconSize,
    super.key,
  });

  /// The accessible name; also shown as tooltip.
  final String tooltip;
  final IconData icon;

  /// `null` disables the button.
  final VoidCallback? onPressed;
  final bool selected;
  final bool prominent;

  /// Draws the same border and faint glow as the panels.
  final bool framed;

  /// Side of the touch area and size of the icon; the defaults are the touch
  /// target and, for [prominent], the primary play size.
  final double? extent;
  final double? iconSize;

  @override
  Widget build(BuildContext context) {
    if (prominent) {
      final filled = IconButton.filled(
        tooltip: tooltip,
        onPressed: onPressed,
        iconSize: iconSize ?? AppSpace.s8,
        padding: EdgeInsets.zero,
        constraints: BoxConstraints.tightFor(
          width: extent ?? AppSize.primaryPlay,
          height: extent ?? AppSize.primaryPlay,
        ),
        icon: Icon(icon),
      );
      // The primary control glows in the action colour while it can be used.
      return onPressed == null
          ? filled
          : DecoratedBox(
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                boxShadow: [
                  BoxShadow(
                    color: AppColors.action.withValues(alpha: 0.4),
                    blurRadius: AppSpace.s6,
                  ),
                ],
              ),
              child: filled,
            );
    }
    final button = IconButton(
      tooltip: tooltip,
      onPressed: onPressed,
      isSelected: selected,
      style: framed
          ? IconButton.styleFrom(
              padding: EdgeInsets.zero,
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(AppRadius.medium),
              ),
            )
          : null,
      iconSize: iconSize,
      constraints: extent == null
          ? null
          : BoxConstraints.tightFor(width: extent, height: extent),
      icon: Icon(icon),
    );
    return framed
        ? DecoratedBox(decoration: frameDecoration(), child: button)
        : button;
  }
}
