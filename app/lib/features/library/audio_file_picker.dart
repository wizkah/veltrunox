import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';

/// An audio file the person chose, read completely.
class PickedAudio {
  const PickedAudio({required this.name, required this.bytes});

  final String name;
  final Uint8List bytes;
}

/// Port: lets the person choose an audio file on their device.
abstract interface class AudioFilePicker {
  /// `null` if the person cancelled.
  Future<PickedAudio?> pick();
}

/// Supplies the [AudioFilePicker]; overridden at the application root.
final audioFilePickerProvider = Provider<AudioFilePicker>(
  (ref) =>
      throw UnimplementedError('audioFilePickerProvider must be overridden'),
);
