import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../shared_ui/server_reason.dart';
import 'audio_file_picker.dart';
import 'library_controller.dart';
import 'recording_source.dart';
import 'upload_state.dart';

final uploadControllerProvider =
    NotifierProvider<UploadController, UploadState>(UploadController.new);

/// Lets the person choose files and sends them one after the other. The new
/// recordings appear in the library through the server's notice; their
/// analysis shows there as well.
class UploadController extends Notifier<UploadState> {
  @override
  UploadState build() => const UploadIdle();

  Future<void> chooseAndUpload() async {
    if (state is UploadSending) return;
    final List<PickedAudio> picked;
    try {
      picked = await ref.read(audioFilePickerProvider).pick();
    } on Object catch (error) {
      state = UploadFailed(fileName: '', reason: '$error');
      return;
    }
    if (picked.isEmpty) return;
    var uploaded = 0;
    var existing = 0;
    final failures = <UploadFailed>[];
    UploadState last = const UploadIdle();
    for (final (index, file) in picked.indexed) {
      state = UploadSending(
        file.name,
        position: index + 1,
        total: picked.length,
      );
      last = await _send(file);
      switch (last) {
        case UploadDone(existing: true):
          existing++;
        case UploadDone():
          uploaded++;
        case UploadFailed():
          failures.add(last);
        default:
      }
      await ref.read(libraryProvider.notifier).refresh();
    }
    state = picked.length == 1
        ? last
        : UploadBatchDone(
            uploaded: uploaded,
            existing: existing,
            failures: failures,
          );
  }

  Future<UploadState> _send(PickedAudio file) async {
    try {
      final uploaded = await ref
          .read(recordingSourceProvider)
          .upload(file.name, await file.readBytes());
      return UploadDone(title: uploaded.title, existing: uploaded.existing);
    } on RecordingActionException catch (error) {
      return UploadFailed(
        fileName: file.name,
        reason: serverReasonText(error.reason),
      );
    } on Object catch (error) {
      return UploadFailed(fileName: file.name, reason: '$error');
    }
  }

  void dismiss() {
    if (state is! UploadSending) state = const UploadIdle();
  }
}
