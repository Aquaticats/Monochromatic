/**
 Command helpers for disposable bypass integration test.
 
 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  cp,
  mkdir,
  mkdtemp,
  rm,
  symlink,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { text, } from 'node:stream/consumers';
import { fileURLToPath, } from 'node:url';

/**
 Result from allowed-failure integration command.
 */
export type FixtureCommandResult = {
  readonly exitCode: number;
  readonly stderr: string;
  readonly stdout: string;
};

/**
 Persisted state fields consumed by integration checks.
 */
export type FixtureBypassState = {
  readonly preference: number;
  readonly table: number;
};

/**
 Disposable network namespace fixture.
 */
export type BypassFixture = {
  readonly namespace: string;
  readonly hostPrimary: string;
  readonly hostSecondary: string;
  readonly peerPrimary: string;
  readonly peerSecondary: string;
  readonly stateDirectory: string;
  readonly statePath: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Package directory, whose `.cache` holds staged copies of the built CLI.
 Staging inside the package keeps the bundle's bare imports resolvable through the package's `node_modules`.
 */
const PACKAGE_DIRECTORY = fileURLToPath(new URL(
  '..',
  import.meta.url,
),);

/**
 Staged copy of the built CLI paired with a harmless watcher.
 */
export type StagedCli = {
  /**
   Bundle entry point to run.
   */
  readonly cliPath: string;

  /**
   Removes the staged copy.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Copies the built CLI into a disposable tree whose paired Rust watcher is `/usr/bin/true`.

 The CLI finds its watcher only next to its own package, so the integration test cannot name a watcher
 through the environment, which would be a switch for the process policy in production too.
 A staged tree puts the stand-in where the CLI looks, and leaves the real watcher untouched.

 @returns Entry point and a disposer that removes the tree.

 @example
 ```ts
 await using staged = await stageWgQuickerCli();
 ```
 */
export async function stageWgQuickerCli(): Promise<StagedCli> {
  /**
   Parent directory of the staged tree.
   */
  const cacheDirectory = join(
    PACKAGE_DIRECTORY,
    '.cache',
    'integration-stage',
  );
  await mkdir(
    cacheDirectory,
    { recursive: true, },
  );
  /**
   Root of this staged tree, laid out as `package/wg/<package>`.
   */
  const root = await mkdtemp(join(
    cacheDirectory,
    'cli-',
  ),);
  /**
   Staged copy of the TypeScript package's built output.
   */
  const bundleDirectory = join(
    root,
    'package',
    'wg',
    'cli-quicker',
    'dist',
    'final',
    'node',
  );
  await mkdir(
    bundleDirectory,
    { recursive: true, },
  );
  await cp(
    join(
      PACKAGE_DIRECTORY,
      'dist',
      'final',
      'node',
    ),
    bundleDirectory,
    { recursive: true, },
  );
  /**
   Where the CLI looks for the paired release watcher.
   */
  const watcherDirectory = join(
    root,
    'package',
    'wg',
    'cli-quicker-exempt',
    'target',
    'release',
  );
  await mkdir(
    watcherDirectory,
    { recursive: true, },
  );
  await symlink(
    '/usr/bin/true',
    join(
      watcherDirectory,
      'wg-quicker-exempt',
    ),
  );
  return {
    cliPath: join(
      bundleDirectory,
      'index.mjs',
    ),
    [Symbol.asyncDispose]: async function removeStage(): Promise<void> {
      await rm(
        root,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 Built bypass bundle exercised by namespace child.
 */
const BYPASS_BUNDLE_URL = new URL(
  '../dist/final/node/tunnel-bypass.mjs',
  import.meta.url,
).href;

/**
 Runs privileged command and returns stdout.
 
 @param args - Arguments after `sudo`.
 
 @returns Captured UTF-8 stdout.
 
 @example
 ```ts
 await runSudo({ args: ['ip', 'netns', 'list'] });
 ```
 */
export async function runSudo(
  { args, }: { readonly args: readonly string[]; },
): Promise<string> {
  /**
   Fresh argument container sharing only immutable strings.
   */
  const isolatedArgs = args.map(function copyArgument(value,): string {
    return value;
  },);
  /**
   Captured command result checked at privilege boundary.
   */
  const result = await runSudoAllowingFailure({ args: isolatedArgs, },);
  if (result.exitCode !== 0) {
    throw new Error([
      `sudo ${isolatedArgs.join(' ',)} exited ${String(result.exitCode,)}.`,
      result.stdout,
      result.stderr,
    ].filter(Boolean,)
      .join('\n',),);
  }
  return result.stdout;
}

/**
 Runs privileged command while capturing nonzero result.
 
 @param args - Arguments after `sudo`.
 
 @returns Exit code and output streams.
 
 @example
 ```ts
 await runSudoAllowingFailure({ args: ['ip', 'netns', 'delete', 'missing'] });
 ```
 */
export async function runSudoAllowingFailure(
  { args, }: { readonly args: readonly string[]; },
): Promise<FixtureCommandResult> {
  /**
   Fresh argument container sharing only immutable strings.
   */
  const isolatedArgs = args.map(function copyArgument(value,): string {
    return value;
  },);
  /**
   Child with piped output and inherited execution environment.
   */
  const child = spawn(
    'sudo',
    isolatedArgs,
    {
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  if ((child.stdout === null) || (child.stderr === null))
    throw new Error('Fixture command did not expose output streams.',);
  /**
   Concurrent stdout collection preventing pipe backpressure.
   */
  const stdoutPromise = text(child.stdout,);
  /**
   Concurrent stderr collection preventing pipe backpressure.
   */
  const stderrPromise = text(child.stderr,);
  await once(
    child,
    'close',
  );
  /**
   Exit code available after close event.
   */
  const { exitCode, } = child;
  if (exitCode === null)
    throw new Error('Fixture command closed without an exit code.',);
  /**
   Captured streams complete after child closes.
   */
  const [stdout, stderr,] = await Promise.all([
    stdoutPromise,
    stderrPromise,
  ],);
  return {
    exitCode,
    stdout,
    stderr,
  };
}

/**
 Writes root-owned fixture file without shell interpolation.
 
 @param path - Destination path.
 
 @param contents - UTF-8 fixture contents.
 
 @example
 ```ts
 await writeRootFixtureFile({ path: '/tmp/state', contents: '{}' });
 ```
 */
export async function writeRootFixtureFile(
  {
    path,
    contents,
  }: {
    readonly path: string;
    readonly contents: string;
  },
): Promise<void> {
  await runSudo({
    args: [
      process.execPath,
      '--input-type=module',
      '--eval',
      "const { writeFile } = await import('node:fs/promises'); await writeFile(process.argv[1], Buffer.from(process.argv[2], 'base64'));",
      path,
      Buffer.from(
        contents,
        'utf8',
      )
        .toString('base64',),
    ],
  },);
}

/**
 Runs `ip` command inside fixture namespace.
 
 @param fixture - Namespace fixture.
 
 @param args - Arguments after `ip`.
 
 @returns Captured stdout.
 
 @example
 ```ts
 await runNamespaceIp({ fixture, args: ['route', 'show'] });
 ```
 */
export async function runNamespaceIp(
  {
    fixture,
    args,
  }: {
    readonly fixture: BypassFixture;
    readonly args: readonly string[];
  },
): Promise<string> {
  return await runSudo({
    args: [
      'ip',
      'netns',
      'exec',
      fixture.namespace,
      'ip',
      ...args,
    ],
  },);
}

/**
 Runs built CLI command inside fixture namespace.
 
 @param fixture - Namespace fixture.
 
 @param operation - `up` or `down` command.
 
 @param configPath - Explicit config path.
 
 @returns Successful command result with both output streams.
 
 @example
 ```ts
 await runWgQuickerCli({ fixture, operation: 'up', configPath: '/tmp/wgtest.conf' });
 ```
 */
export async function runWgQuickerCli(
  {
    fixture,
    operation,
    configPath,
  }: {
    readonly fixture: BypassFixture;
    readonly operation: 'down' | 'up';
    readonly configPath: string;
  },
): Promise<FixtureCommandResult> {
  /**
   Staged copy of the built CLI whose paired watcher is a harmless stand-in.
   */
  await using staged = await stageWgQuickerCli();
  /**
   Captured built CLI result checked before return.
   The watched user is the one `sudo` reports through `SUDO_UID`.
   */
  const result = await runSudoAllowingFailure({
    args: [
      'ip',
      'netns',
      'exec',
      fixture.namespace,
      'env',
      `WG_QUICKER_RUNTIME_DIRECTORY=${fixture.stateDirectory}`,
      process.execPath,
      staged.cliPath,
      operation,
      configPath,
    ],
  },);
  if (result.exitCode !== 0) {
    throw new Error([
      `wg-quicker ${operation} exited ${String(result.exitCode,)}.`,
      result.stdout,
      result.stderr,
    ].filter(Boolean,)
      .join('\n',),);
  }
  return result;
}

/**
 Runs built bypass operation inside fixture namespace.
 
 @param fixture - Namespace fixture.
 
 @param source - JavaScript operation after bundle import.
 
 @returns Captured stdout.
 
 @example
 ```ts
 await runBypassOperation({ fixture, source: 'await removeExemptRule(...);' });
 ```
 */
export async function runBypassOperation(
  {
    fixture,
    source,
  }: {
    readonly fixture: BypassFixture;
    readonly source: string;
  },
): Promise<string> {
  return await runSudo({
    args: [
      'ip',
      'netns',
      'exec',
      fixture.namespace,
      'env',
      `WG_QUICKER_RUNTIME_DIRECTORY=${fixture.stateDirectory}`,
      process.execPath,
      '--input-type=module',
      '--eval',
      `const { addExemptRule, claimBypassAllocationOperation, claimBypassInterfaceOperation, readBypassStatePath, removeExemptRule, synchronizeBypassRoutes } = await import('${BYPASS_BUNDLE_URL}'); ${source}`,
    ],
  },);
}

/**
 Runs built bypass operation expected to fail.
 
 @param fixture - Namespace fixture.
 
 @param source - JavaScript operation body.
 
 @returns Captured nonzero result.
 
 @example
 ```ts
 await runBypassOperationAllowingFailure({ fixture, source: 'await addExemptRule(...);' });
 ```
 */
export async function runBypassOperationAllowingFailure(
  {
    fixture,
    source,
  }: {
    readonly fixture: BypassFixture;
    readonly source: string;
  },
): Promise<FixtureCommandResult> {
  return await runSudoAllowingFailure({
    args: [
      'ip',
      'netns',
      'exec',
      fixture.namespace,
      'env',
      `WG_QUICKER_RUNTIME_DIRECTORY=${fixture.stateDirectory}`,
      process.execPath,
      '--input-type=module',
      '--eval',
      `const { addExemptRule, claimBypassAllocationOperation, claimBypassInterfaceOperation, readBypassStatePath, removeExemptRule, synchronizeBypassRoutes } = await import('${BYPASS_BUNDLE_URL}'); ${source}`,
    ],
  },);
}

/**
 Reads persisted bypass state through privileged boundary.
 
 @param fixture - Namespace fixture.
 
 @returns Parsed table field.
 
 @example
 ```ts
 await readFixtureState({ fixture });
 ```
 */
export async function readFixtureState(
  { fixture, }: { readonly fixture: BypassFixture; },
): Promise<FixtureBypassState> {
  /**
   Parsed state JSON.
   */
  const value: unknown = JSON.parse(await runSudo({
    args: [
      'cat',
      fixture.statePath,
    ],
  },),);
  if (((typeof value) !== 'object')
    || (value === null)
    || (!('preference' in value))
    || (!('table' in value))
    || ((typeof value.preference) !== 'number')
    || ((typeof value.table) !== 'number')) {
    throw new Error('Fixture bypass state lacks numeric table or preference.',);
  }
  return {
    preference: value.preference,
    table: value.table,
  };
}
