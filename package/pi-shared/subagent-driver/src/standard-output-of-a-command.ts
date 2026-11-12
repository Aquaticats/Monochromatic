/**
 Where a command says a line as it happens: the standard output of the process, which for a command started as a systemd user unit
 is that unit's output file.

 A command that waits for something and prints nothing looks like a command that never began, which is how two reviews waited
 behind four running ones on 2026-11-12 while the launcher's output file was empty.

 @module
 */

import { stdout, } from 'node:process';

/**
 Says a line on this process's standard output.

 @param line - the line to say, with its own line ending

 @example
 ```ts
 reportOnStandardOutput('WAITING: a asks for the first pool with room;\n',);
 ```
 */
export function reportOnStandardOutput(line: string,): void {
  stdout.write(line,);
}
