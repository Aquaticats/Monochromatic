#!/usr/bin/env node
/**
 Claude Code statusline command entry point.
 
 @module
 */

import { text as readStreamText, } from 'node:stream/consumers';
import { pathToFileURL, } from 'node:url';

import { renderStatusline as renderStatuslineForMain, } from './render.ts';
import type { StatuslineInput, } from './types.ts';

/**
 Parses trusted Claude Code statusline JSON input.
 
 @param raw - raw stdin JSON payload
 
 @returns parsed statusline input
 
 @example
 ```ts
 parseStatuslineInput('{"model":{"display_name":"Opus"}}');
 ```
 */
function parseStatuslineInput(raw: string,): StatuslineInput {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- trusted JSON from Claude Code statusline dispatch.
  return JSON.parse(raw,) as StatuslineInput;
}

/**
 Reads stdin, renders the statusline, and writes one line to stdout.
 
 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  /**
   Parsed statusline payload from stdin.
   */
  const input = parseStatuslineInput(await readStreamText(process.stdin,),);
  /**
   Rendered statusline text.
   */
  const line = await renderStatuslineForMain({
    input,
    renderedAtMs: Date.now(),
  },);

  if (line.length > 0)
    console.log(line,);
}

/**
 Checks whether this module is the process entrypoint.
 
 @param metaUrl - current module URL
 
 @param entrypointPath - process entrypoint path
 
 @returns whether the module should execute the CLI
 
 @example
 ```ts
 isDirectRun({ metaUrl: import.meta.url, entrypointPath: process.argv[1] });
 ```
 */
function isDirectRun({
  metaUrl,
  entrypointPath,
}: Readonly<{
  metaUrl: string;
  entrypointPath: string;
}>,): boolean {
  /**
   Entrypoint file URL.
   */
  const entrypointUrl = pathToFileURL(entrypointPath,);
  return entrypointUrl.href === metaUrl;
}

/**
 Entrypoint path supplied by Node.
 */
const [, entrypointPath,] = process.argv;
if ((entrypointPath !== undefined) && isDirectRun({
  metaUrl: import.meta.url,
  entrypointPath,
},))
  await main();

export {
  isDirectRun,
  main,
  parseStatuslineInput,
};

export { renderStatusline, } from './render.ts';

/**
 Exposed so built-artifact tests reach the activity word reader.
 
 @internal
 */
export {
  assistantTextOfLine,
  readActivityWord,
} from './activity.ts';

/**
 Exposed so built-artifact tests reach the gerund scanner.
 
 @internal
 */
export {
  findGerundInText,
  scanGerundMatches,
} from './gerund.ts';

/**
 Exposed so built-artifact tests reach the model display formatter.
 
 @internal
 */
export { formatModelDisplay, } from './model.ts';

/**
 Exposed so built-artifact tests reach the per-session activity memory.
 
 @internal
 */
export {
  memoryPathOf,
  readRememberedActivityWord,
} from './activity-memory.ts';
