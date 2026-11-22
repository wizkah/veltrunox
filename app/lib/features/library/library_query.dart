import 'analysis_format.dart';
import 'analysis_info.dart';
import 'recording_summary.dart';

/// How the list is ordered.
enum LibrarySort { title, tempo, key }

/// Which recordings the list shows, by the state of their analysis.
enum LibraryFilter { all, analysed, analysing, failed }

/// What the user asked of the list: a search text, an order and a filter.
class LibraryQuery {
  const LibraryQuery({
    this.search = '',
    this.sort = LibrarySort.title,
    this.filter = LibraryFilter.all,
  });

  final String search;
  final LibrarySort sort;
  final LibraryFilter filter;

  LibraryQuery copyWith({
    String? search,
    LibrarySort? sort,
    LibraryFilter? filter,
  }) => LibraryQuery(
    search: search ?? this.search,
    sort: sort ?? this.sort,
    filter: filter ?? this.filter,
  );

  /// Whether the user narrowed the list down (so an empty result is a miss, not an empty library).
  bool get narrows => search.trim().isNotEmpty || filter != LibraryFilter.all;
}

/// The recordings that fit the [query], in its order.
List<RecordingSummary> applyQuery(
  Iterable<RecordingSummary> recordings,
  LibraryQuery query,
) {
  final words = query.search
      .toLowerCase()
      .split(RegExp(r'\s+'))
      .where((word) => word.isNotEmpty)
      .toList();
  final shown = recordings
      .where(
        (recording) =>
            _passes(recording, query.filter) && _matches(recording, words),
      )
      .toList();
  shown.sort((a, b) {
    final byOrder = switch (query.sort) {
      LibrarySort.title => 0,
      LibrarySort.tempo => _compareValue(
        a.analysis.values?.tempoBpm,
        b.analysis.values?.tempoBpm,
      ),
      LibrarySort.key => _compareValue(_keyRank(a), _keyRank(b)),
    };
    return byOrder != 0 ? byOrder : _compareTitle(a, b);
  });
  return shown;
}

bool _passes(RecordingSummary recording, LibraryFilter filter) {
  final status = recording.analysis.status;
  return switch (filter) {
    LibraryFilter.all => true,
    LibraryFilter.analysed => status == AnalysisStatus.done,
    LibraryFilter.analysing =>
      status == AnalysisStatus.queued || status == AnalysisStatus.running,
    LibraryFilter.failed => status == AnalysisStatus.failed,
  };
}

/// Every word of the search must be found in the title or in the key and tempo.
bool _matches(RecordingSummary recording, List<String> words) {
  if (words.isEmpty) return true;
  final haystack = '${recording.title} ${analysisLine(recording.analysis)}'
      .toLowerCase();
  return words.every(haystack.contains);
}

/// Recordings without a value come last.
int _compareValue(num? a, num? b) {
  if (a == null && b == null) return 0;
  if (a == null) return 1;
  if (b == null) return -1;
  return a.compareTo(b);
}

int? _keyRank(RecordingSummary recording) {
  final values = recording.analysis.values;
  return values == null
      ? null
      : values.keyTonic * 2 + (values.keyMinor ? 1 : 0);
}

int _compareTitle(RecordingSummary a, RecordingSummary b) {
  final byTitle = a.title.toLowerCase().compareTo(b.title.toLowerCase());
  return byTitle != 0 ? byTitle : a.id.compareTo(b.id);
}
