/// What a person is on the server.
enum UserRole { admin, user, guest }

/// Who the server takes this client for and what it may do. The interface
/// hides what is not allowed; the server refuses it anyway.
class Rights {
  const Rights({
    required this.role,
    this.userId,
    this.userName,
    this.canControlPlayback = true,
    this.canEditBookmarks = true,
    this.canUpload = true,
    this.canManageRecordings = false,
    this.canManageUsers = false,
  });

  final UserRole role;

  /// Absent for a guest.
  final int? userId;
  final String? userName;
  final bool canControlPlayback;
  final bool canEditBookmarks;
  final bool canUpload;
  final bool canManageRecordings;
  final bool canManageUsers;

  bool get isGuest => role == UserRole.guest;
}

/// What the server tells before anybody signs in.
class AccessStatus {
  const AccessStatus({required this.setupNeeded, required this.guestEnabled});

  /// No user exists yet: the first admin must be set up.
  final bool setupNeeded;
  final bool guestEnabled;
}

/// A completed sign-in; the token resumes it later.
class SignInOutcome {
  const SignInOutcome({required this.token, required this.rights});

  final String token;
  final Rights rights;
}

/// Why signing in or an administration call failed, in terms of what the
/// person can do about it.
sealed class AccessFailure {
  const AccessFailure();
}

/// Wrong user name or password, or the sign-in is no longer valid.
class WrongCredentials extends AccessFailure {
  const WrongCredentials();
}

/// The role may not do this.
class NotAllowed extends AccessFailure {
  const NotAllowed();
}

class TooManyAttempts extends AccessFailure {
  const TooManyAttempts(this.retryAfterSeconds);

  final int retryAfterSeconds;
}

/// The server refused the values; [reason] is meant for the person.
class Refused extends AccessFailure {
  const Refused(this.reason);

  final String reason;
}

/// The server cannot be reached or answered unexpectedly.
class Unavailable extends AccessFailure {
  const Unavailable();
}

/// Thrown by sources; carries the failure for the presentation to explain.
class AccessException implements Exception {
  const AccessException(this.failure);

  final AccessFailure failure;
}
