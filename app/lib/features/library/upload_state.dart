/// Where an upload stands. One batch of files is sent at a time.
sealed class UploadState {
  const UploadState();
}

class UploadIdle extends UploadState {
  const UploadIdle();
}

/// A file is on its way (number [position] of [total], counted from 1). The
/// server reports the bytes it has received; until its first report there is
/// no [percent].
class UploadSending extends UploadState {
  const UploadSending(
    this.fileName, {
    this.position = 1,
    this.total = 1,
    this.sentBytes,
    this.fileBytes,
  });

  final String fileName;
  final int position;
  final int total;

  /// Bytes of the file the server has received so far.
  final int? sentBytes;

  /// Size of the file.
  final int? fileBytes;

  /// How much of the file has arrived, 0 to 100; `null` without a report.
  int? get percent {
    final sent = sentBytes;
    final size = fileBytes;
    if (sent == null || size == null || size <= 0) return null;
    return (sent * 100 ~/ size).clamp(0, 100);
  }
}

class UploadDone extends UploadState {
  const UploadDone({required this.title, required this.existing});

  final String title;

  /// The same audio was in the library already.
  final bool existing;
}

class UploadFailed extends UploadState {
  const UploadFailed({required this.fileName, required this.reason});

  final String fileName;
  final String reason;
}

/// Several files were sent; some may have been refused.
class UploadBatchDone extends UploadState {
  const UploadBatchDone({
    required this.uploaded,
    required this.existing,
    required this.failures,
  });

  final int uploaded;

  /// Files whose audio was in the library already.
  final int existing;
  final List<UploadFailed> failures;
}
