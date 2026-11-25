import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../access/access_models.dart';
import '../access/access_texts.dart';
import 'exchange_models.dart';
import 'exchange_source.dart';
import 'exchange_texts.dart';
import 'file_exchange.dart';

/// What an export covers: the whole library or one recording.
class ExportScope {
  const ExportScope.library() : recordingId = null, title = null;

  const ExportScope.recording({required int id, required String this.title})
    : recordingId = id;

  /// `null` for the whole library.
  final int? recordingId;
  final String? title;
}

/// The name of the exported file: after the recording, or
/// `veltrunox-bookmarks` for the library; characters a file system may refuse
/// are replaced.
String exportFileName(ExportScope scope, ExchangeFormat format) {
  final title = scope.title;
  final stem = title == null
      ? 'veltrunox-bookmarks'
      : '${title.replaceAll(RegExp(r'[\\/:*?"<>|\x00-\x1f]'), '_').trim()}'
            '.bookmarks';
  return '$stem.${format.extension}';
}

sealed class ExportState {
  const ExportState();
}

class ExportIdle extends ExportState {
  const ExportIdle();
}

class ExportRunning extends ExportState {
  const ExportRunning();
}

class ExportSaved extends ExportState {
  const ExportSaved(this.fileName);

  final String fileName;
}

/// The person closed the save dialog without saving.
class ExportCancelled extends ExportState {
  const ExportCancelled();
}

class ExportFailed extends ExportState {
  const ExportFailed(this.message);

  /// Already worded for the person.
  final String message;
}

final exportControllerProvider =
    NotifierProvider<ExportController, ExportState>(ExportController.new);

/// Reads the bookmarks from the server and hands them to the device as a file.
class ExportController extends Notifier<ExportState> {
  @override
  ExportState build() => const ExportIdle();

  /// Exports and returns how it went; the state says the same.
  Future<ExportState> export(ExportScope scope, ExchangeFormat format) async {
    if (state is ExportRunning) return state;
    state = const ExportRunning();
    final fileName = exportFileName(scope, format);
    try {
      final content = await ref
          .read(exchangeSourceProvider)
          .exportText(recordingId: scope.recordingId, format: format);
      final saved = await ref
          .read(fileSaverProvider)
          .save(
            fileName: fileName,
            content: content,
            mimeType: format.mimeType,
          );
      state = saved ? ExportSaved(fileName) : const ExportCancelled();
    } on AccessException catch (error) {
      state = ExportFailed(AccessTexts.failure(error.failure));
    } on Object catch (error) {
      state = ExportFailed('$error');
    }
    return state;
  }
}

/// The notice for a finished export; `null` where nothing needs saying.
String? exportNotice(ExportState state) => switch (state) {
  ExportSaved(:final fileName) => ExchangeTexts.exported(fileName),
  ExportFailed(:final message) => ExchangeTexts.exportFailed(message),
  _ => null,
};
