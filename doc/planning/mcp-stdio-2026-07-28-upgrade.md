# Fix `mvm` MCP and move `@monochromatic-dev/mcp-stdio` to protocol revision 2026-07-28

Status:
 done.
Opened 2026-08-16 after `/mcp` reported `Failed to reconnect to mvm: CONNECTION_CLOSED`.

## Defect 1: `mvm` server path was stale

`~/.claude.json` registered the server as
`/home/user/Monochromatic/packages/mcp/mvm/dist/final/node/index.mjs`.
The path segment is `packages`,
 but the repo directory is `package` (rule `SGD`,
 singular segments):
commit `ece5b7553` (2026-07-15) renamed `packages/` to `package/` and the registration was never updated.
Reproduced directly:

```text
Error: Cannot find module '/home/user/Monochromatic/packages/mcp/mvm/dist/final/node/index.mjs'
code: 'MODULE_NOT_FOUND'
```

Node exits before writing a single JSON-RPC frame,
 which Claude Code surfaces as `CONNECTION_CLOSED`.

The built artifact also predated the rename:
 `dist/final/node/index.mjs` was built 2026-07-13,
while the last source change under `package/mcp/mvm/src` is the 2026-07-15 rename commit.

Fixed by rebuilding and re-registering through the CLI rather than hand-editing the config,
since concurrent sessions rewrite `~/.claude.json`:

```sh
claude mcp remove mvm -s user
claude mcp add --scope user mvm -- node /home/user/Monochromatic/package/mcp/mvm/dist/final/node/index.mjs
```

## Defect 2: the library implemented a revision two eras behind

`protocol.ts` pinned `PROTOCOL_VERSION = '2025-03-26'`.
Released revisions,
 from `modelcontextprotocol/modelcontextprotocol` `schema/`:
`2024-11-05`,
 `2025-03-26`,
 `2025-06-18`,
 `2025-11-25`,
 `2026-07-28`,
 plus `draft`.
`https://modelcontextprotocol.io/specification/versioning` names **2026-07-28** the current revision.

### What changed in 2026-07-28

The `initialize` handshake is no longer the negotiation mechanism.
The spec calls handshake revisions (`2025-11-25` and earlier) **legacy** and the new scheme **modern**:

-   Every request carries `_meta["io.modelcontextprotocol/protocolVersion"]`,
    plus a required `_meta["io.modelcontextprotocol/clientCapabilities"]`
    and an optional `_meta["io.modelcontextprotocol/clientInfo"]`.
-   Servers **MUST** implement `server/discover`,
     returning `supportedVersions`,
     `capabilities`,
     `instructions`.
-   Unsupported versions get `UnsupportedProtocolVersionError`,
     JSON-RPC code `-32022`,
    with `data.supported` and `data.requested`.
-   Every result **MUST** carry `resultType` (`'complete'` for ordinary results).
-   `server/discover` and `tools/list` results are `CacheableResult`:
     they also carry `ttlMs` and `cacheScope`.
-   Results **SHOULD** carry `_meta["io.modelcontextprotocol/serverInfo"]`.
-   `Tool.description` became optional and `Tool` gained `title`,
     `outputSchema`,
     `annotations`,
     `icons`,
     `_meta`.
-   `CallToolResult` gained `structuredContent`.
-   `ping` was removed.

### Scope decision: modern only

The user directed "make our mcp/stdio support the latest mcp spec only",
so the library serves 2026-07-28 and nothing else rather than becoming dual-era.
`initialize` is answered with `-32601` naming the revision this server implements,
which is the diagnostic the spec asks a modern-only server to provide,
since a handshake-era client has no way to fall forward.

#### Re-affirmed once the cost was known

The scope was first chosen while Claude Code appeared to probe `server/discover` first.
It does so only while a remote feature gate is on
(`doc/troubleshooting/claude-code-mcp-protocol-negotiation.md`),
 and with that gate off the server is unreachable from Claude Code entirely,
 not merely mis-reported by `claude mcp list`.

Presented with that cost and the alternatives,
 the user chose to stay modern-only and accept the unreachability.
Adding a legacy fallback and patching the client were both considered,
 and both declined.
So this is now a decision made with the price known,
 rather than one resting on the earlier reading.

## Measured client behavior: Claude Code 2.1.233 is dual-era

Probed with a throwaway stdio server registered at local scope in a scratch directory,
health-checked through `claude mcp list` and `claude mcp get`.

When the probe answers `server/discover`,
 the client stays modern and sends `tools/list`
with the same `_meta`.
Returning `{"tools":[]}` without `resultType` is rejected:

```text
probe: ... - ! Connected - tools fetch failed - Invalid result for tools/list: missing required resultType
- servers implementing protocol revision 2026-07-28 MUST include it
(the absent-means-complete bridge applies only to earlier-revision servers)
```

When the probe rejects `server/discover` with `-32601`,
 the client falls back to `initialize`
and asks for `2025-11-25`.

Consequence:
 the modern-only choice is safe for this client's session path,
 because it probes discovery first.
A client that opens with `initialize` instead is not served.

### Correction, measured later the same day: the CLI health check never probes

The `Consequence` paragraph of `Measured client behavior: Claude Code 2.1.233 is dual-era`
describes the in-session connection.
The `claude mcp get` and `claude mcp list` health checks behave differently,
 and this was measured by tapping a disposable registration pointed at the same binary:

```text
IN  {"method":"initialize","params":{"protocolVersion":"2025-11-25",...}}
OUT {"jsonrpc":"2.0","id":0,"error":{"code":-32601,"message":"Method not found: initialize..."}}
```

One message in,
 one error out,
 and `server/discover` never sent.
So a modern-only server can never pass the CLI health check,
 and `claude mcp list` reports it as failed while it works in session.

This also corrects how this work was verified.
Several `claude mcp get mvm` runs earlier that day reported Connected,
 and were cited as boundary evidence.
A fresh health check cannot produce that result against this server,
 so those readings reflected connection state the session already held rather than a
handshake the check performed.
The durable evidence is driving the built binary over stdio,
 which is what the package tests do.

## Delivered

Commits `b27a18322`,
 `8d0713ba9`,
 `08ade26bf`,
 `9e3fd6c6a`,
 `62a7cfed7`.

Protocol work in `package/mcp/stdio`:

-   `server/discover`,
     per-request revision validation,
     `-32022` with its mandated `data`,
    `resultType` on every result,
     `ttlMs`/`cacheScope` on the two cacheable results,
    `serverInfo` in result `_meta`,
     and the `initialize` diagnostic.
-   Tool definitions gained `title`,
     `outputSchema`,
     `annotations`;
     results gained `structuredContent`.

Defects corrected along the way,
 several surfaced by a source-bearing `sol` review:

-   Structurally invalid messages reported `-32700` where JSON-RPC requires `-32600`.
-   The message guard admitted `null`,
     boolean,
     and object ids that no response could echo.
-   Malformed `tools/call` arguments were silently replaced with `{}`.
-   Tool failures surfaced as JSON-RPC errors the model cannot see;
     they are now `isError` results.
-   Duplicate tool names silently overwrote each other in the registry.
-   An unserializable result closed the connection instead of returning a frame.
-   A multi-byte character split across the final stdin chunk was dropped.
-   `destroy_vm` destroyed every VM when given both `name` and `all: true`.

## Rejected along the way

-   **Dual-era server.**
     Planned first and supported by the spec,
     dropped when the user scoped
    the work to the latest revision only.
     Revisit only if a client that opens with `initialize`
    needs serving.
-   **Empty object as absent metadata.**
     `readRequestMeta` returned `{}` when `_meta` was unusable,
    to escape a `no-nullish-union` report.
     The rule names an empty object outright as a banned
    stand-in for absent,
     and the shape was wrong regardless:
     it made "client sent nothing"
    indistinguishable from "client sent empty metadata".
     Replaced by failing loud at the boundary,
    the rule's prescribed branch,
     which also deleted the helper.
-   **Testing `destroy_vm` by importing package source.**
     `require-eventual-artifact` requires tests
    to exercise shipped output.
     The cases moved to the stdio boundary suite,
     which is stronger evidence.

## Verified at the user boundary

A tap on the real in-session exchange confirms the client negotiates 2026-07-28 with no
`initialize` fallback.
This tap,
 not a `claude mcp get` reading,
 is the evidence:

```text
IN  {"jsonrpc":"2.0","id":"server-discover-probe-1","method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28",...}}}
OUT {"jsonrpc":"2.0","id":"server-discover-probe-1","result":{"resultType":"complete","supportedVersions":["2026-07-28"],"capabilities":{"tools":{}},"ttlMs":0,"cacheScope":"private","instructions":"...","_meta":{"io.modelcontextprotocol/serverInfo":{"name":"mvm","version":"0.1.0","title":"mvm VM manager"}}}}
IN  {"method":"tools/list","jsonrpc":"2.0","id":0,"params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28",...}}}
OUT {"jsonrpc":"2.0","id":0,"result":{"resultType":"complete","tools":[...8 tools...],"ttlMs":0,"cacheScope":"private",...}}
```

## Known gaps

All four gaps recorded when this plan was written are now closed.
Two went in commit `107bd707a`;
 the other two were filed as issues first,
 decided with the user,
 then built.

### Closed in `107bd707a`

-   `process.stdout.write` backpressure was ignored,
     so large tool output buffered in memory.
    Fixed:
     `processStdoutWriter` now awaits `drain` whenever the stream refuses a chunk.
    The guard test in `package/mcp/stdio/src/transport.unit.test.ts`
    ("parks a write on a backed-up stream until it drains")
    was proven to fail without the fix.
    The measured failure was worse than the buffering this entry predicted:
     `serve` could return with replies still unflushed,
     which a process exiting on stdin close then loses outright.
-   Tool results carried text content only.
    Fixed:
     `ToolContent` in `package/mcp/stdio/src/protocol-tool.ts` is now the full
    content-block union of text,
     image,
     audio,
     resource link,
     and embedded resource.
-   A long `run_in_vm` blocked the read loop,
     so `notifications/cancelled` could not be observed while it ran.
    Fixed in the commit closing
    [#433](https://github.com/Aquaticat/Monochromatic/issues/433):
     the read loop only enqueues,
     and `package/mcp/stdio/src/transport-queue.ts` runs entries one at a time.
    Execution stayed serial deliberately,
     so the mvm races that concurrent handlers would have opened do not arise.
    Guards were each proven to fail without the fix.

### Closed later the same day

-   Advertised `inputSchema` was not validated against incoming arguments before dispatch,
     so a missing required field failed inside the handler.
    Fixed in the commit closing
    [#434](https://github.com/Aquaticat/Monochromatic/issues/434):
     a tool declares its arguments once in valibot,
     `package/mcp/stdio/src/tool-schema.ts` converts that into the advertised JSON Schema,
     and the same declaration gates every call.
    Conversion targets draft-2020-12 and restores the object root the spec requires,
     since a union converts to a bare `anyOf`.

Every gap recorded when this plan was written is now closed.
The review that once gated the last two never returned;
 `doc/handover/mcp-stdio-2026-07-28.md` records that it stalled and exited on its timeout.
