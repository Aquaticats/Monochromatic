/**
 The checks of `old-format-fixtures.mjs` for stored state that no transaction phase explains: the capture store of
 finished commits and the hook lock of a dead incumbent.
 */
/// <reference types="node" />
import {
  readFile,
  stat,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  restoreFixture,
  captureSequence
} from './old-format-restore.mjs';
import {
  excerpt,
  LONG_EXCERPT,
  linesOf,
} from './old-format-text.mjs';
import {
  isMissingPath,
  leftovers,
  namesIn,
  realGit,
  runThrough,
} from './support.mjs';

/**
 The lines of the hook's log.

 The repository ignores `*.log`, so the committed fixture carries no log and the killed incumbent's `start` line is
 not stored; an absent log therefore counts as empty, and the native commit must still add `start` and `end`.

 @param {string} log - log path
 @returns {Promise<string[]>} the non-empty lines, none when the log is absent
 */
async function hookLines(log) {
  try {
    return linesOf(await readFile(
      log,
      'utf8',
    ));
  }
  catch (error) {
    if (isMissingPath(error))
      return [];
    throw error;
  }
}

/**
 Check that a native commit lands over the incumbent's capture store and advances its sequence.

 @param {{ matrix: import('./old-format-phase-checks.mjs').Matrix }} request - recorder of the driver's checks
 */
export async function checkCompleted({ matrix }) {
  const tag = 'completed';
  const fixture = await restoreFixture('completed');
  const before = await captureSequence(fixture);
  const landedBefore = await namesIn(join(
    fixture.repository,
    '.git/cli-git-captures/landed',
  ));
  matrix.check({
    condition: before > 0,
    message: `${tag}: the stored capture sequence is ${String(before)} and ${String(landedBefore.length)} landed-capture records are stored`,
  });
  await writeFile(
    join(
      fixture.repository,
      'a.txt',
    ),
    'native after the incumbent\n',
  );
  const commit = await runThrough({
    wrapper: 'native',
    fixture,
    args: [
      'commit',
      '--quiet',
      '--message=native after the incumbent',
      '--',
      'a.txt',
    ],
  });
  matrix.check({
    condition: commit.status === 0,
    message: `${tag}: a native commit landed over the incumbent's capture store (status ${String(commit.status)}; ${excerpt({
      text: commit.stderr,
      limit: LONG_EXCERPT,
    })})`,
  });
  const after = await captureSequence(fixture);
  matrix.check({
    condition: after > before,
    message: `${tag}: the capture sequence advanced from ${String(before)} to ${String(after)}`,
  });
  const shown = await realGit({
    fixture,
    args: [
      'show',
      'HEAD:a.txt'
    ],
  });
  matrix.check({
    condition: shown === 'native after the incumbent',
    message: `${tag}: the native commit holds the edit`,
  });
  const left = await leftovers({ fixture });
  matrix.check({
    condition: left.length === 0,
    message: `${tag}: nothing is left behind (${left.join(', ') || 'none'})`,
  });
}

/**
 Check that a native commit retires the hook lock of a dead incumbent, runs the hook once and lands.

 @param {{ matrix: import('./old-format-phase-checks.mjs').Matrix }} request - recorder of the driver's checks
 */
export async function checkDeadHookLock({ matrix }) {
  const tag = 'hook-lock-dead';
  const fixture = await restoreFixture('hook-lock-dead');
  const lock = join(
    fixture.repository,
    '.git/cli-git/hook.lock/owner.json',
  );
  const lockRecord = await stat(lock);
  matrix.check({
    condition: lockRecord.size > 0,
    message: `${tag}: the stored incumbent hook lock record is present`,
  });
  const log = join(
    fixture.root,
    'hook.log',
  );
  const before = await hookLines(log);
  const commit = await runThrough({
    wrapper: 'native',
    fixture,
    args: [
      'commit',
      '--quiet',
      '--message=native over the dead hook lock',
      '--',
      'a.txt',
    ],
  });
  matrix.check({
    condition: commit.status === 0,
    message: `${tag}: a native commit retired the dead lock and landed (status ${String(commit.status)}; ${excerpt({
      text: commit.stderr,
      limit: LONG_EXCERPT,
    })})`,
  });
  const lines = await hookLines(log);
  const ran = lines.slice(before.length)
    .join(',');
  matrix.check({
    condition: ran === 'start,end',
    message: `${tag}: the hook ran once to the end (${ran})`,
  });
  const left = await leftovers({ fixture });
  matrix.check({
    condition: left.length === 0,
    message: `${tag}: nothing is left behind (${left.join(', ') || 'none'})`,
  });
}
