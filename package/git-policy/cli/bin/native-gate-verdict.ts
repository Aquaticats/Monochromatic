#!/usr/bin/env node
/**
 Wait, bounded, for a gate log's verdict and print it with the run's summary and failures.

 This is what `native:gate:verdict` runs, and it is how a coordinator learns how a gate ended: the log's last line is the
 exit status the unit wrote, nextest prints its summary and the failed tests last, and the printed lines are those. The exit
 status of this command says whether a verdict arrived at all, never whether the gate passed: the printed exit-status line
 carries the gate's own status, and a run that passed and a run that failed both earn a zero here.

 Wider than the gate it replaces, it names what ends a wait and what a wait that ends without a verdict prints: `--marker`
 for a log that says something else, `--unit` so a run that died is reported at once instead of at the end of the deadline,
 and `--tail-lines` for how much of the log a person sees then.
 */
import {
  gateOptions,
  numberOption,
  textOption,
  wholeNumber,
} from './native-gate-arguments.ts';
import { NativeGateError } from './native-gate-error.ts';
import {
  gateRefusalExitCode,
  runGateMain,
} from './native-gate-main.ts';
import { millisecondsPerSecond } from './native-release-host-control-requirements.ts';
import { gateExitStatusMarker } from './native-gate-unit.ts';
import {
  defaultVerdictPollMilliseconds,
  noVerdictTailLines,
  waitForGateVerdict,
} from './native-gate-verdict-reader.ts';
import { userUnitStillRunning } from './systemd-user-unit-running.ts';

/**
 Option that names the interval between two looks at the log.
 */
const pollOption = '--poll-milliseconds';

/**
 Option that names the text the log has to carry before the wait ends.
 */
const markerOption = '--marker';

/**
 Option that names the systemd unit whose stopping ends the wait.
 */
const unitOption = '--unit';

/**
 Option that names how many of the log's last lines a wait without a verdict prints.
 */
const tailLinesOption = '--tail-lines';

/**
 How the command is called, printed by every refusal that reads the command line.
 */
const usage = 'usage: native:gate:verdict [--poll-milliseconds <count>] [--marker <text>] [--unit <name>] [--tail-lines <count>] <log path> <deadline seconds>';

/**
 Wait for one gate's verdict and print it.

 @returns zero when a verdict arrived, and {@link gateRefusalExitCode} when it did not

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
    knownOptions: [
      pollOption,
      markerOption,
      unitOption,
      tailLinesOption,
    ],
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
   The unit whose stopping ends the wait, when the caller names one.
   */
  const unit = options.values
    .get(unitOption);
  /**
   What the wait found.
   */
  const outcome = await waitForGateVerdict({
    logPath,
    marker: textOption({
      options,
      name: markerOption,
      fallback: gateExitStatusMarker,
    }),
    deadlineMilliseconds: wholeNumber({
      text: deadlineText,
      name: 'the deadline in seconds',
    }) * millisecondsPerSecond,
    pollMilliseconds: numberOption({
      options,
      name: pollOption,
      fallback: defaultVerdictPollMilliseconds,
    }),
    ...(unit === undefined ? {} : {
      stillRunning: async function stillRunning(): Promise<boolean> {
        return await userUnitStillRunning({ unit, });
      },
    }),
  });
  if (outcome.kind === 'verdict') {
    console.log(outcome.lines
      .join('\n'));
    return 0;
  }
  console.log(outcome.reason === 'unit-ended'
    ? `unit ${unit ?? ''} stopped without a verdict in ${logPath}`
    : `no verdict in ${logPath} within ${deadlineText} s`);
  /**
   How many of the log's last lines to print, which is none when the caller asks for none.
   */
  const tailLines = numberOption({
    options,
    name: tailLinesOption,
    fallback: noVerdictTailLines,
  });
  if ((tailLines > 0) && (outcome.tail
    .length
    > 0))
    console.log(outcome.tail
      .slice(-tailLines)
      .join('\n'));
  return gateRefusalExitCode;
}

if (import.meta.main)
  process.exitCode = await runGateMain({ main, });
