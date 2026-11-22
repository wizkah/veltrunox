import 'dart:typed_data';

import 'recording_summary.dart';

/// What the server made of an uploaded file.
class UploadedRecording {
  const UploadedRecording({
    required this.id,
    required this.title,
    required this.existing,
  });

  final int id;
  final String title;

  /// The same audio was in the library already; nothing was added.
  final bool existing;
}

/// An action on a recording failed; [message] is the reason for the person.
class RecordingActionException implements Exception {
  const RecordingActionException(this.message);

  final String message;

  @override
  String toString() => message;
}

/// Port: where the library gets its recordings from, and what it may do to them.
abstract interface class RecordingSource {
  Future<List<RecordingSummary>> listRecordings();

  /// Ids of recordings whose analysis status or result changed.
  Stream<int> analysisChanges();

  /// Fires when recordings were added, renamed or removed, by anyone.
  Stream<void> libraryChanges();

  /// Asks the server to analyse the recording again.
  Future<void> reanalyse(int recordingId);

  /// Sends an audio file; throws [RecordingActionException] when it is refused.
  Future<UploadedRecording> upload(String fileName, Uint8List content);

  Future<void> rename(int recordingId, String title);

  Future<void> delete(int recordingId);
}
