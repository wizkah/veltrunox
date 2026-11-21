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
  Future<List<RecordingSummary>> build() =>
      ref.read(recordingSourceProvider).listRecordings();

  Future<void> reload() async {
    state = const AsyncLoading();
    state = await AsyncValue.guard(
      () => ref.read(recordingSourceProvider).listRecordings(),
    );
  }
}
