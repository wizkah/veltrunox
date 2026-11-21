/// Reads a tempo typed by the user ("96", "96,5" or "96.5"); `null` if it is
/// not a positive number.
double? parseBpm(String text) {
  final value = double.tryParse(text.trim().replaceAll(',', '.'));
  if (value == null || !value.isFinite || value <= 0) return null;
  return value;
}
