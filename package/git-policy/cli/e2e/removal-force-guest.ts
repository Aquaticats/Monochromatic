#!/usr/bin/env node
/**
 Drive the forced removal's guest arms in a throwaway `mvm` guest: what one `--force` passes, what two pass, and what
 neither does.

 The removal reads `/dev/zfs` and makes mounts, so no control on a person's machine may run it against a pool it did not
 build. This driver borrows the ZFS binding's guest recipe for the parts that are not the wrapper's
 (`package/zfs/request-to-openzfs-kernel-device/e2e/run-fixture.ts`): it creates the guest and prepares it with OpenZFS
 2.4.4, the Rust toolchain and `cargo-nextest`, then builds this crate inside it, runs the arms, and destroys the guest.

 The arms, one per scene, each on its own pool (`e2e/removal-force-guest-arms.ts` holds their names and their lines):

 - a clone an interrupted removal left mounted on the staging folder: the pool refuses to destroy it there, and one force
   unmounts it there and then destroys it, on the handler's own receipt;
 - the forced run's live proof and its snapshot batch: the names the pool lists are the names the report records as
   destroyed, in the pool's order, and the head goes after them;
 - a clone the wrapper cannot prove, because the name its record holds answers another GUID: no force and one force
   refuse, and two forces destroy it naming both GUIDs;
 - a destruction plan whose file cannot be read: the settling command and the removal with no force, with one force and
   with two, and the state the building round chose to refuse when the clone is gone as well;
 - a person's lock Git wrote on a real registration: no force and one force refuse it, and two forces delete the lock and
   remove the worktree;
 - a worktree at the bound slot holding a modified file, an untracked file and a staged change.

 Usage: node e2e/removal-force-guest.ts <create|prepare|build|arm|destroy|all> [--name <guest>] [--log-dir <dir>]
        [--fresh] [--arms <list>] [--keep-guest]

 `--fresh` throws the guest's built tree away before building, which a first build needs and a rebuilt plant does not.
 `--arms` selects arms by the names of `e2e/removal-force-guest-arms.ts`, and a run with none selected runs them all.

 The evidence of one run is `removal-force-arm.log`, pulled back into the log directory, and a run passes only when it
 carries every `REMOVAL-FORCE` line the selected arms promised and no `SKIPPED` line, so an arm that returned without
 running fails the run instead of reporting a green suite.
 */
import {
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';
import {
  mkdir,
} from 'node:fs/promises';
import {
  resolve,
} from 'node:path';

import {
  FOLDER_MODE,
} from '../bin/mvm-guest-bounds.ts';
import {
  holdsLineStartingWith,
  missingMarkers,
} from '../bin/mvm-guest-arm-log.ts';
import {
  GuestDriverError,
} from '../bin/mvm-guest-error.ts';
import {
  guestBackend,
} from '../bin/mvm-guest-backend.ts';
import type {
  GuestFacts,
} from '../bin/mvm-guest-guest-facts.ts';
import {
  pushWorktreeSnapshot,
} from '../bin/mvm-guest-host-command.ts';
import {
  type GuestDriverOptions,
  flagWord,
  guestDriverOptions,
  runArguments,
} from '../bin/mvm-guest-options.ts';
import {
  recipeStepOf,
} from '../bin/mvm-guest-recipe-step.ts';
import {
  awaitDetachedGuestScript,
  pullGuestLog,
  startDetachedGuestScript,
} from '../bin/mvm-guest-detached-script.ts';
import {
  runWholeGuestRun,
} from '../bin/mvm-guest-whole-run.ts';
import {
  REMOVAL_FORCE_ARMS,
  type RemovalForceArm,
  filtersetOf,
  markersOf,
  selectedArms,
} from './removal-force-guest-arms.ts';
import {
  ARM_LOG,
  SKIP_WORD,
  armScript,
  buildScript,
} from './removal-force-guest-arm-scripts.ts';

//region Constants

/**
 Milliseconds the guest's build of the library and its tests may take.

 Measured on this host: a first build of this crate at one job took 8m 30s while five other guests ran, and a rebuild of
 one planted file takes minutes. The bound is a stop for a build that hangs, not a deadline the machine is expected to
 meet.
 */
const CRATE_BUILD_TIMEOUT_MS = 2_400_000;

/**
 Milliseconds the guest's selected arms may take.

 Eleven arms each make a pool, a checkout, a registration, a clone and its records, and several drive the wrapper's
 removal against a real handler: the guest arm of the sweep took 34 s for its own scene, and this bound carries that
 scene eleven times over beside the handler's own polls.
 */
const ARM_TIMEOUT_MS = 1_800_000;

/**
 Milliseconds the host waits past the build's own bound, so the guest can write the status line the host reads.
 */
const BUILD_FINISH_MARGIN_MS = 180_000;

/**
 Milliseconds the host waits past the arms' own bound, so the guest can write the status line the host reads.
 */
const ARM_FINISH_MARGIN_MS = 120_000;

/**
 The guest name a run takes when the command line names none.
 */
const DEFAULT_GUEST_NAME = 'removal-force-guest';

/**
 The log directory a run takes when the command line names none: this package's build folder.
 */
const DEFAULT_LOG_DIRECTORY_NAME = 'removal-force-guest-logs';

/**
 The log the guest starts and ends the arm script's own output in.
 */
const ARM_SHELL_LOG = 'removal-force-arm-shell.log';

//endregion Constants

//region Arguments

/**
 What this run was asked to do beyond the four flags every driver shares.
 */
type Options = GuestDriverOptions & {
  /**
   Whether the guest's built tree is thrown away first.
   */
  readonly fresh: boolean;
  /**
   The arms this run selected, in the order the catalog holds them.
   */
  readonly arms: readonly RemovalForceArm[];
};

/**
 The log directory a run keeps its logs in, which this package's build folder holds unless the command line names one.

 @returns the directory this package's build folder holds for this driver's logs

 @example
 ```ts
 defaultLogDirectory(); // <package>/target/removal-force-guest-logs
 ```
 */
function defaultLogDirectory(): string {
  return resolve(
    import.meta.dirname,
    '..',
    'target',
    DEFAULT_LOG_DIRECTORY_NAME,
  );
}

/**
 The step and the values the command line gave it.

 @returns the step, the guest's name, the log directory, the keep decision, the fresh decision and the selected arms

 @example
 ```ts
 const chosen = options();
 ```
 */
function options(): Options {
  /**
   The words this run was started with, after the program and this file's own path.
   */
  const argumentLine = process.argv
    .slice(2,);
  /**
   The four flags every driver shares, and the words after the step that only this driver reads.
   */
  const read = runArguments({ argv: argumentLine, },);
  return {
    step: read.step,
    name: flagWord({
      read,
      flag: '--name',
      fallback: DEFAULT_GUEST_NAME,
    },),
    logDir: flagWord({
      read,
      flag: '--log-dir',
      fallback: defaultLogDirectory(),
    },),
    keepGuest: read.flags
      .has('--keep-guest',),
    fresh: read.flags
      .has('--fresh',),
    arms: selectedArms({
      word: flagWord({
        read,
        flag: '--arms',
        fallback: '',
      },),
    },),
  };
}

//endregion Arguments

//region Steps

/**
 Push a repository-shaped snapshot of the wrapper and build it in the guest.

 @param chosen - the run's options

 @throws GuestDriverError naming the step that refused

 @example
 ```ts
 await build({ chosen });
 ```
 */
async function build({ chosen, }: { readonly chosen: Options; },): Promise<void> {
  await pushWorktreeSnapshot({ chosen, },);
  await startDetachedGuestScript({
    chosen,
    script: buildScript({ fresh: chosen.fresh, },),
    log: 'removal-force-build.log',
    timeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS,
    nameCheck: 'none',
  },);
  await awaitDetachedGuestScript({
    chosen,
    log: 'removal-force-build.log',
    finishTimeoutMilliseconds: CRATE_BUILD_TIMEOUT_MS + BUILD_FINISH_MARGIN_MS,
  },);
}

/**
 What the guest's own status line said the arm script ended with, which a marker check reads beside the pulled log.

 @param chosen - the run's options

 @returns the script's ending, or an empty string when it ended zero

 @example
 ```ts
 const failed = await theArmsEnding({ chosen }); // '' when every arm ended
 ```
 */
async function theArmsEnding({ chosen, }: { readonly chosen: Options; },): Promise<string> {
  try {
    await awaitDetachedGuestScript({
      chosen,
      log: ARM_SHELL_LOG,
      finishTimeoutMilliseconds: ARM_TIMEOUT_MS + ARM_FINISH_MARGIN_MS,
    },);
    return '';
  } catch (error) {
    /**
     The refusal's own words, which a pulled log carries beside them so the person reads which line was missing.
     */
    const said = caughtValueText(error,);
    return said;
  }
}

/**
 Run the selected arms in the guest, pull their log back, and require every line they promised.

 @param chosen - the run's options

 @throws GuestDriverError naming the arm that failed, or the marker the pulled log lacks

 @example
 ```ts
 await arm({ chosen });
 ```
 */
async function arm({ chosen, }: { readonly chosen: Options; },): Promise<void> {
  /**
   The arms this run selected, which the script's filterset and its markers both name.
   */
  const arms: readonly RemovalForceArm[] = chosen.arms;
  await startDetachedGuestScript({
    chosen,
    script: armScript({
      name: backend.hostnameOf(chosen.name,),
      filterset: filtersetOf({ arms, },),
      markers: markersOf({ arms, },),
    },),
    log: ARM_SHELL_LOG,
    timeoutMilliseconds: ARM_TIMEOUT_MS,
    nameCheck: 'none',
  },);
  /**
   The script's own ending, read before the log so a failing run's evidence is on this host either way.
   */
  const failed: string = await theArmsEnding({ chosen, },);
  /**
   The arms' own output, as the host pulled it back, which a failing run needs as much as a passing one.
   */
  const pulled: string = await pullGuestLog({
    chosen,
    file: ARM_LOG,
    log: 'host-pull-arm.log',
  },);
  /**
   The lines the selected arms promised and the pulled log does not hold.
   */
  const missing: readonly string[] = missingMarkers({
    text: pulled,
    markers: markersOf({ arms, },),
  },);
  if (missing.length > 0) {
    throw new GuestDriverError(
      `the pulled arm log lacks ${missing.join('; ')}, so the guest ran the arm without performing it: ${failed}`,
    );
  }
  if (holdsLineStartingWith({
    text: pulled,
    word: SKIP_WORD,
  },)) {
    throw new GuestDriverError('the pulled arm log holds a SKIPPED line, so the arm did not run',);
  }
  if (failed !== '') {
    throw new GuestDriverError(`the arm script ended nonzero: ${failed}`,);
  }
}

/**
 Create, prepare, build, run and destroy, in that order, destroying even when a step refused.

 @param chosen - the run's options

 @throws GuestDriverError naming the step that refused, after the guest it created is gone

 @example
 ```ts
 await all({ chosen });
 ```
 */
async function all({ chosen, }: { readonly chosen: Options; },): Promise<void> {
  await runWholeGuestRun({
    label: 'git-policy removal-force guest',
    chosen,
    keepGuest: chosen.keepGuest,
    steps: {
      ...recipeSteps,
      work: async function buildThenArm({ chosen: workChosen, }: { readonly chosen: GuestFacts; }) {
        /**
         This run's own options, with the guest the whole run made.
         */
        const inside: Options = {
          ...chosen,
          ...workChosen,
        };
        await build({ chosen: inside, },);
        await arm({ chosen: inside, },);
      },
    },
  },);
}

//endregion Steps

//region The command line

/**
 The backend this run drives, which decides the guest's hostname and how a root command is started.
 */
const backend = guestBackend();

/**
 The options this run was given.
 */
const runOptions: Options = options();

/**
 The recipe's three steps, which this driver's command line and its whole run both perform.
 */
const recipeSteps = {
  create: recipeStepOf({ step: 'create', },),
  prepare: recipeStepOf({ step: 'prepare', },),
  destroy: recipeStepOf({ step: 'destroy', },),
};

/**
 The steps this driver knows, and nothing else.
 */
const steps: ReadonlyMap<string, (chosen: Options,) => Promise<void>> = new Map([
  [
    'create',
    async function createStep(chosen: Options) {
      await recipeSteps.create({ chosen, },);
    },
  ],
  [
    'prepare',
    async function prepareStep(chosen: Options) {
      await recipeSteps.prepare({ chosen, },);
    },
  ],
  [
    'build',
    async function buildStep(chosen: Options) {
      await build({ chosen, },);
    },
  ],
  [
    'arm',
    async function armStep(chosen: Options) {
      await arm({ chosen, },);
    },
  ],
  [
    'destroy',
    async function destroyStep(chosen: Options) {
      await recipeSteps.destroy({ chosen, },);
    },
  ],
  [
    'all',
    async function allStep(chosen: Options) {
      await all({ chosen, },);
    },
  ],
],);

/**
 The step this run performs.
 */
const step = steps.get(runOptions.step,);
if (step === undefined) {
  throw new GuestDriverError(`unknown step '${runOptions.step}'; use ${[...steps.keys(),].join(', ',)}`,);
}
await mkdir(
  runOptions.logDir,
  {
    recursive: true,
    mode: FOLDER_MODE,
  },
);
await step(runOptions,);

//endregion The command line
