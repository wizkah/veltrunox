import 'dart:typed_data';

import '../features/waveform/waveform_data.dart';
import '../features/waveform/waveform_source.dart';
import 'generated/api.dart' as rust;

/// Adapter: the waveform from the Rust client core, which fetches it once and
/// keeps it.
class BridgeWaveformSource implements WaveformSource {
  BridgeWaveformSource({required this.serverUrl});

  final String serverUrl;

  @override
  Future<LoadedWaveform?> load(int recordingId) async {
    final rust.WaveformData found;
    try {
      found = await rust.recordingWaveform(
        serverUrl: serverUrl,
        recordingId: BigInt.from(recordingId),
      );
    } on Object {
      return null;
    }
    if (found.levels.isEmpty) return null;
    return LoadedWaveform(
      lengthMs: found.lengthMs.toInt(),
      data: WaveformData(
        sampleRate: found.sampleRate,
        levels: [
          for (final level in found.levels)
            WaveformLevel(
              framesPerBlock: level.framesPerBlock,
              peaks: Int8List.sublistView(level.peaks),
            ),
        ],
      ),
    );
  }
}
