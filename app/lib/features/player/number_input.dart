/// Reads a positive number typed by the user ("96", "96,5" or "96.5"); `null` if
/// it is not one.
double? parsePositiveNumber(String text) {
  final value = double.tryParse(text.trim().replaceAll(',', '.'));
  if (value == null || !value.isFinite || value <= 0) return null;
  return value;
}
