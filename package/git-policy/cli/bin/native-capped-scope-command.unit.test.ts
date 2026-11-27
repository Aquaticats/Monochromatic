import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { realpath } from 'node:fs/promises';
import { join } from 'node:path';

import {
  SCOPE_NOT_READ,
  checkScopeCgroupPath,
} from './native-check-scope-accounting.ts';
import {
  checkScopeName,
  checkedScopeCaps,
  defaultCheckScopeCaps,
  endCappedScope,
  runInCappedScope,
  runTokenOfScopes,
  scopeArguments,
  scopeCapsProblems,
} from './native-capped-scope-command.ts';
import { createGateScratchDirectory } from './stand-in-gate-programs-for-tests.ts';
import { NativeGateError } from './native-gate-error.ts';
import {
  type ScopeStandIn,
  createScopeStandIn,
  recordedCalls,
  scopeStandInStopFailureVariable,
} from './stand-in-scope-programs-for-tests.ts';

/**
 A scope name the tests start, which no real run of this machine ever uses.
 */
const testScopeName = 'gpc-check-unit-test-20261111T133700-0500';

/**
 Milliseconds one deadline test gives its command, which is short so the test ends quickly.
 */
const deadlineMilliseconds = 1_000;

/**
 Milliseconds the command of a deadline test keeps running after it, so its own end cannot be what ended the wait.
 */
const outlastMilliseconds = 4_000;

/**
 Milliseconds a test's own run may take, which is far more than any fixture here needs.
 */
const testDeadlineMilliseconds = 60_000;

/**
 Milliseconds a fixture's scope runs for, which is long enough for the accounting watch to read it several times.
 */
const fixtureScopeMilliseconds = 400;

/**
 Milliseconds between two reads of a fixture's scope, which is short so the fixture needs no long hold.
 */
const fixturePollMilliseconds = 20;

/**
 Reads every call one stand-in recorded, so the assertions name what a scope was started and ended with.
 */
type RecordedCalls = readonly (readonly string[])[];

/**
 The scope starts and stops of one fixture.

 @param standIn - the fixture whose calls are read

 @returns what the stand-in `systemd-run` and `systemctl` were asked, each in the order it was asked
 */
async function called({
  standIn,
}: {
  readonly standIn: ScopeStandIn;
}): Promise<{ readonly starts: RecordedCalls; readonly stops: RecordedCalls; }> {
  return {
    starts: await recordedCalls({
      directory: standIn.directory,
      fileName: 'systemd-run.jsonl',
    }),
    stops: await recordedCalls({
      directory: standIn.directory,
      fileName: 'systemctl.jsonl',
    }),
  };
}

await describe({
  name: 'the capped scope runner, which starts one bounded command per check',
  children: [
    it({
      name: 'names a scope after what it covers, the moment it started, and the run it belongs to',
      fn: async () => {
        expect(checkScopeName({
          label: 'git-policy-cli-test-unit',
          stamp: '20261111T133700-0500',
          runToken: 'a1b2c3d4',
        }),).toBe('gpc-check-git-policy-cli-test-unit-20261111T133700-0500-a1b2c3d4',);
        // Two runs of one check started in the same second must not ask the manager for one name.
        expect(checkScopeName({
          label: 'git-policy-cli-test-unit',
          stamp: '20261111T133700-0500',
          runToken: 'e5f60718',
        }),).not.toBe(checkScopeName({
          label: 'git-policy-cli-test-unit',
          stamp: '20261111T133700-0500',
          runToken: 'a1b2c3d4',
        }),);
      },
    }),
    it({
      name: 'draws a run identity that is a UUID without hyphens, so two runs of one check keep their names apart',
      fn: async () => {
        /**
         One run's token.
         */
        const token = runTokenOfScopes();
        expect(token,).toHaveLength(32,);
        expect(token.includes('-',),).toBe(false,);
        expect(token,).not.toBe(runTokenOfScopes(),);
      },
    }),
    it({
      name: 'gives systemd the caps, the unit name and then the command, in the order it reads them',
      fn: async () => {
        expect(scopeArguments({
          name: testScopeName,
          caps: {
            unitCpuQuota: '250%',
            unitMemoryMax: '2G',
            unitTasksMax: 128,
            unitIoWeight: 4,
          },
          program: 'mise',
          args: ['run', 'a:b',],
        }),).toEqual([
          '--user',
          '--scope',
          '--quiet',
          '--slice=app.slice',
          `--unit=${testScopeName}`,
          '--property=CPUQuota=250%',
          '--property=MemoryMax=2G',
          '--property=TasksMax=128',
          '--property=IOWeight=4',
          '--',
          'mise',
          'run',
          'a:b',
        ],);
      },
    }),
    it({
      name: 'accepts the caps every recorded run used',
      fn: async () => {
        expect(checkedScopeCaps({ caps: defaultCheckScopeCaps, }),).toBe(defaultCheckScopeCaps,);
        expect(scopeCapsProblems({ caps: defaultCheckScopeCaps, }),).toEqual([],);
      },
    }),
    it({
      name: 'refuses a disk weight above ten, an empty quota and a task limit of zero, naming each option',
      fn: async () => {
        /**
         Every reason the three unusable caps are refused.
         */
        const problems = scopeCapsProblems({
          caps: {
            unitCpuQuota: '',
            unitMemoryMax: '4G',
            unitTasksMax: 0,
            unitIoWeight: 11,
          },
        });
        expect(problems,).toHaveLength(3,);
        expect(problems[0],).toContain('--unit-cpu-quota is empty',);
        expect(problems[1],).toContain('--unit-tasks-max 0 is not a whole number from one',);
        expect(problems[2],).toContain('--unit-io-weight 11 is above 10',);
        expect(() => {
          checkedScopeCaps({
            caps: {
              unitCpuQuota: '200%',
              unitMemoryMax: '4G',
              unitTasksMax: 400,
              unitIoWeight: 11,
            },
          });
        },).toThrow(NativeGateError,);
      },
    }),
    it({
      name: 'runs the command in its own scope and answers with its status and both streams',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        /**
         What the runner answered.
         */
        const outcome = await runInCappedScope({
          name: testScopeName,
          caps: defaultCheckScopeCaps,
          program: process.execPath,
          args: [
            '-e',
            String.raw`process.stdout.write("the check ran\n"); process.stderr.write("and said why\n"); process.exit(3);`,
          ],
          deadlineMilliseconds: testDeadlineMilliseconds,
          environment: standIn.environment,
        });
        expect(outcome.status,).toBe(3,);
        expect(outcome.timedOut,).toBe(false,);
        expect(outcome.stdout,).toContain('the check ran',);
        expect(outcome.stderr,).toContain('and said why',);
        /**
         What the fixture recorded.
         */
        const calls = await called({ standIn, });
        expect(calls.starts,).toHaveLength(1,);
        expect(calls.starts[0],).toContain('--scope',);
        expect(calls.starts[0],).toContain(`--unit=${testScopeName}`,);
        expect(calls.starts[0],).toContain('--property=IOWeight=1',);
        expect(calls.starts[0]?.slice(-2),).toEqual([
          '-e',
          String.raw`process.stdout.write("the check ran\n"); process.stderr.write("and said why\n"); process.exit(3);`,
        ],);
        expect(calls.stops,).toHaveLength(0,);
      },
    }),
    it({
      name: 'ends a command that outlasts its deadline by the name of its scope, and says the deadline did it',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        /**
         What the runner answered.
         */
        const outcome = await runInCappedScope({
          name: testScopeName,
          caps: defaultCheckScopeCaps,
          program: process.execPath,
          args: [
            '-e',
            `setTimeout(function end() {}, ${String(outlastMilliseconds)});`,
          ],
          deadlineMilliseconds,
          environment: standIn.environment,
        });
        expect(outcome.timedOut,).toBe(true,);
        /**
         What the fixture recorded.
         */
        const calls = await called({ standIn, });
        expect(calls.stops,).toHaveLength(1,);
        expect(calls.stops[0],).toEqual([
          '--user',
          'kill',
          '--signal=SIGKILL',
          '--kill-whom=all',
          `${testScopeName}.scope`,
        ],);
      },
    }),
    it({
      name: 'refuses a check whose scope the manager would not end, instead of calling the check stopped',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        /**
         The refusal the runner threw.
         */
        let refusal: unknown;
        try {
          await runInCappedScope({
            name: testScopeName,
            caps: defaultCheckScopeCaps,
            program: process.execPath,
            args: [
              '-e',
              `setTimeout(function end() {}, ${String(outlastMilliseconds)});`,
            ],
            deadlineMilliseconds,
            environment: {
              ...standIn.environment,
              [scopeStandInStopFailureVariable]: '1',
            },
          });
        }
        catch (error) {
          refusal = error;
        }
        expect(refusal,).toBeInstanceOf(NativeGateError,);
        expect(String(refusal),).toContain('could not be ended',);
        expect(String(refusal),).toContain('the stand-in was asked to refuse this stop',);
      },
    }),
    it({
      name: 'treats a scope the manager does not know as already over',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        expect(await endCappedScope({
          name: testScopeName,
          environment: standIn.environment,
        }),).toEqual({
          ended: true,
          detail: '',
        },);
      },
    }),
    it({
      name: 'keeps what a scope counted while it ran, so a bound is known after the scope is gone',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        await using scratch = await createGateScratchDirectory({ prefix: 'check-scope-accounting-', });
        /**
         The control group this fixture's scope takes.
         */
        const cgroupPath = checkScopeCgroupPath({
          name: testScopeName,
          root: scratch.path,
        });
        /**
         What the runner answered.
         */
        const outcome = await runInCappedScope({
          name: testScopeName,
          caps: defaultCheckScopeCaps,
          program: process.execPath,
          args: [
            '-e',
            `const fs = require('node:fs');
const path = require('node:path');
const directory = process.env.CHECK_TEST_CGROUP_PATH;
fs.mkdirSync(directory, { recursive: true });
fs.writeFileSync(path.join(directory, 'pids.peak'), '434\\n');
fs.writeFileSync(path.join(directory, 'pids.max'), '400\\n');
fs.writeFileSync(path.join(directory, 'pids.events'), 'max 6\\n');
fs.writeFileSync(path.join(directory, 'memory.peak'), '1953531904\\n');
fs.writeFileSync(path.join(directory, 'memory.events'), 'oom_kill 2\\n');
fs.writeFileSync(path.join(directory, 'cpu.max'), '200000 100000\\n');
fs.writeFileSync(path.join(directory, 'cpu.stat'), 'nr_periods 557\\nnr_throttled 314\\nusage_usec 74283909\\n');
setTimeout(function end() {
  fs.rmSync(directory, { recursive: true, force: true });
  process.exit(0);
}, ${String(fixtureScopeMilliseconds)});`,
          ],
          deadlineMilliseconds: testDeadlineMilliseconds,
          environment: {
            ...standIn.environment,
            CHECK_TEST_CGROUP_PATH: cgroupPath,
          },
          scopeRoot: scratch.path,
          pollMilliseconds: fixturePollMilliseconds,
        });
        expect(outcome.status,).toBe(0,);
        // The scope's work deleted its own control group before it ended, so only a reading taken while it ran is here.
        if (outcome.accounting === SCOPE_NOT_READ)
          throw new Error('the runner read no control group while the fixture scope ran');
        expect(outcome.accounting.pidsPeak,).toBe(434,);
        expect(outcome.accounting.pidsEvents.max,).toBe(6,);
        expect(outcome.accounting.memoryEvents.oom_kill,).toBe(2,);
        expect(outcome.accounting.cpuStat.nr_throttled,).toBe(314,);
        expect(outcome.elapsedMilliseconds,).toBeGreaterThanOrEqual(fixtureScopeMilliseconds,);
      },
    }),
    it({
      name: 'runs the command in the directory it was given',
      fn: async () => {
        await using standIn = await createScopeStandIn();
        /**
         What the runner answered.
         */
        const outcome = await runInCappedScope({
          name: testScopeName,
          caps: defaultCheckScopeCaps,
          program: process.execPath,
          args: [
            '-e',
            'process.stdout.write(process.cwd());',
          ],
          deadlineMilliseconds: testDeadlineMilliseconds,
          workingDirectory: standIn.directory,
          environment: standIn.environment,
        });
        expect(outcome.status,).toBe(0,);
        // A command reports its directory as the filesystem names it, and the scratch parent is reached through one link.
        expect(outcome.stdout,).toBe(await realpath(standIn.directory),);
      },
    }),
  ],
});
