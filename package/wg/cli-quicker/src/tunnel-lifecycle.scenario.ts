/**
 Scenario run inside a disposable user and network namespace by `tunnel-lifecycle.integration.test.ts`.
 It brings real WireGuard interfaces up through the built CLI and checks what one `up` leaving a
 kernel object behind does to another:
 one `up` of an interface that loses the race for its link, and two concurrent `up`s of two
 interfaces that need a policy table of their own.
 It changes nothing outside the namespace it runs in,
 and it writes no OpenSnitch or tunnel configuration of the machine it runs on.

 @module
 */

import { spawn, spawnSync, } from 'node:child_process';
import { existsSync, } from 'node:fs';
import {
  chmod,
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { setTimeout as sleep, } from 'node:timers/promises';

/**
 Built CLI bundle exercised at the user boundary.
 */
const CLI_BUNDLE = new URL(
  '../dist/final/node/index.mjs',
  import.meta.url,
).pathname;

/**
 Interface whose link two `up`s race for.
 */
const RACED_INTERFACE = 'wglrace';

/**
 First interface of the concurrent pair.
 */
const FIRST_INTERFACE = 'wgla';

/**
 Second interface of the concurrent pair.
 */
const SECOND_INTERFACE = 'wglb';

/**
 Bound for one wait of this scenario, in milliseconds.
 */
const WAIT_BOUND_MILLISECONDS = 60_000;

/**
 Delay the stand-in `ip` and `wg` put in front of one call, in milliseconds.

 Long enough that the other `up` finishes while this one is held at the same point,
 which is the scheduling gap the finding needs, without any load on the host.
 */
const STAND_IN_DELAY_MILLISECONDS = 1_500;

/**
 Directory the scenario writes its fixtures into, given by the test.
 */
const scratch = process.argv[2] ?? '';

/**
 Finds a program on the search path, without the stand-ins this scenario installs.

 @param name - Program to find.

 @returns Its absolute path.

 @throws {@link Error} when the search path has no such program.
 */
function resolveProgram({ name, }: { readonly name: string; },): string {
  for (const directory of (process.env.PATH ?? '').split(':',)) {
    if (directory === '')
      continue;
    /**
     Candidate path of the program in this directory.
     */
    const candidate = join(directory, name,);
    if (existsSync(candidate,))
      return candidate;
  }
  throw new Error(`The scenario found no ${name} on its search path.`,);
}

/**
 Runs one command and answers its exit code and output.

 @param command - Program to run.

 @param args - Arguments for it.

 @returns Exit code, standard output and standard error.
 */
function command(
  command: string,
  args: readonly string[],
): { readonly status: number; readonly stdout: string; readonly stderr: string; } {
  /**
   What the command ended with.
   */
  const result = spawnSync(command, [...args,], { encoding: 'utf8', },);
  return {
    status: result.status ?? (-1),
    stdout: result.stdout ?? '',
    stderr: result.stderr ?? '',
  };
}

/**
 Runs one `ip` call in the namespace.

 @param args - Arguments for `ip`.

 @returns Exit code and output.
 */
function ip(args: readonly string[],): ReturnType<typeof command> {
  return command(process.env.WG_SCENARIO_IP ?? 'ip', args,);
}

/**
 Runs one `wg` call in the namespace.

 @param args - Arguments for `wg`.

 @returns Exit code and output.
 */
function wg(args: readonly string[],): ReturnType<typeof command> {
  return command(process.env.WG_SCENARIO_WG ?? 'wg', args,);
}

/**
 Runs one `nft` call in the namespace.

 @param args - Arguments for `nft`.

 @returns Exit code and output.
 */
function nft(args: readonly string[],): ReturnType<typeof command> {
  return command(process.env.WG_SCENARIO_NFT ?? 'nft', args,);
}

/**
 Deletes what earlier rounds of this scenario left, so each round starts from nothing.
 */
async function cleanInterfaces(): Promise<void> {
  for (const iface of [RACED_INTERFACE, FIRST_INTERFACE, SECOND_INTERFACE,])
    ip(['link', 'delete', 'dev', iface,],);
  for (const iface of [RACED_INTERFACE, FIRST_INTERFACE, SECOND_INTERFACE,])
    nft(['delete', 'table', 'inet', `wg-quicker-${iface}`,],);
  // Every policy rule above the kernel's own three belongs to a tunnel; none is left for the next round.
  for (const line of ip(['-4', 'rule', 'show',],).stdout.split('\n',)) {
    /**
     Preference of the rule on this line.
     */
    const preference = Number(line.split(':',)[0],);
    if (Number.isInteger(preference,) && ![0, 32_766, 32_767,].includes(preference,))
      ip(['-4', 'rule', 'delete', 'pref', String(preference,),],);
  }
}

/**
 Writes a stand-in `ip` and a stand-in `wg` that can hold one call open.

 The held call is where the race falls in life: between a probe and the kernel change it decides.
 The stand-ins otherwise run the real program the caller found.

 @param bin - Directory the stand-ins go into.

 @returns Nothing.
 */
async function writeStandIns(
  {
    bin,
    realIp,
    realWg,
  }: {
    readonly bin: string;
    readonly realIp: string;
    readonly realWg: string;
  },
): Promise<void> {
  await writeFile(
    join(bin, 'ip',),
    `#!${process.execPath}
import { spawnSync, } from 'node:child_process';
import { existsSync, writeFileSync, } from 'node:fs';
const words = process.argv.slice(2);
const mode = process.env.WG_SHIM_IP ?? 'none';
if (mode === 'hold-link-add' && words[0] === 'link' && words[1] === 'add') {
  writeFileSync(process.env.WG_SHIM_WAITING_FILE, '1',);
  const deadline = Date.now() + ${String(WAIT_BOUND_MILLISECONDS,)};
  while ((!existsSync(process.env.WG_SHIM_GO_FILE)) && (Date.now() < deadline))
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4,),), 0, 0, 25,);
}
const done = spawnSync(${JSON.stringify(realIp,)}, words, { stdio: 'inherit', },);
process.exit(done.status ?? 1,);
`,
  );
  await writeFile(
    join(bin, 'wg',),
    `#!${process.execPath}
import { spawnSync, } from 'node:child_process';
const words = process.argv.slice(2);
const mode = process.env.WG_SHIM_WG ?? 'none';
if (mode === 'hold-fwmark' && words[0] === 'set' && words.includes('fwmark'))
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4,),), 0, 0, ${String(STAND_IN_DELAY_MILLISECONDS,)});
const done = spawnSync(${JSON.stringify(realWg,)}, words, { stdio: 'inherit', },);
process.exit(done.status ?? 1,);
`,
  );
  await chmod(join(bin, 'ip',), 0o700,);
  await chmod(join(bin, 'wg',), 0o700,);
}

/**
 Generates a private key and its public key inside the namespace.

 @returns The pair.

 @throws {@link Error} when `wg` cannot generate the pair.
 */
function generateKeyPair(): { readonly privateKey: string; readonly publicKey: string; } {
  /**
   Program that generates the pair.
   */
  const program = process.env.WG_SCENARIO_WG ?? 'wg';
  /**
   Private key the interface keeps.
   */
  const privateKey = command(program, ['genkey',],).stdout.trim();
  /**
   Public key its peer configures, which `wg pubkey` reads from standard in.
   */
  const publicKey = spawnSync(
    program,
    ['pubkey',],
    {
      input: `${privateKey}\n`,
      encoding: 'utf8',
    },
  ).stdout.trim();
  if ((privateKey === '') || (publicKey === ''))
    throw new Error('wg did not generate a key pair.',);
  return { privateKey, publicKey, };
}

/**
 Writes one interface config in the form `wg-quicker` reads.

 @param path - Config path to write.

 @param privateKey - Interface private key.

 @param peerPublicKey - Peer public key.

 @param address - Interface address.

 @param allowedIps - Peer `AllowedIPs`.

 @param port - Peer endpoint port.

 @returns Nothing.
 */
async function writeConfig(
  {
    path,
    privateKey,
    peerPublicKey,
    address,
    allowedIps,
    port,
  }: {
    readonly path: string;
    readonly privateKey: string;
    readonly peerPublicKey: string;
    readonly address: string;
    readonly allowedIps: string;
    readonly port: number;
  },
): Promise<void> {
  await writeFile(
    path,
    [
      '[Interface]',
      `PrivateKey = ${privateKey}`,
      `Address = ${address}`,
      '',
      '[Peer]',
      `PublicKey = ${peerPublicKey}`,
      `AllowedIPs = ${allowedIps}`,
      `Endpoint = 127.0.0.1:${String(port,)}`,
      '',
    ].join('\n',),
  );
}

/**
 Starts one CLI command that inherits the scenario's namespace and fixtures.

 @param operation - `up` or `down`.

 @param configPath - Config path for the interface.

 @param environment - Stand-in modes and files for this run.

 @returns The child and a promise of its exit code and output.
 */
function startCli(
  {
    operation,
    configPath,
    environment,
  }: {
    readonly operation: 'up' | 'down';
    readonly configPath: string;
    readonly environment: Readonly<Record<string, string>>;
  },
): {
  readonly done: Promise<{ readonly exitCode: number; readonly output: string; }>;
  readonly printedSoFar: () => string;
} {
  /**
   Child running the built CLI.
   */
  const child = spawn(process.execPath, [CLI_BUNDLE, operation, configPath,], {
    env: {
      ...process.env,
      ...environment,
    },
    stdio: ['ignore', 'pipe', 'pipe',],
  },);
  /**
   Everything the child printed.
   */
  let output = '';
  child.stdout.on('data', (chunk: Buffer,) => {
    output += chunk.toString('utf8',);
  },);
  child.stderr.on('data', (chunk: Buffer,) => {
    output += chunk.toString('utf8',);
  },);
  return {
    done: new Promise(function finished(resolve,) {
      child.once('close', (exitCode,) => {
        resolve({ exitCode: exitCode ?? (-1), output, },);
      },);
    },),
    printedSoFar: () => output,
  };
}

/**
 Waits for a file to appear, with a bound.

 @param path - File the scenario waits for.

 @returns Whether it appeared within the bound.
 */
async function waitForFile({ path, }: { readonly path: string; },): Promise<boolean> {
  /**
   When the wait gives up.
   */
  const deadline = Date.now() + WAIT_BOUND_MILLISECONDS;
  while (Date.now() < deadline) {
    try {
      await readFile(path,);
      return true;
    }
    catch (error: unknown) {
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT')
        throw error;
    }
    await sleep(25,);
  }
  return false;
}

/**
 Reads the policy table the interface carries in its fwmark.

 @param interfaceName - Interface to read.

 @returns Table number, absent when the interface has no fwmark or no link.
 */
function tableOf({ interfaceName, }: { readonly interfaceName: string; },): number | undefined {
  /**
   Output of `wg show <interface> fwmark`.
   */
  const shown = wg(['show', interfaceName, 'fwmark',],);
  if (shown.status !== 0)
    return undefined;
  /**
   Table number carried in the hexadecimal mark.
   */
  const table = Math.trunc(Number(shown.stdout.trim(),),);
  return Number.isSafeInteger(table,) && (table > 0) ? table : undefined;
}

/**
 Lists the routes of one policy table.

 @param table - Table number.

 @returns The listed routes, one per line.
 */
function routesInTable({ table, }: { readonly table: number; },): readonly string[] {
  /**
   IPv4 routes of the table, which is where the interfaces of this scenario route.
   */
  const listed = ip(['-4', 'route', 'show', 'table', String(table,),],);
  return listed.stdout.split('\n',)
    .map((line,) => line.trim(),)
    .filter((line,) => line !== '',);
}

/**
 The kernel state the scenario reports about one interface.
 */
type InterfaceState = {
  /**
   Whether the link is present.
   */
  readonly link: boolean;
  /**
   Whether the link is a WireGuard interface.
   */
  readonly wireguard?: boolean;
  /**
   Policy table carried in the fwmark, absent when none.
   */
  readonly table?: number;
  /**
   Routes of that table.
   */
  readonly routes: readonly string[];
};

/**
 Reads the kernel state of one interface.

 @param interfaceName - Interface to read.

 @returns What the kernel holds for it.
 */
function stateOf({ interfaceName, }: { readonly interfaceName: string; },): InterfaceState {
  /**
   Whether the link is present.
   */
  const detailed = ip(['-d', 'link', 'show', 'dev', interfaceName,],);
  const link = detailed.status === 0;
  if (!link)
    return { link, routes: [], };
  /**
   Policy table carried in the fwmark.
   */
  const table = tableOf({ interfaceName, },);
  return {
    link,
    wireguard: detailed.stdout.includes('wireguard',),
    ...(table === undefined ? {} : { table, }),
    routes: table === undefined ? [] : routesInTable({ table, },),
  };
}

if (scratch === '')
  throw new Error('The scenario needs its scratch directory as the first argument.',);
/**
 Directory the stand-ins go into.
 */
const bin = join(scratch, 'bin',);
/**
 * Directory the CLI keeps its runtime state in.
 */
const runtime = join(scratch, 'run',);
await mkdir(bin, { recursive: true, },);
await mkdir(runtime, { recursive: true, },);
/**
 Real programs, resolved before the stand-ins are first on the search path,
 so a stand-in never runs itself.
 */
const realIp = resolveProgram({ name: 'ip', },);
const realWg = resolveProgram({ name: 'wg', },);
process.env.WG_SCENARIO_IP = realIp;
process.env.WG_SCENARIO_WG = realWg;
process.env.WG_SCENARIO_NFT = resolveProgram({ name: 'nft', },);
await writeStandIns({
  bin,
  realIp,
  realWg,
},);
/**
 * Environment every CLI run of this scenario gets.
 */
const baseEnvironment = {
  PATH: `${bin}:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`,
  HOME: join(scratch, 'home',),
  TMPDIR: join(scratch, 'tmp',),
  XDG_RUNTIME_DIR: join(scratch, 'xdg',),
  WG_QUICKER_RUNTIME_DIRECTORY: runtime,
};
await mkdir(baseEnvironment.HOME, { recursive: true, },);
await mkdir(baseEnvironment.TMPDIR, { recursive: true, },);
await mkdir(baseEnvironment.XDG_RUNTIME_DIR, { recursive: true, },);
/**
 * Configs of the three interfaces, each with a peer key generated here.
 */
const racedKey = generateKeyPair();
const firstKey = generateKeyPair();
const secondKey = generateKeyPair();
// One peer key for all three interfaces: a peer that is the interface's own key is not added,
// because a WireGuard device never holds itself as a peer.
const peerKey = generateKeyPair();
/**
 * Config path of the raced interface.
 */
const racedConfig = join(scratch, `${RACED_INTERFACE}.conf`,);
await writeConfig({
  path: racedConfig,
  privateKey: racedKey.privateKey,
  peerPublicKey: peerKey.publicKey,
  address: '10.99.13.1/24',
  allowedIps: '0.0.0.0/0',
  port: 51_813,
},);
/**
 * Config path of the first interface of the concurrent pair.
 */
const firstConfig = join(scratch, `${FIRST_INTERFACE}.conf`,);
await writeConfig({
  path: firstConfig,
  privateKey: firstKey.privateKey,
  peerPublicKey: peerKey.publicKey,
  address: '10.99.11.1/24',
  allowedIps: '0.0.0.0/0',
  port: 51_811,
},);
/**
 * Config path of the second interface of the concurrent pair, which allows only 10/8.
 */
const secondConfig = join(scratch, `${SECOND_INTERFACE}.conf`,);
await writeConfig({
  path: secondConfig,
  privateKey: secondKey.privateKey,
  peerPublicKey: peerKey.publicKey,
  address: '10.99.12.1/24',
  allowedIps: '10.0.0.0/8',
  port: 51_812,
},);

await cleanInterfaces();

// The raced link: one `up` is held between its probe and its `ip link add` while the other finishes.
/**
 File the held stand-in writes when it reached the held call.
 */
const waitingFile = join(scratch, 'raced-waiting',);
/**
 File that releases the held call.
 */
const goFile = join(scratch, 'raced-go',);
/**
 The held `up`, which loses the race for the link.
 */
const held = startCli({
  operation: 'up',
  configPath: racedConfig,
  environment: {
    ...baseEnvironment,
    WG_SHIM_IP: 'hold-link-add',
    WG_SHIM_WAITING_FILE: waitingFile,
    WG_SHIM_GO_FILE: goFile,
  },
},);
if (!(await waitForFile({ path: waitingFile, },))) {
  /**
   What the held `up` answered in the second after the wait gave up, when it had already ended.
   */
  const early = await Promise.race([
    held.done,
    sleep(1_000,).then(() => undefined),
  ],);
  throw new Error(`The held up never reached its link add (${early === undefined ? 'still running' : `exit ${String(early.exitCode,)}`}): ${held.printedSoFar()}`,);
}
/**
 The `up` that finishes first, while the held one waits.
 */
const winner = startCli({
  operation: 'up',
  configPath: racedConfig,
  environment: { ...baseEnvironment, WG_SHIM_IP: 'none', },
},);
/**
 What the winning `up` answered.
 */
const winnerResult = await winner.done;
/**
 Kernel state after the winning `up`.
 */
const racedAfterWinner = stateOf({ interfaceName: RACED_INTERFACE, },);
/**
 Policy rules after the winning `up`.
 */
const rulesAfterWinner = ip(['-4', 'rule', 'show',],).stdout;
/**
 The kill-switch table the winning `up` installed.
 */
const killSwitchAfterWinner = nft(['list', 'table', 'inet', `wg-quicker-${RACED_INTERFACE}`,],).status
  === 0;
await writeFile(goFile, '1',);
/**
 What the losing `up` answered.
 */
const heldResult = await held.done;
/**
 Kernel state after the losing `up` rolled back.
 */
const racedAfterLoser = stateOf({ interfaceName: RACED_INTERFACE, },);
/**
 The kill-switch table after the losing `up` rolled back.
 */
const killSwitchAfterLoser = nft(['list', 'table', 'inet', `wg-quicker-${RACED_INTERFACE}`,],).status
  === 0;
/**
 Policy rules after the losing `up` rolled back.
 */
const rulesAfterLoser = ip(['-4', 'rule', 'show',],).stdout;
await startCli({
  operation: 'down',
  configPath: racedConfig,
  environment: { ...baseEnvironment, WG_SHIM_IP: 'none', },
},).done;

// The concurrent pair: one solo `up` first, as the positive control that a tunnel gets a table of its own.
/**
 What the solo `up` answered.
 */
const soloResult = await startCli({
  operation: 'up',
  configPath: firstConfig,
  environment: { ...baseEnvironment, WG_SHIM_WG: 'none', },
},).done;
/**
 Kernel state of the solo tunnel.
 */
const soloState = stateOf({ interfaceName: FIRST_INTERFACE, },);
await startCli({
  operation: 'down',
  configPath: firstConfig,
  environment: { ...baseEnvironment, WG_SHIM_WG: 'none', },
},).done;

// Both `up`s are held where the table is chosen, so both would choose the same free table.
/**
 First concurrent `up`.
 */
const firstRun = startCli({
  operation: 'up',
  configPath: firstConfig,
  environment: { ...baseEnvironment, WG_SHIM_WG: 'hold-fwmark', },
},);
/**
 Second concurrent `up`.
 */
const secondRun = startCli({
  operation: 'up',
  configPath: secondConfig,
  environment: { ...baseEnvironment, WG_SHIM_WG: 'hold-fwmark', },
},);
/**
 What the concurrent pair answered.
 */
const [firstResult, secondResult,] = await Promise.all([firstRun.done, secondRun.done,],);
// A start that lost the choice of a table to the other start is run again, as the person would:
// the point is that the two never share one, not that one of them must fail for good.
if (firstResult.exitCode !== 0) {
  await startCli({
    operation: 'up',
    configPath: firstConfig,
    environment: { ...baseEnvironment, WG_SHIM_WG: 'none', },
  },).done;
}
if (secondResult.exitCode !== 0) {
  await startCli({
    operation: 'up',
    configPath: secondConfig,
    environment: { ...baseEnvironment, WG_SHIM_WG: 'none', },
  },).done;
}
/**
 State of each interface after the concurrent pair and the retry of a start that lost.
 */
const concurrent = {
  first: stateOf({ interfaceName: FIRST_INTERFACE, },),
  second: stateOf({ interfaceName: SECOND_INTERFACE, },),
};
/**
 What the kernel answers for one address of each tunnel's own prefixes.
 */
const routeGets = {
  ten: ip(['-4', 'route', 'get', '10.0.0.1',],).stdout.trim(),
  eight: ip(['-4', 'route', 'get', '8.8.8.8',],).stdout.trim(),
};
await cleanInterfaces();

console.log(JSON.stringify({
  raced: {
    winnerExit: winnerResult.exitCode,
    loserExit: heldResult.exitCode,
    loserOutput: heldResult.output.slice(-400,),
    afterWinner: racedAfterWinner,
    afterLoser: racedAfterLoser,
    killSwitchAfterWinner,
    killSwitchAfterLoser,
    rulesAfterWinner,
    rulesAfterLoser,
    winnerOutput: winnerResult.output.slice(-600,),
  },
  solo: {
    exit: soloResult.exitCode,
    state: soloState,
  },
  concurrent: {
    firstExit: firstResult.exitCode,
    secondExit: secondResult.exitCode,
    firstOutput: firstResult.output.slice(-400,),
    secondOutput: secondResult.output.slice(-400,),
    ...concurrent,
    routeGets,
  },
},),);
