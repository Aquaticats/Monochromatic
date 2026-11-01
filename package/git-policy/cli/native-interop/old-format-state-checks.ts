/**
 The checks of `old-format-driver.ts` for stored state that no transaction phase explains: the capture store of
 finished commits and the hook lock of a dead incumbent.
 */
import {
  readFile,
  stat,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  restoreFixture,
  captureSequence
} from './old-format-restore.ts';
import {
  excerpt,
  LONG_EXCERPT,
  linesOf,
} from './old-format-text.ts';
import {
  isMissingPath,
  leftovers,
  namesIn,
  realGit,
  runThrough,
} from './support.ts';
import type { Matrix } from './old-format-phase-checks.ts';

/**
 The lines of the hook's log.

 The repository ignores `*.log`, so the committed fixture carries no log and the killed incumbent's `start` line is
 not stored; an absent log therefore counts as empty, and the native commit must still add `start` and `end`.

 @param log - log path
 
 @returns the non-empty lines, none when the log is absent
 */
async function hookLines(log: string): Promise<string[]> {
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

 @param matrix - recorder of the driver's checks

 @example
 ```ts
 await checkCompleted({ matrix });
 ```
 */
export async function checkCompleted({ matrix, }: {
  readonly matrix: Matrix;
},): Promise<void> {
  /**
   Message prefix of this check.
   */
  const tag = 'completed';
  /**
   The restored fixture holding the incumbent's finished commits.
   */
  const fixture = await restoreFixture('completed');
  /**
   Capture sequence before the native commit.
   */
  const before = await captureSequence(fixture);
  /**
   Landed-capture records stored before the native commit.
   */
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
  /**
   The native commit over the incumbent's capture store.
   */
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
  /**
   Capture sequence after the native commit.
   */
  const after = await captureSequence(fixture);
  matrix.check({
    condition: after > before,
    message: `${tag}: the capture sequence advanced from ${String(before)} to ${String(after)}`,
  });
  /**
   Content of the committed file as real Git reads it.
   */
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
  /**
   What the wrapper left behind.
   */
  const left = await leftovers({ fixture });
  matrix.check({
    condition: left.length === 0,
    message: `${tag}: nothing is left behind (${left.join(', ') || 'none'})`,
  });
}

/**
 Check that a native commit retires the hook lock of a dead incumbent, runs the hook once and lands.

 @param matrix - recorder of the driver's checks

 @example
 ```ts
 await checkDeadHookLock({ matrix });
 ```
 */
export async function checkDeadHookLock({ matrix, }: {
  readonly matrix: Matrix;
},): Promise<void> {
  /**
   Message prefix of this check.
   */
  const tag = 'hook-lock-dead';
  /**
   The restored fixture holding a dead incumbent's hook lock.
   */
  const fixture = await restoreFixture('hook-lock-dead');
  /**
   Record file of the stored hook lock.
   */
  const lock = join(
    fixture.repository,
    '.git/cli-git/hook.lock/owner.json',
  );
  /**
   Status of the lock record, which must be present and non-empty.
   */
  const lockRecord = await stat(lock);
  matrix.check({
    condition: lockRecord.size > 0,
    message: `${tag}: the stored incumbent hook lock record is present`,
  });
  /**
   Log the hook appends to.
   */
  const log = join(
    fixture.root,
    'hook.log',
  );
  /**
   Hook log lines before the native commit.
   */
  const before = await hookLines(log);
  /**
   The native commit that must retire the dead lock.
   */
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
  /**
   Hook log lines after the native commit.
   */
  const lines = await hookLines(log);
  /**
   Lines the native commit added, joined for the comparison.
   */
  const ran = lines.slice(before.length)
    .join(',');
  matrix.check({
    condition: ran === 'start,end',
    message: `${tag}: the hook ran once to the end (${ran})`,
  });
  /**
   What the wrapper left behind.
   */
  const left = await leftovers({ fixture });
  matrix.check({
    condition: left.length === 0,
    message: `${tag}: nothing is left behind (${left.join(', ') || 'none'})`,
  });
}
