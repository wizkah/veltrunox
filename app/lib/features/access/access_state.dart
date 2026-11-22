import 'access_models.dart';

/// Where the person stands before the app: checking, setting up, signing in
/// or in.
sealed class AccessState {
  const AccessState();
}

class AccessChecking extends AccessState {
  const AccessChecking();
}

/// The server cannot be reached at the start.
class AccessUnreachable extends AccessState {
  const AccessUnreachable();
}

/// First start of the server: the first admin has to be set up.
class AccessSetupNeeded extends AccessState {
  const AccessSetupNeeded({this.busy = false, this.failure});

  final bool busy;
  final AccessFailure? failure;
}

class AccessSignedOut extends AccessState {
  const AccessSignedOut({
    required this.guestEnabled,
    this.busy = false,
    this.failure,
  });

  final bool guestEnabled;
  final bool busy;
  final AccessFailure? failure;
}

class AccessSignedIn extends AccessState {
  const AccessSignedIn(this.rights);

  final Rights rights;
}
