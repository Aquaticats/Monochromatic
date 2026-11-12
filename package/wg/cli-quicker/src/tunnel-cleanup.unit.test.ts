import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  access,
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import { cleanup, } from '../dist/final/node/tunnel-cleanup.mjs';
import type { WireguardConfig, } from '../dist/final/node/config.mjs';
import { bypassStatePath, } from '../dist/final/node/tunnel-bypass-path.mjs';

/**
 Interface name this file's fixtures use.
 */
const INTERFACE = 'wgcleanupfix';

/**
 Text of a program standing in for `ip` on a host where another tool deleted the link.
 It reports the link absent, answers a delete of something that is not there with the
 diagnostic `ip` itself prints, and succeeds at everything else.
 */
const ABSENT_LINK_PROGRAM = `#!/usr/bin/env node
const words = process.argv.slice(2);
if (words.includes('link') && words.includes('show'))
  process.exit(1);
if (words.includes('delete')) {
  process.stderr.write('RTNETLINK answers: No such file or directory\\n');
  process.exit(2);
}
process.exit(0);
`;

/**
 Text of a program standing in for a tool that succeeds.
 */
const SUCCEEDING_PROGRAM = '#!/usr/bin/env node\nprocess.exit(0,);\n';

/**
 Builds a config for one interface.

 @param dns - DNS servers the config sets.

 @returns The config.
 */
function configOf({ dns, }: { readonly dns?: readonly string[]; },): WireguardConfig {
  return {
    interfaceName: INTERFACE,
    addresses: [],
    dns: dns ?? [],
    dnsSearch: [],
    table: 'auto',
    exemptMark: 100,
    endpointPorts: [],
    allowedFromFiles: [],
    preUp: [],
    postUp: [],
    preDown: [],
    postDown: [],
    wgConfig: '',
  };
}

/**
 Runs `cleanup` for an interface whose link a stand-in `ip` reports absent,
 with a persisted bypass state whose watcher is gone,
 and hands back what the run left behind.

 @param scratch - Directory holding the stand-ins, the runtime state and the programs.

 @returns Whether the bypass state file is still there, and whether its watcher record is.
 */
async function leftoversAfterCleanup(scratch: string,): Promise<{
  readonly statePresent: boolean;
  readonly watcherRecordPresent: boolean;
}> {
  /**
   Runtime directory the cleanup reads its state from.
   */
  const runtime = join(scratch, 'run',);
  process.env.WG_QUICKER_RUNTIME_DIRECTORY = runtime;
  /**
   A program that has already exited, so the recorded watcher reads as gone.
   */
  const ended = spawn(process.execPath, ['--eval', '0',], { stdio: 'ignore', },);
  await once(ended, 'close',);
  /**
   The state a completed up leaves behind, with no owned routes.
   */
  const statePath = bypassStatePath({ interfaceName: INTERFACE, },);
  await writeFile(
    statePath,
    JSON.stringify({
      version: 2,
      interfaceName: INTERFACE,
      mark: 100,
      table: 52_000,
      preference: 50,
      ownerId: 'owner-of-the-fixture',
      routes: [],
    },),
  );
  await writeFile(
    `${statePath}.watcher.json`,
    JSON.stringify({
      ownerId: 'owner-of-the-fixture',
      pid: ended.pid ?? 1,
      startTime: '1',
    },),
  );
  await cleanup({ config: configOf({},), },);
  /**
   Whether a path still exists.
   */
  const exists = async (path: string,): Promise<boolean> => {
    try {
      await access(path,);
      return true;
    }
    catch (error: unknown) {
      if ((error as NodeJS.ErrnoException).code === 'ENOENT')
        return false;
      throw error;
    }
  };
  return {
    statePresent: await exists(statePath,),
    watcherRecordPresent: await exists(`${statePath}.watcher.json`,),
  };
}

await describe({
  name: cleanup.name,
  // One at a time: each case owns the search path and the runtime directory of the process.
  concurrency: 1,
  children: [
    it({
      name: 'takes away the application-bypass state and the kill switch of an interface whose link is already gone, so a down of a link another tool removed leaves no bypass routing behind',
      fn: async () => {
        /**
         Scratch directory of this test.
         */
        const scratch = await mkdtemp(join(scratchRoot(), 'wg-quicker-cleanup-',),);
        /**
         Programs this test stands in for.
         */
        const bin = join(scratch, 'bin',);
        await mkdir(bin,);
        await mkdir(join(scratch, 'run',),);
        await Promise.all(
          ([
            ['ip', ABSENT_LINK_PROGRAM,],
            ['nft', SUCCEEDING_PROGRAM,],
            ['systemd-cat', SUCCEEDING_PROGRAM,],
            ['busctl', SUCCEEDING_PROGRAM,],
          ] as const).map(async ([name, text,]) => {
            const path = join(bin, name,);
            await writeFile(path, text,);
            await chmod(path, 0o700,);
          }),
        );
        /**
         The search path the cleanup and the programs it runs see.
         */
        const previousPath = process.env.PATH;
        process.env.PATH = `${bin}:${previousPath ?? ''}`;
        await using restore = {
          async [Symbol.asyncDispose](): Promise<void> {
            if (previousPath === undefined)
              delete process.env.PATH;
            else
              process.env.PATH = previousPath;
            delete process.env.WG_QUICKER_RUNTIME_DIRECTORY;
            await rm(scratch, { recursive: true, force: true, },);
          },
        };
        const leftovers = await leftoversAfterCleanup(scratch,);
        expect(leftovers.statePresent,).toBe(false,);
        expect(leftovers.watcherRecordPresent,).toBe(false,);
      },
    },),
    it({
      name: 'reverts the DNS settings and removes the OpenSnitch allowance of an interface whose link is gone, without failing on the absent link',
      fn: async () => {
        /**
         Scratch directory of this test.
         */
        const scratch = await mkdtemp(join(scratchRoot(), 'wg-quicker-cleanup-dns-',),);
        /**
         Programs this test stands in for, and what they were asked to do.
         */
        const bin = join(scratch, 'bin',);
        const calls = join(scratch, 'calls.txt',);
        await mkdir(bin,);
        await mkdir(join(scratch, 'run',),);
        await writeFile(calls, '',);
        await Promise.all(
          ['ip', 'nft', 'resolvectl', 'systemd-cat', 'busctl',].map(async (name,) => {
            const path = join(bin, name,);
            await writeFile(
              path,
              `#!/usr/bin/env node
const fs = require('node:fs');
const words = process.argv.slice(2);
fs.appendFileSync(${JSON.stringify(calls,)}, ${JSON.stringify(`${name} `,)} + words.join(' ') + '\\n',);
if (${JSON.stringify(name,)} !== 'ip')
  process.exit(0);
if (words.includes('link') && words.includes('show'))
  process.exit(1);
if (words.includes('delete')) {
  process.stderr.write('RTNETLINK answers: No such file or directory\\n');
  process.exit(2);
}
process.exit(0);
`,
          );
          await chmod(path, 0o700,);
          }),
        );
        /**
         The search path the cleanup and the programs it runs see.
         */
        const previousPath = process.env.PATH;
        process.env.PATH = `${bin}:${previousPath ?? ''}`;
        await using restore = {
          async [Symbol.asyncDispose](): Promise<void> {
            if (previousPath === undefined)
              delete process.env.PATH;
            else
              process.env.PATH = previousPath;
            delete process.env.WG_QUICKER_RUNTIME_DIRECTORY;
            await rm(scratch, { recursive: true, force: true, },);
          },
        };
        process.env.WG_QUICKER_RUNTIME_DIRECTORY = join(scratch, 'run',);
        await cleanup({ config: configOf({ dns: ['10.9.0.1',], },), },);
        /**
         What the cleanup ran.
         */
        const ran = await readFile(calls, 'utf8',);
        expect(ran,).toContain('resolvectl revert',);
        expect(ran,).not.toContain('ip link delete',);
      },
    },),
  ],
},);
