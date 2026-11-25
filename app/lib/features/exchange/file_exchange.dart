import 'package:flutter_riverpod/flutter_riverpod.dart';

/// A text file the person chose.
class PickedText {
  const PickedText({required this.name, required this.content});

  final String name;
  final String content;
}

/// Port: lets the person choose a JSON or CSV file on their device.
abstract interface class TextFilePicker {
  /// The chosen file, or `null` if the person cancelled.
  Future<PickedText?> pick();
}

/// Port: lets the person keep a text file on their device.
abstract interface class FileSaver {
  /// `true` once the file is saved, `false` if the person cancelled.
  Future<bool> save({
    required String fileName,
    required String content,
    required String mimeType,
  });
}

/// Supplies the [TextFilePicker]; overridden at the application root.
final textFilePickerProvider = Provider<TextFilePicker>(
  (ref) =>
      throw UnimplementedError('textFilePickerProvider must be overridden'),
);

/// Supplies the [FileSaver]; overridden at the application root.
final fileSaverProvider = Provider<FileSaver>(
  (ref) => throw UnimplementedError('fileSaverProvider must be overridden'),
);
