/**
 The two sections of the lifecycle scenario that need their own tunnels:
 two tunnels whose main-table suppressions must stay apart,
 and a start killed between its fwmark and its first rule.

 Each part answers what it observed, so the scenario's own report carries the numbers.

 @module
 */

import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  cleanInterfaces,
  ipCall,
  ruleListing,
  stateOf,
  type ProgramPaths,
} from './tunnel-lifecycle-fixture-programs.ts';
import {
  startCli,
  waitForFile,
} from './tunnel-lifecycle-scenario-fixtures.ts';

/**
 Interfaces these two sections own.
 */
const INTERFACES: readonly string[] = [
  'wgls',
  'wglt',
  'wglk',
  'wgld',
];

/**
 Characters of a run's output the report keeps from its end.
 */
const REPORTED_OUTPUT_CHARACTERS = 500;

/**
 Milliseconds the section waits after a kill for the lock holder to notice its owner is gone.
 */
const LOCK_OWNER_NOTICE_MILLISECONDS = 800;

/**
 Runs the two sections.

 @param programs - Programs the scenario runs, with the stand-ins in front for these calls.

 @param environment - Environment every CLI run of the scenario gets.

 @param bundle - Absolute path of the built CLI bundle.

 @param scratch - Directory the scenario writes its fixtures into.

 @param configs - Config paths of the two tunnels of the suppression pair and of the two of the killed start.

 @returns What each section observed.

 @example
 ```ts
 await runLaterParts({ programs, environment, bundle, scratch, configs });
 ```
 */
export async function runLaterParts(
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
      readonly suppressionFirst: string;
      readonly suppressionSecond: string;
      readonly killed: string;
      readonly afterKill: string;
    };
  },
): Promise<{
  readonly suppression: unknown;
  readonly killed: unknown;
}> {
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

// The suppression pair: two tunnels whose main-table suppressions must stay apart.
/**
 What the two ups of the suppression pair answered.
 */
  const suppressionUps = [
    (await startCli({
      bundle,
      operation: 'up',
      configPath: configs.suppressionFirst,
      environment,
    },)
      .done).exitCode,
    (await startCli({
      bundle,
      operation: 'up',
      configPath: configs.suppressionSecond,
      environment,
    },)
      .done).exitCode,
  ];
/**
 Rules with both tunnels of the suppression pair up.
 */
  const suppressionRulesBothUp = (await ruleListing({ programs, },)).stdout;
/**
 What the kernel answers for the LAN address with both up.
 */
  const lanRouteBothUp = await routeGet({ address: '192.168.77.9', },);
/**
 What the down of the first tunnel answered.
 */
  const suppressionDown = (await startCli({
    bundle,
    operation: 'down',
    configPath: configs.suppressionFirst,
    environment,
  },)
    .done).exitCode;
/**
 Rules after the first tunnel of the pair is down.
 */
  const suppressionRulesAfterDown = (await ruleListing({ programs, },)).stdout;
/**
 What the kernel answers for the LAN address after that down.
 */
  const lanRouteAfterDown = await routeGet({ address: '192.168.77.9', },);
/**
 What the kernel answers for the internet after that down: it must still be the second tunnel.
 */
  const internetRouteAfterDown = await routeGet({ address: '8.8.8.8', },);
// A second start of the first tunnel that fails at its first rule, with the second tunnel up.
/**
 What the failed start answered.
 */
  const failedReclaim = (await startCli({
    bundle,
    operation: 'up',
    configPath: configs.suppressionFirst,
    environment: {
      ...environment,
      WG_SHIM_IP: 'fail-rule-add',
    },
  },)
    .done).exitCode;
/**
 Rules after that failed start rolled back.
 */
  const suppressionRulesAfterFailedUp = (await ruleListing({ programs, },)).stdout;
/**
 What the kernel answers for the LAN address after that failed start.
 */
  const lanRouteAfterFailedUp = await routeGet({ address: '192.168.77.9', },);
  await startCli({
    bundle,
    operation: 'down',
    configPath: configs.suppressionSecond,
    environment,
  },)
    .done;

// The killed start: one up is killed between its fwmark and its first rule, then another tunnel starts.
/**
 File the held stand-in writes when it reached the held call.
 */
  const killedWaiting = join(
    scratch,
    'killed-waiting',
  );
/**
 File that releases the held call.
 */
  const killedGo = join(
    scratch,
    'killed-go',
  );
/**
 The up that is killed in the window.
 */
  const killed = startCli({
    bundle,
    operation: 'up',
    configPath: configs.killed,
    environment: {
      ...environment,
      WG_SHIM_IP: 'hold-rule-add',
      WG_SHIM_WAITING_FILE: killedWaiting,
      WG_SHIM_GO_FILE: killedGo,
    },
  },);
  if (!(await waitForFile({ path: killedWaiting, },)))
    throw new Error(`The killed up never reached its first rule add: ${killed.printedSoFar()}`,);
/**
 State of the killed tunnel while it waits, with its fwmark set and no rule yet.
 */
  const killedWhileHeld = await stateOf({
    programs,
    interfaceName: 'wglk',
  },);
  process.kill(
    killed.pid,
    'SIGKILL',
  );
  await killed.done;
  // The lock holder notices its owner is gone within a hundred milliseconds.
  await wait(LOCK_OWNER_NOTICE_MILLISECONDS,);
/**
 What the tunnel that starts after the kill answered.
 */
  const afterKillResult = await startCli({
    bundle,
    operation: 'up',
    configPath: configs.afterKill,
    environment,
  },)
    .done;
/**
 State of both tunnels after the second one came up.
 */
  const afterKillStates = {
    killed: await stateOf({
      programs,
      interfaceName: 'wglk',
    },),
    after: await stateOf({
      programs,
      interfaceName: 'wgld',
    },),
  };
/**
 What the kernel answers for the internet with the second tunnel up.
 */
  const internetAfterKill = await routeGet({ address: '8.8.8.8', },);
/**
 What the down of the killed tunnel answered.
 */
  const downKilled = await startCli({
    bundle,
    operation: 'down',
    configPath: configs.killed,
    environment,
  },)
    .done;
/**
 State of the live tunnel after the dead one's down.
 */
  const afterKillStateAfterDown = await stateOf({
    programs,
    interfaceName: 'wgld',
  },);
/**
 Everything the rules hold after the dead one's down.
 */
  const rulesAfterDownOfKilled = (await ruleListing({ programs, },)).stdout;
/**
 What the kernel answers for the internet after the dead one's down.
 */
  const internetAfterDownOfKilled = await routeGet({ address: '8.8.8.8', },);
  return {
    suppression: {
      ups: suppressionUps,
      rulesBothUp: suppressionRulesBothUp,
      lanRouteBothUp,
      down: suppressionDown,
      rulesAfterDown: suppressionRulesAfterDown,
      lanRouteAfterDown,
      internetRouteAfterDown,
      failedReclaim,
      rulesAfterFailedUp: suppressionRulesAfterFailedUp,
      lanRouteAfterFailedUp,
    },
    killed: {
      whileHeld: killedWhileHeld,
      afterKillExit: afterKillResult.exitCode,
      afterKillOutput: afterKillResult.output
        .slice(-REPORTED_OUTPUT_CHARACTERS,),
      states: afterKillStates,
      internetAfterKill,
      downExit: downKilled.exitCode,
      afterDownState: afterKillStateAfterDown,
      rulesAfterDown: rulesAfterDownOfKilled,
      internetAfterDown: internetAfterDownOfKilled,
    },
  };
}
