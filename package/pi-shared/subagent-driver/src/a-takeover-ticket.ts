/**
 The ticket one start takes before it removes a dead start's record from a review's folder.

 Two starts can read the same dead record and both decide to take that start's place. Removing the record, then making a new one,
 lets the slower start remove the record the faster start has just made. The ticket is the lock-free step that decides which start
 may remove the record it read: a file whose name is the record's claim, made only once, so exactly one start holds it; the holder
 then reads the record again and removes it only if it still carries that claim. A start that finds the ticket held by a live start
 is refused, and a ticket whose holder has ended is moved aside by exactly one start, because a rename of a path that is gone fails.

 @module
 */

import {
  createHash,
  randomUUID,
} from 'node:crypto';
import {
  link,
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';
import { errorCodeOf, } from './error-code-of.ts';
import { readFileOrNothing, } from './finished-worktrees.ts';
import {
  type ProcessIdentity,
  identityStillRuns,
} from './liveness-of-a-process.ts';
import { isRecord, } from './read-pi-fields.ts';

/**
 The code the operating system gives a path that another start already made.
 */
const EXISTS_CODE = 'EEXIST';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 Reads the part of a ticket's path that names the claim a takeover removes.

 The claim is a record's own name, which may hold any text a JSON string holds; a digest keeps the ticket's path one component and
 free of separators.

 @param claim - the claim name the record carries

 @returns the hex digest that names the claim's ticket

 @example
 ```ts
 const key = claimKeyOf({ claim: 'a-claim', },);
 ```
 */
export function claimKeyOf({ claim, }: { readonly claim: string },): string {
  return createHash('sha256',)
    .update(
    claim,
    'utf8',
    )
    .digest('hex',);
}

/**
 Takes the ticket that lets one start remove one claim's record, or says that another start holds it.

 @param ticketPath - the ticket's path, which names the claim

 @param identity - the start that would hold the ticket

 @returns whether this start now holds the ticket

 @example
 ```ts
 const held = await takeTheTicket({ ticketPath, identity },);
 ```
 */
export async function takeTheTicket({
  ticketPath,
  identity,
}: {
  readonly ticketPath: string;
  readonly identity: ProcessIdentity;
},): Promise<boolean> {
  if (await makeTheTicket({
    ticketPath,
    identity,
  },)) {
    return true;
  }
  /**
   What the ticket that is there says of its holder.
   */
  const kept = ticketStartIn({ shown: (await readFileOrNothing(ticketPath)).trim(), },);
  if ((kept.kind === 'a-start') && (await identityStillRuns({ identity: kept.identity, },))) {
    return false;
  }
  /**
   The name this start moves the ended ticket to, which only one start can use because the rename fails once the ticket is gone.
   */
  const moved = `${ticketPath}.${randomUUID()}.stale`;
  try {
    await rename(
      ticketPath,
      moved,
    );
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return false;
    }
    throw error;
  }
  /**
   Whether this start then made the ticket, which a third start may have made first.
   */
  const taken = await makeTheTicket({
    ticketPath,
    identity,
  },);
  await rm(
    moved,
    { force: true, },
  );
  return taken;
}

/**
 Makes the ticket when no ticket is there.

 @param ticketPath - the ticket's path

 @param identity - the start that would hold it

 @returns whether this start made it

 @example
 ```ts
 const made = await makeTheTicket({ ticketPath, identity },);
 ```
 */
async function makeTheTicket({
  ticketPath,
  identity,
}: {
  readonly ticketPath: string;
  readonly identity: ProcessIdentity;
},): Promise<boolean> {
  /**
   A file of this start's own, beside the ticket, which no other start writes.
   */
  const temporary = `${ticketPath}.${String(process.pid,)}.${randomUUID()}.tmp`;
  await writeFile(
    temporary,
    `${JSON.stringify(
      { process: identity, },
      undefined,
      2,
    )}\n`,
    'utf8',
  );
  /**
   Whether this start's file was linked into the ticket's place.
   */
  const made = await linked({
    from: temporary,
    to: ticketPath,
  },);
  await rm(
    temporary,
    { force: true, },
  );
  return made;
}

/**
 Links a written file into place, which fails when another start already made that path.

 @param from - this start's own file

 @param to - the path only one start can make

 @returns whether this start made the path

 @example
 ```ts
 const made = await linked({ from: temporary, to: ticketPath },);
 ```
 */
async function linked({
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
    return true;
  }
  catch (error) {
    if (errorCodeOf(error,) === EXISTS_CODE) {
      return false;
    }
    throw error;
  }
}

/**
 The start a ticket says holds it, when the ticket can be read as one.
 */
type AStartInATicket =
  | {
    /**
     The start the ticket names.
     */
    readonly kind: 'a-start';

    /**
     The start's identity.
     */
    readonly identity: ProcessIdentity;
  }
    | {
      /**
       The ticket names no start this package can read.
       */
      readonly kind: 'no-start';
    };

/**
 The answer for a ticket that cannot be read as naming a start.
 */
const NO_START: AStartInATicket = { kind: 'no-start', };

/**
 Reads the start a ticket says holds it.

 @param shown - the ticket's text

 @returns that start, or no start when the ticket cannot be read as one

 @example
 ```ts
 const kept = ticketStartIn({ shown: '{"process":{"pid":1,"bootId":"b","startTicks":"1"}}', },);
 ```
 */
function ticketStartIn({ shown, }: { readonly shown: string },): AStartInATicket {
  if (shown === '') {
    return NO_START;
  }
  /**
   The ticket, of any shape.
   */
  const kept: unknown = JSON.parse(shown,);
  if (!isRecord(kept,)) {
    return NO_START;
  }
  /**
   What the ticket says of the start that holds it.
   */
  const { process, } = kept;
  if (!isRecord(process,)) {
    return NO_START;
  }
  if ((typeof process.pid) !== 'number') {
    return NO_START;
  }
  if ((typeof process.bootId) !== 'string') {
    return NO_START;
  }
  if ((typeof process.startTicks) !== 'string') {
    return NO_START;
  }
  return {
    kind: 'a-start',
    identity: {
      pid: process.pid,
      bootId: process.bootId,
      startTicks: process.startTicks,
    },
  };
}
