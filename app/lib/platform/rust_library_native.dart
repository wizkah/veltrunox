import 'dart:io';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

import '../bridge/generated/frb_generated.dart';

Future<void> initRustLibrary() {
  // <bundle>/Contents/MacOS/<program> lies next to Contents/Frameworks.
  final contents = File(Platform.resolvedExecutable).parent.parent.path;
  return RustLib.init(
    externalLibrary: ExternalLibrary.open(
      '$contents/Frameworks/libveltrunox_client_bridge.dylib',
    ),
  );
}
