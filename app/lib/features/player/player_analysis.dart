/// What the player needs to know about the analysis of the loaded recording.
class PlayerAnalysis {
  const PlayerAnalysis({required this.tuningHz, required this.tempoBpm});

  final double tuningHz;
  final double tempoBpm;
}
