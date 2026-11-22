import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../library/analysis_format.dart';
import 'player_texts.dart';

/// The key the recording plays in, chosen from the semitone steps up and down
/// (+12 at the top, −12 at the bottom). With the analysed key each step is
/// named as the key it gives.
class KeyDropdown extends StatelessWidget {
  const KeyDropdown({
    required this.semitones,
    required this.onChanged,
    this.originalTonic,
    this.originalMinor = false,
    this.trailing,
    super.key,
  });

  static const range = 12;

  final int semitones;
  final ValueChanged<int> onChanged;

  /// Tonic of the analysed key (0 = C … 11 = B), if there is one.
  final int? originalTonic;
  final bool originalMinor;

  /// Shown after the dropdown, for example a reset button.
  final Widget? trailing;

  String _label(int shift) {
    final tonic = originalTonic;
    final steps = PlayerTexts.keyValue(shift);
    if (tonic == null) return steps;
    final name = keyName(tonic + shift, minor: originalMinor);
    return shift == 0 ? '$name (${PlayerTexts.keyOriginal})' : '$name ($steps)';
  }

  @override
  Widget build(BuildContext context) => Wrap(
    crossAxisAlignment: WrapCrossAlignment.center,
    children: [
      SizedBox(width: 152, child: Text(PlayerTexts.key)),
      DropdownButton<int>(
        style: Theme.of(context).textTheme.bodyMedium,
        dropdownColor: AppColors.surfaceRaised,
        value: semitones.clamp(-range, range),
        hint: Text(PlayerTexts.keyChoice),
        items: [
          for (var shift = range; shift >= -range; shift--)
            DropdownMenuItem(value: shift, child: Text(_label(shift))),
        ],
        onChanged: (shift) {
          if (shift != null) onChanged(shift);
        },
      ),
      ?trailing,
    ],
  );
}
