import '../features/exchange/exchange_models.dart';
import '../features/exchange/exchange_source.dart';
import 'bridge_access_source.dart';
import 'generated/api.dart' as rust;
import 'server_reason_of.dart';

/// Adapter: exports and imports bookmarks through the Rust client core.
class BridgeExchangeSource implements ExchangeSource {
  BridgeExchangeSource({required this.serverUrl});

  final String serverUrl;

  @override
  Future<String> exportText({
    int? recordingId,
    required ExchangeFormat format,
  }) => guardAccess(
    () => rust.exportBookmarksText(
      serverUrl: serverUrl,
      recordingId: recordingId == null ? null : BigInt.from(recordingId),
      format: _format(format),
    ),
  );

  @override
  Future<ImportReport> importText({
    required String content,
    required ExchangeFormat format,
    required bool dryRun,
    required bool overwrite,
  }) async {
    final report = await guardAccess(
      () => rust.importBookmarksText(
        serverUrl: serverUrl,
        content: content,
        format: _format(format),
        dryRun: dryRun,
        overwrite: overwrite,
      ),
    );
    return ImportReport(
      dryRun: report.dryRun,
      recordings: [for (final found in report.recordings) _recording(found)],
    );
  }

  static rust.ExchangeFormatChoice _format(ExchangeFormat format) =>
      switch (format) {
        ExchangeFormat.json => rust.ExchangeFormatChoice.json,
        ExchangeFormat.csv => rust.ExchangeFormatChoice.csv,
      };

  static RecordingImport _recording(rust.RecordingImportInfo found) =>
      RecordingImport(
        recordingId: found.recordingId.toInt(),
        status: switch (found.status) {
          rust.ImportStatusInfo.ready => ImportStatus.ready,
          rust.ImportStatusInfo.conflict => ImportStatus.conflict,
          rust.ImportStatusInfo.invalid => ImportStatus.invalid,
          rust.ImportStatusInfo.unknownRecording =>
            ImportStatus.unknownRecording,
        },
        added: found.added,
        changed: found.changed,
        unchanged: found.unchanged,
        removed: found.removed,
        issues: [
          for (final issue in found.issues)
            ImportIssue(
              kind: switch (issue.kind) {
                rust.ImportEntryKind.label => ImportEntryKind.label,
                rust.ImportEntryKind.region => ImportEntryKind.region,
              },
              index: issue.index,
              reason: serverReasonOf(issue.reason),
            ),
        ],
      );
}
