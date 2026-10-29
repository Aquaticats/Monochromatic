/**
 Bounded container-cell execution shared by the package's matrix runners.

 Every cell runs `podman run --memory=2g --cpus=2 --rm` (AGENTS.md RXI, the
 bounds the rule text names) with subprocess output captured under an explicit
 `maxBuffer` (never unbounded: an unbounded capture drove a matrix driver to
 42 GB RSS on 2026-10-26) and a wall-clock timeout per cell.
 */

import { execFile, } from 'node:child_process';
import {
  readFile,
  realpath,
} from 'node:fs/promises';
import {
  isAbsolute,
  join,
  relative,
  sep,
} from 'node:path';
import { promisify, } from 'node:util';

//region Bounds

/**
 Bytes in one kibibyte.
 */
const BYTES_PER_KIBIBYTE = 1_024;

/**
 Bytes in one mebibyte.
 */
const BYTES_PER_MEBIBYTE: number = BYTES_PER_KIBIBYTE * BYTES_PER_KIBIBYTE;

/**
 Capture cap for one cell's combined output, in mebibytes.
 */
const CAPTURE_CAP_MEBIBYTES = 32;

/**
 Wall-clock cap for one cell, in minutes.
 */
const CELL_TIMEOUT_MINUTES = 20;

/**
 Seconds in one minute.
 */
const SECONDS_PER_MINUTE = 60;

/**
 Milliseconds in one second.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 Explicit capture cap for one cell's combined output.
 Overflow fails the cell loudly instead of growing the driver's heap.
 */
export const MAX_CAPTURED_OUTPUT_BYTES: number = CAPTURE_CAP_MEBIBYTES * BYTES_PER_MEBIBYTE;

/**
 Wall-clock cap for one cell, in milliseconds.
 */
export const CELL_TIMEOUT_MS: number = CELL_TIMEOUT_MINUTES
  * SECONDS_PER_MINUTE
  * MILLISECONDS_PER_SECOND;

/**
 Podman resource flags applied to every container cell.
 */
export const PODMAN_BOUNDS: readonly string[] = [
  '--memory=2g',
  '--cpus=2',
] as const;

//endregion Bounds

//region Cell execution

/**
 Outcome of one bounded subprocess run.
 */
export type BoundedRun = {
  readonly stdout: string;
  readonly stderr: string;
  readonly code: number;
};

/**
 Exit code reported when the capture cap, the timeout, or a failed start
 stopped a command without a numeric exit code of its own.
 */
const STOPPED_COMMAND_EXIT_CODE = 1;

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.

 The generic `promisify(execFile)` form trips `strict-void-return`, because
 Node's overload set includes void-returning callbacks.
 See `doc/troubleshooting/oxlint-promisify-void-return.md`.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 `execFile` with a promise result instead of a callback.
 */
const execFileAsync = promisifyExecFile(execFile,);

/**
 Reads one `execFile` rejection into the outcome shape.

 @param error - Rejection of the promisified `execFile`

 @returns Captured output and exit code; `code` is `1` when the error carries
 no numeric exit code

 @example
 ```ts
 const outcome = readExecFailure(new Error('spawn failed',),);
 ```
 */
function readExecFailure(error: unknown,): BoundedRun {
  if (!Error.isError(error,)) {
    return {
      stdout: '',
      stderr: String(error,),
      code: STOPPED_COMMAND_EXIT_CODE,
    };
  }
  /**
   Captured stdout the failed command produced before it stopped.
   */
  const stdout = ('stdout' in error) && ((typeof error.stdout) === 'string')
    ? error.stdout
    : '';
  /**
   Captured stderr the failed command produced before it stopped.
   */
  const stderr = ('stderr' in error) && ((typeof error.stderr) === 'string')
    ? error.stderr
    : '';
  /**
   Exit code from the error object, or the generic stop code.
   */
  const code = ('code' in error) && ((typeof error.code) === 'number')
    ? error.code
    : STOPPED_COMMAND_EXIT_CODE;
  return {
    stdout,
    stderr: `${stderr}\n${error.message}`,
    code,
  };
}

/**
 Runs one command with an explicit output cap and timeout.

 @param command - Executable to start

 @param args - Arguments passed to the executable

 @param timeoutMs - Wall-clock cap in milliseconds; absent means
 {@link CELL_TIMEOUT_MS}

 @returns Captured output and exit code; `code` is `1` when the capture cap
 or the timeout stopped the command

 @example
 ```ts
 const outcome = await runBounded({ command: 'echo', args: ['hello',], },);
 ```
 */
export async function runBounded(
  {
    command,
    args,
    timeoutMs = CELL_TIMEOUT_MS,
  }: {
    readonly command: string;
    readonly args: readonly string[];
    readonly timeoutMs?: number;
  },
): Promise<BoundedRun> {
  try {
    /**
     Captured output of a command that exited with code 0.
     */
    const completed = await execFileAsync(
      command,
      [...args],
      {
        timeout: timeoutMs,
        maxBuffer: MAX_CAPTURED_OUTPUT_BYTES,
        encoding: 'utf8',
        killSignal: 'SIGKILL',
      },
    );
    return {
      stdout: completed.stdout,
      stderr: completed.stderr,
      code: 0,
    };
  } catch (error) {
    return readExecFailure(error,);
  }
}

/**
 Runs one bounded podman container cell:
 `podman run --memory=2g --cpus=2 --rm` over the mounted worktree.

 The mount skips a SELinux `:Z` relabel (the repository's volume refuses
 label xattrs, `lsetxattr: operation not permitted`) and passes
 `--security-opt label=disable` instead.

 @param image - Fully tagged image reference to run

 @param command - Shell command executed inside the container

 @param monorepoRoot - Worktree root mounted at `/workspace`

 @returns Captured output and exit code of the cell

 @example
 ```ts
 const outcome = await runContainerCell({ image: 'ubuntu:latest', command: 'true', monorepoRoot: '/repo', },);
 ```
 */
export async function runContainerCell(
  {
    image,
    command,
    monorepoRoot,
  }: {
    readonly image: string;
    readonly command: string;
    readonly monorepoRoot: string;
  },
): Promise<BoundedRun> {
  return await runBounded({
    command: 'podman',
    args: [
      'run',
      '--rm',
      ...PODMAN_BOUNDS,
      // label=disable instead of a :Z relabel: the repository sits on a
      // volume where SELinux label xattrs are refused, which aborts the mount
      // setup before any test runs.
      '--security-opt',
      'label=disable',
      '--volume',
      `${monorepoRoot}:/workspace`,
      image,
      'sh',
      '-c',
      command,
    ],
  },);
}

//endregion Cell execution

//region Readable workspace

/**
 Prefix of a pnpm workspace protocol dependency specifier.
 */
const WORKSPACE_PROTOCOL = 'workspace:';

/**
 Raised when `chmod` cannot make a directory the container reads world-readable.
 */
export class ReadabilityHealError extends Error {
  /**
   @param dir - Directory `chmod` failed on

   @param stderr - Diagnostic `chmod` printed
   */
  constructor(
    {
      dir,
      stderr,
    }: {
      readonly dir: string;
      readonly stderr: string;
    },
  ) {
    super(`chmod could not make ${dir} world-readable for the container cells: ${stderr}`,);
    this.name = 'ReadabilityHealError';
  }
}

/**
 Reads the workspace-protocol dependency names one manifest declares.

 @param manifest - Parsed `package.json`

 @param fields - Dependency tables to read; the tested package needs its dev
 dependencies (test harnesses), its dependencies need only what they import

 @returns Package names whose specifier uses `workspace:`

 @example
 ```ts
 workspaceNames({ manifest: { dependencies: { a: 'workspace:*', b: '^1.0.0', }, }, fields: ['dependencies',], },); // ['a']
 ```
 */
function workspaceNames(
  {
    manifest,
    fields,
  }: {
    readonly manifest: unknown;
    readonly fields: readonly string[];
  },
): readonly string[] {
  if (((typeof manifest) !== 'object') || (manifest === null))
    return [];
  return fields.flatMap(function namesIn(field,) {
    /**
     Dependency table under this field, when the manifest has one.
     */
    const table: unknown = Reflect.get(
      manifest,
      field,
    );
    if (((typeof table) !== 'object') || (table === null))
      return [];
    return Object.entries(table,)
      .filter(function isWorkspace([, specifier,],) {
        return ((typeof specifier) === 'string') && specifier.startsWith(WORKSPACE_PROTOCOL,);
      },)
      .map(function nameOf([name,],) {
        return name;
      },);
  },);
}

/**
 Narrows a caught value to the filesystem error for a path that does not exist.

 @param error - Value a filesystem call rejected with

 @returns Whether the value is an `ENOENT` error

 @example
 ```ts
 isMissingPathError(Object.assign(new Error('gone',), { code: 'ENOENT', },),); // true
 ```
 */
function isMissingPathError(error: unknown,): error is Error & { readonly code: 'ENOENT'; } {
  if (!Error.isError(error,))
    return false;
  return Reflect.get(
    error,
    'code',
  ) === 'ENOENT';
}

/**
 Resolves one package's workspace dependencies to their real directories.

 @param dir - Package directory holding `package.json`

 @param monorepoRoot - Highest directory whose `node_modules` is searched

 @param fields - Dependency tables to follow

 @returns Real directories of the linked workspace packages, each found the
 way Node resolves a bare import: the nearest `node_modules/<name>` from the
 package directory up to the monorepo root. pnpm's `dedupeDirectDeps` links a
 dependency the root also declares only at the root, so the package's own
 `node_modules` alone misses it. A dependency with no link anywhere on that
 path is skipped with a warning: the cell resolves imports the same way, so
 it cannot read that package either.

 @example
 ```ts
 await linkedWorkspaceDirs({ dir: '/repo/package/file-enforcer/core', monorepoRoot: '/repo', fields: ['dependencies',], },);
 ```
 */
async function linkedWorkspaceDirs(
  {
    dir,
    monorepoRoot,
    fields,
  }: {
    readonly dir: string;
    readonly monorepoRoot: string;
    readonly fields: readonly string[];
  },
): Promise<readonly string[]> {
  /**
   Directories whose `node_modules` Node would search, nearest first.
   */
  const searchDirs = nodeModulesSearchDirs({
    dir,
    monorepoRoot,
  },);
  /**
   Parsed manifest of the package.
   */
  const manifest: unknown = JSON.parse(await readFile(
    join(
      dir,
      'package.json',
    ),
    'utf8',
  ),);
  /**
   Real directory of each declared link, or an empty list for a missing one.
   */
  const resolved = await Promise.all(
    workspaceNames({
      manifest,
      fields,
    },)
      .map(async function resolveLink(name,): Promise<readonly string[]> {
        /**
         Every candidate link on the search path, probed together.
         */
        const probes = await Promise.allSettled(searchDirs.map(function probe(searchDir,) {
          return realpath(join(
            searchDir,
            'node_modules',
            name,
          ),);
        },),);
        /**
         Nearest probe that resolved or failed for a reason other than absence.
         */
        const decisive = probes.find(function isDecisive(outcome,) {
          return (outcome.status === 'fulfilled') || (!isMissingPathError(outcome.reason,));
        },);
        if (decisive === undefined) {
          console.warn(
            `[container-readability] ${name} is declared in ${dir}/package.json but no node_modules/${name} exists from ${dir} up to ${monorepoRoot}; run pnpm install to link it (skipped)`,
          );
          return [];
        }
        if (decisive.status === 'rejected')
          throw decisive.reason;
        return [decisive.value,];
      },),
  );
  return resolved.flat();
}

/**
 Lists the directories whose `node_modules` Node searches for a bare import
 made from one directory, nearest first, stopping at the monorepo root.

 @param dir - Directory the import is made from

 @param monorepoRoot - Highest directory searched

 @returns `dir`, its ancestors, and `monorepoRoot` last; only `dir` when it
 lies outside the monorepo root. The list length is the path depth, so
 building it is bounded by the number of path segments.

 @example
 ```ts
 nodeModulesSearchDirs({ dir: '/repo/package/a/core', monorepoRoot: '/repo', },);
 // ['/repo/package/a/core', '/repo/package/a', '/repo/package', '/repo']
 ```
 */
function nodeModulesSearchDirs(
  {
    dir,
    monorepoRoot,
  }: {
    readonly dir: string;
    readonly monorepoRoot: string;
  },
): readonly string[] {
  /**
   Path from the monorepo root down to the directory.
   */
  const below = relative(
    monorepoRoot,
    dir,
  );
  if (below.startsWith('..',) || isAbsolute(below,))
    return [dir,];
  /**
   Segments from the root down to the directory.
   */
  const segments = below === ''
    ? []
    : below.split(sep,);
  /**
   Each prefix directory, root first.
   */
  const prefixes = segments.reduce(
    function extend(
      dirs: string[],
      segment: string,
    ): string[] {
      dirs.push(join(
        dirs.at(-1,) ?? monorepoRoot,
        segment,
      ),);
      return dirs;
    },
    [],
  );
  return [
    ...prefixes.toReversed(),
    monorepoRoot,
  ];
}

/**
 Lists the workspace packages one package reaches through `workspace:`
 dependencies, transitively, as monorepo-relative directories.

 The tested package's dev dependencies count (test harnesses run in the
 cell); deeper packages contribute only their runtime dependencies.

 @param monorepoRoot - Worktree root the relative paths start from

 @param packageDir - Absolute directory of the tested package

 @returns Monorepo-relative directories, each listed once

 @example
 ```ts
 await workspaceDependencyDirs({ monorepoRoot: '/repo', packageDir: '/repo/package/file-enforcer/core', },);
 // ['package/logger/core', 'package/test-harness/core', ...]
 ```
 */
export async function workspaceDependencyDirs(
  {
    monorepoRoot,
    packageDir,
  }: {
    readonly monorepoRoot: string;
    readonly packageDir: string;
  },
): Promise<readonly string[]> {
  /**
   Real directory of the tested package.
   */
  const self = await realpath(packageDir,);
  /**
   Real directories already reached, the tested package included.
   */
  const reached = new Set<string>([self,],);
  /**
   Marks a level's first-time directories as reached and reads their
   runtime dependencies in parallel.

   @param frontier - Directories found at this level, possibly repeated

   @returns Directories the next level holds
   */
  async function nextLevel(frontier: readonly string[],): Promise<readonly string[]> {
    /**
     Frontier entries reached for the first time.
     */
    const fresh = [...new Set(frontier,),].filter(function isNew(dir,) {
      return !reached.has(dir,);
    },);
    for (const dir of fresh)
      reached.add(dir,);
    /**
     Dependencies of every fresh directory.
     */
    const next = await Promise.all(fresh.map(function dependenciesOf(dir,) {
      return linkedWorkspaceDirs({
        dir,
        monorepoRoot,
        fields: ['dependencies',],
      },);
    },),);
    return next.flat();
  }
  for (
    let frontier = await linkedWorkspaceDirs({
      dir: packageDir,
      monorepoRoot,
      fields: [
        'dependencies',
        'devDependencies',
      ],
    },);
    frontier.length > 0;
    // oxlint-disable-next-line no-await-in-loop -- each level's manifests name the next level, so levels run in sequence while each level reads in parallel
    frontier = await nextLevel(frontier,)
  );
  reached.delete(self,);
  return [...reached,].map(function toRelative(dir,) {
    return relative(
      monorepoRoot,
      dir,
    );
  },);
}

/**
 Makes the directories a container cell reads world-readable (read and
 traverse bits only, never content). Rootless podman shows the mounted
 worktree as container-root-owned, so a non-root cell reads only
 world-readable files, and agent tooling here creates sources with mode 600.

 @param monorepoRoot - Worktree root the cells mount

 @param packageDir - Absolute directory of the tested package; its transitive
 workspace dependencies are healed whole

 @param ownDirs - Monorepo-relative directories of the tested package the
 cells read

 @returns Monorepo-relative directories that were healed

 @throws ReadabilityHealError when `chmod` fails on a directory

 @example
 ```ts
 await healReadableDirs({ monorepoRoot: '/repo', packageDir: '/repo/package/file-enforcer/core', ownDirs: ['package/file-enforcer/core/src',], },);
 ```
 */
export async function healReadableDirs(
  {
    monorepoRoot,
    packageDir,
    ownDirs,
  }: {
    readonly monorepoRoot: string;
    readonly packageDir: string;
    readonly ownDirs: readonly string[];
  },
): Promise<readonly string[]> {
  /**
   Every directory the cells read.
   */
  const dirs = [
    ...ownDirs,
    ...await workspaceDependencyDirs({
      monorepoRoot,
      packageDir,
    },),
  ];
  await Promise.all(dirs.map(async function heal(dir,) {
    /**
     Outcome of the recursive chmod; symlinks are not followed.
     */
    const outcome = await runBounded({
      command: 'chmod',
      args: [
        '--recursive',
        'a+rX',
        join(
          monorepoRoot,
          dir,
        ),
      ],
    },);
    if (outcome.code !== 0)
      throw new ReadabilityHealError({
        dir,
        stderr: outcome.stderr,
      },);
  },),);
  return dirs;
}

//endregion Readable workspace
