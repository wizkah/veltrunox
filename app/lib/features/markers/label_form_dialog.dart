import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import '../../shared_ui/clock_format.dart';
import 'marker_form_validation.dart';
import 'marker_texts.dart';
import 'time_field.dart';

/// What the label form hands back: a checked name and position.
typedef LabelDraft = ({String title, int positionMs});

/// The form that renames a label or moves it to another position.
class LabelFormDialog extends StatefulWidget {
  const LabelFormDialog({
    required this.initialTitle,
    required this.initialPositionMs,
    required this.lengthMs,
    super.key,
  });

  final String initialTitle;
  final int initialPositionMs;

  /// The recording's length, if known; positions beyond it are refused.
  final int? lengthMs;

  /// Resolves to the draft, or `null` if the user cancelled.
  static Future<LabelDraft?> show(
    BuildContext context, {
    required String title,
    required int positionMs,
    required int? lengthMs,
  }) => showDialog<LabelDraft>(
    context: context,
    builder: (_) => LabelFormDialog(
      initialTitle: title,
      initialPositionMs: positionMs,
      lengthMs: lengthMs,
    ),
  );

  @override
  State<LabelFormDialog> createState() => _LabelFormDialogState();
}

class _LabelFormDialogState extends State<LabelFormDialog> {
  late final _title = TextEditingController(text: widget.initialTitle);
  late final _position = TextEditingController(
    text: formatClockExact(widget.initialPositionMs),
  );
  String? _titleError;
  String? _positionError;

  @override
  void dispose() {
    _title.dispose();
    _position.dispose();
    super.dispose();
  }

  void _submit() {
    final title = checkName(_title.text);
    final position = checkTime(_position.text, lengthMs: widget.lengthMs);
    setState(() {
      _titleError = title.error;
      _positionError = position.error;
    });
    if (title.value == null || position.value == null) return;
    Navigator.of(context)
        .pop((title: title.value!, positionMs: position.value!));
  }

  @override
  Widget build(BuildContext context) => AlertDialog(
    title: const Text(MarkerTexts.editLabel),
    content: SingleChildScrollView(
      child: SizedBox(
        width: 360,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const SizedBox(height: AppSpace.s1),
            TextField(
              controller: _title,
              autofocus: true,
              decoration: InputDecoration(
                labelText: MarkerTexts.name,
                errorText: _titleError,
              ),
              textInputAction: TextInputAction.next,
            ),
            const SizedBox(height: AppSpace.s3),
            TimeField(
              controller: _position,
              label: MarkerTexts.position,
              errorText: _positionError,
              textInputAction: TextInputAction.done,
              onSubmitted: _submit,
            ),
          ],
        ),
      ),
    ),
    actions: [
      ActionButton(
        label: MarkerTexts.cancel,
        kind: ActionKind.secondary,
        onPressed: () => Navigator.of(context).pop(),
      ),
      ActionButton(label: MarkerTexts.save, onPressed: _submit),
    ],
  );
}
