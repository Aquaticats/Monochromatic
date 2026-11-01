# @monochromatic-dev/mcp-stdio

Minimal MCP server framework for stdio transport.

## Why

The official `@modelcontextprotocol/sdk` pulls in 5.8 MB and 17 dependencies
(Express,
 Hono,
 jose,
 CORS,
 rate limiting,
 SSE infrastructure)
to support HTTP transport,
 OAuth,
 and session management.
None of that applies to a local stdio server with a handful of tools.

This package implements only what a stdio tool server needs:
JSON-RPC 2.0 over newline-delimited stdin/stdout,
`server/discover`,
 `tools/list`,
 and `tools/call`.

## Usage

```ts
// package/mcp/my-server/src/index.ts
import {
  createMcpServer,
  defineTool,
  serve,
} from '@monochromatic-dev/mcp-stdio';
import * as v from 'valibot';

const server = createMcpServer({
  config: {
    name: 'my-server',
    version: '0.1.0',
    instructions: 'Greets people by name.',
  },
  tools: [
    defineTool({
      name: 'greet',
      entry: {
        description: 'Greets by name.',
        // Declared once: this becomes the JSON Schema clients see in `tools/list`, and
        // gates every call, so the advertised contract cannot drift from the enforced one.
        schema: v.strictObject({
          name: v.pipe(v.string(), v.description('Who to greet',),),
        },),
        handler: async args => ({
          content: [{ type: 'text', text: `Hello, ${args.name}!`, },],
        }),
      },
    },),
  ],
},);

await serve({ server, },);
```

## Protocol coverage

Implements MCP spec revision **2026-07-28** (stdio transport only),
and only that revision.

Revision 2026-07-28 removed the `initialize` handshake.
Instead of negotiating once per session,
every request declares its own revision in `params._meta`,
and a mandatory `server/discover` RPC reports what the server supports:

```jsonc
// client -> server
{
  "jsonrpc": "2.0",
  "id": "discover-1",
  "method": "server/discover",
  "params": {
    "_meta": {
      "io.modelcontextprotocol/protocolVersion": "2026-07-28",
      "io.modelcontextprotocol/clientCapabilities": {}
    }
  }
}
```

Implemented:

- `server/discover` reporting supported revisions,
   capabilities,
   and optional `instructions`
- `tools/list` with full tool definitions,
   including `title`,
   `outputSchema`,
   and `annotations`
- `tools/call` with argument dispatch,
   `structuredContent`,
   and the full content-block union of text,
   image,
   audio,
   resource links,
   and embedded resources
- `tools/call` arguments validated against the schema the tool advertises,
   answering `-32602` before dispatch when they do not match
- `resources/list` and `resources/read` through an optional `ResourceProvider`,
   advertised in `server/discover` only when configured
- `resultType` on every result,
   plus `ttlMs` and `cacheScope` on cacheable results
- `io.modelcontextprotocol/serverInfo` stamped into the `_meta` of every result
- `notifications/cancelled` read while a tool is running,
   and applied
- JSON-RPC error codes for parse errors,
   invalid requests,
   unknown methods,
   unknown tools,
   and unsupported protocol revisions (`-32022`)

### Clients this serves

A client that opens with `server/discover` is served.
A client that opens with `initialize` is not:
it receives a `-32601` naming the revision this server implements,
which is all a handshake-era client can be told,
since the older revisions have no way to fall forward.

Claude Code 2.1.233 behaves differently on its two paths,
 both measured by tapping the real exchange.

Its in-session connection probes `server/discover` first and falls back to `initialize`
only when that probe returns a non-modern error,
 so it stays on the modern path against this package.

Its CLI health check does not probe at all.
`claude mcp get` and `claude mcp list` send `initialize` as the first and only message,
 so a modern-only server can never pass them:
 the tapped exchange is one `initialize` in,
 one `-32601` out,
 and no `server/discover` ever sent.

Whether it probes at all is not a property of the client version.
It sits behind a remote feature gate,
 `tengu_mcp_protocol_negotiation_stdio`,
 which defaults to off,
 so the same binary and client can connect one hour and fail the next with no local change.

**Expect `claude mcp list` to report this server as failed whenever that gate is off.**
Verify by driving the binary over stdio instead,
 which is what this package's own tests do.
Source trace,
 tapped exchange,
 and the options for changing this:
 `doc/troubleshooting/claude-code-mcp-protocol-negotiation.md`.

### Deliberately omitted

HTTP/SSE transport,
 OAuth,
 resource subscriptions,
 prompts,
 sampling,
 elicitation,
 completions,
 progress notifications,
 and pagination cursors.
Tool registries here are fixed at construction,
so `tools/list` always returns one complete page.

Tool handlers run one at a time,
 in the order their requests arrived,
 so two handlers can never interleave against a shared backend.
Reading is not blocked by that:
 the stdin loop only enqueues,
 so a `notifications/cancelled` sent during a long-running tool is read and applied
immediately rather than waiting for the tool it cancels.

A cancelled request that has not started yet is dropped without ever being dispatched.
One already running is allowed to finish and its reply is withheld,
 since abandoning a half-finished side effect is usually worse than completing it.
A cancellation naming an unknown or already-answered request is ignored,
 which revision 2026-07-28 requires:
 it states one "MAY arrive after the request has already finished".

Independent calls gain no parallelism from this.
A `tools/list` issued during a long tool call waits behind it.

## Error handling

A failure thrown inside a tool handler comes back as a successful response whose result
carries `isError: true` and the message as text content,
not as a JSON-RPC error.
The spec asks for this so the model can see what failed and correct its next call;
a protocol-level error would hide the failure from it entirely.

JSON-RPC errors are reserved for failures in finding or addressing a tool:
an unknown tool name,
malformed arguments,
an unknown method,
or an unsupported protocol revision.

## Architecture

```text
src/
  json-rpc.ts                JSON-RPC 2.0 types, error codes, and message validation
  plain-object.ts            Guard narrowing untrusted parsed JSON to a keyed object
  protocol.ts                Revision constants, result envelope, cache hints, capabilities
  protocol-meta.ts           Reserved `_meta` keys and their payload types
  protocol-tool.ts           Tool definition, content, and handler types
  server.ts                  createMcpServer factory and JSON-RPC dispatch
  server-define-tool.ts      defineTool convenience helper
  server-protocol-error.ts   Error classes for the two version-validation failures
  server-request-version.ts  Per-request revision validation
  server-response.ts         Response builders and the notification handler
  server-result.ts           Result builders that stamp the protocol envelope
  server-tool-call.ts        tools/call dispatch to registered handlers
  server-tool-registry.ts    Tool entry normalization and duplicate-name rejection
  server-types.ts            Server configuration, tool entry, and handle types
  tool-schema.ts             Valibot schema conversion to JSON Schema, and call validation
  transport.ts               Connects server handle to stdin/stdout
  transport-queue.ts         Serial execution queue, cancellation, and single write path
  line-reader.ts             Async iterator yielding newline-delimited lines from a byte stream
  index.ts                   Public API re-exports
```
