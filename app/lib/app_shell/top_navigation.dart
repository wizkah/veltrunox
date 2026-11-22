import 'package:material_ui/material_ui.dart';

import '../design/tokens/app_colors.dart';
import '../design/tokens/app_geometry.dart';
import 'brand_mark.dart';
import 'shell_destination.dart';

/// Compact navigation along the top edge (desktop), with the global status
/// area at its end.
class TopNavigation extends StatelessWidget implements PreferredSizeWidget {
  const TopNavigation({
    required this.selected,
    required this.onSelect,
    required this.status,
    super.key,
  });

  final ShellDestination selected;
  final ValueChanged<ShellDestination> onSelect;
  final Widget status;

  @override
  Size get preferredSize => const Size.fromHeight(kToolbarHeight);

  @override
  Widget build(BuildContext context) => AppBar(
    title: Row(
      children: [
        const BrandMark(),
        const SizedBox(width: AppSpace.s6),
        for (final destination in ShellDestination.values)
          Padding(
            padding: const EdgeInsets.only(right: AppSpace.s2),
            child: _Tab(
              destination: destination,
              selected: destination == selected,
              onTap: () => onSelect(destination),
            ),
          ),
      ],
    ),
    actions: [status],
  );
}

class _Tab extends StatelessWidget {
  const _Tab({
    required this.destination,
    required this.selected,
    required this.onTap,
  });

  final ShellDestination destination;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) => TextButton.icon(
    onPressed: onTap,
    icon: Icon(selected ? destination.selectedIcon : destination.icon),
    label: Text(destination.label),
    style: TextButton.styleFrom(
      foregroundColor: selected ? AppColors.action : AppColors.textPrimary,
      backgroundColor: selected ? AppColors.surfaceRaised : null,
    ),
  );
}
