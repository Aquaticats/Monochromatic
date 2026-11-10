// Checks of the process-number reader the two lifecycle inspection scripts keep the numeric `/proc` entries with.
// What: each expected value is what the anchored `^\d+$` pattern the reader replaced answered for the same name.
// Why: both scripts walk `/proc` looking for server processes, and an entry name read as a process number when it is
// not one makes a kernel view (`self`, `net`) a process, while a process number read as something else hides a
// server that outlived its session.
import assert from 'node:assert/strict';
import { test } from 'node:test';

import { isProcessNumberName } from './process-number-name.ts';

test(
  'reads a process number',
  () => {
    assert.equal(isProcessNumberName('1'), true);
    assert.equal(isProcessNumberName('0'), true);
    assert.equal(isProcessNumberName('4194304'), true);
  }
);

test(
  'reads the number of a live process',
  () => {
    assert.equal(isProcessNumberName(String(process.pid)), true);
  }
);

test(
  'reads no number from an empty name',
  () => {
    assert.equal(isProcessNumberName(''), false);
  }
);

test(
  'reads no number from a kernel view',
  () => {
    assert.equal(isProcessNumberName('self'), false);
    assert.equal(isProcessNumberName('net'), false);
    assert.equal(isProcessNumberName('sys'), false);
    assert.equal(isProcessNumberName('uptime'), false);
  }
);

test(
  'reads no number from a name that holds a digit and something else',
  () => {
    assert.equal(isProcessNumberName('1a'), false);
    assert.equal(isProcessNumberName('a1'), false);
    assert.equal(isProcessNumberName('1_000'), false);
    assert.equal(isProcessNumberName('-1'), false);
    assert.equal(isProcessNumberName('1.0'), false);
  }
);

test(
  'reads no number from a name with whitespace around it',
  () => {
    assert.equal(isProcessNumberName(' 1'), false);
    assert.equal(isProcessNumberName('1 '), false);
    assert.equal(isProcessNumberName('1\n'), false);
    assert.equal(isProcessNumberName('\t1'), false);
  }
);

test(
  'reads no number from digits outside ASCII',
  () => {
    // `\d` without the `u` flag reaches the ten ASCII digits only, so the other decimal-digit scripts are other names.
    assert.equal(isProcessNumberName('\u0661\u0662'), false);
    assert.equal(isProcessNumberName('\uff11\uff12'), false);
    assert.equal(isProcessNumberName('\u0967'), false);
  }
);

test(
  'reads no number from a name holding a zero-width joining character',
  () => {
    assert.equal(isProcessNumberName('1\u200b2'), false);
    assert.equal(isProcessNumberName('1\u200d'), false);
  }
);
