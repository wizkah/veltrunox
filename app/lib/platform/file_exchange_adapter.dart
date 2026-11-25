import 'dart:convert';
import 'dart:typed_data';

import 'package:file_picker/file_picker.dart';

import '../features/exchange/file_exchange.dart';

/// Adapter: the system's file dialog to choose a JSON or CSV file.
class SystemTextFilePicker implements TextFilePicker {
  @override
  Future<PickedText?> pick() async {
    final file = await FilePicker.pickFile(
      type: FileType.custom,
      allowedExtensions: const ['json', 'csv'],
    );
    if (file == null) return null;
    final bytes = await file.readAsBytes();
    // A byte order mark of a spreadsheet's export is not part of the text.
    final text = utf8.decode(bytes, allowMalformed: true);
    return PickedText(
      name: file.name,
      content: text.startsWith('\u{FEFF}') ? text.substring(1) : text,
    );
  }
}

/// Adapter: saves a file through the system's save dialog (a download in a
/// browser).
class SystemFileSaver implements FileSaver {
  @override
  Future<bool> save({
    required String fileName,
    required String content,
    required String mimeType,
  }) async {
    final saved = await FilePicker.saveFile(
      fileName: fileName,
      bytes: Uint8List.fromList(utf8.encode(content)),
      mimeType: mimeType,
    );
    return saved != null;
  }
}
