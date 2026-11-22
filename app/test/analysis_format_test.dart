import 'package:flutter_test/flutter_test.dart';
import 'package:veltrunox_app/features/library/analysis_format.dart';
import 'package:veltrunox_app/features/library/analysis_info.dart';

void main() {
  test('key names use sharps and German mode names', () {
    expect(keyName(0, minor: false), 'C-Dur');
    expect(keyName(9, minor: true), 'A-Moll');
    expect(keyName(6, minor: false), 'F#-Dur');
    expect(keyName(11, minor: true), 'B-Moll');
  });

  test('decimals use a comma', () {
    expect(decimal(96.54), '96,5');
    expect(decimal(100), '100,0');
  });

  test('the line says nothing for a recording without analysis', () {
    expect(analysisLine(const AnalysisInfo()), '');
  });

  test('a failure without a reason still says that it failed', () {
    expect(
      analysisLine(const AnalysisInfo(status: AnalysisStatus.failed)),
      'Analyse fehlgeschlagen',
    );
  });
}
