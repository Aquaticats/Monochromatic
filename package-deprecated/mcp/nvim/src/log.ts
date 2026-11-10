/**
 Root logger for the mcp-nvim server.
 
 An MCP server writes its JSON-RPC frames on stdout, so this logger is built
 over the console sink alone: `debug`, `warn` and `error` records reach
 stderr, nothing logs at `info`, and no file sink writes into whichever
 project directory the embedding host runs from
 (`package/mcp/stdio/src/diagnostic-logger.ts` states the same constraints).
 
 @module
 */

import { createLogger, } from '@monochromatic-dev/logger-core/ts/create-logger.ts';
import { createConsoleSink, } from '@monochromatic-dev/logger-core/ts/sink/console.ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts/tagged.ts';
import type { Logger, } from '@monochromatic-dev/logger-core/ts/types.ts';

/**
 Logger over the console sink alone, so the server logs to its host's streams
 and never to a file.
 */
const { logger: consoleOnlyLogger, } = createLogger({
  sinks: [createConsoleSink(),],
},);

/**
 Root logger of the server, tagged so every line names this package.
 */
const rootLogger: Logger = tagged({
  l: consoleOnlyLogger,
  tag: 'mcp-nvim',
},);

export { rootLogger, };
