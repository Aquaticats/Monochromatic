// Checks of the version reader behind demandCurrentCompositor and of how a check ends a session.
// What: each expected value is what the previous regular expression `(\d+)\.(\d+)\.(\d+)` returned for the same text;
// an exhaustive comparison against that expression over every string up to eight characters of a six-symbol
// alphabet found no difference when the reader was replaced.
// Why: the `--version` line decides whether a check may open a window on the desktop, so a misread version must not pass.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { endSession, endedWithin, readVersion } from './nested-compositor.ts';

/// A promise that never settles, standing for a compositor that ignores its quit request.
const never = (): Promise<unknown> => new Promise<unknown>(() => {});

/// A signal log, so a test can see which signals a session was ended with.
const signals = (): { sent: NodeJS.Signals[]; kill: (signal: NodeJS.Signals) => void } => {
  const sent: NodeJS.Signals[] = [];
  return {
    sent,
    kill: (signal: NodeJS.Signals): void => {
      sent.push(signal);
    },
  };
};

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

test(
  'a session that has ended is seen as ended at once',
  async () => {
    assert.equal(await endedWithin(Promise.resolve(), 60000), true);
  }
);

test(
  'a session that never ends is not seen as ended',
  async () => {
    assert.equal(await endedWithin(never(), 1), false);
  }
);

test(
  'a session that ignores its quit request is killed after the patience',
  async () => {
    const log = signals();
    await endSession({
      ask: async () => 'ok',
      exited: never(),
      kill: log.kill,
      patienceMs: 1
    });
    assert.deepEqual(log.sent, ['SIGKILL'], 'the session was asked to quit and then killed');
  }
);

test(
  'a session that ends after its quit request is not killed',
  async () => {
    const log = signals();
    await endSession({
      ask: async () => 'ok',
      exited: Promise.resolve(),
      kill: log.kill,
      patienceMs: 60000
    });
    assert.deepEqual(log.sent, [], 'a session that ended is left alone');
  }
);

test(
  'a session that cannot be asked to quit is stopped by a signal first',
  async () => {
    const log = signals();
    await endSession({
      ask: async () => {
        throw new Error('the compositor closed the control connection without replying to quit');
      },
      exited: never(),
      kill: log.kill,
      patienceMs: 1
    });
    assert.deepEqual(log.sent, ['SIGTERM', 'SIGKILL'], 'the failed request falls back to a signal');
  }
);
