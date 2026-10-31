/**
 Unit tests running the built `search-fetch` bin as a real process.

 These cases pin the user boundary: real exit codes, real stream separation,
 real configuration loading through a disposable home, and degraded mode
 without provider keys. Nothing touches the network or the real home.

 @module
 */

import {
  mkdir,
  mkdtempDisposable,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  join,
} from 'node:path';
import {
  fileURLToPath,
} from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  scratchRoot,
} from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import spawn, { SubprocessError, } from 'nano-spawn';

//region Constants

/**
 Built bin artifact under test.
 */
const CLI_PATH = fileURLToPath(new URL('../dist/final/node/index.mjs', import.meta.url,),);

/**
 Host fixture matching the blocklist entry in the fixture config.
 */
const BLOCKED_HOST = 'blocked.example';

//endregion Constants

//region Helpers

/**
 Spawn the built bin in a disposable home with no provider keys.

 @param directory - disposable working directory and home

 @param args - command-line arguments after the script path

 @returns captured process result for a run expected to exit 0

 @example
 ```ts
 await spawnCli({ directory, args: ['--help'] });
 ```
 */
async function spawnCli(
  {
    directory,
    args,
  }: {
    readonly directory: string;
    readonly args: readonly string[];
  },
): Promise<{
  readonly stdout: string;
  readonly stderr: string;
}> {
  return await spawn(
    process.execPath,
    [CLI_PATH, ...args,],
    {
      cwd: directory,
      env: {
        HOME: directory,
        EXA_API_KEY: '',
        LINKUP_API_KEY: '',
        DEEPSEEK_API_KEY: '',
        ENV_LOCAL_JSON_PATH: join(directory, 'absent-env-local.json',),
      },
      stdin: 'ignore',
    },
  );
}

/**
 Spawn the built bin and capture the expected nonzero exit.

 @param directory - disposable working directory and home

 @param args - command-line arguments after the script path

 @returns captured subprocess failure

 @example
 ```ts
 await spawnCliFailure({ directory, args: [] });
 ```
 */
async function spawnCliFailure(
  {
    directory,
    args,
  }: {
    readonly directory: string;
    readonly args: readonly string[];
  },
): Promise<SubprocessError> {
  /**
   Captured expected failure, or the settled result kept when no failure occurred.
   */
  const caught = await caughtSpawnFailure({
    directory,
    args,
  },);
  expect(caught,).toBeInstanceOf(SubprocessError,);
  return caught as SubprocessError;
}

/**
 Spawn the built bin and keep whatever ended the run.

 @param directory - disposable working directory and home

 @param args - command-line arguments after the script path

 @returns thrown subprocess failure, or the settled result when the run exited 0

 @example
 ```ts
 await caughtSpawnFailure({ directory, args: ['--help'] });
 ```
 */
async function caughtSpawnFailure(
  {
    directory,
    args,
  }: {
    readonly directory: string;
    readonly args: readonly string[];
  },
): Promise<unknown> {
  try {
    return await spawnCli({
      directory,
      args,
    },);
  }
  catch (error: unknown) {
    return error;
  }
}

//endregion Helpers

await describe({
  name: 'built bin',
  concurrency: 1,
  children: [
    it({
      name: 'prints usage on stderr and exits 2 for a bare invocation',
      fn: async () => {
        /**
         Disposable home and working directory.
         */
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'search-fetch-cli-',),);
        /**
         Captured bare-invocation failure.
         */
        const failure = await spawnCliFailure({
          directory: directory.path,
          args: [],
        },);
        expect(failure.exitCode,).toBe(2,);
        expect(failure.stdout,).toBe('',);
        expect(failure.stderr.includes('Usage:'),).toBe(true,);
      },
    },),
    it({
      name: 'prints global usage on stdout and exits 0 for --help',
      fn: async () => {
        /**
         Disposable home and working directory.
         */
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'search-fetch-cli-',),);
        /**
         Captured help run.
         */
        const result = await spawnCli({
          directory: directory.path,
          args: ['--help',],
        },);
        expect(result.stdout.includes('search-fetch search <query>'),).toBe(true,);
        expect(result.stdout.includes('search-fetch fetch <url>'),).toBe(true,);
        expect(result.stderr.trim(),).toBe('',);
      },
    },),
    it({
      name: 'prints verb usage on stdout and exits 0 for search --help',
      fn: async () => {
        /**
         Disposable home and working directory.
         */
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'search-fetch-cli-',),);
        /**
         Captured verb help run.
         */
        const result = await spawnCli({
          directory: directory.path,
          args: ['search', '--help',],
        },);
        expect(result.stdout.includes('search-fetch search <query>'),).toBe(true,);
        expect(result.stderr.trim(),).toBe('',);
      },
    },),
    it({
      name: 'prints the package version on stdout and exits 0 for --version',
      fn: async () => {
        /**
         Disposable home and working directory.
         */
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'search-fetch-cli-',),);
        /**
         Captured version run.
         */
        const result = await spawnCli({
          directory: directory.path,
          args: ['--version',],
        },);
        /**
         Manifest version this package reports.
         */
        const manifest: unknown = JSON.parse(
          await readFile(new URL('../package.json', import.meta.url,), 'utf8',),
        );
        if (((typeof manifest) !== 'object')
          || (manifest === null)
          || (!('version' in manifest))
          || ((typeof manifest.version) !== 'string'))
          throw new Error('package.json has no string version',);
        expect(result.stdout.trim(),).toBe(manifest.version,);
        expect(result.stderr.trim(),).toBe('',);
      },
    },),
    it({
      name: 'prints usage on stderr and exits 2 for an unknown flag',
      fn: async () => {
        /**
         Disposable home and working directory.
         */
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'search-fetch-cli-',),);
        /**
         Captured unknown-flag refusal.
         */
        const failure = await spawnCliFailure({
          directory: directory.path,
          args: ['search', '--nope', 'docs',],
        },);
        expect(failure.exitCode,).toBe(2,);
        expect(failure.stdout,).toBe('',);
        expect(failure.stderr.includes('Unknown option'),).toBe(true,);
        expect(failure.stderr.includes('Usage:'),).toBe(true,);
      },
    },),
    it({
      name: 'refuses a blocklist URL with exit 2 and empty stdout through the real config path',
      fn: async () => {
        /**
         Disposable home and working directory.
         */
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'search-fetch-cli-',),);
        /**
         Disposable pi extension config directory.
         */
        const configDirectory = join(
          directory.path,
          '.pi',
          'agent',
          'extensions',
        );
        await mkdir(configDirectory, { recursive: true, },);
        await writeFile(
          join(configDirectory, 'pi-search-fetch.json',),
          JSON.stringify({
            blocklist: [BLOCKED_HOST,],
          },),
          'utf8',
        );
        /**
         Captured blocklist refusal.
         */
        const failure = await spawnCliFailure({
          directory: directory.path,
          args: ['fetch', `https://${BLOCKED_HOST}/page`,],
        },);
        expect(failure.exitCode,).toBe(2,);
        expect(failure.stdout,).toBe('',);
        expect(failure.stderr.includes('pi-search-fetch blocklist'),).toBe(true,);
      },
    },),
    it({
      name: 'fails a key-less search with exit 1 and empty stdout in degraded mode',
      fn: async () => {
        /**
         Disposable home and working directory.
         */
        await using directory = await mkdtempDisposable(join(scratchRoot(), 'search-fetch-cli-',),);
        /**
         Captured provider failure.
         */
        const failure = await spawnCliFailure({
          directory: directory.path,
          args: ['search', 'cachyos kernel',],
        },);
        expect(failure.exitCode,).toBe(1,);
        expect(failure.stdout,).toBe('',);
        expect(failure.stderr.includes('Search Fetch search failed'),).toBe(true,);
        expect(failure.stderr.includes('missing DeepSeek API key'),).toBe(true,);
      },
    },),
  ],
},);
