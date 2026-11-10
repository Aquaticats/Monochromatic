#!/usr/bin/env node
/**
 Wait, bounded, for a gate log's verdict and print it with the run's summary and failures.

 This is what `native:gate:verdict` runs. A gate is started in its own unit and answers at once, so this is how a
 coordinator learns how it ended: the log's last line is the exit status the unit wrote, nextest prints its summary and
 the failed tests last, and the printed lines are those. The exit status of this command says whether a verdict arrived at
 all, never whether the gate passed: the printed exit-status line carries the gate's own status, and a run that passed and
 a run that failed both earn a zero here.
 */
import { millisecondsPerSecond } from './native-release-host-control-requirements.ts';
import { NativeGateError } from './native-gate-error.ts';
import {
  gateRefusalExitCode,
  runGateMain
} from './native-gate-main.ts';
import {
  defaultVerdictPollMilliseconds,
  waitForGateVerdict,
} from './native-gate-verdict-reader.ts';
import {
  gateOptions,
  numberOption,
  wholeNumber,
} from './native-gate-arguments.ts';

/**
 Option that names the interval between two looks at the log.
 */
const pollOption = '--poll-milliseconds';

/**
 How the command is called, printed by every refusal that reads the command line.
 */
const usage = 'usage: native:gate:verdict [--poll-milliseconds <count>] <log path> <deadline seconds>';

/**
 Wait for one gate's verdict and print it.

 @returns zero when a verdict arrived, and {@link gateRefusalExitCode} when the deadline passed without one

 @throws NativeGateError when the command line, or a log that is there and cannot be read, is the answer

 @example
 ```ts
 process.exitCode = await runGateMain({ main });
 ```
 */
async function main(): Promise<number> {
  /**
   The command line, whose two positionals are the log and the deadline.
   */
  const options = gateOptions({
    argv: process.argv
      .slice(2),
    knownOptions: [pollOption,],
  });
  /**
   The log to watch and how long this may wait for it.
   */
  const [logPath, deadlineText, ...rest] = options.positionals;
  if ((logPath === undefined) || (deadlineText === undefined))
    throw new NativeGateError(`native:gate:verdict needs a log path and a deadline. ${usage}`);
  if (rest.length > 0)
    throw new NativeGateError(`native:gate:verdict takes one log path and one deadline, and "${rest.join(' ')}" is neither. ${usage}`);
  /**
   What the wait found.
   */
  const outcome = await waitForGateVerdict({
    logPath,
    deadlineMilliseconds: wholeNumber({
      text: deadlineText,
      name: 'the deadline in seconds',
    }) * millisecondsPerSecond,
    pollMilliseconds: numberOption({
      options,
      name: pollOption,
      fallback: defaultVerdictPollMilliseconds,
    }),
  });
  if (outcome.kind === 'none') {
    console.log(`no verdict in ${logPath} within ${deadlineText} s`);
    return gateRefusalExitCode;
  }
  console.log(outcome.lines
    .join('\n'));
  return 0;
}

if (import.meta.main)
  process.exitCode = await runGateMain({ main, });
