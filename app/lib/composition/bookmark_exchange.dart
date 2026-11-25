import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../features/access/access_controller.dart';
import '../features/exchange/exchange_menu.dart';
import '../features/exchange/export_controller.dart';
import '../features/library/library_controller.dart';

/// The exchange menu wired to the library (titles) and the person's rights:
/// for one recording, or for the whole library when [recordingId] is `null`.
class BookmarkExchangeMenu extends ConsumerWidget {
  const BookmarkExchangeMenu({this.recordingId, super.key});

  final int? recordingId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final recordings = ref.watch(libraryProvider).value ?? const [];
    String? titleOf(int id) {
      for (final recording in recordings) {
        if (recording.id == id) return recording.title;
      }
      return null;
    }

    final id = recordingId;
    return ExchangeMenu(
      scope: id == null
          ? const ExportScope.library()
          : ExportScope.recording(
              id: id,
              title: titleOf(id) ?? 'recording-$id',
            ),
      canImport: ref.watch(rightsProvider)?.canEditBookmarks ?? false,
      titleOf: titleOf,
    );
  }
}
