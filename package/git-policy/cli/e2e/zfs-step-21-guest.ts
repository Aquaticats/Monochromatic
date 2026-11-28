#!/usr/bin/env node
/**
 Drive plan step 21's guest arms in one throwaway `mvm` guest.

 The arms add worktrees on a file-backed pool and mount clones of it, so no control on a person's machine may run them.
 This driver borrows the ZFS binding's guest recipe for the parts that are not the wrapper's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest and prepares it
 with OpenZFS 2.4.4 and the Rust toolchain, installs the Node tooling the pnpm arm starts, builds this crate's
 `native_binary` test target inside it, layers the pool, runs the arms there, pulls their log, and destroys the guest.

 The arms are the ones the plan's step 21 asks for
 (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, "Step 21"): pnpm's offline frozen-lockfile install after an
 add beside the same install on the checkout's own dataset, the Cargo clean with its lock recovered across file systems,
 the mise launcher repair, the watch rebinding after a clone's fill, and the queued repair beside the inline one.

 Usage: node e2e/zfs-step-21-guest.ts <create|prepare|tools|build|run|destroy|all> [--name <guest>] [--log-dir <dir>]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 The evidence of one run is `step-21-arms.log`, pulled back into the log directory, and a run passes only when it carries
 every `STEP21` line the driver expects and no `SKIP` line, so an arm that returned without running fails the run instead
 of reporting a green suite.

 The guest's own steps are in `e2e/step-21-arms/`: the tool install, the build, the pool, and the arms.
 */
import { mkdir, } from 'node:fs/promises';
import { resolve, } from 'node:path';

import type {
  GuestFacts,
} from '../bin/mvm-guest-guest-facts.ts';
import {
  recipe,
} from '../bin/mvm-guest-host-command.ts';
import {
  runArguments,
  type RunArguments,
} from '../bin/mvm-guest-options.ts';
import {
  runWholeGuestRun,
} from '../bin/mvm-guest-whole-run.ts';




import {
  buildInGuest,
} from './step-21-arms/crate-build.ts';
import {
  installTheTools,
} from './step-21-arms/guest-tools.ts';
import {
  layThePool,
  runTheArms,
} from './step-21-arms/arms.ts';

/**
 The guest name a run uses when the command line names none.
 */
const defaultGuestName = 'zfs-r13-step-21-arms';

/**
 The log folder a run uses when the command line names none.
 */
const defaultLogFolder = 'target/step-21-guest-logs';

/**
 The mode the run's own log folder is made with: readable and writable by the account that runs the driver, and by
 nobody else.
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
async function create(chosen: GuestFacts,): Promise<void> {
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
async function prepare(chosen: GuestFacts,): Promise<void> {
  await recipe({
    chosen,
    step: 'prepare',
  },);
}

/**
 Install the Node tooling the pnpm arm starts.

 @param chosen - the run's options

 @throws Error naming the step that refused

 @example
 ```ts
 await tools(chosen);
 ```
 */
async function tools(chosen: GuestFacts,): Promise<void> {
  await installTheTools({ chosen, },);
}

/**
 Push this checkout into the guest and build its test target there.

 @param chosen - the run's options

 @throws Error naming the step that refused

 @example
 ```ts
 await build(chosen);
 ```
 */
async function build(chosen: GuestFacts,): Promise<void> {
  await buildInGuest({ chosen, },);
}

/**
 Lay the pool and run the arms.

 @param chosen - the run's options

 @throws Error naming the step that refused, or the marker the pulled log lacks

 @example
 ```ts
 await run(chosen);
 ```
 */
async function run(chosen: GuestFacts,): Promise<void> {
  await layThePool({ chosen, },);
  await runTheArms({ chosen, },);
}

/**
 Destroy the guest.

 @param chosen - the run's options

 @throws Error naming the recipe step and its status

 @example
 ```ts
 await destroy(chosen);
 ```
 */
async function destroy(chosen: GuestFacts,): Promise<void> {
  await recipe({
    chosen,
    step: 'destroy',
  },);
}

/**
 Create, prepare, install the tools, build, run and destroy, in that order, destroying even when a step refused.

 @param chosen - the run's options

 @throws Error naming the step that refused, after the guest it created is gone

 @example
 ```ts
 await all(chosen);
 ```
 */
async function all(chosen: GuestFacts,): Promise<void> {
  await runWholeGuestRun({
    label: 'git-policy step 21 guest',
    chosen,
    keepGuest: false,
    steps: {
      create: createTheGuest,
      prepare: prepareTheGuest,
      work: theToolsTheBuildAndTheArms,
      destroy: destroyTheGuest,
    },
  },);
}

/**
 Make the guest this run acts on.

 @param chosen - the run's guest and log folder

 @throws GuestDriverError when the recipe's creation refused

 @example
 ```ts
 await createTheGuest({ chosen });
 ```
 */
async function createTheGuest({ chosen }: { readonly chosen: GuestFacts },): Promise<void> {
  await create(chosen,);
}

/**
 Install OpenZFS 2.4.4, the Rust toolchain and `cargo-nextest` in the guest.

 @param chosen - the run's guest and log folder

 @throws GuestDriverError when the recipe's preparation refused

 @example
 ```ts
 await prepareTheGuest({ chosen });
 ```
 */
async function prepareTheGuest({ chosen }: { readonly chosen: GuestFacts },): Promise<void> {
  await prepare(chosen,);
}

/**
 Install the Node tooling, build the test target, lay the pool and run the arms.

 @param chosen - the run's guest and log folder

 @throws GuestDriverError naming the step that refused

 @example
 ```ts
 await theToolsTheBuildAndTheArms({ chosen });
 ```
 */
async function theToolsTheBuildAndTheArms({ chosen }: { readonly chosen: GuestFacts },): Promise<void> {
  await tools(chosen,);
  await build(chosen,);
  await run(chosen,);
}

/**
 Take the guest away.

 @param chosen - the run's guest and log folder

 @throws GuestDriverError when the recipe's destruction refused

 @example
 ```ts
 await destroyTheGuest({ chosen });
 ```
 */
async function destroyTheGuest({ chosen }: { readonly chosen: GuestFacts },): Promise<void> {
  await destroy(chosen,);
}

/**
 The steps this driver knows, and nothing else.
 */
const steps: ReadonlyMap<string, (chosen: GuestFacts,) => Promise<void>> = new Map([
  [
    'create',
    create,
  ],
  [
    'prepare',
    prepare,
  ],
  [
    'tools',
    tools,
  ],
  [
    'build',
    build,
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
    throw new Error(`unknown step '${chosen.step}'; use create, prepare, tools, build, run, destroy or all`,);
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
 The words the command line gave, after the script's path.
 */
const words: readonly string[] = process.argv
  .slice(2);
/**
 The step and the values the command line gave it.
 */
const chosen: Options = options({ argumentsRead: runArguments({ argv: words, },), },);
await runTheStep(chosen,);
