#!/usr/bin/env node
/**
 Drive one end-to-end run of the root-run clone-mount handler's adoption mode, in a throwaway `mvm` guest.

 The arms write the store folder `/var/lib/monochromatic-git-policy-cli/git-worktree-clone-slot-identities` and open
 a checkout as root, so no control on a person's machine may run them. This driver borrows the ZFS binding's guest
 recipe for the parts that are not the wrapper's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the Ubuntu 24.04 guest and prepares it
 with OpenZFS 2.4.4 and the Rust toolchain, then builds this crate's `native_binary` test target inside it, runs the
 adoption and store controls there, runs the store's publication-fsync probe, and destroys the guest.

 Usage: node e2e/mount-git-worktree-clone-datasets-guest.ts <create|prepare|run|sweep|delegation|snapdir|destroy|all>
          [--name <guest>] [--log-dir <dir>] [--keep-guest]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 `delegation` answers the owner's question of 2026-11-22 by a run: under the
 design's own grants, as `ubuntu`, it tries every route that could give a
 dataset of the checkout's leaf name a row in the host's mount table, and then
 runs root's `zfs mount -a` and reads the table.

 The evidence of one run is `adoption-arm.log` and `store-fsync.log`, pulled back into the log directory, and a run
 passes only when the first carries every `ADOPTION` and `STORE` line below, the second carries both `PROBE` lines, and
 neither holds a `SKIPPED` line, so an arm that returned without running fails the run instead of reporting a green
 suite.

 The arms themselves are in `e2e/adoption-guest-arms/`, one file per arm.
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
  guestBackend,
  refuseAStepWhoseArmsCannotRun,
  STEPS_THAT_RUN_NO_ARM,
} from '../bin/mvm-guest-backend.ts';
import { runTheAdoption, } from './adoption-guest-arms/adoption-arm.ts';
import { buildInGuest, } from './adoption-guest-arms/crate-build.ts';
import { delegation, } from './adoption-guest-arms/delegation-arm.ts';
import { holderScanArms, } from './adoption-guest-arms/holder-scan-arm.ts';
import { snapdirMatrix, } from './adoption-guest-arms/snapdir-arm.ts';
import {
  publicationFsync,
  storeDataset,
} from './adoption-guest-arms/store-dataset-and-fsync.ts';
import { sweepArms, } from './adoption-guest-arms/sweep-arm.ts';
import { unmountArms, } from './adoption-guest-arms/unmount-arm.ts';

/**
 The guest name a run uses when the command line names none.
 */
const defaultGuestName = 'git-policy-cli-adoption';

/**
 The log folder a run uses when the command line names none, relative to the
 package's own folder.
 */
const defaultLogFolder = 'target/adoption-guest-logs';

/**
 The mode the run's own log folder is made with: readable and writable by the
 account that runs the driver, and by nobody else.
 */
const logFolderMode = 0o700;

/**
 What one end-to-end run names: the step, the guest, the log folder, whether `all`
 keeps the guest, and the `mvm` template the guest is made from.
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
  /**
   Whether `all` keeps the guest for a person to inspect.
   */
  readonly keepGuest: boolean;
  /**
   The `mvm` custom template the guest is made from, or absent for the Ubuntu
   24.04 image the recipe prepares.
   */
  readonly image?: string;
  /**
   The kernel release that template must boot, named with `--image` when the
   template does not say.
   */
  readonly kernelRelease?: string;
};

/**
 The step and the values the command line gave it.

 @param argumentsRead - the words the command line gave, after the script's path

 @returns the step, the guest's name, the log folder and the guest's template

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
  /**
   The `mvm` template, which the recipe takes only when the command line gave it.
   */
  const image = argumentsRead.values
    .get('--image',);
  /**
   The kernel release the template boots.
   */
  const kernelRelease = argumentsRead.values
    .get('--kernel-release',);
  return {
    step: argumentsRead.step,
    name: named ?? defaultGuestName,
    logDir: folder
      ?? resolve(
        import.meta.dirname,
        '..',
        defaultLogFolder,
      ),
    keepGuest: argumentsRead.flags
      .has('--keep-guest',),
    ...(image === undefined ? {} : { image, }),
    ...(kernelRelease === undefined ? {} : { kernelRelease, }),
  };
}

/**
 The recipe options a run's own template adds.

 @param chosen - the run's options

 @returns the recipe's own `--image` and `--kernel-release` words, in its order

 @example
 ```ts
 imageArguments({ chosen });
 ```
 */
function imageArguments(chosen: Options,): readonly string[] {
  /**
   The template the guest is made from, absent when the recipe's own image is
   the one this run wants, and the kernel release that template boots.
   */
  const {
    image,
    kernelRelease,
  } = chosen;
  if (image === undefined) {
    if (kernelRelease === undefined) return [];
    return [
      '--kernel-release',
      kernelRelease,
    ];
  }
  if (kernelRelease === undefined) {
    return [
      '--image',
      image,
    ];
  }
  return [
    '--image',
    image,
    '--kernel-release',
    kernelRelease,
  ];
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
async function create(chosen: GuestFacts & Options,): Promise<void> {
  await recipe({
    chosen,
    step: 'create',
    extraArgumentWords: imageArguments(chosen,),
  },);
}

/**
 Prepare the guest with OpenZFS 2.4.4 and the Rust toolchain.

 @param chosen - the run's options

 @throws Error naming the recipe step and its status

 @example
 ```ts
 await prepare(chosen);
 ```
 */
async function prepare(chosen: GuestFacts & Options,): Promise<void> {
  await recipe({
    chosen,
    step: 'prepare',
    extraArgumentWords: imageArguments(chosen,),
  },);
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
async function destroy(chosen: GuestFacts & Options,): Promise<void> {
  await recipe({
    chosen,
    step: 'destroy',
    extraArgumentWords: imageArguments(chosen,),
  },);
}

/**
 Build the test target inside the guest and run every arm.

 @param chosen - the run's options

 @throws Error naming the step that refused, or the marker a pulled log lacks

 @example
 ```ts
 await run(chosen);
 ```
 */
async function run(chosen: GuestFacts,): Promise<void> {
  await buildInGuest({ chosen, },);
  await storeDataset({ chosen, },);
  await runTheAdoption({ chosen, },);
  await publicationFsync({ chosen, },);
  await sweepArms({ chosen, },);
  await holderScanArms({ chosen, },);
  await unmountArms({ chosen, },);
}

/**
 Run plan step 8's sweep arms alone, for a round whose arm is one of them: the
 same push, build and store dataset the whole run makes, and none of the arms that
 follow the sweep.

 @param chosen - the run's options

 @throws Error naming the step that refused, or the marker the pulled log lacks

 @example
 ```ts
 await sweepOnly(chosen);
 ```
 */
async function sweepOnly(chosen: GuestFacts,): Promise<void> {
  await buildInGuest({ chosen, },);
  await storeDataset({ chosen, },);
  await sweepArms({ chosen, },);
}

/**
 Create, prepare, run every arm and destroy, destroying even when a step refused.

 @param chosen - the run's options

 @throws Error naming the step that refused, after the guest it created is gone

 @example
 ```ts
 await all(chosen);
 ```
 */
async function all(chosen: GuestFacts & Options,): Promise<void> {
  await runWholeGuestRun({
    label: 'git-policy adoption guest',
    chosen,
    keepGuest: chosen.keepGuest,
    steps: {
      create: async function createTheGuest({ chosen: facts }: { readonly chosen: GuestFacts }) {
        await recipe({
          chosen: facts,
          step: 'create',
          extraArgumentWords: imageArguments(chosen,),
        },);
      },
      prepare: async function prepareTheGuest({ chosen: facts }: { readonly chosen: GuestFacts }) {
        await recipe({
          chosen: facts,
          step: 'prepare',
          extraArgumentWords: imageArguments(chosen,),
        },);
      },
      work: runEveryArm,
      destroy: async function destroyTheGuest({ chosen: facts }: { readonly chosen: GuestFacts }) {
        await recipe({
          chosen: facts,
          step: 'destroy',
          extraArgumentWords: imageArguments(chosen,),
        },);
      },
    },
  },);
}

/**
 Build the test target in the guest and run every arm of this driver.

 @param chosen - the run's guest and log folder

 @throws GuestDriverError naming the step that refused, or the marker a pulled log lacks

 @example
 ```ts
 await runEveryArm({ chosen });
 ```
 */
async function runEveryArm({ chosen }: { readonly chosen: GuestFacts },): Promise<void> {
  await run(chosen,);
}

/**
 The steps this driver knows, and nothing else.
 */
const steps: ReadonlyMap<string, (chosen: GuestFacts & Options,) => Promise<void>> = new Map([
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
    'sweep',
    sweepOnly,
  ],
  [
    'delegation',
    async function delegationOnly(chosen: GuestFacts,): Promise<void> {
      await delegation({ chosen, },);
    },
  ],
  [
    'snapdir',
    async function snapdirOnly(chosen: GuestFacts,): Promise<void> {
      await snapdirMatrix({ chosen, },);
    },
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
async function runTheStep(chosen: GuestFacts & Options,): Promise<void> {
  /**
   The step the command line named.
   */
  const step = steps.get(chosen.step,);
  if (step === undefined)
    throw new Error(`unknown step '${chosen.step}'; use ${[...steps.keys(),].join(', ',)}`);
  await mkdir(
    chosen.logDir,
    {
      recursive: true,
      mode: logFolderMode,
    },
  );
  await step(chosen,);
  console.log(`git-policy adoption guest: step '${chosen.step}' finished; logs in ${chosen.logDir}`);
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
// The steps that run an arm in the guest refuse on a backend this package's arm scripts cannot reach, before this run
// makes anything. The three that only make, prepare and take away a guest run on either backend.
refuseAStepWhoseArmsCannotRun({
  backend: guestBackend(),
  step: chosen.step,
  stepsThatRunNoArm: STEPS_THAT_RUN_NO_ARM,
});
await runTheStep(chosen,);
