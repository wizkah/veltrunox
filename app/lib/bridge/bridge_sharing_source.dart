import '../features/server_sharing/sharing_models.dart';
import '../features/server_sharing/sharing_source.dart';
import 'generated/host_api.dart' as rust;

/// Adapter: the server inside the desktop application.
class BridgeSharingSource implements SharingSource {
  @override
  Future<SharingStatus> status() async => _status(await rust.sharingInfo());

  @override
  Future<SharingStatus> apply(SharingSettings settings) async {
    try {
      return _status(
        await rust.applySharing(
          choice: rust.SharingChoice(
            enabled: settings.enabled,
            port: settings.port,
            https: settings.https,
          ),
        ),
      );
    } on rust.SharingProblem catch (problem) {
      // Refused: the server remembers what was asked and tells that it is off.
      return _status(await rust.sharingInfo(), refused: _problem(problem));
    }
  }

  SharingStatus _status(rust.SharingInfo info, {SharingProblem? refused}) {
    final problem = info.problem;
    return SharingStatus(
      settings: SharingSettings(
        enabled: info.choice.enabled,
        port: info.choice.port,
        https: info.choice.https,
      ),
      addresses: info.urls,
      problem: refused ?? (problem == null ? null : _problem(problem)),
      hasWebApp: info.hasWebApp,
    );
  }

  static SharingProblem _problem(rust.SharingProblem problem) =>
      switch (problem) {
        rust.SharingProblem_NoAccount() => const NoAccountYet(),
        rust.SharingProblem_PortInUse() => const PortInUse(),
        rust.SharingProblem_Failed(:final detail) => SharingFailed(detail),
      };
}
