import 'package:material_ui/material_ui.dart';

import '../design/tokens/app_colors.dart';
import '../design/tokens/app_geometry.dart';

enum ActivityState { idle, loading, progress, success, error }

/// What a long-running activity (upload, analysis) is doing. Shows an icon and
/// a text, never colour alone. Only [ActivityState.progress] with a [progress]
/// value shows a percentage; without one the progress is indeterminate.
class ActivityStatus extends StatelessWidget {
  const ActivityStatus({
    required this.state,
    required this.label,
    this.progress,
    super.key,
  });

  final ActivityState state;
  final String label;

  /// 0 to 1; only used with [ActivityState.progress].
  final double? progress;

  @override
  Widget build(BuildContext context) => Semantics(
    liveRegion: true,
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        _indicator(),
        const SizedBox(width: AppSpace.s2),
        Flexible(child: Text(label)),
      ],
    ),
  );

  Widget _indicator() => switch (state) {
    ActivityState.idle => const Icon(
      Icons.radio_button_unchecked,
      size: AppSpace.s4,
      color: AppColors.textSecondary,
    ),
    ActivityState.loading => const _Spinner(),
    ActivityState.progress => _Spinner(value: progress?.clamp(0.0, 1.0)),
    ActivityState.success => const Icon(
      Icons.check_circle_outline,
      size: AppSpace.s4,
      color: AppColors.success,
    ),
    ActivityState.error => const Icon(
      Icons.error_outline,
      size: AppSpace.s4,
      color: AppColors.error,
    ),
  };
}

class _Spinner extends StatelessWidget {
  const _Spinner({this.value});

  final double? value;

  @override
  Widget build(BuildContext context) => SizedBox.square(
    dimension: AppSpace.s4,
    child: CircularProgressIndicator(strokeWidth: 2, value: value),
  );
}
