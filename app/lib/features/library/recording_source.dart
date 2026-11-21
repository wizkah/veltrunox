import 'recording_summary.dart';

/// Port: where the library gets its recordings from.
abstract interface class RecordingSource {
  Future<List<RecordingSummary>> listRecordings();

  /// Ids of recordings whose analysis status or result changed.
  Stream<int> analysisChanges();

  /// Asks the server to analyse the recording again.
  Future<void> reanalyse(int recordingId);
}
