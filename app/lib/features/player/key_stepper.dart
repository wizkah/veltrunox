import 'package:material_ui/material_ui.dart';

import 'player_texts.dart';

/// Shifts the key by one semitone per tap.
class KeyStepper extends StatelessWidget {
  const KeyStepper({
    required this.semitones,
    required this.onChanged,
    super.key,
  });

  final int semitones;
  final ValueChanged<int> onChanged;

  @override
  Widget build(BuildContext context) => Row(
    children: [
      const SizedBox(width: 120, child: Text(PlayerTexts.key)),
      IconButton(
        tooltip: PlayerTexts.keyLower,
        onPressed: () => onChanged(semitones - 1),
        icon: const Icon(Icons.remove),
      ),
      Text(PlayerTexts.keyValue(semitones)),
      IconButton(
        tooltip: PlayerTexts.keyHigher,
        onPressed: () => onChanged(semitones + 1),
        icon: const Icon(Icons.add),
      ),
    ],
  );
}
