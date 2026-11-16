/**
 The two steps that change a queue only while the thing they act on is still the one they decided about: giving one claim back, and
 taking one entry out.

 Both are one rename each. The file is renamed to a name of this caller's own, read, and then either removed or renamed back, so the caller
 acts on the file it read and never on whatever took its name meanwhile. A claim is compared by its token and an entry by its identity,
 which is what tells a filling's own attempt from a namesake another filling took or a person queued in the same instant.

 Where a claim's holder has gone is read from the process number *and* from the moment that process started, which the kernel reports for
 every process: a number given to an unrelated process after its holder died holds the same number but not the same start moment, and a
 claim is not this caller's to take from a holder that might still be alive.

 @module
 */

import {
  link,
  readFile,
  rename,
  rm,
} from 'node:fs/promises';
import { randomUUID, } from 'node:crypto';
import { errorCodeOf, } from './error-code-of.ts';
import {
  type AClaimOnAnEntry,
  claimFileOf,
  entryFileOf,
  type QueueRecord,
  queueIdOf,
  QueueFileError,
} from './queue-of-runs.ts';
import { aNameTakenAside, } from './places-and-names-of-a-queue-folder.ts';
import { aClaimIn, } from './reading-the-claims-beside-a-queue.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The code the operating system gives a path that is there already, which is a name another writer took.
 */
const EXISTS_CODE = 'EEXIST';

/**
 Gives one claim back, only while the claim's file still holds the attempt this caller took.

 @param queueFolder - the queue's folder

 @param name - the entry's name

 @param token - the token naming the attempt whose claim is given back

 @returns whether the claim was this caller's and is now gone

 @throws QueueFileError when a claim's file cannot be read, renamed or removed

 @example
 ```ts
 const givenBack = await aClaimGivenBackOnlyIfOurs({ queueFolder: '/runs/q', name: 'a', token: 't1' });
 ```
 */
export async function aClaimGivenBackOnlyIfOurs({
  queueFolder,
  name,
  token,
}: {
  readonly queueFolder: string;
  readonly name: string;
  readonly token: string;
},): Promise<boolean> {
  return await aClaimRemovedOnlyIfItSays({
    queueFolder,
    name,
    holds: function isThisAttempt(claim,) {
      return claim.claim
        .token
        === token;
    },
  },);
}

/**
 Removes one claim, only while its file holds what the caller decided about, by renaming the file aside first.

 The rename is the step that makes the decision safe: the caller reads the file it moved, so a claim another filling linked a moment later
 is renamed back rather than removed, and the filling that holds it keeps it.

 @param queueFolder - the queue's folder

 @param name - the entry's name

 @param holds - what the claim the caller decided about says

 @returns whether the claim was the one decided about and is now gone

 @throws QueueFileError when the claim's file cannot be read, renamed or removed

 @example
 ```ts
 const removed = await aClaimRemovedOnlyIfItSays({ queueFolder: '/runs/q', name: 'a', holds: function holds(claim) { return claim.claim.pid === 1; } });
 ```
 */
export async function aClaimRemovedOnlyIfItSays({
  queueFolder,
  name,
  holds,
}: {
  readonly queueFolder: string;
  readonly name: string;
  readonly holds: (claim: AClaimOnAnEntry) => boolean;
},): Promise<boolean> {
  /**
   The claim's file.
   */
  const file = claimFileOf({
    queueFolder,
    name,
  },);
  /**
   Where the claim is renamed to while it is read.
   */
  const aside = aNameTakenAside({ path: file, },);
  try {
    await rename(
      file,
      aside,
    );
  }
  catch (error) {
    // A claim that is not there is one no filling holds, which is what a completed start and a given-back claim both leave.
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw new QueueFileError(`the claim at ${file} cannot be taken aside: ${String(error,)}`,);
  }
  /**
   What the claim the rename moved says, or what kept it from being read.
   */
  const read = await theClaimInAFile({ path: aside, },);
  if ('problem' in read) {
    // A claim that can no longer be read is a name a person has to look at: it is put back where it was, unless another writer took
    // the name meanwhile, and carried out.
    await putBackUnlessTheNameIsTaken({
      from: aside,
      to: file,
    },);
    throw read.problem;
  }
  if (!holds(read.claim,)) {
    // Another filling linked its claim between this caller's decision and this step, so the claim this caller moved is put back
    // unless that filling's own claim now holds the name.
    await putBackUnlessTheNameIsTaken({
      from: aside,
      to: file,
    },);
    return false;
  }
  await rm(
    aside,
    { force: true, },
  );
  return true;
}

/**
 Reads one claim's file, or answers why it could not be read as a claim.

 @param path - the claim's path

 @returns the claim, or the error a caller carries out

 @example
 ```ts
 const read = await theClaimInAFile({ path: '/runs/q/claimed/a.claim.json' });
 ```
 */
async function theClaimInAFile({ path, }: { readonly path: string },): Promise<
  | { readonly claim: AClaimOnAnEntry }
    | { readonly problem: QueueFileError }
  > {
  try {
    return {
      claim: aClaimIn({
        parsed: JSON.parse(await readFile(
          path,
          'utf8',
        ),),
        path,
      },),
    };
  }
  catch (error) {
    return {
      problem: new QueueFileError(`the claim taken aside from ${path} cannot be read as one: ${String(error,)}`,),
    };
  }
}

/**
 Puts one file a caller renamed aside back to its own name, and never writes over a file another writer put there meanwhile.

 The rename back is what a decision about a claim or an entry uses when the file turns out not to be the one decided about. A plain
 rename would replace whatever took the free name in the moment the file was aside, so a third filling's claim would be lost, or a
 person's entry would be written over by an older namesake. A link refuses a name already taken, which is the step that makes the put
 back safe.

 @param from - the file this caller renamed aside

 @param to - the name it had

 @returns whether it went back, or the name was taken

 @throws QueueFileError when the file cannot be linked back for any other reason

 @example
 ```ts
 const back = await putBackUnlessTheNameIsTaken({ from: '/runs/q/.being-taken-a.json-1-2', to: '/runs/q/a.json' });
 ```
 */
export async function putBackUnlessTheNameIsTaken({
  from,
  to,
}: {
  readonly from: string;
  readonly to: string;
},): Promise<boolean> {
  try {
    await link(
      from,
      to,
    );
  }
  catch (error) {
    if (errorCodeOf(error,) === EXISTS_CODE) {
      // Another writer made a file of that name while this caller held the one aside: it is theirs, and the file this caller moved
      // stays where a person finds it rather than being written over theirs.
      return false;
    }
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      // The file this caller moved is gone, which another writer's sweep of a file it judged abandoned can leave.
      return false;
    }
    throw new QueueFileError(`the file at ${from} cannot be put back at ${to}: ${String(error,)}`,);
  }
  await rm(
    from,
    { force: true, },
  );
  return true;
}

/**
 Takes one entry out of the queue, only while its file still holds the attempt this caller decided about.

 @param queueFolder - the queue's folder

 @param name - the entry's name

 @param queueId - the identity of the entry the attempt was claimed on

 @returns whether the entry was that attempt and is now out of the queue

 @throws QueueFileError when the entry's file is there and cannot be read or renamed

 @example
 ```ts
 const removed = await anEntryRemovedOnlyIfTheAttempt({ queueFolder: '/runs/q', name: 'a', queueId: 'q1' });
 ```
 */
export async function anEntryRemovedOnlyIfTheAttempt({
  queueFolder,
  name,
  queueId,
}: {
  readonly queueFolder: string;
  readonly name: string;
  readonly queueId: string;
},): Promise<boolean> {
  /**
   The entry's file.
   */
  const file = entryFileOf({
    queueFolder,
    name,
  },);
  /**
   Where the entry is renamed to while it is read.
   */
  const aside = aNameTakenAside({ path: file, },);
  try {
    await rename(
      file,
      aside,
    );
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw new QueueFileError(`the entry at ${file} cannot be taken aside: ${String(error,)}`,);
  }
  /**
   The entry the rename moved holds, or what kept it from being read for this step.
   */
  const read = await theEntryInAFile({ path: aside, },);
  if ('problem' in read) {
    await putBackUnlessTheNameIsTaken({
      from: aside,
      to: file,
    },);
    throw read.problem;
  }
  if ('none' in read) {
    // The file this caller renamed aside was gone when it was read, which cannot be this caller's entry.
    return false;
  }
  /**
   The entry as the file holds it.
   */
  const held = read.entry;
  if (queueIdOf({ record: held, },) !== queueId) {
    // The queue holds another entry of that name now, which is not this caller's to remove: the entry this caller moved is put back
    // unless the name was taken meanwhile, in which case it stays in the file beside and the answer says so.
    await putBackUnlessTheNameIsTaken({
      from: aside,
      to: file,
    },);
    return false;
  }
  await rm(
    aside,
    { force: true, },
  );
  return true;
}

/**
 Reads the identity one entry's file holds, without taking the entry out of the queue.

 @param queueFolder - the queue's folder

 @param name - the entry's name

 @returns the identity the queue holds under that name, or nothing when no entry of that name is queued

 @throws QueueFileError when the entry's file is there and cannot be read

 @example
 ```ts
 const queueId = await theIdentityQueued({ queueFolder: '/runs/q', name: 'a' });
 ```
 */
export async function theIdentityQueued({
  queueFolder,
  name,
}: {
  readonly queueFolder: string;
  readonly name: string;
},): Promise<string | typeof NO_ENTRY_OF_THAT_NAME_QUEUED> {
  /**
   The identity the entry's file carries, or that no entry of that name is queued.
   */
  return await anIdentityAt({
    path: entryFileOf({
      queueFolder,
      name,
    },),
    name,
  },);
}

/**
 No identity under that name, as a symbol so no identity can be mistaken for none: the queue holds no entry of that name.

 */
export const NO_ENTRY_OF_THAT_NAME_QUEUED: unique symbol = Symbol('the queue holds an entry of no such name',);

/**
 Reads one parsed file as an entry of a queue, or answers why it is none.

 @param parsed - what the file's text parsed to

 @param path - the file's path, for the message

 @returns the entry, or the problem

 @example
 ```ts
 const entry = anEntryOf({ parsed, path: '/runs/q/a.json' });
 ```
 */
function anEntryOf({
  parsed,
  path,
}: {
  readonly parsed: unknown;
  readonly path: string;
},): { readonly entry: QueueRecord } | { readonly problem: QueueFileError } {
  if (((typeof parsed) !== 'object') || (parsed === null)
    || (!('name' in parsed))) {
    return {
      problem: new QueueFileError(`${path} holds no JSON object naming an entry`,),
    };
  }
  return { entry: {
    ...parsed,
    name: String(parsed.name,),
  }, };
}

/**
 Reads the identity one entry's file carries, or that no entry of that name is queued.

 @param path - the entry's file

 @param name - the entry's name, for the message

 @returns the identity, or that no entry is queued under that name

 @throws QueueFileError when the file is there and cannot be read

 @example
 ```ts
 const queueId = await anIdentityAt({ path: '/runs/q/a.json', name: 'a' });
 ```
 */
async function anIdentityAt({
  path,
  name,
}: {
  readonly path: string;
  readonly name: string;
},): Promise<string | typeof NO_ENTRY_OF_THAT_NAME_QUEUED> {
  /**
   The entry the file holds, or what kept it from being read.
   */
  const read = await theEntryInAFile({
    path,
    missingIsNone: true,
  },);
  if ('problem' in read) {
    throw new QueueFileError(`the entry named ${JSON.stringify(name,)} cannot be read: ${String(read.problem,)}`,);
  }
  if ('none' in read) {
    return NO_ENTRY_OF_THAT_NAME_QUEUED;
  }
  /**
   The identity it carries, when it carries one.
   */
  const queueId = queueIdOf({ record: read.entry, },);
  return ((typeof queueId) === 'string') ? queueId : NO_ENTRY_OF_THAT_NAME_QUEUED;
}

/**
 Reads the entry one file holds, or answers why it could not be read, or that no file is there.

 @param path - the entry's file

 @param missingIsNone - whether a file that is not there is the answer this caller wants, rather than a problem

 @returns the entry, the problem, or that no file is there

 @example
 ```ts
 const read = await theEntryInAFile({ path: '/runs/q/a.json' });
 ```
 */
async function theEntryInAFile({
  path,
  missingIsNone = false,
}: {
  readonly path: string;
  readonly missingIsNone?: boolean;
},): Promise<
  | { readonly entry: QueueRecord }
    | { readonly problem: QueueFileError }
    | { readonly none: true }
  > {
  try {
    /**
     What the file holds.
     */
    const read = anEntryOf({
      parsed: JSON.parse(await readFile(
        path,
        'utf8',
      ),) as unknown,
      path,
    },);
    return ('problem' in read) ? read : read;
  }
  catch (error) {
    if (missingIsNone && (errorCodeOf(error,) === MISSING_PATH_CODE)) {
      return { none: true, };
    }
    return {
      problem: new QueueFileError(`the file at ${path} cannot be read: ${String(error,)}`,),
    };
  }
}

/**
 Reads a token that names one attempt, which a caller writes into a claim it takes itself.

 @returns a fresh token

 @example
 ```ts
 const token = aFreshToken();
 ```
 */
export function aFreshToken(): string {
  return randomUUID();
}
