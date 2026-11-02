import { systemdRunProgram } from './constants.ts';
import {
  type ProgramResult,
  runProgram,
} from './run-program.ts';
import { formatStartLine } from './report.ts';

/**
 Starts one program and answers how it ended; `runProgram` in production, a stand-in in tests.
 */
export type ProgramRunner = (call: {
  readonly program: string;
  readonly args: readonly string[];
  readonly inheritStdio: boolean;
}) => Promise<ProgramResult>;

/**
 How starting one unit ended.
 */
export type StartedUnit = {
  readonly unit: string;
  readonly started: boolean;
  readonly line: string;
};

/**
 Start one transient unit and describe the outcome the way the launcher prints it.

 @param unit - unit name, for the line

 @param systemdRunArguments - the arguments after the `systemd-run` program name

 @param run - runs the program; defaults to `runProgram`

 @returns the unit, whether `systemd-run` accepted it, and the line `<unit>: started` or `<unit>: <error text>`

 @example
 ```ts
 await startUnit({ unit: 'u', systemdRunArguments: ['--user', '--unit=u', 'sh', '-c', 'true'] });
 ```
 */
export async function startUnit({
  unit,
  systemdRunArguments,
  run = runProgram,
}: {
  readonly unit: string;
  readonly systemdRunArguments: readonly string[];
  readonly run?: ProgramRunner;
}): Promise<StartedUnit> {
  /**
   Outcome of starting this unit.
   */
  const outcome = await run({
    program: systemdRunProgram,
    args: systemdRunArguments,
    inheritStdio: false,
  });
  return {
    unit,
    started: outcome.status === 0,
    line: formatStartLine({
      unit,
      status: outcome.status,
      stderr: outcome.stderr,
    }),
  };
}
