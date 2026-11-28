import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../shared_ui/server_reason.dart';
import '../waveform/waveform_source.dart';
import 'stem_models.dart';

/// A stem action the server or the connection refused.
class StemFailure implements Exception {
  const StemFailure(this.reason);

  final ServerReason reason;
}

/// Port: the stems of the recordings on the server. The writing actions throw
/// [StemFailure].
abstract interface class MixerSource {
  /// Without stems if the recording has none or they cannot be read.
  Future<StemOverview> stems(int recordingId);

  /// Starts splitting the recording with [model]; the job shows in [stems].
  Future<void> split(int recordingId, SplitModel model);

  /// Creates regions from the structure of the stems: at once if it is known,
  /// else by a job that shows in [stems].
  Future<void> createRegions(int recordingId);

  /// `null` if it cannot be read.
  Future<LoadedWaveform?> waveform(int recordingId, int stem);

  /// The package says which recording it belongs to.
  Future<ImportedStems> importPackage(Uint8List package);
  Future<Uint8List> exportPackage(int recordingId);
  Future<void> remove(int recordingId);
}

/// Port: stem packages on the person's device.
abstract interface class StemPackageFiles {
  /// The content of the chosen package, or `null` if the person cancelled.
  Future<Uint8List?> pick();

  /// `false` if the person cancelled.
  Future<bool> save(String fileName, Uint8List package);
}

/// Supplies the [MixerSource]; overridden at the application root.
final mixerSourceProvider = Provider<MixerSource>(
  (ref) => throw UnimplementedError('mixerSourceProvider must be overridden'),
);

/// Supplies the [StemPackageFiles]; overridden at the application root.
final stemPackageFilesProvider = Provider<StemPackageFiles>(
  (ref) =>
      throw UnimplementedError('stemPackageFilesProvider must be overridden'),
);

/// The stems of a recording; invalidated when they may have changed.
final stemsProvider = FutureProvider.family<StemOverview, int>(
  (ref, recordingId) => ref.watch(mixerSourceProvider).stems(recordingId),
);

/// The waveform of one stem; the key of the stem makes a replaced one load anew.
final stemWaveformProvider =
    FutureProvider.family<
      LoadedWaveform?,
      ({int recordingId, int stem, String key})
    >(
      (ref, id) =>
          ref.watch(mixerSourceProvider).waveform(id.recordingId, id.stem),
    );
