// Checks of the version reader behind demandCurrentCompositor.
// What: each expected value is what the previous regular expression `(\d+)\.(\d+)\.(\d+)` returned for the same text;
// an exhaustive comparison against that expression over every string up to eight characters of a six-symbol
// alphabet found no difference when the reader was replaced.
// Why: the `--version` line decides whether a check may open a window on the desktop, so a misread version must not pass.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readVersion } from './nested-compositor.ts';

test(
  'reads the version line the compositor prints',
  () => {
    assert.deepEqual(
      readVersion('monochromatic-nested-wayland-session 0.6.0\n'),
      [
        0,
        6,
        0
      ]
    );
  }
);

test(
  'ignores a suffix after the patch number',
  () => {
    assert.deepEqual(
      readVersion('x 0.6.1-rc.1'),
      [
        0,
        6,
        1
      ]
    );
    assert.deepEqual(
      readVersion('v10.20.30+build5'),
      [
        10,
        20,
        30
      ]
    );
  }
);

test(
  'keeps the first three fields when a fourth follows',
  () => {
    assert.deepEqual(
      readVersion('1.2.3.4'),
      [
        1,
        2,
        3
      ]
    );
    assert.deepEqual(
      readVersion('9.1.2.3'),
      [
        9,
        1,
        2
      ]
    );
  }
);

test(
  'reads leading zeros as numbers',
  () => {
    assert.deepEqual(
      readVersion('01.02.3'),
      [
        1,
        2,
        3
      ]
    );
  }
);

test(
  'resumes after a failed candidate',
  () => {
    assert.deepEqual(
      readVersion('12.3 4.5.6'),
      [
        4,
        5,
        6
      ]
    );
    assert.deepEqual(
      readVersion('1.2.x 3.4.5'),
      [
        3,
        4,
        5
      ]
    );
  }
);

test(
  'returns null when no three fields exist',
  () => {
    assert.equal(
      readVersion(''),
      null
    );
    assert.equal(
      readVersion('garbage'),
      null
    );
    assert.equal(
      readVersion('1.2'),
      null
    );
    assert.equal(
      readVersion('1..2.3'),
      null
    );
    assert.equal(
      readVersion('..'),
      null
    );
  }
);

test(
  'accepts only ASCII digits',
  () => {
    assert.equal(
      readVersion('١.٢.٣'),
      null
    );
  }
);
