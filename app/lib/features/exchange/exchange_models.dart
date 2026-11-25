import '../../shared_ui/server_reason.dart';

/// The file formats bookmarks are exchanged in.
enum ExchangeFormat {
  json('json', 'application/json'),
  csv('csv', 'text/csv');

  const ExchangeFormat(this.extension, this.mimeType);

  /// The file name extension, without the dot.
  final String extension;
  final String mimeType;

  /// The format a file name's extension names; `null` for any other.
  static ExchangeFormat? fromFileName(String fileName) {
    final dot = fileName.lastIndexOf('.');
    if (dot < 0) return null;
    final extension = fileName.substring(dot + 1).toLowerCase();
    for (final format in values) {
      if (format.extension == extension) return format;
    }
    return null;
  }
}

/// What became of one recording of an import.
enum ImportStatus {
  /// Applied, or, in a preview, would be applied.
  ready,

  /// An entry's stored original was changed or deleted meanwhile.
  conflict,

  /// Entries are invalid; see the issues.
  invalid,
  unknownRecording,
}

enum ImportEntryKind { label, region }

/// A problem with one entry of an import: the [index]-th label or region of
/// the file's set for the recording, counted from 0.
class ImportIssue {
  const ImportIssue({
    required this.kind,
    required this.index,
    required this.reason,
  });

  final ImportEntryKind kind;
  final int index;
  final ServerReason reason;
}

/// The outcome for one recording; the counts are 0 unless it is [ready].
class RecordingImport {
  const RecordingImport({
    required this.recordingId,
    required this.status,
    this.added = 0,
    this.changed = 0,
    this.unchanged = 0,
    this.removed = 0,
    this.issues = const [],
  });

  final int recordingId;
  final ImportStatus status;
  final int added;
  final int changed;
  final int unchanged;
  final int removed;
  final List<ImportIssue> issues;
}

/// The server's answer to an import, one entry per recording of the file.
class ImportReport {
  const ImportReport({required this.dryRun, required this.recordings});

  final bool dryRun;
  final List<RecordingImport> recordings;

  Iterable<RecordingImport> get _ready =>
      recordings.where((found) => found.status == ImportStatus.ready);

  /// How many recordings are (or would be) applied.
  int get readyCount => _ready.length;

  /// How many recordings stay as they are.
  int get leftAsIsCount => recordings.length - readyCount;

  bool get hasConflicts =>
      recordings.any((found) => found.status == ImportStatus.conflict);

  /// The bookmarks the ready recordings lose.
  int get removedCount => _ready.fold(0, (sum, found) => sum + found.removed);
}
