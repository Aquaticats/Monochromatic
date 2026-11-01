/**
 Shell dispatch helpers for guest agent command execution.
 Handles base64 decoding of guest agent output, OS-specific
 shell argument construction for Linux and Windows guests,
 and the marker line that ties captured output to one launch.
 */

import { randomBytes, } from 'node:crypto';

/**
 Decodes a base64-encoded string to UTF-8 text.

 @param encoded - base64 string from guest agent response

 @returns Decoded UTF-8 string

 @example
 ```ts
 decodeBase64('aGVsbG8='); // => "hello"
 ```
 */
export function decodeBase64(encoded: string,): string {
  return Buffer
    .from(
    encoded,
    'base64',
    )
    .toString('utf8',);
}

/**
 Builds the guest-exec path and arguments for the given OS family and command.
 Linux uses the configured shell (bash/ash) with `-c`; Windows uses
 `powershell.exe` with `-NoProfile -NonInteractive -Command`.

 @param command - Shell command string to execute

 @param osFamily - Guest OS family (`linux` or `windows`)

 @param shell - Shell executable path or name

 @returns Object with `path` and `arg` array for the guest-exec payload

 @example
 ```ts
 execArgs({ osFamily: 'linux', shell: '/bin/bash', command: 'uname -a' });
 // => { path: '/bin/bash', arg: ['-c', 'uname -a'] }

 execArgs({ osFamily: 'windows', shell: 'powershell.exe', command: 'hostname' });
 // => { path: 'powershell.exe', arg: ['-NoProfile', '-NonInteractive', '-Command', 'hostname'] }
 ```
 */
export function execArgs({
  command,
  osFamily,
  shell,
}: {
  readonly command: string;
  readonly osFamily: string;
  readonly shell: string;
},): {
  arg: readonly string[];
  path: string;
} {
  if (osFamily === 'windows') {
    return {
      arg: [
        '-NoProfile',
        '-NonInteractive',
        '-Command',
        command,
      ],
      path: shell,
    };
  }
  return {
    arg: [
      '-c',
      command,
    ],
    path: shell,
  };
}

//region Launch marker

/**
 Text a guest command prints first so its captured output can be told apart
 from any other command's. Made only by {@link createExecMarker}: lowercase
 letters, digits and hyphens, which need no quoting in a POSIX shell or in PowerShell.

 @example
 ```ts
 const marker: ExecMarker = createExecMarker();
 ```
 */
export type ExecMarker = string & { readonly execMarker: unique symbol; };

/**
 Number of random bytes in a marker; 16 bytes make a collision between two launches implausible.
 */
const MARKER_RANDOM_BYTES = 16;

/**
 Fixed start of every marker, so a marker line is recognizable in captured output.
 */
const MARKER_PREFIX = 'mvm-exec-marker-';

/**
 Characters a marker's random part may hold.
 */
const MARKER_DIGITS = '0123456789abcdef';

/**
 Checks that `text` is a marker: the fixed prefix followed by lowercase hexadecimal digits.
 Both shells take such text inside single quotes without any escaping.

 @param text - Candidate marker

 @returns Whether `text` may be interpolated into a shell command as a marker

 @example
 ```ts
 isExecMarker('mvm-exec-marker-0f3a'); // true
 isExecMarker("mvm-exec-marker-'; rm"); // false
 ```
 */
function isExecMarker(text: string,): text is ExecMarker {
  if ((!text.startsWith(MARKER_PREFIX,)) || (text.length === MARKER_PREFIX.length)) {
    return false;
  }
  for (const character of text.slice(MARKER_PREFIX.length,)) {
    if (!MARKER_DIGITS.includes(character,)) {
      return false;
    }
  }
  return true;
}

/**
 Creates a marker no earlier launch can have printed.

 @returns Fresh marker text

 @throws Error when the generated text is not a marker, which the generator cannot produce

 @example
 ```ts
 createExecMarker(); // => 'mvm-exec-marker-3f9c...'
 ```
 */
export function createExecMarker(): ExecMarker {
  /**
   Candidate marker; hexadecimal digits after the fixed prefix.
   */
  const text = `${MARKER_PREFIX}${
    randomBytes(MARKER_RANDOM_BYTES,)
      .toString('hex',)
  }`;
  if (!isExecMarker(text,)) {
    throw new Error(`generated launch marker has unexpected characters: ${text}`,);
  }
  return text;
}

/**
 Builds the guest-exec path and arguments for a command that first prints `marker`
 on a line of its own, then runs `command` unchanged.

 A POSIX shell gets the marker statement on its own first line, so it runs
 before the shell reads the command, even a command with a syntax error.
 PowerShell gets `'marker'; command`; it parses the whole text before running
 any of it, so a command that does not parse prints no marker.

 @param command - Shell command string to execute after the marker

 @param marker - Marker to print first

 @param osFamily - Guest OS family (`linux` or `windows`)

 @param shell - Shell executable path or name

 @returns Object with `path` and `arg` array for the guest-exec payload

 @example
 ```ts
 markedExecArgs({ command: 'uname -a', marker, osFamily: 'linux', shell: '/bin/bash' });
 // => { path: '/bin/bash', arg: ['-c', "printf '%s\\n' 'mvm-exec-marker-...'\nuname -a"] }
 ```
 */
export function markedExecArgs({
  command,
  marker,
  osFamily,
  shell,
}: {
  readonly command: string;
  readonly marker: ExecMarker;
  readonly osFamily: string;
  readonly shell: string;
},): {
  arg: readonly string[];
  path: string;
} {
  return execArgs({
    command: osFamily === 'windows'
      ? `'${marker}'; ${command}`
      : `printf '%s\\n' '${marker}'\n${command}`,
    osFamily,
    shell,
  },);
}

/**
 Sentinel returned by {@link outputAfterMarker} when the output does not begin with the marker.
 A unique symbol models "not this launch's output" without a nullish union.
 */
export const NOT_MARKED: unique symbol = Symbol(
  'returned when captured output does not begin with the expected launch marker',
);

/**
 Removes the marker line from the start of a command's captured standard output.

 @param marker - Marker the launch was told to print first

 @param stdout - Decoded standard output reported by the guest agent

 @returns Output after the marker line, or {@link NOT_MARKED} when `stdout` does not begin with `marker`

 @example
 ```ts
 outputAfterMarker({ marker, stdout: `${marker}\nhello\n` }); // => 'hello\n'
 outputAfterMarker({ marker, stdout: 'hello\n' });            // => NOT_MARKED
 ```
 */
export function outputAfterMarker({
  marker,
  stdout,
}: {
  readonly marker: ExecMarker;
  readonly stdout: string;
},): string | typeof NOT_MARKED {
  if (!stdout.startsWith(marker,)) {
    return NOT_MARKED;
  }
  /**
   Output after the marker text; still begins with the line ending the shell wrote.
   */
  const rest = stdout.slice(marker.length,);
  if (rest.startsWith('\r\n',)) {
    return rest.slice('\r\n'.length,);
  }
  if (rest.startsWith('\n',)) {
    return rest.slice('\n'.length,);
  }
  return rest;
}

//endregion Launch marker
