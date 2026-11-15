/**
 Where a claim on an inbox message lives, whose it is, and how a claimed message is laid down.

 A driver claims a message by renaming it into a folder named for the driver: `<inbox>/.claims/<pid>-<start>`, where `<start>` is
 the clock ticks the driver began at, read from the process folder. The message keeps its own name inside that folder, so at every
 instant a person and a later driver read which message is where. A later driver reads the same process folder and tells a driver
 that is gone from one that is merely slow without waiting on it: a missing entry, or one that names a different start time for the
 same process number, is a driver that is gone, and an entry that cannot be read proves nothing either way.

 A message that is laid down is kept under a name that says why it was not taken: `.failed-<name>` when Pi refused it while it
 rested or never answered, and `.orphaned-<name>` when the driver that claimed it died before it knew whether Pi took it.

 @module
 */

import { randomUUID, } from 'node:crypto';
import { constants as fileConstants, } from 'node:fs';
import {
  copyFile,
  link as linkOnDisk,
  lstat,
  mkdir,
  readdir,
  readFile,
  rename,
  rmdir,
  unlink,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import process from 'node:process';
import { errorCodeOf, } from './error-code-of.ts';
import { isDigitsOnly, } from './text-of-an-option.ts';
import type { WatchRequest, } from './watch-request.ts';

/**
 The folder inside an inbox that holds the claims of every driver that has taken a message and not yet laid it down.
 */
export const INBOX_CLAIMS_FOLDER = '.claims';

/**
 The text a claim's owner folder carries in place of a start time when the driver could not read its own, which no later driver
 can verify.
 */
const START_UNKNOWN = 'unknown';

/**
 The field of a process's own record that holds the clock ticks it began at, counted from the end of the record's command name.
 */
const START_TIME_FIELD = 19;

/**
 What a claim folder's name is recognised by when it says nothing of the process that owns it.
 */
const NOT_AN_OWNER: unique symbol = Symbol('claim folder names no process');

/**
 The process folder holds no entry for the process a claim names, so its driver is gone.
 */
const PROCESS_IS_MISSING: unique symbol = Symbol('process folder has no entry');

/**
 The process folder has an entry the driver may not read, so whether that driver is gone cannot be told from it.
 */
const START_UNREADABLE: unique symbol = Symbol('its record cannot be read');

/**
 The absence of a kept name: the message could not be kept aside at all.
 */
export const NO_KEPT_NAME: unique symbol = Symbol('message could not be kept');

/**
 The absence of a claim owner: this driver cannot read the clock ticks its own process began at, so it cannot name a folder a
 later driver could prove it owns.
 */
export const NO_CLAIM_OWNER: unique symbol = Symbol('own process start cannot be read');

/**
 A driver that cannot read the start time of its own process, so it cannot claim a message under a name a later driver could
 check. The message stays in the inbox, under its own name, rather than being claimed unverifiably.
 */
export class ClaimOwnerUnreadableError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ClaimOwnerUnreadableError';
}

/**
 The absence of an entry: nothing is at the path that was read.
 */
const NO_ENTRY: unique symbol = Symbol('nothing is at the path');

/**
 What a process folder says of when one process began.
 */
type ProcessStart = string | typeof PROCESS_IS_MISSING | typeof START_UNREADABLE;

/**
 A claim's owner as its folder names it.
 */
type ClaimOwner = {
  /**
   The process number of the driver that claimed the message.
   */
  readonly pid: string;

  /**
   The clock ticks that driver began at, or {@link START_UNKNOWN}.
   */
  readonly start: string;
};

/**
 A message the driver holds: it was claimed out of the inbox and Pi has not taken it.
 */
export type HeldMessage = {
  /**
   The name the message was published under, which is the name the summary and the feed carry.
   */
  readonly name: string;

  /**
   Where the claim moved it, inside the claiming driver's folder under the message's own name.
   */
  readonly claim: string;
};

/**
 Reads when one process began, from the process folder's own record of it.

 @param processFolder - the folder holding one entry per process, `/proc` here

 @param entry - the process's entry in that folder, or `self`

 @returns the clock ticks it began at, or which of the two ways that cannot be read

 @example
 ```ts
 const start = await startTimeOf({ processFolder: '/proc', entry: 'self', },);
 ```
 */
async function startTimeOf({
  processFolder,
  entry,
}: {
  readonly processFolder: string;
  readonly entry: string
},): Promise<ProcessStart> {
  try {
    /**
     The process's own record, whose command name may hold spaces or a closing bracket, so the fields after it are read from the
     last closing bracket.
     */
    const stat = await readFile(
      join(
        processFolder,
        entry,
        'stat',
      ),
      'utf8',
    );
    /**
     Where the record's command name ends.
     */
    const close = stat.lastIndexOf(')',);
    if (close === (-1)) {
      return START_UNREADABLE;
    }
    /**
     The fields after the command name, the first of which is the process's state.
     */
    const after = stat.slice(close + 1,)
      .trim()
      .split(' ',);
    /**
     The clock ticks the process began at.
     */
    const start = after[START_TIME_FIELD];
    return ((start !== undefined) && isDigitsOnly(start,)) ? start : START_UNREADABLE;
  }
  catch (error) {
    return (errorCodeOf(error,) === 'ENOENT') ? PROCESS_IS_MISSING : START_UNREADABLE;
  }
}

/**
 Reads a claim folder's name as the process that owns it.

 @param name - the folder's name, `<pid>-<start>`

 @returns the owner, or {@link NOT_AN_OWNER} when the name says nothing of one

 @example
 ```ts
 const owner = ownerIn('4242-123456',);
 ```
 */
function ownerIn(name: string,): ClaimOwner | typeof NOT_AN_OWNER {
  /**
   Where the process number ends and the start time begins.
   */
  const at = name.indexOf('-',);
  if (at <= 0) {
    return NOT_AN_OWNER;
  }
  /**
   The process number the folder names.
   */
  const pid = name.slice(
    0,
    at,
  );
  /**
   The start time the folder names, or the word that says it could not be read.
   */
  const start = name.slice(at + 1,);
  if ((!isDigitsOnly(pid,)) || ((start !== START_UNKNOWN) && (!isDigitsOnly(start,)))) {
    return NOT_AN_OWNER;
  }
  return {
    pid,
    start,
  };
}

/**
 Tells whether the driver that claimed a message is gone, rather than merely slow, from the process folder alone.

 A claim is left alone while its owner's process entry names the same start time the claim does. An entry that cannot be read, or
 that says nothing usable, is not proof of anything, so its claim is left alone too. An entry with a different start time is a
 process number the operating system has given to something else, and a missing entry is a driver that is gone.

 @param processFolder - the folder holding one entry per process

 @param ownerName - the claim folder's name, `<pid>-<start>`

 @returns whether the claim's driver is gone

 @example
 ```ts
 const gone = await claimOwnerIsGone({ processFolder: '/proc', ownerName: '4242-123456', },);
 ```
 */
export async function claimOwnerIsGone({
  processFolder,
  ownerName,
}: {
  readonly processFolder: string;
  readonly ownerName: string
},): Promise<boolean> {
  /**
   The driver the claim folder names.
   */
  const owner = ownerIn(ownerName,);
  if ((typeof owner) === 'symbol') {
    // A folder whose name says nothing of a process is not a claim this code made, and nothing here proves its owner gone.
    return false;
  }
  /**
   What the process folder says of that process now.
   */
  const start = await startTimeOf({
    processFolder,
    entry: owner.pid,
  },);
  if (owner.start === START_UNKNOWN) {
    // A claim whose own start could not be read is proven gone only by its process number being gone: while the number names a
    // process, that process may be the owner, and nothing else here proves anything.
    return start === PROCESS_IS_MISSING;
  }
  if (start === PROCESS_IS_MISSING) {
    return true;
  }
  if (start === START_UNREADABLE) {
    return false;
  }
  return start !== owner.start;
}

/**
 The process folder a run's claims are read against.

 @param watch - watch request

 @returns the folder, `/proc` when the run names none

 @example
 ```ts
 const folder = processFolderOf(watch,);
 ```
 */
export function processFolderOf(watch: WatchRequest,): string {
  return watch.processFolder ?? '/proc';
}

/**
 Takes a folder away once it is empty, so an inbox that was used does not keep an empty folder for ever.

 @param folder - the folder to take away

 @example
 ```ts
 await removeFolderIfEmpty('/runs/a-1/inbox/.claims/4242-123456',);
 ```
 */
export async function removeFolderIfEmpty(folder: string,): Promise<void> {
  try {
    await rmdir(folder,);
  }
  catch (error) {
    /**
     What the operating system said about the folder.
     */
    const code = errorCodeOf(error,);
    // Something else of the same driver is still there, or the folder has already gone.
    if ((code !== 'ENOTEMPTY') && (code !== 'ENOENT')) {
      throw error;
    }
  }
}

/**
 Takes a claim's owner folder away once it holds no claim, and the folder that holds the owners with it when that too is empty, so
 an inbox that has handed its messages over reads as it did before any claim.

 @param ownerFolder - the owner's folder

 @example
 ```ts
 await removeClaimOwnerFolder('/runs/a-1/inbox/.claims/4242-123456',);
 ```
 */
export async function removeClaimOwnerFolder(ownerFolder: string,): Promise<void> {
  await removeFolderIfEmpty(ownerFolder,);
  await removeFolderIfEmpty(dirname(ownerFolder,),);
}

/**
 Takes a claim's folder away, and its owner's folder and the claims folder with it, once each is empty.

 @param claim - where a message was

 @example
 ```ts
 await removeClaimFolderOf('/runs/a-1/inbox/.claims/4242-123456/9a1b/m.txt',);
 ```
 */
export async function removeClaimFolderOf(claim: string,): Promise<void> {
  /**
   The folder one claim lives in.
   */
  const claimFolder = dirname(claim,);
  await removeFolderIfEmpty(claimFolder,);
  await removeClaimOwnerFolder(dirname(claimFolder,),);
}

/**
 Takes a claim's owner folder away once it holds no claim.

 @param claim - where a message was

 @example
 ```ts
 await removeOwnerFolderOf('/runs/a-1/inbox/.claims/4242-123456/9a1b/m.txt',);
 ```
 */
export async function removeOwnerFolderOf(claim: string,): Promise<void> {
  await removeClaimFolderOf(claim,);
}

/**
 Reads what a path is, or answers the sentinel that says nothing is there.

 @param path - the path to read

 @returns what is there, or {@link NO_ENTRY}

 @example
 ```ts
 const entry = await entryOrNothing('/runs/a-1/inbox',);
 ```
 */
export async function entryOrNothing(path: string,): Promise<Awaited<ReturnType<typeof lstat>> | typeof NO_ENTRY> {
  try {
    return await lstat(path,);
  }
  catch (error) {
    if (errorCodeOf(error,) === 'ENOENT') {
      return NO_ENTRY;
    }
    throw error;
  }
}

/**
 The last part of a path, which is the name a message was kept under.

 @param path - the path

 @returns its last part

 @example
 ```ts
 const name = lastPartOf('/runs/a-1/inbox/.failed-m.txt',);
 ```
 */
export function lastPartOf(path: string,): string {
  return path.slice(path.lastIndexOf('/',) + 1,);
}

/**
 The name of the folder a driver's claims live in, which is the driver's process number and the clock ticks it began at.

 @param processFolder - the folder holding one entry per process

 @returns the folder's name

 @example
 ```ts
 const name = await ownerFolderNameOf('/proc',);
 ```
 */
async function ownerFolderNameOf(processFolder: string,): Promise<string | typeof NO_CLAIM_OWNER> {
  /**
   When this driver began, which tells its own claim from one a killed driver left.
   */
  const start = await startTimeOf({
    processFolder,
    entry: 'self',
  },);
  if ((typeof start) !== 'string') {
    return NO_CLAIM_OWNER;
  }
  return `${String(process.pid,)}-${start}`;
}

/**
 Claims one inbox entry by renaming it into the claiming driver's folder, so that a producer publishing a newer message under the
 same name cannot be deleted or kept in its place, and a later driver reads the owner and the message's own name from the claim.

 @param watch - watch request

 @param name - message's name

 @returns where the message now is

 @example
 ```ts
 const claim = await claimMessage({ watch, name: 'm', },);
 ```
 */
export async function claimMessage({
  watch,
  name,
}: {
  readonly watch: WatchRequest;
  readonly name: string
},): Promise<string> {
  /**
   The folder this driver's claims live in, named for the driver so a later one can tell whether it is still there.
   */
  const processFolder = processFolderOf(watch,);
  /**
   The name of the owner folder, or the sentinel that says this driver cannot name one a later driver could verify.
   */
  const ownerName = await ownerFolderNameOf(processFolder,);
  if ((typeof ownerName) === 'symbol') {
    throw new ClaimOwnerUnreadableError(
      `this driver cannot read the start time of its own process from ${processFolder}, so it cannot claim a message under a name a later driver could prove it owns`,
    );
  }
  /**
   The folder this driver's claims live in.
   */
  const ownerFolder = join(
    watch.inbox,
    INBOX_CLAIMS_FOLDER,
    ownerName,
  );
  await mkdir(
    ownerFolder,
    { recursive: true, },
  );
  /**
   The claim's own folder, named by an identifier nothing else can hold, so this claim can never land on another one: a second
   message published under the same name is claimed into a folder of its own, and a claim left by an earlier failed keep stays.
   */
  const claimFolder = join(
    ownerFolder,
    randomUUID(),
  );
  await mkdir(claimFolder,);
  /**
   Where the claim goes, under the message's own name so a person and a later driver both read what it was.
   */
  const claimed = join(
    claimFolder,
    name,
  );
  try {
    await rename(
      join(
        watch.inbox,
        name,
      ),
      claimed,
    );
  }
  catch (error) {
    // A claim that did not happen leaves no empty folder behind.
    await removeFolderIfEmpty(claimFolder,);
    throw error;
  }
  return claimed;
}

/**
 Publishes a kept message under one name, without ever replacing a file already there: a link fails when its name is taken, so the
 caller tries the next number.

 @param claim - where the message now is

 @param target - the name this attempt would keep it under

 @param link - the hard link a filesystem gives, which a test replaces to make one fail

 @returns what this attempt achieved, so the caller knows whether to try another name
 */
async function publishKept({
  claim,
  target,
  link,
}: {
  readonly claim: string;
  readonly target: string;
  readonly link: (
    existing: string,
    created: string,
  ) => Promise<void>
},): Promise<'published' | 'name-taken'> {
  try {
    await link(
      claim,
      target,
    );
    await unlink(claim,);
    await removeOwnerFolderOf(claim,);
    return 'published';
  }
  catch (error) {
    /**
     What the operating system said about this name.
     */
    const code = errorCodeOf(error,);
    if (code === 'EEXIST') {
      return 'name-taken';
    }
    if ((code === 'EPERM') || (code === 'EACCES')) {
      /**
       What the claim holds, which a copy reproduces only when it is an ordinary file.
       */
      const info = await entryOrNothing(claim,);
      if (((typeof info) === 'symbol') || (!info.isFile())) {
        // A folder is not a file a copy can hold; the message stays under its claim, where its own name reads it, and is named.
        throw error;
      }
      // A filesystem without hard links still keeps the no-replacement promise: a copy that creates its name exclusively fails
      // when something already stands there, exactly as the link does.
      try {
        await copyFile(
          claim,
          target,
          fileConstants.COPYFILE_EXCL,
        );
      }
      catch (copyError) {
        if (errorCodeOf(copyError,) === 'EEXIST') {
          return 'name-taken';
        }
        throw copyError;
      }
      await unlink(claim,);
      await removeClaimFolderOf(claim,);
      return 'published';
    }
    throw error;
  }
}

/**
 Keeps a message that did not reach Pi, so it stays readable and is never sent again. Publication never replaces an earlier kept
 message: a link fails when its name is taken, so the next number is tried instead.

 @param watch - watch request

 @param name - message's name

 @param claim - where the message now is

 @param keptAs - the word the kept name carries, which says why the message was not taken

 @param link - the hard link a filesystem gives, which a test replaces to make one fail

 @returns the name the message was kept under

 @example
 ```ts
 const kept = await keepMessage({ watch, name: 'm', claim, keptAs: 'failed', },);
 ```
 */
export async function keepMessage({
  watch,
  name,
  claim,
  keptAs,
  link = linkOnDisk,
}: {
  readonly watch: WatchRequest;
  readonly name: string;
  readonly claim: string;
  readonly keptAs: 'failed' | 'orphaned';
  readonly link?: (
    existing: string,
    created: string,
  ) => Promise<void>
},): Promise<string> {
  /**
   The most numbered names to try before giving up: there are always fewer kept messages than names in the inbox.
   */
  const attempts = (await readdir(watch.inbox,)).length + 2;
  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    /**
     The name this attempt would keep the message under.
     */
    const target = join(
      watch.inbox,
      (attempt === 1) ? `.${keptAs}-${name}` : `.${keptAs}-${name}.${String(attempt,)}`,
    );
    /**
     What this attempt achieved.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one name after another until one is free, and a kept message must never replace another; see doc/lint/sequential-awaits.md
    const outcome = await publishKept({
      claim,
      target,
      link,
    },);
    if (outcome === 'published') {
      return target;
    }
  }
  throw new Error(`every name for a kept message of ${name} is taken`,);
}

/**
 Keeps one message, or answers the sentinel when it cannot be kept at all.

 @param watch - watch request

 @param held - the message and where its claim is

 @param keptAs - the word the kept name carries

 @returns where the message was kept, or {@link NO_KEPT_NAME}
 */
async function keepOrNothing({
  watch,
  held,
  keptAs,
}: {
  readonly watch: WatchRequest;
  readonly held: HeldMessage;
  readonly keptAs: 'failed' | 'orphaned'
},): Promise<string | typeof NO_KEPT_NAME> {
  try {
    return await keepMessage({
      watch,
      name: held.name,
      claim: held.claim,
      keptAs,
    },);
  }
  catch (error) {
    // A claim that is already gone was kept or delivered by another driver, and nothing here is left to keep.
    if (errorCodeOf(error,) !== 'ENOENT') {
      watch.record
        .note({
          kind: 'problem',
          problem: `could not keep ${held.name}: ${String(error,)}`,
        },);
    }
    return NO_KEPT_NAME;
  }
}

/**
 Keeps a message Pi never took, and names it in the summary. A message whose file cannot be kept is still named: the summary is
 what a reader of the run has, and a message that is neither with Pi nor under its name must not read as delivered.

 @param watch - watch request

 @param held - the message and where its claim is

 @param keptAs - the word the kept name carries, which says why the message was not taken

 @returns where the message was kept, or {@link NO_KEPT_NAME}

 @example
 ```ts
 await noteMessageNotTaken({ watch, held, keptAs: 'failed', },);
 ```
 */
export async function noteMessageNotTaken({
  watch,
  held,
  keptAs,
}: {
  readonly watch: WatchRequest;
  readonly held: HeldMessage;
  readonly keptAs: 'failed' | 'orphaned'
},): Promise<string | typeof NO_KEPT_NAME> {
  /**
   Where the message was kept, or the sentinel that says it could not be kept at all.
   */
  const kept = await keepOrNothing({
    watch,
    held,
    keptAs,
  },);
  watch.record
    .noteMessageNotTaken(held.name,);
  return kept;
}
