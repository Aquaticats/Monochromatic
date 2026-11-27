/**
 Quote one block of shell for a guest's `sh -ec`.

 A driver's guest script holds quotes of its own (a path a pool's listing prints, a marker line it greps for), so the
 script is closed at every quote inside it, that quote is escaped, and the script is reopened: the word the guest's
 shell reads is the script byte for byte, which a bare pair of quotes around it would not give.
 */

/**
 Quote one block of shell, quotes included.

 @param script - the shell script to quote

 @returns the script as one shell word, which the guest's `sh -ec` reads back unchanged

 @example
 ```ts
 shellQuote(`grep --quiet --fixed-strings 'FORCEDACT phase=arm-finished' /root/forced-act-arm.log`);
 ```
 */
export function shellQuote(script: string,): string {
  return `'${script
    .replaceAll(
      "'",
      `'\\''`,
    )}'`;
}
