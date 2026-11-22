import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import 'time_viewport.dart';
import 'waveform_data.dart';
import 'waveform_geometry.dart';
import 'waveform_texts.dart';

/// The amplitudes of the visible time range. Draws only; no input. Without
/// [data] it shows that the waveform is loading (or missing), never a
/// plausible-looking empty one.
class WaveformRenderer extends StatefulWidget {
  const WaveformRenderer({
    required this.data,
    required this.viewport,
    this.missingText = WaveformTexts.loading,
    super.key,
  });

  final WaveformData? data;
  final TimeViewport viewport;

  /// Shown instead of the waveform while [data] is null.
  final String missingText;

  @override
  State<WaveformRenderer> createState() => _WaveformRendererState();
}

class _WaveformRendererState extends State<WaveformRenderer> {
  PeakColumns? _columns;

  @override
  void initState() {
    super.initState();
    _columns = _compute();
  }

  @override
  void didUpdateWidget(WaveformRenderer old) {
    super.didUpdateWidget(old);
    if (old.data != widget.data || old.viewport != widget.viewport) {
      _columns = _compute();
    }
  }

  PeakColumns? _compute() {
    final data = widget.data;
    return data == null ? null : computeColumns(data, widget.viewport);
  }

  @override
  Widget build(BuildContext context) {
    final columns = _columns;
    return SizedBox(
      width: widget.viewport.widthPx,
      height: WaveformGeometry.waveformHeight,
      child: columns == null
          ? Center(
              child: Text(
                widget.missingText,
                style: Theme.of(context).textTheme.bodyMedium
                    ?.copyWith(color: AppColors.textSecondary),
              ),
            )
          : Semantics(
              image: true,
              label: 'Wellenform',
              child: CustomPaint(painter: _PeakPainter(columns)),
            ),
    );
  }
}

class _PeakPainter extends CustomPainter {
  _PeakPainter(this.columns);

  final PeakColumns columns;

  @override
  void paint(Canvas canvas, Size size) {
    final middle = size.height / 2;
    final scale = middle - AppSpace.s1;
    final paint = Paint()
      ..color = AppColors.accent
      ..strokeWidth = 1;
    for (var x = 0; x < columns.length; x++) {
      final low = columns.min[x];
      final high = columns.max[x];
      if (low == 0 && high == 0) continue;
      canvas.drawLine(
        Offset(x + 0.5, middle - high * scale),
        Offset(x + 0.5, middle - low * scale),
        paint,
      );
    }
  }

  @override
  bool shouldRepaint(_PeakPainter old) => old.columns != columns;
}
