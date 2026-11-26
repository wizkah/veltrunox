import 'rust_library_native.dart'
    if (dart.library.js_interop) 'rust_library_web.dart'
    as platform;

/// Loads the Rust client core: the WebAssembly bundle in a browser, the
/// library inside the application bundle on the desktop.
Future<void> initRustLibrary() => platform.initRustLibrary();
