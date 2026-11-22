import 'package:flutter_test/flutter_test.dart';
import 'package:veltrunox_app/features/waveform/new_region_range.dart';

void main() {
  group('defaultRegionRange', () {
    test('starts at the playhead and ends at the end of the view', () {
      final range = defaultRegionRange(
        positionMs: 10000,
        visibleEndMs: 40000,
        lengthMs: 60000,
      );
      expect((range.startMs, range.endMs), (10000, 40000));
    });

    test('is at least a second long', () {
      final range = defaultRegionRange(
        positionMs: 10000,
        visibleEndMs: 10200,
        lengthMs: 60000,
      );
      expect(range.endMs - range.startMs, 1000);
    });

    test('stays inside the recording', () {
      final range = defaultRegionRange(
        positionMs: 59800,
        visibleEndMs: 90000,
        lengthMs: 60000,
      );
      expect((range.startMs, range.endMs), (59000, 60000));
    });

    test('a playhead before the visible range still starts there', () {
      final range = defaultRegionRange(
        positionMs: 2000,
        visibleEndMs: 30000,
        lengthMs: 60000,
      );
      expect(range.startMs, 2000);
    });
  });
}
