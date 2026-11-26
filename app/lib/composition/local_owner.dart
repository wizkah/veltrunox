import '../features/access/access_models.dart';
import '../features/access/access_source.dart';

/// Whoever uses the desktop application is the owner of its server: no first
/// start, no password. The token comes from the server at every start.
class OwnerAccessSource implements AccessSource {
  OwnerAccessSource(this._server);

  final AccessSource _server;

  @override
  Future<AccessStatus> status() async {
    final status = await _server.status();
    return AccessStatus(
      setupNeeded: false,
      guestEnabled: status.guestEnabled,
      passwordPolicy: status.passwordPolicy,
    );
  }

  @override
  Future<SignInOutcome> setUpAdmin(String userName, String password) =>
      _server.setUpAdmin(userName, password);

  @override
  Future<SignInOutcome> signIn(String userName, String password) =>
      _server.signIn(userName, password);

  @override
  Future<Rights> resume(String token) => _server.resume(token);

  @override
  Future<Rights> continueAsGuest() => _server.continueAsGuest();

  @override
  Future<void> signOut() => _server.signOut();

  @override
  Future<void> startSession() => _server.startSession();
}

/// The owner's token lives only as long as the application runs.
class OwnerTokenStore implements TokenStore {
  const OwnerTokenStore(this._token);

  final String _token;

  @override
  Future<String?> read() async => _token;

  @override
  Future<void> write(String token) async {}

  @override
  Future<void> clear() async {}
}
