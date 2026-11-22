import 'package:material_ui/material_ui.dart';

/// A tuning fork, drawn as an outline in the colour of the surrounding icons.
class TuningForkIcon extends StatelessWidget {
  const TuningForkIcon({this.size = 24, this.color, super.key});

  final double size;
  final Color? color;

  @override
  Widget build(BuildContext context) => CustomPaint(
    size: Size.square(size),
    painter: _TuningForkPainter(color ?? IconTheme.of(context).color!),
  );
}

class _TuningForkPainter extends CustomPainter {
  _TuningForkPainter(this.color);

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

    final fork = Path()
      ..moveTo(8.5 * unit, 3 * unit)
      ..lineTo(8.5 * unit, 10 * unit)
      ..quadraticBezierTo(8.5 * unit, 14.5 * unit, 12 * unit, 14.5 * unit)
      ..quadraticBezierTo(15.5 * unit, 14.5 * unit, 15.5 * unit, 10 * unit)
      ..lineTo(15.5 * unit, 3 * unit)
      ..moveTo(12 * unit, 14.5 * unit)
      ..lineTo(12 * unit, 21 * unit);
    canvas.drawPath(fork, stroke);
  }

  @override
  bool shouldRepaint(_TuningForkPainter old) => old.color != color;
}
