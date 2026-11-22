import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'waveform_data.dart';

/// A recording's waveform with the length it covers.
final class LoadedWaveform {
  const LoadedWaveform({required this.data, required this.lengthMs});

  final WaveformData data;
  final int lengthMs;
}

/// Port: where the waveform of a recording comes from. The source keeps what it
/// fetched; asking again is cheap.
abstract interface class WaveformSource {
  /// `null` while the recording has no scan or the server cannot be reached.
  Future<LoadedWaveform?> load(int recordingId);
}

/// Supplies the [WaveformSource]; overridden at the application root.
final waveformSourceProvider = Provider<WaveformSource>(
  (ref) =>
      throw UnimplementedError('waveformSourceProvider must be overridden'),
);

/// The waveform of a recording. Like the length it comes from the scan, so it
/// is only asked for once the recording is `scanned` (no request that is bound
/// to fail); `null` before.
final recordingWaveformProvider =
    FutureProvider.family<LoadedWaveform?, ({int recordingId, bool scanned})>(
      (ref, key) => key.scanned
          ? ref.watch(waveformSourceProvider).load(key.recordingId)
          : Future.value(),
    );
