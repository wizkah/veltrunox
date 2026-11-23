import 'package:flutter_test/flutter_test.dart';
import 'package:veltrunox_app/shared_ui/app_language.dart';
import 'package:veltrunox_app/shared_ui/server_reason.dart';

void main() {
  setUp(() => AppLanguage.current = AppLanguage.de);
  tearDown(() => AppLanguage.current = AppLanguage.de);

  test('known codes come out in German', () {
    expect(
      serverReasonText(const ServerReason('password_needs_digit')),
      'Das Passwort braucht eine Ziffer.',
    );
    expect(
      serverReasonText(const ServerReason('file_too_large', args: [200])),
      'Die Datei ist größer als das Limit von 200 MB.',
    );
    expect(
      serverReasonText(const ServerReason('password_too_short', args: [10])),
      'Das Passwort braucht mindestens 10 Zeichen.',
    );
  });

  test('known codes come out in English', () {
    AppLanguage.current = AppLanguage.en;
    expect(
      serverReasonText(const ServerReason('too_many_attempts', args: [30])),
      'Too many failed attempts. Please try again in 30 s.',
    );
  });

  test('an unknown code shows the English message of the server', () {
    expect(
      serverReasonText(
        const ServerReason('something_new', message: 'it broke'),
      ),
      'It broke.',
    );
  });

  test('an unknown code without a message says the server refused', () {
    expect(
      serverReasonText(const ServerReason('something_new')),
      'Der Server hat die Anfrage abgelehnt.',
    );
    expect(knownReasonText(const ServerReason('something_new')), isNull);
  });

  test('a plain text is shown as it is', () {
    expect(
      serverReasonText(const ServerReason.plain('No network')),
      'No network',
    );
  });
}
