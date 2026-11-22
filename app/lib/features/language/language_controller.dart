import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../shared_ui/app_language.dart';
import 'language_store.dart';

/// Supplies the [LanguageStore]; overridden at the application root. Without
/// one the choice is not kept.
final languageStoreProvider = Provider<LanguageStore?>((ref) => null);

/// The language the application starts in (the saved choice, else the
/// device's); overridden at the application root.
final initialLanguageProvider = Provider<AppLanguage>((ref) => AppLanguage.de);

/// The language of the interface.
final appLanguageProvider = NotifierProvider<LanguageController, AppLanguage>(
  LanguageController.new,
);

class LanguageController extends Notifier<AppLanguage> {
  @override
  AppLanguage build() {
    final language = ref.read(initialLanguageProvider);
    AppLanguage.current = language;
    return language;
  }

  Future<void> select(AppLanguage language) async {
    if (language == state) return;
    AppLanguage.current = language;
    state = language;
    try {
      await ref.read(languageStoreProvider)?.write(language);
    } on Object {
      // The choice then only holds until the page is closed.
    }
  }
}
