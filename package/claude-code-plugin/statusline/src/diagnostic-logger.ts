/**
 Logger the statusline command reports degraded rendering through.

 Claude Code renders this command's stdout as the status line itself, so a
 diagnostic has to reach stderr: the logger is built from the console sink alone,
 and the console sink never sends the levels used here to stdout.

 The default sink set is deliberately not used.
 A render is a fresh process, so the default file sink would create one log file
 per render for a cause that repeats on every render, such as a home directory
 without a settings file.
 A `debug` record reaches stderr only when the operator sets
 `MONOCHROMATIC_VERBOSE=true`, which is what keeps an ordinary render silent.

 @module
 */

import { createLogger, } from '@monochromatic-dev/logger-core/ts/create-logger.ts';
import { createConsoleSink, } from '@monochromatic-dev/logger-core/ts/sink/console.ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts/tagged.ts';
import type { Logger, } from '@monochromatic-dev/logger-core/ts/types.ts';

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

export { diagnosticLogger, };
