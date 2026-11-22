import 'dart:typed_data';

import 'time_viewport.dart';

/// One resolution of the waveform: a minimum and maximum amplitude per block
/// of [framesPerBlock] audio frames (`-127..127`, interleaved min, max).
final class WaveformLevel {
  const WaveformLevel({required this.framesPerBlock, required this.peaks});

  final int framesPerBlock;
  final Int8List peaks;

  int get blockCount => peaks.length ~/ 2;
}

/// The waveform of a recording in several resolutions.
final class WaveformData {
  WaveformData({required this.sampleRate, required List<WaveformLevel> levels})
    : levels = List.unmodifiable(
        [...levels]..sort((a, b) => b.framesPerBlock - a.framesPerBlock),
      ) {
    if (levels.isEmpty) {
      throw ArgumentError.value(levels, 'levels', 'must not be empty');
    }
  }

  final int sampleRate;

  /// Coarse to fine.
  final List<WaveformLevel> levels;

  double blockMs(WaveformLevel level) =>
      level.framesPerBlock * 1000 / sampleRate;

  /// The coarsest level that still gives at least one block per pixel, or the
  /// finest one when zoomed in further.
  WaveformLevel levelFor(double msPerPx) {
    for (final level in levels) {
      if (blockMs(level) <= msPerPx) return level;
    }
    return levels.last;
  }
}

/// The amplitude range of every pixel column of a viewport.
final class PeakColumns {
  const PeakColumns({required this.min, required this.max});

  /// `-1..1`; empty where the viewport lies beyond the recording.
  final Float32List min;
  final Float32List max;

  int get length => min.length;
}

/// Reduces the waveform to one minimum and maximum per pixel column of
/// [viewport], using the level that fits its zoom.
PeakColumns computeColumns(WaveformData data, TimeViewport viewport) {
  final width = viewport.widthPx.ceil();
  final minima = Float32List(width);
  final maxima = Float32List(width);
  final level = data.levelFor(viewport.msPerPx);
  final blockMs = data.blockMs(level);
  final lastBlock = level.blockCount - 1;

  for (var column = 0; column < width; column++) {
    final fromBlock = (viewport.xToTime(column.toDouble()) / blockMs).floor();
    final toBlock = ((viewport.xToTime(column + 1.0)) / blockMs).ceil() - 1;
    final first = fromBlock.clamp(0, lastBlock);
    final last = (toBlock < fromBlock ? fromBlock : toBlock).clamp(
      0,
      lastBlock,
    );
    if (viewport.xToTime(column.toDouble()) >= level.blockCount * blockMs) {
      continue;
    }
    var low = 127;
    var high = -127;
    for (var block = first; block <= last; block++) {
      final blockLow = level.peaks[block * 2];
      final blockHigh = level.peaks[block * 2 + 1];
      if (blockLow < low) low = blockLow;
      if (blockHigh > high) high = blockHigh;
    }
    minima[column] = low / 127;
    maxima[column] = high / 127;
  }
  return PeakColumns(min: minima, max: maxima);
}
