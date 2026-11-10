/**
 MCP response formatting helpers shared across tool handlers.
 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type {
  TextContent,
  ToolCallResult,
} from '@monochromatic-dev/mcp-stdio/ts';

import { diagnosticLogger, } from './diagnostic-logger.ts';

//region Logging

/**
 Logger for this module, re-tagged with the enclosing function name at each
 site.
 */
const responseLogger = tagged({
  l: diagnosticLogger,
  tag: 'response',
},);

//endregion Logging

//region Types: response shapes narrowed from the protocol's own

/**
 Successful response, narrowed from {@link ToolCallResult} to the single text item every
 tool here returns.
 
 Derived rather than restated, so a field the protocol adds or removes reaches these
 builders; `Omit` replaces the property instead of intersecting two `content` types, and
 the one-element tuple is what makes a second content item a compile error here rather
 than a surprise at a client.
 */
type TextResult = Omit<ToolCallResult, 'content'> & {
  readonly content: readonly [TextContent,];
};

/**
 Refused call: a text result narrowed to the error flag every failure sets.
 */
type TextErrorResult = Omit<TextResult, 'isError'> & { readonly isError: true; };

//endregion Types

//region Response builders: construct MCP-compliant response objects

/**
 Build a successful MCP text response.
 
 @param text - Response message body
 
 @returns MCP response with a single text content item
 
 @example
 ```ts
 return textResponse('VM created.');
 ```
 */
export function textResponse(text: string,): TextResult {
  return { content: [{
    type: 'text',
    text,
  },], };
}

/**
 Build an MCP error response from a caught exception.
 Logs noncoercing error text to stderr before returning it to client.
 
 @param tag - Tool name or label for the log prefix.
 
 @param err - Caught exception value.
 
 @returns MCP response with `isError: true`.
 
 @mutates err - `caughtValueText` may invoke string-conversion hooks.
 
 @example
 ```ts
 catch (err: unknown) { return errorResponse({ tag: 'exec_in_vm', err }); }
 ```
 */
export function errorResponse({
  tag,
  err,
}: {
  readonly err: unknown;
  readonly tag: string;
},): TextErrorResult {
  /**
   Human-readable text preserving caller-provided diagnostics.
   */
  const message = caughtValueText(err,);
  tagged({
    tag: errorResponse.name,
    l: responseLogger,
  },)
    .error(`${tag} failed: ${message}`,);
  return {
    content: [{
      type: 'text',
      text: `Error: ${message}`,
    },],
    isError: true,
  };
}

/**
 Build an MCP error response for arguments rejected before any backend work began.
 Separate from {@link errorResponse} because nothing was caught: the call was refused,
 so there is no exception text to preserve and nothing was attempted to roll back.
 
 @param tag - Tool name or label for the log prefix.
 
 @param text - Explanation of what the caller must send instead.
 
 @returns MCP response with `isError: true`.
 
 @example
 ```ts
 return invalidArgumentsResponse({
   tag: 'destroy_vm',
   text: 'Provide either `name` or `all: true`, not both.',
 });
 ```
 */
export function invalidArgumentsResponse({
  tag,
  text,
}: {
  readonly tag: string;
  readonly text: string;
},): TextErrorResult {
  tagged({
    tag: invalidArgumentsResponse.name,
    l: responseLogger,
  },)
    .warn(`${tag} refused: ${text}`,);
  return {
    content: [{
      type: 'text',
      text: `Error: ${text}`,
    },],
    isError: true,
  };
}

/**
 Format an exec/run result into a human-readable string.
 Includes stdout, stderr (when non-empty), and exit code.
 
 @param result - Execution result with stdout, stderr, and exitCode
 
 @returns Formatted multi-section string
 
 @example
 ```ts
 return textResponse(formatExecResult(result));
 ```
 */
export function formatExecResult(
  result: {
    readonly exitCode: number;
    readonly stderr: string;
    readonly stdout: string;
  },
): string {
  /**
   Output sections accumulated in order: stdout, stderr (when non-empty), exit code. Joined with blank lines below.
   */
  const parts: string[] = [];
  if (result.stdout
    .length
    > 0)
    parts.push(`stdout:\n${result.stdout}`,);
  if (result.stderr
    .length
    > 0)
    parts.push(`stderr:\n${result.stderr}`,);
  parts.push(`exit code: ${String(result.exitCode,)}`,);
  return parts.join('\n\n',);
}

//endregion Response builders
