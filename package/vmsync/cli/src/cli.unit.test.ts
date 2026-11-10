import {
  mkdir,
  mkdtemp,
  writeFile,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  findRootCached,
  MISE_MONOREPO,
} from '@monochromatic-dev/fs-path-core/ts';
import spawn, { type SubprocessError, } from 'nano-spawn';

/** Mise monorepo root for spawn cwd, so the built bin path is invariant to the task's launch directory. */
const REPO_ROOT = await findRootCached({ marker: MISE_MONOREPO, },);

/** Built bin path, resolved from the monorepo root. */
const BIN_PATH = 'package/vmsync/cli/dist/final/node/cli.mjs';

/**
 Scratch root every fixture lives under, so no fixture outlives a reboot.
 */
const SCRATCH_ROOT = join(
  homedir(),
  'temp',
  'agent',
);

/**
 Config fixture carrying every form the README documents: line comments, a block
 comment, and a trailing comma after the last member of an object.
 */
const COMMENTED_CONFIG = `{
  // VM identity, copied from the image at import time
  "name": "alpine",
  "importedFrom": "/home/user/images/alpine.qcow2",
  "importedAt": "2026-03-27T12:00:00.000Z",

  /* disk geometry */
  "diskSizeBytes": 2147483648,

  "boot": {
    "memory": "4G",
    "cpus": 4,
  },

  "state": {
    "synced": true,
    "checksums": {
      "qcow2": "sha256:abc",
      "vhdx": "sha256:def",
    },
  },
}`;

/**
 Config fixture a JSONC parser must still refuse: two members with no comma
 between them.
 */
const MALFORMED_CONFIG = `{
  "name": "alpine"
  "importedAt": "2026-03-27T12:00:00.000Z"
}`;

/**
 Creates a disposable home directory, so a config read touches no real VM state.
 
 @returns Absolute path to a fresh directory holding no vmsync data
 
 @example
 ```ts
 const home = await createScratchHome();
 ```
 */
async function createScratchHome(): Promise<string> {
  await mkdir(
    SCRATCH_ROOT,
    {
      recursive: true,
      mode: 0o700,
    },
  );
  return mkdtemp(join(
    SCRATCH_ROOT,
    'vmsync-cli-home-',
  ),);
}

/**
 Writes one VM's `vmsync.jsonc` byte for byte into a disposable home, creating
 the directory layout vmsync reads.
 
 @param home - Disposable home the config belongs to
 
 @param vmName - VM directory name under the vmsync data directory
 
 @param source - File contents written verbatim, comments included
 */
async function writeVmConfig(
  {
    home,
    vmName,
    source,
  }: {
    readonly home: string;
    readonly vmName: string;
    readonly source: string;
  },
): Promise<void> {
  const dir = join(
    home,
    '.local',
    'share',
    'vmsync',
    vmName,
  );
  await mkdir(
    dir,
    { recursive: true, },
  );
  await writeFile(
    join(
      dir,
      'vmsync.jsonc',
    ),
    source,
    'utf8',
  );
}

/**
 Runs the built vmsync bin as a subprocess and returns stdout, stderr, and exit code.
 
 @param args - CLI arguments to pass after `vmsync`
 
 @param home - Disposable home to run against, so no run touches real VM state
 
 @returns Stdout text, stderr text, and numeric exit code
 
 @example
 ```ts
 const result = await runVmsync({ args: ['--help'] });
 // result.exitCode === 0
 ```
 */
async function runVmsync(
  {
    args,
    home,
  }: {
    args: readonly string[];
    readonly home?: string;
  },
): Promise<{
  stdout: string;
  stderr: string;
  exitCode: number;
}> {
  /**
   Environment overrides for this run: the child inherits the current
   environment and only HOME is replaced, so the other variables still reach it.
   */
  const env = (home === undefined) ? {} : { HOME: home, };
  try {
    const result = await spawn('node', [BIN_PATH, ...args,], {
      cwd: REPO_ROOT,
      env,
    },);
    return {
      exitCode: 0,
      stderr: result.stderr,
      stdout: result.stdout,
    };
  }
  catch (error: unknown) {
    const spawnError = error as SubprocessError;
    return {
      exitCode: spawnError.exitCode ?? 1,
      stderr: spawnError.stderr,
      stdout: spawnError.stdout,
    };
  }
}

await describe({
  name: 'vmsync bin (built artifact smoke test)',
  children: [
    //region Help: --help is the only inert path (every subcommand provisions or mutates VM disk images)

    it({
      name: 'prints help and exits 0 with --help',
      fn: async () => {
        const result = await runVmsync({ args: ['--help',], },);
        expect(result.exitCode,).toBe(0,);
        expect(result.stdout,).toContain('vmsync',);
        expect(result.stdout,).toContain('Usage:',);
        expect(result.stdout,).toContain('import',);
        expect(result.stdout,).toContain('sync',);
      },
    },),

    //endregion Help

    //region Config read: the README's JSONC forms reach the parser vmsync owns

    it({
      name: 'reads a config holding comments and a trailing comma',
      fn: async () => {
        const home = await createScratchHome();
        await writeVmConfig({
          home,
          vmName: 'alpine',
          source: COMMENTED_CONFIG,
        },);
        const result = await runVmsync({
          args: ['status', 'alpine',],
          home,
        },);
        expect(result.stderr,).toBe('',);
        expect(result.exitCode,).toBe(0,);
        expect(result.stdout,).toContain('name:       alpine',);
        expect(result.stdout,).toContain('memory:     4G',);
        expect(result.stdout,).toContain('synced:     true',);
      },
    },),

    it({
      name: 'refuses a config whose members have no comma between them',
      fn: async () => {
        const home = await createScratchHome();
        await writeVmConfig({
          home,
          vmName: 'alpine',
          source: MALFORMED_CONFIG,
        },);
        const result = await runVmsync({
          args: ['status', 'alpine',],
          home,
        },);
        expect(result.exitCode,).toBe(1,);
        expect(result.stderr,).toContain('JsoncParseError',);
        expect(result.stderr,).toContain('at offset',);
      },
    },),

    //endregion Config read

    //region Import refusal: a name vmsync may not use is refused and explained

    it({
      name: 'refuses a name derived from the image file that starts with an underscore',
      fn: async () => {
        const home = await createScratchHome();
        /**
         Image path the derived name comes from; the file itself need not exist,
         because a refused name must be refused before vmsync reads any image.
         */
        const imagePath = join(
          home,
          '_alpine.qcow2',
        );
        const result = await runVmsync({
          args: ['import', imagePath,],
          home,
        },);
        expect(result.exitCode,).toBe(1,);
        expect(result.stderr,).toContain('"_alpine"',);
        expect(result.stderr,).toContain('starts with a letter or a digit',);
        expect(result.stderr,).toContain(imagePath,);
        expect(result.stderr,).toContain('--name',);
        expect(result.stderr,).not.toContain('qemu-img',);
      },
    },),

    it({
      name: 'refuses a typed name holding a space, and quotes the space',
      fn: async () => {
        const home = await createScratchHome();
        const result = await runVmsync({
          args: [
            'import',
            join(
              home,
              'alpine.qcow2',
            ),
            '--name',
            'my vm',
          ],
          home,
        },);
        expect(result.exitCode,).toBe(1,);
        expect(result.stderr,).toContain('"my vm"',);
        expect(result.stderr,).toContain('" " (a space)',);
        expect(result.stderr,).not.toContain('qemu-img',);
      },
    },),

    //endregion Import refusal
  ],
},);
