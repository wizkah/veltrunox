import 'package:material_ui/material_ui.dart';

import 'player_texts.dart';
import 'value_entry_field.dart';

/// Shows the tempo the recording plays at and takes a wanted tempo.
class TargetTempoField extends StatelessWidget {
  const TargetTempoField({
    required this.currentBpm,
    required this.onSubmit,
    super.key,
  });

  final double currentBpm;

  /// Reports whether the tempo could be set.
  final Future<bool> Function(double bpm) onSubmit;

  @override
  Widget build(BuildContext context) => ValueEntryField(
    label: PlayerTexts.tempo,
    currentText: PlayerTexts.tempoValue(currentBpm),
    fieldLabel: PlayerTexts.targetTempo,
    notANumber: PlayerTexts.tempoInvalid,
    outOfRange: PlayerTexts.tempoOutOfRange,
    onSubmit: onSubmit,
  );
}
