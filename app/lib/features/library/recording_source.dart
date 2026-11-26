import 'dart:typed_data';

import 'recording_summary.dart';
import '../../shared_ui/server_reason.dart';

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

/// An action on a recording failed; [reason] is worded by the interface.
class RecordingActionException implements Exception {
  const RecordingActionException(this.reason);

  final ServerReason reason;

  @override
  String toString() => reason.toString();
}

/// Port: where the library gets its recordings from, and what it may do to them.
abstract interface class RecordingSource {
  Future<List<RecordingSummary>> listRecordings();

  /// Ids of recordings whose analysis status or result changed.
  Stream<int> analysisChanges();

  /// Fires when recordings were added, renamed or removed, by anyone.
  Stream<void> libraryChanges();

  /// Sends an audio file; throws [RecordingActionException] when it is refused.
  /// [onProgress] hears how many bytes of [content] the server has received.
  Future<UploadedRecording> upload(
    String fileName,
    Uint8List content, {
    void Function(int receivedBytes)? onProgress,
  });

  Future<void> rename(int recordingId, String title);

  Future<void> delete(int recordingId);
}
