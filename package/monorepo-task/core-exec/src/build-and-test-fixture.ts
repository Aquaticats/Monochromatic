/**
 Disposable process boundary for generated mise task regression tests. @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdir,
  mkdtempDisposable,
  readFile,
  realpath,
  stat,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import {
  delimiter,
  dirname,
  join,
} from 'node:path';
import { text, } from 'node:stream/consumers';

import {
  type BuildAndTestFixture,
  type BuildAndTestResult,
  fixtureConfig,
} from './build-and-test-fixture-config.ts';

/**
 Upper bound prevents a broken orchestration fixture from hanging verification.
 */
const CHILD_TIMEOUT_MS = 20_000;

//region Executable resolution: keep ambient credentials and repository tools out of fixtures.

/**
 Resolve mise once from the caller's executable path before isolating child PATH.

 @returns absolute executable path for direct process invocation

 @throws Error when mise cannot be found

 @example
 ```ts
 const mise = await resolveMise();
 ```
 */
async function resolveMise(): Promise<string> {
  /**
   Platform executable name avoids invoking an interactive shell alias.
   */
  const name = process.platform === 'win32' ? 'mise.exe' : 'mise';
  /**
   Each candidate remains associated with its caller-defined search order.
   */
  const candidates = await Promise.all((process.env
    .PATH
    ?? '').split(delimiter,)
    .map(async function candidate(path,): Promise<{
      /**
       Candidate path preserves lookup order independently of file metadata.
       */
      readonly path: string;
      /**
       Only regular-file candidates can supply the mise executable.
       */
      readonly isFile: boolean;
    }> {
    /**
     File metadata distinguishes executable candidates from directories without sync I/O.
     */
      const candidatePath = join(
        path,
        name,
      );
    /**
     Missing PATH entries are expected, not process execution errors.
     */
      const metadata = await stat(
        candidatePath,
        { throwIfNoEntry: false, },
      );
      return {
        path: candidatePath,
        isFile: metadata?.isFile() === true,
      };
    },),);
  /**
   First existing candidate preserves normal PATH precedence.
   */
  const mise = candidates.find(function available(candidate,): boolean {
    return candidate.isFile;
  },);
  if (mise === undefined)
    throw new Error('mise executable is required for buildAndTest regression tests',);
  return await realpath(mise.path,);
}

/**
 Construct a process-local environment without inherited mise settings or credentials.

 @param root - disposable configuration and state root

 @param mise - resolved executable supplying nested mise invocations

 @returns minimal task environment with fixture-owned state directories

 @throws Error when Windows system command discovery lacks SystemRoot

 @example
 ```ts
 fixtureEnvironment({ root: '/tmp/fixture', mise: '/usr/bin/mise', });
 ```
 */
function fixtureEnvironment({
  root,
  mise,
}: {
  readonly root: string;
  readonly mise: string
},): NodeJS.ProcessEnv {
  /**
   Windows command execution needs its measured system directory, not a guessed install path.
   */
  const systemRoot = process.env
    .SystemRoot;
  if ((process.platform === 'win32') && (systemRoot === undefined))
    throw new Error('SystemRoot is required for Windows mise fixtures',);
  /**
   Shell lookup is limited to operating-system directories after mise and Node.
   */
  const systemPaths = systemRoot === undefined ? [
    '/usr/bin',
    '/bin',
  ] : [join(
    systemRoot,
    'System32',
  ),];
  return {
    PATH: [
      dirname(mise,),
      dirname(process.execPath,),
      ...systemPaths,
    ].join(delimiter,),
    ...(systemRoot === undefined ? {} : { SystemRoot: systemRoot, }),
    HOME: join(
      root,
      'home',
    ),
    XDG_CONFIG_HOME: join(
      root,
      'config',
    ),
    XDG_CACHE_HOME: join(
      root,
      'cache',
    ),
    XDG_DATA_HOME: join(
      root,
      'data',
    ),
    XDG_STATE_HOME: join(
      root,
      'state',
    ),
    MISE_CONFIG_DIR: join(
      root,
      'config',
      'mise',
    ),
    MISE_SYSTEM_CONFIG_DIR: join(
      root,
      'system',
    ),
    MISE_GLOBAL_CONFIG_FILE: join(
      root,
      'config',
      'mise',
      'config.toml',
    ),
    MISE_STATE_DIR: join(
      root,
      'state',
      'mise',
    ),
    MISE_CACHE_DIR: join(
      root,
      'cache',
      'mise',
    ),
    MISE_DATA_DIR: join(
      root,
      'data',
      'mise',
    ),
    MISE_CEILING_PATHS: dirname(root,),
    MISE_TRUSTED_CONFIG_PATHS: root,
    MISE_TASK_RUN_AUTO_INSTALL: 'false',
    MISE_JOBS: '1',
    NO_COLOR: '1',
    FIXTURE_EVENTS: join(
      root,
      'events',
    ),
  };
}

//endregion Executable resolution

/**
 Run generated orchestration against fake phase scripts and isolated mise state.

 @param args - arguments forwarded through mise's own usage parser

 @param buildFails - whether fake builds throw after recording execution

 @param packageBuild - whether package discovery finds a build task

 @param prepareFails - whether preparation throws for the tolerant-wrapper control

 @param task - generated root task to invoke

 @param testFails - whether fake tests throw after recording execution

 @returns exit status, diagnostic output, and file-backed phase order

 @throws Error if mise is unavailable or the child cannot exit normally

 @example
 ```ts
 await runBuildAndTestFixture({ args: [], buildFails: true, testFails: false, });
 ```
 */
export async function runBuildAndTestFixture({
  args,
  buildFails,
  packageBuild = true,
  prepareFails = false,
  task = 'buildAndTest',
  testFails,
}: BuildAndTestFixture,): Promise<BuildAndTestResult> {
  /**
   Disposable resource removes configuration and outputs after the child has closed.
   */
  await using directory = await mkdtempDisposable(join(
    tmpdir(),
    'mise-build-and-test-',
  ),);
  /**
   Canonical paths keep mise's directory ceiling effective on symlinked homes.
   */
  const root = await realpath(directory.path,);
  /**
   Resolved executable is shared by outer and nested mise invocations.
   */
  const mise = await resolveMise();
  /**
   Package path exercises monorepo task selection without invoking actual packages.
   */
  const packageRoot = join(
    root,
    'package',
    'fixture',
    'demo',
  );
  await mkdir(
    packageRoot,
    { recursive: true, },
  );
  await writeFile(
    join(
      root,
      'events',
    ),
    '',
  );
  await Promise.all([
    'root-build',
    'package-build',
    'prepare',
    'test',
  ].map(async function writePhase(phase,): Promise<void> {
    /**
     Failure choice is encoded as a boolean literal, never executable caller input.
     */
    const fails = phase === 'test' ? testFails : phase === 'prepare' ? prepareFails : buildFails;
    /**
     JSON encoding preserves the destination JavaScript string grammar.
     */
    const script = `import { appendFile } from 'node:fs/promises';
await appendFile(process.env.FIXTURE_EVENTS, ${JSON.stringify(`${phase}\n`,)});
if (${JSON.stringify(fails,)}) { throw new Error(${JSON.stringify(`fixture ${phase} failed`,)}); }
`;
    await Promise.all([
      writeFile(
        join(
          root,
          `${phase}.ts`,
        ),
        script,
      ),
      writeFile(
        join(
          packageRoot,
          `${phase}.ts`,
        ),
        script,
      ),
      ...(phase === 'test' ? [writeFile(
        join(
          packageRoot,
          'test file.ts',
        ),
        script,
      ),] : []),
    ],);
  },),);
  await Promise.all([
    writeFile(
      join(
        packageRoot,
        'mise.toml',
      ),
      packageBuild ? '[tasks.build]\nrun = "node package-build.ts"\n' : '',
    ),
    writeFile(
      join(
        root,
        'mise.toml',
      ),
      fixtureConfig,
    ),
  ],);
  /**
   Child output is drained concurrently so diagnostics cannot block process completion.
   */
  const child = spawn(
    mise,
    [
      'run',
      task,
      '--',
      ...args,
    ],
    {
      cwd: root,
      timeout: CHILD_TIMEOUT_MS,
      env: fixtureEnvironment({
        root,
        mise,
      },),
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Close waits for process termination and pipe shutdown before disposable cleanup.
   */
  const [stdout, stderr,] = await Promise.all([
    text(child.stdout,),
    text(child.stderr,),
    once(
      child,
      'close',
    ),
  ],);
  /**
   Signal termination and timeout must never masquerade as the expected task exit code.
   */
  const { exitCode, } = child;
  if (exitCode === null)
    throw new Error(`mise fixture did not exit normally (${child.signalCode}): ${stderr}`,);
  /**
   File-backed execution trace cannot be satisfied by mise echoing a command string.
   */
  const events = await readFile(
    join(
      root,
      'events',
    ),
    'utf8',
  );
  return {
    exitCode,
    output: `${stdout}\n${stderr}`,
    events: events.trim()
      .split('\n',),
  };
}
