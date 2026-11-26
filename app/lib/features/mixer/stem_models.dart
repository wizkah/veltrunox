/// One stem of a recording.
class Stem {
  const Stem({required this.name, required this.key});

  final String name;

  /// Changes with the content, so a new stem gets a new waveform.
  final String key;

  @override
  bool operator ==(Object other) =>
      other is Stem && other.name == name && other.key == key;

  @override
  int get hashCode => Object.hash(name, key);
}

/// The stems of a recording and what can become of them.
class StemOverview {
  const StemOverview({
    this.stems = const [],
    this.analysisAvailable = false,
    this.analysingPercent,
    this.failureCode,
  });

  final List<Stem> stems;

  /// Whether the server can analyse recordings.
  final bool analysisAvailable;

  /// Percent done while a job waits or runs; `null` without one.
  final int? analysingPercent;

  /// Why the last job failed.
  final String? failureCode;
}

/// What an imported stem package gave.
class ImportedStems {
  const ImportedStems({
    required this.title,
    required this.stems,
    required this.regionsAdded,
  });

  final String title;
  final int stems;
  final int regionsAdded;
}
