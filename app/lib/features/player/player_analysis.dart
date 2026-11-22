/// What the player needs to know about the analysis of the loaded recording.
class PlayerAnalysis {
  const PlayerAnalysis({
    required this.tuningHz,
    required this.tempoBpm,
    this.keyTonic,
    this.keyMinor = false,
  });

  final double tuningHz;
  final double tempoBpm;

  /// Pitch class of the key's tonic (0 = C … 11 = B), if a key was found.
  final int? keyTonic;
  final bool keyMinor;
}
