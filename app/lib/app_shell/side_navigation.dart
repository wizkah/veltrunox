import 'package:material_ui/material_ui.dart';

import 'shell_destination.dart';

/// Navigation along the left edge (tablet).
class SideNavigation extends StatelessWidget {
  const SideNavigation({
    required this.selected,
    required this.onSelect,
    super.key,
  });

  final ShellDestination selected;
  final ValueChanged<ShellDestination> onSelect;

  @override
  Widget build(BuildContext context) => NavigationRail(
    selectedIndex: selected.index,
    labelType: NavigationRailLabelType.all,
    onDestinationSelected: (index) => onSelect(ShellDestination.values[index]),
    destinations: [
      for (final destination in ShellDestination.values)
        NavigationRailDestination(
          icon: Icon(destination.icon),
          selectedIcon: Icon(destination.selectedIcon),
          label: Text(destination.label),
        ),
    ],
  );
}
