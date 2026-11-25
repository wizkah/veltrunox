import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../features/access/access_controller.dart';
import '../features/access/account_menu.dart';
import '../features/connection/connection_indicator.dart';
import '../features/language/language_menu.dart';
import '../features/own_password/own_password_dialog.dart';
import '../features/tokens/tokens_dialog.dart';
import '../features/users/users_dialog.dart';

/// The global status area of the shell: the connection, the language and the
/// person's menu.
/// The menu opens the user administration, the API tokens and the change of
/// the own password, which the features do not know about each other.
class ShellStatus extends ConsumerWidget {
  const ShellStatus({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      const ConnectionIndicator(),
      const LanguageMenu(),
      AccountMenu(
        onManageUsers: () => UsersDialog.show(context),
        onManageTokens: () => TokensDialog.show(context),
        onChangePassword: () => _changePassword(context, ref),
      ),
    ],
  );

  /// A changed password ended the sign-in on the server; the app goes back to
  /// the sign-in screen.
  Future<void> _changePassword(BuildContext context, WidgetRef ref) async {
    final changed = await OwnPasswordDialog.show(context);
    if (!changed) return;
    await ref.read(accessControllerProvider.notifier).ownPasswordChanged();
  }
}
