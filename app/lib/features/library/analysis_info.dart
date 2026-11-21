/// Where a recording stands in the analysis.
enum AnalysisStatus { none, queued, running, failed, done }

/// Tuning, key and tempo as estimated from the audio.
class AnalysisValues {
  const AnalysisValues({
    required this.tuningHz,
    required this.keyTonic,
    required this.keyMinor,
    required this.tempoBpm,
  });

  final double tuningHz;

  /// Pitch class of the key's tonic: 0 = C … 11 = B.
  final int keyTonic;
  final bool keyMinor;
  final double tempoBpm;
}

/// The analysis side of a library entry.
class AnalysisInfo {
  const AnalysisInfo({
    this.status = AnalysisStatus.none,
    this.failure,
    this.values,
  });

  final AnalysisStatus status;

  /// Why the analysis failed; only set in [AnalysisStatus.failed].
  final String? failure;

  /// Only set once the analysis is [AnalysisStatus.done].
  final AnalysisValues? values;
}
