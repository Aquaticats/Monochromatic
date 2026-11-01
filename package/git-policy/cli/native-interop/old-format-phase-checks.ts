/**
 The per-phase checks of `old-format-driver.ts`: the stored incumbent transaction phases.
 */
import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  fingerprintOf,
  lostEntries,
  STORED_STATE_DIRECTORIES,
} from './old-format-fingerprint.ts';
import { restoreFixture } from './old-format-restore.ts';
import {
  excerpt,
  fsckOf,
  LONG_EXCERPT,
  linesOf,
  SHORT_EXCERPT,
} from './old-format-text.ts';
import {
  leftovers,
  namesIn,
  realGit,
  run,
  runEnvironment,
  runThrough,
  REAL_GIT,
  type createMatrix,
} from './support.ts';

/**
 A stored phase and whether the incumbent's commit had landed when it died.
 */
export type StoredPhase = {
  readonly phase: string;
  readonly landed: boolean;
};

/**
 Recorder of the driver's checks.
 */
export type Matrix = ReturnType<typeof createMatrix>;

/**
 Words, in lower case, that would mean the native wrapper could not read a stored record.
 */
const UNREADABLE_RECORD_WORDS = [
  'malformed',
  'unsupported',
  'unexpected',
  'not valid',
  'cannot parse',
  'failed to parse',
];
/**
 Most lost entries a check message names.
 */
const LOST_ENTRIES_SHOWN = 6;
/**
 The incumbent leaves exactly one transaction directory in a restorable phase.
 */
const EXPECTED_DEAD_TRANSACTIONS = 1;

/**
 Whether a native command's output says it could not read a stored record.

 @param output - combined standard output and standard error
 
 @returns true when any telltale word occurs
 */
function reportsUnreadableRecord(output: string): boolean {
  /**
   The output in lower case, so the telltale words match whatever case the wrapper printed.
   */
  const lowered = output.toLowerCase();
  return UNREADABLE_RECORD_WORDS.some(function mentioned(word) {
    return lowered.includes(word);
  });
}

/**
 Check a phase whose records name identities that a copy cannot reproduce.

 @param phase - stored phase

 @param matrix - recorder of the driver's checks

 @example
 ```ts
 await checkIdentityBoundPhase({ phase: 'prepared', matrix });
 ```
 */
export async function checkIdentityBoundPhase({
  phase,
  matrix,
}: {
  readonly phase: string;
  readonly matrix: Matrix;
}): Promise<void> {
  /**
   Fixture name and message prefix of this phase.
   */
  const name = `phase-${phase}`;
  /**
   The restored fixture of this phase.
   */
  const fixture = await restoreFixture(name);
  /**
   HEAD before the wrapper runs, which a refusal must leave alone.
   */
  const base = await realGit({
    fixture,
    args: [
      'rev-parse',
      'HEAD'
    ],
  });
  /**
   Stored state before the wrapper runs.
   */
  const before = await fingerprintOf({
    repository: fixture.repository,
    subdirectories: STORED_STATE_DIRECTORIES,
  });
  /**
   The native `git status`, which recovers the stored records or refuses on identity.
   */
  const status = await runThrough({
    wrapper: 'native',
    fixture,
    args: [
      'status',
      '--porcelain'
    ],
  });
  /**
   Both streams of the status run, searched for refusal wording.
   */
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
  /**
   HEAD after the wrapper ran.
   */
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
    /**
     Stored state after the wrapper refused.
     */
    const after = await fingerprintOf({
      repository: fixture.repository,
      subdirectories: STORED_STATE_DIRECTORIES,
    });
    /**
     Stored files that a refusal lost beyond housekeeping.
     */
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
  /**
   Subjects of the commits above the base.
   */
  const log = await realGit({
    fixture,
    args: [
      'log',
      '--format=%s',
      `${base}..HEAD`
    ],
  });
  /**
   Subjects of the landed commits, as lines.
   */
  const subjects = linesOf(log);
  matrix.check({
    condition: (head === base) || (subjects.join('|') === `change at ${phase}`),
    message: `${name}: HEAD is at the base or at the incumbent's landed commit (${subjects.join('|') || 'base'})`,
  });
  /**
   Real Git's integrity check of the repository after a successful recovery.
   */
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

 @param matrix - recorder of the driver's checks

 @param phase - stored phase

 @param landed - whether the incumbent's commit had landed

 @example
 ```ts
 await checkRestorablePhase({ matrix, phase: 'landed', landed: true });
 ```
 */
export async function checkRestorablePhase({
  matrix,
  phase,
  landed,
}: StoredPhase & { readonly matrix: Matrix; }): Promise<void> {
  /**
   Fixture name and message prefix of this phase.
   */
  const tag = `phase-${phase}`;
  /**
   The restored fixture of this phase.
   */
  const fixture = await restoreFixture(tag);
  /**
   HEAD before the wrapper runs, the point the incumbent's commit may sit above.
   */
  const base = await realGit({
    fixture,
    args: [
      'rev-parse',
      'HEAD'
    ],
  });
  /**
   Entries of the stored transaction directory.
   */
  const stored = await namesIn(join(
    fixture.repository,
    '.git/cli-git-transactions',
  ));
  /**
   Stored transaction directories left by the dead incumbent, as opposed to lock directories.
   */
  const dead = stored.filter(function isTransaction(entry) {
    return !entry.endsWith('.lock');
  });
  matrix.check({
    condition: dead.length === EXPECTED_DEAD_TRANSACTIONS,
    message: `${tag}: the stored incumbent transaction directory is present (${dead.join(', ')})`,
  });
  /**
   The native `git status`, which must recover the stored transaction.
   */
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
  /**
   Subjects of the commits above the base, which show whether the incumbent's commit landed.
   */
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
  /**
   Real Git's comparison of the index with HEAD for the edited file.
   */
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
  /**
   Content of the edited file, which recovery must leave in the worktree.
   */
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
  /**
   Real Git's integrity check of the repository after recovery.
   */
  const fsck = await fsckOf(fixture);
  matrix.check({
    condition: fsck.status === 0,
    message: `${tag}: git fsck is clean (${excerpt({
      text: fsck.stderr,
      limit: SHORT_EXCERPT,
    })})`,
  });
  /**
   What the wrapper left behind after recovery.
   */
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
  /**
   The native follow-up commit of the new file.
   */
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
  /**
   Subject of the newest commit, which must be the follow-up.
   */
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
