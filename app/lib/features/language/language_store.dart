import '../../shared_ui/app_language.dart';

/// Port: keeps the chosen language on this device.
abstract interface class LanguageStore {
  /// `null` if the person never chose one.
  Future<AppLanguage?> read();

  Future<void> write(AppLanguage language);
}
