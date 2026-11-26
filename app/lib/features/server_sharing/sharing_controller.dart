import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'sharing_source.dart';
import 'sharing_models.dart';

/// Supplies the [SharingSource]; `null` where the application carries no
/// server, as in a browser. Overridden at the application root.
final sharingSourceProvider = Provider<SharingSource?>((ref) => null);

final sharingProvider =
    AsyncNotifierProvider.autoDispose<SharingController, SharingStatus>(
      SharingController.new,
    );

class SharingController extends AsyncNotifier<SharingStatus> {
  SharingSource get _source => ref.read(sharingSourceProvider)!;

  @override
  Future<SharingStatus> build() => _source.status();

  Future<void> apply(SharingSettings settings) async {
    final result = await AsyncValue.guard(() => _source.apply(settings));
    if (ref.mounted) state = result;
  }
}
