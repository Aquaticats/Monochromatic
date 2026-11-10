// Checks of the elapsed-time field read the bundle checks compare parser timings by.
// What: which line states a microsecond count and which states none, over the records the application prints and over
// the near misses: the field without digits, the digits of another script, and two fields on one line.
// Why: a start that unpacks a parser reports how long it took, and a later start that reuses it reports its own, so a
// read that invented a zero where a line states nothing would make the comparison between the two meaningless.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { loggedElapsedMicroseconds } from './logged-elapsed-time.ts';

test('reads the microseconds a record states', () => {
  assert.equal(loggedElapsedMicroseconds('language parser unpacked into the private cache elapsed_us=1234'), 1234);
  assert.equal(loggedElapsedMicroseconds('elapsed_us=7'), 7);
  assert.equal(loggedElapsedMicroseconds('elapsed_us=007'), 7);
  assert.equal(loggedElapsedMicroseconds('elapsed_us=0'), 0);
});

test('reads the first field of a line that states more than one', () => {
  assert.equal(loggedElapsedMicroseconds('elapsed_us=12 elapsed_us=34'), 12);
  // A field written without digits after it states nothing, so the next field is the one read.
  assert.equal(loggedElapsedMicroseconds('elapsed_us= elapsed_us=34'), 34);
  assert.equal(loggedElapsedMicroseconds('elapsed_us=elapsed_us=elapsed_us=1'), 1);
});

test('reads a field that stands inside a longer word', () => {
  assert.equal(loggedElapsedMicroseconds('recorded_elapsed_us=9'), 9);
  assert.equal(loggedElapsedMicroseconds('xelapsed_us=9 ; released'), 9);
});

test('reads the digits as far as they run and no further', () => {
  assert.equal(loggedElapsedMicroseconds('elapsed_us=12abc'), 12);
  assert.equal(loggedElapsedMicroseconds('elapsed_us=12.5'), 12);
});

test('reads no count from a line that states none', () => {
  assert.equal(loggedElapsedMicroseconds(''), undefined);
  assert.equal(loggedElapsedMicroseconds('language parser unpacked into the private cache'), undefined);
  assert.equal(loggedElapsedMicroseconds('elapsed=1234'), undefined);
  assert.equal(loggedElapsedMicroseconds('elapsed_us =1234'), undefined);
  assert.equal(loggedElapsedMicroseconds('elapsed_us='), undefined);
  assert.equal(loggedElapsedMicroseconds('elapsed_us= ms'), undefined);
});

test('reads no count from digits of another script', () => {
  // An Arabic-Indic digit and a full-width digit each read as a number elsewhere and state nothing here.
  assert.equal(loggedElapsedMicroseconds('elapsed_us=\u0661\u0662'), undefined);
  assert.equal(loggedElapsedMicroseconds('elapsed_us=\uFF11\uFF12'), undefined);
});
