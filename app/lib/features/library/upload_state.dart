/// Where an upload stands. One batch of files is sent at a time.
sealed class UploadState {
  const UploadState();
}

class UploadIdle extends UploadState {
  const UploadIdle();
}

/// A file is on its way (number [position] of [total], counted from 1). The
/// browser reports no byte count, so no percentage.
class UploadSending extends UploadState {
  const UploadSending(this.fileName, {this.position = 1, this.total = 1});

  final String fileName;
  final int position;
  final int total;
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
