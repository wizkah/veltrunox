import 'package:material_ui/material_ui.dart';

import 'shell_destination.dart';

/// Navigation along the bottom edge (mobile).
class BottomNavigation extends StatelessWidget {
  const BottomNavigation({
    required this.selected,
    required this.onSelect,
    super.key,
  });

  final ShellDestination selected;
  final ValueChanged<ShellDestination> onSelect;

  @override
  Widget build(BuildContext context) => NavigationBar(
    selectedIndex: selected.index,
    onDestinationSelected: (index) => onSelect(ShellDestination.values[index]),
    destinations: [
      for (final destination in ShellDestination.values)
        NavigationDestination(
          icon: Icon(destination.icon),
          selectedIcon: Icon(destination.selectedIcon),
          label: destination.label,
        ),
    ],
  );
}
