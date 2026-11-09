/**
 Commands that the disposable bypass integration test runs inside its namespace,
 and the reader of the state those commands persist.

 @module
 */

import { stageWgQuickerCli, } from './tunnel-bypass-command-fixture-staging.ts';
import {
  runSudo,
  runSudoAllowingFailure,
} from './tunnel-bypass-command-fixture-sudo.ts';
import type {
  BypassFixture,
  FixtureBypassState,
  FixtureCommandResult,
  StagedCli,
} from './tunnel-bypass-command-fixture-types.ts';

/**
 Built bypass bundle exercised by namespace child.
 */
const BYPASS_BUNDLE_URL = new URL(
  '../dist/final/node/tunnel-bypass.mjs',
  import.meta.url,
).href;

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

 @param staged - Staged copy of the CLI to run.
 A tunnel started by one staged copy can only be stopped by the same copy,
 because the watcher check compares the exact path of the watcher script,
 so a test that brings a tunnel up and takes it down passes one copy to both calls.
 Absent, a fresh copy is staged for this call.

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
    staged: given,
  }: {
    readonly fixture: BypassFixture;
    readonly operation: 'down' | 'up';
    readonly configPath: string;
    readonly staged?: StagedCli;
  },
): Promise<FixtureCommandResult> {
  /**
   Staged copy of the built CLI whose paired watcher is a harmless stand-in,
   the one the caller gave or a fresh one removed at the end of this call.
   */
  await using fresh = given === undefined ? await stageWgQuickerCli() : undefined;
  /**
   Copy this call runs.
   */
  const staged = given ?? fresh;
  if (staged === undefined)
    throw new Error('No staged CLI copy.',);
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
  /**
   Recorded routes, read only when they have the persisted shape.
   */
  const routes = ('routes' in value) && Array.isArray(value.routes,)
    ? value.routes
      .flatMap(function routeOf(route: unknown,): readonly {
        readonly proto: string;
        readonly tokens: readonly string[]
      }[] {
        return ((typeof route) === 'object') && (route !== null)
          && ('proto' in route)
          && ('tokens' in route)
          && ((typeof route.proto) === 'string')
          && Array.isArray(route.tokens,)
          ? [{
            proto: route.proto,
            tokens: route.tokens
              .map(String,),
          },]
          : [];
      },)
    : [];
  return {
    preference: value.preference,
    table: value.table,
    routes,
  };
}
