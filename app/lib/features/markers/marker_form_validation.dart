import '../../shared_ui/clock_format.dart';
import 'marker_texts.dart';

/// The outcome of checking one text field: the value or what is wrong.
typedef Checked<T> = ({T? value, String? error});

Checked<String> checkName(String text) {
  final name = text.trim();
  return name.isEmpty
      ? (value: null, error: MarkerTexts.nameMissing)
      : (value: name, error: null);
}

/// A time inside the recording; the length is only checked when it is known.
Checked<int> checkTime(String text, {required int? lengthMs}) {
  final milliseconds = parseClock(text);
  if (milliseconds == null) {
    return (value: null, error: MarkerTexts.timeUnreadable);
  }
  if (lengthMs != null && milliseconds > lengthMs) {
    return (
      value: null,
      error: MarkerTexts.outsideRecording(formatClock(lengthMs)),
    );
  }
  return (value: milliseconds, error: null);
}

/// The end of a region, which must lie after its [startMs].
Checked<int> checkRegionEnd(
  String text, {
  required int? startMs,
  required int? lengthMs,
}) {
  final end = checkTime(text, lengthMs: lengthMs);
  if (end.value != null && startMs != null && end.value! <= startMs) {
    return (value: null, error: MarkerTexts.endNotAfterStart);
  }
  return end;
}
