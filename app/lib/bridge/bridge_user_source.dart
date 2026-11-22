import '../features/access/access_models.dart';
import '../features/users/user_models.dart';
import '../features/users/user_source.dart';
import 'bridge_access_source.dart';
import 'generated/api.dart' as rust;

/// Adapter: user administration through the Rust client core.
class BridgeUserSource implements UserSource {
  BridgeUserSource({required this.serverUrl});

  final String serverUrl;

  @override
  Future<UserAdministration> load() => guardAccess(() async {
    final users = await rust.listUsers(serverUrl: serverUrl);
    final status = await rust.accessStatus(serverUrl: serverUrl);
    return UserAdministration(
      users: [
        for (final user in users)
          ManagedUser(
            id: user.id.toInt(),
            userName: user.userName,
            role: BridgeAccessSource.roleFrom(user.role),
            active: user.active,
          ),
      ],
      guestEnabled: status.guestEnabled,
      passwordPolicy: BridgeAccessSource.policyFrom(status.passwordPolicy),
    );
  });

  @override
  Future<void> create({
    required String userName,
    required String password,
    required UserRole role,
  }) => guardAccess(
    () => rust.createUser(
      serverUrl: serverUrl,
      userName: userName,
      password: password,
      role: BridgeAccessSource.roleTo(role),
    ),
  );

  @override
  Future<void> change(
    int id, {
    UserRole? role,
    bool? active,
    String? password,
  }) => guardAccess(
    () => rust.changeUser(
      serverUrl: serverUrl,
      userId: BigInt.from(id),
      role: role == null ? null : BridgeAccessSource.roleTo(role),
      active: active,
      password: password,
    ),
  );

  @override
  Future<void> delete(int id) => guardAccess(
    () => rust.removeUser(serverUrl: serverUrl, userId: BigInt.from(id)),
  );

  @override
  Future<void> setGuestAccess({required bool enabled}) => guardAccess(
    () => rust.switchGuestAccess(serverUrl: serverUrl, enabled: enabled),
  );

  @override
  Future<void> setPasswordPolicy(PasswordPolicy policy) => guardAccess(
    () => rust.switchPasswordPolicy(
      serverUrl: serverUrl,
      policy: BridgeAccessSource.policyTo(policy),
    ),
  );
}
