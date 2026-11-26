/// What the person asks of the server for the network.
class SharingSettings {
  const SharingSettings({
    required this.enabled,
    required this.port,
    required this.https,
  });

  final bool enabled;
  final int port;
  final bool https;
}

/// Why the server for the network is not on.
sealed class SharingProblem {
  const SharingProblem();
}

/// Without a user account the first browser could take over the server.
class NoAccountYet extends SharingProblem {
  const NoAccountYet();
}

class PortInUse extends SharingProblem {
  const PortInUse();
}

class SharingFailed extends SharingProblem {
  const SharingFailed(this.detail);

  final String detail;
}

class SharingStatus {
  const SharingStatus({
    required this.settings,
    this.addresses = const [],
    this.problem,
    this.hasWebApp = true,
  });

  /// What is in effect and remembered.
  final SharingSettings settings;

  /// Where browsers reach the server; empty while it is off.
  final List<String> addresses;

  final SharingProblem? problem;

  /// Whether browsers get the web app, or only the interface for scripts.
  final bool hasWebApp;
}
