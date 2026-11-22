import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import '../../shared_ui/clock_format.dart';
import 'marker_form_validation.dart';
import 'marker_texts.dart';
import 'time_field.dart';

/// What the marker form hands back: a checked name, a start and an optional
/// end. Without an end the marker is a label, with one it is a region.
typedef MarkerDraft = ({String title, int startMs, int? endMs});

/// The form that creates a marker or edits one. A label and a region differ
/// only in the end, so leaving the end empty makes a label and filling it in
/// makes a region, also when it changes an existing marker. Invalid input shows
/// a message at its field and is not handed back.
class MarkerFormDialog extends StatefulWidget {
  const MarkerFormDialog({
    required this.heading,
    required this.initialTitle,
    required this.initialStartMs,
    required this.initialEndMs,
    required this.lengthMs,
    super.key,
  });

  final String heading;
  final String initialTitle;
  final int initialStartMs;

  /// `null` for a label.
  final int? initialEndMs;

  /// The recording's length, if known; times beyond it are refused.
  final int? lengthMs;

  /// Resolves to the draft, or `null` if the user cancelled.
  static Future<MarkerDraft?> show(
    BuildContext context, {
    required String heading,
    required int startMs,
    required int? endMs,
    required int? lengthMs,
    String title = '',
  }) => showDialog<MarkerDraft>(
    context: context,
    builder: (_) => MarkerFormDialog(
      heading: heading,
      initialTitle: title,
      initialStartMs: startMs,
      initialEndMs: endMs,
      lengthMs: lengthMs,
    ),
  );

  @override
  State<MarkerFormDialog> createState() => _MarkerFormDialogState();
}

class _MarkerFormDialogState extends State<MarkerFormDialog> {
  late final _title = TextEditingController(text: widget.initialTitle);
  late final _start = TextEditingController(
    text: formatClockExact(widget.initialStartMs),
  );
  late final _end = TextEditingController(
    text: widget.initialEndMs == null
        ? ''
        : formatClockExact(widget.initialEndMs!),
  );
  String? _titleError;
  String? _startError;
  String? _endError;

  @override
  void dispose() {
    _title.dispose();
    _start.dispose();
    _end.dispose();
    super.dispose();
  }

  void _submit() {
    final title = checkName(_title.text);
    final start = checkTime(_start.text, lengthMs: widget.lengthMs);
    final hasEnd = _end.text.trim().isNotEmpty;
    final end = hasEnd
        ? checkRegionEnd(
            _end.text,
            startMs: start.value,
            lengthMs: widget.lengthMs,
          )
        : (value: null, error: null);
    setState(() {
      _titleError = title.error;
      _startError = start.error;
      _endError = end.error;
    });
    if (title.value == null || start.value == null || end.error != null) {
      return;
    }
    Navigator.of(context)
        .pop((title: title.value!, startMs: start.value!, endMs: end.value));
  }

  @override
  Widget build(BuildContext context) => AlertDialog(
    title: Text(widget.heading),
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
              controller: _start,
              label: MarkerTexts.start,
              errorText: _startError,
            ),
            const SizedBox(height: AppSpace.s3),
            TimeField(
              controller: _end,
              label: MarkerTexts.end,
              helperText: MarkerTexts.endOptional,
              errorText: _endError,
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
