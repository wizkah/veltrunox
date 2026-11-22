import 'package:veltrunox_app/features/waveform/waveform_source.dart';

import 'waveform/waveform_core_test.dart' show loadCleanFixture;

/// Answers with the real peaks of the test recording; counts the requests.
class FakeWaveformSource implements WaveformSource {
  final requests = <int>[];

  /// `null` makes the source answer that there is no waveform.
  bool available = true;

  @override
  Future<LoadedWaveform?> load(int recordingId) async {
    requests.add(recordingId);
    if (!available) return null;
    final fixture = loadCleanFixture();
    return LoadedWaveform(data: fixture.data, lengthMs: fixture.durationMs);
  }
}
