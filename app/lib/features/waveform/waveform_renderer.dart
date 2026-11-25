import 'dart:typed_data';
import 'dart:ui' show PointMode;

import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import 'time_viewport.dart';
import 'waveform_data.dart';
import 'waveform_geometry.dart';
import 'waveform_texts.dart';

/// The amplitudes of the visible time range on a dark stage. Draws only; no
/// input. Without [data] it shows that the waveform is loading (or missing),
/// never a plausible-looking empty one. What lies after [playedMs] is drawn
/// muted, so the played part stands out; the amplitudes stay visible.
class WaveformRenderer extends StatefulWidget {
  WaveformRenderer({
    required this.data,
    required this.viewport,
    this.height = WaveformGeometry.waveformHeight,
    this.playedMs,
    String? missingText,
    super.key,
  }) : missingText = missingText ?? WaveformTexts.loading;

  final WaveformData? data;
  final TimeViewport viewport;
  final double height;

  /// Up to here the recording counts as played; `null` draws all alike.
  final double? playedMs;

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
    final played = widget.playedMs;
    return SizedBox(
      width: widget.viewport.widthPx,
      height: widget.height,
      child: DecoratedBox(
        position: DecorationPosition.foreground,
        decoration: _stageEdge,
        child: ClipRRect(
          borderRadius: _stageRadius,
          child: ColoredBox(
            color: AppColors.canvas,
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
                    label: WaveformTexts.title,
                    child: CustomPaint(
                      size: Size.infinite,
                      painter: _PeakPainter(
                        columns,
                        playedX: played == null
                            ? null
                            : widget.viewport.timeToX(played),
                      ),
                    ),
                  ),
          ),
        ),
      ),
    );
  }
}

final _stageRadius = BorderRadius.circular(AppRadius.small);

/// The stage's edge, drawn over the waveform so it takes no room.
final _stageEdge = BoxDecoration(
  border: Border.all(
    color: Color.lerp(AppColors.border, AppColors.accent, 0.25)!,
    width: AppSize.borderWidth,
  ),
  borderRadius: _stageRadius,
);

/// Bright in the middle, deeper towards the peaks: gives a dense, loud
/// recording some depth instead of a flat block.
final _waveColors = [
  Color.lerp(AppColors.accent, AppColors.canvas, 0.3)!,
  AppColors.accentSoft,
  Color.lerp(AppColors.accent, AppColors.canvas, 0.3)!,
];

/// How much of its colour the part after the playhead keeps.
const _unplayedAlpha = 0.55;

class _PeakPainter extends CustomPainter {
  _PeakPainter(this.columns, {this.playedX});

  final PeakColumns columns;

  /// Columns from here on are not played yet; `null`: all count as played.
  final double? playedX;

  @override
  void paint(Canvas canvas, Size size) {
    final middle = size.height / 2;
    final scale = middle - AppSpace.s1;
    final bounds = Offset.zero & size;
    final played = _paint(bounds, 1);
    final unplayed = _paint(bounds, _unplayedAlpha);
    canvas.drawLine(
      Offset(0, middle),
      Offset(size.width, middle),
      Paint()
        ..color = AppColors.accent.withValues(alpha: 0.25)
        ..strokeWidth = 1,
    );
    final split = playedX ?? double.infinity;
    // All columns of one colour go to the canvas in a single call: a call per
    // column is too slow while zooming, most of all in a browser.
    final lines = Float32List(columns.length * 4);
    var used = 0;
    var playedUsed = 0;
    for (var x = 0; x < columns.length; x++) {
      final low = columns.min[x];
      final high = columns.max[x];
      if (low == 0 && high == 0) continue;
      lines[used++] = x + 0.5;
      lines[used++] = middle - high * scale;
      lines[used++] = x + 0.5;
      lines[used++] = middle - low * scale;
      if (x < split) playedUsed = used;
    }
    canvas.drawRawPoints(
      PointMode.lines,
      Float32List.sublistView(lines, 0, playedUsed),
      played,
    );
    canvas.drawRawPoints(
      PointMode.lines,
      Float32List.sublistView(lines, playedUsed, used),
      unplayed,
    );
  }

  static Paint _paint(Rect bounds, double alpha) => Paint()
    ..strokeWidth = 1
    ..shader = LinearGradient(
      begin: Alignment.topCenter,
      end: Alignment.bottomCenter,
      colors: [for (final color in _waveColors) color.withValues(alpha: alpha)],
    ).createShader(bounds);

  @override
  bool shouldRepaint(_PeakPainter old) =>
      old.columns != columns || old.playedX != playedX;
}
