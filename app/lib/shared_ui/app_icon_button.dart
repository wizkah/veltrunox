import 'package:material_ui/material_ui.dart';

import '../design/tokens/app_geometry.dart';

/// An icon button with an accessible name. [prominent] makes it the large
/// primary control (play).
class AppIconButton extends StatelessWidget {
  const AppIconButton({
    required this.tooltip,
    required this.icon,
    required this.onPressed,
    this.selected = false,
    this.prominent = false,
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

  /// Side of the touch area and size of the icon; the defaults are the touch
  /// target and, for [prominent], the primary play size.
  final double? extent;
  final double? iconSize;

  @override
  Widget build(BuildContext context) {
    if (prominent) {
      return IconButton.filled(
        tooltip: tooltip,
        onPressed: onPressed,
        iconSize: iconSize ?? AppSpace.s8,
        constraints: BoxConstraints.tightFor(
          width: extent ?? AppSize.primaryPlay,
          height: extent ?? AppSize.primaryPlay,
        ),
        icon: Icon(icon),
      );
    }
    return IconButton(
      tooltip: tooltip,
      onPressed: onPressed,
      isSelected: selected,
      iconSize: iconSize,
      constraints: extent == null
          ? null
          : BoxConstraints.tightFor(width: extent, height: extent),
      icon: Icon(icon),
    );
  }
}
