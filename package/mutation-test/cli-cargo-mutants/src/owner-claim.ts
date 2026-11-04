/**
 The right of one running command to one item, held as a numbered claim file that names its owner by process birth.

 This is the TypeScript counterpart of `package/git-policy/cli/src/native/lock/single_owner_claim.rs`, with the same rules, and
 the comments cite its lines. A claim file is made whole in a private temporary file and published with one exclusive
 `link` under the next generation number `<prefix>.<generation>` (`single_owner_claim.rs:182`), so the kernel lets exactly
 one command create generation `n + 1`. No lock is held and nothing ever waits: a command that finds a live owner is told
 so and stops. The newest generation is the claim. A newer generation replaces an older one only when the older one records
 a released claim or an owner whose process no longer exists with that birth (the process id alone is not trusted, because
 it is reused; `single_owner_claim.rs:211`), so a killed command leaves nothing to clean up by hand. The newest generation
 file is never removed, because removing it would let a command that read an older listing create the same number again; a
 release rewrites it in place by renaming a one-line record over it (`single_owner_claim.rs:308`), and the command that
 takes the next generation removes the older ones (`single_owner_claim.rs:276`). An owner in another PID namespace, or one
 whose namespace is not named, is searched for among the processes this one can see by its id inside its own namespace and its
 start time (`src/foreign-process-scan.ts`); only evidence ends its claim, and no age does.

 What a claim cannot know, and the kernel's advisory lock it replaces does: whether the recorded process is the one that
 still does the work. A claim judges its owner from process id, start time and boot, which is exact for a process that was
 killed or lost to a reboot, but an owner that hands the work to a child and exits reads as gone.
 */
import { unlink } from 'node:fs/promises';

import {
  claimGone,
  publishNew,
  readClaimRecord,
  removeGenerations,
  rewriteAsReleased,
} from './claim-files.ts';
import {
  liveOwnerPid,
  noLiveOwner,
} from './claim-liveness.ts';
import {
  claimPath,
  type ClaimRecord,
  listGenerations,
} from './claim-record.ts';
import { CargoMutantsToolingError } from './error.ts';
import { isMissing } from './fs-guards.ts';
import { currentProcessIdentity } from './process-identity.ts';

/**
 The most rounds one {@link takeClaim} makes (`single_owner_claim.rs:44`). Each round after the first follows the progress of
 another command, so more rounds than this mean claimants keep arriving faster than this one can look.
 */
const mostRounds = 64;

/**
 A claim this process holds: the right to work on one item.
 */
export type OwnerClaim = {
  /**
   The claim file this process created.
   */
  readonly path: string;
  /**
   The number of the claim file.
   */
  readonly generation: number;
  /**
   Give the item back: the claim file is rewritten, atomically, as a released claim, so the next command takes the next
   generation without judging a process (`single_owner_claim.rs:308`). It is kept as the newest generation. Giving back
   twice does nothing the second time.
   */
  readonly release: () => Promise<void>;
  /**
   Give the item back when the scope ends, so every way out of the command, an early error included, releases it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 The live owner that stopped a claim.
 */
export type BusyOwner = {
  /**
   The owner's process id.
   */
  readonly pid: number;
  /**
   The claim file that records the owner.
   */
  readonly path: string;
};

/**
 This process holds the item.
 */
export type HeldOutcome = {
  /**
   Marks the item as taken.
   */
  readonly kind: 'held';
  /**
   The claim, to release when the work ends.
   */
  readonly claim: OwnerClaim;
};

/**
 A live process holds the item.
 */
export type BusyOutcome = {
  /**
   Marks the item as owned by another.
   */
  readonly kind: 'busy';
  /**
   The owner.
   */
  readonly owner: BusyOwner;
};

/**
 How a claim attempt ended: a busy item is a normal answer (the other command is working), not a failure.
 */
export type ClaimOutcome = HeldOutcome | BusyOutcome;

/**
 What one round decided: publish this number, stop for this owner, or look again because the listing went stale.
 */
type Decision = BusyOutcome | {
  readonly kind: 'publish';
  readonly generation: number
} | { readonly kind: 'again' };

/**
 Everything a round needs, and which round it is.
 */
type RoundParameters = {
  readonly directory: string;
  readonly prefix: string;
  readonly record: ClaimRecord;
  readonly afterListing?: () => Promise<void>;
  readonly afterPublishing?: () => Promise<void>;
  readonly round: number;
};

/**
 Make the claim this process holds on a published generation.

 @param directory - the directory of the claim files

 @param path - the claim file

 @param generation - its number

 @returns the claim

 @example
 ```ts
 createClaim({ directory: '/d', path: '/d/item.owner.1', generation: 1 });
 ```
 */
function createClaim({
  directory,
  path,
  generation,
}: {
  readonly directory: string;
  readonly path: string;
  readonly generation: number;
}): OwnerClaim {
  /**
   Whether the claim was given back already.
   */
  const state = { released: false };
  /**
   Give the item back once.
   */
  async function release(): Promise<void> {
    if (state.released)
      return;
    state.released = true;
    await rewriteAsReleased({
      directory,
      path,
    });
  }
  return Object.freeze({
    path,
    generation,
    release,
    [Symbol.asyncDispose]: release,
  });
}

/**
 Remove the generation this call published and lost with.

 @param path - the claim file

 @example
 ```ts
 await withdraw('/d/item.owner.1');
 ```
 */
async function withdraw(path: string): Promise<void> {
  try {
    await unlink(path);
  }
  catch (error) {
    if (!isMissing(error))
      throw error;
  }
}

/**
 Decide what a round does from the generations it listed (`single_owner_claim.rs:252`).

 @param directory - the directory of the claim files

 @param prefix - the claim series' name

 @param existing - the generations listed, ascending

 @returns the number to publish, the live owner, or that the listing went stale

 @throws CargoMutantsToolingError when the newest generation cannot be judged

 @example
 ```ts
 await decide({ directory: '/d', prefix: 'item.owner', existing: [1] });
 ```
 */
async function decide({
  directory,
  prefix,
  existing,
}: {
  readonly directory: string;
  readonly prefix: string;
  readonly existing: readonly number[];
}): Promise<Decision> {
  /**
   The newest generation, the claim.
   */
  const newest = existing.at(-1);
  if (newest === undefined)
    return {
      kind: 'publish',
      generation: 1,
    };
  /**
   Its file.
   */
  const path = claimPath({
    directory,
    prefix,
    generation: newest,
  });
  /**
   What it says.
   */
  const record = await readClaimRecord(path);
  if (record === claimGone)
    return { kind: 'again' };
  /**
   Its owner, when alive.
   */
  const pid = await liveOwnerPid({
    record,
    path,
  });
  if (pid !== noLiveOwner)
    return {
      kind: 'busy',
      owner: {
        pid,
        path,
      },
    };
  return {
    kind: 'publish',
    generation: newest + 1,
  };
}

/**
 Run one round of {@link takeClaim}, and the rounds that follow when another command moved first.

 @param parameters - the series, this process's claim record, the pause for tests, and the number of this round

 @returns the outcome

 @throws CargoMutantsToolingError when the newest generation cannot be judged, or claimants kept arriving faster than this
 one could look

 @example
 ```ts
 await takeInRounds({ directory: '/d', prefix: 'item.owner', record, round: 1 });
 ```
 */
async function takeInRounds(parameters: RoundParameters): Promise<ClaimOutcome> {
  /**
   The parts of the parameters this round reads.
   */
  const {
    directory,
    prefix,
    record,
    afterListing,
    afterPublishing,
    round,
  } = parameters;
  if (round > mostRounds)
    throw new CargoMutantsToolingError(`Other commands kept taking the claim in ${directory} while this one looked; nothing was changed, run the command again.`);
  /**
   The generations now.
   */
  const existing = await listGenerations({
    directory,
    prefix,
  });
  if (afterListing !== undefined)
    await afterListing();
  /**
   What this round does.
   */
  const decision = await decide({
    directory,
    prefix,
    existing,
  });
  if (decision.kind === 'busy')
    return decision;
  if (decision.kind === 'again')
    return await takeInRounds({
      ...parameters,
      round: round + 1,
    });
  /**
   Where this claim goes.
   */
  const path = claimPath({
    directory,
    prefix,
    generation: decision.generation,
  });
  /**
   Whether this call created the generation, the kernel having let exactly one command do so.
   */
  const created = await publishNew({
    directory,
    target: path,
    record,
  });
  if (!created)
    return await takeInRounds({
      ...parameters,
      round: round + 1,
    });
  if (afterPublishing !== undefined)
    await afterPublishing();
  // A command that read an older listing may have published a lower number after the others were removed; a newer
  // generation than this one means this claim lost (`single_owner_claim.rs:272`).
  /**
   The generations after the publication.
   */
  const after = await listGenerations({
    directory,
    prefix,
  });
  if (after.some(function isNewer(generation) {
    return generation > decision.generation;
  })) {
    // Withdrawn; a later claimant may have removed it already as an older generation, which is the same outcome
    // (`single_owner_claim.rs:273` ignores the result of the removal).
    await withdraw(path);
    return await takeInRounds({
      ...parameters,
      round: round + 1,
    });
  }
  await removeGenerations({
    directory,
    prefix,
    older: existing.filter(function isOlder(generation) {
      return generation < decision.generation;
    }),
  });
  return {
    kind: 'held',
    claim: createClaim({
      directory,
      path,
      generation: decision.generation,
    }),
  };
}

/**
 Take the claim on the item whose claim files are `<prefix>.<generation>` in `directory`, or report the live owner
 (`single_owner_claim.rs:248`).

 The call never waits: it lists the generations, reads the newest, publishes the next one exclusively, confirms that no
 newer one appeared, and removes the older ones.

 @param directory - the directory of the claim files, which must exist

 @param prefix - the claim series' name

 @param afterListing - runs between the listing and the publication, where a test lets another command act; absent in use

 @param afterPublishing - runs between the publication and the check for a newer generation, for the same purpose; absent in
 use

 @returns the held claim or the live owner

 @throws CargoMutantsToolingError when the newest generation is not a claim, or whether its owner runs cannot be decided

 @example
 ```ts
 const outcome = await takeClaim({ directory: '/d', prefix: 'item.owner' });
 ```
 */
export async function takeClaim({
  directory,
  prefix,
  afterListing,
  afterPublishing,
}: {
  readonly directory: string;
  readonly prefix: string;
  readonly afterListing?: () => Promise<void>;
  readonly afterPublishing?: () => Promise<void>;
}): Promise<ClaimOutcome> {
  /**
   This process, as an owner (`single_owner_claim.rs:227`).
   */
  const own = await currentProcessIdentity();
  return await takeInRounds({
    directory,
    prefix,
    record: {
      kind: 'owned',
      ...own,
    },
    ...(afterListing === undefined ? {} : { afterListing }),
    ...(afterPublishing === undefined ? {} : { afterPublishing }),
    round: 1,
  });
}

/**
 List the generations of a series, none for a directory that is not there.

 @param directory - the directory of the claim files

 @param prefix - the claim series' name

 @returns the generations, ascending

 @example
 ```ts
 await listOrNone({ directory: '/d', prefix: 'item.owner' });
 ```
 */
async function listOrNone({
  directory,
  prefix,
}: {
  readonly directory: string;
  readonly prefix: string;
}): Promise<readonly number[]> {
  try {
    return await listGenerations({
      directory,
      prefix,
    });
  }
  catch (error) {
    if (isMissing(error))
      return [];
    throw error;
  }
}

/**
 The live owner of the item whose claim files are `<prefix>.<generation>` in `directory`, if any; nothing is created
 (`single_owner_claim.rs:294`).

 @param directory - the directory of the claim files; absent is read as no claim

 @param prefix - the claim series' name

 @returns the owner, or {@link noLiveOwner}

 @throws CargoMutantsToolingError when the newest generation is not a claim, or whether its owner runs cannot be decided

 @example
 ```ts
 await liveOwnerOf({ directory: '/d', prefix: 'item.owner' });
 ```
 */
export async function liveOwnerOf({
  directory,
  prefix,
}: {
  readonly directory: string;
  readonly prefix: string;
}): Promise<BusyOwner | typeof noLiveOwner> {
  /**
   The newest generation.
   */
  const newest = (await listOrNone({
    directory,
    prefix,
  })).at(-1);
  if (newest === undefined)
    return noLiveOwner;
  /**
   Its file.
   */
  const path = claimPath({
    directory,
    prefix,
    generation: newest,
  });
  /**
   What it says.
   */
  const record = await readClaimRecord(path);
  if (record === claimGone)
    return noLiveOwner;
  /**
   Its owner, when alive.
   */
  const pid = await liveOwnerPid({
    record,
    path,
  });
  if (pid === noLiveOwner)
    return noLiveOwner;
  return {
    pid,
    path,
  };
}
