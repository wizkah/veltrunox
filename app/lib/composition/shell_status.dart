import 'package:material_ui/material_ui.dart';

import '../features/access/account_menu.dart';
import '../features/connection/connection_indicator.dart';
import '../features/language/language_menu.dart';
import '../features/users/users_dialog.dart';

/// The global status area of the shell: the connection, the language and the
/// person's menu.
/// The menu opens the user administration, which the two features do not know
/// about each other.
class ShellStatus extends StatelessWidget {
  const ShellStatus({super.key});

  @override
  Widget build(BuildContext context) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      const ConnectionIndicator(),
      const LanguageMenu(),
      AccountMenu(onManageUsers: () => UsersDialog.show(context)),
    ],
  );
}
