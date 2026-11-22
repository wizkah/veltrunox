import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../access/access_models.dart';
import 'user_models.dart';
import 'user_source.dart';

/// Supplies the [UserSource]; overridden at the application root.
final userSourceProvider = Provider<UserSource>(
  (ref) => throw UnimplementedError('userSourceProvider must be overridden'),
);

final usersProvider =
    AsyncNotifierProvider.autoDispose<UsersController, UserAdministration>(
      UsersController.new,
    );

/// The administration of users: each change goes to the server and the list is
/// read again, so it shows what the server holds. Changes report their failure
/// to the caller, which shows it where the person acted.
class UsersController extends AsyncNotifier<UserAdministration> {
  UserSource get _source => ref.read(userSourceProvider);

  @override
  Future<UserAdministration> build() => _source.load();

  Future<AccessFailure?> create({
    required String userName,
    required String password,
    required UserRole role,
  }) => _change(
    () => _source.create(userName: userName, password: password, role: role),
  );

  Future<AccessFailure?> setRole(int id, UserRole role) =>
      _change(() => _source.change(id, role: role));

  Future<AccessFailure?> setActive(int id, {required bool active}) =>
      _change(() => _source.change(id, active: active));

  Future<AccessFailure?> setPassword(int id, String password) =>
      _change(() => _source.change(id, password: password));

  Future<AccessFailure?> delete(int id) => _change(() => _source.delete(id));

  Future<AccessFailure?> setGuestAccess({required bool enabled}) =>
      _change(() => _source.setGuestAccess(enabled: enabled));

  Future<AccessFailure?> setPasswordPolicy(PasswordPolicy policy) =>
      _change(() => _source.setPasswordPolicy(policy));

  Future<AccessFailure?> _change(Future<void> Function() action) async {
    AccessFailure? failure;
    try {
      await action();
    } on AccessException catch (error) {
      failure = error.failure;
    }
    final refreshed = await AsyncValue.guard(_source.load);
    // A failed refresh keeps the list that is shown.
    if (ref.mounted && !refreshed.hasError) state = refreshed;
    return failure;
  }
}
