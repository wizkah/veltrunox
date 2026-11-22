import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';

/// An audio file the person chose. Its content is read when it is sent, so
/// several chosen files are not all in memory at once.
class PickedAudio {
  PickedAudio({required this.name, required Uint8List bytes})
    : readBytes = (() async => bytes);

  const PickedAudio.lazy({required this.name, required this.readBytes});

  final String name;
  final Future<Uint8List> Function() readBytes;
}

/// Port: lets the person choose audio files on their device.
abstract interface class AudioFilePicker {
  /// The chosen files in the order of choosing; empty if the person cancelled.
  Future<List<PickedAudio>> pick();
}

/// Supplies the [AudioFilePicker]; overridden at the application root.
final audioFilePickerProvider = Provider<AudioFilePicker>(
  (ref) =>
      throw UnimplementedError('audioFilePickerProvider must be overridden'),
);
