import 'package:material_ui/material_ui.dart';

import 'player_texts.dart';
import 'tempo_input.dart';

/// Shows the tempo the recording plays at and takes a wanted tempo.
class TargetTempoField extends StatefulWidget {
  const TargetTempoField({
    required this.currentBpm,
    required this.onSubmit,
    super.key,
  });

  final double currentBpm;

  /// Reports whether the tempo could be set.
  final Future<bool> Function(double bpm) onSubmit;

  @override
  State<TargetTempoField> createState() => _TargetTempoFieldState();
}

class _TargetTempoFieldState extends State<TargetTempoField> {
  final _controller = TextEditingController();
  String? _error;

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _submit(String text) async {
    final bpm = parseBpm(text);
    if (bpm == null) {
      setState(() => _error = PlayerTexts.tempoInvalid);
      return;
    }
    final set = await widget.onSubmit(bpm);
    if (!mounted) return;
    setState(() => _error = set ? null : PlayerTexts.tempoOutOfRange);
    if (set) _controller.clear();
  }

  @override
  Widget build(BuildContext context) => Row(
    children: [
      const SizedBox(width: 120, child: Text(PlayerTexts.tempo)),
      Text(PlayerTexts.tempoValue(widget.currentBpm)),
      const SizedBox(width: 16),
      SizedBox(
        width: 140,
        child: TextField(
          controller: _controller,
          keyboardType: const TextInputType.numberWithOptions(decimal: true),
          decoration: InputDecoration(
            labelText: PlayerTexts.targetTempo,
            errorText: _error,
            isDense: true,
          ),
          onSubmitted: _submit,
        ),
      ),
    ],
  );
}
