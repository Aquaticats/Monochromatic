/**
 What a run's Pi loads and registers for the guard: the entry the driver names with `--extension`, and the words the guard's command
 carries in Pi's own command list.

 @module
 */

import type { MainCheckoutGuard, } from './main-checkout-guard-of-a-run.ts';
import { siblingCommandEntry, } from './sibling-command-entry.ts';

//region Constants

/**
 The entry of this package that Pi loads as the guard's extension.
 */
export const MAIN_CHECKOUT_GUARD_ENTRY_NAME = 'main-checkout-guard-extension';

/**
 The command the extension registers, which says in Pi's command list which folders this run may not change.
 */
export const MAIN_CHECKOUT_GUARD_COMMAND = 'main-checkout-guard';

//endregion Constants

//region Pi's command line and command list

/**
 Builds the words that load the guard's extension in the Pi of one run.

 @param folder - the folder the driver's own entry is in, which the extension's entry sits beside

 @returns the words Pi is started with

 @example
 ```ts
 const words = await mainCheckoutGuardArguments({ folder: import.meta.dirname, });
 ```
 */
export async function mainCheckoutGuardArguments({ folder, }: { readonly folder: string },): Promise<readonly string[]> {
  return [
    '--extension',
    await siblingCommandEntry({
      folder,
      command: MAIN_CHECKOUT_GUARD_ENTRY_NAME,
    },),
  ];
}

/**
 The description the extension registers its command with, which says in Pi's command list what this run may not change.

 @param guard - the guard of this run

 @returns the description, which names the folders a reader would need

 @example
 ```ts
 const description = guardArmedDescription({ guard, });
 ```
 */
export function guardArmedDescription({ guard, }: { readonly guard: MainCheckoutGuard },): string {
  if (!guard.armed) {
    return `no main checkout is guarded for this run: ${guard.why}`;
  }
  return `refuses edit and write calls that name the main checkout ${guard.mainCheckout}; this run changes its worktree ${guard.worktree} instead`;
}

//endregion Pi's command line and command list
