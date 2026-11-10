/**
 Logger this package's tool responses report through.

 These are libraries an MCP client embeds, so they must not choose where the
 embedder's logs live: the default sink set would add a file sink writing into
 whichever project directory the host runs from.

 The console sink alone puts nothing on disk.
 It writes `debug`, `warn` and `error` records to stderr,
 and `info` records to stdout through `console.info`.
 Stdout carries the JSON-RPC frames,
 so this package logs nothing at `info`.

 @module
 */

import { createLogger, } from '@monochromatic-dev/logger-core/ts/create-logger.ts';
import { createConsoleSink, } from '@monochromatic-dev/logger-core/ts/sink/console.ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts/tagged.ts';
import type { Logger, } from '@monochromatic-dev/logger-core/ts/types.ts';

/**
 Logger over the console sink alone,
 so this library writes to the embedder's console streams and to no file.
 */
const { logger: consoleOnlyLogger, } = createLogger({ sinks: [createConsoleSink(),], },);

/**
 Root logger of this package, tagged so a line names it.
 */
const diagnosticLogger: Logger = tagged({
  l: consoleOnlyLogger,
  tag: 'mcp-mvm',
},);

export { diagnosticLogger, };
