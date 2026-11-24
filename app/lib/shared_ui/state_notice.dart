import 'package:material_ui/material_ui.dart';

import '../design/tokens/app_colors.dart';
import '../design/tokens/app_geometry.dart';
import 'action_button.dart';
import 'frame_decoration.dart';

/// What a state notice is about; an error is framed in the error colour.
enum NoticeTone { neutral, error }

/// The common shape of empty, loading and error states: an icon badge (a
/// spinner while [busy]), the message and, where sensible, the next action.
/// The state is told by icon and text, not by colour alone.
class StateNotice extends StatelessWidget {
  const StateNotice({
    required this.message,
    this.title,
    this.icon = Icons.info_outline,
    this.tone = NoticeTone.neutral,
    this.busy = false,
    this.actionLabel,
    this.actionIcon,
    this.onAction,
    this.actionKind = ActionKind.secondary,
    this.framed = true,
    super.key,
  });

  final String message;

  /// A short heading above [message], for a state that needs one.
  final String? title;
  final IconData icon;
  final NoticeTone tone;

  /// Something is under way; the badge shows a spinner instead of [icon].
  final bool busy;

  /// The next action; shown only with [onAction].
  final String? actionLabel;
  final IconData? actionIcon;
  final VoidCallback? onAction;
  final ActionKind actionKind;

  /// Draws the notice on a panel; off where it already stands on one.
  final bool framed;

  static const _maxWidth = 360.0;
  static const _badgeExtent = AppSpace.s8 + AppSpace.s6;

  Color get _toneColor =>
      tone == NoticeTone.error ? AppColors.error : AppColors.accentSoft;

  @override
  Widget build(BuildContext context) {
    final content = Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        _badge(),
        const SizedBox(height: AppSpace.s3),
        if (title != null) ...[
          Text(
            title!,
            textAlign: TextAlign.center,
            style: Theme.of(context).textTheme.titleLarge,
          ),
          const SizedBox(height: AppSpace.s2),
        ],
        Text(
          message,
          textAlign: TextAlign.center,
          style: Theme.of(context).textTheme.bodyMedium,
        ),
        if (actionLabel != null && onAction != null) ...[
          const SizedBox(height: AppSpace.s4),
          ActionButton(
            label: actionLabel!,
            icon: actionIcon,
            kind: actionKind,
            onPressed: onAction,
          ),
        ],
      ],
    );
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(AppSpace.s4),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: _maxWidth),
          child: Semantics(
            liveRegion: busy || tone == NoticeTone.error,
            child: framed
                ? DecoratedBox(
                    decoration: _frame(),
                    child: Padding(
                      padding: const EdgeInsets.all(AppSpace.s6),
                      child: content,
                    ),
                  )
                : content,
          ),
        ),
      ),
    );
  }

  BoxDecoration _frame() {
    final frame = frameDecoration();
    if (tone != NoticeTone.error) return frame;
    return frame.copyWith(
      border: Border.all(color: AppColors.error, width: AppSize.borderWidth),
      boxShadow: [
        BoxShadow(
          color: AppColors.error.withValues(alpha: 0.18),
          blurRadius: AppSpace.s4,
        ),
      ],
    );
  }

  Widget _badge() => Container(
    width: _badgeExtent,
    height: _badgeExtent,
    decoration: BoxDecoration(
      shape: BoxShape.circle,
      color: _toneColor.withValues(alpha: 0.12),
      border: Border.all(color: _toneColor.withValues(alpha: 0.5)),
    ),
    child: busy
        ? Padding(
            padding: const EdgeInsets.all(AppSpace.s3),
            child: CircularProgressIndicator(strokeWidth: 3, color: _toneColor),
          )
        : Icon(icon, color: _toneColor, size: AppSpace.s6),
  );
}
