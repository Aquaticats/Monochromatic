import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import spawn, { type SubprocessError, } from 'nano-spawn';
import { spawnSync, } from 'node:child_process';
import { createServer, } from 'node:http';
import {
  chmodSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs';
import { join, } from 'node:path';

import {
  BOOTSTRAP_CACHE_DIRECTORY,
  hostLockPlatformId,
  lockPlatformId,
  MpmShadowLockError,
  parseVersionOutput,
  readLockedMpmShadow,
  sha256Hex,
  TOOLCHAIN_BIN_LINK,
} from '../dist/final/node/testing.mjs';

//region Fixtures: stand-in programs, a release archive, a registry that serves it

/**
 This package's directory, where fixtures go under `dist/temp/test`.
 */
const PACKAGE_DIRECTORY = join(import.meta.dirname, '..',);

/**
 The command-line entry, run from source as the root tasks do when nothing is built.
 */
const ENTRY = join(PACKAGE_DIRECTORY, 'src', 'mpm-shadow.ts',);

/**
 The version the lock names in most tests, and the one a stale copy reports.
 */
const LOCKED_VERSION = '0.2.2';
const STALE_VERSION = '0.2.0';

/**
 This machine's id in the lock, so that the fixture's row is the one the entry looks up.
 */
const PLATFORM_ID = await hostLockPlatformId();

/**
 A stand-in `mpm-shadow` of a version: `--version` prints it, anything else says which copy ran and exits with
 `STUB_EXIT` (0 by default).

 @param version - Version the stand-in reports

 @returns Source of a POSIX shell script

 @example
 ```ts
 writeFileSync(path, lockedStub('0.2.2',),);
 ```
 */
function lockedStub(version: string,): string {
  return `#!/bin/sh\nif [ "$1" = "--version" ]; then echo "mpm-shadow ${version}"; exit 0; fi\n`
    + `echo "locked ${version} ran: $*"\nexit "\${STUB_EXIT:-0}"\n`;
}

/**
 What the old `mpm-shadow` 0.2.0 does with the lock and plan of this repository: it reports its version and refuses
 everything else with the message of the real program.
 */
const STALE_STUB = `#!/bin/sh\nif [ "$1" = "--version" ]; then echo "mpm-shadow ${STALE_VERSION}"; exit 0; fi\n`
  + 'echo "error: tools.mpm-shadow.options.tagPrefix is not a known field" >&2\nexit 2\n';

/**
 A program named `mpm-shadow` that must never run: it is what `PATH` offers.
 */
const PATH_STUB = '#!/bin/sh\necho "ran from PATH: $*"\nexit 99\n';

/**
 A release archive of the stand-in, built with `tar` the way releases are: one top folder holding `mpm-shadow`.

 @param directory - Scratch folder to build in

 @param version - Version the stand-in inside reports

 @returns Path of the `.tar.gz`

 @example
 ```ts
 const archive = buildArchive({ directory, version: '0.2.2', },);
 ```
 */
function buildArchive(
  {
    directory,
    version,
  }: {
    readonly directory: string;
    readonly version: string;
  },
): string {
  /**
   Name of the top folder, as in `monochromatic-mpm-shadow-<version>-<target>.tar.gz`.
   */
  const top = `monochromatic-mpm-shadow-${version}-test`;
  mkdirSync(join(directory, 'pkg', top,), { recursive: true, },);
  writeFileSync(join(directory, 'pkg', top, 'mpm-shadow',), lockedStub(version,),);
  chmodSync(join(directory, 'pkg', top, 'mpm-shadow',), 0o755,);
  /**
   Where the archive goes.
   */
  const archive = join(directory, `${top}.tar.gz`,);
  /**
   The `tar` run.
   */
  const made = spawnSync('tar', ['--create', '--gzip', '--file', archive, '--directory', join(directory, 'pkg',), top,],);
  expect(made.status,).toBe(0,);
  return archive;
}

/**
 A fixture directory that removes itself.

 @example
 ```ts
 using fixture = makeDirectory();
 ```
 */
type Directory = {
  readonly path: string;
  readonly [Symbol.dispose]: () => void;
};

/**
 Makes a fresh directory below this package's `dist/temp/test`.

 @returns The directory, removed by `using`

 @example
 ```ts
 using directory = makeDirectory();
 ```
 */
function makeDirectory(): Directory {
  /**
   Unique path.
   */
  const path = join(
    PACKAGE_DIRECTORY,
    'dist',
    'temp',
    'test',
    `mpm-shadow-${Date.now()}-${Math.random().toString(36,).slice(2, 8,)}`,
  );
  mkdirSync(path, { recursive: true, },);
  return {
    path,
    [Symbol.dispose]() {
      rmSync(path, { recursive: true, force: true, },);
    },
  };
}

/**
 A local registry that serves one body and counts the requests it got.

 @example
 ```ts
 await using served = await serve(bytes,);
 ```
 */
type Served = {
  readonly url: string;
  readonly requests: readonly string[];
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Starts a server on a free local port that answers every request with the same bytes.

 @param body - What to serve

 @returns The address, the request log and the means to stop it

 @example
 ```ts
 await using served = await serve(new Uint8Array(),);
 ```
 */
async function serve(body: Uint8Array,): Promise<Served> {
  /**
   Paths requested so far.
   */
  const requests: string[] = [];
  /**
   The server.
   */
  const server = createServer(function handle(request, response,) {
    requests.push(request.url ?? '',);
    response.writeHead(200, { 'content-type': 'application/gzip', },);
    response.end(body,);
  },);
  await new Promise<void>(function listening(resolve,) {
    server.listen(0, '127.0.0.1', resolve,);
  },);
  /**
   Where it listens.
   */
  const address = server.address();
  if ((address === null) || ((typeof address) === 'string'))
    throw new Error('the test server has no port',);
  return {
    url: `http://127.0.0.1:${String(address.port,)}/mpm-shadow.tar.gz`,
    requests,
    async [Symbol.asyncDispose]() {
      server.closeAllConnections();
      await new Promise<void>(function closed(resolve,) {
        server.close(function done() {
          resolve();
        },);
      },);
    },
  };
}

/**
 Options of {@link makeRepository}.

 @example
 ```ts
 const options: RepositoryOptions = { url: 'http://127.0.0.1:1/a', sha256: 'ab', row: true, staleLink: true, };
 ```
 */
type RepositoryOptions = {
  /**
   Archive address recorded in the lock's row
   */
  readonly url: string;
  /**
   SHA-256 recorded in the lock's row
   */
  readonly sha256: string;
  /**
   Whether the lock has a row for this machine's platform
   */
  readonly row: boolean;
  /**
   Whether `.cache/toolchain-bin/mpm-shadow` is a link to the stale 0.2.0 stand-in
   */
  readonly staleLink: boolean;
};

/**
 Makes a repository root with a lock that names `LOCKED_VERSION`, optionally a stale exposure link, and a `PATH` folder
 whose `mpm-shadow` must never run.

 @param directory - Scratch folder to build in

 @param options - What the lock records and whether the stale link exists

 @returns The root and the `PATH` folder

 @example
 ```ts
 const { root, pathDirectory, } = makeRepository({ directory, options, },);
 ```
 */
function makeRepository(
  {
    directory,
    options,
  }: {
    readonly directory: string;
    readonly options: RepositoryOptions;
  },
) {
  /**
   The root.
   */
  const root = join(directory, 'repository',);
  mkdirSync(root, { recursive: true, },);
  /**
   Lock content: the platform's row, or none.
   */
  const artifacts = options.row ? { [PLATFORM_ID]: { url: options.url, sha256: options.sha256, }, } : {};
  writeFileSync(
    join(root, 'toolchain.lock.json',),
    JSON.stringify({ tools: { 'mpm-shadow': { version: LOCKED_VERSION, artifacts, }, }, },),
  );
  if (options.staleLink) {
    mkdirSync(join(root, '.cache', 'toolchain-bin',), { recursive: true, },);
    mkdirSync(join(directory, 'cargo-bin',), { recursive: true, },);
    writeFileSync(join(directory, 'cargo-bin', 'mpm-shadow',), STALE_STUB,);
    chmodSync(join(directory, 'cargo-bin', 'mpm-shadow',), 0o755,);
    symlinkSync(join(directory, 'cargo-bin', 'mpm-shadow',), join(root, TOOLCHAIN_BIN_LINK,),);
  }
  /**
   The folder `PATH` starts with.
   */
  const pathDirectory = join(directory, 'path-bin',);
  mkdirSync(pathDirectory, { recursive: true, },);
  writeFileSync(join(pathDirectory, 'mpm-shadow',), PATH_STUB,);
  chmodSync(join(pathDirectory, 'mpm-shadow',), 0o755,);
  return { root, pathDirectory, };
}

/**
 What running the entry gave.

 @example
 ```ts
 const run: EntryRun = { exitCode: 0, stdout: '', stderr: '', };
 ```
 */
type EntryRun = {
  readonly exitCode: number;
  readonly stdout: string;
  readonly stderr: string;
};

/**
 Runs the entry in a repository root with the given arguments.

 @param root - Repository root, used as the working directory

 @param pathDirectory - Folder placed first on `PATH`

 @param args - Arguments after the entry

 @param environment - Extra environment variables

 @returns Exit status and both output streams

 @example
 ```ts
 const run = await runEntry({ root, pathDirectory, args: ['install',], environment: {}, },);
 ```
 */
async function runEntry(
  {
    root,
    pathDirectory,
    args,
    environment,
  }: {
    readonly root: string;
    readonly pathDirectory: string;
    readonly args: readonly string[];
    readonly environment: Readonly<Record<string, string>>;
  },
): Promise<EntryRun> {
  return await spawn(process.execPath, [ENTRY, ...args,], {
    cwd: root,
    env: { PATH: `${pathDirectory}:/usr/bin:/bin`, ...environment, },
  },).then(
    function ok(result,) {
      return { exitCode: 0, stdout: result.stdout, stderr: result.stderr, };
    },
    function failed(error: SubprocessError,) {
      return { exitCode: error.exitCode ?? (-1), stdout: error.stdout, stderr: error.stderr, };
    },
  );
}

//endregion Fixtures: stand-in programs, a release archive, a registry that serves it

await describe({
  name: '',
  children: [
    describe({
      name: lockPlatformId.name,
      children: [
        it({
          name: 'names Linux machines by architecture, with -musl only for musl',
          fn: async () => {
            expect(lockPlatformId({ platform: 'linux', arch: 'x64', musl: false, },),).toBe('linux-x64',);
            expect(lockPlatformId({ platform: 'linux', arch: 'arm64', musl: false, },),).toBe('linux-arm64',);
            expect(lockPlatformId({ platform: 'linux', arch: 'x64', musl: true, },),).toBe('linux-x64-musl',);
            expect(lockPlatformId({ platform: 'linux', arch: 'arm64', musl: true, },),).toBe('linux-arm64-musl',);
          },
        },),
        it({
          name: 'names macOS and Windows machines and ignores musl there',
          fn: async () => {
            expect(lockPlatformId({ platform: 'darwin', arch: 'arm64', musl: false, },),).toBe('macos-arm64',);
            expect(lockPlatformId({ platform: 'win32', arch: 'x64', musl: true, },),).toBe('windows-x64',);
          },
        },),
      ],
    },),
    describe({
      name: parseVersionOutput.name,
      children: [
        it({
          name: 'reads the version from the first line and refuses any other output',
          fn: async () => {
            expect(parseVersionOutput('mpm-shadow 0.2.1\n',),).toBe('0.2.1',);
            expect(parseVersionOutput('mpm-shadow 0.2.1\nextra line\n',),).toBe('0.2.1',);
            expect(parseVersionOutput('mpm 8.2.0',),).toBe('',);
            expect(parseVersionOutput('mpm-shadow',),).toBe('',);
            expect(parseVersionOutput('mpm-shadow 0.2.1 beta',),).toBe('',);
            expect(parseVersionOutput('',),).toBe('',);
          },
        },),
      ],
    },),
    describe({
      name: readLockedMpmShadow.name,
      children: [
        it({
          name: 'returns the version and the lowercase row of the platform',
          fn: async () => {
            const locked = readLockedMpmShadow({
              lockText: JSON.stringify({
                tools: { 'mpm-shadow': { version: '0.2.1', artifacts: { 'linux-x64': { url: 'https://x.invalid/a', sha256: 'ABCD', }, }, }, },
              },),
              platformId: 'linux-x64',
            },);
            expect(locked,).toEqual({ version: '0.2.1', archive: { found: true, row: { url: 'https://x.invalid/a', sha256: 'abcd', }, }, },);
          },
        },),
        it({
          name: 'says the platform has no row when the lock lacks it, and keeps the version',
          fn: async () => {
            const locked = readLockedMpmShadow({
              lockText: JSON.stringify({ tools: { 'mpm-shadow': { version: '0.2.1', artifacts: {}, }, }, },),
              platformId: 'macos-arm64',
            },);
            expect(locked,).toEqual({ version: '0.2.1', archive: { found: false, }, },);
          },
        },),
        it({
          name: 'throws MpmShadowLockError for text that is not JSON, a lock without the tool, and a tool without a version',
          fn: async () => {
            const bad = ['not json', '[]', '{}', '{"tools":{}}', '{"tools":{"mpm-shadow":{}}}', '{"tools":{"mpm-shadow":{"version":""}}}',];
            const caught = bad.map(function tryText(lockText,) {
              try {
                readLockedMpmShadow({ lockText, platformId: 'linux-x64', },);
                return 'no error';
              }
              catch (error) {
                return error instanceof MpmShadowLockError ? error.message : `other: ${String(error,)}`;
              }
            },);
            expect(caught.every(function isLockError(message,) {
              return (message !== 'no error') && (!message.startsWith('other:',));
            },),).toBe(true,);
            expect(caught[0],).toContain('not valid JSON',);
            expect(caught[2],).toContain('no "tools" object',);
            expect(caught[3],).toContain('has no "mpm-shadow" tool',);
            expect(caught[4],).toContain('has no version',);
          },
        },),
      ],
    },),
    describe({
      name: 'command-line entry',
      children: [
        it({
          name: 'ignores a stale link and the PATH copy, fetches the locked archive, checks it, and runs it with the arguments',
          fn: async () => {
            using directory = makeDirectory();
            const archive = buildArchive({ directory: directory.path, version: LOCKED_VERSION, },);
            const bytes = new Uint8Array(readFileSync(archive,),);
            await using served = await serve(bytes,);
            const { root, pathDirectory, } = makeRepository({
              directory: directory.path,
              options: { url: served.url, sha256: sha256Hex(bytes,), row: true, staleLink: true, },
            },);

            const run = await runEntry({ root, pathDirectory, args: ['install', '--frozen-lockfile',], environment: {}, },);

            expect(run.exitCode,).toBe(0,);
            // Standard output is the locked program's alone.
            expect(run.stdout.trim(),).toBe(`locked ${LOCKED_VERSION} ran: install --frozen-lockfile`,);
            expect(run.stderr,).toContain(`names mpm-shadow ${LOCKED_VERSION}`,);
            expect(run.stderr,).toContain(`${TOOLCHAIN_BIN_LINK} is mpm-shadow ${STALE_VERSION}`,);
            expect(run.stderr,).toContain(`fetched and verified mpm-shadow ${LOCKED_VERSION}`,);
            expect(
              existsSync(join(root, BOOTSTRAP_CACHE_DIRECTORY, LOCKED_VERSION, 'mpm-shadow',),),
            ).toBe(true,);
            expect(served.requests.length,).toBe(1,);
          },
        },),
        it({
          name: 'finds the fetched copy on the next run and downloads nothing again',
          fn: async () => {
            using directory = makeDirectory();
            const archive = buildArchive({ directory: directory.path, version: LOCKED_VERSION, },);
            const bytes = new Uint8Array(readFileSync(archive,),);
            await using served = await serve(bytes,);
            const { root, pathDirectory, } = makeRepository({
              directory: directory.path,
              options: { url: served.url, sha256: sha256Hex(bytes,), row: true, staleLink: true, },
            },);

            await runEntry({ root, pathDirectory, args: ['list',], environment: {}, },);
            const second = await runEntry({ root, pathDirectory, args: ['list', '--global',], environment: {}, },);

            expect(second.exitCode,).toBe(0,);
            expect(second.stdout.trim(),).toBe(`locked ${LOCKED_VERSION} ran: list --global`,);
            expect(second.stderr,).toBe('',);
            expect(served.requests.length,).toBe(1,);
          },
        },),
        it({
          name: 'runs the exposure link without any download when it is already at the locked version',
          fn: async () => {
            using directory = makeDirectory();
            await using served = await serve(new Uint8Array(),);
            const { root, pathDirectory, } = makeRepository({
              directory: directory.path,
              options: { url: served.url, sha256: 'unused', row: true, staleLink: false, },
            },);
            mkdirSync(join(root, '.cache', 'toolchain-bin',), { recursive: true, },);
            writeFileSync(join(root, TOOLCHAIN_BIN_LINK,), lockedStub(LOCKED_VERSION,),);
            chmodSync(join(root, TOOLCHAIN_BIN_LINK,), 0o755,);

            const run = await runEntry({ root, pathDirectory, args: ['bin', '--json',], environment: {}, },);

            expect(run.exitCode,).toBe(0,);
            expect(run.stdout.trim(),).toBe(`locked ${LOCKED_VERSION} ran: bin --json`,);
            expect(run.stderr,).toBe('',);
            expect(served.requests.length,).toBe(0,);
          },
        },),
        it({
          name: 'passes the program exit status on unchanged',
          fn: async () => {
            using directory = makeDirectory();
            const archive = buildArchive({ directory: directory.path, version: LOCKED_VERSION, },);
            const bytes = new Uint8Array(readFileSync(archive,),);
            await using served = await serve(bytes,);
            const { root, pathDirectory, } = makeRepository({
              directory: directory.path,
              options: { url: served.url, sha256: sha256Hex(bytes,), row: true, staleLink: false, },
            },);

            const run = await runEntry({ root, pathDirectory, args: ['outdated',], environment: { STUB_EXIT: '7', }, },);

            expect(run.exitCode,).toBe(7,);
            expect(run.stdout.trim(),).toBe(`locked ${LOCKED_VERSION} ran: outdated`,);
          },
        },),
        it({
          name: 'refuses an archive whose SHA-256 differs from the lock, installs nothing and runs nothing',
          fn: async () => {
            using directory = makeDirectory();
            const archive = buildArchive({ directory: directory.path, version: LOCKED_VERSION, },);
            const bytes = new Uint8Array(readFileSync(archive,),);
            await using served = await serve(bytes,);
            const { root, pathDirectory, } = makeRepository({
              directory: directory.path,
              options: { url: served.url, sha256: sha256Hex(new Uint8Array([1, 2, 3,],),), row: true, staleLink: true, },
            },);

            const run = await runEntry({ root, pathDirectory, args: ['install',], environment: {}, },);

            expect(run.exitCode,).not.toBe(0,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toContain(`has SHA-256 ${sha256Hex(bytes,)}, but toolchain.lock.json records ${sha256Hex(new Uint8Array([1, 2, 3,],),)}`,);
            expect(
              existsSync(join(root, BOOTSTRAP_CACHE_DIRECTORY,),),
            ).toBe(false,);
          },
        },),
        it({
          name: 'refuses an archive whose program prints another version than the lock names',
          fn: async () => {
            using directory = makeDirectory();
            const archive = buildArchive({ directory: directory.path, version: '0.2.1', },);
            const bytes = new Uint8Array(readFileSync(archive,),);
            await using served = await serve(bytes,);
            const { root, pathDirectory, } = makeRepository({
              directory: directory.path,
              options: { url: served.url, sha256: sha256Hex(bytes,), row: true, staleLink: false, },
            },);

            const run = await runEntry({ root, pathDirectory, args: ['install',], environment: {}, },);

            expect(run.exitCode,).not.toBe(0,);
            expect(run.stderr,).toContain(`prints version 0.2.1, but toolchain.lock.json names ${LOCKED_VERSION}`,);
            expect(
              existsSync(join(root, BOOTSTRAP_CACHE_DIRECTORY, LOCKED_VERSION,),),
            ).toBe(false,);
          },
        },),
        it({
          name: 'uses the mpm-shadow on PATH with a note when the lock has no archive row for this platform',
          fn: async () => {
            using directory = makeDirectory();
            const { root, pathDirectory, } = makeRepository({
              directory: directory.path,
              options: { url: 'http://127.0.0.1:1/none', sha256: 'unused', row: false, staleLink: false, },
            },);

            const run = await runEntry({ root, pathDirectory, args: ['list',], environment: {}, },);

            expect(run.exitCode,).toBe(99,);
            expect(run.stdout.trim(),).toBe('ran from PATH: list',);
            expect(run.stderr,).toContain(`has no mpm-shadow archive for ${PLATFORM_ID}`,);
          },
        },),
        it({
          name: 'fails with a message when the lock has no mpm-shadow tool',
          fn: async () => {
            using directory = makeDirectory();
            const { root, pathDirectory, } = makeRepository({
              directory: directory.path,
              options: { url: 'http://127.0.0.1:1/none', sha256: 'unused', row: false, staleLink: false, },
            },);
            writeFileSync(join(root, 'toolchain.lock.json',), '{"tools":{}}',);
            const noTool = await runEntry({ root, pathDirectory, args: ['list',], environment: {}, },);
            expect(noTool.exitCode,).not.toBe(0,);
            expect(noTool.stderr,).toContain('has no "mpm-shadow" tool',);
          },
        },),
      ],
    },),
  ],
},);
