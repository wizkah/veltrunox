import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import '../../shared_ui/activity_status.dart';
import '../../shared_ui/confirm_dialog.dart';
import '../access/access_models.dart';
import '../access/access_texts.dart';
import 'guest_access_switch.dart';
import 'new_user_form.dart';
import 'password_dialog.dart';
import 'password_policy_switch.dart';
import 'user_models.dart';
import 'user_row.dart';
import 'users_controller.dart';
import 'users_texts.dart';

/// The user administration for the admin: guest access, the users, a new user.
/// Opened from the account menu; it is not a place in the navigation.
class UsersDialog extends ConsumerWidget {
  const UsersDialog({super.key});

  static Future<void> show(BuildContext context) =>
      showDialog<void>(context: context, builder: (_) => const UsersDialog());

  @override
  Widget build(BuildContext context, WidgetRef ref) => Dialog(
    child: ConstrainedBox(
      constraints: const BoxConstraints(maxWidth: 640, maxHeight: 720),
      // Its own messenger and scaffold keep messages inside the dialog.
      child: ScaffoldMessenger(
        child: Scaffold(
          backgroundColor: Colors.transparent,
          body: Padding(
            padding: const EdgeInsets.all(AppSpace.s4),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Row(
                  children: [
                    Expanded(
                      child: Semantics(
                        header: true,
                        child: Text(
                          UsersTexts.title,
                          style: Theme.of(context).textTheme.titleLarge,
                        ),
                      ),
                    ),
                    IconButton(
                      tooltip: UsersTexts.close,
                      icon: const Icon(Icons.close),
                      onPressed: () => Navigator.of(context).pop(),
                    ),
                  ],
                ),
                Flexible(
                  child: switch (ref.watch(usersProvider)) {
                    AsyncData(:final value) => _Content(administration: value),
                    AsyncError() => _Failed(
                      onRetry: () => ref.invalidate(usersProvider),
                    ),
                    _ => const Center(
                      child: ActivityStatus(
                        state: ActivityState.loading,
                        label: UsersTexts.loading,
                      ),
                    ),
                  },
                ),
              ],
            ),
          ),
        ),
      ),
    ),
  );
}

class _Failed extends StatelessWidget {
  const _Failed({required this.onRetry});

  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) => Column(
    mainAxisSize: MainAxisSize.min,
    children: [
      const Text(UsersTexts.failure),
      const SizedBox(height: AppSpace.s3),
      ActionButton(label: UsersTexts.retry, onPressed: onRetry),
    ],
  );
}

class _Content extends ConsumerWidget {
  const _Content({required this.administration});

  final UserAdministration administration;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final controller = ref.read(usersProvider.notifier);
    return ListView(
      shrinkWrap: true,
      children: [
        GuestAccessSwitch(
          enabled: administration.guestEnabled,
          onChanged: (enabled) =>
              _report(context, controller.setGuestAccess(enabled: enabled)),
        ),
        PasswordPolicySwitch(
          strict: administration.passwordPolicy == PasswordPolicy.strict,
          onChanged: (strict) => _report(
            context,
            controller.setPasswordPolicy(
              strict ? PasswordPolicy.strict : PasswordPolicy.standard,
            ),
          ),
        ),
        const Divider(),
        for (final user in administration.users)
          UserRow(
            user: user,
            onRole: (role) =>
                _report(context, controller.setRole(user.id, role)),
            onActive: (active) =>
                _report(context, controller.setActive(user.id, active: active)),
            onPassword: () => _changePassword(context, controller, user),
            onDelete: () => _delete(context, controller, user),
          ),
        const Divider(),
        NewUserForm(
          policy: administration.passwordPolicy,
          onCreate: (name, password, role) =>
              controller.create(userName: name, password: password, role: role),
        ),
      ],
    );
  }

  Future<void> _changePassword(
    BuildContext context,
    UsersController controller,
    ManagedUser user,
  ) async {
    final password = await PasswordDialog.show(
      context,
      user.userName,
      administration.passwordPolicy,
    );
    if (password == null || !context.mounted) return;
    await _report(
      context,
      controller.setPassword(user.id, password),
      success: UsersTexts.passwordChanged,
    );
  }

  Future<void> _delete(
    BuildContext context,
    UsersController controller,
    ManagedUser user,
  ) async {
    final confirmed = await ConfirmDialog.ask(
      context,
      title: UsersTexts.deleteTitle(user.userName),
      message: UsersTexts.deleteMessage,
      confirmLabel: UsersTexts.delete,
      cancelLabel: UsersTexts.cancel,
    );
    if (!confirmed || !context.mounted) return;
    await _report(context, controller.delete(user.id));
  }

  /// A failure becomes a short message at the bottom; success is silent
  /// unless it is something the person cannot see in the list.
  Future<void> _report(
    BuildContext context,
    Future<AccessFailure?> change, {
    String? success,
  }) async {
    final failure = await change;
    if (!context.mounted) return;
    final text = failure == null ? success : AccessTexts.failure(failure);
    if (text == null) return;
    ScaffoldMessenger.of(context)
      ..hideCurrentSnackBar()
      ..showSnackBar(SnackBar(content: Text(text)));
  }
}
