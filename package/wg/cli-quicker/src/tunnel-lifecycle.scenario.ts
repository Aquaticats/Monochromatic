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
import { runLaterParts, } from './tunnel-lifecycle-scenario-later-parts.ts';
import { runRacedParts, } from './tunnel-lifecycle-scenario-raced-parts.ts';
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
  'wgls',
  'wglt',
  'wglk',
  'wgld',
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
/**
 Config path of the first tunnel of the suppression pair.
 */
const suppressionFirstConfig = join(
  scratch,
  'wgls.conf',
);
/**
 Config path of the second tunnel of the suppression pair.
 */
const suppressionSecondConfig = join(
  scratch,
  'wglt.conf',
);
/**
 Config path of the tunnel whose start is killed in the table window.
 */
const killedConfig = join(
  scratch,
  'wglk.conf',
);
/**
 Config path of the tunnel that comes up while the killed one's fwmark is still there.
 */
const afterKillConfig = join(
  scratch,
  'wgld.conf',
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
await writeConfig({
  path: suppressionFirstConfig,
  privateKey: (await generateKeyPair({ wgPath: programs.wg, },)).privateKey,
  peerPublicKey: peer.publicKey,
  address: '10.99.21.1/24',
  allowedIps: '0.0.0.0/0',
  port: 51_821,
},);
await writeConfig({
  path: suppressionSecondConfig,
  privateKey: (await generateKeyPair({ wgPath: programs.wg, },)).privateKey,
  peerPublicKey: peer.publicKey,
  address: '10.99.22.1/24',
  allowedIps: '0.0.0.0/0',
  port: 51_822,
},);
await writeConfig({
  path: killedConfig,
  privateKey: (await generateKeyPair({ wgPath: programs.wg, },)).privateKey,
  peerPublicKey: peer.publicKey,
  address: '10.99.23.1/24',
  allowedIps: '0.0.0.0/0',
  port: 51_823,
},);
await writeConfig({
  path: afterKillConfig,
  privateKey: (await generateKeyPair({ wgPath: programs.wg, },)).privateKey,
  peerPublicKey: peer.publicKey,
  address: '10.99.24.1/24',
  allowedIps: '0.0.0.0/0',
  port: 51_824,
},);
await cleanInterfaces({
  programs,
  interfaceNames: INTERFACES,
},);
// A LAN-like connected route in the main table, which must keep its own interface.
await ipCall({
  programs,
  args: [
    'link',
    'add',
    'lan0',
    'type',
    'dummy',
  ],
},);
await ipCall({
  programs,
  args: [
    'addr',
    'add',
    '192.168.77.2/24',
    'dev',
    'lan0',
  ],
},);
await ipCall({
  programs,
  args: [
    'link',
    'set',
    'lan0',
    'up',
  ],
},);

/**
 What the raced-link and concurrent-pair sections observed.
 */
const early = await runRacedParts({
  programs,
  environment,
  bundle: CLI_BUNDLE,
  scratch,
  configs: {
    raced: racedConfig,
    first: firstConfig,
    second: secondConfig,
  },
},);

/**
 What the two later sections observed: the suppression pair and the killed start.
 */
const later = await runLaterParts({
  programs,
  environment,
  bundle: CLI_BUNDLE,
  scratch,
  configs: {
    suppressionFirst: suppressionFirstConfig,
    suppressionSecond: suppressionSecondConfig,
    killed: killedConfig,
    afterKill: afterKillConfig,
  },
},);

await cleanInterfaces({
  programs,
  interfaceNames: INTERFACES,
},);
await ipCall({
  programs,
  args: [
    'link',
    'delete',
    'dev',
    'lan0',
  ],
},);

console.log(JSON.stringify({
  raced: early.raced,
  solo: early.solo,
  concurrent: early.concurrent,
  suppression: later.suppression,
  killed: later.killed,
},),);
