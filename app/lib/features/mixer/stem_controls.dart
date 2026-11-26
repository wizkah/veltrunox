import 'dart:async';

import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import 'mixer_texts.dart';

/// Name, volume, solo and mute of one stem. Without callbacks ([enabled] off)
/// they show their values but take no input. A silent stem says why.
class StemControls extends StatefulWidget {
  const StemControls({
    required this.name,
    required this.volumePercent,
    required this.muted,
    required this.solo,
    required this.audible,
    required this.enabled,
    required this.onVolume,
    required this.onMuted,
    required this.onSolo,
    super.key,
  });

  final String name;
  final int volumePercent;
  final bool muted;
  final bool solo;
  final bool audible;
  final bool enabled;
  final ValueChanged<int> onVolume;
  final ValueChanged<bool> onMuted;
  final ValueChanged<bool> onSolo;

  @override
  State<StemControls> createState() => _StemControlsState();
}

class _StemControlsState extends State<StemControls> {
  /// The value under the finger; `null` shows the shared one.
  double? _dragged;
  Timer? _throttle;
  int? _waiting;

  /// Sends at most one value per interval while dragging, the latest last.
  void _send(int percent) {
    if (_throttle != null) {
      _waiting = percent;
      return;
    }
    widget.onVolume(percent);
    _throttle = Timer(const Duration(milliseconds: 60), () {
      _throttle = null;
      final waiting = _waiting;
      _waiting = null;
      if (waiting != null) _send(waiting);
    });
  }

  void _release(double value) {
    _throttle?.cancel();
    _throttle = null;
    _waiting = null;
    widget.onVolume(value.round());
    setState(() => _dragged = null);
  }

  @override
  void dispose() {
    _throttle?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final silent = widget.enabled && !widget.audible;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        Text(
          silent
              ? '${widget.name} · '
                    '${widget.muted ? MixerTexts.muted : MixerTexts.notInSolo}'
              : widget.name,
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
          style: Theme.of(context).textTheme.bodyMedium?.copyWith(
            color: silent ? AppColors.textSecondary : AppColors.textPrimary,
          ),
        ),
        Row(
          children: [
            Expanded(
              child: Slider(
                value: _dragged ?? widget.volumePercent.toDouble(),
                max: 100,
                semanticFormatterCallback: (value) =>
                    '${MixerTexts.volumeOf(widget.name)} ${value.round()} %',
                onChanged: widget.enabled
                    ? (value) {
                        setState(() => _dragged = value);
                        _send(value.round());
                      }
                    : null,
                onChangeEnd: _release,
              ),
            ),
            _Toggle(
              letter: 'S',
              label: MixerTexts.soloOf(widget.name),
              selected: widget.solo,
              color: AppColors.warning,
              onChanged: widget.enabled ? widget.onSolo : null,
            ),
            _Toggle(
              letter: 'M',
              label: MixerTexts.muteOf(widget.name),
              selected: widget.muted,
              color: AppColors.error,
              onChanged: widget.enabled ? widget.onMuted : null,
            ),
          ],
        ),
      ],
    );
  }
}

class _Toggle extends StatelessWidget {
  const _Toggle({
    required this.letter,
    required this.label,
    required this.selected,
    required this.color,
    required this.onChanged,
  });

  final String letter;
  final String label;
  final bool selected;
  final Color color;
  final ValueChanged<bool>? onChanged;

  @override
  Widget build(BuildContext context) => Semantics(
    button: true,
    toggled: selected,
    enabled: onChanged != null,
    label: label,
    excludeSemantics: true,
    onTap: onChanged == null ? null : () => onChanged!(!selected),
    child: Tooltip(
      message: label,
      child: InkWell(
        borderRadius: BorderRadius.circular(AppRadius.small),
        onTap: onChanged == null ? null : () => onChanged!(!selected),
        child: SizedBox(
          width: 36,
          height: AppSize.touchTarget,
          child: Center(
            child: Container(
              width: 28,
              height: 28,
              alignment: Alignment.center,
              decoration: BoxDecoration(
                color: selected ? color : null,
                border: Border.all(color: selected ? color : AppColors.border),
                borderRadius: BorderRadius.circular(AppRadius.small),
              ),
              child: Text(
                letter,
                style: TextStyle(
                  fontWeight: FontWeight.w700,
                  color: selected
                      ? AppColors.onAction
                      : AppColors.textSecondary,
                ),
              ),
            ),
          ),
        ),
      ),
    ),
  );
}
