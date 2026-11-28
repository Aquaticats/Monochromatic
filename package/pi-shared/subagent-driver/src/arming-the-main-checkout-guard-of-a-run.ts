/**
 Arming the guard of one run: the folders it holds, the word Pi reads them from, and the words its command line loads the extension
 with. Kept in a file of its own so the whole of a run's start says what it arms in one line.

 @module
 */

import type { PiRecord, } from './driver-types.ts';
import {
  type MainCheckoutGuard,
  mainCheckoutGuardOfARun,
} from './main-checkout-guard-of-a-run.ts';
import { mainCheckoutGuardArguments, } from './main-checkout-guard-in-pi.ts';
import {
  MAIN_CHECKOUT_GUARD_VARIABLE,
  mainCheckoutGuardWordOf,
} from './main-checkout-guard-words.ts';

//region Types

/**
 What one run's Pi is armed with, in the two shapes the driver hands it over.
 */
export type ArmingOfTheMainCheckoutGuardOfARun = {
  /**
   The folders this run may not change, or why it has none.
   */
  readonly guard: MainCheckoutGuard;

  /**
   The variables added to Pi's environment, which the guard's extension reads.
   */
  readonly environment: Readonly<Record<string, string>>;

  /**
   The words added to Pi's command line, which load the guard's extension.
   */
  readonly piArguments: readonly string[];
};

//endregion Types

//region Arming

/**
 Reads the folders of one run, notes them in the run's feed, and builds both handovers for it.

 The feed line says which folders the guard holds, or why it holds none, so a reader of a run that left an edit where it should not
 have can see whether the guard was armed at all.

 @param workingFolder - the folder this run's Pi works in, which is its worktree

 @param folder - the folder the driver's own entry is in, which the extension's entry sits beside

 @param note - where the run's feed says what the guard holds

 @returns the guard, the variables Pi reads it from, and the words that load it

 @example
 ```ts
 const arming = await armingOfTheMainCheckoutGuard({ workingFolder: '/repo/pi-a', folder: import.meta.dirname, note });
 ```
 */
export async function armingOfTheMainCheckoutGuard({
  workingFolder,
  folder,
  note,
}: {
  readonly workingFolder: string;
  readonly folder: string;
  readonly note: (entry: PiRecord,) => void;
},): Promise<ArmingOfTheMainCheckoutGuardOfARun> {
  /**
   The folders this run may not change, read from the worktree it works in.
   */
  const guard = await mainCheckoutGuardOfARun({ workingFolder, },);
  note({
    kind: 'main-checkout-guard',
    ...(guard.armed
      ? {
        mainCheckout: guard.mainCheckout,
        worktree: guard.worktree,
      }
      : { off: guard.why, }),
  },);
  return {
    guard,
    environment: { [MAIN_CHECKOUT_GUARD_VARIABLE]: mainCheckoutGuardWordOf({ guard, },), },
    piArguments: await mainCheckoutGuardArguments({ folder, },),
  };
}

//endregion Arming
