/**
 Logger the statusline command reports degraded rendering through.

 Claude Code renders this command's stdout as the status line itself, so a
 diagnostic has to reach stderr: the logger is built from the console sink alone,
 and the console sink never sends the levels used here to stdout.

 The default sink set is deliberately not used.
 A render is a fresh process, so the default file sink would create one log file
 per render for a cause that repeats on every render, such as a home directory
 without a settings file.

 Levels follow how long the failure lasts.
 A failure that repeats on every render while it lasts (an absent settings file,
 a truncated transcript line, an unwritable memory directory) reports at `debug`,
 which stays quiet unless the operator sets `MONOCHROMATIC_VERBOSE=true`.
 A durable failure that would otherwise leave a segment missing with no trace
 (unparsable settings JSON, a refused permission) reports at `warn`, which reaches
 stderr on its own.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { createLogger, } from '@monochromatic-dev/logger-core/ts/create-logger.ts';
import { createConsoleSink, } from '@monochromatic-dev/logger-core/ts/sink/console.ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts/tagged.ts';
import type { Logger, } from '@monochromatic-dev/logger-core/ts/types.ts';

import { isAbsentFileError, } from './absent-file-error.ts';

/**
 Logger of the console sink alone: a record reaches stderr and nothing else, so
 this per-render process never creates a log file.
 */
const { logger: consoleOnlyLogger, } = createLogger({ sinks: [createConsoleSink(),], },);

/**
 Root logger of the statusline command, tagged so a line names the process that
 emitted it.
 */
const diagnosticLogger: Logger = tagged({
  l: consoleOnlyLogger,
  tag: 'claude-statusline',
},);

/**
 Reports a read whose failure left a segment of the status line out.

 An absent path is the ordinary state of a first render or of a home without a
 settings file, so it reports at `debug`, which reaches stderr only when the
 operator asks.
 Any other failure (a refused permission, a directory where a file belongs) is
 durable, so it reports at `warn`: without that, a permanently broken home would
 be invisible in every render.

 @param l - logger tagged for the function that hit the failure

 @param subject - what could not be rendered, naming the path

 @param error - caught value

 @example
 ```ts
 reportUnreadable({ l, subject: `effort level unreadable from ${path}`, error });
 ```
 */
function reportUnreadable({
  l,
  subject,
  error,
}: Readonly<{
  l: Logger;
  subject: string;
  error: unknown;
}>): void {
  /**
   Report line naming the path and the reason it was not read.
   */
  const line = `${subject}: ${caughtValueText(error,)}`;
  if (isAbsentFileError(error,)) {
    l.debug(line,);
    return;
  }
  l.warn(line,);
}

export {
  diagnosticLogger,
  reportUnreadable,
};
