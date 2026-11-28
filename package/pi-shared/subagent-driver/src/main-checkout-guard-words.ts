/**
 The words the driver and a run's Pi exchange about the guard: the one variable that says which folders this run may not change,
 and the answer a Pi gives a word this package did not write.

 It is the only channel between the driver and the guard's extension. It names no secret, so any command the run starts may read it,
 and a Pi whose extension is loaded by hand without it refuses to arm rather than guess at folders to protect.

 @module
 */

import type { MainCheckoutGuard, } from './main-checkout-guard-of-a-run.ts';
import { isRecord, } from './read-pi-fields.ts';

//region Constants

/**
 The variable the driver puts in its Pi's environment, holding the folders this run may not change or the word
 {@link MAIN_CHECKOUT_GUARD_OFF}.
 */
export const MAIN_CHECKOUT_GUARD_VARIABLE = 'MONOCHROMATIC_RUN_MAIN_CHECKOUT_GUARD';

/**
 The word that says this run has no main checkout to guard, which is what a run started in no worktree is handed.
 */
export const MAIN_CHECKOUT_GUARD_OFF = 'off';

/**
 The fields the word holds, named once so the writer and the reader cannot drift apart.
 */
const FOLDER_FIELDS = [
  'mainCheckout',
  'worktree',
  'worktreesFolder',
] as const;

//endregion Constants

//region The types

/**
 A guard description this extension could not read, which stops the run rather than leaving it unguarded.
 */
export class MainCheckoutGuardUnreadableError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'MainCheckoutGuardUnreadableError';
}

//endregion The types

//region Writing and reading the word

/**
 The word the driver puts in Pi's environment for this guard: the folders, or the word that says there are none.

 @param guard - the guard of this run

 @returns the text of {@link MAIN_CHECKOUT_GUARD_VARIABLE}

 @example
 ```ts
 const word = mainCheckoutGuardWordOf({ guard, });
 ```
 */
export function mainCheckoutGuardWordOf({ guard, }: { readonly guard: MainCheckoutGuard },): string {
  if (!guard.armed) {
    return MAIN_CHECKOUT_GUARD_OFF;
  }
  return JSON.stringify({
    mainCheckout: guard.mainCheckout,
    worktree: guard.worktree,
    worktreesFolder: guard.worktreesFolder,
  },);
}

/**
 Reads the folders this Pi may not change out of its own environment.

 @param environment - Pi's environment, which the driver wrote before the start

 @returns the guard, armed with the folders the driver named or off as the driver said

 @throws MainCheckoutGuardUnreadableError when the entry is absent or is not a guard this extension can read, which is a Pi handed
 the extension by something other than this driver

 @example
 ```ts
 const guard = mainCheckoutGuardOfPiEnvironment({ environment: process.env, });
 ```
 */
export function mainCheckoutGuardOfPiEnvironment({ environment, }: {
  readonly environment: Readonly<NodeJS.ProcessEnv>;
},): MainCheckoutGuard {
  /**
   What the driver wrote.
   */
  const word = environment[MAIN_CHECKOUT_GUARD_VARIABLE];
  if (word === undefined) {
    throw new MainCheckoutGuardUnreadableError(
      `this Pi loads the main-checkout guard, which only a run started by this package's driver is given, and ${MAIN_CHECKOUT_GUARD_VARIABLE} names nothing, so there are no folders to guard. Start the run with \`drive-pi-subagent\`, or load Pi without this extension.`,
    );
  }
  if (word === MAIN_CHECKOUT_GUARD_OFF) {
    return {
      armed: false,
      why: `the driver armed no guard: ${MAIN_CHECKOUT_GUARD_VARIABLE} is ${MAIN_CHECKOUT_GUARD_OFF}`,
    };
  }
  return guardOfItsWord({ word, },);
}

/**
 Reads the folders out of the word the driver wrote.

 @param word - what {@link MAIN_CHECKOUT_GUARD_VARIABLE} holds

 @returns the guard the word names

 @throws MainCheckoutGuardUnreadableError when the word is not the JSON this package writes, or names no folder
 */
function guardOfItsWord({ word, }: { readonly word: string },): MainCheckoutGuard {
  /**
   The three folders the driver writes, read one at a time, or why the word holds none of them.
   */
  const folders = foldersOfItsWord({ word, },);
  if ('unreadable' in folders) {
    throw folders.unreadable;
  }
  return {
    armed: true,
    ...folders,
  };
}

/**
 Reads the driver's word into three folders.

 @param word - what {@link MAIN_CHECKOUT_GUARD_VARIABLE} holds

 @returns the folders, or the refusal a word this package did not write gets
 */
function foldersOfItsWord({ word, }: { readonly word: string },):
  | {
    readonly mainCheckout: string;
    readonly worktree: string;
    readonly worktreesFolder: string;
  }
    | { readonly unreadable: MainCheckoutGuardUnreadableError } {
  /**
   What the word parsed to, of any shape.
   */
  const parsed = parsedWord({ word, },);
  if ('unreadable' in parsed) {
    return parsed;
  }
  /**
   What the word parsed to, as the record it must be.
   */
  const held = parsed.parsed;
  if (!isRecord(held,)) {
    return {
      unreadable: new MainCheckoutGuardUnreadableError(
        `${MAIN_CHECKOUT_GUARD_VARIABLE} holds no folders, so the folders this run may not change cannot be read: ${word}`,
      ),
    };
  }
  /**
   The folders the word names, each one empty when it is not text.
   */
  const named = FOLDER_FIELDS.map(function readOneField(field,) {
    /**
     The field as the word holds it.
     */
    const value = held[field];
    return ((typeof value) === 'string') ? value : '';
  },);
  /**
   The three folders, each one empty text when the word names none of them.
   */
  const [mainCheckout, worktree, worktreesFolder,] = named;
  if ((mainCheckout === undefined)
    || (worktree === undefined)
    || (worktreesFolder === undefined)
    || (mainCheckout === '')
    || (worktree === '')
    || (worktreesFolder === '')) {
    return {
      unreadable: new MainCheckoutGuardUnreadableError(
        `${MAIN_CHECKOUT_GUARD_VARIABLE} names no main checkout, worktree and worktrees folder, so there are no folders to guard: ${word}`,
      ),
    };
  }
  return {
    mainCheckout,
    worktree,
    worktreesFolder,
  };
}

/**
 Parses the word the driver wrote.

 @param word - what {@link MAIN_CHECKOUT_GUARD_VARIABLE} holds

 @returns what it parsed to, or the refusal a word that is no JSON gets
 */
function parsedWord({ word, }: { readonly word: string },):
  | { readonly parsed: unknown }
    | { readonly unreadable: MainCheckoutGuardUnreadableError } {
  try {
    return { parsed: JSON.parse(word,), };
  }
  catch (error) {
    return {
      unreadable: new MainCheckoutGuardUnreadableError(
        `${MAIN_CHECKOUT_GUARD_VARIABLE} is not the JSON this package's driver writes, so the folders it guards cannot be read: ${
          Error.isError(error,) ? error.name : 'a failure'
        }`,
      ),
    };
  }
}

//endregion Writing and reading the word
