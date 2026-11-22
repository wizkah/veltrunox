import 'package:file_picker/file_picker.dart';

import '../features/library/audio_file_picker.dart';

/// Adapter: the system's file dialog through the `file_picker` package.
class SystemAudioFilePicker implements AudioFilePicker {
  @override
  Future<List<PickedAudio>> pick() async {
    final files = await FilePicker.pickFiles(type: FileType.audio);
    // A browser offers no way to stream a chosen file, so each is read whole,
    // one at a time when it is sent.
    return [
      for (final file in files)
        PickedAudio.lazy(name: file.name, readBytes: file.readAsBytes),
    ];
  }
}
