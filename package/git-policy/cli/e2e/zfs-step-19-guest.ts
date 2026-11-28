#!/usr/bin/env node
/**
 Drive plan step 19's guest arms in one throwaway `mvm` guest: the boot units and their drop-ins parsed by a real manager,
 the immutable covered entry's verdict against a real immutable folder, `snapdir=disabled` set and inherited, the
 plan's boot arms across the guest's own reboots, and the runbook's migration and rollback on a layout the guest can throw
 away.

 The arms write `/etc/systemd/system`, `/usr/libexec` and a store under `/var/lib`, and they reboot the machine they run
 in, so they run in a guest and nowhere else: the guest-name check is in the launcher every detached script carries, and
 every arm reads `CLI_GIT_ZFS_STEP19_GUEST` before it does anything.

 The guest's own recipe is the ZFS binding's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts` through `bin/mvm-guest-host-command.ts`): it creates the
 Ubuntu 24.04 guest and installs OpenZFS 2.4.4 and the Rust toolchain, and it owns the destruction too.

 Usage: node e2e/zfs-step-19-guest.ts <create|prepare|layout|build|units|refusals|underlay|snapdir|boot|migration|remove|destroy|all>
          [--name <guest>] [--log-dir <dir>] [--keep-guest] [--image <template>] [--kernel-release <release>]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 The evidence of one run is one log per step in the log folder, and a run passes only when the arm's own log carries every
 line the driver expects: an arm that returned without running prints `SKIPPED` and exits zero, which is why a green exit
 status alone proves nothing (`AGENTS.md` rule `TRV`).

 @example
 ```sh
 node e2e/zfs-step-19-guest.ts all --name zfs-r12-step19-a --log-dir ~/temp/agent/run/keep/logs
 ```
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
  theImmutableEntry,
  theRefusals,
  theSnapdirInheritance,
  theUnitsReadBack,
} from './step-19-guest-arms/arms.ts';
import {
  installThePlan,
  makeTheClone,
  readTheBoot,
  rebootTheGuest,
  removeThePlan,
  theFailingBoots,
} from './step-19-guest-arms/boot.ts';
import { buildInGuest, } from './step-19-guest-arms/build.ts';
import {
  releaseBinary,
} from './step-19-guest-arms/fixture.ts';
import {
  guestLayout,
  installTheProgram,
  recordTheCheckout,
} from './step-19-guest-arms/layout.ts';
import {
  migrateAndRollBack,
} from './step-19-guest-arms/migration.ts';

/**
 The guest name a run uses when the command line names none.
 */
const defaultGuestName = 'zfs-step19-arms';

/**
 The log folder a run uses when the command line names none, relative to the package's own folder.
 */
const defaultLogFolder = 'target/step19-guest-logs';

/**
 The mode the run's own log folder is made with: readable and writable by the account that runs the driver, and by nobody
 else.
 */
const logFolderMode = 0o700;

/**
 What one run names: the step, the guest, the log folder, whether `all` keeps the guest, and the `mvm` template.
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
   The `mvm` custom template the guest is made from, or absent for the Ubuntu 24.04 image the recipe prepares.
   */
  readonly image?: string;
  /**
   The kernel release that template boots, named with `--image` when the template does not say.
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
   The template the guest is made from and the kernel release it boots, either of which may be absent.
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
 Run one step of the guest recipe.

 @param chosen - the run's options

 @param step - the recipe's own step

 @throws Error naming the recipe step and its status

 @example
 ```ts
 await recipeStep(chosen, 'create');
 ```
 */
async function recipeStep({
  chosen,
  step,
}: {
  readonly chosen: GuestFacts & Options;
  readonly step: string;
},): Promise<void> {
  await recipe({
    chosen,
    step,
    extraArgumentWords: imageArguments(chosen,),
  },);
}

/**
 Build the crate in the guest, then install the release program and record the checkout, which the arms need.

 @param chosen - the run's guest and log folder

 @throws GuestDriverError naming the step that refused

 @example
 ```ts
 await buildAndInstall(chosen);
 ```
 */
async function buildAndInstall(chosen: GuestFacts,): Promise<void> {
  await buildInGuest({ chosen, },);
  await installTheProgram({
    chosen,
    releaseBinary,
  },);
  await recordTheCheckout({ chosen, },);
}

/**
 The arms the plan asks for, each in the guest, in the order a boot would meet them.

 The order is the plan's: the texts are parsed and read back first, then the immutable entry and the `snapdir` value the
 setup sets, then the boots themselves, and the migration last, because it writes a checkout record for a checkout of its
 own.

 @param chosen - the run's guest and log folder

 @throws GuestDriverError naming the step that refused, or the marker a pulled log lacks

 @example
 ```ts
 await everyArm({ chosen });
 ```
 */
async function everyArm({ chosen }: { readonly chosen: GuestFacts },): Promise<void> {
  await theUnitsReadBack({ chosen, },);
  await theRefusals({ chosen, },);
  await theImmutableEntry({ chosen, },);
  await theSnapdirInheritance({ chosen, },);
  await installThePlan({ chosen, },);
  await rebootTheGuest({ chosen, },);
  await readTheBoot({
    chosen,
    expect: 'healthy',
  },);
  await makeTheClone({ chosen, },);
  await rebootTheGuest({ chosen, },);
  await readTheBoot({
    chosen,
    expect: 'healthy',
  },);
  await theFailingBoots({ chosen, },);
  await migrateAndRollBack({ chosen, },);
  await removeThePlan({ chosen, },);
}

/**
 Run one named step of the driver.

 @param chosen - the run's options

 @throws Error naming a step this driver does not have, or what the step refused

 @example
 ```ts
 await runTheStep(chosen);
 ```
 */
async function runTheStep(chosen: GuestFacts & Options,): Promise<void> {
  /**
   The steps this driver knows, and nothing else.
   */
  const steps: ReadonlyMap<string, () => Promise<void>> = new Map([
    [
      'create',
      async function create() {
        await recipeStep({
          chosen,
          step: 'create',
        },);
      },
    ],
    [
      'prepare',
      async function prepare() {
        await recipeStep({
          chosen,
          step: 'prepare',
        },);
      },
    ],
    [
      'layout',
      async function layout() {
        await guestLayout({ chosen, },);
      },
    ],
    [
      'build',
      async function build() {
        await buildAndInstall(chosen,);
      },
    ],
    [
      'units',
      async function units() {
        await theUnitsReadBack({ chosen, },);
      },
    ],
    [
      'underlay',
      async function underlay() {
        await theImmutableEntry({ chosen, },);
      },
    ],
    [
      'snapdir',
      async function snapdir() {
        await theSnapdirInheritance({ chosen, },);
      },
    ],
    [
      'boot',
      async function boot() {
        await installThePlan({ chosen, },);
        await rebootTheGuest({ chosen, },);
        await readTheBoot({
          chosen,
          expect: 'healthy',
        },);
        await makeTheClone({ chosen, },);
        await rebootTheGuest({ chosen, },);
        await readTheBoot({
          chosen,
          expect: 'healthy',
        },);
        await theFailingBoots({ chosen, },);
      },
    ],
    [
      'migration',
      async function migration() {
        await migrateAndRollBack({ chosen, },);
      },
    ],
    [
      'remove',
      async function remove() {
        await removeThePlan({ chosen, },);
      },
    ],
    [
      'destroy',
      async function destroy() {
        await recipeStep({
          chosen,
          step: 'destroy',
        },);
      },
    ],
    [
      'all',
      async function allEveryArm() {
        await all(chosen,);
      },
    ],
  ],);
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
  await step();
  console.log(`zfs step 19 guest: step '${chosen.step}' finished; logs in ${chosen.logDir}`);
}

/**
 The work one `all` run performs between the recipe's own create and destroy steps.

 @param chosen - the run's guest

 @throws GuestDriverError naming the step that refused

 @example
 ```ts
 await theWholeRun({ chosen });
 ```
 */
async function theWholeRun({ chosen }: { readonly chosen: GuestFacts },): Promise<void> {
  await guestLayout({ chosen, },);
  await buildAndInstall(chosen,);
  await everyArm({ chosen, },);
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
    label: 'zfs step 19 guest',
    chosen,
    keepGuest: chosen.keepGuest,
    steps: {
      create: async function createTheGuest() {
        await recipeStep({
          chosen,
          step: 'create',
        },);
      },
      prepare: async function prepareTheGuest() {
        await recipeStep({
          chosen,
          step: 'prepare',
        },);
      },
      work: theWholeRun,
      destroy: async function destroyTheGuest() {
        await recipeStep({
          chosen,
          step: 'destroy',
        },);
      },
    },
  },);
}

/**
 The words the command line gave, after the script's path.
 */
const words: readonly string[] = process.argv
  .slice(2,);
/**
 The step and the values the command line gave it.
 */
const chosen: Options = options({ argumentsRead: runArguments({ argv: words, },), },);
await runTheStep(chosen,);
