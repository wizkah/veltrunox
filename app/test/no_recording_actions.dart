import 'dart:typed_data';

import 'package:veltrunox_app/features/library/recording_source.dart';

/// For fakes of [RecordingSource] whose test does not upload, rename or delete.
mixin NoRecordingActions {
  Future<UploadedRecording> upload(String fileName, Uint8List content) =>
      throw UnimplementedError();

  Future<void> rename(int recordingId, String title) =>
      throw UnimplementedError();

  Future<void> delete(int recordingId) => throw UnimplementedError();
}
