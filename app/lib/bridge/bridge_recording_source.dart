import 'dart:typed_data';

import '../features/library/analysis_info.dart';
import '../features/library/recording_source.dart';
import '../features/library/recording_summary.dart';
import 'bridge_session.dart';
import 'generated/api.dart' as rust;

/// Adapter: reads the recording list through the Rust client core.
class BridgeRecordingSource implements RecordingSource {
  BridgeRecordingSource({required this.serverUrl, required this.session});

  final String serverUrl;
  final BridgeSession session;

  @override
  Future<List<RecordingSummary>> listRecordings() async {
    final items = await rust.listRecordings(serverUrl: serverUrl);
    return [for (final item in items) _summary(item)];
  }

  @override
  Stream<int> analysisChanges() => session.analysisChanges();

  @override
  Stream<void> libraryChanges() => session.libraryChanges();

  @override
  Future<void> reanalyse(int recordingId) => rust.reanalyseRecording(
    serverUrl: serverUrl,
    recordingId: BigInt.from(recordingId),
  );

  @override
  Future<UploadedRecording> upload(String fileName, Uint8List content) async {
    final uploaded = await _reasoned(
      () => rust.uploadRecording(
        serverUrl: serverUrl,
        fileName: fileName,
        content: content,
      ),
    );
    return UploadedRecording(
      id: uploaded.id.toInt(),
      title: uploaded.title,
      existing: uploaded.existing,
    );
  }

  @override
  Future<void> rename(int recordingId, String title) => _reasoned(
    () => rust.renameRecordingTo(
      serverUrl: serverUrl,
      recordingId: BigInt.from(recordingId),
      title: title,
    ),
  );

  @override
  Future<void> delete(int recordingId) => _reasoned(
    () => rust.deleteRecordingForGood(
      serverUrl: serverUrl,
      recordingId: BigInt.from(recordingId),
    ),
  );

  /// The bridge reports a failure as text; it becomes the reason shown.
  static Future<T> _reasoned<T>(Future<T> Function() call) async {
    try {
      return await call();
    } on Object catch (error) {
      throw RecordingActionException(error is String ? error : '$error');
    }
  }

  static RecordingSummary _summary(rust.RecordingItem item) => RecordingSummary(
    id: item.id.toInt(),
    title: item.title,
    lengthMs: item.lengthMs?.toInt(),
    analysis: AnalysisInfo(
      status: switch (item.analysisState) {
        rust.AnalysisState.none => AnalysisStatus.none,
        rust.AnalysisState.queued => AnalysisStatus.queued,
        rust.AnalysisState.running => AnalysisStatus.running,
        rust.AnalysisState.failed => AnalysisStatus.failed,
        rust.AnalysisState.done => AnalysisStatus.done,
      },
      percent: item.analysisPercent,
      failure: item.analysisFailure,
      values: item.analysis.let(
        (values) => AnalysisValues(
          tuningHz: values.tuningHz,
          keyTonic: values.keyTonic,
          keyMinor: values.keyMinor,
          tempoBpm: values.tempoBpm,
        ),
      ),
    ),
  );
}

extension _Let<T> on T? {
  R? let<R>(R Function(T) transform) {
    final value = this;
    return value == null ? null : transform(value);
  }
}
