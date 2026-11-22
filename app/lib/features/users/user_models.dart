import '../access/access_models.dart';

/// A user account as the admin sees it.
class ManagedUser {
  const ManagedUser({
    required this.id,
    required this.userName,
    required this.role,
    required this.active,
  });

  final int id;
  final String userName;
  final UserRole role;
  final bool active;
}

/// What the admin sees in the user administration.
class UserAdministration {
  const UserAdministration({
    required this.users,
    required this.guestEnabled,
    this.passwordPolicy = PasswordPolicy.standard,
  });

  final List<ManagedUser> users;
  final bool guestEnabled;
  final PasswordPolicy passwordPolicy;
}
