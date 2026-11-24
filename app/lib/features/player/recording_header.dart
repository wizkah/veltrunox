import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/clock_format.dart';
import '../../shared_ui/frame_decoration.dart';
import '../library/analysis_format.dart';
import 'player_analysis.dart';
import 'player_texts.dart';

/// The head of the player: an icon tile, the recording's title and a line with
/// what is known about it (key, tempo, length). Values that are not known yet
/// are left out, never guessed.
class RecordingHeader extends StatelessWidget {
  const RecordingHeader({
    required this.title,
    this.analysis,
    this.lengthMs,
    super.key,
  });

  final String title;
  final PlayerAnalysis? analysis;
  final int? lengthMs;

  static const _tileExtent = AppSize.touchTarget;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context).textTheme;
    final details = _details();
    return Row(
      children: [
        Container(
          width: _tileExtent,
          height: _tileExtent,
          decoration: frameDecoration(),
          child: const Icon(Icons.music_note, color: AppColors.accentSoft),
        ),
        const SizedBox(width: AppSpace.s3),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                title,
                style: theme.headlineMedium,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
              if (details.isNotEmpty) Text(details, style: theme.bodySmall),
            ],
          ),
        ),
      ],
    );
  }

  /// "E-Dur · 122,2 bpm · 3:14".
  String _details() {
    final found = analysis;
    final tonic = found?.keyTonic;
    final length = lengthMs;
    return [
      if (tonic != null) keyName(tonic, minor: found!.keyMinor),
      if (found != null) PlayerTexts.tempoValue(found.tempoBpm),
      if (length != null) formatClock(length),
    ].join(' · ');
  }
}
