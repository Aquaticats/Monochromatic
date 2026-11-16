/**
 Scenario run inside a disposable user and network namespace by `tunnel-lifecycle.integration.test.ts`.
 It brings real WireGuard interfaces up through the built CLI and checks what one `up` leaving a
 kernel object behind does to another:
 one `up` of an interface that loses the race for its link, and two concurrent `up`s of two
 interfaces whose policy tables must not be shared.
 It changes nothing outside the namespace it runs in,
 and it writes no OpenSnitch or tunnel configuration of the machine it runs on.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  cleanInterfaces,
  findProgram,
  ipCall,
  killSwitchTable,
  ruleListing,
  stateOf,
  type ProgramPaths,
  type ProgramResult,
} from './tunnel-lifecycle-fixture-programs.ts';
import {
  createRunEnvironment,
  generateKeyPair,
  startCli,
  waitForFile,
  writeConfig,
  writeStandIns,
} from './tunnel-lifecycle-scenario-fixtures.ts';

/**
 Built CLI bundle exercised at the user boundary.
 */
const CLI_BUNDLE = new URL(
  '../dist/final/node/index.mjs',
  import.meta.url,
).pathname;

/**
 Interfaces the scenario owns.
 */
const INTERFACES: readonly string[] = [
  'wglrace',
  'wgla',
  'wglb',
];

/**
 Directory the scenario writes its fixtures into, given by the test.
 */
const scratch = process.argv[2] ?? '';

/**
 Milliseconds the scenario gives a start that may have ended before its held call was reached.
 */
const EARLY_ANSWER_WAIT_MILLISECONDS = 1_000;

/**
 Characters of a run's output the report keeps from its end.
 */
const REPORTED_OUTPUT_CHARACTERS = 500;

/**
 Answers within a second, so a race with a run that has already ended finishes.

 @returns Nothing.
 */
async function afterOneSecond(): Promise<undefined> {
  await wait(EARLY_ANSWER_WAIT_MILLISECONDS,);
  return undefined;
}

if (scratch === '')
  throw new Error('The scenario needs its scratch directory as the first argument.',);

/**
 Programs the scenario and its stand-ins run, resolved before any stand-in is on the path.
 */
const programs: ProgramPaths = {
  ip: await findProgram({ name: 'ip', },),
  wg: await findProgram({ name: 'wg', },),
  nft: await findProgram({ name: 'nft', },),
};
/**
 Environment every CLI run of the scenario gets.
 */
const environment = await createRunEnvironment({ scratch, },);
await writeStandIns({
  bin: join(
    scratch,
    'bin',
  ),
  realIp: programs.ip,
  realWg: programs.wg,
},);
await mkdir(
  join(
    scratch,
    'run',
  ),
  { recursive: true, },
);
/**
 The peer key all three interfaces carry: a device never holds itself as a peer.
 */
const peer = await generateKeyPair({ wgPath: programs.wg, },);
/**
 Configs of the raced interface, the first interface of the concurrent pair, and the second.
 */
/**
 Config path of the raced interface.
 */
const racedConfig = join(
  scratch,
  'wglrace.conf',
);
/**
 Config path of the first interface of the concurrent pair.
 */
const firstConfig = join(
  scratch,
  'wgla.conf',
);
/**
 Config path of the second interface of the concurrent pair.
 */
const secondConfig = join(
  scratch,
  'wglb.conf',
);
await writeConfig({
  path: racedConfig,
  privateKey: (await generateKeyPair({ wgPath: programs.wg, },)).privateKey,
  peerPublicKey: peer.publicKey,
  address: '10.99.13.1/24',
  allowedIps: '0.0.0.0/0',
  port: 51_813,
},);
await writeConfig({
  path: firstConfig,
  privateKey: (await generateKeyPair({ wgPath: programs.wg, },)).privateKey,
  peerPublicKey: peer.publicKey,
  address: '10.99.11.1/24',
  allowedIps: '0.0.0.0/0',
  port: 51_811,
},);
await writeConfig({
  path: secondConfig,
  privateKey: (await generateKeyPair({ wgPath: programs.wg, },)).privateKey,
  peerPublicKey: peer.publicKey,
  address: '10.99.12.1/24',
  allowedIps: '10.0.0.0/8',
  port: 51_812,
},);
await cleanInterfaces({
  programs,
  interfaceNames: INTERFACES,
},);

// The raced link: one up is held between its probe and its `ip link add` while the other finishes.
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
 The held up, which loses the race for the link.
 */
const held = startCli({
  bundle: CLI_BUNDLE,
  operation: 'up',
  configPath: racedConfig,
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
 The up that finishes first, while the held one waits.
 */
const winner = startCli({
  bundle: CLI_BUNDLE,
  operation: 'up',
  configPath: racedConfig,
  environment: {
    ...environment,
    WG_SHIM_IP: 'none',
  },
},);
/**
 What the winning up answered.
 */
const winnerResult = await winner.done;
/**
 Kernel state after the winning up.
 */
const racedAfterWinner = await stateOf({
  programs,
  interfaceName: 'wglrace',
},);
/**
 Policy rules after the winning up.
 */
const rulesAfterWinner = (await ruleListing({ programs, },)).stdout;
/**
 The kill-switch table the winning up installed.
 */
const killSwitchAfterWinner = (await killSwitchTable({
  programs,
  interfaceName: 'wglrace',
},)).status === 0;
await writeFile(
  goFile,
  '1',
);
/**
 What the losing up answered.
 */
const heldResult = await held.done;
/**
 Kernel state after the losing up rolled back.
 */
const racedAfterLoser = await stateOf({
  programs,
  interfaceName: 'wglrace',
},);
/**
 Policy rules after the losing up rolled back.
 */
const rulesAfterLoser = (await ruleListing({ programs, },)).stdout;
/**
 The kill-switch table after the losing up rolled back.
 */
const killSwitchAfterLoser = (await killSwitchTable({
  programs,
  interfaceName: 'wglrace',
},)).status === 0;
await startCli({
  bundle: CLI_BUNDLE,
  operation: 'down',
  configPath: racedConfig,
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
  bundle: CLI_BUNDLE,
  operation: 'up',
  configPath: firstConfig,
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
  bundle: CLI_BUNDLE,
  operation: 'down',
  configPath: firstConfig,
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
  bundle: CLI_BUNDLE,
  operation: 'up',
  configPath: firstConfig,
  environment: {
    ...environment,
    WG_SHIM_WG: 'hold-fwmark',
  },
},);
/**
 Second concurrent up.
 */
const secondRun = startCli({
  bundle: CLI_BUNDLE,
  operation: 'up',
  configPath: secondConfig,
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
    bundle: CLI_BUNDLE,
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
  await retryUp({ configPath: firstConfig, },);
if (secondResult.exitCode !== 0)
  await retryUp({ configPath: secondConfig, },);
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
await cleanInterfaces({
  programs,
  interfaceNames: INTERFACES,
},);

console.log(JSON.stringify({
  raced: {
    winnerExit: winnerResult.exitCode,
    loserExit: heldResult.exitCode,
    loserOutput: heldResult.output
      .slice(-REPORTED_OUTPUT_CHARACTERS,),
    winnerOutput: winnerResult.output
      .slice(-REPORTED_OUTPUT_CHARACTERS,),
    afterWinner: racedAfterWinner,
    afterLoser: racedAfterLoser,
    killSwitchAfterWinner,
    killSwitchAfterLoser,
    rulesAfterWinner,
    rulesAfterLoser,
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
},),);
