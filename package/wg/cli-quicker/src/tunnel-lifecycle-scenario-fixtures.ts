/**
 Fixtures of the lifecycle scenario: configs, keys, stand-ins and CLI runs.

 The scenario brings real interfaces up through the built CLI inside a disposable
 user and network namespace, so it needs generated keys and configs, a stand-in in
 front of one call of each race, a way to start one CLI run, and the directories
 that run keeps its state in.

 @module
 */

import { spawn, } from 'node:child_process';
import {
  access,
  chmod,
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import { run, } from '../dist/final/node/runner.mjs';

/**
 Bound for one wait of the scenario, in milliseconds.
 */
const WAIT_BOUND_MILLISECONDS = 60_000;

/**
 Milliseconds between checks of a file the scenario waits for.
 */
const WAIT_POLL_MILLISECONDS = 25;

/**
 Mode of the stand-in programs: executable by their owner, and by nobody else.
 */
const STAND_IN_MODE = 0o700;

/**
 Delay the stand-in `ip` and `wg` put in front of one call, in milliseconds.

 Long enough that the other `up` finishes while this one is held at the same point,
 which is the scheduling gap the findings need, without any load on the host.
 */
const STAND_IN_DELAY_MILLISECONDS = 1_500;

/**
 Writes a stand-in `ip` and a stand-in `wg` that can hold one call open.

 The held call is where a race falls in life: between a probe and the kernel change it decides.
 The stand-ins otherwise run the real program the scenario found.

 @param bin - Directory the stand-ins go into.

 @param realIp - Absolute path of the real `ip`.

 @param realWg - Absolute path of the real `wg`.

 @example
 ```ts
 await writeStandIns({ bin: '/fixtures/bin', realIp: '/usr/sbin/ip', realWg: '/usr/bin/wg' });
 ```

 @example
 ```ts
 await writeStandIns({ bin, realIp, realWg });
 ```
 */
export async function writeStandIns(
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
    join(
      bin,
      'ip',
    ),
    `#!${process.execPath}
import { spawn } from 'node:child_process';
import { access, writeFile } from 'node:fs/promises';
const words = process.argv.slice(2);
const mode = process.env.WG_SHIM_IP ?? 'none';
async function held() {
  await writeFile(process.env.WG_SHIM_WAITING_FILE, '1');
  const deadline = Date.now() + ${String(WAIT_BOUND_MILLISECONDS,)};
  for (;;) {
    try { await access(process.env.WG_SHIM_GO_FILE); return; } catch {}
    if (Date.now() >= deadline) return;
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4,),), 0, 0, 25,);
  }
}
async function main() {
  if (mode === 'hold-link-add' && words[0] === 'link' && words[1] === 'add')
    await held();
  // Held between the fwmark and the first rule of a tunnel: the window a kill lands in.
  if (mode === 'hold-rule-add' && words.includes('rule') && words.includes('add'))
    await held();
  // Refused at the first rule, so a second tunnel's up fails after its own table is claimed.
  if (mode === 'fail-rule-add' && words.includes('rule') && words.includes('add')) {
    process.stderr.write('shim: refused the rule\\n');
    process.exit(2);
  }
  const child = spawn(${JSON.stringify(realIp,)}, words, { stdio: 'inherit' });
  child.on('close', (code) => process.exit(code ?? 1));
}
await main();
`,
  );
  await writeFile(
    join(
      bin,
      'wg',
    ),
    `#!${process.execPath}
import { spawn } from 'node:child_process';
const words = process.argv.slice(2);
const mode = process.env.WG_SHIM_WG ?? 'none';
if (mode === 'hold-fwmark' && words[0] === 'set' && words.includes('fwmark'))
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4,),), 0, 0, ${String(STAND_IN_DELAY_MILLISECONDS,)});
const child = spawn(${JSON.stringify(realWg,)}, words, { stdio: 'inherit' });
child.on('close', (code) => process.exit(code ?? 1));
`,
  );
  await chmod(
    join(
      bin,
      'ip',
    ),
    STAND_IN_MODE,
  );
  await chmod(
    join(
      bin,
      'wg',
    ),
    STAND_IN_MODE,
  );
}

/**
 Generates a private key and its public key inside the namespace.

 @param wgPath - Absolute path of `wg`.

 @returns The pair.

 @throws {@link Error} when `wg` cannot generate the pair.

 @example
 ```ts
 await generateKeyPair({ wgPath: '/usr/bin/wg' });
 ```
 */
export async function generateKeyPair(
  { wgPath, }: { readonly wgPath: string; },
): Promise<{
  readonly privateKey: string;
  readonly publicKey: string
}> {
  /**
   Private key the interface keeps.
   */
  const { stdout: privateKey, } = await run({
    command: wgPath,
    args: ['genkey',],
  },);
  /**
   Public key its peer configures, which `wg pubkey` reads from standard in.
   */
  const { stdout: publicKey, } = await run({
    command: wgPath,
    args: ['pubkey',],
    input: privateKey,
  },);
  /**
   Private key without the newline `wg genkey` ends it with.
   */
  const trimmedPrivate = privateKey.trim();
  /**
   Public key without the newline `wg pubkey` ends it with.
   */
  const trimmedPublic = publicKey.trim();
  if ((trimmedPrivate === '') || (trimmedPublic === ''))
    throw new Error('wg did not generate a key pair.',);
  return {
    privateKey: trimmedPrivate,
    publicKey: trimmedPublic,
  };
}

/**
 Writes one interface config in the form `wg-quicker` reads.

 @param path - Config path to write.

 @param privateKey - Interface private key.

 @param peerPublicKey - Peer public key.

 @param address - Interface address.

 @param allowedIps - Peer `AllowedIPs`.

 @param port - Peer endpoint port.

 @example
 ```ts
 await writeConfig({ path: '/fixtures/wg0.conf', privateKey, peerPublicKey, address: '10.0.0.1/24', allowedIps: '0.0.0.0/0', port: 51_820 });
 ```
 */
export async function writeConfig(
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
 One CLI run of the scenario: what it printed, and how it ended.
 */
export type RunningCli = {
  /**
   Program number of the run, for a scenario that has to end it.
   */
  readonly pid: number;
  /**
   Resolves with the exit code and everything the run printed.
   */
  readonly done: Promise<{
    readonly exitCode: number;
    readonly output: string
  }>;
  /**
   Everything the run has printed so far.
   */
  readonly printedSoFar: () => string;
};

/**
 Starts one CLI command in the scenario's namespace.

 @param bundle - Absolute path of the built CLI bundle.

 @param operation - `up` or `down`.

 @param configPath - Config path for the interface.

 @param environment - Stand-in modes and files for this run.

 @returns The run.

 @example
 ```ts
 const run = startCli({ bundle: '/pkg/index.mjs', operation: 'up', configPath: '/fixtures/wg0.conf', environment: {} });
 await run.done;
 ```
 */
export function startCli(
  {
    bundle,
    operation,
    configPath,
    environment,
  }: {
    readonly bundle: string;
    readonly operation: 'up' | 'down';
    readonly configPath: string;
    readonly environment: Readonly<Record<string, string>>;
  },
): RunningCli {
  /**
   Child running the built CLI.
   */
  const child = spawn(
    process.execPath,
    [
      bundle,
      operation,
      configPath,
    ],
    {
      env: {
        ...process.env,
        ...environment,
      },
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Everything the child printed so far.
   */
  const printed = { text: '', };
  /**
   Keeps what a stream delivers.

   @param chunk - A piece of its output.
   */
  function collect(chunk: Buffer,): void {
    printed.text += chunk.toString('utf8',);
  }
  child.stdout
    .on(
    'data',
    collect,
    );
  child.stderr
    .on(
    'data',
    collect,
    );
  /**
   Answer of the run.
   */
  const {
    promise,
    resolve,
  } = Promise.withResolvers<{
    readonly exitCode: number;
    readonly output: string
  }>();
  child.once(
    'close',
    function finished(exitCode,): void {
      resolve({
        exitCode: exitCode ?? (-1),
        output: printed.text,
      },);
    },
  );
  return {
    pid: child.pid ?? 0,
    done: promise,
    printedSoFar: function printedSoFar(): string {
      return printed.text;
    },
  };
}

/**
 Waits for a file to appear, with a bound.

 @param path - File the scenario waits for.

 @returns Whether it appeared within the bound.

 @example
 ```ts
 await waitForFile({ path: '/fixtures/wg0.go' });
 ```
 */
export async function waitForFile({ path, }: { readonly path: string; },): Promise<boolean> {
  /**
   When the wait gives up.
   */
  const deadline = Date.now() + WAIT_BOUND_MILLISECONDS;
  while (Date.now() < deadline) {
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- The file is looked for again within one bound.
      await readFile(path,);
      return true;
    }
    catch (error) {
      // Not there yet; the wait goes on until its bound, and the read's reason is only its absence.
      void error;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- The pause is part of the same bounded wait.
    await wait(WAIT_POLL_MILLISECONDS,);
  }
  return false;
}

/**
 Creates the directories one CLI run keeps its state in.

 @param scratch - Directory of the scenario.

 @returns The environment every run of the scenario gets.

 @example
 ```ts
 await createRunEnvironment({ scratch: '/fixtures' });
 ```
 */
export async function createRunEnvironment(
  { scratch, }: { readonly scratch: string; },
): Promise<Readonly<Record<string, string>>> {
  /**
   Directory the stand-ins go into.
   */
  const bin = join(
    scratch,
    'bin',
  );
  /**
   Environment of every CLI run.
   */
  const environment = {
    PATH: `${bin}:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin`,
    HOME: join(
      scratch,
      'home',
    ),
    TMPDIR: join(
      scratch,
      'tmp',
    ),
    XDG_RUNTIME_DIR: join(
      scratch,
      'xdg',
    ),
    WG_QUICKER_RUNTIME_DIRECTORY: join(
      scratch,
      'run',
    ),
  };
  await mkdir(
    bin,
    { recursive: true, },
  );
  await mkdir(
    environment.HOME,
    { recursive: true, },
  );
  await mkdir(
    environment.TMPDIR,
    { recursive: true, },
  );
  await mkdir(
    environment.XDG_RUNTIME_DIR,
    { recursive: true, },
  );
  await mkdir(
    environment.WG_QUICKER_RUNTIME_DIRECTORY,
    { recursive: true, },
  );
  return environment;
}
