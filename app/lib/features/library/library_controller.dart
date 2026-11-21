import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'recording_source.dart';
import 'recording_summary.dart';

/// Supplies the [RecordingSource]; overridden at the application root.
final recordingSourceProvider = Provider<RecordingSource>(
  (ref) =>
      throw UnimplementedError('recordingSourceProvider must be overridden'),
);

/// Presentation state of the library: loading, loaded or failed.
final libraryProvider =
    AsyncNotifierProvider<LibraryController, List<RecordingSummary>>(
      LibraryController.new,
    );

class LibraryController extends AsyncNotifier<List<RecordingSummary>> {
  @override
  Future<List<RecordingSummary>> build() {
    final source = ref.read(recordingSourceProvider);
    // A finished analysis changes the list; it is loaded again without a spinner.
    final changes = source.analysisChanges().listen((_) => _refresh());
    ref.onDispose(changes.cancel);
    return source.listRecordings();
  }

  Future<void> reanalyse(int recordingId) =>
      ref.read(recordingSourceProvider).reanalyse(recordingId);

  Future<void> _refresh() async {
    final refreshed = await AsyncValue.guard(
      () => ref.read(recordingSourceProvider).listRecordings(),
    );
    // A failed refresh keeps what is shown; the next change tries again.
    if (ref.mounted && !refreshed.hasError) state = refreshed;
  }

  Future<void> reload() async {
    state = const AsyncLoading();
    state = await AsyncValue.guard(
      () => ref.read(recordingSourceProvider).listRecordings(),
    );
  }
}
