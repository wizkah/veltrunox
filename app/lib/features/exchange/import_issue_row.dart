import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/server_reason.dart';
import 'exchange_models.dart';
import 'exchange_texts.dart';

/// One problem of an import: where it lies and why, in words.
class ImportIssueRow extends StatelessWidget {
  const ImportIssueRow({required this.issue, super.key});

  final ImportIssue issue;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(top: AppSpace.s1),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Icon(
          Icons.subdirectory_arrow_right,
          size: AppSpace.s4,
          color: AppColors.textSecondary,
        ),
        const SizedBox(width: AppSpace.s2),
        Expanded(
          child: Text(
            '${ExchangeTexts.entry(issue.kind, issue.index)}: '
            '${reasonText(issue.reason)}',
          ),
        ),
      ],
    ),
  );

  /// In an import the two bookmark reasons mean that the original of an
  /// entry is gone or was changed, so they are worded for that.
  static String reasonText(ServerReason reason) => switch (reason.code) {
    'bookmark_not_found' => ExchangeTexts.originalGone,
    'bookmark_conflict' => ExchangeTexts.originalChanged,
    _ => serverReasonText(reason),
  };
}
