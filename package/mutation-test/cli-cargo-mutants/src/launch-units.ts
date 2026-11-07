import type { UnitPlan } from './plan.ts';
import { formatWaitHint } from './report.ts';
import {
  type StartedUnit,
  startUnit,
} from './start-units.ts';

/**
 Starts one unit and answers how it went; `startUnit` in production, a stand-in in tests.
 */
export type UnitStarter = (call: {
  readonly unit: string;
  readonly systemdRunArguments: readonly string[];
}) => Promise<StartedUnit>;

/**
 A unit `systemd-run` refused, with the file whose campaign it was to run.
 */
export type FailedUnit = {
  readonly unit: string;
  readonly file: string;
};

/**
 How a launch of the file units went.
 */
export type LaunchOutcome = {
  /**
   Lines to print: one per unit, then a summary, then the wait command when some unit started.
   */
  readonly lines: readonly string[];
  /**
   Names of the units that started, in file order.
   */
  readonly started: readonly string[];
  /**
   Units that did not start, in file order.
   */
  readonly failed: readonly FailedUnit[];
  /**
   0 when every unit started, 1 when any did not.
   */
  readonly exitCode: number;
};

/**
 One planned unit with how starting it went.
 */
type UnitOutcome = {
  readonly plan: UnitPlan;
  readonly started: StartedUnit;
};

/**
 Exit status of a launch where some unit did not start.
 */
const failedLaunchExitCode = 1;

/**
 Start every planned unit together and report which started and which did not.

 The units are independent and resumable, so one refusal stops none of the others, and the started ones are left running.
 A launch where any unit did not start still exits nonzero, so a caller that waits on the exit status does not take it for
 a launch of every file.

 @param plans - the units to start, in file order

 @param start - starts one unit; `startUnit` in production

 @param alsoWaitFor - names of units the product started itself, which go first in the wait command

 @returns the lines to print, the started and failed units, and the exit status

 @example
 ```ts
 const { lines, exitCode } = await launchUnits({ plans });
 ```
 */
export async function launchUnits({
  plans,
  start = startUnit,
  alsoWaitFor = [],
}: {
  readonly plans: readonly UnitPlan[];
  readonly start?: UnitStarter;
  readonly alsoWaitFor?: readonly string[];
}): Promise<LaunchOutcome> {
  /**
   Outcome of every unit, in file order; the units start together.
   */
  const outcomes: readonly UnitOutcome[] = await Promise.all(plans.map(async function startPlan(plan) {
    return {
      plan,
      started: await start({
        unit: plan.unit,
        systemdRunArguments: plan.systemdRunArguments,
      }),
    };
  }));
  /**
   Units that started.
   */
  const started = outcomes
    .filter(function didStart({ started: result }: UnitOutcome) {
      return result.started;
    })
    .map(function unitOf({ plan }: UnitOutcome) {
      return plan.unit;
    });
  /**
   Units that did not.
   */
  const failed = outcomes
    .filter(function didNotStart({ started: result }: UnitOutcome) {
      return !result.started;
    })
    .map(function failureOf({ plan }: UnitOutcome): FailedUnit {
      return {
        unit: plan.unit,
        file: plan.file,
      };
    });
  return {
    lines: [
      ...outcomes.map(function lineOf({ started: result }: UnitOutcome) {
        return result.line;
      }),
      ...(failed.length === 0 ? [] : [
        `Started ${started.length} of ${plans.length} units; ${failed.length} did not start:`,
        ...failed.map(function failedLine({
          unit,
          file,
        }: FailedUnit) {
          return `- ${file} (${unit}); the systemd-run error is on that unit's line above`;
        }),
        'The units that started keep running: they are independent, and a stopped one resumes when launched again.',
        'To start the failed files, launch only them under another label: a tag belongs to a file\'s position in the list, so the same label would reuse the tags of the running units.',
      ]),
      ...(started.length === 0 ? [] : [
        formatWaitHint([
          ...alsoWaitFor,
          ...started,
        ])
      ]),
    ],
    started,
    failed,
    exitCode: failed.length === 0 ? 0 : failedLaunchExitCode,
  };
}
