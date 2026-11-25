import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'exchange_models.dart';

/// Port: bookmarks in and out of the server as the text of a file. Failures
/// are `AccessException`s.
abstract interface class ExchangeSource {
  /// The bookmarks of one recording, or of all when [recordingId] is `null`.
  Future<String> exportText({int? recordingId, required ExchangeFormat format});

  /// Sends the text of a file. With [dryRun] the server only reports what it
  /// would do; [overwrite] takes entries whose stored original changed
  /// meanwhile.
  Future<ImportReport> importText({
    required String content,
    required ExchangeFormat format,
    required bool dryRun,
    required bool overwrite,
  });
}

/// Supplies the [ExchangeSource]; overridden at the application root.
final exchangeSourceProvider = Provider<ExchangeSource>(
  (ref) =>
      throw UnimplementedError('exchangeSourceProvider must be overridden'),
);
