// Checks of the /proc entry name read the leftover sweep skips entries by.
// What: which names count as a process identifier, over the names /proc really holds and over the near misses a digit
// can be mistaken for.
// Why: the sweep reads and can kill the process behind every entry it accepts, so a name accepted by mistake makes it
// read a system entry as a process, and a name refused by mistake leaves a leftover of a damaged session running.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { isProcessIdentifier } from './process-identifier.ts';

test('reads the identifier of a process folder', () => {
  assert.equal(isProcessIdentifier('1'), true);
  assert.equal(isProcessIdentifier('0'), true);
  assert.equal(isProcessIdentifier('4242'), true);
  assert.equal(isProcessIdentifier(String(Number.MAX_SAFE_INTEGER)), true);
});

test('reads the entries /proc holds beside its process folders as no process', () => {
  for (const name of ['self', 'thread-self', 'net', 'sys', 'sysrq-trigger', 'meminfo', 'uptime']) {
    assert.equal(isProcessIdentifier(name), false);
  }
});

test('reads an entry that only starts or only ends with digits as no process', () => {
  assert.equal(isProcessIdentifier('1a'), false);
  assert.equal(isProcessIdentifier('a1'), false);
  assert.equal(isProcessIdentifier('42 '), false);
  assert.equal(isProcessIdentifier(' 42'), false);
  assert.equal(isProcessIdentifier('4 2'), false);
  assert.equal(isProcessIdentifier('-1'), false);
  assert.equal(isProcessIdentifier('+1'), false);
  assert.equal(isProcessIdentifier('1.0'), false);
  assert.equal(isProcessIdentifier('1\n'), false);
  assert.equal(isProcessIdentifier('0x10'), false);
});

test('reads a digit that is not the ASCII one as no process', () => {
  // An Arabic-Indic digit, a full-width digit and a subscript digit each look like a number and are not one here.
  assert.equal(isProcessIdentifier('\u0661'), false);
  assert.equal(isProcessIdentifier('\uFF11\uFF12'), false);
  assert.equal(isProcessIdentifier('\u2081'), false);
  assert.equal(isProcessIdentifier('1\u0661'), false);
});

test('reads an empty name as no process', () => {
  assert.equal(isProcessIdentifier(''), false);
});
