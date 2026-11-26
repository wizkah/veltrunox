import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import 'access_controller.dart';
import 'access_models.dart';
import 'access_texts.dart';

/// Whether the person is the owner of the application's own server, who has no
/// account to sign out of. Overridden at the application root.
final localOwnerProvider = Provider<bool>((ref) => false);

/// The person's menu in the top bar: who they are, administration for the
/// admin, and signing out. Nothing while nobody is signed in.
class AccountMenu extends ConsumerWidget {
  const AccountMenu({
    required this.onManageUsers,
    required this.onManageTokens,
    required this.onChangePassword,
    super.key,
  });

  final VoidCallback onManageUsers;

  /// Opens the person's API tokens; offered to accounts, not to guests.
  final VoidCallback onManageTokens;

  /// Opens the change of the person's own password; offered to accounts, not
  /// to guests.
  final VoidCallback onChangePassword;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final rights = ref.watch(rightsProvider);
    if (rights == null) return const SizedBox.shrink();
    final owner = ref.watch(localOwnerProvider);
    final hasAccount = !owner && !rights.isGuest;
    final controller = ref.read(accessControllerProvider.notifier);
    return PopupMenuButton<_Choice>(
      tooltip: AccessTexts.accountMenu,
      icon: const Icon(Icons.account_circle_outlined),
      onSelected: (choice) => switch (choice) {
        _Choice.users => onManageUsers(),
        _Choice.tokens => onManageTokens(),
        _Choice.password => onChangePassword(),
        _Choice.signOut => controller.signOut(),
      },
      itemBuilder: (_) => [
        PopupMenuItem<_Choice>(
          enabled: false,
          child: Text(
            owner
                ? AccessTexts.localOwner
                : AccessTexts.signedInAs(_name(rights)),
          ),
        ),
        if (rights.canManageUsers)
          PopupMenuItem(value: _Choice.users, child: Text(AccessTexts.users)),
        if (hasAccount)
          PopupMenuItem(
            value: _Choice.tokens,
            child: Text(AccessTexts.apiTokens),
          ),
        if (hasAccount)
          PopupMenuItem(
            value: _Choice.password,
            child: Text(AccessTexts.changePassword),
          ),
        if (!owner)
          PopupMenuItem(
            value: _Choice.signOut,
            child: Text(AccessTexts.signOut),
          ),
      ],
    );
  }

  static String _name(Rights rights) =>
      rights.userName ?? AccessTexts.guestName;
}

enum _Choice { users, tokens, password, signOut }
