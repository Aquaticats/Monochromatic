/**
 The three sections of the lifecycle scenario that measure the raced link and the concurrent pair:
 one `up` that loses the race for an interface's link, and two tunnels that must not share a table.

 Each part answers what it observed, so the scenario's own report carries the numbers.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  cleanInterfaces,
  ipCall,
  killSwitchTable,
  ruleListing,
  stateOf,
  type ProgramPaths,
} from './tunnel-lifecycle-fixture-programs.ts';
import {
  startCli,
  waitForFile,
} from './tunnel-lifecycle-scenario-fixtures.ts';

/**
 Interfaces these sections own.
 */
const INTERFACES: readonly string[] = [
  'wglrace',
  'wgla',
  'wglb',
];

/**
 Milliseconds the section gives a start that may have ended before its held call was reached.
 */
const EARLY_ANSWER_WAIT_MILLISECONDS = 1_000;

/**
 Characters of a run's output the report keeps from its end.
 */
const REPORTED_OUTPUT_CHARACTERS = 500;

/**
 How much more of a refused start's output the report keeps than of a start that succeeded.

 The refusal is printed before a Node stack trace, so a report cut at the shorter length holds the trace and not the refusal.
 */
const REFUSAL_OUTPUT_CHARACTERS_MULTIPLIER = 4;

/**
 Characters of a refused second start's output the report keeps.
 */
const REFUSAL_OUTPUT_CHARACTERS = REFUSAL_OUTPUT_CHARACTERS_MULTIPLIER * REPORTED_OUTPUT_CHARACTERS;

/**
 Answers within a second, so a race with a run that has already ended finishes.

 @returns Nothing.
 */
async function afterOneSecond(): Promise<undefined> {
  await wait(EARLY_ANSWER_WAIT_MILLISECONDS,);
  return undefined;
}

/**
 Runs the three sections.

 @param programs - Programs the scenario runs, with the stand-ins in front for these calls.

 @param environment - Environment every CLI run of the scenario gets.

 @param bundle - Absolute path of the built CLI bundle.

 @param scratch - Directory the scenario writes its fixtures into.

 @param configs - Config paths of the raced interface and of the concurrent pair.

 @returns What each section observed.

 @example
 ```ts
 await runRacedParts({ programs, environment, bundle, scratch, configs });
 ```
 */
export async function runRacedParts(
  {
    programs,
    environment,
    bundle,
    scratch,
    configs,
  }: {
    readonly programs: ProgramPaths;
    readonly environment: Readonly<Record<string, string>>;
    readonly bundle: string;
    readonly scratch: string;
    readonly configs: {
      readonly raced: string;
      readonly first: string;
      readonly second: string;
    };
  },
): Promise<{
  readonly raced: unknown;
  readonly solo: unknown;
  readonly concurrent: unknown;
}> {
// The raced start: one up is held at its `ip link add`, and a second start of the same interface runs.
/**
 File the held stand-in writes when it reached the held call.
 */
  const waitingFile = join(
    scratch,
    'raced-waiting',
  );
/**
 File that releases the held call.
 */
  const goFile = join(
    scratch,
    'raced-go',
  );
/**
 The held up, which is the one that goes on to finish.
 */
  const held = startCli({
    bundle,
    operation: 'up',
    configPath: configs.raced,
    environment: {
      ...environment,
      WG_SHIM_IP: 'hold-link-add',
      WG_SHIM_WAITING_FILE: waitingFile,
      WG_SHIM_GO_FILE: goFile,
    },
  },);
  if (!(await waitForFile({ path: waitingFile, },))) {
  /**
   What the held up answered in the second after the wait gave up, when it had already ended.
   */
    const early = await Promise.race([
      held.done,
      afterOneSecond(),
    ],);
    throw new Error(
      `The held up never reached its link add (${early === undefined ? 'still running' : `exit ${String(early.exitCode,)}`}): ${held.printedSoFar()}`,
    );
  }
/**
 The second start of the same interface, which the whole-lifecycle lock refuses.
 */
  const secondStart = await startCli({
    bundle,
    operation: 'up',
    configPath: configs.raced,
    environment: {
      ...environment,
      WG_SHIM_IP: 'none',
    },
  },)
    .done;
/**
 Kernel state while the first start is still held, so nothing of it may be gone yet.
 */
  const racedWhileHeld = await stateOf({
    programs,
    interfaceName: 'wglrace',
  },);
  await writeFile(
    goFile,
    '1',
  );
/**
 What the first start answered once it was released.
 */
  const heldResult = await held.done;
/**
 Kernel state after the first start finished.
 */
  const racedAfterHeld = await stateOf({
    programs,
    interfaceName: 'wglrace',
  },);
/**
 Policy rules after the first start finished.
 */
  const rulesAfterHeld = (await ruleListing({ programs, },)).stdout;
/**
 The kill-switch table the first start installed.
 */
  const killSwitchAfterHeld = (await killSwitchTable({
    programs,
    interfaceName: 'wglrace',
  },)).status === 0;
  await startCli({
    bundle,
    operation: 'down',
    configPath: configs.raced,
    environment: {
      ...environment,
      WG_SHIM_IP: 'none',
    },
  },)
    .done;
// The concurrent pair: a solo up first, as the positive control that one tunnel gets a table of its own.
/**
 What the solo up answered.
 */
  const soloResult = await startCli({
    bundle,
    operation: 'up',
    configPath: configs.first,
    environment: {
      ...environment,
      WG_SHIM_WG: 'none',
    },
  },)
    .done;
/**
 Kernel state of the solo tunnel.
 */
  const soloState = await stateOf({
    programs,
    interfaceName: 'wgla',
  },);
  await startCli({
    bundle,
    operation: 'down',
    configPath: configs.first,
    environment: {
      ...environment,
      WG_SHIM_WG: 'none',
    },
  },)
    .done;

// Both ups are held where the table is chosen, so both would choose the same free table.
/**
 First concurrent up.
 */
  const firstRun = startCli({
    bundle,
    operation: 'up',
    configPath: configs.first,
    environment: {
      ...environment,
      WG_SHIM_WG: 'hold-fwmark',
    },
  },);
/**
 Second concurrent up.
 */
  const secondRun = startCli({
    bundle,
    operation: 'up',
    configPath: configs.second,
    environment: {
      ...environment,
      WG_SHIM_WG: 'hold-fwmark',
    },
  },);
/**
 What the concurrent pair answered.
 */
  const [firstResult, secondResult,] = await Promise.all([
    firstRun.done,
    secondRun.done,
  ],);
/**
 Runs one up again after it lost the choice of a table, as the person would.
 */
  async function retryUp({ configPath, }: { readonly configPath: string; },): Promise<void> {
    await startCli({
      bundle,
      operation: 'up',
      configPath,
      environment: {
        ...environment,
        WG_SHIM_WG: 'none',
      },
    },)
      .done;
  }
  if (firstResult.exitCode !== 0)
    await retryUp({ configPath: configs.first, },);
  if (secondResult.exitCode !== 0)
    await retryUp({ configPath: configs.second, },);
/**
 State of each interface after the concurrent pair and the retry of a start that lost.
 */
  const firstState = await stateOf({
    programs,
    interfaceName: 'wgla',
  },);
/**
 State of the second interface of the concurrent pair.
 */
  const secondState = await stateOf({
    programs,
    interfaceName: 'wglb',
  },);
/**
 Asks the kernel which way one address leaves.

 @param address - Address to ask about.

 @returns The route, trimmed.
 */
  async function routeGet({ address, }: { readonly address: string; },): Promise<string> {
  /**
   What the kernel answered.
   */
    const answered = await ipCall({
      programs,
      args: [
        '-4',
        'route',
        'get',
        address,
      ],
    },);
    return answered.stdout
      .trim();
  }

/**
 What the kernel answers for one address of each tunnel's own prefixes.
 */
  const routeGets = {
    ten: (await ipCall({
      programs,
      args: [
        '-4',
        'route',
        'get',
        '10.0.0.1',
      ],
    },)).stdout
      .trim(),
    eight: (await ipCall({
      programs,
      args: [
        '-4',
        'route',
        'get',
        '8.8.8.8',
      ],
    },)).stdout
      .trim(),
  };

  // The pair is taken down, so the sections after this one measure their own tunnels only.
  await startCli({
    bundle,
    operation: 'down',
    configPath: configs.first,
    environment,
  },)
    .done;
  await startCli({
    bundle,
    operation: 'down',
    configPath: configs.second,
    environment,
  },)
    .done;

  return {
    raced: {
      secondExit: secondStart.exitCode,
      secondOutput: secondStart.output
        .slice(-REFUSAL_OUTPUT_CHARACTERS,),
      whileHeld: racedWhileHeld,
      heldExit: heldResult.exitCode,
      afterHeld: racedAfterHeld,
      killSwitchAfterHeld,
      rulesAfterHeld,
    },
    solo: {
      exit: soloResult.exitCode,
      state: soloState,
    },
    concurrent: {
      firstExit: firstResult.exitCode,
      secondExit: secondResult.exitCode,
      firstOutput: firstResult.output
        .slice(-REPORTED_OUTPUT_CHARACTERS,),
      secondOutput: secondResult.output
        .slice(-REPORTED_OUTPUT_CHARACTERS,),
      first: firstState,
      second: secondState,
      routeGets,
    },
  };
}
