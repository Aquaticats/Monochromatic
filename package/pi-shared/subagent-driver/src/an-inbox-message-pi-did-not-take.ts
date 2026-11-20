/**
 What becomes of an inbox message Pi did not take: a refusal or an unanswered wait keeps it under a dot name, and an answer that
 refuses the moment, such as an agent already in a turn or a session compacting, puts it back in the inbox for a later look.
 Neither ever replaces another message of the same name.

 @module
 */

import {
  link,
  readdir,
  rename,
  stat,
  unlink,
} from 'node:fs/promises';
import { join, } from 'node:path';
import type { WatchRequest, } from './watch-request.ts';

/**
 The first characters of the dot name a claim is held under: no message is read from such a name, and no kept failure is published
 as one.
 */
export const CLAIM_NAME_START = '.claimed-';

/**
 The code the operating system gave a failure, when it gave one.

 @param error - what was caught

 @returns the code, or nothing when the failure carries none
 */
function codeOf(error: unknown,): string {
  if (((typeof error) !== 'object') || (error === null)
    || (!('code' in error))) {
    return '';
  }
  return ((typeof error.code) === 'string') ? error.code : '';
}

/**
 Publishes a message under one name, without ever replacing a file already there: a link fails when its name is taken, so the
 caller decides what a taken name means.

 @param claim - where the message now is

 @param target - the name this attempt would publish it under

 @returns what this attempt achieved, so the caller knows whether to try another name or leave the message where it is
 */
async function publishUnder({
  claim,
  target,
}: {
  readonly claim: string;
  readonly target: string
},): Promise<'published' | 'name-taken' | 'kept-by-rename'> {
  try {
    await link(
      claim,
      target,
    );
    await unlink(claim,);
    return 'published';
  }
  catch (error) {
    /**
     What the operating system said about this name.
     */
    const code = codeOf(error,);
    if (code === 'EEXIST') {
      return 'name-taken';
    }
    if ((code === 'EPERM') || (code === 'EACCES')) {
      // A folder cannot be linked, and holding one of Pi's messages is not at stake in it.
      await rename(
        claim,
        target,
      );
      return 'kept-by-rename';
    }
    throw error;
  }
}

/**
 Keeps a message that did not reach Pi, so it stays readable and is never sent again. Publication never replaces an earlier
 kept failure: a link fails when its name is taken, so the next number is tried instead.

 @param watch - watch request

 @param name - message's name

 @param claim - where the message now is

 @example
 ```ts
 await keepFailure({ watch, name: 'message.txt', claim: '/runs/a-1/inbox/.claimed-4f9a', },);
 ```
 */
export async function keepFailure({
  watch,
  name,
  claim,
}: {
  readonly watch: WatchRequest;
  readonly name: string;
  readonly claim: string
},): Promise<void> {
  /**
   The most numbered names to try before giving up: there are always fewer kept failures than names in the inbox.
   */
  const attempts = (await readdir(watch.inbox,)).length + 2;
  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    /**
     The name this attempt would keep the failure under.
     */
    const target = join(
      watch.inbox,
      (attempt === 1) ? `.failed-${name}` : `.failed-${name}.${String(attempt,)}`,
    );
    /**
     What this attempt achieved.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one name after another until one is free, and a kept failure must never replace another; see doc/lint/sequential-awaits.md
    const outcome = await publishUnder({
      claim,
      target,
    },);
    if ((outcome === 'published') || (outcome === 'kept-by-rename')) {
      return;
    }
  }
  throw new Error(`every name for a kept failure of ${name} is taken`,);
}

/**
 Puts a message back in the inbox for a later look, because Pi answered that its agent is busy rather than that it refused the
 message. Publication never replaces a file already there: a newer message of the same name keeps the name, and this one stays under
 the dot name of its claim, readable and never sent again.

 The claim is kept after a publication that worked, as the copy no producer's rename can take away: the next look reads the name,
 and a newer message that takes that name before then leaves the older one readable here instead of deleting it.

 @param watch - watch request

 @param name - message's name

 @param claim - where the message now is

 @example
 ```ts
 await returnToInbox({ watch, name: 'message.txt', claim: '/runs/a-1/inbox/.claimed-4f9a', },);
 ```
 */
export async function returnToInbox({
  watch,
  name,
  claim,
}: {
  readonly watch: WatchRequest;
  readonly name: string;
  readonly claim: string
},): Promise<void> {
  try {
    await link(
      claim,
      join(
        watch.inbox,
        name,
      ),
    );
  }
  catch (error) {
    watch.record
      .note({
        kind: 'problem',
        problem: `Pi could not take ${name} yet, which is no refusal of it: the message could not go back to its own name (${String(error,)}), so it stays under a dot name, readable and never sent again`,
      },);
  }
}

/**
 Tells whether a path holds the file another one holds.

 @param path - path to tell

 @param device - device of the file the claim holds

 @param number - number of that file on its device

 @returns whether the path is another link to that file
 */
async function holdsTheSameFile({
  path,
  device,
  number,
}: {
  readonly path: string;
  readonly device: number;
  readonly number: number
},): Promise<boolean> {
  try {
    /**
     What the path holds.
     */
    const stats = await stat(path,);
    return ((stats.dev === device) && (stats.ino === number));
  }
  catch (error) {
    // A claim another look removed between the listing and this read holds nothing to unlink.
    if (codeOf(error,) === 'ENOENT') {
      return false;
    }
    throw error;
  }
}

/**
 Unlinks the claims an earlier busy round left of the same message, so a Pi that answers busy on every look adds one entry per
 message rather than one per look. The claims of one message are hard links to the same file, so the file's device and number tell
 them from another message's claim, and every kept failure keeps its own name.

 @param inbox - inbox folder

 @param claim - the claim this delivery just made

 @example
 ```ts
 await dropEarlierClaimsOf({ inbox: '/runs/a-1/inbox', claim: '/runs/a-1/inbox/.claimed-4f9a', },);
 ```
 */
export async function dropEarlierClaimsOf({
  inbox,
  claim,
}: {
  readonly inbox: string;
  readonly claim: string
},): Promise<void> {
  /**
   The file this delivery's claim holds, which any earlier claim of the same message holds too.
   */
  const held = await stat(claim,);
  /**
   The earlier claims found, which are unlinked once the folder has been read.
   */
  const earlier: string[] = [];
  for (const entry of await readdir(inbox,)) {
    if (!entry.startsWith(CLAIM_NAME_START,)) {
      continue;
    }
    /**
     Where that entry is.
     */
    const path = join(
      inbox,
      entry,
    );
    if (path === claim) {
      continue;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- the folder is read once and each entry is told from this claim by the file it holds; see doc/lint/sequential-awaits.md
    if (await holdsTheSameFile({
      path,
      device: held.dev,
      number: held.ino,
    },)) {
      earlier.push(path,);
    }
  }
  await Promise.all(earlier.map(function unlinkOne(path,) {
    return unlink(path,);
  },),);
}
