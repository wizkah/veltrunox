import 'package:material_ui/material_ui.dart';

import '../../app_shell/brand_mark.dart';
import '../../design/tokens/app_geometry.dart';

/// The frame of the screens before the app: brand, heading and the content,
/// centred and no wider than a phone.
class AccessCard extends StatelessWidget {
  const AccessCard({required this.title, required this.child, super.key});

  final String title;
  final Widget child;

  @override
  Widget build(BuildContext context) => Scaffold(
    body: SafeArea(
      child: Center(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(AppSpace.s4),
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 400),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                const Center(child: BrandMark()),
                const SizedBox(height: AppSpace.s6),
                Semantics(
                  header: true,
                  child: Text(
                    title,
                    style: Theme.of(context).textTheme.headlineSmall,
                  ),
                ),
                const SizedBox(height: AppSpace.s4),
                child,
              ],
            ),
          ),
        ),
      ),
    ),
  );
}
