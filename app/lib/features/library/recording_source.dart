import 'recording_summary.dart';

/// Port: where the library gets its recordings from.
abstract interface class RecordingSource {
  Future<List<RecordingSummary>> listRecordings();
}
