import 'package:file_picker/file_picker.dart';

import '../features/library/audio_file_picker.dart';

/// Adapter: the system's file dialog through the `file_picker` package.
class SystemAudioFilePicker implements AudioFilePicker {
  @override
  Future<PickedAudio?> pick() async {
    final file = await FilePicker.pickFile(type: FileType.audio);
    if (file == null) return null;
    // A browser offers no way to stream a chosen file, so it is read whole.
    return PickedAudio(name: file.name, bytes: await file.readAsBytes());
  }
}
