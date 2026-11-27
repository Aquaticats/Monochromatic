/**
 The rules every guest driver follows, and what this suite's tests read: the words
 one command line gives a run, how a shell script is quoted for a guest, what a
 pulled log has to prove, the bound a guest command gets, and the two files one
 detached guest script runs from.

 The drivers share these so a rule is written once: a flag is read the same way by
 every driver, a script is quoted the same way before `mvm` hands it to a guest's
 shell, and every arm states its own lines against the same marker check. The
 `-fixture` in this file's name is this package's mark of a module its end-to-end
 suite and that suite's tests share, which is why the unit test beside it reads
 this module rather than one driver's copy.

 @example
 ```ts
 const { step, values, flags } = runArguments({ argv: ['run', '--name', 'guest-one'] });
 ```
 */

import { readFile, } from 'node:fs/promises';

import { millisecondsPerSecond, } from '../../bin/native-release-host-control-requirements.ts';

/**
 The prefix that makes a word a flag rather than a value.
 */
const flagPrefix = '--';

/**
 The guest folder every detached script and its log live in, which `mvm push`
 writes and the host can read for a guest that has no agent answer.
 */
export const sharedFolder = '/mnt/shared';

/**
 Seconds the guest's own `timeout` fires before `mvm`'s kill, so a script that
 runs away is stopped inside the guest, where its log is.
 */
export const guestTimeoutMarginSeconds = 60;

/**
 The prefix a detached guest script writes its own status behind.
 */
const donePrefix = 'DONE=';

/**
 One command line, split the way every driver in this folder reads it.
 */
export type RunArguments = {
  /**
   The step the command names, which is the first word after the script's path
   and is empty text when the command gives none.
   */
  readonly step: string;
  /**
   The word that follows each flag-shaped word, with the first occurrence of a
   flag winning.
   */
  readonly values: ReadonlyMap<string, string>;
  /**
   Every word after the step, so a bare flag is one lookup and no order matters.
   */
  readonly flags: ReadonlySet<string>;
};

/**
 Read one command line into the step, the flag values and the bare flags.

 The lookup answers as the drivers always did: the first occurrence of a flag
 wins, and a flag given as the last word has no value, so the caller's own default
 stands.

 @param argv - the arguments after the script's path

 @returns the step, each flag's value and every word after the step

 @example
 ```ts
 runArguments({ argv: ['all', '--keep-guest'] });
 ```
 */
export function runArguments({
  argv,
}: {
  readonly argv: readonly string[];
},): RunArguments {
  /**
   The step, which is the first word, and every word after it.
   */
  const [step = '', ...afterStep] = argv;
  /**
   The values the command line gave each flag, first occurrence winning.
   */
  const values = new Map<string, string>();
  for (const [at, word] of argv.entries()) {
    /**
     The word after this one, which is this flag's value when it is not a flag.
     */
    const value = argv[at + 1];
    /**
     Whether this word is a flag at all.
     */
    const isFlag = word.startsWith(flagPrefix,);
    /**
     Whether a word follows it to be its value.
     */
    const hasValue = value !== undefined;
    /**
     Whether this is the flag's first appearance, which is the one the drivers
     have always read.
     */
    const isFirst = !values.has(word,);
    if (isFlag && (hasValue && isFirst))
      values.set(
        word,
        value,
      );
  }
  return {
    step,
    values,
    flags: new Set(afterStep,),
  };
}

/**
 Quote one block of shell so a guest's `sh -ec` reads it as one word.

 A guest script is one command line the host hands `mvm`, and `mvm` hands that
 line to the guest's shell, so the script's own quoting has to survive both. A
 single quote ends the quoted run, an escaped quote follows, and the run opens
 again: the shape `sh` itself documents for quoting arbitrary text.

 @param script - the shell script, with the guest's own quoting already in it

 @returns the script as one shell word, single quotes included

 @example
 ```ts
 const quoted = shellQuote(`test "$(hostname)" = guest-one`);
 ```
 */
export function shellQuote(script: string,): string {
  return `'${script.replaceAll(
    '\'',
    `'\\''`,
  )}'`;
}

/**
 Whether one detached guest script said how it ended.
 */
export type DoneStatus = {
  /**
   True for a log whose `DONE=` line names a status.
   */
  readonly found: true;
  /**
   The digits that line holds, as text, so a status of zero is not `0` alone.
   */
  readonly status: string;
} | {
  /**
   True for a log with no `DONE=` line naming a status.
   */
  readonly found: false;
};

/**
 Read one pulled log.

 @param path - where the pulled log sits on the host

 @returns its whole text

 @example
 ```ts
 await readLog({ path: join(logDir, 'adoption-arm.log') });
 ```
 */
export function readLog({
  path,
}: {
  readonly path: string;
},): Promise<string> {
  return readFile(
    path,
    'utf8',
  );
}

/**
 Refuse a pulled log that lacks any of the lines an arm performs.

 @param pulled - the log's whole text

 @param markers - the lines the arm prints, one per claim it makes

 @param subject - what the log is called in the refusal

 @param because - why a missing line is a failure, after the semicolon

 @throws Error naming the first marker the log lacks

 @example
 ```ts
 requireMarkers({ pulled, markers: ['SWEEP done'], subject: 'the pulled sweep-arm log', because: 'the guest ran the arm without performing it' });
 ```
 */
export function requireMarkers({
  pulled,
  markers,
  subject,
  because,
}: {
  readonly pulled: string;
  readonly markers: readonly string[];
  readonly subject: string;
  readonly because: string;
},): void {
  for (const marker of markers)
    if (!pulled.includes(marker))
      throw new Error(`${subject} lacks ${marker}; ${because}`);
}

/**
 Whether one of a log's lines starts with a word, which is how a skipped arm
 announces itself.

 @param pulled - the log's whole text

 @param word - the word a line has to start with

 @returns true when any line starts with that word

 @example
 ```ts
 holdsLineStartingWith({ pulled, word: 'SKIPPED' });
 ```
 */
export function holdsLineStartingWith({
  pulled,
  word,
}: {
  readonly pulled: string;
  readonly word: string;
},): boolean {
  return pulled
    .split('\n')
    .some(function startsWithTheWord(line,) {
      return line.startsWith(word,);
    },);
}

/**
 The status a detached guest script ended with, read from the `DONE=` line it
 appends to its own log.

 The line is the script's last word on how it ended, so the parser takes the
 first line that is exactly `DONE=` and digits: anything else, including no line
 at all, is a script that ended without saying.

 @param pulled - the pulled log's whole text

 @returns whether the log names a status, and the digits it names

 @example
 ```ts
 doneStatusOf({ pulled: 'arm output\nDONE=0\n' });
 ```
 */
export function doneStatusOf({
  pulled,
}: {
  readonly pulled: string;
},): DoneStatus {
  /**
   The digits of every `DONE=` line that names a whole number, in the order the
   log holds them.
   */
  const named = pulled
    .split('\n')
    .filter(function isDoneLine(line,) {
      return line.startsWith(donePrefix,);
    },)
    .map(function digitsOfDoneLine(line,) {
      return line.slice(donePrefix.length,);
    },)
    .filter(function namesAStatus(text,) {
      return isWholeNumber({ text, },);
    },);
  /**
   The first status the log names, absent when it names none.
   */
  const [status,] = named;
  if (status === undefined) return { found: false, };
  return {
    found: true,
    status,
  };
}

/**
 Whether one text is a whole number written in digits.

 The scan counts characters rather than code points on purpose: a status is
 ASCII digits, and anything else, including a character an emoji is built from,
 is a status this does not name.

 @param text - the text after a `DONE=` line's prefix

 @returns true when the text is one or more digits and nothing else

 @example
 ```ts
 isWholeNumber({ text: '17' });
 ```
 */
function isWholeNumber({
  text,
}: {
  readonly text: string;
},): boolean {
  if (text.length === 0) return false;
  for (let at = 0; at < text.length; at += 1) {
    /**
     The character at this position, which is one UTF-16 unit and not one word.
     */
    const character = text.charAt(at,);
    if ((character < '0') || (character > '9')) return false;
  }
  return true;
}

/**
 The two files one detached guest script runs from.
 */
export type DetachedScriptFiles = {
  /**
   The bounded script itself, which runs detached inside the guest.
   */
  readonly step: string;
  /**
   The launcher the host starts, which returns as soon as the script is detached.
   */
  readonly launch: string;
};

/**
 The step and launcher files one detached guest script runs from.

 The launcher is what the host starts, and the step is what it runs detached:
 both are pushed into the guest's shared folder under their own names, which is
 where the launcher looks for the step.

 @param chosen - the guest's name and the folder this run's logs go in

 @param script - the shell script to run inside the guest

 @param log - the log's name inside the shared folder

 @param timeout - milliseconds the script may take inside the guest

 @param hostnameCheckInLaunch - whether the launcher refuses a guest that is not
   this one, rather than the step naming the check

 @returns the two files' whole text

 @example
 ```ts
 const files = detachedScriptFiles({ chosen, script: 'id', log: 'guest-id.log', timeout: 900_000, hostnameCheckInLaunch: true });
 ```
 */
export function detachedScriptFiles({
  chosen,
  script,
  log,
  timeout,
  hostnameCheckInLaunch,
}: {
  readonly chosen: { readonly name: string };
  readonly script: string;
  readonly log: string;
  readonly timeout: number;
  readonly hostnameCheckInLaunch: boolean;
},): DetachedScriptFiles {
  /**
   The check that the command reached the guest it names, in whichever file
   carries it.
   */
  const hostnameCheck = `test "$(hostname)" = ${chosen.name}`;
  /**
   How the detached script puts its own status into its log, which is what the
   host waits for.
   */
  const doneLine = `echo "DONE=$status" >> ${sharedFolder}/${log}`;
  /**
   The bounded script, which is what runs detached.
   */
  const stepLines = [
    ...(hostnameCheckInLaunch ? [] : [hostnameCheck,]),
    `timeout ${String(guestCommandTimeoutSeconds({ timeoutMilliseconds: timeout, },))} sh -ec ${shellQuote(script,)} > ${sharedFolder}/${log} 2>&1`,
    'status=$?',
    doneLine,
    '',
  ];
  /**
   The launcher, which is what the host starts and which returns at once.
   */
  const launchLines = [
    ...(hostnameCheckInLaunch ? [`${hostnameCheck} || exit 1`,] : []),
    `setsid nohup sh ${sharedFolder}/${log}.step.sh < /dev/null > /dev/null 2>&1 &`,
    'exit 0',
    '',
  ];
  return {
    step: stepLines.join('\n',),
    launch: launchLines.join('\n',),
  };
}

/**
 Seconds a guest command is given, after the margin the guest's own `timeout`
 leaves before `mvm`'s kill.

 @param timeoutMilliseconds - milliseconds the host waits for the command

 @returns the whole seconds the guest's `timeout` is given

 @example
 ```ts
 guestCommandTimeoutSeconds({ timeoutMilliseconds: 900_000 });
 ```
 */
export function guestCommandTimeoutSeconds({
  timeoutMilliseconds,
}: {
  readonly timeoutMilliseconds: number;
},): number {
  return Math.floor(timeoutMilliseconds / millisecondsPerSecond,)
    - guestTimeoutMarginSeconds;
}
