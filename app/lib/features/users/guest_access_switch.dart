import 'package:material_ui/material_ui.dart';

import 'users_texts.dart';

/// Switches guest access on or off.
class GuestAccessSwitch extends StatelessWidget {
  const GuestAccessSwitch({
    required this.enabled,
    required this.onChanged,
    super.key,
  });

  final bool enabled;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) => SwitchListTile(
    title: Text(UsersTexts.guestAccess),
    subtitle: Text(UsersTexts.guestAccessHint),
    value: enabled,
    onChanged: onChanged,
  );
}
