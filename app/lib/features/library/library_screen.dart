import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_colors.dart';
import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/frame_decoration.dart';
import '../../shared_ui/state_notice.dart';
import '../access/access_controller.dart';
import 'library_controller.dart';
import 'library_failure.dart';
import 'library_query.dart';
import 'library_query_controller.dart';
import 'library_texts.dart';
import 'library_toolbar.dart';
import 'upload_bar.dart';
import 'recording_details.dart';
import 'recording_flows.dart';
import 'recording_list.dart';
import 'recording_summary.dart';

/// The library: a large song list with search, order and filter. On a wide
/// screen the details of a recording stand beside the list; on a narrow one
/// they replace it. Opening or playing a recording is left to the caller.
class LibraryScreen extends ConsumerWidget {
  const LibraryScreen({
    required this.loadedId,
    required this.onOpenInPlayer,
    required this.onPlay,
    this.previewBuilder,
    this.exchange,
    super.key,
  });

  /// The recording the player has loaded.
  final int? loadedId;
  final ValueChanged<int> onOpenInPlayer;
  final ValueChanged<int> onPlay;

  /// Builds the picture of a recording shown above its details.
  final Widget Function(RecordingSummary recording)? previewBuilder;

  /// What the caller offers beside the upload, for example the exchange menu.
  final Widget? exchange;

  /// From this width the details stand beside the list.
  static const _detailsBesideMin = AppBreakpoint.desktopMin;
  static const _detailsWidth = 400.0;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final library = ref.watch(libraryProvider);
    return library.when(
      data: (recordings) => _loaded(context, ref, recordings),
      loading: () => const _Loading(),
      error: (_, _) => LibraryFailure(
        onRetry: () => ref.read(libraryProvider.notifier).reload(),
      ),
    );
  }

  Widget _loaded(
    BuildContext context,
    WidgetRef ref,
    List<RecordingSummary> recordings,
  ) {
    final query = ref.watch(libraryQueryProvider);
    final detailId = ref.watch(libraryDetailProvider);
    final detailed = recordings.where((recording) => recording.id == detailId);
    final details = detailed.isEmpty ? null : detailed.first;
    return LayoutBuilder(
      builder: (context, constraints) {
        final beside = constraints.maxWidth >= _detailsBesideMin;
        if (!beside && details != null) {
          return _details(context, ref, details, withBack: true);
        }
        final list = _list(context, ref, recordings, query, detailId);
        if (!beside) return list;
        return Row(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Expanded(child: list),
            SizedBox(
              width: _detailsWidth,
              child: Padding(
                padding: const EdgeInsets.fromLTRB(
                  0,
                  AppSpace.s4,
                  AppSpace.s4,
                  AppSpace.s4,
                ),
                child: DecoratedBox(
                  decoration: frameDecoration(),
                  child: details == null
                      ? const _NoDetails()
                      : _details(context, ref, details, withBack: false),
                ),
              ),
            ),
          ],
        );
      },
    );
  }

  static bool _manages(WidgetRef ref) =>
      ref.watch(rightsProvider)?.canManageRecordings ?? false;

  Widget _list(
    BuildContext context,
    WidgetRef ref,
    List<RecordingSummary> recordings,
    LibraryQuery query,
    int? detailId,
  ) {
    final queryController = ref.read(libraryQueryProvider.notifier);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(
            AppSpace.s4,
            AppSpace.s4,
            AppSpace.s4,
            AppSpace.s2,
          ),
          child: Text(
            LibraryTexts.title,
            style: Theme.of(context).textTheme.headlineMedium,
          ),
        ),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: AppSpace.s4),
          child: LibraryToolbar(
            query: query,
            onSearch: queryController.search,
            onSort: queryController.sortBy,
            onFilter: queryController.filterBy,
          ),
        ),
        if ((ref.watch(rightsProvider)?.canUpload ?? false) || exchange != null)
          Padding(
            padding: const EdgeInsets.fromLTRB(
              AppSpace.s4,
              AppSpace.s3,
              AppSpace.s4,
              0,
            ),
            // The upload bar keeps the full width it always had: it shows the
            // progress of a running upload, which must not be squeezed.
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Expanded(
                  child: (ref.watch(rightsProvider)?.canUpload ?? false)
                      ? const UploadBar()
                      : const SizedBox.shrink(),
                ),
                ?exchange,
              ],
            ),
          ),
        const SizedBox(height: AppSpace.s2),
        Expanded(
          child: RecordingList(
            recordings: applyQuery(recordings, query),
            detailId: detailId,
            loadedId: loadedId,
            narrowed: query.narrows,
            onShowDetails: ref.read(libraryDetailProvider.notifier).show,
            onPlay: onPlay,
          ),
        ),
      ],
    );
  }

  Widget _details(
    BuildContext context,
    WidgetRef ref,
    RecordingSummary recording, {
    required bool withBack,
  }) => RecordingDetails(
    recording: recording,
    preview: previewBuilder?.call(recording),
    onOpenInPlayer: () => onOpenInPlayer(recording.id),
    onPlay: () => onPlay(recording.id),
    onReanalyse: () =>
        ref.read(libraryProvider.notifier).reanalyse(recording.id),
    onRename: _manages(ref)
        ? () => renameRecordingFlow(context, ref, recording)
        : null,
    onDelete: _manages(ref)
        ? () => deleteRecordingFlow(context, ref, recording)
        : null,
    onBack: withBack ? ref.read(libraryDetailProvider.notifier).close : null,
  );
}

/// Stands in the detail panel while no recording is chosen.
class _NoDetails extends StatelessWidget {
  const _NoDetails();

  @override
  Widget build(BuildContext context) => Center(
    child: Padding(
      padding: const EdgeInsets.all(AppSpace.s6),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(
            Icons.library_music_outlined,
            size: AppSpace.s8 + AppSpace.s4,
            color: AppColors.accentSoft,
          ),
          const SizedBox(height: AppSpace.s3),
          Text(
            LibraryTexts.chooseForDetails,
            textAlign: TextAlign.center,
            style: Theme.of(context).textTheme.bodySmall,
          ),
        ],
      ),
    ),
  );
}

class _Loading extends StatelessWidget {
  const _Loading();

  @override
  Widget build(BuildContext context) =>
      StateNotice(message: LibraryTexts.loading, busy: true);
}
