/**
 The claim one filling holds on one entry of a queue, and the three steps that use it: taking it before anything is started, completing it
 when the entry has started, and giving it back when the machine refused the start.

 An entry used to be started first and taken out of the queue afterwards, by name, so two fillings started one entry, a filling killed
 between its start and its removal left the entry to be started again, and a completed start removed whatever entry bore that name when it
 finished, which could be a replacement queued meanwhile. A name is not an identity: the entry a filling starts is the one whose `queueId`
 it read, and the claim it holds is a file of its own, named after the entry, holding the token of the attempt.

 The claim is a file linked into a name only one filling can link, so exactly one filling takes it. The start happens while that file is
 there, and the batch list, its unit and that unit's output file are named from the claim's token, so the claim names the evidence a reader
 asks about after a kill without naming a path.

 What a kill leaves, and what the next pass does with it, is written in the README's "The states of an entry, and the one step between
 each two", and settled by `settling-the-queue-after-a-kill.ts`.

 @module
 */

import {
  mkdir,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { randomUUID, } from 'node:crypto';
import { isThere, } from './newest-finished-run-of-a-name.ts';
import {
  type AClaimOnAnEntry,
  claimFileOf,
  NO_QUEUE_ID,
  type QueueClaim,
  type QueueRecord,
  QueueFileError,
  claimedFolderOf,
  entryFileOf,
  queueIdOf,
} from './queue-of-runs.ts';
import { aClaimOf, } from './places-and-names-of-a-queue-folder.ts';
import { readTheClaimsOnAQueue, } from './reading-the-claims-beside-a-queue.ts';
import {
  NO_ENTRY_OF_THAT_NAME,
  aFileLaidDown,
  readTheEntryNamed,
} from './reading-and-writing-the-folder-a-queue-is.ts';

/**
 Characters in the stamp a batch list's name begins with, up to the second: `2026-11-14T09:21:14`.
 */
const STAMP_CHARACTERS = 19;

/**
 How many characters of an entry's name go into the batch list's file name, which also becomes its unit's name.
 */
const NAME_CHARACTERS_IN_A_FILE = 60;

/**
 The extension of the batch list a start writes, which its unit's output file is named from.
 */
const LIST_EXTENSION = '.json';

/**
 The ending of the output file one attempt's unit writes both of its streams to.
 */
const OUTPUT_EXTENSION = '.out';

/**
 No claim on that entry, as a symbol so no claim can be mistaken for none.
 */
export const NO_CLAIM_ON_THAT_ENTRY: unique symbol = Symbol('this entry carries the claim of no filling',);

/**
 What taking a claim on one entry answered: the claim this filling holds, or why there was nothing to claim.
 */
export type ClaimedAnEntry =
  | {
    readonly kind: 'claimed';

    /**
     The entry's own identity, which the claim names.
     */
    readonly queueId: string;

    /**
     The claim this filling holds.
     */
    readonly claim: QueueClaim;
  }
    | {
      /**
       The entry left the queue before this filling claimed it, so nothing is started for it.
       */
      readonly kind: 'no-longer-queued';
    }
    | {
      /**
       Another filling holds a claim on the entry, or the queue holds an entry of that name other than the one read, so this filling
       starts nothing for it and asks about the entry behind it.
       */
      readonly kind: 'claimed-by-another';
    };

/**
 The batch list, the unit and the output file one start attempt is named with.

 The names come from the claim's token, so the filling that recovers a dead claim asks about the same file the dead attempt would have
 written, and a second attempt on the same entry never answers for the first.

 @param runsFolder - folder the driver writes its runs into

 @param name - the entry's name

 @param claim - the claim naming the attempt

 @returns the list's path and the path the attempt's unit writes its streams to

 @example
 ```ts
 const names = namesOfAnAttemptIn({ runsFolder: '/runs', name: 'a', claim });
 ```
 */
export function namesOfAnAttemptIn({
  runsFolder,
  name,
  claim,
}: {
  readonly runsFolder: string;
  readonly name: string;
  readonly claim: QueueClaim;
},): {
  readonly batchFile: string;
  readonly outputFile: string;
} {
  /**
   The stamp the list's name begins with, taken from the claim's own time so every reader of the attempt names it alike.
   */
  const stamp = claim.at
    .slice(
    0,
    STAMP_CHARACTERS,
    )
    .replaceAll(
    ':',
    '',
    );
  /**
   The list's path, beside the runs, named after the attempt and the entry.
   */
  const batchFile = join(
    runsFolder,
    `batch-${stamp}-${claim.token}-queue-entry-${name.slice(
      0,
      NAME_CHARACTERS_IN_A_FILE,
    )}${LIST_EXTENSION}`,
  );
  return {
    batchFile,
    outputFile: `${batchFile.slice(
      0,
      -LIST_EXTENSION.length,
    )}${OUTPUT_EXTENSION}`,
  };
}

/**
 Says whether one attempt's start took effect: whether its unit wrote the output file it was given.

 A unit's output file is opened by the manager when the unit starts, and it stays after the unit ends, so its presence is the evidence
 that the attempt was started. A claim whose holder is gone and whose output file is not there names an attempt that never reached its
 unit, which is the one an entry may be started for again.

 @param runsFolder - folder the driver writes its runs into

 @param name - the entry's name

 @param claim - the claim naming the attempt

 @returns whether the attempt's unit wrote its output

 @example
 ```ts
 const started = await startOfAnAttemptTookEffect({ runsFolder: '/runs', name: 'a', claim });
 ```
 */
export async function startOfAnAttemptTookEffect({
  runsFolder,
  name,
  claim,
}: {
  readonly runsFolder: string;
  readonly name: string;
  readonly claim: QueueClaim;
},): Promise<boolean> {
  /**
   Where the attempt's unit would write both of its streams.
   */
  const { outputFile, } = namesOfAnAttemptIn({
    runsFolder,
    name,
    claim,
  },);
  return await isThere(outputFile,);
}

/**
 Reads one entry's identity, which this filling read before it claimed the entry.

 @param record - the entry as this filling read it

 @returns the identity

 @throws QueueFileError when the entry carries no identity, which only an entry written by other means does

 @example
 ```ts
 const queueId = anIdentityOf({ record });
 ```
 */
function anIdentityOf({ record, }: { readonly record: QueueRecord },): string {
  /**
   The identity the entry carries.
   */
  const queueId = queueIdOf({ record, },);
  if (queueId === NO_QUEUE_ID) {
    throw new QueueFileError(
      `${record.name} carries no identity, so no filling can say which entry of that name it starts: every entry of a queue folder is written by this package's own put, which mints one`,
    );
  }
  return queueId;
}

/**
 Says whether the queue still holds the entry this filling read, by the identity it wrote down.

 @param queueFolder - the queue's folder

 @param name - the entry's name

 @param queueId - the identity this filling claims

 @returns whether the queue holds that very entry

 @throws QueueFileError when the entry's file is there and cannot be read

 @example
 ```ts
 const mine = await theQueueHoldsThisAttempt({ queueFolder: '/runs/q', name: 'a', queueId: 'q1' });
 ```
 */
async function theQueueHoldsThisAttempt({
  queueFolder,
  name,
  queueId,
}: {
  readonly queueFolder: string;
  readonly name: string;
  readonly queueId: string;
},): Promise<boolean> {
  /**
   The entry the queue holds under that name, or that it holds none.
   */
  const held = await readTheEntryNamed({
    queueFolder,
    name,
  },);
  return (held !== NO_ENTRY_OF_THAT_NAME) && (queueIdOf({ record: held, },) === queueId);
}

/**
 Takes a claim on one entry of the queue, before anything is started for it, so exactly one filling starts it.

 The claim is one file linked into a name of its own, which is a step only one filling can take. The queue's own entry is then read again,
 and a claim whose entry is gone or no longer the one read is given back and answered, so a claim is never held on an entry this filling
 did not read.

 @param queueFolder - the queue's folder

 @param record - the entry as this filling read it

 @param now - the clock the claim's time is read from

 @returns the claim this filling holds, or why there was nothing to claim

 @throws QueueFileError when the entry carries no identity, or the claim's file cannot be written

 @example
 ```ts
 const taken = await claimAnEntryOfAQueue({ queueFolder: '/runs/q', record, now: Date.now });
 ```
 */
export async function claimAnEntryOfAQueue({
  queueFolder,
  record,
  now,
}: {
  readonly queueFolder: string;
  readonly record: QueueRecord;
  readonly now: () => number;
},): Promise<ClaimedAnEntry> {
  /**
   The identity of the entry this filling read.
   */
  const queueId = anIdentityOf({ record, },);
  /**
   The claim this filling takes, whose token names the one start attempt.
   */
  const claim: QueueClaim = {
    token: randomUUID(),
    pid: process.pid,
    at: new Date(now(),).toISOString(),
  };
  try {
    await mkdir(
      claimedFolderOf({ queueFolder, },),
      { recursive: true, },
    );
  }
  catch (error) {
    throw new QueueFileError(`the claims beside the queue at ${queueFolder} cannot be made: ${String(error,)}`,);
  }
  /**
   Whether this filling linked its claim into place, or another filling holds the entry.
   */
  const linked = await aFileLaidDown({
    path: claimFileOf({
      queueFolder,
      name: record.name,
    },),
    text: `${JSON.stringify(aClaimOf({
      claim,
      name: record.name,
      queueId,
    },),)}\n`,
    replace: false,
  },);
  if (!linked) {
    return { kind: 'claimed-by-another', };
  }
  if (!(await theQueueHoldsThisAttempt({
    queueFolder,
    name: record.name,
    queueId,
  },))) {
    // The entry this claim is on is gone, or the queue holds another entry of that name now: the claim is given back, so nothing is
    // started for an entry this filling did not read.
    await rm(
      claimFileOf({
        queueFolder,
        name: record.name,
      },),
      { force: true, },
    );
    return (await readTheEntryNamed({
      queueFolder,
      name: record.name,
    },)) === NO_ENTRY_OF_THAT_NAME
      ? { kind: 'no-longer-queued', }
      : { kind: 'claimed-by-another', };
  }
  return {
    kind: 'claimed',
    queueId,
    claim,
  };
}

/**
 Completes one claim by taking its entry out of the queue, matched on the entry's identity.

 This is the step that makes a started entry started. The claim's file goes after the entry's, so a kill between them leaves a claim whose
 entry is gone, which the next pass clears; and an entry queued under the same name later is another identity and is left where it is.

 @param queueFolder - the queue's folder

 @param name - the entry's name

 @param queueId - the identity of the entry the claim was taken on

 @returns whether the entry was taken out, and whether the claim was still this filling's

 @throws QueueFileError when the entry's file is there and cannot be read

 @example
 ```ts
 const completed = await completeTheClaim({ queueFolder: '/runs/q', name: 'a', queueId: 'q1' });
 ```
 */
export async function completeTheClaim({
  queueFolder,
  name,
  queueId,
}: {
  readonly queueFolder: string;
  readonly name: string;
  readonly queueId: string;
},): Promise<{
  readonly removed: boolean;
  readonly claimWasHeld: boolean;
}> {
  /**
   Whether the queue still held this attempt's entry, which is the entry this step takes out.
   */
  const held = await theQueueHoldsThisAttempt({
    queueFolder,
    name,
    queueId,
  },);
  if (held) {
    await rm(
      entryFileOf({
        queueFolder,
        name,
      },),
      { force: true, },
    );
  }
  return {
    removed: held,
    claimWasHeld: await aClaimGivenBack({
      queueFolder,
      name,
    },),
  };
}

/**
 Gives one claim back, leaving its entry where it stands in the queue for the next pass to start.

 This is what a failure of the machine leaves: the entry keeps its place, and the queue's order is the whole point of the queue.

 @param queueFolder - the queue's folder

 @param name - the entry's name

 @returns whether a claim was there to give back

 @example
 ```ts
 const givenBack = await giveBackTheClaim({ queueFolder: '/runs/q', name: 'a' });
 ```
 */
export async function giveBackTheClaim({
  queueFolder,
  name,
}: {
  readonly queueFolder: string;
  readonly name: string;
},): Promise<boolean> {
  return await aClaimGivenBack({
    queueFolder,
    name,
  },);
}

/**
 Removes the claim on one entry, which is what giving a claim back and completing one both end with.

 @param queueFolder - the queue's folder

 @param name - the entry's name

 @returns whether a claim was there to remove

 @example
 ```ts
 const wasThere = await aClaimGivenBack({ queueFolder: '/runs/q', name: 'a' });
 ```
 */
async function aClaimGivenBack({
  queueFolder,
  name,
}: {
  readonly queueFolder: string;
  readonly name: string;
},): Promise<boolean> {
  /**
   The claim's file.
   */
  const file = claimFileOf({
    queueFolder,
    name,
  },);
  try {
    await rm(
      file,
      { force: true, },
    );
  }
  catch (error) {
    throw new QueueFileError(`the claim at ${file} cannot be removed: ${String(error,)}`,);
  }
  return true;
}

/**
 Reads what one claim's file holds, which the next pass reads when the filling that wrote it is gone.

 @param queueFolder - the queue's folder

 @param name - the entry's name

 @returns what the file holds, or nothing when there is no claim on that entry

 @throws QueueFileError when the file is there and cannot be read as a claim

 @example
 ```ts
 const claim = await theClaimOn({ queueFolder: '/runs/q', name: 'a' });
 ```
 */
export async function theClaimOn({
  queueFolder,
  name,
}: {
  readonly queueFolder: string;
  readonly name: string;
},): Promise<AClaimOnAnEntry | typeof NO_CLAIM_ON_THAT_ENTRY> {
  /**
   Every claim the queue holds, of which this asks about one name.
   */
  const claims = await readTheClaimsOnAQueue({ queueFolder, },);
  /**
   The claim on that entry, when a filling holds one.
   */
  const held = claims.find(function isOnIt(claim,) {
    return claim.name === name;
  },);
  return held ?? NO_CLAIM_ON_THAT_ENTRY;
}
