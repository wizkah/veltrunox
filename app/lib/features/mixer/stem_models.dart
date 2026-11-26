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
