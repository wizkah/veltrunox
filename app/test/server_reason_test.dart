import 'package:flutter_test/flutter_test.dart';
import 'package:veltrunox_app/shared_ui/server_reason.dart';

void main() {
  test('known reasons come out in German', () {
    expect(
      serverReasonText('password must contain a digit'),
      'Das Passwort braucht eine Ziffer.',
    );
    expect(
      serverReasonText('the file is larger than the limit of 200 MB'),
      'Die Datei ist größer als das Limit von 200 MB.',
    );
    expect(
      serverReasonText('password must have at least 10 characters'),
      'Das Passwort braucht mindestens 10 Zeichen.',
    );
  });

  test('a reason that is not known stays as the server said it', () {
    expect(serverReasonText('something new'), 'something new');
  });
}
