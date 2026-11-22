import 'package:flutter_test/flutter_test.dart';
import 'package:veltrunox_app/features/access/access_models.dart';

void main() {
  test('the standard rules ask for six characters only', () {
    expect(PasswordPolicy.standard.check('12345'), PasswordProblem.tooShort);
    expect(PasswordPolicy.standard.check('123456'), isNull);
  });

  test('the strict rules ask for length and every kind of character', () {
    const strict = PasswordPolicy.strict;
    expect(strict.check('Ab1!xyz'), PasswordProblem.tooShort);
    expect(strict.check('ABCDEFG1!2'), PasswordProblem.noLowercase);
    expect(strict.check('abcdefg1!2'), PasswordProblem.noUppercase);
    expect(strict.check('Abcdefghi!'), PasswordProblem.noDigit);
    expect(strict.check('Abcdefghi1'), PasswordProblem.noSpecial);
    expect(strict.check('Abcdef ghi1'), PasswordProblem.noSpecial);
    expect(strict.check('Abcdefgh1!'), isNull);
  });
}
