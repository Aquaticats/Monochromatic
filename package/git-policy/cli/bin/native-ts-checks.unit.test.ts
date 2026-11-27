import {
  mkdir,
  readFile,
  realpath,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { NativeGateError } from './native-gate-error.ts';
import { isMissingPathError } from './native-gate-paths.ts';
import { runGateCommand } from './native-gate-run.ts';
import { SCOPE_NOT_READ } from './native-check-scope-accounting.ts';
import type { ScopeOutcome } from './native-capped-scope-command.ts';
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
 Milliseconds from the start of a second at which two runs are started, so both stamp their scopes with that second.

 A scope name carries a second-precise stamp, so two runs started in one second are the case where a run's own identity
 has to keep their names apart.
 */
const startOfSecondOffsetMilliseconds = 1_100;

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
 Environment variable that names the folder the stand-in `git` treats as the checkout's root, which is the one directory it
 answers a command from.
 */
const gitRootVariable = 'CHECK_TEST_GIT_ROOT';

/**
 Environment variable that says what the stand-in `git` answers: the checkout's own commits, no commit at all, or nothing
 whatsoever.
 */
const gitAnswerVariable = 'CHECK_TEST_GIT_ANSWER';

/**
 What the stand-in `git` answers when it is told to answer nothing whatsoever, failing as a git that cannot read the
 repository does.
 */
const gitAnswersNothing = 'nothing';

/**
 What the stand-in `git` answers when it is told to name no commit: it exits zero and prints nothing at all.
 */
const gitAnswersNoCommit = 'no-commit';

/**
 The reason the stand-in `git` gives when it is told to answer nothing, so a control reads the reason the check printed.
 */
const gitCannotAnswerReason = 'the stand-in git cannot answer: it was told to answer nothing at all';

/**
 Environment variable that says how many `[PASS]` suite lines the stand-in `mise` prints.

 A harness run prints one `[info]` `[PASS]` line per suite it ran (`package/test-harness/core/src/describe.ts`, the
 `logPass` line of `runOnce`), and the level and stamp are the console sink's (`formatRecord`,
 `package/logger/core/src/sink/console.ts:214`). This stand-in builds that shape itself, so it says nothing about a real
 harness run.
 */
const passLinesVariable = 'CHECK_TEST_PASS_LINES';

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
const passLines = Number(process.env.${passLinesVariable} ?? '0');
for (let index = 0; index < passLines; index += 1) {
  process.stdout.write('[info] [' + new Date().toISOString() + '] [suite ' + String(index) + '] [PASS] a test (1ms)\\n');
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
 The stand-in `git`: the checkout's own Git with this repository's wrapper rule added, so a command whose effective working
 directory is not the checkout's root is refused.

 The rule and the words it refuses with are the wrapper's (`resolve_require_root`,
 `src/native/command/rule_require_root.rs`), and the line it writes is the finding the wrapper serializes (`event_head` and
 `push_finding`, `src/native/policy/policy_events.rs`). The wrapper's own answer to `git -C <package folder> rev-parse
 --short=10 HEAD` is that finding, which issue 6617 records. The standing Git is `/usr/bin/git`, which the wrapper shadows
 on `PATH`, because no test reads the wrapper's own behaviour from a stand-in.
 */
const standInGit = `#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const directory = process.env.CHECK_STAND_IN_DIR;
const args = process.argv.slice(2);
const root = path.resolve(process.env.${gitRootVariable} ?? '');
// \`-C\` moves the directory the command runs in, each one relative to the one before it, as Git reads it.
let effective = process.cwd();
for (let at = 0; at < args.length; at += 1) {
  if ((args[at] === '-C') && ((at + 1) < args.length)) {
    effective = path.resolve(effective, args[at + 1]);
    at += 1;
  }
}
// Paths compare by components, which is what the wrapper's rule compares.
effective = path.resolve(effective);
const answersNothing = process.env.${gitAnswerVariable} === '${gitAnswersNothing}';
const answersNoCommit = process.env.${gitAnswerVariable} === '${gitAnswersNoCommit}';
const refused = (!answersNothing) && (!answersNoCommit) && (effective !== root);
fs.appendFileSync(path.join(directory, 'git.jsonl'), JSON.stringify({ args, cwd: process.cwd(), effective, refused }) + '\\n');
if (answersNothing) {
  process.stderr.write('fatal: ${gitCannotAnswerReason}\\n');
  process.exit(128);
}
if (answersNoCommit) {
  process.exit(0);
}
if (refused) {
  const message = 'monochromatic-git-policy-cli: not at the root of the git repository. Repo root is ' + root
    + ' but effective cwd is ' + effective + '. This rule exists because agents and humans frequently forget which'
    + ' directory they are in. Tip: cd to ' + root + ' or pass -C ' + root + ' before the subcommand.';
  process.stderr.write(JSON.stringify({
    schemaVersion: 1, sequence: 0, type: 'finding', trigger: 'pre-forward', policyId: 'require-root',
    severity: 'error', code: 'require-root/not-at-root', message, fix: 'none',
  }) + '\\n');
  process.exit(1);
}
const done = spawnSync('/usr/bin/git', args, { stdio: 'inherit' });
process.exit(done.status === null ? 1 : done.status);
`;

/**
 Writes the stand-in `git` into a scope fixture's own `path` directory, so it goes first on `PATH` and the commands the
 checks run ask it, beside the stand-in `systemd-run`, `systemctl` and `mise`.

 @param standIn - the fixture whose `path` directory is used
 @param root - the folder the stand-in answers a command from, which is the checkout's root
 @param environment - further environment values the stand-in reads

 @returns the environment a check runs with
 */
async function withStandInGit({
  standIn,
  root,
  environment = {},
}: {
  readonly standIn: ScopeStandIn;
  readonly root: string;
  readonly environment?: Readonly<Record<string, string>>;
}): Promise<Readonly<Record<string, string>>> {
  await writeFile(
    join(
      standIn.directory,
      'path',
      'git'
    ),
    standInGit,
    { mode: 0o700, },
  );
  return {
    ...environment,
    [gitRootVariable]: root,
  };
}

/**
 Writes a fixture checkout holding one commit, so a control reads a commit that is the checkout's own.

 @param checkout - the folder the checkout is made in
 */
async function writeFixtureCheckout({ checkout }: { readonly checkout: string; }): Promise<void> {
  await fixtureGit({
    directory: checkout,
    args: [
      'init',
      '--quiet',
      '--initial-branch=main',
      '.',
    ],
  });
  await writeFile(
    join(
      checkout,
      'tracked.txt'
    ),
    'tracked\n'
  );
  await fixtureGit({
    directory: checkout,
    args: [
      'add',
      'tracked.txt',
    ],
  });
  await fixtureGit({
    directory: checkout,
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
}

/**
 The commit a fixture checkout points at, read with the checkout's own Git.

 @param checkout - the fixture checkout

 @returns the short commit the checkout points at
 */
async function fixtureHead({ checkout }: { readonly checkout: string; }): Promise<string> {
  return (await fixtureGit({
    directory: checkout,
    args: [
      'rev-parse',
      '--short=10',
      'HEAD',
    ],
  })).trim();
}

/**
 One `git` call the stand-in recorded.

 @param directory - the fixture's directory

 @returns the argument words, the directory it ran in, the directory it ran in once `-C` moved it, and whether the
 stand-in refused it
 */
async function recordedGitCalls({ directory }: { readonly directory: string; }): Promise<readonly {
  readonly args: readonly string[];
  readonly cwd: string;
  readonly effective: string;
  readonly refused: boolean;
}[]> {
  /**
   The recorded lines, empty when the stand-in `git` was never called.
   */
  const text = await recordedText({
    directory,
    fileName: 'git.jsonl',
  });
  return text.split('\n')
    .filter(function hasText(line) {
      return line !== '';
    })
    .map(function parsed(line) {
      return JSON.parse(line) as {
        readonly args: readonly string[];
        readonly cwd: string;
        readonly effective: string;
        readonly refused: boolean;
      };
    });
}

/**
 Writes the unit test files a fixture package holds, named the way the harness discovers them.

 The harness globs `**\/*.unit.test.ts` below the package it runs (`mise.toml`,
 `[task_templates."test:unit"]`), so a fixture package holds however many of these a test writes, and the count a check
 holds its suite lines against is that many.

 @param checkout - the fixture checkout the package sits in

 @param packagePath - the package's path inside the checkout, as a task name spells it

 @param count - how many unit test files the package holds
 */
async function writeUnitTestFiles({
  checkout,
  packagePath,
  count,
}: {
  readonly checkout: string;
  readonly packagePath: string;
  readonly count: number;
},): Promise<void> {
  /**
   The package's own directory, which the files go in.
   */
  const directory = join(
    checkout,
    packagePath
  );
  await mkdir(
    directory,
    { recursive: true, }
  );
  await Promise.all(Array.from(
    { length: count, },
    function fileFor(_unused, index) {
      return writeFile(
        join(
          directory,
          `suite-${String(index)}.unit.test.ts`
        ),
        '',
      );
    },
  ));
}

/**
 Reads a file a stand-in wrote, answering empty text when it never wrote it.

 @param directory - the fixture's directory
 @param fileName - the file's name inside the fixture's directory

 @returns the file's text, or empty text when the stand-in was never called
 */
async function recordedText({
  directory,
  fileName,
}: {
  readonly directory: string;
  readonly fileName: string;
}): Promise<string> {
  try {
    return await readFile(
      join(
        directory,
        fileName
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
  const text = await recordedText({
    directory,
    fileName: 'mise.jsonl',
  });
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

/**
 The figures a check's scope holds after its task bound refused forks.

 These are a stand-in, and no reading of any run stands behind them: the file shapes are the kernel's and the numbers are
 the ones the refused forks of issue 6605 were reported with, written here so the check's own line is asserted without a
 run that has to be brought to its bound.
 */
const refusedForkAccounting: ScopeOutcome['accounting'] = {
  cgroupPath: '/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/app.slice/gpc-check-a-b.scope',
  readAt: '2026-11-27T15:23:29.041Z',
  pidsPeak: 434,
  pidsMax: 400,
  pidsEvents: { max: 6 },
  memoryEvents: {},
  cpuStat: {},
};

/**
 One check's result, as the check command reads it.

 @param outcome - how the check ended and what its scope counted

 @returns the check and its outcome, named as the command names them
 */
function checkRun({ outcome }: { readonly outcome: ScopeOutcome; },) {
  return {
    task: '//package/a:b',
    scope: 'gpc-check-a-b-20261127T152200-0500',
    outcome,
  };
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
        /**
         Everything it printed, whose first line says the checkout it ran in and the commit it read there.
         */
        const lines = printed.stdout.split('\n',);
        // The folder this test owns is no checkout, so the check says so where a log looks for the commit it ran on.
        expect(lines[0],).toContain(`${scratch.path}: `,);
        expect(lines[0],).toContain('the commit could not be read',);
        expect(lines.slice(1,),).toEqual([
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
      name: 'names each run\'s scopes for that run, so two runs of one check in one second both run',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ts-checks-', });
        /**
         The environment with the stand-in `mise` first on `PATH`.
         */
        const environment = await withStandInMise({ standIn, });
        // Both runs start inside one second, which is the case the second-precise stamp alone cannot tell apart.
        await delay(startOfSecondOffsetMilliseconds - (Date.now() % 1_000));
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
        await writeFixtureCheckout({ checkout: scratch.path, });
        /**
         The commit the fixture checkout points at, which the check has to print.
         */
        const head = await fixtureHead({ checkout: scratch.path, });
        /**
         How the command ended.
         */
        const printed = await tsChecks({
          worktree: scratch.path,
          args: ['//package/a:b',],
          environment: await withStandInMise({ standIn, }),
        });
        expect(printed.status,).toBe(0,);
        expect(printed.stdout.split('\n',)[0],).toBe(`${scratch.path} at ${head}`,);
      },
    }),
    it({
      name: 'reads the commit from the checkout\'s root, where the wrapper lets a git command run, when it starts in a package folder',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ts-checks-', });
        await writeFixtureCheckout({ checkout: scratch.path, });
        /**
         The package folder the checks run in, which `mise` gives a package's own task as its working directory.
         */
        const packageDirectory = join(
          scratch.path,
          'package',
          'a',
        );
        await mkdir(
          packageDirectory,
          { recursive: true, },
        );
        /**
         The commit the fixture checkout points at, which the check has to print.
         */
        const head = await fixtureHead({ checkout: scratch.path, });
        /**
         How the command ended, with a `git` that refuses every command run away from the checkout's root.
         */
        const printed = await tsChecks({
          worktree: packageDirectory,
          args: ['//package/a:b',],
          environment: await withStandInGit({
            standIn,
            root: scratch.path,
            environment: await withStandInMise({ standIn, }),
          }),
        });
        expect(printed.status,).toBe(0,);
        expect(printed.stdout.split('\n',)[0],).toBe(`${packageDirectory} at ${head}`,);
        /**
         Every `git` call the check made.
         */
        const calls = await recordedGitCalls({ directory: standIn.directory, });
        expect(calls.length,).toBeGreaterThan(0,);
        // Not one call was refused, and each ran in the checkout's root rather than in the package folder the check started in.
        expect(calls.filter(function refused(call) {
          return call.refused;
        }),).toEqual([],);
        expect(calls.every(function fromTheRoot(call) {
          return call.effective === scratch.path;
        }),).toBe(true,);
      },
    }),
    it({
      name: 'says the commit could not be read and why, never silence, when git cannot answer',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ts-checks-', });
        await writeFixtureCheckout({ checkout: scratch.path, });
        /**
         How the command ended, with a `git` that answers nothing at all.
         */
        const printed = await tsChecks({
          worktree: scratch.path,
          args: ['//package/a:b',],
          environment: await withStandInGit({
            standIn,
            root: scratch.path,
            environment: {
              ...(await withStandInMise({ standIn, })),
              [gitAnswerVariable]: gitAnswersNothing,
            },
          }),
        });
        expect(printed.status,).toBe(0,);
        /**
         The line the check printed in place of the checkout's path and commit.
         */
        const firstLine = printed.stdout.split('\n',)[0] ?? '';
        expect(firstLine,).toContain(`${scratch.path}: `,);
        expect(firstLine,).toContain('the commit could not be read',);
        // The reason is git's own, so a person reads why the log holds no commit instead of finding the line missing.
        expect(firstLine,).toContain(gitCannotAnswerReason,);
      },
    }),
    it({
      name: 'says the commit could not be read when git names no commit, never silence',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ts-checks-', });
        await writeFixtureCheckout({ checkout: scratch.path, });
        /**
         How the command ended, with a `git` that exits zero and prints nothing.
         */
        const printed = await tsChecks({
          worktree: scratch.path,
          args: ['//package/a:b',],
          environment: await withStandInGit({
            standIn,
            root: scratch.path,
            environment: {
              ...(await withStandInMise({ standIn, })),
              [gitAnswerVariable]: gitAnswersNoCommit,
            },
          }),
        });
        expect(printed.status,).toBe(0,);
        /**
         The line the check printed in place of the checkout's path and commit.
         */
        const firstLine = printed.stdout.split('\n',)[0] ?? '';
        expect(firstLine,).toContain('the commit could not be read',);
        expect(firstLine,).toContain('printed no commit',);
      },
    }),
    it({
      name: 'says the commit could not be read when git cannot be run at all, and names why',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ts-checks-', });
        await writeFixtureCheckout({ checkout: scratch.path, });
        /**
         How the command ended, with `git` on no `PATH` at all and the fixture's own stand-ins on it.
         */
        const printed = await tsChecks({
          worktree: scratch.path,
          args: ['//package/a:b',],
          environment: {
            ...(await withStandInMise({ standIn, })),
            // The stand-ins are node programs, so node's own folder stays on the path and no folder holding a git does.
            PATH: `${join(
              standIn.directory,
              'path'
            )}:${dirname(process.execPath,)}`,
          },
        });
        expect(printed.status,).toBe(0,);
        /**
         The line the check printed in place of the checkout's path and commit.
         */
        const firstLine = printed.stdout.split('\n',)[0] ?? '';
        expect(firstLine,).toContain('the commit could not be read',);
        expect(firstLine,).toContain('could not be run',);
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
        // A task that names no package's unit tests and no type check has no count to earn, so this directory is never
        // read for it.
        const result = await resultLinesOf({
          check: checkRun({
            outcome: {
              status: 0,
              stdout: 'ok 2 tests\n',
              stderr: '',
              timedOut: false,
              elapsedMilliseconds: 16_097,
              accounting: SCOPE_NOT_READ,
            },
          }),
          worktree: import.meta.dirname,
        });
        expect(result.lines,).toEqual(['//package/a:b: exit 0'],);
        expect(result.passed,).toBe(true,);
      },
    }),
    it({
      name: 'says its scope\'s bound and not a failing test refused the fork, beside the lines that say why it failed',
      fn: async () => {
        /**
         The lines the check printed for a scope that counted refused forks.
         */
        const result = await resultLinesOf({
          check: checkRun({
            outcome: {
              status: 1,
              stdout: 'FAIL [   0.4s] (2/3) a test that needed a process\n',
              stderr: '',
              timedOut: false,
              elapsedMilliseconds: 16_097,
              accounting: refusedForkAccounting,
            },
          }),
          worktree: import.meta.dirname,
        });
        /**
         The lines themselves.
         */
        const { lines, } = result;
        expect(lines[0],).toBe('//package/a:b: exit 1',);
        expect(lines.slice(1,),).toContain('FAIL [   0.4s] (2/3) a test that needed a process',);
        // A failing check keeps exactly what its own output said, so the run does not count it as passed.
        expect(result.passed,).toBe(false,);
        /**
         Everything the check printed, which the claim about the bound is read from.
         */
        const printed = lines.join('\n',);
        // Six forks were refused at a bound of 400 tasks, which is the bound and not a test's own failure.
        expect(printed,).toContain('6 refused forks',);
        expect(printed,).toContain('400-task bound',);
        expect(printed,).toContain('That is the bound and not a failing test',);
        expect(printed,).toContain('--unit-tasks-max',);
      },
    }),
    it({
      name: 'refuses a passing check that printed no suite line',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ts-checks-', });
        // Two unit test files, so two `[PASS]` suite lines are the least this package's run could report.
        await writeUnitTestFiles({
          checkout: scratch.path,
          packagePath: 'package/a',
          count: 2,
        });
        /**
         How the command ended: its check exited 0 and printed no suite line at all.
         */
        const printed = await tsChecks({
          worktree: scratch.path,
          args: ['//package/a:test:unit',],
          environment: await withStandInMise({
            standIn,
            environment: { [passLinesVariable]: '0', },
          }),
        });
        expect(printed.status,).toBe(2,);
        expect(printed.stdout,).toContain('no [PASS] suite line',);
        expect(printed.stdout,).toContain('2 unit test files',);
        // A run that checked nothing is not a pass, and says so rather than reading as one.
        expect(printed.stdout,).toContain('this run does not count as passed',);
      },
    }),
    it({
      name: 'refuses a passing check that reported fewer suites than the package holds files',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ts-checks-', });
        await writeUnitTestFiles({
          checkout: scratch.path,
          packagePath: 'package/a',
          count: 2,
        });
        /**
         How the command ended: one suite reported, and the package holds two files.
         */
        const printed = await tsChecks({
          worktree: scratch.path,
          args: ['//package/a:test:unit',],
          environment: await withStandInMise({
            standIn,
            environment: { [passLinesVariable]: '1', },
          }),
        });
        expect(printed.status,).toBe(2,);
        // The refusal names the count it saw and the count it expected.
        expect(printed.stdout,).toContain('1 suite reported [PASS]',);
        expect(printed.stdout,).toContain('2 unit test files',);
      },
    }),
    it({
      name: 'names the files the package holds and the suites that reported [PASS]',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ts-checks-', });
        await writeUnitTestFiles({
          checkout: scratch.path,
          packagePath: 'package/a',
          count: 1,
        });
        /**
         How the command ended: one file, and three suites inside it reporting.
         */
        const printed = await tsChecks({
          worktree: scratch.path,
          args: ['//package/a:test:unit',],
          environment: await withStandInMise({
            standIn,
            environment: { [passLinesVariable]: '3', },
          }),
        });
        expect(printed.status,).toBe(0,);
        expect(printed.stdout,).toContain('//package/a:test:unit: exit 0',);
        expect(printed.stdout,).toContain('1 unit test file, and 3 suites reported [PASS]',);
      },
    }),
    it({
      name: 'says a passing type check reports no count of the files it checked',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-ts-checks-', });
        /**
         How the command ended.
         */
        const printed = await tsChecks({
          worktree: scratch.path,
          args: ['//package/a:lint:types',],
          environment: await withStandInMise({ standIn, }),
        });
        expect(printed.status,).toBe(0,);
        expect(printed.stdout,).toContain('reports no count of the files it checked',);
      },
    }),
  ],
});
