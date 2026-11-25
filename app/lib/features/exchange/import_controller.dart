import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../access/access_models.dart';
import '../access/access_texts.dart';
import 'exchange_models.dart';
import 'exchange_source.dart';
import 'exchange_texts.dart';
import 'file_exchange.dart';

sealed class ImportState {
  const ImportState();
}

class ImportIdle extends ImportState {
  const ImportIdle();
}

/// The file is on its way to the server for a preview.
class ImportReading extends ImportState {
  const ImportReading(this.fileName);

  final String fileName;
}

/// What the import would do; nothing is written yet. While [refreshing], the
/// server is asked again after [overwrite] changed.
class ImportPreview extends ImportState {
  const ImportPreview({
    required this.fileName,
    required this.report,
    required this.overwrite,
    this.refreshing = false,
  });

  final String fileName;
  final ImportReport report;
  final bool overwrite;
  final bool refreshing;

  bool get canApply => !refreshing && report.readyCount > 0;
}

class ImportApplying extends ImportState {
  const ImportApplying(this.fileName);

  final String fileName;
}

class ImportDone extends ImportState {
  const ImportDone({required this.applied, required this.leftAsIs});

  /// Recordings whose bookmarks were replaced.
  final int applied;

  /// Recordings that stayed as they were.
  final int leftAsIs;
}

class ImportFailed extends ImportState {
  const ImportFailed(this.message);

  /// Already worded for the person.
  final String message;
}

final importControllerProvider =
    NotifierProvider<ImportController, ImportState>(ImportController.new);

/// Imports bookmarks from a file in two steps: a preview that writes nothing,
/// and, after the person confirmed, the import itself.
class ImportController extends Notifier<ImportState> {
  PickedText? _file;
  ExchangeFormat? _format;
  Future<void> _loading = Future.value();

  @override
  ImportState build() => const ImportIdle();

  /// Completes when the request that was started last has been answered.
  Future<void> get loading => _loading;

  /// Lets the person choose a file and asks the server for a preview. `false`
  /// if the person cancelled; otherwise the state tells how it goes.
  Future<bool> choose() async {
    if (state is ImportReading || state is ImportApplying) return false;
    final picked = await ref.read(textFilePickerProvider).pick();
    if (picked == null) return false;
    final format = ExchangeFormat.fromFileName(picked.name);
    if (format == null) {
      state = ImportFailed(ExchangeTexts.unsupportedFile);
      return true;
    }
    _file = picked;
    _format = format;
    state = ImportReading(picked.name);
    _loading = _preview(overwrite: false);
    return true;
  }

  /// Asks for the preview again with the overwrite choice changed.
  void setOverwrite({required bool overwrite}) {
    final current = state;
    if (current is! ImportPreview || current.refreshing) return;
    if (current.overwrite == overwrite) return;
    state = ImportPreview(
      fileName: current.fileName,
      report: current.report,
      overwrite: overwrite,
      refreshing: true,
    );
    _loading = _preview(overwrite: overwrite);
  }

  /// Applies the import the preview showed.
  Future<void> apply() async {
    final current = state;
    final file = _file;
    final format = _format;
    if (current is! ImportPreview ||
        !current.canApply ||
        file == null ||
        format == null) {
      return;
    }
    state = ImportApplying(current.fileName);
    _loading = _run(() async {
      final report = await ref
          .read(exchangeSourceProvider)
          .importText(
            content: file.content,
            format: format,
            dryRun: false,
            overwrite: current.overwrite,
          );
      state = ImportDone(
        applied: report.readyCount,
        leftAsIs: report.leftAsIsCount,
      );
    });
    await _loading;
  }

  /// Closes the import; not possible while it is being applied.
  void dismiss() {
    if (state is ImportApplying) return;
    _file = null;
    _format = null;
    state = const ImportIdle();
  }

  Future<void> _preview({required bool overwrite}) => _run(() async {
    final file = _file;
    final format = _format;
    if (file == null || format == null) return;
    final report = await ref
        .read(exchangeSourceProvider)
        .importText(
          content: file.content,
          format: format,
          dryRun: true,
          overwrite: overwrite,
        );
    state = ImportPreview(
      fileName: file.name,
      report: report,
      overwrite: overwrite,
    );
  });

  Future<void> _run(Future<void> Function() action) async {
    try {
      await action();
    } on AccessException catch (error) {
      if (ref.mounted) {
        state = ImportFailed(
          ExchangeTexts.importFailed(AccessTexts.failure(error.failure)),
        );
      }
    } on Object catch (error) {
      if (ref.mounted) {
        state = ImportFailed(ExchangeTexts.importFailed('$error'));
      }
    }
  }
}
