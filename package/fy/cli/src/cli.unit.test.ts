import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  findRootCached,
  MISE_MONOREPO,
} from '@monochromatic-dev/fs-path-core/ts';
import spawn, { type SubprocessError, } from 'nano-spawn';

/** Mise monorepo root for the spawn cwd, so the bin path is invariant to the task's launch directory. */
const REPO_ROOT = await findRootCached({ marker: MISE_MONOREPO, },);

/** Prefix emitted by the tagged logger on info-level lines */
const LOG_PREFIX = '[info]';

/**
 Filters out logger `[info]` lines from raw stdout,
 returning only the application output.
 
 @param raw - Raw stdout string including logger lines
 @returns Lines that are not logger output, joined with newlines
 
 @example
 ```ts
 stripLogLines({ raw: '[info] ...debug\n/tmp/test\n' });
 // => '/tmp/test'
 ```
 */
function stripLogLines({ raw, }: { raw: string; },): string {
  return raw
    .split('\n',)
    .filter(function isNotLogLine(line,) {
      return !line.startsWith(LOG_PREFIX,);
    },)
    .join('\n',)
    .trim();
}

/**
 Runs cli-fy as a subprocess and returns stdout (with log lines stripped),
 raw stderr, and exit code.
 
 @param args - CLI arguments to pass after `cli-fy`
 @returns Cleaned stdout text, raw stderr text, and numeric exit code
 
 @example
 ```ts
 const result = await runCliFy({ args: ['node:path', 'join', '/tmp', 'test'] });
 // result.stdout === '/tmp/test'
 ```
 */
async function runCliFy({ args, }: { args: readonly string[]; },): Promise<{
  stdout: string;
  stderr: string;
  exitCode: number;
}> {
  try {
    const result = await spawn('node', ['package/fy/cli/dist/final/node/index.mjs', ...args,], {
      cwd: REPO_ROOT,
    },);
    return {
      exitCode: 0,
      stderr: result.stderr,
      stdout: stripLogLines({ raw: result.stdout, },),
    };
  }
  catch (error: unknown) {
    const spawnError = error as SubprocessError;
    return {
      exitCode: spawnError.exitCode ?? 1,
      stderr: spawnError.stderr,
      stdout: stripLogLines({ raw: spawnError.stdout, },),
    };
  }
}

await describe({
  name: 'cli-fy integration',
  children: [
    //region Function calls: calling exported functions with arguments

    it({
      name: 'calls node:path join with two arguments',
      fn: async () => {
        const result = await runCliFy({ args: ['node:path', 'join', '/tmp', 'test',], },);
        expect(result.stdout.trim(),).toBe('/tmp/test',);
        expect(result.exitCode,).toBe(0,);
      },
    },),

    it({
      name: 'calls node:path basename with ext argument',
      fn: async () => {
        const result = await runCliFy({
          args: ['node:path', 'basename', '/tmp/foo.txt', '.txt',],
        },);
        expect(result.stdout.trim(),).toBe('foo',);
        expect(result.exitCode,).toBe(0,);
      },
    },),

    it({
      name: 'coerces numeric arguments for arithmetic',
      fn: async () => {
        const result = await runCliFy({ args: ['node:path', 'join', '/a', 'b',], },);
        expect(result.stdout.trim(),).toBe('/a/b',);
        expect(result.exitCode,).toBe(0,);
      },
    },),

    //endregion Function calls

    //region Non-function exports: accessing values without calling

    it({
      name: 'prints non-function export value when no args given',
      fn: async () => {
        const result = await runCliFy({ args: ['node:path', 'sep',], },);
        expect(result.stdout.trim(),).toBe('/',);
        expect(result.exitCode,).toBe(0,);
      },
    },),

    it({
      name: 'prints delimiter export value',
      fn: async () => {
        const result = await runCliFy({ args: ['node:path', 'delimiter',], },);
        expect(result.stdout.trim(),).toBe(':',);
        expect(result.exitCode,).toBe(0,);
      },
    },),

    //endregion Non-function exports

    //region Default export: accessing default export via "default" keyword

    it({
      name: 'prints default export value from a local fixture',
      fn: async () => {
        const result = await runCliFy({
          args: ['./package/fy/cli/src/fixture/return1.ts', 'default',],
        },);
        expect(result.stdout,).toBe('1',);
        expect(result.exitCode,).toBe(0,);
      },
    },),

    it({
      name: 'calls default export function and prints its return value',
      fn: async () => {
        const result = await runCliFy({
          args: ['./package/fy/cli/src/fixture/return1-fn.ts', 'default',],
        },);
        expect(result.stdout,).toBe('1',);
        expect(result.exitCode,).toBe(0,);
      },
    },),

    //endregion Default export

    //region Error cases: non-existent exports, type mismatches, bad specifiers

    it({
      name: 'errors when export does not exist',
      fn: async () => {
        const result = await runCliFy({ args: ['node:path', 'doesNotExist',], },);
        expect(result.exitCode,).not.toBe(0,);
        expect(result.stderr,).toContain('not found',);
        expect(result.stderr,).toContain('Available exports',);
      },
    },),

    it({
      name: 'errors when non-function export receives arguments',
      fn: async () => {
        const result = await runCliFy({ args: ['node:path', 'sep', 'extraArg',], },);
        expect(result.exitCode,).not.toBe(0,);
        expect(result.stderr,).toContain('not a function',);
      },
    },),

    it({
      name: 'errors when specifier cannot be resolved',
      fn: async () => {
        const result = await runCliFy({ args: ['nonexistent-pkg-99999', 'foo',], },);
        expect(result.exitCode,).not.toBe(0,);
        expect(result.stderr,).toContain('Cannot resolve',);
      },
    },),

    //endregion Error cases

    //region Help: verifies --help output

    it({
      name: 'prints help with --help flag',
      fn: async () => {
        const result = await runCliFy({ args: ['--help',], },);
        expect(result.stdout,).toContain('cli-fy',);
        expect(result.stdout,).toContain('SPECIFIER',);
        expect(result.stdout,).toContain('EXPORT',);
      },
    },),

    //endregion Help

    //region Missing arguments: verifies parser errors

    it({
      name: 'errors when no arguments provided',
      fn: async () => {
        const result = await runCliFy({ args: [], },);
        expect(result.exitCode,).not.toBe(0,);
      },
    },),

    it({
      name: 'errors when only specifier provided',
      fn: async () => {
        const result = await runCliFy({ args: ['node:path',], },);
        expect(result.exitCode,).not.toBe(0,);
      },
    },),
    //endregion Missing arguments
  ],
},);
