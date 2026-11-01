import { shellQuote } from '@monochromatic-dev/shell-argument-core/ts/shell-quote.ts';

import {
  nodeProgram,
  searchProgram,
  taskRunnerProgram,
} from './constants.ts';
import { CargoMutantsToolingError } from './error.ts';

/**
 Name of the shell variable that holds the log path prefix.
 */
const logVariable = 'log';

/**
 Name of the shell loop variable that holds the attempt number.
 */
const attemptVariable = 'attempt';

/**
 Quote one caller-supplied value as a shell word, refusing NUL in this package's own error type.

 @param value - file name, option or task text that becomes one shell argument

 @returns the single-quoted word

 @throws CargoMutantsToolingError when the value holds a NUL character, which no shell argument can hold
 */
function quoteWord(value: string): string {
  if (value.includes('\0'))
    throw new CargoMutantsToolingError(`A shell body argument cannot contain a NUL character, got ${JSON.stringify(value)}.`);
  return shellQuote(value);
}

/**
 Build the shell body that runs one file's campaign, rerunning it while its log shows the retry marker.

 The body is a real `sh` `for` loop, because `break` only works inside one.
 Each attempt appends to its own log, `<log base>.<attempt>`, and the loop stops after the first attempt whose log lacks
 the marker; after the last attempt it ends whatever the log holds.
 Every caller-supplied value is single-quoted at this final interpolation, so file names with spaces, quotes, `$`, `;`,
 backticks or newlines reach the commands as one literal argument each.

 @param logBase - log path prefix; attempt `n` writes `<logBase>.n`
 
 @param attempts - runs before giving up
 
 @param retryMarker - fixed text whose presence in a log requests a rerun
 
 @param gateSlotPath - path of the gate-slot limiter script
 
 @param gateSlotArguments - options of the limiter, empty to use its defaults
 
 @param task - mise task that runs the campaign
 
 @param taskArguments - arguments after `--` on the task's command line
 
 @returns one-line `sh` program
 
 @throws CargoMutantsToolingError when the attempt count is not a positive whole number or an argument holds NUL

 @example
 ```ts
 buildShellBody({ logBase: '/l/x.log', attempts: 3, retryMarker: 'FAILED', gateSlotPath: '/g.ts', gateSlotArguments: [], task: '//p:t', taskArguments: [] });
 ```
 */
export function buildShellBody({
  logBase,
  attempts,
  retryMarker,
  gateSlotPath,
  gateSlotArguments,
  task,
  taskArguments,
}: {
  readonly logBase: string;
  readonly attempts: number;
  readonly retryMarker: string;
  readonly gateSlotPath: string;
  readonly gateSlotArguments: readonly string[];
  readonly task: string;
  readonly taskArguments: readonly string[];
}): string {
  if ((!Number.isSafeInteger(attempts)) || (attempts < 1))
    throw new CargoMutantsToolingError(`The attempt count must be a positive whole number, got ${attempts}.`);
  /**
   Attempt numbers the loop visits, written out as shell words.
   */
  const attemptNumbers = Array.from(
    { length: attempts },
    function attemptNumber(
      _,
      index
    ) {
      return String(index + 1);
    },
  )
    .join(' ');
  /**
   Quoted words of the campaign command, gate-slot limiter first.
   */
  const command = [
    nodeProgram,
    quoteWord(gateSlotPath),
    ...gateSlotArguments.map(function quoteArgument(argument) { return quoteWord(argument); }),
    taskRunnerProgram,
    'run',
    quoteWord(task),
    '--',
    ...taskArguments.map(function quoteArgument(argument) { return quoteWord(argument); }),
  ].join(' ');
  /**
   Attempt log path, expanded by the shell at run time.
   */
  const attemptLog = `"$${logVariable}.$${attemptVariable}"`;
  return [
    `${logVariable}=${quoteWord(logBase)}`,
    `for ${attemptVariable} in ${attemptNumbers}`,
    `do ${command} >> ${attemptLog} 2>&1`,
    `if ! ${searchProgram} --quiet --fixed-strings -- ${quoteWord(retryMarker)} ${attemptLog}`,
    'then break',
    'fi',
    'done',
  ].join('; ');
}
