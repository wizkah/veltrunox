import 'package:material_ui/material_ui.dart';

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
  Widget build(BuildContext context) => IconButton(
    tooltip: LibraryTexts.reanalyse,
    onPressed: _pending ? null : onPressed,
    icon: const Icon(Icons.refresh),
  );
}
