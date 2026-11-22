import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import '../../shared_ui/clock_format.dart';
import 'marker_form_validation.dart';
import 'marker_texts.dart';
import 'time_field.dart';

/// What the region form hands back: a checked name and span.
typedef RegionDraft = ({String title, int startMs, int endMs});

/// The form that creates a region from a name, a start and an end, or edits
/// the same values of an existing one. Invalid input shows a message at its
/// field and is not handed back.
class RegionFormDialog extends StatefulWidget {
  const RegionFormDialog({
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
  final int initialEndMs;

  /// The recording's length, if known; times beyond it are refused.
  final int? lengthMs;

  /// Resolves to the draft, or `null` if the user cancelled.
  static Future<RegionDraft?> show(
    BuildContext context, {
    required String heading,
    required int startMs,
    required int endMs,
    required int? lengthMs,
    String title = '',
  }) => showDialog<RegionDraft>(
    context: context,
    builder: (_) => RegionFormDialog(
      heading: heading,
      initialTitle: title,
      initialStartMs: startMs,
      initialEndMs: endMs,
      lengthMs: lengthMs,
    ),
  );

  @override
  State<RegionFormDialog> createState() => _RegionFormDialogState();
}

class _RegionFormDialogState extends State<RegionFormDialog> {
  late final _title = TextEditingController(text: widget.initialTitle);
  late final _start = TextEditingController(
    text: formatClockExact(widget.initialStartMs),
  );
  late final _end = TextEditingController(
    text: formatClockExact(widget.initialEndMs),
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
    final end = checkRegionEnd(
      _end.text,
      startMs: start.value,
      lengthMs: widget.lengthMs,
    );
    setState(() {
      _titleError = title.error;
      _startError = start.error;
      _endError = end.error;
    });
    if (title.value == null || start.value == null || end.value == null) return;
    Navigator.of(context)
        .pop((title: title.value!, startMs: start.value!, endMs: end.value!));
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
