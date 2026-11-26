import 'package:flutter/services.dart';
import 'package:material_ui/material_ui.dart';

import '../../design/tokens/app_geometry.dart';
import '../../shared_ui/action_button.dart';
import 'sharing_models.dart';
import 'sharing_texts.dart';

/// The switch of the server with its port and encryption. The switch applies
/// at once; changes to a running server apply with the button.
class SharingForm extends StatefulWidget {
  const SharingForm({required this.settings, required this.onApply, super.key});

  final SharingSettings settings;
  final ValueChanged<SharingSettings> onApply;

  @override
  State<SharingForm> createState() => _SharingFormState();
}

class _SharingFormState extends State<SharingForm> {
  late final _port = TextEditingController(text: '${widget.settings.port}');
  late bool _https = widget.settings.https;

  @override
  void dispose() {
    _port.dispose();
    super.dispose();
  }

  int? get _validPort {
    final port = int.tryParse(_port.text);
    return port != null && port >= 1024 && port <= 65535 ? port : null;
  }

  bool get _changed =>
      _validPort != widget.settings.port || _https != widget.settings.https;

  void _apply({required bool enabled}) {
    final port = _validPort;
    if (port == null) return;
    widget.onApply(
      SharingSettings(enabled: enabled, port: port, https: _https),
    );
  }

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      SwitchListTile(
        title: Text(SharingTexts.enable),
        subtitle: Text(SharingTexts.enableHint),
        value: widget.settings.enabled,
        onChanged: _validPort == null
            ? null
            : (enabled) => _apply(enabled: enabled),
      ),
      const SizedBox(height: AppSpace.s3),
      TextField(
        controller: _port,
        keyboardType: TextInputType.number,
        inputFormatters: [FilteringTextInputFormatter.digitsOnly],
        decoration: InputDecoration(
          labelText: SharingTexts.port,
          errorText: _validPort == null ? SharingTexts.portInvalid : null,
        ),
        onChanged: (_) => setState(() {}),
      ),
      SwitchListTile(
        title: Text(SharingTexts.https),
        subtitle: Text(SharingTexts.httpsHint),
        value: _https,
        onChanged: (https) => setState(() => _https = https),
      ),
      if (widget.settings.enabled && _changed && _validPort != null)
        ActionButton(
          label: SharingTexts.apply,
          onPressed: () => _apply(enabled: true),
        ),
    ],
  );
}
