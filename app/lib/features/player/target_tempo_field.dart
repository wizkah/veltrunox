import 'package:material_ui/material_ui.dart';

import 'player_texts.dart';
import 'value_entry_field.dart';

/// Shows the tempo the recording plays at and takes a wanted tempo. Without a
/// known tempo ([currentBpm] is `null`) it only shows that.
class TargetTempoField extends StatelessWidget {
  const TargetTempoField({
    required this.currentBpm,
    required this.onSubmit,
    this.trailing,
    super.key,
  });

  final double? currentBpm;

  /// Reports whether the tempo could be set.
  final Future<bool> Function(double bpm) onSubmit;

  /// Shown after the field, for example a reset button.
  final Widget? trailing;

  @override
  Widget build(BuildContext context) => ValueEntryField(
    label: PlayerTexts.targetTempoRow,
    valueText: currentBpm == null ? '' : PlayerTexts.decimal(currentBpm!),
    unit: 'bpm',
    enabled: currentBpm != null,
    notANumber: PlayerTexts.tempoInvalid,
    outOfRange: PlayerTexts.tempoOutOfRange,
    onSubmit: onSubmit,
    trailing: trailing,
  );
}
