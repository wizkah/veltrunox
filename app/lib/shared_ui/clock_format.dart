/// A time as "1:07", or "1:02:03" from an hour on; seconds are cut, not rounded.
String formatClock(int milliseconds) {
  final total = milliseconds ~/ 1000;
  final seconds = (total % 60).toString().padLeft(2, '0');
  final minutes = total ~/ 60 % 60;
  final hours = total ~/ 3600;
  if (hours == 0) return '${total ~/ 60}:$seconds';
  return '$hours:${minutes.toString().padLeft(2, '0')}:$seconds';
}

final _clockPattern = RegExp(
  r'^(?:(?:(\d+):)?(\d+):)?(\d+)(?:[.,](\d{1,3}))?$',
);

/// Reads "1:07", "1:02:03", "75" or "1:07,5" (a decimal comma or point gives
/// fractions of a second) as milliseconds; `null` if it is no time. In the
/// form with colons, minutes and seconds stay below 60.
int? parseClock(String text) {
  final match = _clockPattern.firstMatch(text.trim());
  if (match == null) return null;
  final hours = int.tryParse(match.group(1) ?? '0') ?? 0;
  final hasMinutes = match.group(2) != null;
  final minutes = int.tryParse(match.group(2) ?? '0') ?? 0;
  final seconds = int.tryParse(match.group(3)!) ?? 0;
  if (hasMinutes && seconds >= 60) return null;
  if (match.group(1) != null && minutes >= 60) return null;
  final fraction = (match.group(4) ?? '').padRight(3, '0');
  return ((hours * 60 + minutes) * 60 + seconds) * 1000 +
      (fraction.isEmpty ? 0 : int.parse(fraction));
}

/// Like [formatClock], but keeps fractions of a second, as "1:07,5"; for
/// fields that are edited, so saving loses nothing.
String formatClockExact(int milliseconds) {
  final fraction = milliseconds % 1000;
  final whole = formatClock(milliseconds);
  if (fraction == 0) return whole;
  final digits = fraction
      .toString()
      .padLeft(3, '0')
      .replaceFirst(RegExp(r'0+$'), '');
  return '$whole,$digits';
}

/// Like [formatClock], with tenths of a second, as "1:07,5"; for a position
/// that is shown while it moves.
String formatClockTenths(int milliseconds) =>
    '${formatClock(milliseconds)},${milliseconds ~/ 100 % 10}';
