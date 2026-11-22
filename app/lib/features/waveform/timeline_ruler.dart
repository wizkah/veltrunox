import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_typography.dart';
import 'ruler_ticks.dart';
import 'time_viewport.dart';
import 'waveform_geometry.dart';

/// Time values along the top of the waveform. Pressing it puts the playhead
/// there at once, and dragging goes on from there, like the position slider
/// does. It reports the times; what they mean is up to the owner.
class TimelineRuler extends StatelessWidget {
  const TimelineRuler({
    required this.viewport,
    required this.onScrub,
    required this.onScrubEnd,
    super.key,
  });

  final TimeViewport viewport;

  /// The finger is at this time (on pressing and while dragging).
  final ValueChanged<double> onScrub;

  /// The finger was lifted.
  final VoidCallback onScrubEnd;

  @override
  Widget build(BuildContext context) => GestureDetector(
    behavior: HitTestBehavior.opaque,
    onTapDown: (details) => onScrub(viewport.xToTime(details.localPosition.dx)),
    onTapUp: (_) => onScrubEnd(),
    onTapCancel: () {},
    onHorizontalDragStart: (details) =>
        onScrub(viewport.xToTime(details.localPosition.dx)),
    onHorizontalDragUpdate: (details) =>
        onScrub(viewport.xToTime(details.localPosition.dx)),
    onHorizontalDragEnd: (_) => onScrubEnd(),
    onHorizontalDragCancel: onScrubEnd,
    child: ExcludeSemantics(
      child: CustomPaint(
        size: Size(viewport.widthPx, WaveformGeometry.rulerHeight),
        painter: _RulerPainter(viewport),
      ),
    ),
  );
}

class _RulerPainter extends CustomPainter {
  _RulerPainter(this.viewport) : ticks = computeRulerTicks(viewport);

  final TimeViewport viewport;
  final List<RulerTick> ticks;

  @override
  void paint(Canvas canvas, Size size) {
    final line = Paint()
      ..color = AppColors.border
      ..strokeWidth = 1;
    for (final tick in ticks) {
      final x = viewport.timeToX(tick.ms.toDouble());
      canvas.drawLine(Offset(x, size.height - 8), Offset(x, size.height), line);
      final text = TextPainter(
        text: TextSpan(
          text: tick.label,
          style: const TextStyle(
            color: AppColors.textSecondary,
            fontSize: AppTypography.sizeSmall,
          ),
        ),
        textDirection: TextDirection.ltr,
      )..layout();
      text.paint(canvas, Offset(x + 4, 0));
    }
  }

  @override
  bool shouldRepaint(_RulerPainter old) => old.viewport != viewport;
}
