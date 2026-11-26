import 'dart:typed_data';

import '../features/mixer/mixer_source.dart';
import '../features/mixer/stem_models.dart';
import '../features/waveform/waveform_source.dart';
import '../shared_ui/server_reason.dart';
import 'bridge_waveform_source.dart';
import 'generated/api.dart' as rust;
import 'server_reason_of.dart';

/// Adapter: the stems through the Rust client core.
class BridgeMixerSource implements MixerSource {
  BridgeMixerSource({required this.serverUrl});

  final String serverUrl;

  @override
  Future<StemOverview> stems(int recordingId) async {
    try {
      final found = await rust.recordingStems(
        serverUrl: serverUrl,
        recordingId: BigInt.from(recordingId),
      );
      return StemOverview(
        stems: [
          for (final stem in found.stems) Stem(name: stem.name, key: stem.key),
        ],
        analysisAvailable: found.analysisAvailable,
        analysingPercent: found.analysing ? found.percent : null,
        failureCode: found.failureCode,
      );
    } on Object {
      return const StemOverview();
    }
  }

  @override
  Future<void> analyse(int recordingId) => _reasoned(
    () => rust.analyseStems(
      serverUrl: serverUrl,
      recordingId: BigInt.from(recordingId),
    ),
  );

  @override
  Future<LoadedWaveform?> waveform(int recordingId, int stem) async {
    try {
      return loadedWaveformOf(
        await rust.stemWaveform(
          serverUrl: serverUrl,
          recordingId: BigInt.from(recordingId),
          stem: stem,
        ),
      );
    } on Object {
      return null;
    }
  }

  @override
  Future<ImportedStems> importPackage(Uint8List package) => _reasoned(() async {
    final imported = await rust.importStemPackage(
      serverUrl: serverUrl,
      package: package,
    );
    return ImportedStems(
      title: imported.title,
      stems: imported.stems,
      regionsAdded: imported.regionsAdded,
    );
  });

  @override
  Future<Uint8List> exportPackage(int recordingId) => _reasoned(
    () => rust.exportStemPackage(
      serverUrl: serverUrl,
      recordingId: BigInt.from(recordingId),
    ),
  );

  @override
  Future<void> remove(int recordingId) => _reasoned(
    () => rust.deleteStems(
      serverUrl: serverUrl,
      recordingId: BigInt.from(recordingId),
    ),
  );

  Future<T> _reasoned<T>(Future<T> Function() call) async {
    try {
      return await call();
    } on rust.RecordingActionError catch (error) {
      throw StemFailure(switch (error) {
        rust.RecordingActionError_Refused(:final reason) => serverReasonOf(
          reason,
        ),
        rust.RecordingActionError_Failed(:final detail) => ServerReason.plain(
          detail,
        ),
      });
    } on Object catch (error) {
      throw StemFailure(ServerReason.plain('$error'));
    }
  }
}
