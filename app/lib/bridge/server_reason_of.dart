import '../shared_ui/server_reason.dart';
import 'generated/api.dart' as rust;

/// The reason the Rust client core reports, as the interface knows it.
ServerReason serverReasonOf(rust.ReasonInfo reason) => ServerReason(
  reason.code,
  args: [for (final number in reason.args) number.toInt()],
  message: reason.message,
);
