// The advisory profile's admission rules, run as arithmetic over the 400 push arrival times of
// `advisory-core-newest-400-push-arrivals-2026-11-28.json`, at the three `core` job lengths the executed jobs
// measured: 401 s (run 40864046534), 476 s (run 45531170934) and 557 s (run 45529509832).
//
// It writes the table `admission-arithmetic-output-at-401-476-and-557-seconds.json` in this folder, and its figures
// are the ones the proposal beside it prints.
//
//   node admission-arithmetic-over-those-arrivals.ts advisory-core-newest-400-push-arrivals-2026-11-28.json
//
// Model, and what it leaves out:
// - The workflow's concurrency group is one per ref (`advisory-core-${{ github.ref }}`), so one run may execute and
//   one may wait; a push arriving while one waits replaces the waiting one, and the replaced push never starts.
// - A cap is checked when a push arrives, against the number of runs that started executing inside that window for
//   the rules that count executions, and against the arrivals for the rules that count them. That is the one
//   difference between the two counting rules.
// - The arriving push is counted in its own window, which is one more than the workflow counts: the workflow's step
//   drops the current run by `databaseId`, so it admits one more run in this sample (4 rather than 3).
// - The gate job's own seconds, and the delay between a start and a job actually running, are not modelled.
// - Nothing here says how GitHub schedules. It is arithmetic on arrival times and job lengths, and the proposal
//   states it as that.
import { readFileSync, } from 'node:fs';

/**
 One run of the workflow, as the read kept it.
 */
type Run = {
  readonly createdAt: string;
  readonly event: string;
};

/**
 One admission rule, as the workflow's counting step could be written.
 */
type Rule = {
  readonly name: string;
  readonly windowsSeconds: readonly number[];
  readonly caps: readonly number[];
  readonly cancelInProgress: boolean;
  readonly countsExecutions: boolean;
};

/**
 What one rule does to the 400 arrivals at one job length.
 */
type Outcome = {
  readonly rule: string;
  readonly job_seconds: number;
  readonly pushes: number;
  readonly executions_started: number;
  readonly executions_per_day: number;
  readonly finished: number;
  readonly replaced_while_waiting: number;
  readonly cancelled_by_a_newer_push: number;
  readonly refused_by_the_cap: number;
  readonly waiting_when_the_sample_ended: number;
  readonly hosted_job_hours_per_day: number;
};

const millisecondsPerSecond = 1000;
const secondsPerMinute = 60;
const secondsPerHour = 3600;
const secondsPerDay = 86400;
const millisecondsPerHour = secondsPerHour * millisecondsPerSecond;
const hoursPerDay = 24;
const tenth = 10;

const sameNumbers = [secondsPerMinute, secondsPerHour, secondsPerDay,];
const sameCaps = [2, 4, 8,];
const hourly = [secondsPerHour,];
const hourlyCap = [8,];

const rules: readonly Rule[] = [
  { name: 'window as it is today (arrivals count)', windowsSeconds: sameNumbers, caps: sameCaps, cancelInProgress: false, countsExecutions: false, },
  { name: 'today, but a newer push cancels the running run', windowsSeconds: sameNumbers, caps: sameCaps, cancelInProgress: true, countsExecutions: false, },
  { name: 'count executions, same numbers', windowsSeconds: sameNumbers, caps: sameCaps, cancelInProgress: false, countsExecutions: true, },
  { name: 'count executions, same numbers, cancel in progress', windowsSeconds: sameNumbers, caps: sameCaps, cancelInProgress: true, countsExecutions: true, },
  { name: 'count executions, 8 per hour and no day cap', windowsSeconds: hourly, caps: hourlyCap, cancelInProgress: false, countsExecutions: true, },
  { name: 'count executions, 8 per hour, cancel in progress', windowsSeconds: hourly, caps: hourlyCap, cancelInProgress: true, countsExecutions: true, },
  { name: 'no window, group kept', windowsSeconds: [], caps: [], cancelInProgress: false, countsExecutions: false, },
  { name: 'no window, a newer push cancels the running run', windowsSeconds: [], caps: [], cancelInProgress: true, countsExecutions: false, },
];

const jobLengthsSeconds = [401, 476, 557,];

/**
 Read one required value, refusing rather than carrying an undefined onwards.
 @param value - the value a lookup gave, possibly undefined
 @param what - the lookup, for the refusal's text
 @returns the value
 */
function required<T>(value: T | undefined, what: string,): T {
  if (value === undefined) throw new Error(`the run list has no ${what}`);
  return value;
}

/**
 Read the runs file and return the push arrival times in order.
 @param runsPath - the file the read kept
 @returns every push's `createdAt`, as milliseconds since the epoch, sorted
 */
function pushArrivals(runsPath: string,): number[] {
  const runs = JSON.parse(readFileSync(runsPath, 'utf8',),) as readonly Run[];
  return runs
    .filter((run,) => run.event === 'push')
    .map((run,) => Date.parse(run.createdAt,))
    .sort((a, b,) => a - b);
}

/**
 Simulate one rule at one job length.
 @param rule - the admission rule and the concurrency setting
 @param jobSeconds - the `core` job's length
 @param pushes - the arrival times, in order
 @returns what the rule does to them
 */
function simulate(rule: Rule, jobSeconds: number, pushes: readonly number[],): Outcome {
  const starts: number[] = [];
  const arrivals: number[] = [];
  const spanMilliseconds = required(pushes.at(-1,), 'arrival',) - required(pushes[0], 'arrival',);
  const spanHours = spanMilliseconds / millisecondsPerHour;
  const perDay = (count: number,): number => (count / spanHours) * hoursPerDay;
  const roundToTenth = (value: number,): number => Math.round(value * tenth) / tenth;
  const jobMilliseconds = jobSeconds * millisecondsPerSecond;
  let runningUntil: number | undefined;
  let pending: number | undefined;
  let finished = 0;
  let replacedWhileWaiting = 0;
  let cancelledByANewerPush = 0;
  let refusedByTheCap = 0;

  // The cap reads execution starts for a rule that counts executions, and arrivals otherwise.
  const admit = (at: number,): boolean => rule.windowsSeconds.every((window, index,) => {
    const seen = rule.countsExecutions ? starts : arrivals;
    const inside = seen.filter((start,) => at - start < window * millisecondsPerSecond);
    return inside.length < required(rule.caps[index], 'cap',);
  },);

  for (const at of pushes) {
    // Every run still executing when this push arrives has finished by now; the waiting push starts.
    while (runningUntil !== undefined && runningUntil <= at) {
      const freeAt = runningUntil;
      finished += 1;
      runningUntil = undefined;
      if (pending !== undefined) {
        starts.push(freeAt,);
        runningUntil = freeAt + jobMilliseconds;
        pending = undefined;
      }
    }
    arrivals.push(at,);
    if (!admit(at,)) {
      refusedByTheCap += 1;
      continue;
    }
    if (rule.cancelInProgress) {
      if (runningUntil !== undefined) cancelledByANewerPush += 1;
      starts.push(at,);
      runningUntil = at + jobMilliseconds;
      continue;
    }
    if (runningUntil === undefined) {
      starts.push(at,);
      runningUntil = at + jobMilliseconds;
      continue;
    }
    if (pending !== undefined) replacedWhileWaiting += 1;
    pending = at;
  }
  const waitingWhenTheSampleEnded = pending === undefined ? 0 : 1;
  const ordered = starts.sort((a, b,) => a - b);
  const lastArrival = required(pushes.at(-1,), 'arrival',);
  if (ordered.length > 0 && runningUntil !== undefined && runningUntil <= lastArrival + jobMilliseconds) finished += 1;
  return {
    rule: rule.name,
    job_seconds: jobSeconds,
    pushes: pushes.length,
    executions_started: ordered.length,
    executions_per_day: roundToTenth(perDay(ordered.length,),),
    finished,
    replaced_while_waiting: replacedWhileWaiting,
    cancelled_by_a_newer_push: cancelledByANewerPush,
    refused_by_the_cap: refusedByTheCap,
    waiting_when_the_sample_ended: waitingWhenTheSampleEnded,
    hosted_job_hours_per_day: Math.round(perDay(ordered.length,) * jobSeconds) / secondsPerHour,
  };
}

/**
 Run every rule at every job length and print the table.
 @param argv - the script's arguments; `argv[2]` is the runs file
 */
function main(argv: readonly string[],): void {
  const runsPath = argv[2];
  if (runsPath === undefined) {
    throw new Error('usage: node admission-arithmetic-over-those-arrivals.ts <runs.json>');
  }
  const pushes = pushArrivals(runsPath,);
  const spanHours = (required(pushes.at(-1,), 'arrival',) - required(pushes[0], 'arrival',)) / millisecondsPerHour;
  const pushesPerDay = (pushes.length / spanHours) * hoursPerDay;
  const report = {
    span_hours: Math.round(spanHours * 100,) / 100,
    pushes: pushes.length,
    pushes_per_day: Math.round(pushesPerDay * tenth,) / tenth,
    table: jobLengthsSeconds.flatMap((jobSeconds,) => rules.map((rule,) => simulate(rule, jobSeconds, pushes,))),
  };
  process.stdout.write(`${JSON.stringify(report, undefined, 1,)}\n`,);
}

main(process.argv,);
