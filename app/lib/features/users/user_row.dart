import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import '../access/access_models.dart';
import 'user_models.dart';
import 'users_texts.dart';

/// One user with the changes an admin makes: role, active, password, delete.
/// Each choice is handed to the parent as an action; the row keeps no state.
class UserRow extends StatelessWidget {
  const UserRow({
    required this.user,
    required this.onRole,
    required this.onActive,
    required this.onPassword,
    required this.onDelete,
    super.key,
  });

  final ManagedUser user;
  final ValueChanged<UserRole> onRole;
  final ValueChanged<bool> onActive;
  final VoidCallback onPassword;
  final VoidCallback onDelete;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: AppSpace.s2),
    child: Wrap(
      crossAxisAlignment: WrapCrossAlignment.center,
      spacing: AppSpace.s3,
      runSpacing: AppSpace.s2,
      children: [
        SizedBox(
          width: 160,
          child: Text(
            user.userName,
            overflow: TextOverflow.ellipsis,
            style: TextStyle(
              color: user.active
                  ? AppColors.textPrimary
                  : AppColors.textSecondary,
            ),
          ),
        ),
        DropdownButton<UserRole>(
          value: user.role,
          onChanged: (role) {
            if (role != null && role != user.role) onRole(role);
          },
          items: const [
            DropdownMenuItem(
              value: UserRole.user,
              child: Text(UsersTexts.roleUser),
            ),
            DropdownMenuItem(
              value: UserRole.admin,
              child: Text(UsersTexts.roleAdmin),
            ),
          ],
        ),
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Switch(value: user.active, onChanged: onActive),
            Text(user.active ? UsersTexts.active : UsersTexts.inactive),
          ],
        ),
        IconButton(
          tooltip: UsersTexts.setPassword,
          icon: const Icon(Icons.key),
          onPressed: onPassword,
        ),
        IconButton(
          tooltip: UsersTexts.delete,
          icon: const Icon(Icons.delete_outline, color: AppColors.error),
          onPressed: onDelete,
        ),
      ],
    ),
  );
}
