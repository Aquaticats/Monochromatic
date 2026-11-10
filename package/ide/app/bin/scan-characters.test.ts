// The character classes the inspection scripts' scans share, checked against the pattern classes they replace.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { isAsciiDigit, isDecimalNumberCharacter, isLineTerminator, isLowercaseHexadecimalDigit, isWhitespace, isWordCharacter } from './scan-characters.ts';

const digits = '0123456789';

test('accepts the ten ASCII digits', () => {
  for (const digit of digits) assert.equal(isAsciiDigit(digit), true, digit);
});

test('rejects the characters beside the digits in the code unit table', () => {
  const below = String.fromCharCode('0'.charCodeAt(0) - 1);
  const above = String.fromCharCode('9'.charCodeAt(0) + 1);
  for (const character of [below, above, '\u0000', 'a', 'A', '_', ' ']) assert.equal(isAsciiDigit(character), false, character);
});

test('rejects digits of other scripts and text longer than one character', () => {
  for (const text of ['\u0667', '\uff10', '7 ', '', '77']) assert.equal(isAsciiDigit(text), false, text);
});

test('accepts the four line terminators', () => {
  for (const terminator of ['\n', '\r', '\u2028', '\u2029']) assert.equal(isLineTerminator(terminator), true, JSON.stringify(terminator));
});

test('rejects whitespace and characters that are not line terminators', () => {
  // A vertical tab and a form feed are whitespace a pattern line start does not reach, and so are the next-line and
  // no-break space characters.
  for (const character of [' ', '\t', '\u000b', '\f', '\u0085', '\u00a0', 'a', '', '\r\n']) {
    assert.equal(isLineTerminator(character), false, JSON.stringify(character));
  }
});

test('accepts the digits and the decimal point of a printed number', () => {
  for (const character of digits) assert.equal(isDecimalNumberCharacter(character), true, character);
  assert.equal(isDecimalNumberCharacter('.'), true);
});

test('rejects the characters a printed number never holds', () => {
  for (const character of [',', '+', '-', 'e', 'E', ' ', '', '..']) assert.equal(isDecimalNumberCharacter(character), false, character);
});

test('accepts every code unit whitespace names', () => {
  const printing = Array.from({ length: 0x200a - 0x2000 + 1 }, (_, offset) => String.fromCharCode(0x2000 + offset));
  const named = ['\u0009', '\u000b', '\u000c', '\u0020', '\u00a0', '\u1680', '\u202f', '\u205f', '\u3000', '\ufeff', ...printing];
  for (const character of named) assert.equal(isWhitespace(character), true, JSON.stringify(character));
  // The line terminators are whitespace as well, and they are checked on their own above.
  for (const terminator of ['\n', '\r', '\u2028', '\u2029']) assert.equal(isWhitespace(terminator), true, JSON.stringify(terminator));
});

test('rejects the characters beside whitespace in the code unit table', () => {
  const beside = [0x1fff, 0x200b, 0x2010, 0x202e, 0x2030, 0x205e, 0x2060, 0x2fff, 0x3001, 0xfeff - 1, 0xfeff + 1];
  for (const code of beside) assert.equal(isWhitespace(String.fromCharCode(code)), false, 'code unit ' + code);
  for (const character of ['a', '0', '\u0085', '\u180e', '', '  ']) assert.equal(isWhitespace(character), false, JSON.stringify(character));
});

test('accepts the letters, digits and underscore a word character names', () => {
  const named = ['a', 'z', 'A', 'Z', '_', ...digits];
  for (const character of named) assert.equal(isWordCharacter(character), true, character);
});

test('rejects the characters beside the word characters in the code unit table', () => {
  const beside = ['`', '{', '@', '[', '^', ']', '-', '.', ' ', '\u00e9', '', 'ab'];
  for (const character of beside) assert.equal(isWordCharacter(character), false, JSON.stringify(character));
});

test('accepts the digits and the lowercase letters a hexadecimal print holds', () => {
  for (const character of [...digits, 'a', 'f', 'c']) assert.equal(isLowercaseHexadecimalDigit(character), true, character);
});

test('rejects an uppercase hexadecimal letter, a letter past f, and anything else', () => {
  for (const character of ['A', 'F', 'g', 'G', 'x', '-', ' ', '', 'ff']) assert.equal(isLowercaseHexadecimalDigit(character), false, JSON.stringify(character));
});
