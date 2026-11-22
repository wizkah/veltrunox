import 'package:material_ui/material_ui.dart';

import '../../shared_ui/action_button.dart';
import 'users_texts.dart';

/// Asks for a new password for one user. Resolves to the password, or `null`
/// if the admin cancelled.
class PasswordDialog extends StatefulWidget {
  const PasswordDialog({required this.userName, super.key});

  final String userName;

  static Future<String?> show(BuildContext context, String userName) =>
      showDialog<String>(
        context: context,
        builder: (_) => PasswordDialog(userName: userName),
      );

  @override
  State<PasswordDialog> createState() => _PasswordDialogState();
}

class _PasswordDialogState extends State<PasswordDialog> {
  static const _minPasswordLength = 10;

  final _password = TextEditingController();
  bool _submitted = false;

  @override
  void dispose() {
    _password.dispose();
    super.dispose();
  }

  void _submit() {
    setState(() => _submitted = true);
    if (_password.text.length >= _minPasswordLength) {
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
        errorText: _submitted && _password.text.length < _minPasswordLength
            ? UsersTexts.tooShort
            : null,
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
