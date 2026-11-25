import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../access/access_models.dart';

/// Port: the signed-in account changes its own password. Failures are
/// `AccessException`s.
abstract interface class OwnPasswordSource {
  /// The rules a new password has to meet.
  Future<PasswordPolicy> policy();

  /// Sets [next] after proving the account with [current]. On success the
  /// server has ended every sign-in of the account, this one too.
  Future<void> change({required String current, required String next});
}

/// Supplies the [OwnPasswordSource]; overridden at the application root.
final ownPasswordSourceProvider = Provider<OwnPasswordSource>(
  (ref) =>
      throw UnimplementedError('ownPasswordSourceProvider must be overridden'),
);

/// The rules for a new password, read when the dialog opens.
final ownPasswordPolicyProvider = FutureProvider.autoDispose<PasswordPolicy>(
  (ref) => ref.read(ownPasswordSourceProvider).policy(),
);
