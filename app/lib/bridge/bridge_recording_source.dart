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
  Future<void> reanalyse(int recordingId) => rust.reanalyseRecording(
    serverUrl: serverUrl,
    recordingId: BigInt.from(recordingId),
  );

  static RecordingSummary _summary(rust.RecordingItem item) => RecordingSummary(
    id: item.id.toInt(),
    title: item.title,
    analysis: AnalysisInfo(
      status: switch (item.analysisState) {
        rust.AnalysisState.none => AnalysisStatus.none,
        rust.AnalysisState.queued => AnalysisStatus.queued,
        rust.AnalysisState.running => AnalysisStatus.running,
        rust.AnalysisState.failed => AnalysisStatus.failed,
        rust.AnalysisState.done => AnalysisStatus.done,
      },
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
