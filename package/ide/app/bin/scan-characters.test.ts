// The character classes the inspection scripts' scans share, checked against the pattern classes they replace.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { isAsciiDigit } from './scan-characters.ts';

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
