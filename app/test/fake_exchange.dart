import 'package:veltrunox_app/features/access/access_models.dart';
import 'package:veltrunox_app/features/exchange/exchange_models.dart';
import 'package:veltrunox_app/features/exchange/exchange_source.dart';
import 'package:veltrunox_app/features/exchange/file_exchange.dart';

/// A server for the bookmark exchange that the tests steer.
class FakeExchangeSource implements ExchangeSource {
  FakeExchangeSource({this.exported = '{"format_version":1}'});

  String exported;

  /// What the preview and the import answer; by overwrite choice, else [report].
  ImportReport report = const ImportReport(dryRun: true, recordings: []);
  ImportReport? reportWithOverwrite;
  AccessFailure? failure;
  final calls = <String>[];

  void _maybeFail() {
    final error = failure;
    if (error != null) throw AccessException(error);
  }

  @override
  Future<String> exportText({
    int? recordingId,
    required ExchangeFormat format,
  }) async {
    calls.add('export ${recordingId ?? 'all'} ${format.extension}');
    _maybeFail();
    return exported;
  }

  @override
  Future<ImportReport> importText({
    required String content,
    required ExchangeFormat format,
    required bool dryRun,
    required bool overwrite,
  }) async {
    calls.add('import dryRun=$dryRun overwrite=$overwrite ${format.extension}');
    _maybeFail();
    final chosen = overwrite ? reportWithOverwrite ?? report : report;
    return ImportReport(dryRun: dryRun, recordings: chosen.recordings);
  }
}

class FakeFileSaver implements FileSaver {
  final saved = <({String fileName, String content, String mimeType})>[];
  bool cancel = false;
  Object? failure;

  @override
  Future<bool> save({
    required String fileName,
    required String content,
    required String mimeType,
  }) async {
    final error = failure;
    if (error != null) throw error;
    if (cancel) return false;
    saved.add((fileName: fileName, content: content, mimeType: mimeType));
    return true;
  }
}

class FakeTextFilePicker implements TextFilePicker {
  FakeTextFilePicker([this.file]);

  PickedText? file;

  @override
  Future<PickedText?> pick() async => file;
}
