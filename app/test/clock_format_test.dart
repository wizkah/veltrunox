import 'package:flutter_test/flutter_test.dart';
import 'package:veltrunox_app/features/player/position_display.dart';
import 'package:veltrunox_app/shared_ui/app_language.dart';
import 'package:veltrunox_app/shared_ui/clock_format.dart';

void main() {
  test('formats minutes and seconds, and hours from an hour on', () {
    expect(formatClock(0), '0:00');
    expect(formatClock(67900), '1:07');
    expect(formatClock(3723000), '1:02:03');
  });

  test('keeps fractions of a second where there are some', () {
    expect(formatClockExact(67000), '1:07');
    expect(formatClockExact(67500), '1:07,5');
    expect(formatClockExact(67050), '1:07,05');
  });

  test('writes fractions with a decimal point in English', () {
    AppLanguage.current = AppLanguage.en;
    addTearDown(() => AppLanguage.current = AppLanguage.de);

    expect(formatClockExact(67050), '1:07.05');
    expect(formatClockTenths(67500), '1:07.5');
    expect(formatPosition(67500), '1:07.5');
  });

  test('reads the usual spellings of a time', () {
    expect(parseClock('1:07'), 67000);
    expect(parseClock(' 1:02:03 '), 3723000);
    expect(parseClock('75'), 75000);
    expect(parseClock('1:07,5'), 67500);
    expect(parseClock('1:07.25'), 67250);
    expect(parseClock('0:00'), 0);
  });

  test('refuses what is no time', () {
    for (final text in ['', 'abc', '1:75', '1:61:00', '-3', '1:', ':30']) {
      expect(parseClock(text), isNull, reason: text);
    }
  });

  test('what is shown for editing reads back the same', () {
    for (final milliseconds in [0, 999, 1000, 67500, 3723456]) {
      expect(parseClock(formatClockExact(milliseconds)), milliseconds);
    }
  });
}
