import '../access/access_models.dart';
import 'user_models.dart';

/// Port: user administration on the server (admin only). Failures are
/// `AccessException`s.
abstract interface class UserSource {
  Future<UserAdministration> load();

  Future<void> create({
    required String userName,
    required String password,
    required UserRole role,
  });

  /// Changes only what is given.
  Future<void> change(int id, {UserRole? role, bool? active, String? password});

  Future<void> delete(int id);

  Future<void> setGuestAccess({required bool enabled});
}
