import 'package:material_ui/material_ui.dart';

import 'player_texts.dart';
import 'value_entry_field.dart';

/// Shows the tuning the playback uses and takes a wanted tuning in Hz.
class TuningField extends StatelessWidget {
  const TuningField({
    required this.currentHz,
    required this.onSubmit,
    this.trailing,
    super.key,
  });

  final double currentHz;

  /// Reports whether the tuning could be set.
  final Future<bool> Function(double hertz) onSubmit;

  /// Shown after the field, for example a reset button.
  final Widget? trailing;

  @override
  Widget build(BuildContext context) => ValueEntryField(
    label: PlayerTexts.tuning,
    currentText: PlayerTexts.tuningValue(currentHz),
    fieldLabel: PlayerTexts.tuningTarget,
    notANumber: PlayerTexts.tuningInvalid,
    outOfRange: PlayerTexts.tuningOutOfRange,
    onSubmit: onSubmit,
    trailing: trailing,
  );
}
