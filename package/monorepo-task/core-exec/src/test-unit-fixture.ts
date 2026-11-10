/**
 Disposable process boundary for the shared `test:unit` template: one throwaway monorepo, one package in it. @module
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

import { fixtureEnvironment, resolveMise, } from './build-and-test-fixture.ts';
import { testUnitFixtureConfig, } from './test-unit-fixture-config.ts';

/**
 Upper bound prevents a fixture whose template never returns from hanging verification.
 */
const CHILD_TIMEOUT_MS = 20_000;

/**
 Inputs select the package path the template judges and the unit test files it finds there.
 */
export type TestUnitFixture = {
  /**
   Package path relative to the fixture root, which is the key the shared template matches exemptions on.
   */
  readonly packagePath: string;
  /**
   Unit test files written into the package root; each records its own run in the events file.
   */
  readonly unitTestFiles: readonly string[];
};

/**
 Fixture output reports the exit status, both streams, and which test files actually ran.
 */
export type TestUnitResult = {
  /**
   File-backed execution trace proves a passing run executed something.
   */
  readonly events: readonly string[];
  /**
   Both streams retain the task diagnostic and Node's own error text.
   */
  readonly output: string;
  /**
   Numeric status is returned only after normal child exit, never signal termination.
   */
  readonly exitCode: number;
};

/**
 Run the real shared `test:unit` template over one disposable package.

 @param packagePath - package path relative to the fixture root

 @param unitTestFiles - unit test file names to place in the package root

 @returns exit status, diagnostic output, and the file-backed trace of executed test files

 @throws Error if mise is unavailable or the child cannot exit normally

 @example
 ```ts
 await runTestUnitFixture({ packagePath: 'package/fixture/demo', unitTestFiles: [], });
 ```
 */
export async function runTestUnitFixture({
  packagePath,
  unitTestFiles,
}: TestUnitFixture,): Promise<TestUnitResult> {
  /**
   Disposable resource removes configuration and outputs after the child has closed.
   */
  await using directory = await mkdtempDisposable(join(
    scratchRoot(),
    'mise-test-unit-',
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
   The package the template judges, at the path its exemption list is keyed on.
   */
  const packageRoot = join(
    root,
    packagePath,
  );
  /**
   Trace file the fixture test files append to, created before the run so an empty trace is readable.
   */
  const eventsPath = join(
    root,
    'events',
  );
  await mkdir(
    packageRoot,
    { recursive: true, },
  );
  await writeFile(
    eventsPath,
    '',
  );
  await Promise.all([
    writeFile(
      join(
        root,
        'mise.toml',
      ),
      testUnitFixtureConfig,
    ),
    writeFile(
      join(
        packageRoot,
        'mise.toml',
      ),
      '[tasks."test:unit"]\nextends = "test:unit"\n',
    ),
    ...unitTestFiles.map(async function writeTestFile(name,): Promise<void> {
      /**
       JSON encoding preserves the destination JavaScript string grammar.
       */
      const script = `import { appendFile } from 'node:fs/promises';
await appendFile(process.env.FIXTURE_EVENTS, ${JSON.stringify(`${name}\n`,)});
`;
      await writeFile(
        join(
          packageRoot,
          name,
        ),
        script,
      );
    },),
  ],);
  /**
   Child output is drained concurrently so diagnostics cannot block process completion.
   */
  const child = spawn(
    mise,
    [
      'run',
      `//${packagePath}:test:unit`,
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
    throw new Error(`mise test:unit fixture did not exit normally (${child.signalCode}): ${stderr}`,);
  /**
   File-backed trace cannot be satisfied by mise echoing a command string.
   */
  const trace = (await readFile(
    eventsPath,
    'utf8',
  )).trim();
  return {
    exitCode,
    output: `${stdout}\n${stderr}`,
    events: trace === '' ? [] : trace.split('\n',),
  };
}
