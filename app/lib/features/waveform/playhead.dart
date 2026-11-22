import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import '../../design/tokens/app_typography.dart';
import '../../shared_ui/clock_format.dart';
import 'time_viewport.dart';
import 'waveform_geometry.dart';
import 'waveform_texts.dart';

/// The position line, drawn through all layers ([height]). Only the part from
/// [hitTop] for [hitHeight] takes touches, so labels and regions stay
/// tappable; a drag that starts beside it belongs to the waveform behind. The
/// owner decides what the drag means and which position [shownMs] is shown.
class Playhead extends StatelessWidget {
  const Playhead({
    required this.shownMs,
    required this.viewport,
    required this.height,
    required this.hitTop,
    required this.hitHeight,
    required this.dragging,
    required this.onDragStart,
    required this.onDragUpdate,
    required this.onDragEnd,
    super.key,
  });

  final double shownMs;
  final TimeViewport viewport;
  final double height;
  final double hitTop;
  final double hitHeight;
  final bool dragging;
  final VoidCallback onDragStart;

  /// The handle moved sideways by this many pixels.
  final ValueChanged<double> onDragUpdate;
  final VoidCallback onDragEnd;

  @override
  Widget build(BuildContext context) {
    final x = viewport.timeToX(shownMs);
    final hitWidth = WaveformGeometry.playheadHitWidth;
    if (x < -hitWidth || x > viewport.widthPx + hitWidth) {
      return const SizedBox.shrink();
    }
    return Positioned(
      left: x - hitWidth / 2,
      top: 0,
      width: hitWidth,
      height: height,
      child: Stack(
        children: [
          Positioned.fill(
            child: IgnorePointer(
              child: Align(
                alignment: Alignment.topCenter,
                child: Container(
                  width: dragging ? 4 : 2,
                  margin: const EdgeInsets.only(
                    top: WaveformGeometry.timeRowHeight,
                  ),
                  color: AppColors.action,
                ),
              ),
            ),
          ),
          Positioned(
            top: hitTop,
            height: hitHeight,
            left: 0,
            right: 0,
            child: Semantics(
              label: WaveformTexts.playhead,
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onHorizontalDragStart: (_) => onDragStart(),
                onHorizontalDragUpdate: (details) =>
                    onDragUpdate(details.delta.dx),
                onHorizontalDragEnd: (_) => onDragEnd(),
                onHorizontalDragCancel: onDragEnd,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// The exact time of the playhead in a chip above the ruler; it stays inside
/// the waveform's width.
class PlayheadTime extends StatelessWidget {
  const PlayheadTime({
    required this.shownMs,
    required this.viewport,
    super.key,
  });

  final double shownMs;
  final TimeViewport viewport;

  static const _chipWidth = 72.0;

  @override
  Widget build(BuildContext context) {
    final x = viewport.timeToX(shownMs);
    if (x < 0 || x > viewport.widthPx) return const SizedBox.shrink();
    final left = (x - _chipWidth / 2).clamp(
      0.0,
      (viewport.widthPx - _chipWidth).clamp(0.0, double.infinity),
    );
    return Positioned(
      left: left,
      top: 0,
      width: _chipWidth,
      height: WaveformGeometry.timeRowHeight,
      child: ExcludeSemantics(
        child: Container(
          alignment: Alignment.center,
          decoration: BoxDecoration(
            color: AppColors.action,
            borderRadius: BorderRadius.circular(AppRadius.small),
          ),
          child: Text(
            formatClockTenths(shownMs.round()),
            maxLines: 1,
            style: const TextStyle(
              color: AppColors.onAction,
              fontSize: AppTypography.sizeSmall,
              fontWeight: AppTypography.semibold,
            ),
          ),
        ),
      ),
    );
  }
}
