import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import '../../design/tokens/app_typography.dart';
import 'analysis_format.dart';
import 'analysis_info.dart';

/// Key, tempo and tuning of an analysed recording as small chips, so they can
/// be compared down the list at a glance. Read out as one line.
class AnalysisChips extends StatelessWidget {
  const AnalysisChips({required this.values, super.key});

  final AnalysisValues values;

  @override
  Widget build(BuildContext context) {
    final chips = [
      keyName(values.keyTonic, minor: values.keyMinor),
      '${decimal(values.tempoBpm)} bpm',
      '${decimal(values.tuningHz)} Hz',
    ];
    return Semantics(
      label: chips.join(' · '),
      child: ExcludeSemantics(
        child: Wrap(
          spacing: AppSpace.s2,
          runSpacing: AppSpace.s1,
          children: [for (final chip in chips) _ValueChip(chip)],
        ),
      ),
    );
  }
}

class _ValueChip extends StatelessWidget {
  const _ValueChip(this.text);

  final String text;

  static const _figures = [FontFeature.tabularFigures()];

  @override
  Widget build(BuildContext context) => DecoratedBox(
    decoration: BoxDecoration(
      color: AppColors.accent.withValues(alpha: 0.08),
      border: Border.all(color: AppColors.accent.withValues(alpha: 0.5)),
      borderRadius: BorderRadius.circular(AppRadius.small),
    ),
    child: Padding(
      padding: const EdgeInsets.symmetric(horizontal: AppSpace.s2),
      child: Text(
        text,
        style: const TextStyle(
          color: AppColors.textPrimary,
          fontSize: AppTypography.sizeSmall,
          fontWeight: AppTypography.medium,
          fontFeatures: _figures,
        ),
      ),
    ),
  );
}
