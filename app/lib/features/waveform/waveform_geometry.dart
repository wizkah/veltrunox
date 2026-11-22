import '../../design/tokens/app_geometry.dart';

/// Heights and hit sizes of the waveform's layers, top to bottom.
abstract final class WaveformGeometry {
  /// The exact time of the playhead, above the ruler.
  static const timeRowHeight = 24.0;
  static const rulerHeight = 28.0;
  static const labelBandHeight = AppSize.touchTarget;
  static const waveformHeight = 128.0;
  static const laneHeight = AppSize.regionLaneHeight;
  static const laneGap = AppSpace.s1;

  /// Width of the touch area of a region handle (the drawn one is narrower).
  static const handleHitWidth = 28.0;

  /// Width of the touch area around the playhead line.
  static const playheadHitWidth = AppSize.touchTarget;

  /// Regions never get narrower than this on screen, so they stay tappable.
  static const regionMinWidth = 24.0;

  /// Lanes shown on a narrow screen before "all regions" is chosen.
  static const compactLanes = 2;

  static double lanesHeight(int lanes) =>
      lanes == 0 ? 0 : lanes * (laneHeight + laneGap);
}
