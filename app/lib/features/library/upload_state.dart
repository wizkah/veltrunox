/// Where an upload stands. There is one at a time.
sealed class UploadState {
  const UploadState();
}

class UploadIdle extends UploadState {
  const UploadIdle();
}

/// The file is on its way. The browser reports no byte count, so no percentage.
class UploadSending extends UploadState {
  const UploadSending(this.fileName);

  final String fileName;
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
