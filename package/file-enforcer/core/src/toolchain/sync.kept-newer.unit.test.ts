/**
 Tests that `syncToolchain` and `observeTools` never replace a repository-published tool with an older version,
 warn on every run that meets one, and still converge third-party tools to the lock.

 Each case runs against a disposable fixture home with a fake `cargo` in the toolchain bin directory; nothing touches
 the real home, the real cargo install directory or the network.
 */

import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  writeFileSync,
} from 'node:fs';
import { setImmediate as nextMacrotask, } from 'node:timers/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  observeTools,
  resolveToolchainPaths,
  syncToolchain,
  type ToolchainPaths,
  type ToolPlanEntry,
} from '../../dist/final/node/index.mjs';

//region Fixtures

/**
 Disposable fixture: toolchain paths, environment, the fake-cargo log and its installed-crates table.
 */
type Fixture = {
  readonly paths: ToolchainPaths;
  readonly env: Record<string, string>;
  readonly logFile: string;
};

/**
 Builds a fixture whose fake `cargo` lists one installed crate and logs every call.

 @param crate - Crate name the fake `binstall --list` shows

 @param installed - Version the fake `binstall --list` shows for that crate

 @returns The fixture
 */
function makeFixture(
  {
    crate,
    installed,
  }: {
    readonly crate: string;
    readonly installed: string;
  },
): Fixture {
  /**
   Disposable root doubling as the fixture home.
   */
  const root = mkdtempSync(join(
    scratchRoot(),
    'toolchain-kept-newer-',
  ),);
  mkdirSync(
    join(
      root,
      'data',
      'monochromatic-dev',
      'toolchain',
      'bin',
    ),
    { recursive: true, },
  );
  /**
   Environment isolated to the fixture home.
   */
  const env = {
    HOME: root,
    XDG_DATA_HOME: join(
      root,
      'data',
    ),
  };
  /**
   Fixture paths.
   */
  const paths = resolveToolchainPaths({
    worktreeRoot: root,
    env,
  },);
  /**
   File the fake cargo appends each call's argv to.
   */
  const logFile = join(
    root,
    'cargo.log',
  );
  /**
   Table the fake `cargo binstall --list` prints.
   */
  const table = [
    'Package              Cargo Ver    Binstall Ver   Method            ',
    `${crate}  ${installed}  ${installed}  cargo binstall`,
    'Total: 1 | Binstall: 1 | Source: 0 | Local: 0 | Git: 0 | Drifted: 0',
    '',
  ].join('\n',);
  /**
   Path of the fake cargo.
   */
  const cargoPath = join(
    paths.toolchainBinDir,
    'cargo',
  );
  writeFileSync(
    cargoPath,
    [
      // The interpreter running this test, so the fixture works under any runtime.
      `#!${process.execPath}`,
      `const fs = require('node:fs');`,
      `fs.appendFileSync(${JSON.stringify(logFile,)}, JSON.stringify(process.argv.slice(2,)) + '\\n');`,
      `if (process.argv[2] === 'binstall' && process.argv[3] === '--list') process.stdout.write(${
        JSON.stringify(table,)
      });`,
      'process.exit(0);',
    ].join('\n',),
    'utf8',
  );
  chmodSync(
    cargoPath,
    0o755,
  );
  return {
    paths,
    env: {
      ...env,
      PATH: paths.toolchainBinDir,
    },
    logFile,
  };
}

/**
 Reads the fake-cargo log as one argv array per call.

 @param fixture - Fixture holding the log

 @returns Argv arrays in call order
 */
function cargoCalls(fixture: Fixture,): readonly (readonly string[])[] {
  if (!existsSync(fixture.logFile,))
    return [];
  return readFileSync(
    fixture.logFile,
    'utf8',
  )
    .split('\n',)
    .filter(function nonBlank(line,): boolean {
      return line.trim() !== '';
    },)
    .map(function parse(line,): string[] {
      return JSON.parse(line,) as string[];
    },);
}

/**
 Install calls among the fake-cargo calls: those carrying `--version`.

 @param fixture - Fixture holding the log

 @returns Argv arrays of the installs
 */
function installCalls(fixture: Fixture,): readonly (readonly string[])[] {
  return cargoCalls(fixture,)
    .filter(function isInstall(call,): boolean {
      return call.includes('--version',);
    },);
}

/**
 Builds a `cargo-binstall` plan entry.

 @param crate - Crate the entry installs

 @param version - Version the lock pins

 @returns The plan entry
 */
function cargoEntry(
  {
    crate,
    version,
  }: {
    readonly crate: string;
    readonly version: string;
  },
): ToolPlanEntry {
  return {
    id: `cargo:${crate}`,
    request: { track: 'latest', },
    version,
    route: 'cargo-binstall',
    manager: 'cargo',
    packageId: crate,
    bins: [crate],
    options: {},
    env: {},
    artifacts: {},
  };
}

/**
 Crate whose kept-newer warning marks the end of a collection window.
 */
const SENTINEL_CRATE = 'monochromatic-sentinel';

/**
 Longest wait for the logger to emit the sentinel, in milliseconds.
 */
const FLUSH_DEADLINE_MS = 10_000;

/**
 Replaces `console.warn` with a collector until the returned handle is disposed.

 @param lines - Array the collector appends each warning line to

 @returns Handle that restores `console.warn` when disposed
 */
function collectConsoleWarnings(lines: string[],): Disposable {
  /**
   `console.warn` before the override.
   */
  const original = console.warn;
  console.warn = function collect(line: unknown,): void {
    lines.push(String(line,),);
  };
  return {
    [Symbol.dispose](): void {
      console.warn = original;
    },
  };
}

/**
 What a collection window returns: the work's result and the warnings the work logged.
 */
type Collected<Result,> = {
  readonly result: Result;
  readonly warnings: readonly string[];
};

/**
 Runs work while collecting every `console.warn` line the logger emits.

 The logger emits records on its own schedule (the first record waits for sink verification), so a window cannot end
 on a fixed delay. Records leave the logger in order, so the window ends when a sentinel warning, logged after the work,
 arrives; everything the work logged is out by then.

 @param work - Work whose warnings are collected

 @returns The work's result and the warning lines it logged, sentinel excluded
 */
async function collectWarnings<Result,>(work: () => Promise<Result>,): Promise<Collected<Result>> {
  /**
   Collected lines.
   */
  const lines: string[] = [];
  using restore = collectConsoleWarnings(lines,);
  /**
   Result of the work.
   */
  const result = await work();
  /**
   Fixture whose kept-newer warning is the sentinel.
   */
  const sentinel = makeFixture({
    crate: SENTINEL_CRATE,
    installed: '9.0.0',
  },);
  await observeTools({
    entries: [cargoEntry({
      crate: SENTINEL_CRATE,
      version: '1.0.0',
    },)],
    paths: sentinel.paths,
    env: sentinel.env,
  },);
  /**
   Time the wait started.
   */
  const started = Date.now();
  while (!lines.some(function isSentinel(line,): boolean {
    return line.includes(SENTINEL_CRATE,);
  },)) {
    if ((Date.now() - started) > FLUSH_DEADLINE_MS)
      throw new Error('the logger did not emit the sentinel warning in time',);
    // oxlint-disable-next-line no-await-in-loop -- the wait polls until the logger has flushed
    await nextMacrotask();
  }
  return {
    result,
    warnings: lines.filter(function notSentinel(line,): boolean {
      return !line.includes(SENTINEL_CRATE,);
    },),
  };
}

//endregion Fixtures

await describe({
  name: 'repository tools in the sync and status',
  // One case at a time: each case swaps the shared `console.warn` to read the logger's warnings.
  concurrency: 1,
  children: [
    describe({
      name: syncToolchain.name,
      children: [
        it({
          name: 'keeps a repository tool installed newer than the lock: no install, counted as satisfied, warning with the remedy',
          fn: async () => {
            /**
             Fixture with 0.4.4 installed under a lock that pins 0.4.0.
             */
            const fixture = makeFixture({
              crate: 'monochromatic-fixture',
              installed: '0.4.4',
            },);
            /**
             Warnings logged during the sync.
             */
            const {
              result: report,
              warnings,
            } = await collectWarnings(async function sync() {
              return await syncToolchain({
                tools: [cargoEntry({
                  crate: 'monochromatic-fixture',
                  version: '0.4.0',
                },)],
                paths: fixture.paths,
                env: fixture.env,
              },);
            },);
            expect(installCalls(fixture,),)
              .toEqual([],);
            expect(report.installed,)
              .toEqual([],);
            expect(report.unchanged,)
              .toEqual(['cargo:monochromatic-fixture'],);
            expect(report.keptNewer,)
              .toEqual(['cargo:monochromatic-fixture'],);
            expect(report.failed,)
              .toEqual({},);
            /**
             The single kept-newer warning.
             */
            const kept = warnings.filter(function mentions(line,): boolean {
              return line.includes('older repository tools are never kept',);
            },);
            expect(kept.length,)
              .toBe(1,);
            expect(kept[0],)
              .toContain('cargo:monochromatic-fixture',);
            expect(kept[0],)
              .toContain('pins 0.4.0',);
            expect(kept[0],)
              .toContain('runs 0.4.4',);
            expect(kept[0],)
              .toContain('mise run toolchain:update',);
          },
        },),
        it({
          name: 'upgrades a repository tool installed older than the lock',
          fn: async () => {
            /**
             Fixture with 0.3.0 installed under a lock that pins 0.4.0.
             */
            const fixture = makeFixture({
              crate: 'monochromatic-fixture',
              installed: '0.3.0',
            },);
            const report = await syncToolchain({
              tools: [cargoEntry({
                crate: 'monochromatic-fixture',
                version: '0.4.0',
              },)],
              paths: fixture.paths,
              env: fixture.env,
            },);
            expect(installCalls(fixture,).length,)
              .toBe(1,);
            expect(installCalls(fixture,)[0],)
              .toEqual([
                'binstall',
                '--no-confirm',
                '--version',
                '0.4.0',
                'monochromatic-fixture',
              ],);
            expect(report.installed,)
              .toEqual(['cargo:monochromatic-fixture'],);
            expect(report.keptNewer,)
              .toEqual([],);
          },
        },),
        it({
          name: 'still replaces a third-party tool installed newer than the lock, with the shared-install warning',
          fn: async () => {
            /**
             Fixture with 0.4.4 of a third-party crate installed under a lock that pins 0.4.0.
             */
            const fixture = makeFixture({
              crate: 'fixture-third-party',
              installed: '0.4.4',
            },);
            /**
             Warnings logged during the sync.
             */
            const {
              result: report,
              warnings,
            } = await collectWarnings(async function sync() {
              return await syncToolchain({
                tools: [cargoEntry({
                  crate: 'fixture-third-party',
                  version: '0.4.0',
                },)],
                paths: fixture.paths,
                env: fixture.env,
              },);
            },);
            expect(installCalls(fixture,)[0],)
              .toEqual([
                'binstall',
                '--no-confirm',
                '--version',
                '0.4.0',
                'fixture-third-party',
              ],);
            expect(report.installed,)
              .toEqual(['cargo:fixture-third-party'],);
            expect(report.keptNewer,)
              .toEqual([],);
            expect(warnings.some(function replaces(line,): boolean {
              return line.includes('replaces it for every tree that shares this install',);
            },),)
              .toBe(true,);
            expect(warnings.some(function kept(line,): boolean {
              return line.includes('older repository tools are never kept',);
            },),)
              .toBe(false,);
          },
        },),
        it({
          name: 'leaves a repository tool at its lock untouched and silent',
          fn: async () => {
            /**
             Fixture with exactly the locked version installed.
             */
            const fixture = makeFixture({
              crate: 'monochromatic-fixture',
              installed: '0.4.0',
            },);
            /**
             Warnings logged during the sync.
             */
            const {
              result: report,
              warnings,
            } = await collectWarnings(async function sync() {
              return await syncToolchain({
                tools: [cargoEntry({
                  crate: 'monochromatic-fixture',
                  version: '0.4.0',
                },)],
                paths: fixture.paths,
                env: fixture.env,
              },);
            },);
            expect(installCalls(fixture,),)
              .toEqual([],);
            expect(report.unchanged,)
              .toEqual(['cargo:monochromatic-fixture'],);
            expect(report.keptNewer,)
              .toEqual([],);
            expect(warnings.filter(function ours(line,): boolean {
              return line.includes('monochromatic-fixture',);
            },),)
              .toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: observeTools.name,
      children: [
        it({
          name: 'reports a newer installed repository tool in sync, marked kept, with the warning',
          fn: async () => {
            /**
             Fixture with 0.4.4 installed under a lock that pins 0.4.0.
             */
            const fixture = makeFixture({
              crate: 'monochromatic-fixture',
              installed: '0.4.4',
            },);
            /**
             Warnings logged during the observation.
             */
            const {
              result: observations,
              warnings,
            } = await collectWarnings(async function observe() {
              return await observeTools({
                entries: [cargoEntry({
                  crate: 'monochromatic-fixture',
                  version: '0.4.0',
                },)],
                paths: fixture.paths,
                env: fixture.env,
              },);
            },);
            expect(observations.map(function summary(observation,) {
              return {
                id: observation.id,
                inSync: observation.inSync,
                keptNewer: observation.keptNewer,
                observed: observation.observed,
              };
            },),)
              .toEqual([
                {
                  id: 'cargo:monochromatic-fixture',
                  inSync: true,
                  keptNewer: true,
                  observed: '0.4.4',
                },
              ],);
            /**
             The kept-newer warning.
             */
            const kept = warnings.filter(function mentions(line,): boolean {
              return line.includes('older repository tools are never kept',);
            },);
            expect(kept.length,)
              .toBe(1,);
            expect(kept[0],)
              .toContain('pins 0.4.0',);
            expect(kept[0],)
              .toContain('runs 0.4.4',);
            expect(kept[0],)
              .toContain('mise run toolchain:update',);
          },
        },),
        it({
          name: 'reports an older installed repository tool and a newer third-party tool as drift, without the kept warning',
          fn: async () => {
            /**
             Fixture with 0.3.0 installed under a lock that pins 0.4.0.
             */
            const older = makeFixture({
              crate: 'monochromatic-fixture',
              installed: '0.3.0',
            },);
            /**
             Fixture with a newer third-party crate.
             */
            const third = makeFixture({
              crate: 'fixture-third-party',
              installed: '0.4.4',
            },);
            /**
             Both observations and the warnings logged while making them.
             */
            const {
              result: both,
              warnings,
            } = await collectWarnings(async function observe() {
              return {
                older: await observeTools({
                  entries: [cargoEntry({
                    crate: 'monochromatic-fixture',
                    version: '0.4.0',
                  },)],
                  paths: older.paths,
                  env: older.env,
                },),
                third: await observeTools({
                  entries: [cargoEntry({
                    crate: 'fixture-third-party',
                    version: '0.4.0',
                  },)],
                  paths: third.paths,
                  env: third.env,
                },),
              };
            },);
            expect(both.older[0]?.inSync,)
              .toBe(false,);
            expect(both.older[0]?.keptNewer,)
              .toBe(false,);
            expect(both.third[0]?.inSync,)
              .toBe(false,);
            expect(both.third[0]?.keptNewer,)
              .toBe(false,);
            expect(warnings.some(function kept(line,): boolean {
              return line.includes('older repository tools are never kept',);
            },),)
              .toBe(false,);
          },
        },),
        it({
          name: 'stays silent when the installed repository tool matches the lock',
          fn: async () => {
            /**
             Fixture with exactly the locked version installed.
             */
            const fixture = makeFixture({
              crate: 'monochromatic-fixture',
              installed: '0.4.0',
            },);
            /**
             Warnings logged during the observation.
             */
            const {
              result: observations,
              warnings,
            } = await collectWarnings(async function observe() {
              return await observeTools({
                entries: [cargoEntry({
                  crate: 'monochromatic-fixture',
                  version: '0.4.0',
                },)],
                paths: fixture.paths,
                env: fixture.env,
              },);
            },);
            expect(observations[0]?.inSync,)
              .toBe(true,);
            expect(observations[0]?.keptNewer,)
              .toBe(false,);
            expect(warnings.filter(function ours(line,): boolean {
              return line.includes('monochromatic-fixture',);
            },),)
              .toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
