import { randomUUID, } from 'node:crypto';
import {
  link,
  mkdir,
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { failureName, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  contextRoot,
  rethrowUnlessMissingPath,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  hostIdentity,
  startTicksOf,
} from './process-identity.ts';
import {
  type HeldJudgement,
  holderLiveness,
  type LockHolder,
  lockFileText,
  readHolder,
} from './runs-lock-holder.ts';

//region Runs lock
// ONE pass at a time per runs directory.
//
// Nothing stopped two from sharing one. They would interleave on every durable
// thing the run owns, and each failure is silent in its own way:
//
//   `attempts.json` is read at startup and rewritten before each entry, so the
//   second pass writes counts derived from a map it read before the first pass
//   started. Attempts stop counting attempts, and attempts order the queue.
//
//   A slice cache is opened per entry, and opening one under a different
//   pipeline DELETES it. Two passes under different builds would take turns
//   destroying each other's finished slices, each recomputing what the other
//   just threw away, for as long as both ran.
//
//   Artifacts are written atomically, so neither is torn, but the later write
//   wins and the earlier entry's work is simply gone.
//
// The atomic rename in `writeFileAtomic` protects a READER from a half-written
// file. It says nothing about two writers, which is this. Who holds a lock and
// whether it still runs is `runs-lock-holder.ts`.
//
// THE LOCK APPEARS WITH ITS HOLDER ALREADY IN IT. It used to be created empty
// and written after, and a starter that finds a lock it cannot read takes it
// over, since a pass killed between the two leaves exactly that. So a second
// starter arriving between the create and the write read a live holder's
// empty lock as unreadable, moved it aside and claimed its own, and both
// passes went on: measured on 2026-10-05, two starters at once on a fresh
// directory in one process both held it in 28 of 200 rounds. The holder's
// text is now written to a name only this claim knows and hard-linked to the
// lock's name, which fails when anything stands there, so the claim and the
// text are one filesystem operation. A runs directory on a filesystem without
// hard links refuses the lock with that filesystem's own error.

/**
 Logger every lock line goes through; the lock takes no caller-supplied one.
 */
const lockLog = contextRoot({ tag: 'runs-lock', },);

/**
 Name of the lock file inside a runs directory.
 */
const LOCK_FILE = 'pass.lock';

/**
 What each judgement lets a refusal say about its holder.
 */
const HELD_BECAUSE: Readonly<Record<HeldJudgement, string>> = {
  identity: 'It names a process running now that started when the lock was taken, so the holder is alive.',
  pid: 'It was judged by process id alone, since the lock records no start time or this host could '
    + 'not read one: the id is in use, possibly by a process that received it after the holder ended. '
    + 'If no pass runs under that id, delete the lock file.',
  namespace: 'It was taken in another process-id namespace (a container, say), where its id counts '
    + 'differently, so whether its holder runs cannot be judged from here.',
  host: 'It was taken on another machine, whose processes this one cannot see, so whether its holder '
    + 'runs cannot be judged from here.',
  race: 'Another pass took it over at the same moment as this one.',
};

/**
 Raised when another pass already owns this runs directory.
 */
export class RunsDirectoryBusyError extends StatedRefusalError {
  /**
   Declares this message safe to forward: it names a process id, its start
   time, the directory, and a fixed phrase saying how the holder was judged.
   */
  override readonly messageNamesOnly: true = true;

  /**
   Names the holder, how it was judged, and the two ways forward.

   @param runsDir - directory whose lock is held

   @param holder - what the lock file records, absent when unreadable

   @param judgedBy - how the holder was judged to hold it

   @example
   ```ts
   throw new RunsDirectoryBusyError({ runsDir, holder, judgedBy: 'identity', },);
   ```
   */
  constructor(
    {
      runsDir,
      holder,
      judgedBy,
    }: {
      readonly runsDir: string;
      readonly holder?: LockHolder;
      readonly judgedBy: HeldJudgement;
    },
  ) {
    super({
      says: [
        `Another pass is running in ${runsDir}.`,
        ...(holder === undefined
          ? ['  its lock file records nothing readable',]
          : [
            `  process ${String(holder.pid,)}, since ${holder.startedAt}`,
          ]),
        '',
        'Two passes sharing one runs directory do not conflict loudly. They',
        'overwrite each other\'s attempt counts, delete each other\'s cached',
        'slices whenever their pipelines differ, and the later write of any',
        'entry simply replaces the earlier one. Every one of those looks like',
        'ordinary output.',
        '',
        'Point this run at another directory with TRANSLATION_REPAIR_RUNS_DIR,',
        'or stop the other pass. A lock whose process is gone is taken over',
        `automatically. ${HELD_BECAUSE[judgedBy]}`,
      ].join('\n',),
    },);
    this.name = 'RunsDirectoryBusyError';
  }
}

/**
 Links a written lock into place, failing rather than overwriting.

 @param staged - file already holding the holder's whole text

 @param path - lock file path

 @returns Whether the link made the lock, false where something stands there

 @throws whatever the filesystem refuses the link with, for any failure but
 the lock already existing

 @example
 ```ts
 const won = await linkedInPlace({ staged, path, },);
 ```
 */
async function linkedInPlace(
  {
    staged,
    path,
  }: {
    readonly staged: string;
    readonly path: string;
  },
): Promise<boolean> {
  try {
    await link(
      staged,
      path,
    );
    return true;
  }
  catch (error) {
    if (Error.isError(error,) && ('code' in error)
      && (error.code === 'EEXIST'))
      return false;
    throw error;
  }
}

/**
 Removal of a claim's staged text when the claim's scope ends, whatever its
 link answered.

 A REMOVAL THAT FAILS is said on a warning naming the file and the filesystem
 code rather than thrown: the claim's own answer, won, lost or refused, is
 what the starter acts on, and the file left is one name nothing reads. The
 warning says the file stands only if the claim wrote it, since a write that
 failed (a name too long for the system, say) fails the removal the same way
 with nothing left; until 2026-10-10 it said the file was left either way.

 @param staged - name only that claim knows, absent where its write failed
 before creating it

 @returns Disposable removing it

 @example
 ```ts
 await using _staging = stagedRemoval({ staged, },);
 ```
 */
function stagedRemoval(
  { staged, }: { readonly staged: string; },
): AsyncDisposable {
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      try {
        await rm(
          staged,
          { force: true, },
        );
      }
      catch (error) {
        lockLog.warn(
          `${staged} could not be removed after a claim (${failureName({ error, },)}); it stands beside the lock if `
            + 'the claim wrote it',
        );
      }
    },
  };
}

/**
 Tries to create the lock file, holder and all, failing rather than
 overwriting.

 A LINK MAKES THE CHECK, THE CLAIM AND THE TEXT ONE FILESYSTEM OPERATION,
 which is the whole mechanism. Checking for the file and then creating it
 leaves a window in which two passes both see it absent and both proceed;
 creating it empty and then writing it leaves one in which another starter
 reads it as unreadable and takes it over. The text goes first to a name only
 this claim knows, beside the lock so the link stays on one filesystem, and
 that name is removed whatever the link answers.

 @param path - lock file path

 @param holder - what to record inside it

 @returns Whether this call created it

 @throws whatever the filesystem refuses writing the text or the link with,
 for any failure but the lock already existing

 @example
 ```ts
 const won = await claim({ path, holder, },);
 ```
 */
async function claim(
  {
    path,
    holder,
  }: {
    readonly path: string;
    readonly holder: LockHolder;
  },
): Promise<boolean> {
  /**
   Name only this claim knows, holding the text until the link.
   */
  const staged = `${path}.claim-${randomUUID()}`;
  /**
   Removes the staged text when this claim ends, won, lost or refused.
   */
  await using _staging = stagedRemoval({ staged, },);
  await writeFile(
    staged,
    lockFileText({ holder, },),
    { flag: 'wx', },
  );
  return await linkedInPlace({
    staged,
    path,
  },);
}

/**
 Takes exclusive ownership of a runs directory for the life of a scope.

 Linked into place with its holder already written, so the check, the claim
 and the text are one filesystem operation: two passes starting together
 cannot both win, and no starter ever finds a live holder's lock empty. A
 lock whose process is gone is taken over, since a pass killed at its hard
 cap leaves one behind and refusing forever would make every crash need
 manual cleanup; so is one that says nothing readable, which a pass of a
 build before this one left when killed between creating and writing its
 lock, since a live holder's lock of this build is never seen that way.

 @param runsDir - durable output root this pass owns

 @returns Disposable releasing the lock

 @throws RunsDirectoryBusyError when a live process already holds it, or when
 this call loses a concurrent race for a stale lock to another starter

 @throws whatever the filesystem refuses creating the runs directory, the
 claim's text or the lock file with, for any failure but the lock file
 already existing (an unwritable runs directory, or one on a filesystem
 without hard links, say), unwrapped

 @example
 ```ts
 await using _lock = await lockRunsDir({ runsDir, },);
 ```
 */
export async function lockRunsDir(
  { runsDir, }: { readonly runsDir: string; },
): Promise<AsyncDisposable> {
  // The directory has to exist before anything in it can be opened, and this is
  // the FIRST thing a pass touches. Without this a fresh runs directory fails
  // outright with ENOENT on the lock file, which reads as a locking problem
  // rather than as a missing directory and sends a reader to the wrong module.
  // A pass pointed at a new directory is ordinary: it is how a run is kept to
  // one pipeline generation.
  await mkdir(
    runsDir,
    { recursive: true, },
  );

  /**
   Path of the lock file this pass competes for.
   */
  const path = join(
    runsDir,
    LOCK_FILE,
  );

  /**
   This host, and when this process started, so the next pass can tell this
   process from a later one given the same id (ledger A16).
   */
  const [
    here,
    started,
  ] = await Promise.all([
    hostIdentity(),
    startTicksOf({ pid: process.pid, },),
  ],);

  /**
   What this pass writes into the lock file.
   */
  const holder: LockHolder = {
    pid: process.pid,
    startedAt: new Date().toISOString(),
    token: randomUUID(),
    identity: ((here.kind === 'read') && (started.kind === 'read'))
      ? {
        kind: 'recorded',
        ...here.here,
        startTicks: started.startTicks,
      }
      : { kind: 'unrecorded', },
  };

  if (!await claim({
    path,
    holder,
  },)) {
    /**
     Whoever the existing lock names.
     */
    const existing = await readHolder({ path, },);

    if (existing.kind === 'holder') {
      /**
       Process the existing lock names.
       */
      const { holder: heldBy, } = existing;

      /**
       Whether that process still runs, and on what evidence.
       */
      const liveness = await holderLiveness({
        holder: heldBy,
        here,
      },);

      if (liveness.state === 'held')
        throw new RunsDirectoryBusyError({
          runsDir,
          holder: heldBy,
          judgedBy: liveness.judgedBy,
        },);

      lockLog.info(
        `taking over a stale lock in ${runsDir} from gone process ${
          String(heldBy.pid,)
        } (judged by ${liveness.judgedBy})`,
      );
    }
    else
      lockLog.info(`taking over an unreadable lock in ${runsDir}`,);

    await evictStaleLock({ path, },);

    // ONE retry, not a loop and not recursion. Two passes finding the same
    // stale lock both try to evict it; the eviction is a rename, so exactly
    // one of them evicts and the other finds it gone, then exactly one create
    // succeeds and the loser faces a live holder rather than a stale one, so
    // retrying again could only spin against a lock that is genuinely held.
    if (!await claim({
      path,
      holder,
    },)) {
      /**
       Whoever won the race this pass lost.
       */
      const winner = await readHolder({ path, },);

      throw new RunsDirectoryBusyError({
        runsDir,
        ...(winner.kind === 'unreadable' ? {} : { holder: winner.holder, }),
        judgedBy: 'race',
      },);
    }
  }

  return {
    async [Symbol.asyncDispose](): Promise<void> {
      await releaseIfOwned({
        path,
        holder,
      },);
    },
  };
}

/**
 Removes a stale lock so that exactly one of any number of concurrent
 starters does it.

 A RENAME, NOT A REMOVE. Two starters that both found the lock stale and both
 removed it could interleave as remove, claim, remove, claim, the second
 remove deleting the first starter's fresh lock, and both passes then ran in
 one directory. A rename to a name only this call knows is atomic:
 the first starter's rename succeeds and the second's finds nothing to
 rename, so the second proceeds straight to a claim it will lose.

 @param path - lock file to evict

 @returns `evicted` when this call moved the lock aside, `gone` when another
 starter had already done so

 @example
 ```ts
 const outcome = await evictStaleLock({ path, },);
 ```

 @internal
 */
export async function evictStaleLock(
  { path, }: { readonly path: string; },
): Promise<'evicted' | 'gone'> {
  /**
   Name only this call knows, so two evictions cannot collide on it either.
   */
  const asideName = `${path}.stale-${randomUUID()}`;
  try {
    await rename(
      path,
      asideName,
    );
  }
  catch (error) {
    rethrowUnlessMissingPath({ error, },);
    return 'gone';
  }
  await rm(
    asideName,
    { force: true, },
  );
  return 'evicted';
}

/**
 Removes the lock file only when it still carries this acquisition's token,
 so a starter that lost a takeover cannot delete the winner's lock on its
 way out. Says so when it keeps one.

 @param path - lock file

 @param holder - holder this acquisition wrote, of which only its token decides

 @returns `released` when the file was ours and is gone, `kept` otherwise

 @example
 ```ts
 const outcome = await releaseIfOwned({ path, holder, },);
 ```

 @internal
 */
export async function releaseIfOwned(
  {
    path,
    holder,
  }: {
    readonly path: string;
    readonly holder: Pick<LockHolder, 'token'>;
  },
): Promise<'released' | 'kept'> {
  /**
   Whoever holds the file now.
   */
  const current = await readHolder({ path, },);
  if (current.kind === 'holder') {
    /**
     Holder the file names now.
     */
    const { holder: found, } = current;
    if (found.token === holder.token) {
      await rm(
        path,
        { force: true, },
      );
      return 'released';
    }
  }
  lockLog.warn(`${path} is not this acquisition's any more; leaving it in place`,);
  return 'kept';
}

//endregion Runs lock
