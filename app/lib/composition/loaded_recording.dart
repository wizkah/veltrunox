import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../features/library/library_controller.dart';
import '../features/library/recording_summary.dart';
import '../features/player/player_analysis.dart';
import '../features/player/player_controller.dart';

/// The recording the shared player has loaded, as the library lists it; `null`
/// while none is loaded or the library does not know it (yet).
final loadedRecordingProvider = Provider<RecordingSummary?>((ref) {
  final id = ref.watch(
    playerStateProvider.select((state) => state.value?.recordingId),
  );
  final recordings = ref.watch(libraryProvider).value;
  if (id == null || recordings == null) return null;
  return recordings.where((recording) => recording.id == id).firstOrNull;
});

/// The analysis values of the loaded recording, once its analysis is done.
final loadedAnalysisProvider = Provider<PlayerAnalysis?>((ref) {
  final values = ref.watch(loadedRecordingProvider)?.analysis.values;
  if (values == null) return null;
  return PlayerAnalysis(
    tuningHz: values.tuningHz,
    tempoBpm: values.tempoBpm,
    keyTonic: values.keyTonic,
    keyMinor: values.keyMinor,
  );
});
