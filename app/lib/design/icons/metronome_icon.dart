import 'package:material_ui/material_ui.dart';

/// A metronome, drawn as an outline in the colour of the surrounding icons.
class MetronomeIcon extends StatelessWidget {
  const MetronomeIcon({this.size = 24, this.color, super.key});

  final double size;
  final Color? color;

  @override
  Widget build(BuildContext context) => CustomPaint(
    size: Size.square(size),
    painter: _MetronomePainter(color ?? IconTheme.of(context).color!),
  );
}

class _MetronomePainter extends CustomPainter {
  _MetronomePainter(this.color);

  final Color color;

  @override
  void paint(Canvas canvas, Size size) {
    final unit = size.width / 24;
    final stroke = Paint()
      ..color = color
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2 * unit
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;
    Offset at(double x, double y) => Offset(x * unit, y * unit);

    final body = Path()
      ..moveTo(7 * unit, 21 * unit)
      ..lineTo(17 * unit, 21 * unit)
      ..lineTo(14.5 * unit, 3.5 * unit)
      ..lineTo(9.5 * unit, 3.5 * unit)
      ..close();
    canvas
      ..drawPath(body, stroke)
      ..drawLine(at(12, 17), at(16, 8), stroke)
      ..drawCircle(at(14.6, 11.2), 1.4 * unit, Paint()..color = color);
  }

  @override
  bool shouldRepaint(_MetronomePainter old) => old.color != color;
}
