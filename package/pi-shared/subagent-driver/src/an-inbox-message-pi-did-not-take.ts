/**
 What becomes of an inbox message Pi did not take: a refusal or an unanswered wait keeps it under a dot name, and Pi's answer that
 its agent is busy puts it back in the inbox for a later look. Neither ever replaces another message of the same name.

 @module
 */

import {
  link,
  readdir,
  rename,
  unlink,
} from 'node:fs/promises';
import { join, } from 'node:path';
import type { WatchRequest, } from './watch-request.ts';

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
        problem: `Pi answered that its agent is busy, which is no refusal of ${name}: the message could not go back to its own name (${String(error,)}), so it stays under a dot name, readable and never sent again`,
      },);
  }
}
