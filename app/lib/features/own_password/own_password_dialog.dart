import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import '../access/access_models.dart';
import '../access/access_texts.dart';
import 'own_password_form.dart';
import 'own_password_source.dart';
import 'own_password_texts.dart';

/// The signed-in person changes their own password. Opened from the account
/// menu; resolves to `true` once the password is changed, which also ended the
/// sign-in, and to `false` if the person gave up.
class OwnPasswordDialog extends ConsumerWidget {
  const OwnPasswordDialog({super.key});

  static Future<bool> show(BuildContext context) async =>
      await showDialog<bool>(
        context: context,
        builder: (_) => const OwnPasswordDialog(),
      ) ??
      false;

  @override
  Widget build(BuildContext context, WidgetRef ref) => AlertDialog(
    title: Text(OwnPasswordTexts.title),
    content: SizedBox(
      width: 420,
      child: SingleChildScrollView(
        child: switch (ref.watch(ownPasswordPolicyProvider)) {
          AsyncData(:final value) => OwnPasswordForm(
            policy: value,
            onSubmit: (current, next) => _change(context, ref, current, next),
            onCancel: () => Navigator.of(context).pop(false),
          ),
          AsyncError(:final error) => _Failed(
            failure: error is AccessException
                ? error.failure
                : const Unavailable(),
            onClose: () => Navigator.of(context).pop(false),
          ),
          _ => Text(OwnPasswordTexts.loading),
        },
      ),
    ),
  );

  Future<AccessFailure?> _change(
    BuildContext context,
    WidgetRef ref,
    String current,
    String next,
  ) async {
    try {
      await ref
          .read(ownPasswordSourceProvider)
          .change(current: current, next: next);
    } on AccessException catch (error) {
      return error.failure;
    }
    if (context.mounted) Navigator.of(context).pop(true);
    return null;
  }
}

class _Failed extends StatelessWidget {
  const _Failed({required this.failure, required this.onClose});

  final AccessFailure failure;
  final VoidCallback onClose;

  @override
  Widget build(BuildContext context) => Column(
    mainAxisSize: MainAxisSize.min,
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      Text(AccessTexts.failure(failure)),
      const SizedBox(height: AppSpace.s3),
      ActionButton(label: OwnPasswordTexts.cancel, onPressed: onClose),
    ],
  );
}
