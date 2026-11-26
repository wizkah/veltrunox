import 'sharing_models.dart';

/// Port: the server that comes with the application.
abstract interface class SharingSource {
  Future<SharingStatus> status();

  /// Switches to [settings]; the status tells what is in effect afterwards.
  Future<SharingStatus> apply(SharingSettings settings);
}
