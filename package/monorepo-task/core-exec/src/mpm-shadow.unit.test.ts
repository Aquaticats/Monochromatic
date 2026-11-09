import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import spawn, { type SubprocessError, } from 'nano-spawn';
import {
  spawnSync,
} from 'node:child_process';
import {
  chmodSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  readlinkSync,
  rmSync,
  symlinkSync,
  utimesSync,
  writeFileSync,
} from 'node:fs';
import { rename, } from 'node:fs/promises';
import { createServer, } from 'node:http';
import { join, } from 'node:path';

import {
  checkVersionName,
  COMPLETE_MARKER,
  compareVersions,
  CURRENT_LINK_NAME,
  hostLockPlatformId,
  lockFilePath,
  lockPlatformId,
  MpmShadowLockError,
  olderLockNote,
  parseVersionOutput,
  publishStaged,
  readLockedMpmShadow,
  sha256Hex,
  STALE_LEFTOVER_AGE_MS,
  TOOL_DIRECTORY_NAME,
  TOOLCHAIN_BIN_LINK,
  toolchainRootOf,
  toolchainRootsOf,
  UPGRADE_COMMAND,
  versionOf,
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
 A program named `mpm-shadow` that must never run except as the last resort: it is what `PATH` offers, and it reports
 `PATH_STUB_VERSION` (0.0.1 by default, older than any lock).
 */
const PATH_STUB = '#!/bin/sh\nversion="$PATH_STUB_VERSION"\n[ -n "$version" ] || version=0.0.1\n'
  + 'if [ "$1" = "--version" ]; then echo "mpm-shadow $version"; exit 0; fi\n'
  + 'echo "ran from PATH: $*"\nexit 99\n';

/**
 The older-lock note the entry prints for a copy that runs newer than the lock, spelled as a reader sees it: the version
 pair, the lock file it compared, and the command that moves that lock.

 @param root - Repository root whose lock was compared

 @param usedVersion - The version of the copy that runs

 @returns The note's text

 @example
 ```ts
 expect(run.stderr,).toContain(noteNaming({ root, usedVersion: '0.2.3', },),);
 ```
 */
function noteNaming(
  {
    root,
    usedVersion,
  }: {
    readonly root: string;
    readonly usedVersion: string;
  },
): string {
  return olderLockNote({
    lockFile: lockFilePath(root,),
    lockedVersion: LOCKED_VERSION,
    usedVersion,
  },);
}

/**
 Writes an executable file, making its folder first.

 @param path - File to write

 @param content - Its content

 @example
 ```ts
 writeExecutable({ path: '/x/y/mpm-shadow', content: lockedStub('0.2.2',), },);
 ```
 */
function writeExecutable(
  {
    path,
    content,
  }: {
    readonly path: string;
    readonly content: string;
  },
): void {
  mkdirSync(join(path, '..',), { recursive: true, },);
  writeFileSync(path, content,);
  chmodSync(path, 0o755,);
}

/**
 A release archive of the stand-in, built with `tar` the way releases are: one top folder holding `mpm-shadow`.

 @param directory - Scratch folder to build in

 @param version - Version the stand-in inside reports

 @returns The archive's bytes

 @example
 ```ts
 const bytes = buildArchive({ directory, version: '0.2.2', },);
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
): Uint8Array {
  /**
   Name of the top folder, as in `monochromatic-mpm-shadow-<version>-<target>.tar.gz`.
   */
  const top = `monochromatic-mpm-shadow-${version}-test`;
  writeExecutable({ path: join(directory, 'pkg', top, 'mpm-shadow',), content: lockedStub(version,), },);
  /**
   Where the archive goes.
   */
  const archive = join(directory, `${top}.tar.gz`,);
  /**
   The `tar` run.
   */
  const made = spawnSync('tar', ['--create', '--gzip', '--file', archive, '--directory', join(directory, 'pkg',), top,],);
  expect(made.status,).toBe(0,);
  return new Uint8Array(readFileSync(archive,),);
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
 Makes a repository root with a lock that names `LOCKED_VERSION`, optionally a stale exposure link, a data home that holds the
 toolchain root, and a `PATH` folder whose `mpm-shadow` must never run.

 @param directory - Scratch folder to build in

 @param options - What the lock records and whether the stale link exists

 @returns The root, the `PATH` folder, the data home and the folder of the tool inside the toolchain root

 @example
 ```ts
 const { root, pathDirectory, dataHome, toolDirectory, } = makeRepository({ directory, options, },);
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
    writeExecutable({ path: join(directory, 'cargo-bin', 'mpm-shadow',), content: STALE_STUB, },);
    symlinkSync(join(directory, 'cargo-bin', 'mpm-shadow',), join(root, TOOLCHAIN_BIN_LINK,),);
  }
  /**
   The folder `PATH` starts with.
   */
  const pathDirectory = join(directory, 'path-bin',);
  writeExecutable({ path: join(pathDirectory, 'mpm-shadow',), content: PATH_STUB, },);
  /**
   The data home, whose `monochromatic-dev/toolchain` is the toolchain root the entry installs into.
   */
  const dataHome = join(directory, 'data',);
  const toolDirectory = join(toolchainRootOf({ XDG_DATA_HOME: dataHome, },), TOOL_DIRECTORY_NAME,);
  /**
   The tool folder of the machine-global toolchain root, which the entry scans too.
   */
  const globalToolDirectory = join(toolchainRootsOf({ XDG_DATA_HOME: dataHome, },)[1] ?? '', TOOL_DIRECTORY_NAME,);
  return { root, pathDirectory, dataHome, toolDirectory, globalToolDirectory, };
}

/**
 Puts a version folder of the tool into the toolchain root: the stand-in program inside a top folder, and the completion
 marker when the install counts as finished.

 @param toolDirectory - `<toolchain root>/mpm-shadow`

 @param name - Name of the version folder

 @param prints - Version the program inside reports

 @param finished - Whether the completion marker is written

 @example
 ```ts
 installFolder({ toolDirectory, name: '0.3.0', prints: '0.3.0', finished: true, },);
 ```
 */
function installFolder(
  {
    toolDirectory,
    name,
    prints,
    finished,
  }: {
    readonly toolDirectory: string;
    readonly name: string;
    readonly prints: string;
    readonly finished: boolean;
  },
): void {
  writeExecutable({ path: join(toolDirectory, name, 'pkg', 'mpm-shadow',), content: lockedStub(prints,), },);
  if (finished)
    writeFileSync(join(toolDirectory, name, COMPLETE_MARKER,), `mpm-shadow ${name}\n`,);
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

 @param dataHome - Folder used as `XDG_DATA_HOME`

 @param args - Arguments after the entry

 @param environment - Extra environment variables

 @returns Exit status and both output streams

 @example
 ```ts
 const run = await runEntry({ root, pathDirectory, dataHome, args: ['install',], environment: {}, },);
 ```
 */
async function runEntry(
  {
    root,
    pathDirectory,
    dataHome,
    args,
    environment,
  }: {
    readonly root: string;
    readonly pathDirectory: string;
    readonly dataHome: string;
    readonly args: readonly string[];
    readonly environment: Readonly<Record<string, string>>;
  },
): Promise<EntryRun> {
  return await spawn(process.execPath, [ENTRY, ...args,], {
    cwd: root,
    env: { PATH: `${pathDirectory}:/usr/bin:/bin`, XDG_DATA_HOME: dataHome, ...environment, },
  },).then(
    function ok(result,) {
      return { exitCode: 0, stdout: result.stdout, stderr: result.stderr, };
    },
    function failed(error: SubprocessError,) {
      return { exitCode: error.exitCode ?? (-1), stdout: error.stdout, stderr: error.stderr, };
    },
  );
}

/**
 What a fetching run needs: the archive of the locked version served locally, and a repository whose lock records it.

 @param directory - Scratch folder

 @param staleLink - Whether the exposure link is the stale 0.2.0 stand-in

 @returns The server, the repository pieces and the archive's digest

 @example
 ```ts
 const world = await fetchingWorld({ directory: directory.path, staleLink: true, },);
 ```
 */
async function fetchingWorld(
  {
    directory,
    staleLink,
  }: {
    readonly directory: string;
    readonly staleLink: boolean;
  },
) {
  const bytes = buildArchive({ directory, version: LOCKED_VERSION, },);
  const served = await serve(bytes,);
  return {
    served,
    bytes,
    async [Symbol.asyncDispose]() {
      await served[Symbol.asyncDispose]();
    },
    ...makeRepository({ directory, options: { url: served.url, sha256: sha256Hex(bytes,), row: true, staleLink, }, },),
  };
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
      name: compareVersions.name,
      children: [
        it({
          name: 'orders by the numeric core as numbers, a release before its pre-release, and ignores build metadata',
          fn: async () => {
            expect(compareVersions({ left: '0.2.10', right: '0.2.9', },),).toBeGreaterThan(0,);
            expect(compareVersions({ left: '0.2.2', right: '0.2.2', },),).toBe(0,);
            expect(compareVersions({ left: '0.2.1', right: '0.2.2', },),).toBeLessThan(0,);
            expect(compareVersions({ left: '1.0.0', right: '0.99.99', },),).toBeGreaterThan(0,);
            expect(compareVersions({ left: '0.2.2', right: '0.2.2-rc.1', },),).toBeGreaterThan(0,);
            expect(compareVersions({ left: '0.2.2-rc.1', right: '0.2.2-rc.2', },),).toBeLessThan(0,);
            expect(compareVersions({ left: '0.2.2+build5', right: '0.2.2', },),).toBe(0,);
          },
        },),
      ],
    },),
    describe({
      name: 'compareVersions on pre-releases',
      children: [
        it({
          name: 'follows the SemVer 2.0.0 precedence chain, numeric identifiers as numbers: rc.2 is below rc.10',
          fn: async () => {
            const chain = [
              '1.0.0-alpha',
              '1.0.0-alpha.1',
              '1.0.0-alpha.beta',
              '1.0.0-beta',
              '1.0.0-beta.2',
              '1.0.0-beta.11',
              '1.0.0-rc.1',
              '1.0.0',
            ];
            const pairs = chain.slice(1,).map(function pairWithPrevious(version, index,) {
              return { older: chain[index] ?? '', newer: version, };
            },);
            for (const { older, newer, } of pairs) {
              expect(compareVersions({ left: older, right: newer, },),).toBeLessThan(0,);
              expect(compareVersions({ left: newer, right: older, },),).toBeGreaterThan(0,);
            }
            expect(compareVersions({ left: '0.2.2-rc.2', right: '0.2.2-rc.10', },),).toBeLessThan(0,);
            expect(compareVersions({ left: '0.2.2-rc.10', right: '0.2.2-rc.10', },),).toBe(0,);
            expect(compareVersions({ left: '0.2.2-Beta', right: '0.2.2-alpha', },),).toBeLessThan(0,);
            expect(compareVersions({ left: '0.2.2-1', right: '0.2.2-a', },),).toBeLessThan(0,);
          },
        },),
      ],
    },),
    describe({
      name: toolchainRootOf.name,
      children: [
        it({
          name: 'is monochromatic-dev/toolchain below XDG_DATA_HOME, else below HOME/.local/share, else USERPROFILE',
          fn: async () => {
            expect(toolchainRootOf({ XDG_DATA_HOME: '/data', HOME: '/home/x', },),).toBe('/data/monochromatic-dev/toolchain',);
            expect(toolchainRootOf({ HOME: '/home/x', },),).toBe('/home/x/.local/share/monochromatic-dev/toolchain',);
            expect(toolchainRootOf({ XDG_DATA_HOME: '', HOME: '/home/x', },),).toBe('/home/x/.local/share/monochromatic-dev/toolchain',);
            expect(toolchainRootOf({ USERPROFILE: '/users/x', },),).toBe('/users/x/.local/share/monochromatic-dev/toolchain',);
          },
        },),
        it({
          name: 'lists the repository root and the machine-global root, in that order, below the same data home',
          fn: async () => {
            expect(toolchainRootsOf({ XDG_DATA_HOME: '/data', },),).toEqual([
              '/data/monochromatic-dev/toolchain',
              '/data/monochromatic-dev/toolchain-global',
            ],);
            expect(toolchainRootsOf({ HOME: '/home/x', },),).toEqual([
              '/home/x/.local/share/monochromatic-dev/toolchain',
              '/home/x/.local/share/monochromatic-dev/toolchain-global',
            ],);
          },
        },),
        it({
          name: 'throws MpmShadowLockError when no directory can be found',
          fn: async () => {
            expect(function noDirectory() {
              toolchainRootOf({},);
            },).toThrow(MpmShadowLockError,);
          },
        },),
      ],
    },),
    describe({
      name: checkVersionName.name,
      children: [
        it({
          name: 'accepts a plain version and refuses names that could leave the tool folder',
          fn: async () => {
            expect(function plain() {
              checkVersionName('0.2.2',);
            },).not.toThrow();
            for (const bad of ['', '.', '..', '../x', 'a/b', '.hidden', CURRENT_LINK_NAME,]) {
              expect(function crafted() {
                checkVersionName(bad,);
              },).toThrow(MpmShadowLockError,);
            }
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
      name: versionOf.name,
      children: [
        it({
          name: 'ends a candidate that gives no answer within the timeout and says so',
          fn: async () => {
            using directory = makeDirectory();
            const hanging = join(directory.path, 'hanging',);
            writeExecutable({ path: hanging, content: '#!/bin/sh\nexec sleep 30\n', },);
            const answer = await versionOf({ binary: hanging, timeoutMs: 300, },);
            expect(answer.usable,).toBe(false,);
            expect(answer.usable ? '' : answer.reason,).toContain('gave no answer within 0.3 seconds',);
          },
        },),
        it({
          name: 'says why a missing file is not usable',
          fn: async () => {
            const answer = await versionOf({ binary: '/nonexistent/mpm-shadow', timeoutMs: 1_000, },);
            expect(answer.usable,).toBe(false,);
            expect(answer.usable ? '' : answer.reason,).toContain('ENOENT',);
          },
        },),
      ],
    },),
    describe({
      name: publishStaged.name,
      children: [
        it({
          name: 'uses the good copy another task put at the version\'s name between the first look and the rename, and throws nothing',
          fn: async () => {
            using directory = makeDirectory();
            const toolDirectory = join(directory.path, 'tool',);
            const staging = join(toolDirectory, `.${LOCKED_VERSION}.staging.1`,);
            const final = join(toolDirectory, LOCKED_VERSION,);
            const aside = join(toolDirectory, `.${LOCKED_VERSION}.replaced.1`,);
            installFolder({ toolDirectory, name: `.${LOCKED_VERSION}.staging.1`, prints: LOCKED_VERSION, finished: true, },);

            // The other task's rename wins: its good folder is at the name when this task's rename fails with ENOTEMPTY.
            await publishStaged({
              staging,
              final,
              version: LOCKED_VERSION,
              aside,
              move: async function otherTaskWins() {
                installFolder({ toolDirectory, name: LOCKED_VERSION, prints: LOCKED_VERSION, finished: true, },);
                throw Object.assign(new Error('ENOTEMPTY: directory not empty',), { code: 'ENOTEMPTY', },);
              },
            },);

            expect(
              existsSync(join(final, COMPLETE_MARKER,),),
            ).toBe(true,);
            expect(existsSync(staging,),).toBe(false,);
            expect(existsSync(aside,),).toBe(false,);
          },
        },),
        it({
          name: 'throws MpmShadowLockError and puts a moved-aside copy back when the rename fails and nothing good is at the name',
          fn: async () => {
            using directory = makeDirectory();
            const toolDirectory = join(directory.path, 'tool',);
            const staging = join(toolDirectory, `.${LOCKED_VERSION}.staging.1`,);
            const final = join(toolDirectory, LOCKED_VERSION,);
            const aside = join(toolDirectory, `.${LOCKED_VERSION}.replaced.1`,);
            installFolder({ toolDirectory, name: `.${LOCKED_VERSION}.staging.1`, prints: LOCKED_VERSION, finished: true, },);
            // An unfinished copy sits at the name, so it is moved aside before the rename that fails.
            installFolder({ toolDirectory, name: LOCKED_VERSION, prints: STALE_VERSION, finished: false, },);
            writeFileSync(join(final, 'sentinel',), 'kept',);

            const outcome = await publishStaged({
              staging,
              final,
              version: LOCKED_VERSION,
              aside,
              move: async function failsOnTheSecondMove(from: string, to: string,) {
                if (from === staging)
                  throw new Error('EXDEV: cross-device link not permitted',);
                await rename(from, to,);
              },
            },).then(function resolved() {
              return 'resolved';
            }, function failed(error: unknown,) {
              return error;
            },);

            expect(outcome,).toBeInstanceOf(MpmShadowLockError,);
            expect(
              existsSync(join(final, 'sentinel',),),
            ).toBe(true,);
            expect(existsSync(staging,),).toBe(false,);
          },
        },),
        it({
          name: 'says where the moved-aside copy is when it cannot be put back either',
          fn: async () => {
            using directory = makeDirectory();
            const toolDirectory = join(directory.path, 'tool',);
            const staging = join(toolDirectory, `.${LOCKED_VERSION}.staging.1`,);
            const final = join(toolDirectory, LOCKED_VERSION,);
            const aside = join(toolDirectory, `.${LOCKED_VERSION}.replaced.1`,);
            installFolder({ toolDirectory, name: `.${LOCKED_VERSION}.staging.1`, prints: LOCKED_VERSION, finished: true, },);
            installFolder({ toolDirectory, name: LOCKED_VERSION, prints: STALE_VERSION, finished: false, },);

            const outcome = await publishStaged({
              staging,
              final,
              version: LOCKED_VERSION,
              aside,
              move: async function onlyTheFirstMoveWorks(from: string, to: string,) {
                if (from !== final)
                  throw new Error('EXDEV: cross-device link not permitted',);
                await rename(from, to,);
              },
            },).then(function resolved() {
              return 'resolved';
            }, function failed(error: unknown,) {
              return error;
            },);

            expect(outcome,).toBeInstanceOf(MpmShadowLockError,);
            expect(outcome instanceof MpmShadowLockError ? outcome.message : '',).toContain(`the copy that was there is at ${aside} and could not be put back`,);
            expect(existsSync(join(aside, COMPLETE_MARKER,),) || existsSync(join(aside, 'pkg',),),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: 'command-line entry: which copy runs',
      children: [
        it({
          name: 'ignores a stale link and the PATH copy, fetches the lock\'s archive once, checks it, installs it where mpm-shadow install does, and runs it',
          fn: async () => {
            using directory = makeDirectory();
            await using world = await fetchingWorld({ directory: directory.path, staleLink: true, },);
            const { root, pathDirectory, dataHome, toolDirectory, served, bytes, } = world;

            const run = await runEntry({ root, pathDirectory, dataHome, args: ['install', '--frozen-lockfile',], environment: {}, },);

            expect(run.exitCode,).toBe(0,);
            // Standard output is the locked program's alone.
            expect(run.stdout.trim(),).toBe(`locked ${LOCKED_VERSION} ran: install --frozen-lockfile`,);
            expect(run.stderr,).toContain(`names mpm-shadow ${LOCKED_VERSION} and no installed copy is at that version or above`,);
            expect(run.stderr,).toContain(`is mpm-shadow ${STALE_VERSION}, older than the lock's ${LOCKED_VERSION}, and never runs`,);
            expect(run.stderr,).toContain(`fetched and verified mpm-shadow ${LOCKED_VERSION} (SHA-256 ${sha256Hex(bytes,)})`,);
            // The layout of mpm-shadow's own archive route: version folder with the marker, and the current link.
            expect(readFileSync(join(toolDirectory, LOCKED_VERSION, COMPLETE_MARKER,), 'utf8',),).toBe(`mpm-shadow ${LOCKED_VERSION}\n`,);
            expect(
              existsSync(join(toolDirectory, LOCKED_VERSION, `monochromatic-mpm-shadow-${LOCKED_VERSION}-test`, 'mpm-shadow',),),
            ).toBe(true,);
            expect(
              readlinkSync(join(toolDirectory, CURRENT_LINK_NAME,),),
            ).toBe(LOCKED_VERSION,);
            expect(readdirSync(toolDirectory,).toSorted(),).toEqual([LOCKED_VERSION, CURRENT_LINK_NAME,],);
            expect(served.requests.length,).toBe(1,);
          },
        },),
        it({
          name: 'finds the installed copy on the next run and downloads nothing again',
          fn: async () => {
            using directory = makeDirectory();
            await using world = await fetchingWorld({ directory: directory.path, staleLink: true, },);
            const { root, pathDirectory, dataHome, served, } = world;

            await runEntry({ root, pathDirectory, dataHome, args: ['list',], environment: {}, },);
            const second = await runEntry({ root, pathDirectory, dataHome, args: ['list', '--global',], environment: {}, },);

            expect(second.exitCode,).toBe(0,);
            expect(second.stdout.trim(),).toBe(`locked ${LOCKED_VERSION} ran: list --global`,);
            expect(second.stderr,).toBe('',);
            expect(served.requests.length,).toBe(1,);
          },
        },),
        it({
          name: 'runs the exposure link without any download when it is at the locked version',
          fn: async () => {
            using directory = makeDirectory();
            await using world = await fetchingWorld({ directory: directory.path, staleLink: false, },);
            const { root, pathDirectory, dataHome, served, } = world;
            writeExecutable({ path: join(root, TOOLCHAIN_BIN_LINK,), content: lockedStub(LOCKED_VERSION,), },);

            const run = await runEntry({ root, pathDirectory, dataHome, args: ['bin', '--json',], environment: {}, },);

            expect(run.exitCode,).toBe(0,);
            expect(run.stdout.trim(),).toBe(`locked ${LOCKED_VERSION} ran: bin --json`,);
            expect(run.stderr,).toBe('',);
            expect(served.requests.length,).toBe(0,);
          },
        },),
        it({
          name: 'runs a newer exposure link than the lock, with the note that names the upgrade command, and downloads nothing',
          fn: async () => {
            using directory = makeDirectory();
            await using world = await fetchingWorld({ directory: directory.path, staleLink: false, },);
            const { root, pathDirectory, dataHome, served, } = world;
            writeExecutable({ path: join(root, TOOLCHAIN_BIN_LINK,), content: lockedStub('0.2.3',), },);

            const run = await runEntry({ root, pathDirectory, dataHome, args: ['list',], environment: {}, },);

            expect(run.exitCode,).toBe(0,);
            expect(run.stdout.trim(),).toBe('locked 0.2.3 ran: list',);
            expect(run.stderr,).toContain(noteNaming({ root, usedVersion: '0.2.3', },),);
            expect(UPGRADE_COMMAND,).toBe('mise run toolchain:update mpm-shadow',);
            expect(served.requests.length,).toBe(0,);
          },
        },),
        it({
          name: 'names the lock file it compared, so a reader can tell which worktree is out of date',
          fn: async () => {
            using directory = makeDirectory();
            await using world = await fetchingWorld({ directory: directory.path, staleLink: false, },);
            const { root, pathDirectory, dataHome, } = world;
            writeExecutable({ path: join(root, TOOLCHAIN_BIN_LINK,), content: lockedStub('0.2.3',), },);

            const run = await runEntry({ root, pathDirectory, dataHome, args: ['list',], environment: {}, },);

            expect(run.stderr,).toContain(
              `locked at ${LOCKED_VERSION} in ${join(root, 'toolchain.lock.json',)}, 0.2.3 runs;`,
            );
          },
        },),
        it({
          name: 'runs the newest finished version folder of the toolchain root over a stale link, with the note, and downloads nothing',
          fn: async () => {
            using directory = makeDirectory();
            await using world = await fetchingWorld({ directory: directory.path, staleLink: true, },);
            const { root, pathDirectory, dataHome, toolDirectory, served, } = world;
            installFolder({ toolDirectory, name: '0.2.3', prints: '0.2.3', finished: true, },);
            installFolder({ toolDirectory, name: '0.10.0', prints: '0.10.0', finished: true, },);
            installFolder({ toolDirectory, name: '0.9.0', prints: '0.9.0', finished: true, },);

            const run = await runEntry({ root, pathDirectory, dataHome, args: ['outdated',], environment: {}, },);

            expect(run.exitCode,).toBe(0,);
            expect(run.stdout.trim(),).toBe('locked 0.10.0 ran: outdated',);
            expect(run.stderr,).toContain(noteNaming({ root, usedVersion: '0.10.0', },),);
            expect(served.requests.length,).toBe(0,);
          },
        },),
        it({
          name: 'finds a finished copy that sits only in the machine-global toolchain root, and downloads nothing',
          fn: async () => {
            using directory = makeDirectory();
            await using world = await fetchingWorld({ directory: directory.path, staleLink: true, },);
            const { root, pathDirectory, dataHome, globalToolDirectory, served, } = world;
            installFolder({ toolDirectory: globalToolDirectory, name: '0.2.3', prints: '0.2.3', finished: true, },);

            const run = await runEntry({ root, pathDirectory, dataHome, args: ['list',], environment: {}, },);

            expect(run.exitCode,).toBe(0,);
            expect(run.stdout.trim(),).toBe('locked 0.2.3 ran: list',);
            expect(run.stderr,).toContain(noteNaming({ root, usedVersion: '0.2.3', },),);
            expect(served.requests.length,).toBe(0,);
          },
        },),
        it({
          name: 'treats both toolchain roots as one set: the newest copy wins wherever it is',
          fn: async () => {
            using directory = makeDirectory();
            await using world = await fetchingWorld({ directory: directory.path, staleLink: true, },);
            const { root, pathDirectory, dataHome, toolDirectory, globalToolDirectory, served, } = world;
            installFolder({ toolDirectory, name: '0.2.3', prints: '0.2.3', finished: true, },);
            installFolder({ toolDirectory: globalToolDirectory, name: '0.3.0', prints: '0.3.0', finished: true, },);

            const run = await runEntry({ root, pathDirectory, dataHome, args: ['list',], environment: {}, },);

            expect(run.stdout.trim(),).toBe('locked 0.3.0 ran: list',);
            expect(served.requests.length,).toBe(0,);
          },
        },),
        it({
          name: 'never runs a copy older than the lock: fetches the lock\'s archive, then keeps only the newest version folder',
          fn: async () => {
            using directory = makeDirectory();
            await using world = await fetchingWorld({ directory: directory.path, staleLink: true, },);
            const { root, pathDirectory, dataHome, toolDirectory, served, } = world;
            installFolder({ toolDirectory, name: '0.2.1', prints: '0.2.1', finished: true, },);

            const run = await runEntry({ root, pathDirectory, dataHome, args: ['install',], environment: {}, },);

            expect(run.exitCode,).toBe(0,);
            expect(run.stdout.trim(),).toBe(`locked ${LOCKED_VERSION} ran: install`,);
            expect(served.requests.length,).toBe(1,);
            expect(readdirSync(toolDirectory,).toSorted(),).toEqual([LOCKED_VERSION, CURRENT_LINK_NAME,],);
            expect(run.stderr,).toContain('removed the older mpm-shadow 0.2.1; only the newest is kept',);
          },
        },),
        it({
          name: 'passes the program exit status on unchanged',
          fn: async () => {
            using directory = makeDirectory();
            await using world = await fetchingWorld({ directory: directory.path, staleLink: false, },);
            const { root, pathDirectory, dataHome, } = world;

            const run = await runEntry({ root, pathDirectory, dataHome, args: ['outdated',], environment: { STUB_EXIT: '7', }, },);

            expect(run.exitCode,).toBe(7,);
            expect(run.stdout.trim(),).toBe(`locked ${LOCKED_VERSION} ran: outdated`,);
          },
        },),
        it({
          name: 'uses the mpm-shadow on PATH with a note when the lock has no archive row for this platform, no copy is installed, and it is at the lock\'s version',
          fn: async () => {
            using directory = makeDirectory();
            const { root, pathDirectory, dataHome, } = makeRepository({
              directory: directory.path,
              options: { url: 'http://127.0.0.1:1/none', sha256: 'unused', row: false, staleLink: false, },
            },);

            const run = await runEntry({ root, pathDirectory, dataHome, args: ['list',], environment: { PATH_STUB_VERSION: LOCKED_VERSION, }, },);

            expect(run.exitCode,).toBe(99,);
            expect(run.stdout.trim(),).toBe('ran from PATH: list',);
            expect(run.stderr,).toContain(`has no mpm-shadow archive for ${PLATFORM_ID}`,);
            expect(run.stderr,).toContain(`so the mpm-shadow on PATH (${LOCKED_VERSION}) is used`,);
          },
        },),
        it({
          name: 'runs a newer mpm-shadow on PATH on such a platform with the upgrade note',
          fn: async () => {
            using directory = makeDirectory();
            const { root, pathDirectory, dataHome, } = makeRepository({
              directory: directory.path,
              options: { url: 'http://127.0.0.1:1/none', sha256: 'unused', row: false, staleLink: false, },
            },);

            const run = await runEntry({ root, pathDirectory, dataHome, args: ['list',], environment: { PATH_STUB_VERSION: '0.2.9', }, },);

            expect(run.exitCode,).toBe(99,);
            expect(run.stderr,).toContain(noteNaming({ root, usedVersion: '0.2.9', },),);
          },
        },),
      ],
    },),
    describe({
      name: 'command-line entry: what is refused',
      children: [
        it({
          name: 'refuses an archive whose SHA-256 differs from the lock, installs nothing, runs nothing, and prints one line',
          fn: async () => {
            using directory = makeDirectory();
            const bytes = buildArchive({ directory: directory.path, version: LOCKED_VERSION, },);
            await using served = await serve(bytes,);
            const { root, pathDirectory, dataHome, toolDirectory, } = makeRepository({
              directory: directory.path,
              options: { url: served.url, sha256: sha256Hex(new Uint8Array([1, 2, 3,],),), row: true, staleLink: true, },
            },);

            const run = await runEntry({ root, pathDirectory, dataHome, args: ['install',], environment: {}, },);

            expect(run.exitCode,).toBe(1,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toContain(`has SHA-256 ${sha256Hex(bytes,)}, but toolchain.lock.json records ${sha256Hex(new Uint8Array([1, 2, 3,],),)}`,);
            // A reported failure is one line, not a stack trace.
            expect(run.stderr,).not.toContain('\n    at ',);
            expect(existsSync(toolDirectory,),).toBe(false,);
          },
        },),
        it({
          name: 'refuses an archive whose program prints another version than the lock names, and leaves nothing at the version\'s name',
          fn: async () => {
            using directory = makeDirectory();
            const bytes = buildArchive({ directory: directory.path, version: '0.2.1', },);
            await using served = await serve(bytes,);
            const { root, pathDirectory, dataHome, toolDirectory, } = makeRepository({
              directory: directory.path,
              options: { url: served.url, sha256: sha256Hex(bytes,), row: true, staleLink: false, },
            },);

            const run = await runEntry({ root, pathDirectory, dataHome, args: ['install',], environment: {}, },);

            expect(run.exitCode,).toBe(1,);
            expect(run.stderr,).toContain(`prints version 0.2.1, but toolchain.lock.json names ${LOCKED_VERSION}`,);
            expect(readdirSync(toolDirectory,),).toEqual([],);
          },
        },),
        it({
          name: 'does not run a finished-looking copy at the locked version\'s name that prints another version: replaces it with the verified archive',
          fn: async () => {
            using directory = makeDirectory();
            await using world = await fetchingWorld({ directory: directory.path, staleLink: false, },);
            const { root, pathDirectory, dataHome, toolDirectory, served, } = world;
            installFolder({ toolDirectory, name: LOCKED_VERSION, prints: STALE_VERSION, finished: true, },);

            const run = await runEntry({ root, pathDirectory, dataHome, args: ['install',], environment: {}, },);

            expect(run.exitCode,).toBe(0,);
            expect(run.stdout.trim(),).toBe(`locked ${LOCKED_VERSION} ran: install`,);
            expect(served.requests.length,).toBe(1,);
            expect(
              existsSync(join(toolDirectory, LOCKED_VERSION, 'pkg',),),
            ).toBe(false,);
            expect(readdirSync(toolDirectory,).toSorted(),).toEqual([LOCKED_VERSION, CURRENT_LINK_NAME,],);
          },
        },),
        it({
          name: 'replaces an unfinished folder at the locked version\'s name (no marker) with the verified archive',
          fn: async () => {
            using directory = makeDirectory();
            await using world = await fetchingWorld({ directory: directory.path, staleLink: false, },);
            const { root, pathDirectory, dataHome, toolDirectory, } = world;
            installFolder({ toolDirectory, name: LOCKED_VERSION, prints: STALE_VERSION, finished: false, },);

            const run = await runEntry({ root, pathDirectory, dataHome, args: ['install',], environment: {}, },);

            expect(run.exitCode,).toBe(0,);
            expect(run.stdout.trim(),).toBe(`locked ${LOCKED_VERSION} ran: install`,);
            expect(
              existsSync(join(toolDirectory, LOCKED_VERSION, COMPLETE_MARKER,),),
            ).toBe(true,);
            expect(
              existsSync(join(toolDirectory, LOCKED_VERSION, 'pkg',),),
            ).toBe(false,);
          },
        },),
        it({
          name: 'removes the hidden staging folder a killed task left (gone process, older than the stale age), and says so',
          fn: async () => {
            using directory = makeDirectory();
            await using world = await fetchingWorld({ directory: directory.path, staleLink: false, },);
            const { root, pathDirectory, dataHome, toolDirectory, } = world;
            // A process id that is certainly gone: a child that has already exited.
            const gone = spawnSync('true',).pid;
            const killed = join(toolDirectory, `.${LOCKED_VERSION}.staging.${String(gone,)}`,);
            mkdirSync(join(killed, 'half',), { recursive: true, },);
            const olderThanTheStaleAge = new Date(Date.now() - (2 * STALE_LEFTOVER_AGE_MS),);
            utimesSync(killed, olderThanTheStaleAge, olderThanTheStaleAge,);
            // And one that is alive: this test process, also old.
            const alive = join(toolDirectory, `.0.1.0.staging.${String(process.pid,)}`,);
            mkdirSync(alive, { recursive: true, },);
            utimesSync(alive, olderThanTheStaleAge, olderThanTheStaleAge,);

            const run = await runEntry({ root, pathDirectory, dataHome, args: ['install',], environment: {}, },);

            expect(run.exitCode,).toBe(0,);
            expect(existsSync(killed,),).toBe(false,);
            expect(existsSync(alive,),).toBe(true,);
            expect(run.stderr,).toContain('left by a task that was killed',);
          },
        },),
        it({
          name: 'leaves a recent leftover even when its process id is gone here, because it can belong to a live task in another namespace',
          fn: async () => {
            using directory = makeDirectory();
            await using world = await fetchingWorld({ directory: directory.path, staleLink: false, },);
            const { root, pathDirectory, dataHome, toolDirectory, } = world;
            const gone = spawnSync('true',).pid;
            const recent = join(toolDirectory, `.${LOCKED_VERSION}.staging.${String(gone,)}`,);
            mkdirSync(join(recent, 'half',), { recursive: true, },);

            const run = await runEntry({ root, pathDirectory, dataHome, args: ['install',], environment: {}, },);

            expect(run.exitCode,).toBe(0,);
            expect(existsSync(recent,),).toBe(true,);
            expect(run.stderr,).not.toContain('left by a task that was killed',);
          },
        },),
        it({
          name: 'refuses the mpm-shadow on PATH when it is older than the lock on a platform with no archive row, and runs nothing',
          fn: async () => {
            using directory = makeDirectory();
            const { root, pathDirectory, dataHome, } = makeRepository({
              directory: directory.path,
              options: { url: 'http://127.0.0.1:1/none', sha256: 'unused', row: false, staleLink: true, },
            },);

            const run = await runEntry({ root, pathDirectory, dataHome, args: ['list',], environment: { PATH_STUB_VERSION: STALE_VERSION, }, },);

            expect(run.exitCode,).toBe(1,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toContain(`the mpm-shadow on PATH is ${STALE_VERSION}, older than the lock's ${LOCKED_VERSION}, which never runs`,);
            expect(run.stderr,).not.toContain('\n    at ',);
          },
        },),
        it({
          name: 'fails with one line when the lock has no mpm-shadow tool',
          fn: async () => {
            using directory = makeDirectory();
            const { root, pathDirectory, dataHome, } = makeRepository({
              directory: directory.path,
              options: { url: 'http://127.0.0.1:1/none', sha256: 'unused', row: false, staleLink: false, },
            },);
            writeFileSync(join(root, 'toolchain.lock.json',), '{"tools":{}}',);

            const run = await runEntry({ root, pathDirectory, dataHome, args: ['list',], environment: {}, },);

            expect(run.exitCode,).toBe(1,);
            expect(run.stderr,).toContain('has no "mpm-shadow" tool',);
            expect(run.stderr,).not.toContain('\n    at ',);
          },
        },),
      ],
    },),
  ],
},);
