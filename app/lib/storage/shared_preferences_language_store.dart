import 'package:shared_preferences/shared_preferences.dart';

import '../features/language/language_store.dart';
import '../shared_ui/app_language.dart';

/// Adapter: keeps the chosen language in the browser's or the device's local
/// storage.
class SharedPreferencesLanguageStore implements LanguageStore {
  static const _key = 'veltrunox.language';

  @override
  Future<AppLanguage?> read() async => AppLanguage.fromCode(
    (await SharedPreferences.getInstance()).getString(_key),
  );

  @override
  Future<void> write(AppLanguage language) async {
    await (await SharedPreferences.getInstance()).setString(
      _key,
      language.code,
    );
  }
}
