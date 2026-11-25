import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import 'access_controller.dart';
import 'access_models.dart';
import 'access_texts.dart';

/// The person's menu in the top bar: who they are, administration for the
/// admin, and signing out. Nothing while nobody is signed in.
class AccountMenu extends ConsumerWidget {
  const AccountMenu({
    required this.onManageUsers,
    required this.onManageTokens,
    super.key,
  });

  final VoidCallback onManageUsers;

  /// Opens the person's API tokens; offered to accounts, not to guests.
  final VoidCallback onManageTokens;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final rights = ref.watch(rightsProvider);
    if (rights == null) return const SizedBox.shrink();
    final controller = ref.read(accessControllerProvider.notifier);
    return PopupMenuButton<_Choice>(
      tooltip: AccessTexts.accountMenu,
      icon: const Icon(Icons.account_circle_outlined),
      onSelected: (choice) => switch (choice) {
        _Choice.users => onManageUsers(),
        _Choice.tokens => onManageTokens(),
        _Choice.signOut => controller.signOut(),
      },
      itemBuilder: (_) => [
        PopupMenuItem<_Choice>(
          enabled: false,
          child: Text(AccessTexts.signedInAs(_name(rights))),
        ),
        if (rights.canManageUsers)
          PopupMenuItem(value: _Choice.users, child: Text(AccessTexts.users)),
        if (!rights.isGuest)
          PopupMenuItem(
            value: _Choice.tokens,
            child: Text(AccessTexts.apiTokens),
          ),
        PopupMenuItem(value: _Choice.signOut, child: Text(AccessTexts.signOut)),
      ],
    );
  }

  static String _name(Rights rights) =>
      rights.userName ?? AccessTexts.guestName;
}

enum _Choice { users, tokens, signOut }
