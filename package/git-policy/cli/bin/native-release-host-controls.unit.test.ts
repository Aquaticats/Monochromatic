import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  hostControlLimitSeconds,
  hostControlTask,
  measuredFullRunDate,
  measuredFullRunSeconds,
  requiredHostControls,
} from './native-release-host-control-requirements.ts';
import type { HostControlRun } from './native-release-host-control-run.ts';
import {
  judgeHostControlRun,
  runHostControlGate,
} from './native-release-host-controls.ts';

/**
 One nextest status line of a test that passed, in the reporter shape the gate asks nextest for.

 @param suite - binary ID nextest prints, such as `monochromatic-git-policy-cli`
 @param test - the test's name within that binary
 @returns the line

 @example
 ```ts
 passingLine({ suite: 'monochromatic-git-policy-cli', test: 'a::b' });
 ```
 */
function passingLine({
  suite,
  test,
}: {
  readonly suite: string;
  readonly test: string;
}): string {
  return `        PASS [   0.176s] (1/1) ${suite} ${test}`;
}

/**
 One nextest status line of a test that failed.

 @param suite - binary ID nextest prints
 @param test - the test's name within that binary
 @returns the line

 @example
 ```ts
 failingLine({ suite: 'monochromatic-git-policy-cli', test: 'a::b' });
 ```
 */
function failingLine({
  suite,
  test,
}: {
  readonly suite: string;
  readonly test: string;
}): string {
  return `        FAIL [   0.176s] (1/1) ${suite} ${test}`;
}

/**
 The line a control prints when it finds nothing to check here, as the library and binary-level controls write it.

 @param control - name the control prints
 @param reason - why it did not run, in the control's own words
 @returns the line

 @example
 ```ts
 notRunLine({ control: 'off-btrfs-ioctls', reason: 'needs a filesystem that is not btrfs' });
 ```
 */
function notRunLine({
  control,
  reason,
}: {
  readonly control: string;
  readonly reason: string;
}): string {
  return `BTRFS CONTROL NOT RUN: ${control} (fixture root /btrfs/fixtures is on Btrfs; ${reason})`;
}

/**
 The line of a control that has nothing to check on a btrfs host, which every real control run prints here.
 */
const offBtrfsLine = notRunLine({
  control: 'off-btrfs-ioctls',
  reason: 'this control needs a filesystem that is not btrfs',
});

/**
 Output of a run that passed every required control, in nextest's reporter shape.

 @param extra - lines placed after the required controls
 @returns the captured output

 @example
 ```ts
 passingOutput({ extra: [] });
 ```
 */
function passingOutput({ extra = [], }: { readonly extra?: readonly string[]; } = {}): string {
  return [
    '────────────',
    ' Nextest run ID 59641e20 with nextest profile: default',
    `    Starting ${String(requiredHostControls.length)} tests across 2 binaries (6970 tests skipped)`,
    ...requiredHostControls.map(function line(control) {
      return passingLine(control);
    }),
    offBtrfsLine,
    ...extra,
    '────────────',
    `     Summary [ 2680.000s] ${String(requiredHostControls.length)} tests run: ${String(requiredHostControls.length)} passed, 6970 skipped`,
  ].join('\n');
}

/**
 A finished run judged by the gate.

 @param output - the run's captured output
 @param status - the status the run exited with
 @param timedOut - whether the deadline stopped it
 @returns the run

 @example
 ```ts
 run({ output: passingOutput(), status: 0, timedOut: false });
 ```
 */
function run({
  output,
  status = 0,
  timedOut = false,
}: {
  readonly output: string;
  readonly status?: number;
  readonly timedOut?: boolean;
}): HostControlRun {
  return {
    ending: {
      kind: 'exited',
      status,
    },
    output,
    timedOut,
  };
}

/**
 A run that the deadline stopped, which has no exit status.

 @param output - the run's captured output
 @returns the run

 @example
 ```ts
 stopped({ output: passingOutput() });
 ```
 */
function stopped({ output, }: { readonly output: string; }): HostControlRun {
  return {
    ending: {
      kind: 'stopped',
      signal: 'SIGKILL',
    },
    output,
    timedOut: true,
  };
}

/**
 Every binary a required control belongs to.
 */
const requiredSuites: readonly string[] = requiredHostControls.map(function suiteOf(control) {
  return control.suite;
});

await describe({
  name: 'the btrfs controls a wrapper release waits for',
  children: [
    describe({
      name: judgeHostControlRun.name,
      children: [
        it({
          name: 'passes a run that holds every required control and the one line a btrfs host prints for a control that needs another filesystem',
          fn: async function passes() {
            const verdict = judgeHostControlRun(run({ output: passingOutput() }));
            expect(verdict.kind).toBe('green');
          },
        }),
        it({
          name: 'counts a test once though nextest prints its passing line twice, which is what the run does at its end',
          fn: async function countsOnce() {
            const verdict = judgeHostControlRun(run({
              output: [
                passingOutput(),
                passingOutput(),
              ].join('\n'),
            }));
            if (verdict.kind !== 'green')
              throw new Error('the same required controls twice are still every required control');
            expect(verdict.testsRun).toBe(requiredHostControls.length);
          },
        }),
        it({
          name: 'requires controls from more than one suite, so one binary passing is no evidence for the other',
          fn: async function bothSuites() {
            expect(new Set(requiredSuites).size).toBeGreaterThan(1);
          },
        }),
        it({
          name: 'passes a run whose only failures are tests other than the required controls, naming each one as a notice',
          fn: async function otherFailures() {
            const flaky = {
              suite: 'monochromatic-git-policy-cli::native_binary',
              test: 'service::a_test_that_failed_under_load',
            };
            const verdict = judgeHostControlRun(run({
              status: 100,
              output: [
                passingOutput(),
                failingLine(flaky),
              ].join('\n'),
            }));
            if (verdict.kind !== 'green')
              throw new Error(`a failure outside the required controls holds no release, yet the gate refused: ${verdict.lines.join(' | ')}`);
            const text = verdict.notices.join('\n');
            expect(text).toContain(`${flaky.suite} ${flaky.test}`);
            expect(text).toContain('Alphas don\'t even have to resolve all flakes.');
          },
        }),
        it({
          name: 'refuses a run whose required control failed, naming the suite and the test',
          fn: async function failing() {
            const [failed] = requiredHostControls;
            if (failed === undefined)
              throw new Error('the gate requires no control, so this test checks nothing');
            const verdict = judgeHostControlRun(run({
              status: 1,
              output: [
                passingOutput(),
                failingLine(failed),
              ].join('\n'),
            }));
            expect(verdict.kind).toBe('refused');
            if (verdict.kind !== 'refused')
              throw new Error('unreachable');
            const text = verdict.lines.join('\n');
            expect(text).toContain('REFUSED the wrapper release');
            expect(text).toContain(`${failed.suite} ${failed.test}`);
            expect(text).toContain('nothing was dispatched');
            expect(text).toContain('--dry-run');
            expect(text).toContain('no btrfs filesystem');
            expect(text).toContain('CLI_GIT_TEST_ROOT');
          },
        }),
        it({
          name: 'refuses a run that printed a control which did not run, quoting the line it printed',
          fn: async function notRun() {
            const skipped = notRunLine({
              control: 'worktree-snapshot',
              reason: 'fixture root /btrfs/fixtures is on Ext4',
            });
            const verdict = judgeHostControlRun(run({
              output: [
                passingOutput({ extra: [skipped] }),
              ].join('\n'),
            }));
            expect(verdict.kind).toBe('refused');
            if (verdict.kind !== 'refused')
              throw new Error('unreachable');
            const text = verdict.lines.join('\n');
            expect(text).toContain('BTRFS CONTROL NOT RUN');
            expect(text).toContain(skipped);
          },
        }),
        it({
          name: 'refuses a run that ended nonzero and printed no failing test, showing the run\'s last lines',
          fn: async function noFailureLine() {
            const verdict = judgeHostControlRun(run({
              status: 101,
              output: 'error: could not compile `monochromatic-git-policy-cli`',
            }));
            expect(verdict.kind).toBe('refused');
            if (verdict.kind !== 'refused')
              throw new Error('unreachable');
            const text = verdict.lines.join('\n');
            expect(text).toContain('status 101');
            expect(text).toContain('could not compile');
          },
        }),
        it({
          name: 'refuses a run that lacks one required control, naming the suite and the test that is missing',
          fn: async function lacksOne() {
            const [absent] = requiredHostControls;
            if (absent === undefined)
              throw new Error('the gate requires no control, so this test checks nothing');
            /**
             The status lines of every required control but the absent one.
             */
            const printed = requiredHostControls
              .filter(function isNotTheAbsentOne(control) {
                return control !== absent;
              })
              .map(function line(control) {
                return passingLine(control);
              });
            const verdict = judgeHostControlRun(run({ output: printed.join('\n') }));
            expect(verdict.kind).toBe('refused');
            if (verdict.kind !== 'refused')
              throw new Error('unreachable');
            const text = verdict.lines.join('\n');
            expect(text).toContain(`${absent.suite} ${absent.test}`);
            expect(text).toContain('not evidence');
          },
        }),
        it({
          name: 'refuses a run that the deadline stopped, quoting the limit and the full run it was sized from',
          fn: async function deadline() {
            const verdict = judgeHostControlRun(stopped({ output: passingOutput() }));
            if (verdict.kind !== 'refused')
              throw new Error('a stopped run is not evidence');
            const text = verdict.lines.join('\n');
            expect(text).toContain(`${String(hostControlLimitSeconds)} seconds`);
            expect(text).toContain(`${String(measuredFullRunSeconds)} seconds`);
            expect(text).toContain(measuredFullRunDate);
          },
        }),
      ],
    }),
    describe({
      name: runHostControlGate.name,
      children: [
        it({
          name: 'says what it waits for and what that run costs before it starts, then exits zero on a green run',
          fn: async function green() {
            const printed: string[] = [];
            const status = await runHostControlGate({
              filters: [],
              runHostControls: async function runHostControls() {
                return run({ output: passingOutput() });
              },
              print: function print(line) {
                printed.push(line);
              },
            });
            expect(status).toBe(0);
            expect(printed[0]).toContain('Waiting for the btrfs controls');
            expect(printed[0]).toContain(hostControlTask);
            expect(printed[0]).toContain(`${String(hostControlLimitSeconds)} seconds`);
            expect(printed.join('\n')).toContain('BTRFS CONTROLS PASSED');
          },
        }),
        it({
          name: 'exits zero on a run whose only failures are outside the required controls, printing each as a notice before the pass',
          fn: async function noticed() {
            const printed: string[] = [];
            const flaky = {
              suite: 'monochromatic-git-policy-cli',
              test: 'push::a_test_that_failed_under_load',
            };
            const status = await runHostControlGate({
              filters: [],
              runHostControls: async function runHostControls() {
                return run({
                  status: 100,
                  output: [
                    passingOutput(),
                    failingLine(flaky),
                  ].join('\n'),
                });
              },
              print: function print(line) {
                printed.push(line);
              },
            });
            expect(status).toBe(0);
            const text = printed.join('\n');
            expect(text).toContain(`${flaky.suite} ${flaky.test}`);
            expect(text.indexOf(`${flaky.suite} ${flaky.test}`)).toBeLessThan(text.indexOf('BTRFS CONTROLS PASSED'));
          },
        }),
        it({
          name: 'exits nonzero and prints the refusal when the run does not pass',
          fn: async function refused() {
            const printed: string[] = [];
            const status = await runHostControlGate({
              filters: [],
              runHostControls: async function runHostControls() {
                return run({
                  status: 1,
                  output: failingLine({
                    suite: 'monochromatic-git-policy-cli',
                    test: 'subvolume::a_control',
                  }),
                });
              },
              print: function print(line) {
                printed.push(line);
              },
            });
            expect(status).toBe(1);
            expect(printed.join('\n')).toContain('REFUSED the wrapper release');
          },
        }),
      ],
    }),
  ],
});
