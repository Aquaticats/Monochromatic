import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { exec, } from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { join, } from 'node:path';
import { pathToFileURL, } from 'node:url';
import { promisify, } from 'node:util';

// oxlint-disable-next-line typescript/strict-void-return -- Node's promisify typing treats exec callback shape as void-context even though the promisified command result is intentionally consumed in these CLI integration tests.
const execAsync = promisify(exec,);

/**
 Creates a fresh throwaway test directory under `dist/temp/test` and returns the
 source CLI path plus that directory.
 
 @returns Absolute path to the source `tsc-filter.ts` and a unique scratch directory
 */
function setup() {
  const testFileDir = import.meta.dirname;
  const cliPath = join(testFileDir, 'tsc-filter.ts',);

  const packageDir = join(testFileDir, '..',);
  const timestamp = Date.now();
  const randomId = Math.random().toString(36,).slice(2, 8,);
  const testDir = join(packageDir, 'dist', 'temp', 'test',
    `tsc-filter-${timestamp}-${randomId}`,);

  if (!existsSync(testDir,))
    mkdirSync(testDir, { recursive: true, },);

  return { cliPath, testDir, };
}

/**
 Removes a throwaway test directory created by {@link setup}.
 
 @param testDir - Scratch directory to delete
 */
function teardown({ testDir, }: { testDir: string; },) {
  if (existsSync(testDir,))
    rmSync(testDir, { recursive: true, },);
}

//region Integration tests for CLI execution (spawn real tsc; flaky under parallel load, hence .expensive.)

await describe({
  name: '',
  children: [
    describe({
      name: 'task-tsc CLI',
      children: [
        it({
          name: 'forwards arguments to tsc',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, } = fixtures;

            // tsc --version should succeed and print version info
            const { stdout, } = await execAsync(`node ${cliPath} --version`,);

            expect(stdout.trim().length,).toBeGreaterThan(0,);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'does not execute the CLI path when imported',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, testDir, } = fixtures;
            const importProbe = join(testDir, 'import-probe.ts',);
            writeFileSync(
              importProbe,
              `import ${
                JSON.stringify(pathToFileURL(cliPath,).href,)
              };\nconsole.log('imported only');\n`,
            );

            const { stdout, stderr, } = await execAsync(`node ${importProbe}`,);

            expect(stdout,).toBe('imported only\n',);
            expect(stderr,).toBe('',);

            teardown(fixtures,);
          },
        },),
        it({
          name: 'exits non-zero when tsc reports project errors',
          fn: async () => {
            const fixtures = setup();
            const { cliPath, testDir, } = fixtures;

            // Create a tsconfig that references a non-existent file
            const tsconfig = join(testDir, 'tsconfig.json',);
            writeFileSync(tsconfig, JSON.stringify({
              compilerOptions: { strict: true, noEmit: true, },
              include: ['nonexistent.ts',],
            },),);

            const sourceFile = join(testDir, 'nonexistent.ts',);
            writeFileSync(sourceFile, 'const x: number = "not a number";\n',);

            try {
              await execAsync(`node ${cliPath} --noEmit -p ${tsconfig}`,);
              // Should not reach here; tsc should fail on the type error
              expect(true,).toBe(false,);
            }
            catch (error: unknown) {
              const execError = error as { code: number; stdout: string; };
              expect(execError.code,).toBeGreaterThan(0,);
              expect(execError.stdout,).toContain('error TS',);
            }

            teardown(fixtures,);
          },
        },),
        it({
          name: 'built CLI entry runs tsc and exits non-zero on project errors',
          fn: async () => {
            const fixtures = setup();
            const { testDir, } = fixtures;
            // mise `lint:types` dispatches the built entry; a shared-chunk main guard once made it a silent no-op.
            const builtCliPath = join(
              import.meta.dirname,
              '..',
              'dist',
              'final',
              'node',
              'tsc-filter.mjs',
            );
            const tsconfig = join(testDir, 'tsconfig.json',);
            writeFileSync(tsconfig, JSON.stringify({
              compilerOptions: { strict: true, noEmit: true, },
              include: ['built-entry-error.ts',],
            },),);
            writeFileSync(
              join(testDir, 'built-entry-error.ts',),
              'const x: number = "not a number";\n',
            );

            try {
              await execAsync(`node ${builtCliPath} --noEmit -p ${tsconfig}`,);
              // Should not reach here; built entry must run tsc and fail on the type error
              expect(true,).toBe(false,);
            }
            catch (error: unknown) {
              const execError = error as { code: number; stdout: string; };
              expect(execError.code,).toBeGreaterThan(0,);
              expect(execError.stdout,).toContain('error TS',);
            }

            teardown(fixtures,);
          },
        },),
      ],
    },),
  ],
},);

//endregion Integration tests for CLI execution
