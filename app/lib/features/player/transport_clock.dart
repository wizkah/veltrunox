import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_typography.dart';
import '../../shared_ui/clock_format.dart';

/// The playback position beside the transport, as "1:03,8 / 3:14", in
/// tabular figures so the digits do not jump while it runs. Without a known
/// length only the position is shown.
class TransportClock extends StatelessWidget {
  const TransportClock({required this.positionMs, this.lengthMs, super.key});

  final int positionMs;
  final int? lengthMs;

  static const _figures = [FontFeature.tabularFigures()];

  @override
  Widget build(BuildContext context) {
    final length = lengthMs;
    return Text.rich(
      TextSpan(
        text: formatClockTenths(positionMs),
        style: const TextStyle(
          color: AppColors.textPrimary,
          fontWeight: AppTypography.semibold,
        ),
        children: [
          if (length != null)
            TextSpan(
              text: ' / ${formatClock(length)}',
              style: const TextStyle(
                color: AppColors.textSecondary,
                fontWeight: AppTypography.medium,
              ),
            ),
        ],
      ),
      style: const TextStyle(
        fontSize: AppTypography.sizeSection,
        fontFeatures: _figures,
      ),
    );
  }
}
