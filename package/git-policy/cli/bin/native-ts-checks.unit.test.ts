import {
  readFile,
  realpath,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { NativeGateError } from './native-gate-error.ts';
import { isMissingPathError } from './native-gate-paths.ts';
import { runGateCommand } from './native-gate-run.ts';
import {
  failureLinesOf,
  namedChecks,
  recordedCheckTasks,
  resultLinesOf,
  scopeLabelOfTask,
} from './native-ts-checks.ts';
import { fixtureGit } from './install-fixture-checkout-reads-for-tests.ts';
import {
  type ScopeStandIn,
  createScopeStandIn,
  recordedCalls,
} from './stand-in-scope-programs-for-tests.ts';
import { createGateScratchDirectory } from './stand-in-gate-programs-for-tests.ts';

/**
 Path of the command under test, which runs as TypeScript.
 */
const checksPath = join(
  import.meta.dirname,
  'native-ts-checks.ts',
);

/**
 Milliseconds one run of the command under test is given, which is far more than its fixtures need.
 */
const testDeadlineMilliseconds = 120_000;

/**
 The characters one run's identity holds: a UUID's 128 bits written as hexadecimal digits without its hyphens.
 */
const runTokenLength = 32;

/**
 The alphabet a run identity is written in, which is a UUID's lowercased hexadecimal digits.
 */
const runTokenCharacters = '0123456789abcdef';

/**
 Milliseconds the stand-in `mise` keeps running when a test asks it to hang, so its own end cannot be what ended the wait.
 */
const hangMilliseconds = 4_000;

/**
 Seconds the deadline test gives its check, which is the shortest a whole number of seconds allows.
 */
const deadlineSeconds = 1;

/**
 Environment variable that names the task the stand-in `mise` fails.
 */
const failTaskVariable = 'CHECK_TEST_FAIL_TASK';

/**
 Environment variable that names the task the stand-in `mise` keeps running.
 */
const hangTaskVariable = 'CHECK_TEST_HANG_TASK';

/**
 The stand-in `mise`: records the call, then passes, fails with the lines a failing TypeScript check prints, or keeps
 running for a task the fixture named.
 */
const standInMise = `#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const { Atomics, SharedArrayBuffer } = globalThis;
const directory = process.env.CHECK_STAND_IN_DIR;
const args = process.argv.slice(2);
fs.appendFileSync(path.join(directory, 'mise.jsonl'), JSON.stringify({ args, cwd: process.cwd() }) + '\\n');
const task = args[1] ?? '';
if (process.env.${hangTaskVariable} === task) {
  const cell = new Int32Array(new SharedArrayBuffer(4));
  Atomics.wait(cell, 0, 0, ${String(hangMilliseconds)});
  process.stdout.write('the hanging check ended by itself\\n');
  process.exit(0);
}
if (process.env.${failTaskVariable} === task) {
  process.stdout.write('FAIL [   0.4s] (1/2) the suite a test names\\n');
  process.stdout.write('error TS2322: Type string is not assignable to type number.\\n');
  process.stdout.write('  at bin/a.ts:3:1\\n');
  process.exit(1);
}
process.stdout.write('ok 2 tests\\n');
process.exit(0);
`;

/**
 Writes the stand-in `mise` into a scope fixture's own `path` directory, so it goes first on `PATH` beside the stand-in
 `systemd-run` and `systemctl`.

 @param standIn - the fixture whose `path` directory is used

 @param environment - further environment values the stand-in reads

 @returns the environment a check runs with
 */
async function withStandInMise({
  standIn,
  environment = {},
}: {
  readonly standIn: ScopeStandIn;
  readonly environment?: Readonly<Record<string, string>>;
}): Promise<Readonly<Record<string, string>>> {
  await writeFile(
    join(
      standIn.directory,
      'path',
      'mise',
    ),
    standInMise,
    { mode: 0o700, },
  );
  return {
    ...standIn.environment,
    ...environment,
  };
}

/**
 Whether one word is the identity a run draws for its scope names: a UUID with its hyphens removed.

 Written as a scan and not as a pattern, because a regular expression is refused in this package (`no-regex`) and the
 claim here is only a length and an alphabet.

 @param text - the word to read

 @returns true when the word is one run's identity
 */
function isRunTokenOf(text: string): boolean {
  return (text.length === runTokenLength) && text.split('')
    .every(function isTokenCharacter(character) {
      return runTokenCharacters.includes(character);
    });
}

/**
 Reads the file the stand-in `mise` wrote, answering empty text when it never wrote it.

 @param directory - the fixture's directory

 @returns the file's text, or empty text when the stand-in was never called
 */
async function recordedText({ directory }: { readonly directory: string; }): Promise<string> {
  try {
    return await readFile(
      join(
        directory,
        'mise.jsonl'
      ),
      'utf8'
    );
  }
  catch (error) {
    // The stand-in `mise` writes no file until it is first called, which is an answer and not a failure.
    if (isMissingPathError({ error, }))
      return '';
    throw error;
  }
}

/**
 One `mise` call the stand-in recorded.

 @param directory - the fixture's directory

 @returns the argument words and the directory each call ran in
 */
async function recordedMiseCalls({ directory }: { readonly directory: string; }): Promise<readonly {
  readonly args: readonly string[];
  readonly cwd: string;
}[]> {
  /**
   The recorded lines, empty when the stand-in `mise` was never called.
   */
  const text = await recordedText({ directory, });
  return text.split('\n')
    .filter(function hasText(line) {
      return line !== '';
    })
    .map(function parsed(line) {
      return JSON.parse(line) as {
        readonly args: readonly string[];
        readonly cwd: string;
      };
    });
}

/**
 Runs the command under test.

 @param worktree - the checkout the checks run in

 @param args - arguments after the two the fixture always gives

 @param environment - the environment, with the stand-ins first on `PATH`

 @returns how the command ended, with its output
 */
async function tsChecks({
  worktree,
  args,
  environment,
}: {
  readonly worktree: string;
  readonly args: readonly string[];
  readonly environment: Readonly<Record<string, string>>;
}) {
  return await runGateCommand({
    program: process.execPath,
    args: [
      checksPath,
      '--worktree',
      worktree,
      ...args,
    ],
    deadlineMilliseconds: testDeadlineMilliseconds,
    environment,
  });
}

await describe({
  name: 'native:ts-checks, which runs the checks of a change in capped scopes',
  children: [
    it({
      name: 'names a scope label after the task, in the characters a unit name holds',
      fn: async () => {
        expect(scopeLabelOfTask({ task: '//package/git-policy/cli:test:unit', }),)
          .toBe('package-git-policy-cli-test-unit',);
        expect(scopeLabelOfTask({ task: 'lint:types', }),).toBe('lint-types',);
        expect(scopeLabelOfTask({ task: 'A::B', }),).toBe('a-b',);
        /**
         A task name longer than the label limit.
         */
        const long = scopeLabelOfTask({ task: `x:${'y'.repeat(200)}`, });
        expect(long.length,).toBe(48,);
        expect(() => {
          scopeLabelOfTask({ task: '///', });
        },).toThrow(NativeGateError,);
      },
    }),
    it({
      name: 'numbers the label of a second task that writes the same one, so two checks never share a scope',
      fn: async () => {
        /**
         Two tasks whose labels collide: one spelled in another case, and two long names that differ past the limit.
         */
        const long = 'y'.repeat(60);
        expect(namedChecks({
          tasks: [
            'lint:types',
            'LINT::TYPES',
            `x:${long}a`,
            `x:${long}b`,
          ],
        }),).toEqual([
          {
            task: 'lint:types',
            label: 'lint-types',
          },
          {
            task: 'LINT::TYPES',
            label: 'lint-types-2',
          },
          {
            task: `x:${long}a`,
            label: scopeLabelOfTask({ task: `x:${long}a`, }),
          },
          {
            task: `x:${long}b`,
            label: `${scopeLabelOfTask({ task: `x:${long}a`, })}-2`,
          },
        ],);
      },
    }),
    it({
      name: 'keeps the failure lines of a check and then its tail',
      fn: async () => {
        /**
         The ordinary lines a long check prints after its failures.
         */
        const tail = Array.from(
          { length: 30, },
          function ordinary(_unused, index) {
            return `ordinary line ${String(index)}`;
          }
        );
        /**
         One check's output: a passing line, two failures and a long tail.
         */
        const lines = failureLinesOf({
          output: [
            'ok 1 test',
            'error TS2322: Type string is not assignable to type number.',
            'FAIL [   0.4s] (2/3) the suite a test names',
            ...tail,
          ].join('\n',),
        });
        expect(lines.slice(0, 2,),).toEqual([
          'error TS2322: Type string is not assignable to type number.',
          'FAIL [   0.4s] (2/3) the suite a test names',
        ],);
        // Two failure lines and twelve lines of tail, which is what a person needs to see what a check was doing.
        expect(lines,).toHaveLength(14,);
        expect(lines.at(-1),).toBe('ordinary line 29',);
      },
    }),
    it({
      name: 'prints one line per check, in the order the caller named them, and exits zero when all passed',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        /**
         The directory the checks run in.
         */
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ts-checks-', });
        /**
         The environment with the stand-in `mise` first on `PATH`.
         */
        const environment = await withStandInMise({ standIn, });
        /**
         How the command ended.
         */
        const printed = await tsChecks({
          worktree: scratch.path,
          args: ['//package/a:b', '//package/c:d',],
          environment,
        });
        expect(printed.status,).toBe(0,);
        expect(printed.stdout.split('\n',),).toEqual([
          scratch.path,
          '//package/a:b: exit 0',
          '//package/c:d: exit 0',
          '',
        ],);
        /**
         What the stand-ins recorded.
         */
        const starts = await recordedCalls({
          directory: standIn.directory,
          fileName: 'systemd-run.jsonl',
        });
        expect(starts,).toHaveLength(2,);
        expect(starts.every(function isScope(start) {
          return start.includes('--scope',);
        }),).toBe(true,);
        // The checks start at once, so the order two scopes were recorded in is nobody's decision.
        expect(starts.map(function taskOf(start) {
          return start.slice(-2,)
            .join(' ',);
        })
          .toSorted(),).toEqual([
            'run //package/a:b',
            'run //package/c:d',
          ],);
        // A command reports its directory as the filesystem names it, and the scratch parent is reached through one link.
        expect((await recordedMiseCalls({ directory: standIn.directory, })).map(function cwd(call) {
          return call.cwd;
        }),).toEqual([await realpath(scratch.path,), await realpath(scratch.path,),],);
      },
    }),
    it({
      name: 'names the check that failed with its status and prints the lines that say why, exiting 2',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ts-checks-', });
        /**
         The environment whose stand-in `mise` fails one task.
         */
        const environment = await withStandInMise({
          standIn,
          environment: { [failTaskVariable]: '//package/c:d', },
        });
        /**
         How the command ended.
         */
        const printed = await tsChecks({
          worktree: scratch.path,
          args: ['//package/a:b', '//package/c:d',],
          environment,
        });
        expect(printed.status,).toBe(2,);
        expect(printed.stdout,).toContain('//package/a:b: exit 0',);
        expect(printed.stdout,).toContain('//package/c:d: exit 1',);
        expect(printed.stdout,).toContain('error TS2322: Type string is not assignable',);
        expect(printed.stdout,).toContain('FAIL [   0.4s]',);
      },
    }),
    it({
      name: 'ends a check that outlasts its deadline by the name of its scope and exits 2',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ts-checks-', });
        /**
         The environment whose stand-in `mise` keeps running for one task.
         */
        const environment = await withStandInMise({
          standIn,
          environment: { [hangTaskVariable]: '//package/hangs', },
        });
        /**
         How the command ended.
         */
        const printed = await tsChecks({
          worktree: scratch.path,
          args: ['--deadline-seconds', String(deadlineSeconds,), '//package/hangs',],
          environment,
        });
        expect(printed.status,).toBe(2,);
        expect(printed.stdout,).toContain('//package/hangs: stopped at its deadline',);
        /**
         What the stand-in `systemctl` was asked to end.
         */
        const stops = await recordedCalls({
          directory: standIn.directory,
          fileName: 'systemctl.jsonl',
        });
        expect(stops,).toHaveLength(1,);
        expect(stops[0]?.join(' '),).toContain('--kill-whom=all gpc-check-package-hangs-',);
      },
    }),
    it({
      name: 'gives every run an identity of its own in its scope names, so two runs never ask the manager for one name',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ts-checks-', });
        /**
         The environment with the stand-in `mise` first on `PATH`.
         */
        const environment = await withStandInMise({ standIn, });
        /**
         What two simultaneous runs of one check ended with.
         */
        const runs = await Promise.all([
          tsChecks({
            worktree: scratch.path,
            args: ['//package/a:b',],
            environment,
          }),
          tsChecks({
            worktree: scratch.path,
            args: ['//package/a:b',],
            environment,
          }),
        ]);
        for (const printed of runs) {
          expect(printed.status,).toBe(0,);
          expect(printed.stdout,).toContain('//package/a:b: exit 0',);
        }
        /**
         The scope names the two runs asked the manager for.
         */
        const units = (await recordedCalls({
          directory: standIn.directory,
          fileName: 'systemd-run.jsonl',
        })).map(function unitOf(call) {
          return call.find(function isUnit(argument) {
            return argument.startsWith('--unit=',);
          });
        });
        expect(units,).toHaveLength(2,);
        // A shared name is refused by the manager while the first run's scope is active, so the names must differ.
        expect(new Set(units).size,).toBe(2,);
        /**
         The identity each run put in its scope name, which a scope name carries after its stamp.

         The same-second pair of runs is asserted on one stamp in `native-capped-scope-command.unit.test.ts`, where the
         stamp need not be measured; what this checks is that the identity reaches every name a run asks for, so two runs
         whose stamps do fall in one second are already kept apart.
         */
        const runTokens = units.map(function tokenOf(name) {
          return (name ?? '').split('-',)
            .at(-1);
        });
        for (const token of runTokens)
          expect(isRunTokenOf(token ?? ''),).toBe(true,);
        expect(new Set(runTokens).size,).toBe(2,);
      },
    }),
    it({
      name: 'refuses a cap systemd cannot read before any check starts',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ts-checks-', });
        /**
         How the refused command ended.
         */
        const printed = await tsChecks({
          worktree: scratch.path,
          args: ['--unit-io-weight', '30',],
          environment: await withStandInMise({ standIn, }),
        });
        expect(printed.status,).toBe(1,);
        expect(printed.stderr,).toContain('--unit-io-weight 30 is above 10',);
        expect(await recordedCalls({
          directory: standIn.directory,
          fileName: 'systemd-run.jsonl',
        }),).toHaveLength(0,);
      },
    }),
    it({
      name: 'refuses a deadline that would not bound a check, before any scope starts',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ts-checks-', });
        /**
         The refusal a zero deadline gets.
         */
        const zero = await tsChecks({
          worktree: scratch.path,
          args: [
            '--deadline-seconds',
            '0',
            '//package/a:b',
          ],
          environment: await withStandInMise({ standIn, }),
        });
        expect(zero.status,).toBe(1,);
        expect(zero.stderr,).toContain('--deadline-seconds 0',);
        expect(zero.stderr,).toContain('leaves a command with no deadline at all',);
        /**
         The refusal a deadline longer than Node's timers honour gets.
         */
        const oversized = await tsChecks({
          worktree: scratch.path,
          args: [
            '--deadline-seconds',
            '2147484',
            '//package/a:b',
          ],
          environment: await withStandInMise({ standIn, }),
        });
        expect(oversized.status,).toBe(1,);
        expect(oversized.stderr,).toContain('2147483 seconds',);
        expect(await recordedCalls({
          directory: standIn.directory,
          fileName: 'systemd-run.jsonl',
        }),).toHaveLength(0,);
      },
    }),
    it({
      name: 'prints the commit the checkout points at, so a log says what was checked',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ts-checks-', });
        await fixtureGit({
          directory: scratch.path,
          args: ['init', '--quiet', '--initial-branch=main', '.',],
        });
        await writeFile(
          join(
            scratch.path,
            'tracked.txt'
          ),
          'tracked\n'
        );
        await fixtureGit({
          directory: scratch.path,
          args: [
            'add',
            'tracked.txt',
          ],
        });
        await fixtureGit({
          directory: scratch.path,
          args: [
            '-c',
            'user.email=checks-test@example.invalid',
            '-c',
            'user.name=Checks Test',
            'commit',
            '--quiet',
            '-m',
            'first',
            'tracked.txt',
          ],
        });
        /**
         How the command ended.
         */
        const printed = await tsChecks({
          worktree: scratch.path,
          args: ['//package/a:b',],
          environment: await withStandInMise({ standIn, }),
        });
        expect(printed.status,).toBe(0,);
        expect(printed.stdout.split('\n',)[0],).toContain(`${scratch.path} at `,);
        expect(printed.stdout.split('\n',)[0]?.length,).toBe(`${scratch.path} at `.length + 10,);
      },
    }),
    it({
      name: 'runs this package\'s own unit tests and type checks when the caller names none',
      fn: async () => {
        expect(recordedCheckTasks,).toEqual([
          '//package/git-policy/cli:test:unit',
          '//package/git-policy/cli:lint:types',
        ],);
      },
    }),
    it({
      name: 'prints a passed check as its status alone and never the output of a check that passed',
      fn: async () => {
        expect(resultLinesOf({
          check: {
            task: '//a:b',
            scope: 'gpc-check-a-b-20261111T133700-0500',
            outcome: {
              status: 0,
              stdout: 'ok 2 tests\n',
              stderr: '',
              timedOut: false,
            },
          },
        }),).toEqual(['//a:b: exit 0'],);
      },
    }),
  ],
});
