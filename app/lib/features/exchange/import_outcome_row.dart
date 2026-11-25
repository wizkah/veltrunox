import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import 'exchange_models.dart';
import 'exchange_texts.dart';
import 'import_issue_row.dart';

/// One recording of the import preview: its title, what became of it and
/// either the numbers of changes or the issues.
class ImportOutcomeRow extends StatelessWidget {
  const ImportOutcomeRow({
    required this.title,
    required this.outcome,
    super.key,
  });

  final String title;
  final RecordingImport outcome;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: AppSpace.s2),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Icon(_icon, size: AppSpace.s4, color: _color),
            const SizedBox(width: AppSpace.s2),
            Expanded(
              child: Text(title, style: Theme.of(context).textTheme.titleSmall),
            ),
            Text(
              ExchangeTexts.statusLabel(outcome.status),
              style: TextStyle(color: _color),
            ),
          ],
        ),
        ..._details(),
      ],
    ),
  );

  List<Widget> _details() {
    switch (outcome.status) {
      case ImportStatus.ready:
        return [
          Text(
            ExchangeTexts.counts(
              added: outcome.added,
              changed: outcome.changed,
              unchanged: outcome.unchanged,
            ),
          ),
          if (outcome.removed == 0)
            Text(ExchangeTexts.removed(0))
          else
            Row(
              children: [
                const Icon(
                  Icons.warning_amber,
                  size: AppSpace.s4,
                  color: AppColors.warning,
                ),
                const SizedBox(width: AppSpace.s1),
                Expanded(
                  child: Text(
                    ExchangeTexts.removedWarning(outcome.removed),
                    style: const TextStyle(
                      color: AppColors.warning,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
              ],
            ),
        ];
      case ImportStatus.unknownRecording:
        return [Text(ExchangeTexts.unknownRecording)];
      case ImportStatus.conflict:
      case ImportStatus.invalid:
        return [
          for (final issue in outcome.issues) ImportIssueRow(issue: issue),
        ];
    }
  }

  IconData get _icon => switch (outcome.status) {
    ImportStatus.ready => Icons.check_circle_outline,
    ImportStatus.conflict => Icons.warning_amber,
    ImportStatus.invalid => Icons.error_outline,
    ImportStatus.unknownRecording => Icons.help_outline,
  };

  Color get _color => switch (outcome.status) {
    ImportStatus.ready => AppColors.success,
    ImportStatus.conflict => AppColors.warning,
    ImportStatus.invalid => AppColors.error,
    ImportStatus.unknownRecording => AppColors.textSecondary,
  };
}
