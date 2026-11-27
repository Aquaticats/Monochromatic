/**
 One step of the ZFS binding's own guest recipe, which owns a guest's creation, preparation and destruction.

 The recipe is `package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`, and it is the only thing this package
 uses that knows how to make an Ubuntu 24.04 guest, install OpenZFS 2.4.4 in it, and take it away again. A driver names
 the step and hands it the guest's name and log directory; the recipe's own output stays on the driver's terminal,
 because its steps print as they work and take minutes each.
 */
import {
  resolve,
} from 'node:path';

import { GuestDriverError, } from './guest-driver-error.ts';
import { runVisibleStepCommand, } from './guest-driver-step-command.ts';

/**
 The step's own bound: an hour of the host-command bound a driver gives its smaller commands, because the recipe builds
 the guest's kernel module and toolchain inside the step it is asked for.
 */
const RECIPE_STEP_TIMEOUT_MS = 7_200_000;

/**
 The recipe's driver, found from this file so a step never depends on the directory the run started in.
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
