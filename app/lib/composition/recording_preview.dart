import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../features/waveform/time_viewport.dart';
import '../features/waveform/waveform_renderer.dart';
import '../features/waveform/waveform_source.dart';

/// The whole waveform of a recording as a small picture for the library's
/// details. Only for a scanned recording; it takes no input.
class RecordingPreview extends ConsumerWidget {
  const RecordingPreview({
    required this.recordingId,
    required this.lengthMs,
    super.key,
  });

  final int recordingId;
  final int lengthMs;

  static const _height = 96.0;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final loaded = ref
        .watch(
          recordingWaveformProvider((recordingId: recordingId, scanned: true)),
        )
        .value;
    return ExcludeSemantics(
      child: LayoutBuilder(
        builder: (context, constraints) => WaveformRenderer(
          data: loaded?.data,
          viewport: TimeViewport.fit(
            durationMs: loaded?.lengthMs ?? lengthMs,
            widthPx: constraints.maxWidth,
          ),
          height: _height,
        ),
      ),
    );
  }
}
