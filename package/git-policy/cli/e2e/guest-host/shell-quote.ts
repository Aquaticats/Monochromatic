/**
 Quote one block of shell so a guest's `sh -ec` reads it as one word.

 A guest script is one command line the host hands `mvm`, and `mvm` hands that
 line to the guest's shell, so the script's own quoting has to survive both. A
 single quote ends the quoted run, an escaped quote follows, and the run opens
 again: the shape `sh` itself documents for quoting arbitrary text.

 @example
 ```ts
 shellQuote('echo one');
 ```
 */

/**
 Quote one block of shell for a guest's `sh -ec`, quotes included.

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
