import 'access_models.dart';

/// Port: sign-in with the server. Failures are [AccessException]s.
abstract interface class AccessSource {
  Future<AccessStatus> status();

  /// Creates the first admin and signs in.
  Future<SignInOutcome> setUpAdmin(String userName, String password);

  Future<SignInOutcome> signIn(String userName, String password);

  /// Resumes an earlier sign-in; fails if the token is no longer valid.
  Future<Rights> resume(String token);

  /// Enters without an account; fails while guests are not let in.
  Future<Rights> continueAsGuest();

  Future<void> signOut();

  /// Opens the real-time session with the access chosen last. Call after a
  /// sign-in; replaces an open session.
  Future<void> startSession();
}

/// Port: keeps the sign-in token across restarts.
abstract interface class TokenStore {
  Future<String?> read();

  Future<void> write(String token);

  Future<void> clear();
}
