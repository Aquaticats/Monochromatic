/**
 Disposable process boundary for the root test aggregate. @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdir,
  mkdtempDisposable,
  readFile,
  realpath,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { text, } from 'node:stream/consumers';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  fixtureEnvironment,
  resolveMise,
} from './build-and-test-fixture.ts';
import {
  fixtureConfiguration,
  type RootTestAggregateFixture,
  type RootTestAggregateResult,
} from './root-test-aggregate-fixture-config.ts';

/**
 Upper bound prevents a broken aggregate fixture from hanging verification.
 */
const CHILD_TIMEOUT_MS = 60_000;

/**
 Fixture package that declares its own `test` task, so it owns its unit files.
 */
const OWNED_PACKAGE = 'package/fixture/owned';
/**
 Fixture package with no `test` task, so root discovery must keep its unit files.
 */
const FALLBACK_PACKAGE = 'package/fixture/fallback';

//region Fixture producers

/**
 Build one suite script that records its own execution before optionally failing.

 @param name - suite identifier written to the event trace

 @param fails - whether this suite throws after recording execution

 @returns JavaScript source for one fixture suite

 @example
 ```ts
 suiteScript({ name: 'owned-suite', fails: true, });
 ```
 */
function suiteScript({
  name,
  fails,
}: {
  readonly name: string;
  readonly fails: boolean;
},): string {
  /**
   JSON encoding preserves the destination JavaScript string grammar.
   */
  return `import { appendFile, } from 'node:fs/promises';
await appendFile(process.env.FIXTURE_EVENTS, ${JSON.stringify(`${name}\n`,)});
if (${JSON.stringify(fails,)}) { throw new Error(${JSON.stringify(`fixture ${name} failed`,)}); }
`;
}

/**
 Create the package trees and their `mise.toml` declarations.

 Discovery, not this fixture, decides which of them the aggregate reaches, so the
 owned package declares its own `test` task and the fallback package declares only
 the inherited `test:unit` template that root discovery already covers.

 @param root - disposable repository root

 @param ownedTestFails - whether the owned package's own suite fails

 @throws Error when a fixture directory cannot be created
 */
async function writePackages({
  root,
  ownedTestFails,
}: {
  readonly root: string;
  readonly ownedTestFails: boolean;
},): Promise<void> {
  await Promise.all([
    mkdir(
      join(
        root,
        OWNED_PACKAGE,
      ),
      { recursive: true, },
    ),
    mkdir(
      join(
        root,
        FALLBACK_PACKAGE,
      ),
      { recursive: true, },
    ),
  ],);
  await Promise.all([
    writeFile(
      join(
        root,
        OWNED_PACKAGE,
        'mise.toml',
      ),
      '[tasks.test]\nrun = "node owned-suite.ts"\n',
    ),
    writeFile(
      join(
        root,
        FALLBACK_PACKAGE,
        'mise.toml',
      ),
      '[tasks."test:unit"]\nextends = "test:unit"\n',
    ),
    writeFile(
      join(
        root,
        OWNED_PACKAGE,
        'owned-suite.ts',
      ),
      suiteScript({
        name: 'owned-suite',
        fails: ownedTestFails,
      },),
    ),
    /**
     Root discovery must not run this file, because its package owns its suites.
     */
    writeFile(
      join(
        root,
        OWNED_PACKAGE,
        'owned.unit.test.ts',
      ),
      suiteScript({
        name: 'owned-unit',
        fails: false,
      },),
    ),
    /**
     No package task reaches this file, so root discovery must keep running it.
     */
    writeFile(
      join(
        root,
        FALLBACK_PACKAGE,
        'fallback.unit.test.ts',
      ),
      suiteScript({
        name: 'fallback-unit',
        fails: false,
      },),
    ),
  ],);
}

//endregion Fixture producers

/**
 Run the root test aggregate against fake suites and isolated mise state.

 @param browserFails - whether the fake root browser suite throws after recording execution

 @param ownedTestFails - whether the fake package suite throws after recording execution

 @returns exit status, diagnostic output and the file-backed suite trace

 @throws Error if mise is unavailable, a section under test is absent, or the child cannot exit normally

 @example
 ```ts
 await runRootTestAggregateFixture({ ownedTestFails: true, });
 ```
 */
export async function runRootTestAggregateFixture({
  browserFails = false,
  ownedTestFails = false,
}: RootTestAggregateFixture,): Promise<RootTestAggregateResult> {
  /**
   Disposable resource removes configuration and outputs after the child has closed.
   */
  await using directory = await mkdtempDisposable(join(
    scratchRoot(),
    'mise-root-test-',
  ),);
  /**
   Canonical paths keep mise's directory ceiling effective on symlinked homes.
   */
  const root = await realpath(directory.path,);
  /**
   Resolved executable is shared by the outer and nested mise invocations.
   */
  const mise = await resolveMise();
  await writeFile(
    join(
      root,
      'events',
    ),
    '',
  );
  await writePackages({
    root,
    ownedTestFails,
  },);
  await Promise.all([
    /**
     The extracted sections are read here so a missing aggregate fails this case, not the import.
     */
    writeFile(
      join(
        root,
        'mise.toml',
      ),
      fixtureConfiguration(),
    ),
    writeFile(
      join(
        root,
        'fixture-browser.ts',
      ),
      suiteScript({
        name: 'browser',
        fails: browserFails,
      },),
    ),
    writeFile(
      join(
        root,
        'fixture-e2e.ts',
      ),
      suiteScript({
        name: 'e2e',
        fails: false,
      },),
    ),
  ],);
  /**
   Child output is drained concurrently so diagnostics cannot block process completion.
   */
  const child = spawn(
    mise,
    [
      'run',
      'test',
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
    throw new Error(`root test aggregate fixture did not exit normally (${child.signalCode}): ${stderr}`,);
  /**
   File-backed execution trace cannot be satisfied by mise echoing a command string.
   */
  const events = (await readFile(
    join(
      root,
      'events',
    ),
    'utf8',
  )).split('\n',)
    .filter(function recorded(line,): boolean {
      return line !== '';
    },)
    .toSorted();
  return {
    exitCode,
    events,
    output: `${stdout}\n${stderr}`,
  };
}
