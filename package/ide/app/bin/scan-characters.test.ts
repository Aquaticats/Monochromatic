// The character classes the inspection scripts' scans share, checked against the pattern classes they replace.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { isAsciiDigit, isLineTerminator } from './scan-characters.ts';

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
