/**
 Claims a review's folder for one start, so two branches of one change name never write over each other's review, and so a folder
 left by a start that was killed can be taken again.

 The claim is an owner file created by linking a written record into its place: a link makes the file appear with the record
 complete, or fails because another start made it, so no start can read a half-written record. Only one start holds the folder, and
 a second start says whose it is and whether that start still runs.

 The record names the review's unit, because a review outlives the command that writes its brief and starts it: the unit, not that
 command, is what keeps the folder. A folder whose command has ended but whose review unit still runs is kept, and a taker must
 have the manager's own answer that the unit is gone before it may take the folder; a manager that does not answer is unknown, and
 unknown keeps the folder. A record that names a unit which was never started, because the command ended between writing the record
 and starting the unit, has no review to keep the folder, so the folder is taken.

 Taking a dead start's place removes the exact record that was looked at: the taker takes a ticket named after that record's claim,
 reads the record again, and removes it only when it still carries that claim. A ticket whose holder has ended is moved aside by
 exactly one start, so a taker killed while taking over blocks nobody for good.

 @module
 */

import { randomUUID, } from 'node:crypto';
import {
  link,
  mkdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  claimKeyOf,
  takeTheTicket,
} from './a-takeover-ticket.ts';
import { errorCodeOf, } from './error-code-of.ts';
import { identityStillRuns, } from './liveness-of-a-process.ts';
import {
  type ReviewFolderOwner,
  ownerIn,
} from './the-owner-record-of-a-review-folder.ts';
import {
  type RunCommand,
  runCommandWithChildProcess,
} from './run-command.ts';
import { unitStateNow, } from './the-line-a-launcher-ends-with.ts';

/**
 The code the operating system gives a path that another start already made.
 */
const EXISTS_CODE = 'EEXIST';

/**
 The states a unit is in while the review it runs is still alive, which a takeover must wait out.
 */
const RUNNING_UNIT_STATES: ReadonlySet<string> = new Set<string>([
  'active',
  'activating',
  'reloading',
  'deactivating',
]);

/**
 The states a unit is in when the review it ran has ended, which are the states a takeover may take the folder in.
 */
const GONE_UNIT_STATES: ReadonlySet<string> = new Set<string>([
  'inactive',
  'failed',
  'not-found',
]);

/**
 Says whether the manager's answer about a unit is proof that its review has ended.

 Unknown, empty and every state this module does not know are not proof, so a folder is kept for them.

 @param state - the state the manager printed

 @returns whether the unit is gone

 @example
 ```ts
 const gone = unitStateIsGone('inactive',);
 ```
 */
function unitStateIsGone({ state, }: { readonly state: string },): boolean {
  return GONE_UNIT_STATES.has(state,);
}

/**
 Where a test holds a start that is taking a dead start's place, so two starts can be made to look at one dead record at the same
 moment. It is a seam: no command passes it.
 */
export type ReviewFolderClaimSeams = {
  /**
   Awaited after the start has read a dead owner's record and before it removes that record.
   */
  readonly afterLookingAtADeadOwner?: () => Promise<void>;
};

export type { ReviewFolderOwner, } from './the-owner-record-of-a-review-folder.ts';

/**
 A review folder another start already holds, which is refused so two branches of one change name never write over each other's
 review.
 */
export class ReviewFolderTakenError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ReviewFolderTakenError';
}

/**
 Whether two branches, each known, write one change name: the case a start refuses and names both for.

 @param holderBranch - the branch the folder's record names

 @param branch - the branch this start writes the review for

 @returns whether both are known and differ

 @example
 ```ts
 const differ = namesTwoBranches({ holderBranch: 'fix/a', branch: 'feat/a', },);
 ```
 */
function namesTwoBranches({
  holderBranch,
  branch,
}: {
  readonly holderBranch?: string;
  readonly branch?: string;
},): boolean {
  if ((holderBranch === undefined) || (branch === undefined)) {
    return false;
  }
  return holderBranch !== branch;
}

/**
 What making the owner file by linking it into place answered.
 */
type OwnerFileOutcome =
  | {
    /**
     This start made the owner file.
     */
    readonly kind: 'made';
  }
    | {
      /**
       Another start had already made the owner file.
       */
      readonly kind: 'another-start-has-it';
    }
    | {
      /**
       The link failed for a reason that is not a second owner.
       */
      readonly kind: 'the-link-failed';

      /**
       What the link gave.
       */
      readonly error: unknown;
    };

/**
 Links a written record into the owner file's place, which only one start can do.

 @param temporary - the file the record was written to

 @param ownerFile - the owner file's path

 @returns whether this start made the owner file, or the reason the link did not

 @example
 ```ts
 const outcome = await linkTheOwnerFile({ temporary, ownerFile },);
 ```
 */
async function linkTheOwnerFile({
  temporary,
  ownerFile,
}: {
  readonly temporary: string;
  readonly ownerFile: string;
},): Promise<OwnerFileOutcome> {
  try {
    await link(
      temporary,
      ownerFile,
    );
    return { kind: 'made', };
  }
  catch (error) {
    if (errorCodeOf(error,) === EXISTS_CODE) {
      return { kind: 'another-start-has-it', };
    }
    return {
      kind: 'the-link-failed',
      error,
    };
  }
}

/**
 Creates the owner file with the record in it, only when no owner file is there.

 The record is written whole to a file of its own and then linked into place: a link either makes the owner file appear with the
 record complete or fails because another start made it, so no start can read a half-written record.

 @param ownerFile - the file the record is kept in

 @param owner - the record to keep

 @returns whether this start made the file

 @example
 ```ts
 const taken = await takeTheOwnerFile({ ownerFile, owner },);
 ```
 */
async function takeTheOwnerFile({
  ownerFile,
  owner,
}: {
  readonly ownerFile: string;
  readonly owner: ReviewFolderOwner;
},): Promise<boolean> {
  /**
   A file of this start's own, beside the owner file, which no other start writes.
   */
  const temporary = `${ownerFile}.${String(process.pid,)}.${randomUUID()}.tmp`;
  /**
   The record as it is written.
   */
  const record = JSON.stringify(
    owner,
    undefined,
    2,
  );
  await writeFile(
    temporary,
    `${record}\n`,
    'utf8',
  );
  /**
   What linking the record into the owner file's place answered.
   */
  const outcome = await linkTheOwnerFile({
    temporary,
    ownerFile,
  },);
  await rm(
    temporary,
    { force: true, },
  );
  if (outcome.kind === 'made') {
    return true;
  }
  if (outcome.kind === 'another-start-has-it') {
    return false;
  }
  throw outcome.error;
}

/**
 Takes a review's folder for one start, or refuses when another start holds it.

 @param reviewFolder - the review's own folder

 @param ownerFile - the file the owner record is kept in, inside that folder

 @param branch - the branch whose change this start writes the review for, when it named one

 @param owner - the record to keep when this start takes the folder

 @param unit - the review unit this folder's review runs as, whose life the claim follows

 @param manager - how the user's manager is asked about a unit, which a test hands in

 @param seams - places a test holds a start at, which no command passes

 @throws ReviewFolderTakenError when another start holds the folder, naming both branches when both are known

 @example
 ```ts
 await takeReviewFolder({ reviewFolder, ownerFile, branch, owner, unit: 'pi-review-a.service', },);
 ```
 */
export async function takeReviewFolder({
  reviewFolder,
  ownerFile,
  branch,
  owner,
  unit,
  manager = runCommandWithChildProcess,
  seams,
}: {
  readonly reviewFolder: string;
  readonly ownerFile: string;
  readonly branch?: string;
  readonly owner: ReviewFolderOwner;
  readonly unit: string;
  readonly manager?: RunCommand;
  readonly seams?: ReviewFolderClaimSeams;
},): Promise<void> {
  await mkdir(
    reviewFolder,
    { recursive: true, },
  );
  /**
   What the manager says of the review this folder is for, before this start looks at any record.
   */
  const unitState = await unitStateNow({
    run: manager,
    unitName: unit,
  },);
  if (!unitStateIsGone({ state: unitState, },)) {
    throw takenFolderError({
      reviewFolder,
      ...((branch === undefined) ? {} : { branch, }),
      unit,
      unitState,
      holderRuns: false,
    },);
  }
  if (await takeTheOwnerFile({
    ownerFile,
    owner,
  },)) {
    return;
  }
  /**
   What the folder records of the start that holds it.
   */
  const held = await ownerIn({
    ownerFile,
    unit,
  },);
  /**
   The record itself, when the folder holds one.
   */
  const heldOwner = (held.kind === 'an-owner') ? held.owner : undefined;
  if (heldOwner === undefined) {
    // The owner file was there when this start tried to make it and cannot be read now: another start may be between removing a
    // dead record and making its own, and only a record this start can name is one it may remove.
    if (await takeTheOwnerFile({
      ownerFile,
      owner,
    },)) {
      return;
    }
    throw takenFolderError({
      reviewFolder,
      ...((branch === undefined) ? {} : { branch, }),
      holderRuns: false,
    },);
  }
  /**
   The branch that record names, when it names one.
   */
  const holderBranch = heldOwner.branch;
  /**
   Whether that start still runs.
   */
  const holderRuns = await identityStillRuns({ identity: heldOwner.process, },);
  if (holderRuns || namesTwoBranches({
    ...((holderBranch === undefined) ? {} : { holderBranch, }),
    ...((branch === undefined) ? {} : { branch, }),
  },)) {
    throw takenFolderError({
      reviewFolder,
      holder: heldOwner,
      ...((branch === undefined) ? {} : { branch, }),
      holderRuns,
    },);
  }
  /**
   The unit that record names, which is the life of the review it wrote the folder for.
   */
  const holderUnit = heldOwner.unit;
  /**
   What the manager says of that unit, which this start already has when the record names this folder's own unit.
   */
  const holderUnitState = (holderUnit === unit)
    ? unitState
    : await unitStateNow({
      run: manager,
      unitName: holderUnit,
    },);
  if (!unitStateIsGone({ state: holderUnitState, },)) {
    throw takenFolderError({
      reviewFolder,
      holder: heldOwner,
      ...((branch === undefined) ? {} : { branch, }),
      unit: holderUnit,
      unitState: holderUnitState,
      holderRuns: false,
    },);
  }
  // The command has ended and its review unit is gone: take its record away and try once more. The ticket makes the removal
  // exclusive for the record this start looked at, so two starts cannot both remove and both link.
  await seams?.afterLookingAtADeadOwner?.();
  await takeTheDeadOwnersPlace({
    reviewFolder,
    ownerFile,
    claim: heldOwner.claim,
    owner,
    ...((branch === undefined) ? {} : { branch, }),
    unit,
  },);
}

/**
 Takes a dead start's place in a review's folder, removing the exact record this start read and no other.

 @param reviewFolder - the review's own folder

 @param ownerFile - the file the owner record is kept in

 @param claim - the claim of the record this start read

 @param owner - the record to keep when this start takes the folder

 @param branch - the branch whose change this start writes the review for, when it named one

 @param unit - the review unit this folder's review runs as

 @throws ReviewFolderTakenError when another start took the place first

 @example
 ```ts
 await takeTheDeadOwnersPlace({ reviewFolder, ownerFile, claim: 'a-claim', owner, unit: 'pi-review-a.service', },);
 ```
 */
async function takeTheDeadOwnersPlace({
  reviewFolder,
  ownerFile,
  claim,
  owner,
  branch,
  unit,
}: {
  readonly reviewFolder: string;
  readonly ownerFile: string;
  readonly claim: string;
  readonly owner: ReviewFolderOwner;
  readonly branch?: string;
  readonly unit: string;
},): Promise<void> {
  /**
   The ticket that lets one start remove this claim's record.
   */
  const ticketPath = `${ownerFile}.taking-${claimKeyOf({ claim, },)}`;
  if (!(await takeTheTicket({
    ticketPath,
    identity: owner.process,
  },))) {
    throw takenFolderError({
      reviewFolder,
      ...((branch === undefined) ? {} : { branch, }),
      holderRuns: false,
    },);
  }
  /**
   The ticket, which this start removes when it leaves, however it leaves.
   */
  await using theTicket = {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        ticketPath,
        { force: true, },
      );
    },
  };
  /**
   What the folder holds now that this start holds the ticket.
   */
  const held = await ownerIn({
    ownerFile,
    unit,
  },);
  if ((held.kind !== 'an-owner') || (held.owner
    .claim
    !== claim)) {
    throw takenFolderError({
      reviewFolder,
      ...((held.kind === 'an-owner') ? { holder: held.owner, } : {}),
      ...((branch === undefined) ? {} : { branch, }),
      holderRuns: false,
    },);
  }
  await rm(
    ownerFile,
    { force: true, },
  );
  if (await takeTheOwnerFile({
    ownerFile,
    owner,
  },)) {
    return;
  }
  /**
   What the folder holds when another start made a record between the removal and this start's own.
   */
  const after = await ownerIn({
    ownerFile,
    unit,
  },);
  throw takenFolderError({
    reviewFolder,
    ...((after.kind === 'an-owner') ? { holder: after.owner, } : {}),
    ...((branch === undefined) ? {} : { branch, }),
    holderRuns: false,
  },);
}

/**
 Reads the refusal of a review folder another start holds, naming both branches when both are known.

 @param reviewFolder - the review's own folder

 @param holder - the record of the start that holds it, when one could be read

 @param branch - the branch this start writes the review for, when it named one

 @param holderRuns - whether the holder still runs

 @param unit - the review unit whose state kept the folder, when the manager was asked

 @param unitState - what the manager answered for that unit

 @returns the error to throw

 @example
 ```ts
 const error = takenFolderError({ reviewFolder: '/reviews/a', holder, branch: 'feat/a', holderRuns: true },);
 ```
 */
function takenFolderError({
  reviewFolder,
  holder,
  branch,
  holderRuns,
  unit,
  unitState,
}: {
  readonly reviewFolder: string;
  readonly holder?: ReviewFolderOwner;
  readonly branch?: string;
  readonly holderRuns: boolean;
  readonly unit?: string;
  readonly unitState?: string;
},): ReviewFolderTakenError {
  /**
   The branch the holder writes the review for, when it named one.
   */
  const holderBranch = holder?.branch;
  if (namesTwoBranches({
    ...((holderBranch === undefined) ? {} : { holderBranch, }),
    ...((branch === undefined) ? {} : { branch, }),
  },)) {
    return new ReviewFolderTakenError(
      `the review folder ${reviewFolder} is held for ${String(holderBranch,)}, and this start is for ${String(branch,)}: two branches of one change name cannot share one review folder; ask for them in separate batches, or remove that folder once its start has ended`,
    );
  }
  if (holderRuns) {
    return new ReviewFolderTakenError(
      `the review folder ${reviewFolder} is held by a start${(holderBranch === undefined) ? '' : ` of ${holderBranch}`} that is still running; start this branch again when that start has ended`,
    );
  }
  if ((unit !== undefined) && (unitState !== undefined)) {
    /**
     What the folder is, in words both a record and no record can share.
     */
    const what = (holder === undefined)
      ? `the review folder ${reviewFolder} holds no owner record that can be read`
      : `the review folder ${reviewFolder} is held by a start${(holderBranch === undefined) ? '' : ` of ${holderBranch}`} that has ended`;
    if (RUNNING_UNIT_STATES.has(unitState,)) {
      return new ReviewFolderTakenError(
        `${what}, but its review unit ${unit} is ${unitState}: a review that still runs keeps its folder, because a review outlives the command that wrote its brief; start this branch again when that review has ended`,
      );
    }
    return new ReviewFolderTakenError(
      `${what}, and the manager did not answer for its review unit ${unit} (${unitState}): this start cannot tell whether that review still runs, so the folder is kept; start this branch again when the manager answers`,
    );
  }
  return new ReviewFolderTakenError(
    `the review folder ${reviewFolder} is held by another start that could not be taken over; remove the folder and start the branch again`,
  );
}
