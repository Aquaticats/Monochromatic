/**
 Every piece of text one guest driver hands a shell, or reads back out of a log, kept where a unit test can hold it.

 A guest driver's own work is a command and the guest's answer to it. The text between the two is string work with no
 command in it: quoting one script so a guest's `sh -ec` reads it as a single word, the command line that quotes it, the
 files a detached script is started from, and the two answers a driver reads back out of a pulled log. Those live here,
 and each driver's own file keeps only the steps it performs.
 */

/**
 Milliseconds in one second, so a bound stated in milliseconds becomes the seconds a shell's `timeout` takes.
 */
export const MILLISECONDS_PER_SECOND = 1_000;

/**
 The first word of the line a detached guest script writes when it ends, followed by its exit status.
 */
const ENDED_MARK = 'DONE=';

/**
 Where a word that is not in a list of words stands: minus one, the answer `indexOf` gives.
 */
const ABSENT_INDEX = -1;

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

 @param script - the shell script to quote whole

 @returns the script as one shell word, single quotes included

 @example
 ```ts
 shellQuote({ script: "echo 'hi'" }); // 'echo '\''hi'\'''
 ```
 */
export function shellQuote({ script, }: { readonly script: string; },): string {
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
 a different hostname and stops before the script's first word.

 @param name - the guest's name, which its own hostname must answer

 @param guestTimeoutSeconds - whole seconds the guest's `timeout` gives the script

 @param script - the shell script the guest runs

 @returns the command line, for `mvm exec` to hand to the guest's one shell

 @example
 ```ts
 guestCommandLine({ name: 'git-policy-lock', guestTimeoutSeconds: 1740, script: 'zpool list' });
 ```
 */
export function guestCommandLine({
  name,
  guestTimeoutSeconds,
  script,
}: {
  readonly name: string;
  readonly guestTimeoutSeconds: number;
  readonly script: string;
},): string {
  return `test "$(hostname)" = ${name} && timeout ${String(guestTimeoutSeconds,)} sh -ec ${shellQuote({ script, },)}`;
}

/**
 The step file a detached guest script is started from: its bound, its one log, and the status line it ends with.

 The status line is why a detached script can be judged at all: the host reads it out of the shared folder after the
 script has gone, so a script that outlived the command that started it still reports what it did.

 @param guestTimeoutSeconds - whole seconds the guest's `timeout` gives the script

 @param script - the shell script the guest runs

 @param log - the log's name inside the guest's shared folder

 @returns the file's whole text, ending in a newline

 @example
 ```ts
 await writeFile(stepPath, detachedStepText({ guestTimeoutSeconds: 1740, script, log: 'build.log' }));
 ```
 */
export function detachedStepText({
  guestTimeoutSeconds,
  script,
  log,
}: {
  readonly guestTimeoutSeconds: number;
  readonly script: string;
  readonly log: string;
},): string {
  return [
    `timeout ${String(guestTimeoutSeconds,)} sh -ec ${shellQuote({ script, },)} > /mnt/shared/${log} 2>&1`,
    'status=$?',
    `echo "${ENDED_MARK}$status" >> /mnt/shared/${log}`,
    '',
  ].join('\n',);
}

/**
 The launcher a detached guest script is started from: the step file, in a session of its own, with nothing held open.

 A step file started in the foreground would end with the `mvm exec` that started it, and the guest's own job control
 would take its output with it. `setsid nohup` leaves it holding neither the command's terminal nor its pipes, so the
 host is free to ask the shared folder how the run went.

 @param log - the log's name inside the guest's shared folder, which names the step file beside it

 @returns the file's whole text, ending in a newline

 @example
 ```ts
 await writeFile(launchPath, detachedLaunchText({ log: 'build.log' }));
 ```
 */
export function detachedLaunchText({ log, }: { readonly log: string; },): string {
  return [
    `setsid nohup sh /mnt/shared/${log}.step.sh < /dev/null > /dev/null 2>&1 &`,
    'exit 0',
    '',
  ].join('\n',);
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
 The exit status a detached guest script wrote, read out of its whole pulled log.

 The line is found the way the drivers always found it: the first line of the log that is the mark followed by decimal
 digits and nothing else. A log that holds no such line answers empty text, which a caller reports as a script that
 ended without a status.

 @param text - the whole log the guest wrote

 @returns the digits of the status, or empty text when the log holds no status line

 @example
 ```ts
 endedStatusText({ text: 'building\nDONE=1\n' }); // '1'
 ```
 */
export function endedStatusText({ text, }: { readonly text: string; },): string {
  for (const line of text.split('\n',)) {
    /**
     The status this line carries, empty when the line is not a status line at all.
     */
    const status = statusOfLine({ line, },);
    if (status !== '')
      return status;
  }
  return '';
}

/**
 The value one flag was given on the command line, or the fallback when the flag is absent or holds no value.

 A flag written last, with nothing after it, is absent for this purpose: the drivers always fell back rather than
 reading an empty name, and an empty guest name would be a different guest.

 @param argumentWords - the command line's words after the step

 @param flag - the flag whose value is read, spelled out such as `--name`

 @param fallback - what is answered when the flag holds no value

 @returns the flag's value, or the fallback

 @example
 ```ts
 flagValue({ argumentWords: ['--name', 'guest-a'], flag: '--name', fallback: 'default' }); // 'guest-a'
 ```
 */
export function flagValue({
  argumentWords,
  flag,
  fallback,
}: {
  readonly argumentWords: readonly string[];
  readonly flag: string;
  readonly fallback: string;
},): string {
  /**
   Where the flag stands in the command line, or minus one when it is absent.
   */
  const at = argumentWords.indexOf(flag,);
  if (at === ABSENT_INDEX)
    return fallback;
  return argumentWords[at + 1] ?? fallback;
}

/**
 Whether one flag was written on the command line at all.

 @param argumentWords - the command line's words after the step

 @param flag - the flag whose presence is asked, spelled out such as `--keep-guest`

 @returns true when the flag is one of the words

 @example
 ```ts
 flagGiven({ argumentWords: ['--keep-guest'], flag: '--keep-guest' }); // true
 ```
 */
export function flagGiven({
  argumentWords,
  flag,
}: {
  readonly argumentWords: readonly string[];
  readonly flag: string;
},): boolean {
  return argumentWords.includes(flag,);
}

/**
 The status one log line carries, empty when the line is not a status line.

 @param line - one line of a detached script's log

 @returns the digits after the mark, or empty text

 @example
 ```ts
 statusOfLine({ line: 'DONE=0' }); // '0'
 ```
 */
function statusOfLine({ line, }: { readonly line: string; },): string {
  if (!line.startsWith(ENDED_MARK,))
    return '';
  /**
   What stands after the mark, which is the status when it is decimal digits and nothing else.
   */
  const status = line.slice(ENDED_MARK.length,);
  return holdsOnlyDigits({ word: status, },) ? status : '';
}

/**
 Whether one word is decimal digits and nothing else.

 The scan is one pass over the word with no regular expression and no copy of it: a pulled log is the guest's own
 output and has no bound, and this reads only the short word between a mark and the line's end.

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
