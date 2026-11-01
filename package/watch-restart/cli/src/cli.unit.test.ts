import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  argsToOptions,
  parseArgs,
} from './cli.ts';
import type { ParsedArgs, } from './cli-types.ts';

/**
 Runs {@link parseArgs} against a synthetic argv with output / exit
 hooks wired to a sink so a failing parse cannot kill the test
 process and help text does not pollute the test runner output.
 
 @param argv - argv slice (after the program name) to parse
 
 @returns parsed args; tests assert on this directly
 
 @example
 ```ts
 const args = runParser(['-w', 'src', '--', 'node',],);
 ```
 */
function runParser(argv: readonly string[],): ParsedArgs {
  return parseArgs({
    argv,
    stdout: function discardOut(_text,): void {
      // tests do not assert on help output here.
    },
    stderr: function discardErr(_text,): void {
      // tests do not assert on error output here.
    },
    onExit: function rejectExit(code,): never {
      throw new Error(
        `parseArgs onExit(${code},); argv = ${JSON.stringify(argv,)}`,
      );
    },
  },);
}

await describe({
  name: 'cli',
  children: [
    describe({
      name: 'parser + argsToOptions (round trip)',
      children: [
        it({
          name: 'basic: -w <dir> -- <cmd> sets paths and command',
          fn: async function basicRoundTrip() {
            const args = runParser(['-w', 'src', '--', 'node',],);
            const options = argsToOptions(args,);

            expect(options.paths,).toEqual(['src',],);
            expect(options.command,).toBe('node',);
            expect(options.args,).toBeUndefined();
          },
        },),
        it({
          name: 'multiple -w flags collect into paths array (order preserved)',
          fn: async function multipleWatch() {
            const args = runParser([
              '-w',
              'a',
              '-w',
              'b',
              '-w',
              'c',
              '--',
              'foo',
            ],);
            const options = argsToOptions(args,);

            expect(options.paths,).toEqual(['a', 'b', 'c',],);
          },
        },),
        it({
          name: 'positional args after -- become command + args',
          fn: async function positionalAfterDashDash() {
            const args = runParser([
              '-w',
              'src',
              '--',
              'node',
              'src/server.ts',
              '--port',
              '3000',
            ],);
            const options = argsToOptions(args,);

            expect(options.command,).toBe('node',);
            expect(options.args,).toEqual([
              'src/server.ts',
              '--port',
              '3000',
            ],);
          },
        },),
        it({
          name: 'include and exclude collect into respective arrays',
          fn: async function includeExcludeMaps() {
            const args = runParser([
              '-w',
              'src',
              '-i',
              '*.ts',
              '--include',
              '*.tsx',
              '-e',
              '*.test.ts',
              '--',
              'foo',
            ],);
            const options = argsToOptions(args,);

            expect(options.include,).toEqual(['*.ts', '*.tsx',],);
            expect(options.exclude,).toEqual(['*.test.ts',],);
          },
        },),
        it({
          name: '--ext accepts comma list and repeated flag, flattened together',
          fn: async function extCommaAndRepeated() {
            const args = runParser([
              '-w',
              'src',
              '--ext',
              '.ts,.tsx',
              '--ext',
              '.css',
              '--',
              'foo',
            ],);
            const options = argsToOptions(args,);

            expect(options.extensions,).toEqual(['.ts', '.tsx', '.css',],);
          },
        },),
        it({
          name: '--events maps create/delete to internal add/unlink',
          fn: async function eventsMap() {
            const args = runParser([
              '-w',
              'src',
              '--events',
              'create,change,delete',
              '--',
              'foo',
            ],);
            const options = argsToOptions(args,);

            expect(options.events,).toEqual(['add', 'change', 'unlink',],);
          },
        },),
        it({
          name: '--no-content-changed sets contentChanged to false',
          fn: async function noContentChangedFlag() {
            const args = runParser([
              '-w',
              'src',
              '--no-content-changed',
              '--',
              'foo',
            ],);
            const options = argsToOptions(args,);

            expect(options.contentChanged,).toBe(false,);
          },
        },),
        it({
          name: 'absence of --no-content-changed leaves contentChanged undefined',
          fn: async function defaultContentChangedAbsent() {
            const args = runParser(['-w', 'src', '--', 'foo',],);
            const options = argsToOptions(args,);

            // Undefined means the orchestrator's default-true takes over.
            expect(options.contentChanged,).toBeUndefined();
          },
        },),
        it({
          name: '--no-initial sets initial to false',
          fn: async function noInitialFlag() {
            const args = runParser([
              '-w',
              'src',
              '--no-initial',
              '--',
              'foo',
            ],);
            const options = argsToOptions(args,);

            expect(options.initial,).toBe(false,);
          },
        },),
        it({
          name: '--max-hash-size, --debounce, --stop-timeout map to integer options',
          fn: async function integerOptions() {
            const args = runParser([
              '-w',
              'src',
              '--max-hash-size',
              '1024',
              '--debounce',
              '50',
              '--stop-timeout',
              '3000',
              '--',
              'foo',
            ],);
            const options = argsToOptions(args,);

            expect(options.maxHashSize,).toBe(1_024,);
            expect(options.debounce,).toBe(50,);
            expect(options.stopTimeout,).toBe(3_000,);
          },
        },),
      ],
    },),
    describe({
      name: 'argsToOptions errors',
      children: [
        it({
          name: 'no command after -- throws a usage error',
          fn: async function noCommandThrows() {
            const args = runParser(['-w', 'src', '--',],);

            expect(function callWithEmptyRest(): void {
              argsToOptions(args,);
            },)
              .toThrow('No command supplied',);
          },
        },),
        it({
          name: 'unknown --events token throws',
          fn: async function unknownEventTokenThrows() {
            const args = runParser([
              '-w',
              'src',
              '--events',
              'create,bogus',
              '--',
              'foo',
            ],);

            expect(function callWithBogusEvent(): void {
              argsToOptions(args,);
            },)
              .toThrow('Unknown --events token',);
          },
        },),
      ],
    },),
    describe({
      name: 'Q6 flag round trip',
      children: [
        it({
          name: '--include-regex and --exclude-regex compile into RegExp lists',
          fn: async function regexRoundTrip() {
            const args = runParser([
              '-w',
              'src',
              '--include-regex',
              String.raw`\.story\.ts$`,
              '--exclude-regex',
              String.raw`\.test\.ts$`,
              '--',
              'foo',
            ],);
            const options = argsToOptions(args,);

            expect(options.includeRegex?.length,).toBe(1,);
            expect(options.includeRegex?.[0]?.test('a.story.ts',),).toBe(true,);
            expect(options.excludeRegex?.length,).toBe(1,);
            expect(options.excludeRegex?.[0]?.test('a.test.ts',),).toBe(true,);
          },
        },),
        it({
          name: '--type accepts comma list and repeated flag, mapped to entity types',
          fn: async function typeRoundTrip() {
            const args = runParser([
              '-w',
              'src',
              '--type',
              'file,dir',
              '--',
              'foo',
            ],);
            const options = argsToOptions(args,);

            expect(options.types,).toEqual(['file', 'dir',],);
          },
        },),
        it({
          name: '--ignore-file collects into ignoreFiles array',
          fn: async function ignoreFileRoundTrip() {
            const args = runParser([
              '-w',
              'src',
              '--ignore-file',
              '.watchignore',
              '--ignore-file',
              '.devignore',
              '--',
              'foo',
            ],);
            const options = argsToOptions(args,);

            expect(options.ignoreFiles,).toEqual(['.watchignore', '.devignore',],);
          },
        },),
        it({
          name: '--depth and --poll map to integer options',
          fn: async function depthPollRoundTrip() {
            const args = runParser([
              '-w',
              'src',
              '--depth',
              '3',
              '--poll',
              '500',
              '--',
              'foo',
            ],);
            const options = argsToOptions(args,);

            expect(options.depth,).toBe(3,);
            expect(options.poll,).toBe(500,);
          },
        },),
        it({
          name: 'positive pair flags map to true',
          fn: async function pairFlagsPositive() {
            const args = runParser([
              '-w',
              'src',
              '--hidden',
              '--follow-symlinks',
              '--clear',
              '--process-group',
              '--',
              'foo',
            ],);
            const options = argsToOptions(args,);

            expect(options.hidden,).toBe(true,);
            expect(options.followSymlinks,).toBe(true,);
            expect(options.clear,).toBe(true,);
            expect(options.processGroup,).toBe(true,);
          },
        },),
        it({
          name: 'negative pair flags map to false',
          fn: async function pairFlagsNegative() {
            const args = runParser([
              '-w',
              'src',
              '--no-hidden',
              '--no-follow-symlinks',
              '--no-gitignore',
              '--no-clear',
              '--no-process-group',
              '--',
              'foo',
            ],);
            const options = argsToOptions(args,);

            expect(options.hidden,).toBe(false,);
            expect(options.followSymlinks,).toBe(false,);
            expect(options.gitignore,).toBe(false,);
            expect(options.clear,).toBe(false,);
            expect(options.processGroup,).toBe(false,);
          },
        },),
        it({
          name: 'unset pair flags leave the option undefined',
          fn: async function pairFlagsUnset() {
            const args = runParser(['-w', 'src', '--', 'foo',],);
            const options = argsToOptions(args,);

            expect(options.hidden,).toBeUndefined();
            expect(options.followSymlinks,).toBeUndefined();
            expect(options.gitignore,).toBeUndefined();
            expect(options.clear,).toBeUndefined();
            expect(options.processGroup,).toBeUndefined();
          },
        },),
        it({
          name: '--signal maps to killSignal',
          fn: async function signalRoundTrip() {
            const args = runParser([
              '-w',
              'src',
              '--signal',
              'SIGHUP',
              '--',
              'foo',
            ],);
            const options = argsToOptions(args,);

            expect(options.killSignal,).toBe('SIGHUP',);
          },
        },),
        it({
          name: 'passing both --hidden and --no-hidden throws',
          fn: async function pairBothThrows() {
            const args = runParser([
              '-w',
              'src',
              '--hidden',
              '--no-hidden',
              '--',
              'foo',
            ],);

            expect(function callWithConflict(): void {
              argsToOptions(args,);
            },)
              .toThrow('Cannot pass both --hidden and --no-hidden',);
          },
        },),
        it({
          name: 'invalid --type token throws',
          fn: async function invalidType() {
            const args = runParser([
              '-w',
              'src',
              '--type',
              'symlink',
              '--',
              'foo',
            ],);

            expect(function callWithBogusType(): void {
              argsToOptions(args,);
            },)
              .toThrow('Unknown --type token "symlink"',);
          },
        },),
        it({
          name: 'invalid --signal name throws',
          fn: async function invalidSignal() {
            const args = runParser([
              '-w',
              'src',
              '--signal',
              'SIGNUKE',
              '--',
              'foo',
            ],);

            expect(function callWithBogusSignal(): void {
              argsToOptions(args,);
            },)
              .toThrow('Unknown --signal',);
          },
        },),
        it({
          name: 'invalid --include-regex pattern throws',
          fn: async function invalidRegex() {
            const args = runParser([
              '-w',
              'src',
              '--include-regex',
              '(',
              '--',
              'foo',
            ],);

            expect(function callWithBogusRegex(): void {
              argsToOptions(args,);
            },)
              .toThrow();
          },
        },),
      ],
    },),
  ],
},);
