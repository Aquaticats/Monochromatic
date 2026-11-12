import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runGateCommand } from './native-gate-run.ts';
import {
  installSteps,
  journalIdentifiers,
  journalSince,
  lockFileName,
  newestTimingLine,
  stepNameOf,
  summaryLinesOf,
  timingReportLine,
  wrapperVersionLine,
} from './native-install-steps.ts';
import {
  commitSubjects,
  committedFileText,
  fileTextOf,
  fixtureGit,
  stagedPathNames,
} from './install-fixture-checkout-reads-for-tests.ts';
import {
  type InstallFixture,
  createInstallFixture,
} from './stand-in-install-programs-for-tests.ts';
import { recordedCalls } from './stand-in-scope-programs-for-tests.ts';

/**
 Path of the command under test, which runs as TypeScript.
 */
const installPath = join(
  import.meta.dirname,
  'native-install.ts',
);

/**
 Milliseconds one run of the command under test is given, which is far more than its fixtures need.
 */
const testDeadlineMilliseconds = 120_000;

/**
 The release the fixtures install.
 */
const releaseVersion = '0.6.1-alpha.10';

/**
 The release the fixtures start from, which is the one their lockfile names.
 */
const earlierVersion = '0.6.1-alpha.9';

/**
 A release another process writes into the lockfile while an install of {@link releaseVersion} runs.
 */
const concurrentlyWrittenVersion = '0.6.1-alpha.11';

/**
 One line of the wrapper's own commit timing record, which a fixture writes beside the common Git directory.
 */
const recordedTimingLine = '{"schema":1,"step":"commit","durationMs":37}';

/**
 Milliseconds the moment a journal read starts from may sit away from the test's own clock.

 The install reads the journal from two seconds before the sample commit began, and that commit is the one this test just
 made, so the two moments are the same minute; a minute of tolerance is far more than a slow fixture adds and still fails
 a fixed date, which is what this test used to hold and what stopped passing the day after it was written.
 */
const journalSinceToleranceMilliseconds = 60_000;

/**
 Runs the command under test against one fixture.

 @param fixture - the throwaway checkout to install into

 @param args - the arguments after the `--repository` pair

 @returns how the command ended, with its output
 */
async function install({
  fixture,
  args,
  repository = fixture.repository,
}: {
  readonly fixture: InstallFixture;
  readonly args: readonly string[];
  readonly repository?: string;
}) {
  return await runGateCommand({
    program: process.execPath,
    args: [
      installPath,
      '--repository',
      repository,
      ...args,
    ],
    deadlineMilliseconds: testDeadlineMilliseconds,
    environment: fixture.environment,
  });
}

await describe({
  name: 'native:install, which installs a release and samples one commit with it',
  children: [
    it({
      name: 'names the release text an installed wrapper prints',
      fn: async () => {
        expect(wrapperVersionLine({ version: releaseVersion, }),).toBe('monochromatic-git-policy-cli 0.6.1-alpha.10,',);
      },
    }),
    it({
      name: 'keeps only the lines of a step that carry its summary, and never more than a screenful',
      fn: async () => {
        /**
         Output with one line the report keeps and many it does not.
         */
        const output = [
          'resolving cargo:monochromatic-git-policy-cli',
          'moved cargo:monochromatic-git-policy-cli 0.6.1-alpha.9 -> 0.6.1-alpha.10',
          'wrote toolchain.lock.json',
          ...Array.from(
            { length: 60, },
            function noisy(_unused, index) {
              return `fetching layer ${String(index)}`;
            }
          ),
        ].join('\n',);
        expect(summaryLinesOf({
          output,
          markers: [
            'moved',
            'wrote',
          ],
        }),).toEqual([
          'moved cargo:monochromatic-git-policy-cli 0.6.1-alpha.9 -> 0.6.1-alpha.10',
          'wrote toolchain.lock.json',
        ],);
        /**
         A step whose every line is a summary, which the report cuts to one screenful.
         */
        const everyLine = Array.from(
          { length: 60, },
          function moved(_unused, index) {
            return `moved tool ${String(index)}`;
          }
        ).join('\n',);
        expect(summaryLinesOf({
          output: everyLine,
          markers: ['moved',],
        }),).toHaveLength(40,);
      },
    }),
    it({
      name: 'reads the newest line of a timing record, and says so when it holds none',
      fn: async () => {
        expect(newestTimingLine({ text: '', }),).toEqual({
          line: '',
          count: 0,
        },);
        expect(newestTimingLine({ text: '{"a":1}\n{"b":2}\n', }),).toEqual({
          line: '{"b":2}',
          count: 2,
        },);
        expect(timingReportLine({
          timing: {
            line: '',
            count: 0,
          },
        }),).toContain('holds no line yet',);
        expect(timingReportLine({
          timing: {
            line: '{"b":2}',
            count: 2,
          },
        }),).toContain('== newest timing line (2 lines in all)\n{"b":2}',);
      },
    }),
    it({
      name: 'writes the moment the journal is read from as journalctl reads it',
      fn: async () => {
        expect(journalSince({ moment: new Date('2026-11-11T18:37:02.123Z'), }),).toBe('2026-11-11 18:37:02 UTC',);
      },
    }),
    it({
      name: 'plans the release resolution, the install and then the lock commit, and names each step',
      fn: async () => {
        /**
         The three steps of one install.
         */
        const steps = installSteps({
          repository: '/repo',
          version: releaseVersion,
        });
        expect(steps.map(function words(step) {
          return stepNameOf({ step, });
        }),).toEqual([
          'mise run toolchain:update -- cargo:monochromatic-git-policy-cli',
          'mise run toolchain:install',
          `git commit --message=chore(toolchain): monochromatic-git-policy-cli ${releaseVersion} -- ${lockFileName}`,
        ],);
        expect(steps.every(function checkout(step) {
          return step.repository === '/repo';
        }),).toBe(true,);
      },
    }),
    it({
      name: 'installs the release, commits the lock it moved, and prints the timing line and the journal of the sample',
      fn: async () => {
        await using fixture = await createInstallFixture({
          version: releaseVersion,
          options: { timingLine: recordedTimingLine, },
        });
        /**
         How the install ended.
         */
        const printed = await install({
          fixture,
          args: [releaseVersion,],
        });
        expect(printed.status,).toBe(0,);
        expect(printed.stdout,).toContain('moved cargo:monochromatic-git-policy-cli',);
        expect(printed.stdout,).toContain(`+{"tools":{"cargo:monochromatic-git-policy-cli":"${releaseVersion}"}}`,);
        expect(printed.stdout,).toContain(`installed cargo:monochromatic-git-policy-cli ${releaseVersion}`,);
        expect(printed.stdout,).toContain(wrapperVersionLine({ version: releaseVersion, }),);
        expect(printed.stdout,).toContain('== the sample commit took ',);
        expect(printed.stdout,).toContain('== newest timing line (',);
        expect(printed.stdout,).toContain('== service journal since the sample commit began',);
        expect(printed.stdout,).toContain('the sample commit was answered',);
        expect(printed.stdout,).toContain(`installed ${releaseVersion} and committed the lock`,);
        expect(await commitSubjects({ repository: fixture.repository, }),).toEqual([
          `chore(toolchain): monochromatic-git-policy-cli ${releaseVersion}`,
          'first',
        ],);
        expect(await fileTextOf({
          repository: fixture.repository,
          name: lockFileName,
        }),).toContain(releaseVersion,);
        /**
         What the fixture recorded of the journal read.
         */
        const journalCalls = await recordedCalls({
          directory: fixture.directory,
          fileName: 'journalctl.jsonl',
        });
        expect(journalCalls,).toHaveLength(1,);
        /**
         The moment the journal is read from, which the install puts in the argument `journalctl` reads.
         */
        const sinceArgument = journalCalls[0]?.find(function isSince(argument) {
          return argument.startsWith('--since=',);
        });
        expect(sinceArgument,).toBeDefined();
        /**
         That moment as a number, parsed from `journalctl`'s own text: `2026-11-12 00:20:23 UTC`.
         */
        const sinceMoment = Date.parse((sinceArgument ?? '')
          .slice('--since='.length)
          .replace(' UTC', 'Z')
          .replace(' ', 'T'));
        expect(Number.isSafeInteger(sinceMoment),).toBe(true,);
        expect(Math.abs(Date.now() - sinceMoment),).toBeLessThan(journalSinceToleranceMilliseconds,);
        for (const identifier of journalIdentifiers)
          expect(journalCalls[0],).toContain(identifier,);
      },
    }),
    it({
      name: 'says nothing was committed when the lock already names the release, and reads no journal',
      fn: async () => {
        await using fixture = await createInstallFixture({
          version: earlierVersion,
          options: { lockUnchanged: true, },
        });
        /**
         How the install ended.
         */
        const printed = await install({
          fixture,
          args: [earlierVersion,],
        });
        expect(printed.status,).toBe(0,);
        expect(printed.stdout,).toContain(`${lockFileName} did not change`,);
        expect(printed.stdout,).not.toContain('== newest timing line (',);
        expect(printed.stdout,).toContain(`installed ${earlierVersion}; ${lockFileName} already named it, so nothing was committed`,);
        expect(await commitSubjects({ repository: fixture.repository, }),).toEqual(['first',],);
        expect(await recordedCalls({
          directory: fixture.directory,
          fileName: 'journalctl.jsonl',
        }),).toHaveLength(0,);
      },
    }),
    it({
      name: 'commits a lock the update step resolved and an earlier step had already staged',
      fn: async () => {
        await using fixture = await createInstallFixture({
          version: releaseVersion,
          options: {
            lockUnchanged: true,
            stagedLockVersion: releaseVersion,
          },
        });
        /**
         How the install ended.
         */
        const printed = await install({
          fixture,
          args: [releaseVersion,],
        });
        expect(printed.status,).toBe(0,);
        expect(printed.stdout,).toContain(`+{"tools":{"cargo:monochromatic-git-policy-cli":"${releaseVersion}"}}`,);
        expect(printed.stdout,).not.toContain(`${lockFileName} did not change`,);
        expect(printed.stdout,).toContain(`installed ${releaseVersion} and committed the lock`,);
        expect(await commitSubjects({ repository: fixture.repository, }),).toEqual([
          `chore(toolchain): monochromatic-git-policy-cli ${releaseVersion}`,
          'first',
        ],);
        expect(await committedFileText({
          repository: fixture.repository,
          name: lockFileName,
        }),).toContain(releaseVersion,);
        // The release's own lock is committed, and nothing of it is left staged.
        expect(await stagedPathNames({ repository: fixture.repository, }),).toEqual([],);
      },
    }),
    it({
      name: 'refuses to commit a lock another process wrote while the release was installed',
      fn: async () => {
        await using fixture = await createInstallFixture({
          version: releaseVersion,
          options: { concurrentLockVersion: concurrentlyWrittenVersion, },
        });
        /**
         How the refused install ended.
         */
        const printed = await install({
          fixture,
          args: [releaseVersion,],
        });
        expect(printed.status,).toBe(1,);
        expect(printed.stderr,).toContain('written by something else',);
        expect(printed.stderr,).toContain('nothing was committed',);
        // The refusal prints what holds the lock now, so a person reads the change that was not committed.
        expect(printed.stdout,).toContain(concurrentlyWrittenVersion,);
        expect(await commitSubjects({ repository: fixture.repository, }),).toEqual(['first',],);
        // The other process's write is left exactly where it is.
        expect(await fileTextOf({
          repository: fixture.repository,
          name: lockFileName,
        }),).toContain(concurrentlyWrittenVersion,);
        expect(await committedFileText({
          repository: fixture.repository,
          name: lockFileName,
        }),).toContain(earlierVersion,);
      },
    }),
    it({
      name: 'says nothing was committed when another install of the release committed the lock first',
      fn: async () => {
        await using fixture = await createInstallFixture({
          version: releaseVersion,
          options: { concurrentCommitAt: 'beforeTheDecision', },
        });
        /**
         How the install ended.
         */
        const printed = await install({
          fixture,
          args: [releaseVersion,],
        });
        expect(printed.status,).toBe(0,);
        expect(printed.stdout,).toContain(`installed ${releaseVersion}; ${lockFileName} already named it, so nothing was committed`,);
        expect(await commitSubjects({ repository: fixture.repository, }),).toEqual([
          `chore(toolchain): monochromatic-git-policy-cli ${releaseVersion}`,
          'first',
        ],);
        expect(await committedFileText({
          repository: fixture.repository,
          name: lockFileName,
        }),).toContain(releaseVersion,);
        // No sample commit was made here, so no journal is read for one.
        expect(await recordedCalls({
          directory: fixture.directory,
          fileName: 'journalctl.jsonl',
        }),).toHaveLength(0,);
      },
    }),
    it({
      name: 'records nothing and exits 0 when another install commits the release while this commit runs',
      fn: async () => {
        await using fixture = await createInstallFixture({
          version: releaseVersion,
          options: { concurrentCommitAt: 'atTheCommit', },
        });
        /**
         How the install ended, whose commit step found the lock already committed.
         */
        const printed = await install({
          fixture,
          args: [releaseVersion,],
        });
        expect(printed.status,).toBe(0,);
        expect(printed.stderr,).not.toContain('did not succeed',);
        expect(printed.stdout,).toContain(`installed ${releaseVersion}; ${lockFileName} already named it, so nothing was committed`,);
        // The failing step's own words are printed, so a log reads what Git answered instead of a bare exit 0.
        expect(printed.stdout,).toContain('== the commit step did not succeed: ');
        expect(printed.stdout,).toContain('nothing to commit',);
        // And that the release is recorded all the same, so the install is not reported as a failure.
        expect(printed.stdout,).toContain(`HEAD holds ${lockFileName}, so the release is recorded`,);
        expect(await commitSubjects({ repository: fixture.repository, }),).toEqual([
          `chore(toolchain): monochromatic-git-policy-cli ${releaseVersion}`,
          'first',
        ],);
        expect(await committedFileText({
          repository: fixture.repository,
          name: lockFileName,
        }),).toContain(releaseVersion,);
        // No sample commit was made here, so no journal is read for one.
        expect(await recordedCalls({
          directory: fixture.directory,
          fileName: 'journalctl.jsonl',
        }),).toHaveLength(0,);
      },
    }),
    it({
      name: 'installs into a checkout with no commit yet, where Git refuses HEAD as a revision',
      fn: async () => {
        await using fixture = await createInstallFixture({
          version: releaseVersion,
          options: { initialCommit: false, },
        });
        /**
         How the install ended in a checkout whose HEAD is unborn.
         */
        const printed = await install({
          fixture,
          args: [releaseVersion,],
        });
        expect(printed.status,).toBe(0,);
        // A revision a commit-less checkout cannot answer is never asked for, so no raw Git error reaches the caller.
        expect(printed.stderr,).not.toContain('bad revision',);
        expect(printed.stdout,).toContain(`installed ${releaseVersion}; the checkout has no commit yet, so ${lockFileName} is left for the first one`,);
        // The release the tools resolved is what the worktree holds, and nothing was staged behind the caller's back.
        expect(await fileTextOf({
          repository: fixture.repository,
          name: lockFileName,
        }),).toContain(releaseVersion,);
        expect(await stagedPathNames({ repository: fixture.repository, }),).toEqual([],);
      },
    }),
    it({
      name: 'refuses when the installed wrapper is not the release asked for, and commits nothing',
      fn: async () => {
        await using fixture = await createInstallFixture({ version: earlierVersion, });
        /**
         How the refused install ended.
         */
        const printed = await install({
          fixture,
          args: [releaseVersion,],
        });
        expect(printed.status,).toBe(1,);
        expect(printed.stderr,).toContain(`does not name ${releaseVersion}`,);
        expect(printed.stderr,).toContain('mise run toolchain:install',);
        expect(await commitSubjects({ repository: fixture.repository, }),).toEqual(['first',],);
      },
    }),
    it({
      name: 'names the step and its own last lines when a step fails, before the installed version is read',
      fn: async () => {
        await using fixture = await createInstallFixture({
          version: releaseVersion,
          options: { failStep: 'update', },
        });
        /**
         How the refused install ended.
         */
        const printed = await install({
          fixture,
          args: [releaseVersion,],
        });
        expect(printed.status,).toBe(1,);
        expect(printed.stderr,).toContain('the step mise run toolchain:update -- cargo:monochromatic-git-policy-cli did not succeed',);
        expect(printed.stderr,).toContain('mise run failed with status 1',);
        expect(printed.stderr,).toContain('the registry refused the resolution',);
        expect(printed.stdout,).not.toContain('git version',);
      },
    }),
    it({
      name: 'prints what it would do and changes nothing when it is a dry run',
      fn: async () => {
        await using fixture = await createInstallFixture({ version: releaseVersion, });
        /**
         How the dry run ended.
         */
        const printed = await install({
          fixture,
          args: [
            '--dry-run',
            releaseVersion,
          ],
        });
        expect(printed.status,).toBe(0,);
        expect(printed.stdout,).toContain('would, in',);
        expect(printed.stdout,).toContain('mise run toolchain:update',);
        expect(printed.stdout,).toContain('mise run toolchain:install',);
        expect(printed.stdout,).toContain('git commit',);
        expect(await recordedCalls({
          directory: fixture.directory,
          fileName: 'mise.jsonl',
        }),).toHaveLength(0,);
        expect(await commitSubjects({ repository: fixture.repository, }),).toEqual(['first',],);
        expect(await fileTextOf({
          repository: fixture.repository,
          name: lockFileName,
        }),).toContain(earlierVersion,);
      },
    }),
    it({
      name: 'refuses a command line that names no release, or two',
      fn: async () => {
        await using fixture = await createInstallFixture({ version: releaseVersion, });
        /**
         A run with no release at all.
         */
        const none = await install({
          fixture,
          args: [],
        });
        expect(none.status,).toBe(1,);
        expect(none.stderr,).toContain('needs the release to install',);
        /**
         A run that names two releases.
         */
        const two = await install({
          fixture,
          args: [releaseVersion, earlierVersion,],
        });
        expect(two.status,).toBe(1,);
        expect(two.stderr,).toContain('is a second',);
      },
    }),
    it({
      name: 'refuses an option it does not take, instead of reading it as a release',
      fn: async () => {
        await using fixture = await createInstallFixture({ version: releaseVersion, });
        /**
         How the refused run ended.
         */
        const printed = await install({
          fixture,
          args: [
            '--releaze',
            releaseVersion,
          ],
        });
        expect(printed.status,).toBe(1,);
        expect(printed.stderr,).toContain('--releaze',);
        expect(printed.stderr,).toContain('This command takes --repository, --deadline-seconds, --dry-run',);
      },
    }),
    it({
      name: 'refuses a deadline that would not bound a step, before any step runs',
      fn: async () => {
        await using fixture = await createInstallFixture({ version: releaseVersion, });
        /**
         The refusal a zero deadline gets.
         */
        const zero = await install({
          fixture,
          args: [
            '--deadline-seconds',
            '0',
            releaseVersion,
          ],
        });
        expect(zero.status,).toBe(1,);
        expect(zero.stderr,).toContain('--deadline-seconds 0',);
        expect(zero.stderr,).toContain('leaves a command with no deadline at all',);
        /**
         The refusal a deadline longer than Node's timers honour gets.
         */
        const oversized = await install({
          fixture,
          args: [
            '--deadline-seconds',
            '2147484',
            releaseVersion,
          ],
        });
        expect(oversized.status,).toBe(1,);
        expect(oversized.stderr,).toContain('2147483 seconds',);
        // Neither refusal may start a step: a bound that does not bound is found before the first one.
        expect(await recordedCalls({
          directory: fixture.directory,
          fileName: 'mise.jsonl',
        }),).toHaveLength(0,);
        expect(await commitSubjects({ repository: fixture.repository, }),).toEqual(['first',],);
      },
    }),
    it({
      name: 'reads the timing record beside the common Git directory, which a linked worktree does not hold itself',
      fn: async () => {
        await using fixture = await createInstallFixture({
          version: releaseVersion,
          options: { timingLine: recordedTimingLine, },
        });
        /**
         A worktree of the fixture, whose own `.git` is a file naming the common directory the record sits in.
         */
        const worktree = join(
          fixture.directory,
          'worktree',
        );
        await fixtureGit({
          directory: fixture.repository,
          args: [
            'worktree',
            'add',
            '--quiet',
            '--detach',
            worktree,
          ],
        });
        /**
         How the install in that worktree ended.
         */
        const printed = await install({
          fixture,
          args: [releaseVersion,],
          repository: worktree,
        });
        expect(printed.status,).toBe(0,);
        expect(printed.stdout,).toContain('== newest timing line (',);
        expect(printed.stdout,).not.toContain('holds no line yet',);
      },
    }),
  ],
});
