import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import 'sharing_texts.dart';

/// Where browsers reach the running server, to read out or copy.
class SharingAddresses extends StatelessWidget {
  const SharingAddresses({required this.addresses, super.key});

  final List<String> addresses;

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Text(
        SharingTexts.addresses,
        style: Theme.of(context).textTheme.titleMedium,
      ),
      const SizedBox(height: AppSpace.s2),
      for (final address in addresses) SelectableText(address),
    ],
  );
}
