import '../features/access/access_models.dart';
import '../features/own_password/own_password_source.dart';
import 'bridge_access_source.dart';
import 'generated/api.dart' as rust;

/// Adapter: changing the own password through the Rust client core.
class BridgeOwnPasswordSource implements OwnPasswordSource {
  BridgeOwnPasswordSource({required this.serverUrl});

  final String serverUrl;

  @override
  Future<PasswordPolicy> policy() => guardAccess(() async {
    final status = await rust.accessStatus(serverUrl: serverUrl);
    return BridgeAccessSource.policyFrom(status.passwordPolicy);
  });

  @override
  Future<void> change({required String current, required String next}) =>
      guardAccess(
        () => rust.changeMyPassword(
          serverUrl: serverUrl,
          currentPassword: current,
          newPassword: next,
        ),
      );
}
