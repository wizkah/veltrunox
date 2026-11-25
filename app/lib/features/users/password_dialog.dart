import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import '../access/access_models.dart';
import '../access/access_texts.dart';
import '../access/password_field.dart';
import 'users_texts.dart';

/// Asks for a new password for one user. Resolves to the password, or `null`
/// if the admin cancelled.
class PasswordDialog extends StatefulWidget {
  const PasswordDialog({
    required this.userName,
    required this.policy,
    super.key,
  });

  final String userName;
  final PasswordPolicy policy;

  static Future<String?> show(
    BuildContext context,
    String userName,
    PasswordPolicy policy,
  ) => showDialog<String>(
    context: context,
    builder: (_) => PasswordDialog(userName: userName, policy: policy),
  );

  @override
  State<PasswordDialog> createState() => _PasswordDialogState();
}

class _PasswordDialogState extends State<PasswordDialog> {
  final _password = TextEditingController();
  final _repeat = TextEditingController();
  bool _submitted = false;

  @override
  void dispose() {
    _password.dispose();
    _repeat.dispose();
    super.dispose();
  }

  String? get _error {
    if (!_submitted) return null;
    final problem = widget.policy.check(_password.text);
    return problem == null
        ? null
        : AccessTexts.passwordProblem(widget.policy, problem);
  }

  String? get _repeatError =>
      _submitted && _repeat.text != _password.text ? UsersTexts.mismatch : null;

  void _submit() {
    setState(() => _submitted = true);
    if (_error == null && _repeatError == null) {
      Navigator.of(context).pop(_password.text);
    }
  }

  @override
  Widget build(BuildContext context) => AlertDialog(
    title: Text('${UsersTexts.setPassword}: ${widget.userName}'),
    content: Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        PasswordField(
          controller: _password,
          label: UsersTexts.newPassword,
          helper: AccessTexts.passwordRule(widget.policy),
          error: _error,
          textInputAction: TextInputAction.next,
        ),
        const SizedBox(height: AppSpace.s3),
        PasswordField(
          controller: _repeat,
          label: UsersTexts.repeatPassword,
          error: _repeatError,
          textInputAction: TextInputAction.done,
          onSubmitted: _submit,
        ),
      ],
    ),
    actions: [
      ActionButton(
        label: UsersTexts.cancel,
        kind: ActionKind.secondary,
        onPressed: () => Navigator.of(context).pop(),
      ),
      ActionButton(label: UsersTexts.save, onPressed: _submit),
    ],
  );
}
