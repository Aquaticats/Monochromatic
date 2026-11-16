import {
  chmod,
  mkdir,
  mkdtemp,
  readdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import type { WireguardConfig, } from '../dist/final/node/config.mjs';
import { OPENSNITCH_CONFIG_ENVIRONMENT, OPENSNITCH_DAEMON_CONFIG_ENVIRONMENT, } from '../dist/final/node/opensnitch.mjs';
import { down, up, } from '../dist/final/node/tunnel.mjs';

/**
 Interface name this file's fixtures use.
 */
const INTERFACE = 'wguplinkfix';

/**
 Stand-in `ip`, which reports the link of the fixture from a state file so that
 a made link and a refused one can be told apart.

 The mode is the link-add behaviour:
 `make-and-fail` makes the link and then exits 2, as an `ip` that was killed or
 timed out after the kernel had already made the link does;
 `already-exists` answers the refusal the kernel gives when another start made that link first.
 Every other call succeeds, and `link show` answers from the state file.
 */
const STAND_IN_IP = `#!${process.execPath}
import { existsSync, rmSync, writeFileSync } from 'node:fs';
const words = process.argv.slice(2);
const state = process.env.WG_TEST_LINK_STATE;
const showsLink = () => existsSync(state);
if (words.includes('-json') && words.includes('rule')) {
  process.stdout.write('[]');
  process.exit(0);
}
if (words[0] === 'link' && words[1] === 'show') {
  process.exit(showsLink() ? 0 : 1);
}
if (words[0] === 'link' && words[1] === 'add') {
  const mode = process.env.WG_TEST_IP_MODE;
  if (mode === 'already-exists') {
    writeFileSync(state, '');
    process.stderr.write('RTNETLINK answers: File exists\\n');
    process.exit(2);
  }
  writeFileSync(state, '');
  if (mode === 'make-and-fail') {
    process.stderr.write('shim: made the link, then reported failure\\n');
    process.exit(2);
  }
  process.exit(0);
}
if (words[0] === 'link' && words[1] === 'delete') {
  rmSync(state, { force: true });
  process.exit(0);
}
process.exit(0);
`;

/**
 Stand-in `wg`, which reports no fwmark, accepts one, and names one allowed prefix.

 The prefix makes the up sequence install policy rules and a kill switch,
 which is where this file's failure lands.
 */
const STAND_IN_WG = `#!${process.execPath}
const words = process.argv.slice(2);
if (words[0] === 'show' && words[2] === 'fwmark') {
  process.stdout.write('off\\n');
  process.exit(0);
}
if (words[0] === 'show' && words[2] === 'allowed-ips') {
  process.stdout.write('peer-key\\t0.0.0.0/0\\n');
  process.exit(0);
}
process.exit(0);
`;

/**
 Text of a program standing in for a tool that succeeds.
 */
const SUCCEEDING_PROGRAM = `#!${process.execPath}\nprocess.exit(0,);\n`;

/**
 Stand-in `nft`, which lists no tables.

 The traffic block asks `nft --json list tables` what is there before an up changes anything,
 and the listing must be the JSON nft itself prints for an empty rule set.
 */
const STAND_IN_NFT = `#!${process.execPath}
import { existsSync, writeFileSync } from 'node:fs';
const words = process.argv.slice(2);
if (process.env.WG_TEST_NFT_MODE === 'hold-f' && words.includes('-f')) {
  writeFileSync(process.env.WG_TEST_NFT_WAITING_FILE, '1');
  const deadline = Date.now() + 60000;
  while ((!existsSync(process.env.WG_TEST_NFT_GO_FILE)) && (Date.now() < deadline))
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4,),), 0, 0, 25,);
}
if (words.includes('--json') || words.includes('-j')) {
  process.stdout.write(JSON.stringify({ nftables: [], },));
  process.exit(0);
}
process.exit(0);
`;

/**
 Builds the config of the fixture interface.

 @returns The config.
 */
function configOf(): WireguardConfig {
  return {
    interfaceName: INTERFACE,
    addresses: [],
    dns: [],
    dnsSearch: [],
    table: 'auto',
    endpointPorts: [],
    allowedFromFiles: [],
    preUp: [],
    postUp: [],
    preDown: [],
    postDown: [],
    wgConfig: '[Interface]\nPrivateKey = fixture\n',
  };
}

/**
 Waits, bounded, for a file to appear.

 @param path - File to look for.

 @returns Whether it appeared within the bound.
 */
async function fileAppears({ path, }: { readonly path: string; },): Promise<boolean> {
  /**
   When the wait gives up.
   */
  const deadline = performance.now() + 5_000;
  while (performance.now() < deadline) {
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- The file is looked for again within one bound.
      await readFile(path,);
      return true;
    }
    catch (error: unknown) {
      // Not written yet; the bound above decides when to stop looking.
      if ((error as NodeJS.ErrnoException).code !== 'ENOENT')
        throw error;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- The pause is part of the same bounded wait.
    await wait(25,);
  }
  return false;
}

/**
 Runs the up and the down of one interface at the same time, with the up held at its kill switch.

 The hold is where a `down` landing inside an `up` does its damage:
 the link and the rules are there, the kill switch is not yet, and the `down`
 removes all of it while the `up` goes on.

 @returns What the down answered, whether the up returned, and whether the link is there at the end.
 */
async function upAndDownAtOnce(): Promise<{
  readonly downRefusal: string;
  readonly upReturned: boolean;
  readonly linkPresent: boolean;
}> {
  /**
   Scratch directory of this case.
   */
  const scratch = await mkdtemp(join(scratchRoot(), 'wg-quicker-lifecycle-lock-',),);
  /**
   Programs this case stands in for.
   */
  const bin = join(scratch, 'bin',);
  /**
   File the stand-in `ip` keeps the link in.
   */
  const linkState = join(scratch, 'link',);
  /**
   File the held stand-in `nft` writes when it reached the held call.
   */
  const nftWaiting = join(scratch, 'nft-waiting',);
  /**
   File that releases the held call.
   */
  const nftGo = join(scratch, 'nft-go',);
  await mkdir(bin,);
  await mkdir(join(scratch, 'run',),);
  await Promise.all(
    ([
      ['ip', STAND_IN_IP,],
      ['wg', STAND_IN_WG,],
      ['nft', STAND_IN_NFT,],
      ['resolvectl', SUCCEEDING_PROGRAM,],
      ['sysctl', SUCCEEDING_PROGRAM,],
      ['systemd-cat', SUCCEEDING_PROGRAM,],
      ['busctl', SUCCEEDING_PROGRAM,],
    ] as const).map(async ([name, text,],) => {
      const path = join(bin, name,);
      await writeFile(path, text,);
      await chmod(path, 0o700,);
    },),
  );
  /**
   The settings this case had before it started.
   */
  const previous = { ...process.env, };
  process.env.PATH = `${bin}:${previous.PATH ?? ''}`;
  process.env.WG_QUICKER_RUNTIME_DIRECTORY = join(scratch, 'run',);
  process.env[OPENSNITCH_CONFIG_ENVIRONMENT] = join(scratch, 'no-system-fw.json',);
  process.env[OPENSNITCH_DAEMON_CONFIG_ENVIRONMENT] = join(scratch, 'no-daemon.json',);
  process.env.WG_TEST_LINK_STATE = linkState;
  process.env.WG_TEST_IP_MODE = 'none';
  process.env.WG_TEST_NFT_MODE = 'hold-f';
  process.env.WG_TEST_NFT_WAITING_FILE = nftWaiting;
  process.env.WG_TEST_NFT_GO_FILE = nftGo;
  await using restore = {
    async [Symbol.asyncDispose](): Promise<void> {
      for (const [name, value,] of Object.entries(previous,)) {
        if (value === undefined)
          Reflect.deleteProperty(process.env, name,);
        else
          process.env[name] = value;
      }
      for (const name of ['WG_TEST_LINK_STATE', 'WG_TEST_IP_MODE', 'WG_TEST_NFT_MODE', 'WG_TEST_NFT_WAITING_FILE', 'WG_TEST_NFT_GO_FILE',] as const) {
        if (previous[name] === undefined)
          Reflect.deleteProperty(process.env, name,);
      }
      await rm(scratch, { force: true, recursive: true, },);
    },
  };
  /**
   The up that is held at its kill switch, and what it ends with.
   */
  const upEnd = { outcome: 'still running', };
  const started = (async function runTheHeldUp(): Promise<void> {
    try {
      await up({ config: configOf(), },);
      upEnd.outcome = 'returned';
    }
    catch (error: unknown) {
      upEnd.outcome = String(error,);
    }
  })();
  if (!(await fileAppears({ path: nftWaiting, },))) {
    await writeFile(nftGo, '1',);
    await started;
    return {
      downRefusal: `the up never reached its kill switch: ${upEnd.outcome}`,
      upReturned: false,
      linkPresent: false,
    };
  }
  /**
   What the down of the same interface answered.
   */
  const downEnd = { outcome: 'still running', };
  try {
    await down({ config: configOf(), },);
    downEnd.outcome = 'returned';
  }
  catch (error: unknown) {
    downEnd.outcome = String(error,);
  }
  await writeFile(nftGo, '1',);
  await started;
  /**
   What the scratch directory holds for the link file.
   */
  const left = await readdir(scratch,);
  return {
    downRefusal: downEnd.outcome,
    upReturned: upEnd.outcome === 'returned',
    linkPresent: left.includes('link',),
  };
}

/**
 Runs `up` over the stand-in programs and answers whether the fixture's link is there afterwards.

 The link is a file the stand-in `ip` makes for `link add` and removes for `link delete`,
 so a rollback that deletes the link is what removes it.

 @param mode - Link-add behaviour of the stand-in `ip`.

 @returns Whether the link exists after the up returned or threw, and what it threw.
 */
async function upWithStandIns({ mode, }: { readonly mode: 'make-and-fail' | 'already-exists'; },): Promise<{
  readonly linkPresent: boolean;
  readonly thrown?: Error;
}> {
  /**
   Scratch directory of this case.
   */
  const scratch = await mkdtemp(join(scratchRoot(), 'wg-quicker-up-link-',),);
  /**
   Programs this case stands in for.
   */
  const bin = join(scratch, 'bin',);
  /**
   File the stand-in `ip` keeps the link in.
   */
  const linkState = join(scratch, 'link',);
  await mkdir(bin,);
  await mkdir(join(scratch, 'run',),);
  await Promise.all(
    ([
      ['ip', STAND_IN_IP,],
      ['wg', STAND_IN_WG,],
      ['nft', STAND_IN_NFT,],
      ['resolvectl', SUCCEEDING_PROGRAM,],
      ['sysctl', SUCCEEDING_PROGRAM,],
      ['systemd-cat', SUCCEEDING_PROGRAM,],
      ['busctl', SUCCEEDING_PROGRAM,],
    ] as const).map(async ([name, text,]) => {
      const path = join(bin, name,);
      await writeFile(path, text,);
      await chmod(path, 0o700,);
    },),
  );
  /**
   The settings this case had before it started.
   */
  const previous = {
    PATH: process.env.PATH,
    WG_QUICKER_RUNTIME_DIRECTORY: process.env.WG_QUICKER_RUNTIME_DIRECTORY,
    [OPENSNITCH_CONFIG_ENVIRONMENT]: process.env[OPENSNITCH_CONFIG_ENVIRONMENT],
    [OPENSNITCH_DAEMON_CONFIG_ENVIRONMENT]: process.env[OPENSNITCH_DAEMON_CONFIG_ENVIRONMENT],
    WG_TEST_LINK_STATE: process.env.WG_TEST_LINK_STATE,
    WG_TEST_IP_MODE: process.env.WG_TEST_IP_MODE,
  };
  process.env.PATH = `${bin}:${previous.PATH ?? ''}`;
  process.env.WG_QUICKER_RUNTIME_DIRECTORY = join(scratch, 'run',);
  // Both OpenSnitch paths point into the scratch directory, so no firewall configuration of the machine is read or written.
  process.env[OPENSNITCH_CONFIG_ENVIRONMENT] = join(scratch, 'no-system-fw.json',);
  process.env[OPENSNITCH_DAEMON_CONFIG_ENVIRONMENT] = join(scratch, 'no-daemon.json',);
  process.env.WG_TEST_LINK_STATE = linkState;
  process.env.WG_TEST_IP_MODE = mode;
  await using restore = {
    async [Symbol.asyncDispose](): Promise<void> {
      for (const [name, value,] of Object.entries(previous,)) {
        if (value === undefined)
          Reflect.deleteProperty(process.env, name,);
        else
          process.env[name] = value;
      }
      await rm(scratch, { force: true, recursive: true, },);
    },
  };
  /**
   What the up threw, which is a stand-in error when it returned.
   */
  const answered = { thrown: new Error('the up returned',), };
  try {
    await up({ config: configOf(), },);
  }
  catch (error) {
    answered.thrown = Error.isError(error,) ? error : new Error(String(error,),);
  }
  /**
   What the scratch directory holds for the link file.
   */
  const left = await readdir(scratch,);
  if (answered.thrown.message === 'the up returned')
    return { linkPresent: left.includes('link',), };
  return {
    linkPresent: left.includes('link',),
    thrown: answered.thrown,
  };
}

await describe({
  name: 'up with a link its own command made',
  // One at a time: each case owns the process's search path and its OpenSnitch settings.
  concurrency: 1,
  children: [
    it({
      name: 'rolls back a link its ip link add made and then reported failure about, so a failure after the kernel call leaves no link behind',
      fn: async () => {
        const { linkPresent, thrown, } = await upWithStandIns({ mode: 'make-and-fail', },);
        expect(thrown === undefined ? 'returned' : thrown.name,).toBe('CommandError',);
        expect(linkPresent,).toBe(false,);
      },
    },),
    it({
      name: 'refuses a down while an up of the same interface is running, so a start cannot report success with its tunnel gone',
      fn: async () => {
        const { downRefusal, upReturned, linkPresent, } = await upAndDownAtOnce();
        expect(downRefusal,).toContain('Another wg-quicker lifecycle is running for',);
        expect(upReturned,).toBe(true,);
        expect(linkPresent,).toBe(true,);
      },
    },),
    it({
      name: 'leaves alone the link another start made when ip link add refused because it was already there, so a loser never removes a winner',
      fn: async () => {
        const { linkPresent, thrown, } = await upWithStandIns({ mode: 'already-exists', },);
        expect(thrown === undefined ? 'returned' : thrown.name,).toBe('CommandError',);
        expect(linkPresent,).toBe(true,);
      },
    },),
  ],
},);
