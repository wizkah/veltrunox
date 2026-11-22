import 'package:material_ui/material_ui.dart';

import 'users_texts.dart';

/// Switches between the standard and the strict rules for new passwords.
class PasswordPolicySwitch extends StatelessWidget {
  const PasswordPolicySwitch({
    required this.strict,
    required this.onChanged,
    super.key,
  });

  final bool strict;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) => SwitchListTile(
    title: const Text(UsersTexts.strictPasswords),
    subtitle: const Text(UsersTexts.strictPasswordsHint),
    value: strict,
    onChanged: onChanged,
  );
}
