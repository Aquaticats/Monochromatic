/**
 One step of the ZFS binding's own guest recipe, which owns a guest's creation, preparation and destruction.

 The recipe is `package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`, and it is the only thing this package
 uses that knows how to make an Ubuntu 24.04 guest, install OpenZFS 2.4.4 in it, and take it away again. A driver names
 the step and hands it the guest's name, log directory and any further words the step takes; the recipe's own output
 stays on the driver's terminal, because its steps print as they work and take minutes each.

 The path is resolved from this file's own folder, so a step never depends on the directory the run started in, and
 `mvm-guest-recipe-step.unit.test.ts` holds that answer against the file it names. Two of the three sets of guest helpers
 carried this answer (`bin/guest-route-commands.ts` and `bin/mvm-guest-recipe-step.ts`), and the third
 (`e2e/guest-host/host-side-commands.ts`) resolved the same file four folders up instead of three.
 */
import {
  resolve,
} from 'node:path';

import { GuestDriverError, } from './mvm-guest-error.ts';
import type { GuestFacts, } from './mvm-guest-guest-facts.ts';
import { runVisibleStepCommand, } from './mvm-guest-step-command.ts';
import { RECIPE_STEP_TIMEOUT_MS, } from './mvm-guest-bounds.ts';

/**
 The recipe's driver, found from this file so a step never depends on the directory the run started in.

 `bin`, the package, this crate's parent and then `package`: the ZFS binding's package sits beside this one.
 */
const RECIPE_DRIVER = resolve(
  import.meta.dirname,
  '..',
  '..',
  '..',
  'zfs',
  'request-to-openzfs-kernel-device',
  'e2e',
  'run-fixture.ts',
);

/**
 The guest recipe's driver, which owns every guest's creation, preparation and destruction.

 @returns the recipe's own file, which a run starts with `node`

 @example
 ```ts
 await readFile(guestRecipePath());
 ```
 */
export function guestRecipePath(): string {
  return RECIPE_DRIVER;
}

/**
 One step of the recipe, run with the guest's own name and log folder.

 @param name - the guest's name, which the recipe hands back to every later step

 @param logDirectory - the directory the recipe keeps one log per command in

 @param step - the recipe's own step, such as `create`, `prepare` or `destroy`

 @param extraArgumentWords - further words the recipe's step takes, such as the custom template `prepare` reads

 @throws GuestDriverError when the recipe's step exited nonzero

 @example
 ```ts
 await runGuestRecipeStep({ name: 'git-policy-lock', logDirectory: '/logs', step: 'destroy' });
 ```
 */
export async function runGuestRecipeStep({
  name,
  logDirectory,
  step,
  extraArgumentWords = [],
}: {
  readonly name: string;
  readonly logDirectory: string;
  readonly step: string;
  readonly extraArgumentWords?: readonly string[];
},): Promise<void> {
  /**
   The status the recipe's driver exited with.
   */
  const status = await runVisibleStepCommand({
    program: 'node',
    argumentWords: [
      RECIPE_DRIVER,
      step,
      '--name',
      name,
      '--log-dir',
      logDirectory,
      ...extraArgumentWords,
    ],
    timeoutMilliseconds: RECIPE_STEP_TIMEOUT_MS,
  },);
  if (status !== 0)
    throw new GuestDriverError(`the guest recipe's ${step} step exited ${String(status,)}`,);
}

/**
 One step of the recipe, as a step of a guest driver's own command line.

 A driver's `all` step is a table of four steps, and three of every driver's entries are the recipe's. This makes one of
 those entries without repeating the words the recipe needs.

 @param step - the recipe's own step

 @param extraArgumentWords - further words the recipe's step takes, such as the custom template `prepare` reads

 @returns a step that hands the recipe the guest's name, its log directory and those extra words

 @example
 ```ts
 const steps = { create: recipeStepOf({ step: 'create' }) };
 ```
 */
export function recipeStepOf({
  step,
  extraArgumentWords = [],
}: {
  readonly step: string;
  readonly extraArgumentWords?: readonly string[];
},): (run: { readonly chosen: GuestFacts; }) => Promise<void> {
  return async function recipeStep({ chosen, }) {
    await runGuestRecipeStep({
      name: chosen.name,
      logDirectory: chosen.logDir,
      step,
      extraArgumentWords,
    },);
  };
}
