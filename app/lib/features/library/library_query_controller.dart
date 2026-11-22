import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'library_query.dart';

class LibraryQueryNotifier extends Notifier<LibraryQuery> {
  @override
  LibraryQuery build() => const LibraryQuery();

  void search(String text) => state = state.copyWith(search: text);
  void sortBy(LibrarySort sort) => state = state.copyWith(sort: sort);
  void filterBy(LibraryFilter filter) => state = state.copyWith(filter: filter);
}

/// What the user asked of the list; local to this client.
final libraryQueryProvider =
    NotifierProvider<LibraryQueryNotifier, LibraryQuery>(
      LibraryQueryNotifier.new,
    );

class DetailSelectionNotifier extends Notifier<int?> {
  @override
  int? build() => null;

  void show(int recordingId) => state = recordingId;
  void close() => state = null;
}

/// The recording whose details the library shows; not the one the player has
/// loaded.
final libraryDetailProvider = NotifierProvider<DetailSelectionNotifier, int?>(
  DetailSelectionNotifier.new,
);
