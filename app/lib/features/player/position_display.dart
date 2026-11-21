import 'package:material_ui/material_ui.dart';

/// Shows a playback position as minutes, seconds and tenths.
class PositionDisplay extends StatelessWidget {
  const PositionDisplay({required this.positionMs, super.key});

  final int positionMs;

  @override
  Widget build(BuildContext context) => Text(
    formatPosition(positionMs),
    style: Theme.of(context).textTheme.headlineMedium,
  );
}

String formatPosition(int positionMs) {
  final tenths = positionMs ~/ 100 % 10;
  final seconds = positionMs ~/ 1000 % 60;
  final minutes = positionMs ~/ 60000;
  return '$minutes:${seconds.toString().padLeft(2, '0')}.$tenths';
}
