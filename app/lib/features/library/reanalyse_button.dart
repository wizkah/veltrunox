import 'package:material_ui/material_ui.dart';

import '../../shared_ui/app_icon_button.dart';
import 'analysis_info.dart';
import 'library_texts.dart';

/// Starts the analysis of a recording again; unavailable while one is pending.
class ReanalyseButton extends StatelessWidget {
  const ReanalyseButton({
    required this.status,
    required this.onPressed,
    super.key,
  });

  final AnalysisStatus status;
  final VoidCallback onPressed;

  bool get _pending =>
      status == AnalysisStatus.queued || status == AnalysisStatus.running;

  @override
  Widget build(BuildContext context) => AppIconButton(
    tooltip: LibraryTexts.reanalyse,
    onPressed: _pending ? null : onPressed,
    icon: Icons.refresh,
  );
}
