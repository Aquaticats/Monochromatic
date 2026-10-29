/**
 The native wrapper reads on-disk state in the format the retired incumbent wrapper wrote.

 The fixtures under `native-interop/fixture/` are plain files captured from the incumbent (pinned revision
 `f4fc1fa213ece65672e4314983a75266837892f9`) by `capture-fixtures.mjs`; `old-format-restore.ts` recreates them.
 Each fixture is restored to the absolute location it was captured at (its records name absolute paths), and then
 only the native wrapper runs against it:

 - `phase-capture-locked` and `phase-preparation-done`: the incumbent died before it touched the landing state.
   A native `git status` must recover the dead transaction; afterwards the commit did not land, the real index
   matches `HEAD`, the worktree keeps the edit, `git fsck` is clean, only a dead holder's lock directory may
   remain, and a native commit lands.
 - `phase-landing-locked`, `phase-objects-migrated`, `phase-ref-updated` and `phase-index-installed`: the records
   name device and inode numbers of files the incumbent created (the real `index.lock`, recovery hard links), and a
   restored copy of those files has other numbers. The native wrapper must read every stored record and either
   recover or refuse with its identity-changed message, never with a message about a malformed or unsupported
   record; the repository must stay whole (`git fsck` clean, `HEAD` at the base or at the landed commit).
   Full recovery of these four phases is proven live by `crash-matrix.mjs`.
 - `completed`: two finished incumbent commits left the capture store and landed-capture records; a native commit
   must land and advance the capture sequence.
 - `hook-lock-dead`: the incumbent died holding the hook lock; a native commit must retire the lock, run the hook and land.
 */
import {
  checkIdentityBoundPhase,
  checkRestorablePhase,
  type StoredPhase,
} from './old-format-phase-checks.ts';
import {
  checkCompleted,
  checkDeadHookLock,
} from './old-format-state-checks.ts';
import { createMatrix } from './support.ts';

/**
 Phases whose records hold no identity of a file that a copy cannot reproduce.
 */
const RESTORABLE_PHASES: readonly StoredPhase[] = [
  {
    phase: 'capture-locked',
    landed: false,
  },
  {
    phase: 'preparation-done',
    landed: false,
  },
];
/**
 Phases whose records name device and inode numbers of files the incumbent created.
 */
const IDENTITY_BOUND_PHASES: readonly StoredPhase[] = [
  {
    phase: 'landing-locked',
    landed: false,
  },
  {
    phase: 'objects-migrated',
    landed: false,
  },
  {
    phase: 'ref-updated',
    landed: true,
  },
  {
    phase: 'index-installed',
    landed: true,
  },
];

/**
 Recorder of the driver's checks, finished once all of them ran.
 */
const matrix = createMatrix();

/* oxlint-disable no-await-in-loop -- every phase restores below the same absolute location and runs the wrapper against it, so the phases must not overlap */
for (const stored of IDENTITY_BOUND_PHASES)
  await checkIdentityBoundPhase({
    ...stored,
    matrix,
  });
for (const stored of RESTORABLE_PHASES)
  await checkRestorablePhase({
    ...stored,
    matrix,
  });
/* oxlint-enable no-await-in-loop */
await checkCompleted({ matrix });
await checkDeadHookLock({ matrix });

matrix.finish();
console.log('old-format-driver passed');
