/**
 Logger the stdio transport reports through.

 This package is a library an MCP client embeds, so it must not choose where the
 embedder's logs live. The default sink set would give it a file sink whose
 records go to whichever project directory the host runs from, and that sink
 writes `debug` records too, which the two per-frame traces use: every inbound and
 outbound JSON-RPC frame, tool payloads included, would be appended to disk
 without the embedder asking.

 The console sink alone keeps every record off stdout, which carries the frames
 themselves, and puts nothing on disk.

 @module
 */

import { createLogger, } from '@monochromatic-dev/logger-core/ts/create-logger.ts';
import { createConsoleSink, } from '@monochromatic-dev/logger-core/ts/sink/console.ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts/tagged.ts';
import type { Logger, } from '@monochromatic-dev/logger-core/ts/types.ts';

/**
 Logger over the console sink alone, so this library writes to the embedder's
 stderr and nowhere else.
 */
const { logger: consoleOnlyLogger, } = createLogger({ sinks: [createConsoleSink(),], },);

/**
 Root logger of the transport, tagged so a line names this library.
 */
const diagnosticLogger: Logger = tagged({
  l: consoleOnlyLogger,
  tag: 'mcp-stdio',
},);

export { diagnosticLogger, };
