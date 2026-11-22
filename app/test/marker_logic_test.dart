import 'package:flutter_test/flutter_test.dart';
import 'package:veltrunox_app/features/markers/marker.dart';
import 'package:veltrunox_app/features/markers/marker_form_validation.dart';
import 'package:veltrunox_app/features/markers/marker_order.dart';

LabelMarker _label(int id, int at) =>
    LabelMarker(id: id, version: 1, title: 'L$id', positionMs: at);

RegionMarker _region(int id, int start, int end) =>
    RegionMarker(id: id, version: 1, title: 'R$id', startMs: start, endMs: end);

void main() {
  group('order', () {
    final markers = [
      _region(2, 40000, 92000),
      _label(5, 85000),
      _region(1, 10000, 67000),
      _label(3, 25000),
      _label(4, 40000),
    ];

    test('labels and regions run in one list by start or point in time', () {
      final ids = orderMarkers(
        markers,
        MarkerFilter.all,
      ).map((marker) => marker.title);
      expect(ids, ['R1', 'L3', 'L4', 'R2', 'L5']);
    });

    test(
      'markers at the same time keep a fixed order: labels first, by id',
      () {
        final tied = [_region(1, 1000, 2000), _label(9, 1000), _label(2, 1000)];
        expect(
          orderMarkers(tied, MarkerFilter.all).map((marker) => marker.title),
          ['L2', 'L9', 'R1'],
        );
        expect(
          orderMarkers(tied.reversed, MarkerFilter.all).map((m) => m.title),
          ['L2', 'L9', 'R1'],
        );
      },
    );

    test('the filter keeps one kind', () {
      expect(orderMarkers(markers, MarkerFilter.labels), hasLength(3));
      expect(orderMarkers(markers, MarkerFilter.regions), hasLength(2));
    });
  });

  group('form validation', () {
    test('a name must not be blank and is trimmed', () {
      expect(checkName('  ').error, isNotNull);
      expect(checkName(' Chorus ').value, 'Chorus');
    });

    test('a time must be readable and inside the recording', () {
      expect(checkTime('abc', lengthMs: 60000).error, isNotNull);
      expect(checkTime('1:01', lengthMs: 60000).error, contains('1:00'));
      expect(checkTime('1:00', lengthMs: 60000).value, 60000);
      expect(checkTime('9:00', lengthMs: null).value, 540000);
    });

    test('the end of a region must lie after its start', () {
      expect(
        checkRegionEnd('0:10', startMs: 10000, lengthMs: 60000).error,
        isNotNull,
      );
      expect(
        checkRegionEnd('0:09', startMs: 10000, lengthMs: 60000).error,
        isNotNull,
      );
      expect(
        checkRegionEnd('0:11', startMs: 10000, lengthMs: 60000).value,
        11000,
      );
    });
  });
}
