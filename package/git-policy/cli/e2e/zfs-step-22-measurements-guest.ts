#!/usr/bin/env node
/**
 Drive plan step 22's guest measurements in one throwaway `mvm` guest.

 The arms build a file-backed pool, clone datasets and mounts as root, so no control on a person's machine may run them.
 This driver borrows the ZFS binding's guest recipe for the parts that are not the wrapper's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest and prepares it
 with OpenZFS 2.4.4 and the Rust toolchain, builds this crate's `native_binary` test target inside it, runs step 22's
 measurement arm there, pulls its log, and destroys the guest.

 The arms are the ones the plan's step 22 asks for
 (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, "Step 22"): the add against ordinary Git under the same
 workload with an interleaved control, the mount request's wait, the unmount's, the sweep's cost and the lost-event
 behaviour, what expiration and removal free, the forced arm's three waits, and the request budget.

 Usage: node e2e/zfs-step-22-measurements-guest.ts <create|prepare|run|destroy|all> [--name <guest>] [--log-dir <dir>]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 The evidence of one run is `step-22-measurements-arm.log`, pulled back into the log directory, and a run passes only
 when it carries the `STEP22` lines the arm performs and no `SKIP` line, so an arm that returned without running fails the
 run instead of reporting a green suite.

 The guest's own steps are in `e2e/step-22-arms/`: the build, the two pools, the measurement arm and the holder scan
 beside it.
 */
import { mkdir, } from 'node:fs/promises';
import { resolve, } from 'node:path';

import { runWithAGuestThatIsDestroyed, } from './guest-host/guest-run-lifecycle.ts';
import { recipe, } from './guest-host/host-side-commands.ts';
import type { GuestRun, } from './guest-host/guest-run.ts';
import {
  type RunArguments,
  runArguments,
} from './guest-host/guest-host-rules-fixture.ts';
import { buildInGuest, } from './step-22-arms/crate-build.ts';
import { measureTheHolderScan, } from './step-22-arms/holder-scan-arm.ts';
import {
  layThePools,
  measureStep22,
} from './step-22-arms/measurement-arm.ts';

/**
 The guest name a run uses when the command line names none.
 */
const defaultGuestName = 'zfs-step-22-measurements';

/**
 The log folder a run uses when the command line names none.
 */
const defaultLogFolder = 'target/step-22-guest-logs';

/**
 The mode the run's own log folder is made with: readable and writable by the
 account that runs the driver, and by nobody else.
 */
const logFolderMode = 0o700;

/**
 The step, the guest name and the log directory.
 */
type Options = {
  /**
   The step the command line names.
   */
  readonly step: string;
  /**
   The guest's `mvm` name.
   */
  readonly name: string;
  /**
   The folder this run writes its logs in.
   */
  readonly logDir: string;
};

/**
 The step and the values the command line gave it.

 @param argumentsRead - the words the command line gave, after the script's path

 @returns the step, the guest's name and the log folder

 @example
 ```ts
 options({ argumentsRead: runArguments({ argv: ['run', '--name', 'guest-one'] }) });
 ```
 */
function options({
  argumentsRead,
}: {
  readonly argumentsRead: RunArguments;
},): Options {
  /**
   The guest's name, which the command line may give.
   */
  const named = argumentsRead.values
    .get('--name',);
  /**
   The log folder, which the command line may give.
   */
  const folder = argumentsRead.values
    .get('--log-dir',);
  return {
    step: argumentsRead.step,
    name: named ?? defaultGuestName,
    logDir: folder
      ?? resolve(
        import.meta.dirname,
        '..',
        defaultLogFolder,
      ),
  };
}

/**
 Create the guest, refusing a name another run holds.

 @param chosen - the run's options

 @throws Error naming the recipe step and its status

 @example
 ```ts
 await create(chosen);
 ```
 */
async function create(chosen: GuestRun,): Promise<void> {
  await recipe({
    chosen,
    step: 'create',
  },);
}

/**
 Install OpenZFS 2.4.4 and the Rust toolchain in the guest.

 @param chosen - the run's options

 @throws Error naming the recipe step and its status

 @example
 ```ts
 await prepare(chosen);
 ```
 */
async function prepare(chosen: GuestRun,): Promise<void> {
  await recipe({
    chosen,
    step: 'prepare',
  },);
}

/**
 Push a repository-shaped snapshot of the wrapper, build its test target in the guest, lay the file-backed pools, and run
 the measurement arm and the holder scan beside it.

 @param chosen - the run's options

 @throws Error naming the step that refused, or the marker the pulled log lacks

 @example
 ```ts
 await run(chosen);
 ```
 */
async function run(chosen: GuestRun,): Promise<void> {
  await buildInGuest({ chosen, },);
  await layThePools({ chosen, },);
  await measureStep22({ chosen, },);
  await measureTheHolderScan({ chosen, },);
}

/**
 Destroy the guest and keep the list of what is left.

 @param chosen - the run's options

 @throws Error naming the recipe step and its status

 @example
 ```ts
 await destroy(chosen);
 ```
 */
async function destroy(chosen: GuestRun,): Promise<void> {
  await recipe({
    chosen,
    step: 'destroy',
  },);
}

/**
 Create, prepare, run and destroy, in that order, destroying even when a step
 refused.

 @param chosen - the run's options

 @throws Error naming the step that refused, after the guest it created is gone

 @example
 ```ts
 await all(chosen);
 ```
 */
async function all(chosen: GuestRun,): Promise<void> {
  /**
   The guest this run creates, which `create` refuses to make twice.
   */
  function createThisGuest(): Promise<void> {
    return create(chosen,);
  }

  /**
   Everything the guest was created for: its preparation and the measurement arm.
   */
  async function prepareAndMeasure(): Promise<void> {
    await prepare(chosen,);
    await run(chosen,);
  }

  /**
   The guest's destruction, which happens whatever the work did.
   */
  function destroyThisGuest(): Promise<void> {
    return destroy(chosen,);
  }

  await runWithAGuestThatIsDestroyed({
    name: chosen.name,
    subject: 'git-policy step 22 guest',
    keepGuest: false,
    create: createThisGuest,
    run: prepareAndMeasure,
    destroy: destroyThisGuest,
  },);
}

/**
 The steps this driver knows, and nothing else.
 */
const steps: ReadonlyMap<string, (chosen: GuestRun,) => Promise<void>> = new Map([
  [
    'create',
    create,
  ],
  [
    'prepare',
    prepare,
  ],
  [
    'run',
    run,
  ],
  [
    'destroy',
    destroy,
  ],
  [
    'all',
    all,
  ],
],);

/**
 Run the step the command line names.

 @param chosen - the run's options

 @throws Error naming a step this driver does not have, or what the step refused

 @example
 ```ts
 await runTheStep(chosen);
 ```
 */
async function runTheStep(chosen: Options,): Promise<void> {
  /**
   The step the command line named.
   */
  const step = steps.get(chosen.step,);
  if (step === undefined)
    throw new Error(`unknown step '${chosen.step}'; use create, prepare, run, destroy or all`);
  await mkdir(
    chosen.logDir,
    {
      recursive: true,
      mode: logFolderMode,
    },
  );
  await step(chosen,);
}

/**
 The run's own folder, which every step's logs land in.
 */
/**
 The words the command line gave, after the script's path.
 */
const words: readonly string[] = process.argv
  .slice(2);
/**
 The step and the values the command line gave it.
 */
const chosen: Options = options({ argumentsRead: runArguments({ argv: words, },), },);
await runTheStep(chosen,);
