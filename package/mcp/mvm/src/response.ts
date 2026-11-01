/**
 MCP response formatting helpers shared across tool handlers.
 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

//region Types: response shape definitions

/**
 Single text content item in an MCP response.
 */
type TextContent = {
  text: string;
  type: 'text';
};

/**
 Successful MCP response containing text content.
 */
type TextResponse = { content: [TextContent,]; };

/**
 MCP error response containing text content and an error flag.
 */
type ErrorResponse = {
  content: [TextContent,];
  isError: true;
};

//endregion Types

//region Response builders: construct MCP-compliant response objects

/**
 Literal type constant for text content items.
 */
const TEXT_TYPE = 'text' as const;

/**
 Build a successful MCP text response.
 
 @param text - Response message body
 
 @returns MCP response with a single text content item
 
 @example
 ```ts
 return textResponse('VM created.');
 ```
 */
export function textResponse(text: string,): TextResponse {
  return { content: [{
    type: TEXT_TYPE,
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
},): ErrorResponse {
  /**
   Human-readable text preserving caller-provided diagnostics.
   */
  const message = caughtValueText(err,);
  console.error(`[mcp-mvm] ${tag} failed: ${message}`,);
  return {
    content: [{
      type: TEXT_TYPE,
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
},): ErrorResponse {
  console.error(`[mcp-mvm] ${tag} refused: ${text}`,);
  return {
    content: [{
      type: TEXT_TYPE,
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
