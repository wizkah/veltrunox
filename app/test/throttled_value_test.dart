import 'package:flutter_test/flutter_test.dart';
import 'package:veltrunox_app/features/player/position_display.dart';
import 'package:veltrunox_app/features/player/throttled_value.dart';

void main() {
  testWidgets('passes the first value at once and the latest one later', (
    tester,
  ) async {
    final sent = <int>[];
    final throttled = ThrottledValue<int>(sent.add);

    for (final value in [1, 2, 3, 4]) {
      throttled.add(value);
    }
    expect(sent, [1]);

    await tester.pump(const Duration(milliseconds: 61));
    expect(sent, [1, 4]);

    throttled.add(5);
    await tester.pump(const Duration(milliseconds: 200));
    await tester.pump(const Duration(milliseconds: 200));
    expect(sent, [1, 4, 5]);
    throttled.dispose();
  });

  testWidgets('a disposed throttle sends nothing more', (tester) async {
    final sent = <int>[];
    ThrottledValue<int>(sent.add)
      ..add(1)
      ..add(2)
      ..dispose();
    await tester.pump(const Duration(seconds: 1));
    expect(sent, [1]);
  });

  test('the next tenth comes sooner at a higher speed', () {
    expect(
      untilNextTenth(1234, 1).inMilliseconds,
      inInclusiveRange(66 + 2, 66 + 4),
    );
    expect(
      untilNextTenth(1234, 2).inMilliseconds,
      inInclusiveRange(33 + 2, 33 + 4),
    );
    expect(untilNextTenth(1200, 1).inMilliseconds, greaterThan(100));
  });
}
