import 'package:material_ui/material_ui.dart';

import 'bottom_navigation.dart';
import 'build_stamp.dart';
import 'shell_destination.dart';
import 'shell_layout.dart';
import 'brand_mark.dart';
import 'side_navigation.dart';
import 'top_navigation.dart';

/// The frame around the screens: navigation and the global status area. It
/// knows no feature; what it frames is passed in.
class AppShell extends StatelessWidget {
  const AppShell({
    required this.destination,
    required this.onSelect,
    required this.status,
    required this.body,
    super.key,
  });

  final ShellDestination destination;
  final ValueChanged<ShellDestination> onSelect;

  /// Global status (connection), always visible.
  final Widget status;
  final Widget body;

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, constraints) =>
        switch (ShellLayout.forWidth(constraints.maxWidth)) {
          ShellLayout.mobile => _mobile(),
          ShellLayout.tablet => _tablet(),
          ShellLayout.desktop => _desktop(),
        },
  );

  Widget _mobile() => Scaffold(
    // No room for the build time on a phone.
    appBar: AppBar(title: const BrandMark(), actions: [status]),
    body: body,
    bottomNavigationBar: BottomNavigation(
      selected: destination,
      onSelect: onSelect,
    ),
  );

  Widget _tablet() => Scaffold(
    appBar: AppBar(
      title: const BrandMark(),
      actions: [const BuildStamp(), status],
    ),
    body: Row(
      children: [
        SideNavigation(selected: destination, onSelect: onSelect),
        Expanded(child: body),
      ],
    ),
  );

  Widget _desktop() => Scaffold(
    appBar: TopNavigation(
      selected: destination,
      onSelect: onSelect,
      status: status,
    ),
    body: body,
  );
}
