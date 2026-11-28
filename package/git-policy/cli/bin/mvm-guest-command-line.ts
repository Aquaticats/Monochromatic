/**
 Every piece of text one guest driver hands a shell, or reads back out of a log, kept where a unit test can hold it.

 A guest driver's own work is a command and the guest's answer to it. The text between the two is string work with no
 command in it: quoting one script so a guest's `sh -ec` reads it as a single word, the command line that quotes it, the
 two files a detached script is started from, and the answers a driver reads back out of a log. The three sets of guest
 helpers each held a copy of these (`bin/guest-route-shell-quote.ts`, `bin/guest-route-guest-command.ts`,
 `bin/mvm-guest-command-line.ts` and `e2e/guest-host/guest-host-rules-fixture.ts`), and they agreed byte for byte; what
 survived is the shape each question was answered best in.

 A detached script is the one place the three helper sets did not agree, and the difference is kept: the step 22 driver's
 scripts carry the guest's name check in the step file, the adoption driver's and the start probe's carry it in the
 launcher, and the scripts of the three drivers that carry none are identical only in having none
 (`{@link detachedScriptFiles}`'s `nameCheck`).
 */

import {
  GUEST_TIMEOUT_MARGIN_SECONDS,
  MILLISECONDS_PER_SECOND,
} from './mvm-guest-bounds.ts';
import {
  type GuestBackend,
  guestBackend,
} from './mvm-guest-backend.ts';

/**
 The first word of the line a detached guest script writes when it ends, followed by its exit status.
 */
const ENDED_MARK = 'DONE=';

/**
 Char code of the digit `0`, the first of the decimal digits a status word may hold.
 */
const DIGIT_ZERO_CHAR_CODE = 48;

/**
 Char code of the digit `9`, the last of the decimal digits a status word may hold.
 */
const DIGIT_NINE_CHAR_CODE = 57;

/**
 The whole seconds a millisecond bound gives a shell's `timeout`.

 The division rounds down, as the drivers' bounds always did: a `timeout` given a fraction of a second refuses the run,
 and a bound of one and a half seconds leaves a command the one second the guest's `timeout` can carry.

 @param milliseconds - the bound in milliseconds

 @returns that bound in whole seconds

 @example
 ```ts
 secondsOf({ milliseconds: 2_400_000 }); // 2400
 ```
 */
export function secondsOf({ milliseconds, }: { readonly milliseconds: number; },): number {
  return Math.floor(milliseconds / MILLISECONDS_PER_SECOND,);
}

/**
 Quote one block of shell so a guest's `sh -ec` reads it as a single word.

 A single quote ends the quoted word, so each one in the script is closed, escaped and reopened: `'` becomes `'\''`.
 A driver's guest script holds quotes of its own (a path a pool's listing prints, a marker line it greps for), so the
 word the guest's shell reads is the script byte for byte, which a bare pair of quotes around it would not give.

 @param script - the shell script to quote whole

 @returns the script as one shell word, single quotes included

 @example
 ```ts
 shellQuote("echo 'hi'"); // 'echo '\''hi'\'''
 ```
 */
export function shellQuote(script: string,): string {
  /**
   The script with each of its own single quotes closed, escaped and reopened.
   */
  const escaped = script.replaceAll(
    "'",
    `'\\''`,
  );
  return `'${escaped}'`;
}

/**
 The command line a guest runs: the guest's own name checked first, then the script under the guest's `timeout`.

 The name check is what keeps a script from opening the host's `/dev/zfs`: a run outside the guest it was made for reads
 a different hostname and stops before the script's first word. The guest's `timeout` fires
 {@link GUEST_TIMEOUT_MARGIN_SECONDS} before the bound the host gives `mvm`, so an overrun is a line in the guest's log
 instead of a lost run.

 @param name - the guest's name, which its own hostname must answer

 @param timeoutMilliseconds - the bound the host gives the command, which the guest's `timeout` is sized from

 @param script - the shell script the guest runs

 @param backend - the backend the guest lives on, which decides its hostname and how root is asked for

 @returns the command line, for `mvm exec` to hand to the guest's one shell

 @example
 ```ts
 guestCommandLine({ name: 'git-policy-lock', timeoutMilliseconds: 1_800_000, script: 'zpool list' });
 ```
 */
export function guestCommandLine({
  name,
  timeoutMilliseconds,
  script,
  backend = guestBackend(),
}: {
  readonly name: string;
  readonly timeoutMilliseconds: number;
  readonly script: string;
  readonly backend?: GuestBackend;
},): string {
  /**
   The guest's own words: root when the backend needs asking for it, then the script under the guest's timeout.
   */
  const guestWords: readonly string[] = [
    ...backend.rootWords,
    'timeout',
    String(guestSecondsOf({ timeoutMilliseconds, },),),
    'sh',
    '-ec',
    shellQuote(script,),
  ];
  return `test "$(hostname)" = ${backend.hostnameOf(name,)} && ${guestWords.join(' ',)}`;
}

/**
 The whole seconds the guest's own `timeout` is given, which fires the margin before the host's bound.

 @param timeoutMilliseconds - the bound the host gives the command

 @returns the whole seconds the guest's `timeout` takes

 @example
 ```ts
 guestSecondsOf({ timeoutMilliseconds: 900_000 }); // 840
 ```
 */
export function guestSecondsOf({ timeoutMilliseconds, }: { readonly timeoutMilliseconds: number; },): number {
  return secondsOf({ milliseconds: timeoutMilliseconds, },) - GUEST_TIMEOUT_MARGIN_SECONDS;
}

/**
 Which of a detached script's two files carries the check that the command reached the guest it names.

 - `launcher` puts `test "$(hostname)" = NAME || exit 1` first in the launcher, which refuses a command that reached
   another machine before the step file is started at all (the adoption driver's and the start probe's scripts);
 - `step` puts the same check first in the step file, so the detached script refuses itself, and writes the status line
   the host waits for before it exits, so the host reports the refusal at its next look rather than waiting out the
   script's bound (the step 22 driver's);
 - `none` carries no check, as the removal-lineage and settling drivers' scripts always did, because the host names the
   guest in `mvm exec` and `mvm` itself picks the domain.
 */
export type GuestNameCheck = 'launcher' | 'step' | 'none';

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

 The status line is why a detached script can be judged at all: the host reads it out of the guest's shared folder after
 the script has gone, so a script that outlived the command that started it still reports what it did.

 The launcher is what the host starts, and the step is what it runs detached: both are pushed into the guest's shared
 folder under their own names, which is where the launcher looks for the step. A step file started in the foreground
 would end with the `mvm exec` that started it, and the guest's own job control would take its output with it, so
 `setsid nohup` leaves it holding neither the command's terminal nor its pipes.

 The guest-name check is carried by one of the two files, or by neither, and which one it is, is the distinction the
 drivers already made: the three sets of helpers did not agree, and each driver's scripts are byte-identical only with
 its own answer kept (`{@link GuestNameCheck}`).

 @param name - the guest's name, which its own hostname must answer

 @param script - the shell script to run inside the guest

 @param log - the log's name inside the shared folder, which also names the step file

 @param timeoutMilliseconds - the bound the host gives the script, which the guest's `timeout` is sized from

 @param nameCheck - which of the two files carries the guest's name check, if either

 @returns the two files' whole text, each ending in a newline

 @example
 ```ts
 const files = detachedScriptFiles({ name: 'guest-a', script: 'id', log: 'build.log', timeoutMilliseconds: 900_000, nameCheck: 'step' });
 ```
 */
export function detachedScriptFiles({
  name,
  script,
  log,
  timeoutMilliseconds,
  nameCheck,
  backend = guestBackend(),
}: {
  readonly name: string;
  readonly script: string;
  readonly log: string;
  readonly timeoutMilliseconds: number;
  readonly nameCheck: GuestNameCheck;
  readonly backend?: GuestBackend;
},): DetachedScriptFiles {
  /**
   The check that the command reached the guest it names, in whichever file carries it.
   */
  const hostnameCheck = `test "$(hostname)" = ${backend.hostnameOf(name,)}`;
  /**
   The log inside the guest's shared folder, which the step file writes and the host pulls back.
   */
  const sharedLog = `${backend.guestSharedFolder}/${log}`;
  /**
   The step file inside that folder, which the launcher starts.
   */
  const stepPath = `${backend.guestSharedFolder}/${log}.step.sh`;
  /**
   The bounded script, which is what runs detached.

   A step-shaped check refuses with the status line the host already waits for, because a step file that exited without one
   would leave the host asking until the script's own bound passed.
   */
  /**
   The step's own words: root when the backend needs asking for it, then the script under the guest's timeout.
   */
  const stepWords: readonly string[] = [
    ...backend.rootWords,
    'timeout',
    String(guestSecondsOf({ timeoutMilliseconds, },),),
    'sh',
    '-ec',
    shellQuote(script,),
  ];
  /**
   The step file's lines: the check when it carries one, the bounded script, and the status line the host reads.
   */
  const stepLines = [
    ...(nameCheck === 'step' ? [`${hostnameCheck} || { echo "${ENDED_MARK}1" >> ${sharedLog}; exit 1; }`,] : []),
    `${stepWords.join(' ',)} > ${sharedLog} 2>&1`,
    'status=$?',
    `echo "${ENDED_MARK}$status" >> ${sharedLog}`,
    '',
  ];
  /**
   The launcher, which is what the host starts and which returns at once.
   */
  const launchWords: readonly string[] = [
    ...backend.rootWords,
    'sh',
    stepPath,
  ];
  /**
   The launcher's lines: the check when it carries one, the detached start, and its own end.
   */
  const launchLines = [
    ...(nameCheck === 'launcher' ? [`${hostnameCheck} || exit 1`,] : []),
    `setsid nohup ${launchWords.join(' ',)} < /dev/null > /dev/null 2>&1 &`,
    'exit 0',
    '',
  ];
  return {
    step: stepLines.join('\n',),
    launch: launchLines.join('\n',),
  };
}

/**
 Whether a log's last line is the one a detached guest script writes when it ends.

 The host reads the last line alone while it waits, so the line's own status is not read until the script has gone.

 @param text - the last line of the shared log, as the guest printed it

 @returns true when the script has ended, whatever status it ended with

 @example
 ```ts
 endedLineHolds({ text: 'DONE=0\n' }); // true
 ```
 */
export function endedLineHolds({ text, }: { readonly text: string; },): boolean {
  return text.trim()
    .startsWith(ENDED_MARK,);
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
 The status a detached guest script ended with, read from the `DONE=` line it appends to its own log.

 The line is found the way the drivers always found it: the first line of the log that is the mark followed by decimal
 digits and nothing else. Anything else, including no line at all, is a script that ended without saying, which a caller
 reports as a script that ended without a status.

 @param text - the whole log the guest wrote

 @returns whether the log names a status, and the digits it names

 @example
 ```ts
 doneStatusOf({ text: 'arm output\nDONE=0\n' });
 ```
 */
export function doneStatusOf({ text, }: { readonly text: string; },): DoneStatus {
  /**
   The digits of every `DONE=` line that names a whole number, in the order the log holds them.
   */
  const named = text
    .split('\n',)
    .filter(function isDoneLine(line: string,): boolean {
      return line.startsWith(ENDED_MARK,);
    },)
    .map(function digitsOfDoneLine(line: string,): string {
      return line.slice(ENDED_MARK.length,);
    },)
    .filter(function namesAStatus(word: string,): boolean {
      return holdsOnlyDigits({ word, },);
    },);
  /**
   The first status the log names, absent when it names none.
   */
  const [status,] = named;
  if (status === undefined)
    return { found: false, };
  return {
    found: true,
    status,
  };
}

/**
 Whether one word is decimal digits and nothing else.

 The scan is one pass over the word with no regular expression and no copy of it: a pulled log is the guest's own output
 and has no bound, and this reads only the short word between a mark and the line's end.

 @param word - the word to read

 @returns true when every character is a decimal digit and the word is not empty

 @example
 ```ts
 holdsOnlyDigits({ word: '10' }); // true
 ```
 */
function holdsOnlyDigits({ word, }: { readonly word: string; },): boolean {
  if (word.length === 0)
    return false;
  for (const character of word) {
    /**
     This character's char code, which is a digit's only inside the two ends named above.
     */
    const code = character.codePointAt(0,) ?? 0;
    if ((code < DIGIT_ZERO_CHAR_CODE) || (code > DIGIT_NINE_CHAR_CODE))
      return false;
  }
  return true;
}
