import 'package:material_ui/material_ui.dart';

import '../../shared_ui/action_button.dart';
import '../access/access_models.dart';
import '../access/access_texts.dart';
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
  bool _submitted = false;

  @override
  void dispose() {
    _password.dispose();
    super.dispose();
  }

  String? get _error {
    if (!_submitted) return null;
    final problem = widget.policy.check(_password.text);
    return problem == null
        ? null
        : AccessTexts.passwordProblem(widget.policy, problem);
  }

  void _submit() {
    setState(() => _submitted = true);
    if (widget.policy.check(_password.text) == null) {
      Navigator.of(context).pop(_password.text);
    }
  }

  @override
  Widget build(BuildContext context) => AlertDialog(
    title: Text('${UsersTexts.setPassword}: ${widget.userName}'),
    content: TextField(
      controller: _password,
      autofocus: true,
      obscureText: true,
      autocorrect: false,
      enableSuggestions: false,
      onSubmitted: (_) => _submit(),
      decoration: InputDecoration(
        labelText: UsersTexts.newPassword,
        helperText: AccessTexts.passwordRule(widget.policy),
        helperMaxLines: 2,
        errorMaxLines: 2,
        errorText: _error,
      ),
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
