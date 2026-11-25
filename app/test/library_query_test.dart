import 'package:flutter_test/flutter_test.dart';
import 'package:veltrunox_app/features/library/analysis_info.dart';
import 'package:veltrunox_app/features/library/library_query.dart';
import 'package:veltrunox_app/features/library/recording_summary.dart';

RecordingSummary _done(
  int id,
  String title,
  int tonic,
  bool minor,
  double bpm,
) => RecordingSummary(
  id: id,
  title: title,
  analysis: AnalysisInfo(
    status: AnalysisStatus.done,
    values: AnalysisValues(
      tuningHz: 440,
      keyTonic: tonic,
      keyMinor: minor,
      tempoBpm: bpm,
    ),
  ),
);

RecordingSummary _with(int id, String title, AnalysisStatus status) =>
    RecordingSummary(
      id: id,
      title: title,
      analysis: AnalysisInfo(status: status),
    );

List<String> _titles(List<RecordingSummary> list) =>
    list.map((recording) => recording.title).toList();

void main() {
  final recordings = [
    _done(1, 'Zugabe', 9, true, 90),
    _done(2, 'blues', 4, false, 120),
    _with(3, 'Ballade', AnalysisStatus.running),
    _with(4, 'Kaputt', AnalysisStatus.failed),
    _with(5, 'Neu', AnalysisStatus.none),
  ];

  test('orders by title regardless of case', () {
    expect(_titles(applyQuery(recordings, const LibraryQuery())), [
      'Ballade',
      'blues',
      'Kaputt',
      'Neu',
      'Zugabe',
    ]);
  });

  test('orders by tempo with the unknown ones last', () {
    final shown = applyQuery(
      recordings,
      const LibraryQuery(sort: LibrarySort.tempo),
    );
    expect(_titles(shown).take(2), ['Zugabe', 'blues']);
    expect(_titles(shown).skip(2), ['Ballade', 'Kaputt', 'Neu']);
  });

  test('orders by key with the unknown ones last', () {
    final shown = applyQuery(
      recordings,
      const LibraryQuery(sort: LibrarySort.key),
    );
    expect(_titles(shown).take(2), ['blues', 'Zugabe']);
  });

  test('searches title, key and tempo, every word having to match', () {
    List<String> find(String text) =>
        _titles(applyQuery(recordings, LibraryQuery(search: text)));
    expect(find('BLU'), ['blues']);
    expect(find('a-moll'), ['Zugabe']);
    expect(find('120'), ['blues']);
    expect(find('zug moll'), ['Zugabe']);
    expect(find('zug dur'), isEmpty);
  });

  test('knows whether the user narrowed the list down', () {
    expect(const LibraryQuery().narrows, isFalse);
    expect(const LibraryQuery(search: ' x').narrows, isTrue);
  });
}
