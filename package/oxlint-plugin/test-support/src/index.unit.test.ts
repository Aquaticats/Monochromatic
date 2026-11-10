/**
 Unit tests for the fixture helpers `@monochromatic-dev/oxlint-plugin-test-support` exports.

 `src/run-oxlint-fixture.unit.test.ts` covers the runner end to end against real oxlint runs; this file covers the
 helpers that runner is built from, so a defect in path resolution, temp copying, or spawn-failure reporting fails
 here, where the message names the helper, instead of as a confusing failure inside a plugin's own fixture test.

 @module
 */

import {
  existsSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  dirname,
  isAbsolute,
  join,
  resolve,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createTempFixtureFile,
  fixtureConfigPath,
  fixturePackageRoot,
  fixtureSourceRoot,
  OXLINT_PLUGIN_TEST_ROOT,
  outputFromSpawnError,
  type OxlintRuleDiagnostic,
  resolveFixtureTarget,
  uniqueRuleCodes,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Fixture package that exists in this checkout, so a path test shows the layout it claims rather than the string it
 concatenates.
 */
const REAL_FIXTURE_PACKAGE = 'oxlint-tsdoc';

/**
 Config file the fixture packages keep beside their own `src` directory.
 */
const FIXTURE_CONFIG_FILE = '.oxlintrc.fixture.json';

/**
 Source text every copy test writes and reads back.
 */
const SOURCE_TEXT = 'export const copied = true;\n';

/**
 Basename of the source file every copy test writes.
 */
const SOURCE_FILE_NAME = 'source.ts';

/**
 Message the module throws when a failed spawn carries no standard output.
 */
const MISSING_STDOUT_MESSAGE = 'oxlint spawn failure did not expose stdout.';

/**
 Message the module throws when a failed spawn carries a standard output that is not a string.
 */
const NON_STRING_STDOUT_MESSAGE = 'oxlint spawn failure stdout was not a string.';

/**
 Directory prefix every copy test passes to {@link createTempFixtureFile}.
 */
const TEMP_FIXTURE_PREFIX = 'oxlint-temp-fixture-';

/**
 Disposable directory holding the source file a copy test copies.
 */
type SourceFixture = {
  /**
   Absolute path of the source file inside it.
   */
  readonly filePath: string;
  /**
   Removes the directory and the file it holds.
   */
  readonly [Symbol.dispose]: () => void;
};

//endregion Fixtures

//region Helpers

/**
 Writes the source file every copy test copies.

 @returns the source path, with a disposer that removes the directory holding it

 @example
 ```ts
 using source = writeSourceFixture();
 ```
 */
function writeSourceFixture(): SourceFixture {
  /**
   Directory holding the source file, under the scratch root rather than the system temporary directory.
   */
  const directory = mkdtempSync(join(
    scratchRoot(),
    'oxlint-test-support-source-',
  ),);
  /**
   Absolute path of the source file inside it.
   */
  const filePath = join(
    directory,
    SOURCE_FILE_NAME,
  );
  writeFileSync(
    filePath,
    SOURCE_TEXT,
  );
  return {
    filePath,
    [Symbol.dispose]: function cleanup(): void {
      rmSync(
        directory,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Builds a diagnostic that carries only the code a de-duplication test reads.

 @param code - rule code the diagnostic carries

 @returns diagnostic carrying that code

 @example
 ```ts
 diagnosticWithCode('tsdoc(multiline-blocks)');
 ```
 */
function diagnosticWithCode(code: string,): OxlintRuleDiagnostic {
  return {
    message: 'reported so a rule-code test has a diagnostic to read',
    code,
    severity: 'error',
    filename: SOURCE_FILE_NAME,
  };
}

/**
 Runs a spawn-failure guard and returns the error it threw.

 @param error - value the failed spawn rejected with

 @returns the error the guard threw

 @throws {@link Error} when the guard accepted the value, which every caller here expects it to refuse

 @example
 ```ts
 const thrown = thrownSpawnFailure({ stdout: null });
 ```
 */
function thrownSpawnFailure(error: unknown,): Error {
  try {
    outputFromSpawnError({ error, },);
  }
  catch (thrown: unknown) {
    if (Error.isError(thrown,))
      return thrown;
    throw new Error(
      `the spawn-failure guard threw a value that is not an Error: ${String(thrown,)}`,
      { cause: thrown, },
    );
  }
  throw new Error('the spawn-failure guard accepted a value that carries no usable standard output',);
}

//endregion Helpers

await describe({
  name: '',
  children: [
    describe({
      name: 'OXLINT_PLUGIN_TEST_ROOT',
      children: [
        it({
          name: 'is an absolute path to the monorepo root that holds this package',
          fn: async () => {
            /**
             Manifest of the package this file belongs to, resolved from the workspace root.
             */
            const manifestPath = join(
              OXLINT_PLUGIN_TEST_ROOT,
              'package',
              'oxlint-plugin',
              'test-support',
              'package.json',
            );

            expect(isAbsolute(OXLINT_PLUGIN_TEST_ROOT,),).toBe(true,);
            expect(existsSync(manifestPath,),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: fixturePackageRoot.name,
      children: [
        it({
          name: 'resolves a fixture package under package/test-fixture',
          fn: async () => {
            /**
             Root of the fixture package this test names.
             */
            const packageRoot = fixturePackageRoot({ fixturePackageName: REAL_FIXTURE_PACKAGE, },);

            expect(packageRoot,).toBe(join(
              OXLINT_PLUGIN_TEST_ROOT,
              'package',
              'test-fixture',
              REAL_FIXTURE_PACKAGE,
            ),);
          },
        },),
        it({
          name: 'names a directory that exists in this checkout',
          fn: async () => {
            /**
             Root of the fixture package this test names.
             */
            const packageRoot = fixturePackageRoot({ fixturePackageName: REAL_FIXTURE_PACKAGE, },);

            expect(existsSync(packageRoot,),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: fixtureSourceRoot.name,
      children: [
        it({
          name: 'appends the source directory to the fixture package root',
          fn: async () => {
            /**
             Root of the fixture package this test names.
             */
            const packageRoot = fixturePackageRoot({ fixturePackageName: REAL_FIXTURE_PACKAGE, },);

            expect(fixtureSourceRoot({ fixturePackageName: REAL_FIXTURE_PACKAGE, },),)
              .toBe(join(
                packageRoot,
                'src',
              ),);
          },
        },),
        it({
          name: 'names a source directory that exists in this checkout',
          fn: async () => {
            /**
             Source directory of the fixture package this test names.
             */
            const sourceRoot = fixtureSourceRoot({ fixturePackageName: REAL_FIXTURE_PACKAGE, },);

            expect(existsSync(sourceRoot,),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: fixtureConfigPath.name,
      children: [
        it({
          name: 'resolves the config file inside the fixture package directory',
          fn: async () => {
            /**
             Root of the fixture package this test names.
             */
            const packageRoot = fixturePackageRoot({ fixturePackageName: REAL_FIXTURE_PACKAGE, },);

            expect(fixtureConfigPath({
              fixturePackageName: REAL_FIXTURE_PACKAGE,
              fileName: FIXTURE_CONFIG_FILE,
            },),)
              .toBe(join(
                packageRoot,
                FIXTURE_CONFIG_FILE,
              ),);
          },
        },),
        it({
          name: 'names a config file that exists in this checkout',
          fn: async () => {
            /**
             Config file this test names.
             */
            const configPath = fixtureConfigPath({
              fixturePackageName: REAL_FIXTURE_PACKAGE,
              fileName: FIXTURE_CONFIG_FILE,
            },);

            expect(existsSync(configPath,),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: resolveFixtureTarget.name,
      children: [
        it({
          name: 'resolves a relative fixture path against the source root',
          fn: async () => {
            expect(resolveFixtureTarget({
              fixtureSourceRoot: '/fixture/source',
              fixturePath: join(
                'invalid',
                'file.ts',
              ),
            },),)
              .toBe(join(
                '/fixture/source',
                'invalid',
                'file.ts',
              ),);
          },
        },),
        it({
          name: 'keeps an absolute path exactly as it is given, without normalising it',
          fn: async () => {
            /**
             Absolute path holding a `..` segment, which resolving it would remove.
             */
            const unnormalised = `${join(
              scratchRoot(),
              'temp-fixture',
            )}/../temp-fixture/case.ts`;

            expect(resolveFixtureTarget({
              fixtureSourceRoot: '/fixture/source',
              fixturePath: unnormalised,
            },),).toBe(unnormalised,);
          },
        },),
      ],
    },),
    describe({
      name: createTempFixtureFile.name,
      children: [
        it({
          name: 'copies the source into a directory of its own under the scratch root',
          fn: async () => {
            using source = writeSourceFixture();
            await using fixture = await createTempFixtureFile({
              fileName: SOURCE_FILE_NAME,
              sourcePath: source.filePath,
              tempPrefix: TEMP_FIXTURE_PREFIX,
            },);
            /**
             Directory holding the copy.
             */
            const copyDirectory = dirname(fixture.filePath,);
            /**
             Directory holding the source the copy was made from.
             */
            const sourceDirectory = dirname(source.filePath,);

            /**
             Whether the copy landed under the scratch root rather than somewhere the system temporary directory
             backs.
             */
            const copyIsInScratchRoot = copyDirectory.startsWith(scratchRoot(),);

            expect(readFileSync(
              fixture.filePath,
              'utf8',
            ),).toBe(SOURCE_TEXT,);
            expect(copyDirectory,).not.toBe(sourceDirectory,);
            expect(copyIsInScratchRoot,).toBe(true,);
          },
        },),
        it({
          name: 'removes the copied directory when the fixture is disposed',
          fn: async () => {
            using source = writeSourceFixture();
            /**
             Copy whose directory the disposer removes.
             */
            const fixture = await createTempFixtureFile({
              fileName: SOURCE_FILE_NAME,
              sourcePath: source.filePath,
              tempPrefix: TEMP_FIXTURE_PREFIX,
            },);
            /**
             Directory holding the copy.
             */
            const copyDirectory = dirname(fixture.filePath,);

            expect(existsSync(copyDirectory,),).toBe(true,);
            await fixture[Symbol.asyncDispose]();
            expect(existsSync(copyDirectory,),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: uniqueRuleCodes.name,
      children: [
        it({
          name: 'sorts the codes and keeps one of each',
          fn: async () => {
            expect(uniqueRuleCodes([
              diagnosticWithCode('harness-probe(reports-program)',),
              diagnosticWithCode('harness-probe(fixes-program)',),
              diagnosticWithCode('harness-probe(reports-program)',),
            ],),).toEqual([
              'harness-probe(fixes-program)',
              'harness-probe(reports-program)',
            ],);
          },
        },),
        it({
          name: 'returns no codes for a run that reported nothing',
          fn: async () => {
            expect(uniqueRuleCodes([],),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: outputFromSpawnError.name,
      children: [
        it({
          name: 'reads the command line and both streams a failed spawn carried',
          fn: async () => {
            expect(outputFromSpawnError({
              error: {
                command: 'oxlint --format json',
                stdout: '{"diagnostics":[]}',
                stderr: 'a warning oxlint printed',
              },
            },),).toEqual({
              command: 'oxlint --format json',
              stdout: '{"diagnostics":[]}',
              stderr: 'a warning oxlint printed',
            },);
          },
        },),
        it({
          name: 'falls back to the oxlint command line and to no standard error when the failure omits them',
          fn: async () => {
            expect(outputFromSpawnError({ error: { stdout: '', }, },),).toEqual({
              command: 'oxlint',
              stdout: '',
              stderr: '',
            },);
          },
        },),
        it({
          name: 'names the missing stdout when the failure is not an object',
          fn: async () => {
            /**
             Error the guard threw for a failure value that is not an object.
             */
            const thrown = thrownSpawnFailure('oxlint could not start',);

            expect(thrown.message,).toBe(MISSING_STDOUT_MESSAGE,);
            expect(thrown.cause,).toBe('oxlint could not start',);
          },
        },),
        it({
          name: 'names the missing stdout when the failure is null',
          fn: async () => {
            expect(thrownSpawnFailure(null,).message,).toBe(MISSING_STDOUT_MESSAGE,);
          },
        },),
        it({
          name: 'names the missing stdout when the failure object carries none',
          fn: async () => {
            /**
             Failure object with no `stdout` property, which is what a spawn that never started reports.
             */
            const failure = { stderr: 'oxlint could not start', };
            /**
             Error the guard threw for it.
             */
            const thrown = thrownSpawnFailure(failure,);

            expect(thrown.message,).toBe(MISSING_STDOUT_MESSAGE,);
            expect(thrown.cause,).toBe(failure,);
          },
        },),
        it({
          name: 'rejects a standard output that is not a string, naming what it found',
          fn: async () => {
            expect(thrownSpawnFailure({ stdout: null, },).message,).toBe(NON_STRING_STDOUT_MESSAGE,);
          },
        },),
      ],
    },),
    describe({
      name: 'the missing-stdout failure text',
      children: [
        it({
          name: 'has one owner in the module, so the two guards that throw it cannot drift apart',
          fn: async () => {
            /**
             Source of the module the guards live in, read from this checkout rather than from the built artifact: a
             literal spelled out twice is invisible to a test that only calls the guards, because both copies still
             produce the same message until one of them is edited.
             */
            const source = readFileSync(
              resolve(
                import.meta.dirname,
                'index.ts',
              ),
              'utf8',
            );
            /**
             How many times the module spells the text out.
             */
            const occurrences = source.split(MISSING_STDOUT_MESSAGE,).length - 1;

            expect(occurrences,).toBe(1,);
          },
        },),
        it({
          name: 'is the message of every guard that reports a missing stdout',
          fn: async () => {
            /**
             Error the guard threw for a failure that is not an object.
             */
            const fromPrimitive = thrownSpawnFailure('oxlint could not start',);
            /**
             Error the guard threw for a failure object that carries no stdout.
             */
            const fromObjectWithoutStdout = thrownSpawnFailure({ stderr: 'oxlint could not start', },);

            expect(fromPrimitive.message,).toBe(fromObjectWithoutStdout.message,);
          },
        },),
      ],
    },),
  ],
},);
