/**
 The per-phase checks of `old-format-fixtures.mjs`: the stored incumbent transaction phases.
 */
/// <reference types="node" />
import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  fingerprintOf,
  lostEntries,
  STORED_STATE_DIRECTORIES,
} from './old-format-fingerprint.mjs';
import { restoreFixture } from './old-format-restore.mjs';
import {
  excerpt,
  fsckOf,
  LONG_EXCERPT,
  linesOf,
  SHORT_EXCERPT,
} from './old-format-text.mjs';
import {
  leftovers,
  namesIn,
  realGit,
  run,
  runEnvironment,
  runThrough,
  REAL_GIT,
} from './support.mjs';

/**
 A stored phase and whether the incumbent's commit had landed when it died.
 @typedef {{ phase: string, landed: boolean }} StoredPhase
 */

/**
 Recorder of the driver's checks.
 @typedef {ReturnType<typeof import('./support.mjs').createMatrix>} Matrix
 */

/** Words, in lower case, that would mean the native wrapper could not read a stored record. */
const UNREADABLE_RECORD_WORDS = [
  'malformed',
  'unsupported',
  'unexpected',
  'not valid',
  'cannot parse',
  'failed to parse',
];
/** Most lost entries a check message names. */
const LOST_ENTRIES_SHOWN = 6;
/** The incumbent leaves exactly one transaction directory in a restorable phase. */
const EXPECTED_DEAD_TRANSACTIONS = 1;

/**
 Whether a native command's output says it could not read a stored record.

 @param {string} output - combined standard output and standard error
 @returns {boolean} true when any telltale word occurs
 */
function reportsUnreadableRecord(output) {
  const lowered = output.toLowerCase();
  return UNREADABLE_RECORD_WORDS.some(function mentioned(word) {
    return lowered.includes(word);
  });
}

/**
 Check a phase whose records name identities that a copy cannot reproduce.

 @param {{ phase: string, matrix: Matrix }} request - stored phase and the recorder of the driver's checks
 */
export async function checkIdentityBoundPhase({
  phase,
  matrix,
}) {
  const name = `phase-${phase}`;
  const fixture = await restoreFixture(name);
  const base = await realGit({
    fixture,
    args: [
      'rev-parse',
      'HEAD'
    ],
  });
  const before = await fingerprintOf({
    repository: fixture.repository,
    subdirectories: STORED_STATE_DIRECTORIES,
  });
  const status = await runThrough({
    wrapper: 'native',
    fixture,
    args: [
      'status',
      '--porcelain'
    ],
  });
  const output = status.stderr + status.stdout;
  matrix.check({
    condition: (status.status === 0) || output.includes('identity changed'),
    message: `${name}: native git status read the stored records and recovered or refused on identity (status ${String(status.status)}; ${excerpt({
      text: output,
      limit: SHORT_EXCERPT,
    })})`,
  });
  matrix.check({
    condition: !reportsUnreadableRecord(output),
    message: `${name}: no stored record was reported unreadable`,
  });
  const head = await realGit({
    fixture,
    args: [
      'rev-parse',
      'HEAD'
    ],
  });
  if (status.status !== 0) {
    // A refusal keeps every stored object, reference and transaction record (it may retire a dead holder's lock
    // record and a pack's keep marker). The landed commit of a late phase exists only in the incumbent's shadow
    // repository, so `git log` and `git fsck` cannot judge the repository after a refusal.
    const after = await fingerprintOf({
      repository: fixture.repository,
      subdirectories: STORED_STATE_DIRECTORIES,
    });
    const lost = lostEntries({
      before,
      after,
    });
    matrix.check({
      condition: head === base,
      message: `${name}: the refusal left HEAD at ${base}`,
    });
    matrix.check({
      condition: lost.length === 0,
      message: `${name}: the refusal lost no stored object, reference or transaction record (${lost.slice(
        0,
        LOST_ENTRIES_SHOWN,
      )
        .join(', ')
        || `${String(before.length)} files kept`})`,
    });
    return;
  }
  const log = await realGit({
    fixture,
    args: [
      'log',
      '--format=%s',
      `${base}..HEAD`
    ],
  });
  const subjects = linesOf(log);
  matrix.check({
    condition: (head === base) || (subjects.join('|') === `change at ${phase}`),
    message: `${name}: HEAD is at the base or at the incumbent's landed commit (${subjects.join('|') || 'base'})`,
  });
  const fsck = await fsckOf(fixture);
  matrix.check({
    condition: fsck.status === 0,
    message: `${name}: git fsck is clean (${excerpt({
      text: fsck.stderr,
      limit: SHORT_EXCERPT,
    })})`,
  });
}

/**
 Check a phase the native wrapper must recover completely, then land a follow-up commit.

 @param {StoredPhase & { matrix: Matrix }} request - stored phase, whether the incumbent's commit had landed, and the
   recorder of the driver's checks
 */
export async function checkRestorablePhase({
  matrix,
  phase,
  landed,
}) {
  const tag = `phase-${phase}`;
  const fixture = await restoreFixture(tag);
  const base = await realGit({
    fixture,
    args: [
      'rev-parse',
      'HEAD'
    ],
  });
  const stored = await namesIn(join(
    fixture.repository,
    '.git/cli-git-transactions',
  ));
  const dead = stored.filter(function isTransaction(entry) {
    return !entry.endsWith('.lock');
  });
  matrix.check({
    condition: dead.length === EXPECTED_DEAD_TRANSACTIONS,
    message: `${tag}: the stored incumbent transaction directory is present (${dead.join(', ')})`,
  });
  const status = await runThrough({
    wrapper: 'native',
    fixture,
    args: [
      'status',
      '--porcelain'
    ],
  });
  matrix.check({
    condition: status.status === 0,
    message: `${tag}: native git status recovered the stored transaction (status ${String(status.status)}; ${excerpt({
      text: status.stderr,
      limit: LONG_EXCERPT,
    })})`,
  });
  const log = await realGit({
    fixture,
    args: [
      'log',
      '--format=%s',
      `${base}..HEAD`
    ],
  });
  matrix.check({
    condition: linesOf(log)
      .includes(`change at ${phase}`)
      === landed,
    message: `${tag}: the commit ${landed ? 'landed whole' : 'did not land'}`,
  });
  const cached = await run({
    command: REAL_GIT,
    args: [
      'diff',
      '--cached',
      '--quiet',
      '--',
      'a.txt'
    ],
    cwd: fixture.repository,
    env: runEnvironment({ home: fixture.home }),
  });
  matrix.check({
    condition: cached.status === 0,
    message: `${tag}: the real index equals HEAD for a.txt`,
  });
  const edit = await readFile(
    join(
      fixture.repository,
      'a.txt',
    ),
    'utf8',
  );
  matrix.check({
    condition: edit === 'second\n',
    message: `${tag}: the worktree keeps the edit`,
  });
  const fsck = await fsckOf(fixture);
  matrix.check({
    condition: fsck.status === 0,
    message: `${tag}: git fsck is clean (${excerpt({
      text: fsck.stderr,
      limit: SHORT_EXCERPT,
    })})`,
  });
  const left = await leftovers({
    fixture,
    staleLocks: true,
  });
  matrix.check({
    condition: left.length === 0,
    message: `${tag}: nothing but a dead holder's lock directory is left (${left.join(', ') || 'none'})`,
  });
  await writeFile(
    join(
      fixture.repository,
      'b.txt',
    ),
    'follow-up\n',
  );
  const follow = await runThrough({
    wrapper: 'native',
    fixture,
    args: [
      'commit',
      '--quiet',
      '--message=native follow-up',
      '--',
      'b.txt',
    ],
  });
  const subject = await realGit({
    fixture,
    args: [
      'log',
      '-1',
      '--format=%s'
    ],
  });
  matrix.check({
    condition: (follow.status === 0) && (subject === 'native follow-up'),
    message: `${tag}: a native commit then landed (status ${String(follow.status)}; ${excerpt({
      text: follow.stderr,
      limit: LONG_EXCERPT,
    })})`,
  });
}
