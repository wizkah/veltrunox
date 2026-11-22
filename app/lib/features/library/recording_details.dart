import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import '../../shared_ui/clock_format.dart';
import 'analysis_format.dart';
import 'analysis_info.dart';
import 'analysis_summary.dart';
import 'library_texts.dart';
import 'recording_summary.dart';

/// What the library knows about one recording, and what can be done with it:
/// open it in the player, play it at once, analyse it again. On a narrow
/// screen it is a view of its own, with a way [onBack].
class RecordingDetails extends StatelessWidget {
  const RecordingDetails({
    required this.recording,
    required this.onOpenInPlayer,
    required this.onPlay,
    required this.onReanalyse,
    this.onRename,
    this.onDelete,
    this.onBack,
    super.key,
  });

  final RecordingSummary recording;
  final VoidCallback onOpenInPlayer;
  final VoidCallback onPlay;
  final VoidCallback onReanalyse;

  /// Only for those who may manage recordings; without them the actions are not shown.
  final VoidCallback? onRename;
  final VoidCallback? onDelete;

  /// Leads back to the list; only for the view of its own.
  final VoidCallback? onBack;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context).textTheme;
    final analysis = recording.analysis;
    return SingleChildScrollView(
      padding: const EdgeInsets.all(AppSpace.s4),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (onBack != null)
            TextButton.icon(
              onPressed: onBack,
              icon: const Icon(Icons.arrow_back),
              label: Text(LibraryTexts.back),
            ),
          Text(recording.title, style: theme.headlineMedium),
          const SizedBox(height: AppSpace.s3),
          AnalysisSummary(analysis: analysis),
          const SizedBox(height: AppSpace.s3),
          if (recording.lengthMs != null)
            _Value(LibraryTexts.lengthLabel, formatClock(recording.lengthMs!)),
          ..._values(analysis.values),
          const SizedBox(height: AppSpace.s4),
          Wrap(
            spacing: AppSpace.s3,
            runSpacing: AppSpace.s3,
            children: [
              ActionButton(
                label: LibraryTexts.play,
                icon: Icons.play_arrow,
                onPressed: onPlay,
              ),
              ActionButton(
                label: LibraryTexts.openInPlayer,
                icon: Icons.open_in_new,
                kind: ActionKind.secondary,
                onPressed: onOpenInPlayer,
              ),
              ActionButton(
                label: LibraryTexts.reanalyse,
                icon: Icons.refresh,
                kind: ActionKind.secondary,
                onPressed: _pending(analysis.status) ? null : onReanalyse,
              ),
              if (onRename != null)
                ActionButton(
                  label: LibraryTexts.rename,
                  icon: Icons.edit_outlined,
                  kind: ActionKind.secondary,
                  onPressed: onRename,
                ),
              if (onDelete != null)
                ActionButton(
                  label: LibraryTexts.delete,
                  icon: Icons.delete_outline,
                  kind: ActionKind.danger,
                  onPressed: onDelete,
                ),
            ],
          ),
        ],
      ),
    );
  }

  static bool _pending(AnalysisStatus status) =>
      status == AnalysisStatus.queued || status == AnalysisStatus.running;

  List<Widget> _values(AnalysisValues? values) {
    if (values == null) {
      return [Text(LibraryTexts.notAnalysed)];
    }
    return [
      _Value(
        LibraryTexts.keyLabel,
        keyName(values.keyTonic, minor: values.keyMinor),
      ),
      _Value(LibraryTexts.tempoLabel, '${decimal(values.tempoBpm)} bpm'),
      _Value(LibraryTexts.tuningLabel, '${decimal(values.tuningHz)} Hz'),
    ];
  }
}

class _Value extends StatelessWidget {
  const _Value(this.label, this.value);

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(bottom: AppSpace.s1),
    child: Text.rich(
      TextSpan(
        children: [
          TextSpan(
            text: '$label: ',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          TextSpan(text: value),
        ],
      ),
    ),
  );
}
