import 'package:web/web.dart' as web;

/// Copies [text] through a temporary text field and the legacy copy command.
/// Works where the asynchronous clipboard API is missing, such as a page
/// served over plain HTTP from a host other than localhost.
bool copyTextInBrowser(String text) {
  final field = web.HTMLTextAreaElement()
    ..value = text
    ..readOnly = true
    ..style.position = 'fixed'
    ..style.opacity = '0';
  web.document.body!.append(field);
  try {
    field.select();
    return web.document.execCommand('copy');
  } finally {
    field.remove();
  }
}
