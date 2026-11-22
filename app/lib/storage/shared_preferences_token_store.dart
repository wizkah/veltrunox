import 'package:shared_preferences/shared_preferences.dart';

import '../features/access/access_source.dart';

/// Adapter: keeps the sign-in token in the browser's or the device's local
/// storage, so the person stays signed in across restarts.
class SharedPreferencesTokenStore implements TokenStore {
  static const _key = 'veltrunox.sign_in_token';

  @override
  Future<String?> read() async =>
      (await SharedPreferences.getInstance()).getString(_key);

  @override
  Future<void> write(String token) async {
    await (await SharedPreferences.getInstance()).setString(_key, token);
  }

  @override
  Future<void> clear() async {
    await (await SharedPreferences.getInstance()).remove(_key);
  }
}
