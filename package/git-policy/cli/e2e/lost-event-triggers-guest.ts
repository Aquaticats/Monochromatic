#!/usr/bin/env node
/**
 Run plan step 19's lost-event cases in a throwaway `mvm` guest with a real ZED.

 The handler side of the lost-event triggers is a ZED delivery, the event filter that decides on it, and the merge
 device around one pass, so the cases need a real `zfs-zed.service`, a real kernel event queue and a real `/run`.
 Nothing here touches the host: the guest's own file-backed pool holds every dataset, and the guest's own ZED
 delivers every event.

 The installed zedlet is this arm's own: a link to the crate's built test binary whose one test runs per delivery,
 reads the delivery ZED handed it, decides it through the released filter and, when the event passes, runs the
 released merge device around a stand-in pass that files the product's receipt. The stand-in is the pass's work
 alone: it reads the request this driver wrote on disk rather than the clone's intent property, because the sweep
 that reads that property is plan step 8's and is not this round's.

 Usage: node e2e/lost-event-triggers-guest.ts <create|prepare|arms|judge|destroy|all>
          [--name <guest>] [--log-dir <dir>] [--keep-guest]
 `create` refuses a name `mvm list` already reports, and `all` destroys only a guest its own `create` returned.

 The evidence of one run is `lost-event-arm.log` and `lost-event-judge.log`, pulled back into the log directory,
 and a run passes only when the first carries every `LOST-EVENT` marker and the second carries
 `LOST-EVENTS every case held` and no `SKIPPED` line, so an arm that returned without running fails the run.

 The cases are `e2e/lost-event-arms/zed-arms.ts`: the fixture the pool and the zedlet need, and one shell script
 per case.
 */
import {
  guestBackend,
} from '../bin/mvm-guest-backend.ts';
import {
  mkdir,
} from 'node:fs/promises';
import {
  resolve,
} from 'node:path';

import {
  runArguments,
  type RunArguments,
} from '../bin/mvm-guest-options.ts';
import {
  recipe,
} from '../bin/mvm-guest-host-command.ts';
import {
  runWholeGuestRun,
  type GuestRunSteps,
} from '../bin/mvm-guest-whole-run.ts';
import type {
  GuestFacts,
} from '../bin/mvm-guest-guest-facts.ts';




import {
  buildInGuest,
  caseScript,
  judgeArguments,
  judgeLog,
  pullArmLog,
} from './lost-event-arms/zed-arms.ts';

/**
 The guest name a run uses when the command line names none.
 */
const defaultGuestName = 'zfs-r9-lost-event-triggers';

/**
 The log folder a run uses when the command line names none, relative to the
 package's own folder.
 */
const defaultLogFolder = 'target/lost-event-guest-logs';

/**
 What one run names: the step, the guest, the log folder and whether `all`
 keeps the guest.
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
};

/**
 The step and the values the command line gave it.

 @param argumentsRead - the words the command line gave, after the script's path

 @returns the step, the guest's name, the log folder and whether the guest is kept

 @example
 ```ts
 options({ argumentsRead: runArguments({ argv: ['arms', '--name', 'zfs-r9-lost'] }) });
 ```
 */
function options({
  argumentsRead,
}: {
  readonly argumentsRead: RunArguments;
},): Options {
  return {
    step: argumentsRead.step,
    name: argumentsRead.values
      .get('--name',)
      ?? defaultGuestName,
    logDir: argumentsRead.values
      .get('--log-dir',)
      ?? resolve(
        import.meta.dirname,
        '..',
        defaultLogFolder,
      ),
    keepGuest: argumentsRead.flags
      .has('--keep-guest',),
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
async function create(chosen: Options,): Promise<void> {
  await recipe({
    chosen,
    step: 'create',
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
async function prepare(chosen: Options,): Promise<void> {
  await recipe({
    chosen,
    step: 'prepare',
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
async function destroy(chosen: Options,): Promise<void> {
  await recipe({
    chosen,
    step: 'destroy',
  },);
}

/**
 Build the crate in the guest and run the cases of every delivery.

 @param chosen - the run's guest and log folder

 @throws Error naming the step that refused, or the marker a pulled log lacks

 @example
 ```ts
 await arms(chosen);
 ```
 */
async function arms(chosen: GuestFacts,): Promise<void> {
  /**
   The case log the guest wrote, whose markers the pull refused when one was
   missing.
   */
  const pulled: string = await pullArmLog({ chosen, },);
  console.log(`lost-event arm log: ${String(pulled.length,)} bytes`);
}

/**
 Judge every case from the deliveries and the receipts the run left.

 @param chosen - the run's guest and log folder

 @throws Error naming the marker the judge's log lacks

 @example
 ```ts
 await judge(chosen);
 ```
 */
async function judge(chosen: GuestFacts,): Promise<void> {
  /**
   What the judge printed, whose last line reports every case held.
   */
  const printed: string = await caseScript({
    chosen,
    script: judgeArguments(),
    log: judgeLog,
    seconds: 300,
  },);
  if (!printed.includes('LOST-EVENTS every case held')) {
    throw new Error(`the judge did not report every case: see ${chosen.logDir}/${judgeLog}`,);
  }
}

/**
 Create, prepare, run every case, judge and destroy, destroying even when a step
 refused.

 @param chosen - the run's options

 @throws Error naming the step that refused, after the guest it created is gone

 @example
 ```ts
 await all(chosen);
 ```
 */
async function all(chosen: Options,): Promise<void> {
  await runWholeGuestRun({
    label: 'lost-event triggers guest',
    chosen,
    keepGuest: chosen.keepGuest,
    steps: runSteps,
  },);
}

/**
 The four steps one `all` run performs, named once so no function of them is
 declared inside another.
 */
const runSteps: GuestRunSteps = {
  create: async function createTheGuest(run,) {
    await recipe({
      chosen: run.chosen,
      step: 'create',
    },);
  },
  prepare: async function prepareTheGuest(run,) {
    await recipe({
      chosen: run.chosen,
      step: 'prepare',
    },);
  },
  work: async function runEveryCase(run,) {
    await arms(run.chosen,);
    await judge(run.chosen,);
  },
  destroy: async function destroyTheGuest(run,) {
    await recipe({
      chosen: run.chosen,
      step: 'destroy',
    },);
  },
};

/**
 The steps this driver knows, and nothing else.
 */
const steps: ReadonlyMap<string, (chosen: Options,) => Promise<unknown>> = new Map([
  [
    'create',
    create,
  ],
  [
    'prepare',
    prepare,
  ],
  [
    'arms',
    arms,
  ],
  [
    'judge',
    judge,
  ],
  [
    'destroy',
    destroy,
  ],
  [
    'all',
    all,
  ],
]);

/**
 Run the step the command line named.

 @param chosen - the run's options

 @throws Error naming the step the driver does not know

 @example
 ```ts
 await runTheStep(chosen);
 ```
 */
async function runTheStep(chosen: Options,): Promise<void> {
  /**
   The step the command line named, or nothing when this driver does not know it.
   */
  const step = steps.get(chosen.step,);
  if (step === undefined)
    throw new Error(`unknown step ${chosen.step}; the steps are ${[...steps.keys(),].join(', ')}`);

  await mkdir(
    chosen.logDir,
    {
      recursive: true,
      mode: 0o700,
    },
  );
  await step(chosen,);
  console.log(`lost-event triggers guest: step '${chosen.step}' finished; logs in ${chosen.logDir}`);
}

/**
 The words the command line gave, after the script's path.
 */
const words: readonly string[] = process.argv
  .slice(2);
/**
 The step and the values the command line gave it.
 */
const chosen: Options = options({
  argumentsRead: runArguments({ argv: words, },),
},);
await runTheStep(chosen,);
