import '../features/access/access_models.dart';
import '../features/access/access_source.dart';
import 'bridge_session.dart';
import 'generated/api.dart' as rust;
import 'server_reason_of.dart';

/// Adapter: signs in through the Rust client core.
class BridgeAccessSource implements AccessSource {
  BridgeAccessSource({required this.serverUrl, required this.session});

  final String serverUrl;
  final BridgeSession session;

  @override
  Future<AccessStatus> status() => _guard(() async {
    final status = await rust.accessStatus(serverUrl: serverUrl);
    return AccessStatus(
      setupNeeded: status.setupNeeded,
      guestEnabled: status.guestEnabled,
      passwordPolicy: policyFrom(status.passwordPolicy),
    );
  });

  @override
  Future<SignInOutcome> setUpAdmin(String userName, String password) =>
      _guard(() async {
        final result = await rust.setUpAdmin(
          serverUrl: serverUrl,
          userName: userName,
          password: password,
        );
        return _outcome(result);
      });

  @override
  Future<SignInOutcome> signIn(String userName, String password) =>
      _guard(() async {
        final result = await rust.signInWithPassword(
          serverUrl: serverUrl,
          userName: userName,
          password: password,
        );
        return _outcome(result);
      });

  @override
  Future<Rights> resume(String token) => _guard(
    () async =>
        rightsFrom(await rust.resumeSignIn(serverUrl: serverUrl, token: token)),
  );

  @override
  Future<Rights> continueAsGuest() => _guard(
    () async => rightsFrom(await rust.continueAsGuest(serverUrl: serverUrl)),
  );

  @override
  Future<void> signOut() => _guard(() async {
    await session.close();
    await rust.signOutNow(serverUrl: serverUrl);
  });

  @override
  Future<void> startSession() => session.open();

  static SignInOutcome _outcome(rust.SignInResult result) =>
      SignInOutcome(token: result.token, rights: rightsFrom(result.rights));

  static Rights rightsFrom(rust.RightsInfo info) => Rights(
    role: roleFrom(info.role),
    userId: info.userId?.toInt(),
    userName: info.userName,
    canControlPlayback: info.canControlPlayback,
    canEditBookmarks: info.canEditBookmarks,
    canUpload: info.canUpload,
    canManageRecordings: info.canManageRecordings,
    canManageUsers: info.canManageUsers,
  );

  static PasswordPolicy policyFrom(rust.PasswordPolicyChoice policy) =>
      switch (policy) {
        rust.PasswordPolicyChoice.standard => PasswordPolicy.standard,
        rust.PasswordPolicyChoice.strict => PasswordPolicy.strict,
      };

  static rust.PasswordPolicyChoice policyTo(PasswordPolicy policy) =>
      switch (policy) {
        PasswordPolicy.standard => rust.PasswordPolicyChoice.standard,
        PasswordPolicy.strict => rust.PasswordPolicyChoice.strict,
      };

  static UserRole roleFrom(rust.RoleChoice role) => switch (role) {
    rust.RoleChoice.admin => UserRole.admin,
    rust.RoleChoice.user => UserRole.user,
    rust.RoleChoice.guest => UserRole.guest,
  };

  static rust.RoleChoice roleTo(UserRole role) => switch (role) {
    UserRole.admin => rust.RoleChoice.admin,
    UserRole.user => rust.RoleChoice.user,
    UserRole.guest => rust.RoleChoice.guest,
  };
}

/// Runs a bridge call and turns its typed error into an [AccessException].
Future<T> guardAccess<T>(Future<T> Function() call) => _guard(call);

Future<T> _guard<T>(Future<T> Function() call) async {
  try {
    return await call();
  } on rust.AccessError catch (error) {
    throw AccessException(_failure(error));
  }
}

AccessFailure _failure(rust.AccessError error) => switch (error) {
  rust.AccessError_NotSignedIn() => const WrongCredentials(),
  rust.AccessError_NotAllowed() => const NotAllowed(),
  rust.AccessError_TooManyAttempts(:final retryAfterSeconds) => TooManyAttempts(
    retryAfterSeconds.toInt(),
  ),
  rust.AccessError_Refused(:final reason) => Refused(serverReasonOf(reason)),
  rust.AccessError_Unavailable() => const Unavailable(),
};
