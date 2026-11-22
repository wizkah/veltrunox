import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'access_models.dart';
import 'access_source.dart';
import 'access_state.dart';

/// Supplies the [AccessSource]; overridden at the application root.
final accessSourceProvider = Provider<AccessSource>(
  (ref) => throw UnimplementedError('accessSourceProvider must be overridden'),
);

/// Supplies the [TokenStore]; overridden at the application root.
final tokenStoreProvider = Provider<TokenStore>(
  (ref) => throw UnimplementedError('tokenStoreProvider must be overridden'),
);

final accessControllerProvider =
    NotifierProvider<AccessController, AccessState>(AccessController.new);

/// The rights of the person signed in; `null` before and after.
final rightsProvider = Provider<Rights?>(
  (ref) => switch (ref.watch(accessControllerProvider)) {
    AccessSignedIn(:final rights) => rights,
    _ => null,
  },
);

/// Leads from the start of the app to a signed-in person: first start, saved
/// sign-in, password, guest. Starts the real-time session once signed in.
class AccessController extends Notifier<AccessState> {
  AccessSource get _source => ref.read(accessSourceProvider);
  TokenStore get _tokens => ref.read(tokenStoreProvider);

  @override
  AccessState build() {
    Future.microtask(start);
    return const AccessChecking();
  }

  /// Finds out where the person stands; also the retry after a failure.
  Future<void> start() async {
    state = const AccessChecking();
    try {
      final status = await _source.status();
      if (!ref.mounted) return;
      if (status.setupNeeded) {
        state = const AccessSetupNeeded();
        return;
      }
      if (await _resumeSaved()) return;
      if (!ref.mounted) return;
      state = AccessSignedOut(guestEnabled: status.guestEnabled);
    } on AccessException {
      if (ref.mounted) state = const AccessUnreachable();
    }
  }

  Future<bool> _resumeSaved() async {
    final token = await _tokens.read();
    if (token == null) return false;
    try {
      final rights = await _source.resume(token);
      await _enter(rights);
      return true;
    } on AccessException catch (error) {
      // A token the server does not accept is useless; any other failure may pass.
      if (error.failure is WrongCredentials) await _tokens.clear();
      return false;
    }
  }

  Future<void> setUp(String userName, String password) => _attempt(
    () => _source.setUpAdmin(userName, password),
    (failure) => AccessSetupNeeded(failure: failure),
    busy: const AccessSetupNeeded(busy: true),
  );

  Future<void> signIn(String userName, String password) => _attempt(
    () => _source.signIn(userName, password),
    (failure) => AccessSignedOut(guestEnabled: _guestEnabled, failure: failure),
    busy: AccessSignedOut(guestEnabled: _guestEnabled, busy: true),
  );

  Future<void> continueAsGuest() async {
    final busy = AccessSignedOut(guestEnabled: _guestEnabled, busy: true);
    state = busy;
    try {
      await _enter(await _source.continueAsGuest());
    } on AccessException catch (error) {
      if (ref.mounted) {
        state = AccessSignedOut(
          guestEnabled: _guestEnabled,
          failure: error.failure,
        );
      }
    }
  }

  Future<void> signOut() async {
    try {
      await _source.signOut();
    } on AccessException {
      // The token may already be void; this client forgets it either way.
    }
    await _tokens.clear();
    if (ref.mounted) await start();
  }

  /// The server no longer accepts this sign-in: forget it and ask again.
  Future<void> sessionExpired() async {
    if (state is! AccessSignedIn) return;
    await signOut();
    if (!ref.mounted) return;
    if (state case AccessSignedOut(:final guestEnabled)) {
      state = AccessSignedOut(
        guestEnabled: guestEnabled,
        failure: const SessionExpired(),
      );
    }
  }

  bool get _guestEnabled => switch (state) {
    AccessSignedOut(:final guestEnabled) => guestEnabled,
    _ => false,
  };

  Future<void> _attempt(
    Future<SignInOutcome> Function() action,
    AccessState Function(AccessFailure failure) failed, {
    required AccessState busy,
  }) async {
    state = busy;
    try {
      final outcome = await action();
      await _tokens.write(outcome.token);
      await _enter(outcome.rights);
    } on AccessException catch (error) {
      if (ref.mounted) state = failed(error.failure);
    }
  }

  Future<void> _enter(Rights rights) async {
    await _source.startSession();
    if (ref.mounted) state = AccessSignedIn(rights);
  }
}
