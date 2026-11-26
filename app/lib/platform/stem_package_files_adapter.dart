import 'dart:typed_data';

import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart' show kIsWeb;

import '../features/mixer/mixer_source.dart';

/// Adapter: stem packages through the system's file dialogs.
class SystemStemPackageFiles implements StemPackageFiles {
  @override
  Future<Uint8List?> pick() async {
    final file = await FilePicker.pickFile(
      type: FileType.custom,
      allowedExtensions: const ['zip'],
    );
    return file?.readAsBytes();
  }

  @override
  Future<bool> save(String fileName, Uint8List package) async {
    final saved = await FilePicker.saveFile(
      fileName: fileName,
      bytes: package,
      mimeType: 'application/zip',
    );
    // A browser downloads at once and names no place (see the text exchange).
    return kIsWeb || saved != null;
  }
}
