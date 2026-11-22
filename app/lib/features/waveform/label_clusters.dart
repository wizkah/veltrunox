import '../markers/marker.dart';
import 'time_viewport.dart';

/// Labels whose chips would collide, shown as one chip with a `+N` count.
final class LabelCluster {
  const LabelCluster({required this.labels, required this.xPx});

  /// Chronological; the first one names the chip.
  final List<LabelMarker> labels;

  /// Pixel position of the first label's marker line.
  final double xPx;

  bool get isGroup => labels.length > 1;
  LabelMarker get first => labels.first;
  int get hiddenCount => labels.length - 1;
}

/// Groups the labels visible in [viewport] so that no two chips overlap.
///
/// [chipWidthPx] gives the width of a label's chip; [gapPx] is the least space
/// between neighbouring chips. Labels outside the viewport are left out.
List<LabelCluster> clusterLabels(
  List<LabelMarker> labels,
  TimeViewport viewport, {
  required double Function(LabelMarker label) chipWidthPx,
  double gapPx = 4,
}) {
  final visible =
      labels
          .where((label) => viewport.isVisible(label.positionMs.toDouble()))
          .toList()
        ..sort(_byPositionThenId);

  final clusters = <LabelCluster>[];
  var members = <LabelMarker>[];
  var startX = 0.0;
  var rightEdge = 0.0;

  void flush() {
    if (members.isNotEmpty) {
      clusters.add(LabelCluster(labels: members, xPx: startX));
    }
  }

  for (final label in visible) {
    final x = viewport.timeToX(label.positionMs.toDouble());
    if (members.isNotEmpty && x < rightEdge + gapPx) {
      members.add(label);
      continue;
    }
    flush();
    members = [label];
    startX = x;
    rightEdge = x + chipWidthPx(label);
  }
  flush();
  return clusters;
}

int _byPositionThenId(LabelMarker a, LabelMarker b) {
  final byPosition = a.positionMs.compareTo(b.positionMs);
  return byPosition != 0 ? byPosition : a.id.compareTo(b.id);
}
