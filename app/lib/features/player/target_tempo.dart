/// The speed that plays a recording at a wanted tempo.
class TargetTempo {
  const TargetTempo({required this.speed, required this.supported});

  final double speed;

  /// `false` if the speed lies outside the supported range; it must not be set.
  final bool supported;
}
