import 'package:flutter_riverpod/misc.dart';
import 'package:veltrunox_app/features/access/access_controller.dart';
import 'package:veltrunox_app/features/access/access_models.dart';
import 'package:veltrunox_app/features/access/access_source.dart';

/// A sign-in that the tests steer: what the server says, and what it refuses.
class FakeAccessSource implements AccessSource {
  FakeAccessSource({
    this.setupNeeded = false,
    this.guestEnabled = false,
    this.rights = const Rights(role: UserRole.user, userName: 'ann'),
    this.failure,
    this.unreachable = false,
  });

  bool setupNeeded;
  bool guestEnabled;
  Rights rights;

  /// Refuses the next sign-in calls with this.
  AccessFailure? failure;
  bool unreachable;

  final calls = <String>[];
  int sessionsStarted = 0;

  Never _refuse(AccessFailure failure) => throw AccessException(failure);

  @override
  Future<AccessStatus> status() async {
    if (unreachable) _refuse(const Unavailable());
    return AccessStatus(setupNeeded: setupNeeded, guestEnabled: guestEnabled);
  }

  @override
  Future<SignInOutcome> setUpAdmin(String userName, String password) async {
    calls.add('setUp:$userName');
    if (failure != null) _refuse(failure!);
    setupNeeded = false;
    return SignInOutcome(token: 'new-token', rights: rights);
  }

  @override
  Future<SignInOutcome> signIn(String userName, String password) async {
    calls.add('signIn:$userName:$password');
    if (failure != null) _refuse(failure!);
    return SignInOutcome(token: 'token', rights: rights);
  }

  @override
  Future<Rights> resume(String token) async {
    calls.add('resume:$token');
    if (failure != null) _refuse(failure!);
    return rights;
  }

  @override
  Future<Rights> continueAsGuest() async {
    calls.add('guest');
    if (failure != null) _refuse(failure!);
    return const Rights(role: UserRole.guest);
  }

  @override
  Future<void> signOut() async => calls.add('signOut');

  @override
  Future<void> startSession() async => sessionsStarted++;
}

class MemoryTokenStore implements TokenStore {
  MemoryTokenStore([this.token]);

  String? token;

  @override
  Future<String?> read() async => token;

  @override
  Future<void> write(String token) async => this.token = token;

  @override
  Future<void> clear() async => token = null;
}

/// Overrides that let the app start already signed in as the given person.
List<Override> signedInAs([
  Rights rights = const Rights(role: UserRole.user, userName: 'ann'),
]) => [
  accessSourceProvider.overrideWithValue(FakeAccessSource(rights: rights)),
  tokenStoreProvider.overrideWithValue(MemoryTokenStore('saved')),
];
