import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:material_ui/material_ui.dart';

import '../../shared_ui/action_button.dart';
import '../../shared_ui/activity_status.dart';
import 'access_card.dart';
import 'access_controller.dart';
import 'access_state.dart';
import 'access_texts.dart';
import 'setup_form.dart';
import 'sign_in_form.dart';

/// Shows what the person has to do before the app (set up, sign in) and the
/// app itself once they are in. [builder] runs only while signed in.
class AccessGate extends ConsumerWidget {
  const AccessGate({required this.builder, super.key});

  final WidgetBuilder builder;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final controller = ref.read(accessControllerProvider.notifier);
    return switch (ref.watch(accessControllerProvider)) {
      AccessChecking() => const _Checking(),
      AccessUnreachable() => _Unreachable(onRetry: controller.start),
      AccessSetupNeeded(:final busy, :final failure) => SetupForm(
        busy: busy,
        failure: failure,
        onSetUp: controller.setUp,
      ),
      AccessSignedOut(:final guestEnabled, :final busy, :final failure) =>
        SignInForm(
          guestEnabled: guestEnabled,
          busy: busy,
          failure: failure,
          onSignIn: controller.signIn,
          onGuest: controller.continueAsGuest,
        ),
      AccessSignedIn() => builder(context),
    };
  }
}

class _Checking extends StatelessWidget {
  const _Checking();

  @override
  Widget build(BuildContext context) => Scaffold(
    body: Center(
      child: ActivityStatus(
        state: ActivityState.loading,
        label: AccessTexts.checking,
      ),
    ),
  );
}

class _Unreachable extends StatelessWidget {
  const _Unreachable({required this.onRetry});

  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) => AccessCard(
    title: AccessTexts.unreachable,
    child: ActionButton(label: AccessTexts.retry, onPressed: onRetry),
  );
}
