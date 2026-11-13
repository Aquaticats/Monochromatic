/**
 The folder Pi keeps its session files in, whose room a new run needs: the one drive a run cannot work without that holds none of
 its own work.

 Pi writes every session it starts into a folder of its own, `~/.pi/agent/sessions` unless a session sends it elsewhere, and a
 driver that runs Pi writes one session per run there. On 2026-11-12 that drive filled and every run of the day ended on Pi's own
 write of its session, so the gate reads the room of this folder beside the room of the runs folder.

 @module
 */

import { join, } from 'node:path';

/**
 The variable Pi reads its session folder from, named by Pi's own documentation (`docs/sessions.md`): the CLI option
 `--session-dir`, this variable, or the `sessionDir` setting chooses another location, in that order.
 */
export const SESSION_FOLDER_VARIABLE = 'PI_CODING_AGENT_SESSION_DIR';

/**
 The parts of the home directory Pi keeps its session files under while nothing names another folder.
 */
const SESSION_FOLDER_IN_HOME: readonly string[] = [
  '.pi',
  'agent',
  'sessions',
];

/**
 Reads the folder Pi keeps its session files in.

 The folder a session sends Pi to wins over the one under the home directory, because Pi writes there and that is the drive the
 write fills. A `sessionDir` written in Pi's settings file is not read here: this command does not parse that file, so a session
 that sets it without the variable is guarded by the floor on the folder Pi would have used.

 @param home - the home directory the session runs with

 @param sessionFolder - the folder the environment sends Pi to, empty when it names none

 @returns the folder whose room the gate reads

 @example
 ```ts
 const folder = sessionFolderOf({ home: '/home/a', sessionFolder: '', },);
 ```
 */
export function sessionFolderOf({
  home,
  sessionFolder = '',
}: {
  readonly home: string;
  readonly sessionFolder?: string;
},): string {
  return (sessionFolder === '')
    ? join(
      home,
      ...SESSION_FOLDER_IN_HOME,
    )
    : sessionFolder;
}
