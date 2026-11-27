/// A language the interface is available in (D-21).
enum AppLanguage {
  de('de', 'Deutsch'),
  en('en', 'English');

  const AppLanguage(this.code, this.nativeName);

  /// Two-letter language code, as stored and as the browser reports it.
  final String code;

  /// The language's name in that language, for the menu.
  final String nativeName;

  /// The language the texts are read in now. The language controller sets it
  /// before the interface is built again; texts are looked up when a widget
  /// builds, so a change only needs the interface to build once more.
  static AppLanguage current = AppLanguage.de;

  /// The language for a code such as `de` or `en-GB`; `null` if there is none.
  static AppLanguage? fromCode(String? code) {
    final language = code?.split(RegExp('[-_]')).first.toLowerCase();
    for (final candidate in values) {
      if (candidate.code == language) return candidate;
    }
    return null;
  }
}

/// The text in the current language: the catalogue of each feature gives its
/// German and its English wording side by side.
String tr(String de, String en) =>
    AppLanguage.current == AppLanguage.de ? de : en;

/// The decimal separator of the current language.
String get decimalSeparator => tr(',', '.');

/// One decimal with the decimal separator of the current language: "96,5" or
/// "96.5".
String decimalText(double value) =>
    value.toStringAsFixed(1).replaceAll('.', decimalSeparator);
