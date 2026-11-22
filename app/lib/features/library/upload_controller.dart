import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'audio_file_picker.dart';
import 'library_controller.dart';
import 'recording_source.dart';
import 'upload_state.dart';

final uploadControllerProvider =
    NotifierProvider<UploadController, UploadState>(UploadController.new);

/// Lets the person choose a file and sends it. The new recording appears in the
/// library through the server's notice; its analysis shows there as well.
class UploadController extends Notifier<UploadState> {
  @override
  UploadState build() => const UploadIdle();

  Future<void> chooseAndUpload() async {
    if (state is UploadSending) return;
    final PickedAudio? picked;
    try {
      picked = await ref.read(audioFilePickerProvider).pick();
    } on Object catch (error) {
      state = UploadFailed(fileName: '', reason: '$error');
      return;
    }
    if (picked == null) return;
    state = UploadSending(picked.name);
    try {
      final uploaded = await ref
          .read(recordingSourceProvider)
          .upload(picked.name, picked.bytes);
      state = UploadDone(title: uploaded.title, existing: uploaded.existing);
      await ref.read(libraryProvider.notifier).refresh();
    } on RecordingActionException catch (error) {
      state = UploadFailed(fileName: picked.name, reason: error.message);
    } on Object catch (error) {
      state = UploadFailed(fileName: picked.name, reason: '$error');
    }
  }

  void dismiss() {
    if (state is! UploadSending) state = const UploadIdle();
  }
}
