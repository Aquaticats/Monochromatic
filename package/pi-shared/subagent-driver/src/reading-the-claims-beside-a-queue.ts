/**
 The claims a queue folder holds: one file per entry a filling has taken, read as the attempt's token, its process number and its time,
 beside the identity of the entry the claim was taken on.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import {
  type AClaimOnAnEntry,
  CLAIMED_FOLDER,
  claimedFolderOf,
  QueueFileError,
  type ANameInAFolder,
} from './queue-of-runs.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The ending every claim's file carries.
 */
const ENTRY_FILE_ENDING = '.json';

/**
 Reads what one claim's file holds as a claim, or refuses it by name.

 @param parsed - what the file's text parsed to

 @param path - the file's path, for the message

 @returns the claim

 @throws QueueFileError when it holds no name, identity, token, process number and time

 @example
 ```ts
 const claim = aClaimIn({ parsed, path: '/runs/q/claimed/a.claim.json' });
 ```
 */
export function aClaimIn({
  parsed,
  path,
}: {
  readonly parsed: unknown;
  readonly path: string;
},): AClaimOnAnEntry {
  if (((typeof parsed) !== 'object') || (parsed === null)) {
    throw new QueueFileError(`${path} holds no claim, so which filling started that entry is unknown`,);
  }
  /**
   The fields the claim should hold.
   */
  const name: unknown = Reflect.get(
    parsed,
    'name',
  );
  /**
   The identity of the entry the claim was taken on.
   */
  const queueId: unknown = Reflect.get(
    parsed,
    'queueId',
  );
  /**
   The attempt's own facts, as the file holds them.
   */
  const claim: unknown = Reflect.get(
    parsed,
    'claim',
  );
  if (
    ((typeof name) !== 'string')
      || (name === '')
      || ((typeof queueId) !== 'string')
      || (queueId === '')
      || ((typeof claim) !== 'object')
      || (claim === null)
  ) {
    throw new QueueFileError(`${path} holds no entry's name and identity with the claim on it, so which filling started that entry is unknown`,);
  }
  /**
   The token naming the attempt.
   */
  const token: unknown = Reflect.get(
    claim,
    'token',
  );
  /**
   The filling's process number.
   */
  const pid: unknown = Reflect.get(
    claim,
    'pid',
  );
  /**
   When the claim was taken.
   */
  const at: unknown = Reflect.get(
    claim,
    'at',
  );
  if (
    ((typeof token) !== 'string')
      || (token === '')
      || ((typeof pid) !== 'number')
      || ((typeof at) !== 'string')
  ) {
    throw new QueueFileError(`${path} names no token, process number and time for the attempt, so which filling started that entry is unknown`,);
  }
  return {
    name,
    queueId,
    claim: {
      token,
      pid,
      at,
    },
  };
}

/**
 Reads every claim a queue folder holds: the entries fillings are starting now, or were starting when their filling was killed.

 @param queueFolder - the queue's folder

 @returns one claim per entry a filling has taken, in the order the folder lists them

 @throws QueueFileError when the claims folder or one of its files cannot be read, or holds no claim

 @example
 ```ts
 const claims = await readTheClaimsOnAQueue({ queueFolder: '/runs/q' });
 ```
 */
export async function readTheClaimsOnAQueue({
  queueFolder,
}: {
  readonly queueFolder: string;
},): Promise<readonly AClaimOnAnEntry[]> {
  /**
   The folder the claims live in.
   */
  const claimedFolder = claimedFolderOf({ queueFolder, },);
  /**
   What that folder holds.
   */
  let held: readonly ANameInAFolder[] = [];
  try {
    held = await readdir(
      claimedFolder,
      { withFileTypes: true, },
    );
  }
  catch (error) {
    // A queue nothing has claimed an entry of holds no claims folder, which is a fact about the queue and not a failure to read it.
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return [];
    }
    throw new QueueFileError(`the claims beside the queue at ${queueFolder} cannot be read: ${String(error,)}`,);
  }
  /**
   The claims read so far.
   */
  const claims: AClaimOnAnEntry[] = [];
  for (const file of held) {
    if ((!file.isFile())
      || (!file.name
        .endsWith(ENTRY_FILE_ENDING,))
      || file.name
        .startsWith('.',)) {
      if ((!file.isFile()) && (!file.name
        .startsWith('.',))) {
        throw new QueueFileError(
          `${join(
            claimedFolder,
            file.name,
          )} is neither a claim nor a name a killed command left, so the claims beside the queue at ${queueFolder} cannot be read`,
        );
      }
      continue;
    }
    /**
     The claim's path, for the message.
     */
    const path = join(
      claimedFolder,
      file.name,
    );
    /**
     What the claim's file holds, as anything.
     */
    let parsed: unknown = undefined;
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- the file of the claim just read, one after another
      parsed = JSON.parse(await readFile(
        path,
        'utf8',
      ),);
    }
    catch (error) {
      throw new QueueFileError(`the claim at ${path} cannot be read as one: ${String(error,)}`,);
    }
    claims.push(aClaimIn({
      parsed,
      path,
    },),);
  }
  return claims;
}
