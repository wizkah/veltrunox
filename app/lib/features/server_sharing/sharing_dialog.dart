import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import '../../shared_ui/activity_status.dart';
import 'sharing_addresses.dart';
import 'sharing_controller.dart';
import 'sharing_form.dart';
import 'sharing_models.dart';
import 'sharing_texts.dart';

/// The settings of the server that comes with the application.
class SharingDialog extends ConsumerWidget {
  const SharingDialog({required this.onManageUsers, super.key});

  /// Opens users and guest access; completes when that is closed again.
  final Future<void> Function() onManageUsers;

  static Future<void> show(
    BuildContext context, {
    required Future<void> Function() onManageUsers,
  }) => showDialog<void>(
    context: context,
    builder: (_) => SharingDialog(onManageUsers: onManageUsers),
  );

  @override
  Widget build(BuildContext context, WidgetRef ref) => AlertDialog(
    title: Text(SharingTexts.title),
    content: SizedBox(
      width: 480,
      child: switch (ref.watch(sharingProvider)) {
        AsyncData(:final value) => SingleChildScrollView(
          child: _content(context, ref, value),
        ),
        AsyncError() => Text(SharingTexts.failure),
        _ => ActivityStatus(
          state: ActivityState.loading,
          label: SharingTexts.loading,
        ),
      },
    ),
    actions: [
      TextButton(
        onPressed: () => Navigator.of(context).pop(),
        child: Text(SharingTexts.close),
      ),
    ],
  );

  Widget _content(BuildContext context, WidgetRef ref, SharingStatus status) =>
      Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (status.problem case final problem?) ...[
            Semantics(
              liveRegion: true,
              child: Text(
                SharingTexts.problem(problem),
                style: TextStyle(color: Theme.of(context).colorScheme.error),
              ),
            ),
            if (problem is NoAccessYet)
              ActionButton(
                label: SharingTexts.manageUsers,
                onPressed: () async {
                  await onManageUsers();
                  // The reason may be gone now.
                  if (context.mounted) ref.invalidate(sharingProvider);
                },
              ),
            const SizedBox(height: AppSpace.s3),
          ],
          SharingForm(
            settings: status.settings,
            onApply: ref.read(sharingProvider.notifier).apply,
          ),
          if (status.addresses.isNotEmpty) ...[
            const SizedBox(height: AppSpace.s4),
            SharingAddresses(addresses: status.addresses),
          ],
          if (!status.hasWebApp) Text(SharingTexts.noWebApp),
        ],
      );
}
