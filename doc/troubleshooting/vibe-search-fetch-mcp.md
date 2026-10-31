# Mistral Vibe 3.0.4 cannot connect to the search/fetch MCP server

## Symptom

The repository's search/fetch MCP server
(`package/search-fetch/mcp`,
bin `search-fetch-mcp`)
registered as a stdio server under the name `search-fetch`,
every Mistral Vibe session opens with:

```text
MCP server 'search-fetch' failed to connect: mcp_transport_failed.
```

The session otherwise continues normally;
the built-in `connector_web_search` connectors keep working,
so the failure is easy to miss as a warning line.

Reproduced on 2026-10-31 with Mistral Vibe 3.0.4
and a fresh `vibe -p` one-shot session
run from this repository.

## Root cause

The chain,
from the server's refusal to the surfaced warning:

1. This server implements MCP revision `2026-07-28` exclusively
   and answers the legacy `initialize` handshake with JSON-RPC `-32601`
   (`package/mcp/stdio/src/server-response.ts:188`,
   the same diagnostic recorded for Codex in
   `doc/troubleshooting/codex-search-fetch-mcp.md`):

   ```text
   Method not found: initialize. This server implements MCP revision 2026-07-28,
   which removed the initialize handshake: call server/discover and declare the
   revision in each request's params._meta instead
   ```

2. Vibe's Unified Harness stdio path opens a raw SDK `ClientSession`
   and calls the handshake method directly
   (`_StdioConnection.open`,
   installed with the `mistralai_vibe_local_harness` package of the
   `mistral-vibe` 3.0.4 uv tool,
   and upstream at
   <https://github.com/mistralai/mistral-vibe/blob/338ad4bf0f73c4dd879d667cf7ab011555ae1751/harness/runtimes/python/python/mistralai_vibe_local_harness/vibe/_mcp_transport.py#L275-L284>,
   line numbers confirmed against that commit):

   ```python
   # mistralai_vibe_local_harness/vibe/_mcp_transport.py
               session = await stack.enter_async_context(
                   ClientSession(
                       read,
                       write,
                       read_timeout_seconds=server.startup_timeout_s,
                       sampling_callback=cast(Any, sampling_callback),
                   )
               )
               await session.initialize()
   ```

   Vibe's non-harness loop does the same
   (`vibe/core/tools/mcp/tools.py:236`),
   so both Vibe surfaces are handshake-era.

3. `ClientSession.initialize()` in the vendored `mcp` 2.3.0 SDK
   sends only the legacy handshake
   (`mcp/client/session.py:651`,
   `LATEST_HANDSHAKE_VERSION` inside the installed uv tool's site-packages),
   so the server's `-32601` raises out of the call.

4. The vendored SDK does ship modern-era negotiation:
   `mcp/client/_probe.py` `negotiate_auto` probes `server/discover`
   and falls back to `initialize` for legacy servers.
   But only the higher-level `connect()` helper calls it
   (`mcp/client/client.py:465`);
   the harness uses the raw `ClientSession`,
   so the probe never runs.

5. The raised `-32601` is translated by `_translate_failure`
   into `MCPRuntimeFailure("mcp_transport_failed", ...)`
   (`_mcp_transport.py:475`),
   which the app server projects as the warning quoted in the Symptom
   (`vibe/app_server/_unified_harness_backend_adapter.py:5585`).

## Verification

Versions under test:

- `vibe --version` reports `3.0.4`;
  `vibe --check-upgrade` on 2026-10-31 reports it is the latest.
- The uv tool at
  `/var/home/user/.local/share/uv/tools/mistral-vibe`
  vendors `mcp` 2.3.0 and `mcp_types` 2.3.0.
- The server artifact was rebuilt with
  `mise run //package/search-fetch/mcp:build`
  before registration,
  because commits through 2026-10-29 postdated the previous build.

Harness 1,
raw stdio probe against the built artifact
(reproduce with the same pipe;
the server exits when stdin closes):

```sh
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"probe","version":"0.0.0"}}}' \
  | node package/search-fetch/mcp/dist/final/node/index.mjs
```

Output,
the modern-only diagnostic as a JSON-RPC error with code `-32601`:

```text
{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found: initialize. This server implements MCP revision 2026-07-28, which removed the initialize handshake: call server/discover and declare the revision in each request's params._meta instead","data":{"supported":["2026-07-28"]}}}
```

Harness 2,
registration through Vibe's own CLI
(writes the `[[mcp_servers]]` block to `~/.vibe/config.toml`):

```sh
vibe mcp add search-fetch --transport stdio \
  --command node \
  --arg /var/home/user/Monochromatic/package/search-fetch/mcp/dist/final/node/index.mjs
```

A fresh one-shot session then shows the warning verbatim:

```sh
vibe -p "Call the web_search tool provided by the search-fetch MCP server with query 'Model Context Protocol specification'. ..." --auto-approve --max-turns 6
```

```text
MCP server 'search-fetch' failed to connect: mcp_transport_failed.
```

Catalogs:

- Fails:
  any host that opens the connection with `initialize`
  (Vibe 3.0.4 harness and non-harness loops,
  Codex without the modern feature,
  per `doc/troubleshooting/codex-search-fetch-mcp.md`).
- Works:
  hosts that probe `server/discover`,
  verified for this artifact by the scripted client recorded in
  `doc/planning/search-fetch-mcp-server.md`,
  for Claude Code by
  `doc/troubleshooting/claude-code-mcp-protocol-negotiation.md`,
  and for Codex with its modern feature enabled.

## Verified workarounds

Disposition:
the registration was removed with `vibe mcp remove search-fetch`
on 2026-10-31 by user decision,
so Vibe sessions carry no trace of the server;
the mechanisms below are the verified options for a future re-registration.

- Keep the registration but disable it,
  `disabled = true` on the `[[mcp_servers]]` entry
  (a user-managed field per `vibe/core/config/mcp_servers.py`,
  `_USER_MANAGED_FIELDS`).
  A fresh `vibe -p` session then opens with no warning
  and no wasted `node` spawn,
  verified 2026-10-31.
  Tradeoffs:
  the server stays inert in every Vibe session until Vibe speaks
  revision `2026-07-28`,
  and re-enabling means editing the config again
  or using `/mcp`,
  so this only parks the registration,
  it does not deliver the tools.
- Let Vibe's built-in `connector_web_search` connectors cover search
  inside Vibe sessions.
  Tradeoffs:
  that surface does not share this repository's provider chain
  (DeepSeek first with Exa and Linkup fallback,
  `gh` routing for mapped GitHub URLs,
  the global blocklist),
  so results and rejections differ from the shared core,
  and any prompt guidance written against the shared tools'
  behavior does not carry over.
  Measured on 2026-10-31,
  the built-in surface also loses on top-hit authority,
  page content,
  and Cloudflare-blocked SPA fetches;
  the accepted route around the gap is a one-shot CLI client
  plus a plugin skill,
  recorded with the full evidence in
  `doc/planning/vibe-search-fetch-wiring.md`.

## What does not work

- A per-server protocol or revision knob:
  the user-managed `[[mcp_servers]]` fields are
  `prompt`, `sampling_enabled`, `disabled`, `disabled_tools`
  only,
  so the handshake cannot be configured away.
- Upgrading Vibe:
  3.0.4 is the newest release as of 2026-10-31.
- Reaching the SDK's auto-negotiation by configuration:
  `negotiate_auto` is only invoked inside the higher-level
  `connect()` helper,
  which no Vibe code path uses.

## Upstream filing decision

`.out-of-scope/` was checked on 2026-10-31:
no entry covers Mistral Vibe or MCP host support.

The six constraints:

1. Upstream's fault:
   yes,
   a capability gap.
   The current MCP revision is `2026-07-28`
   (recorded with its versioning evidence in
   `doc/handover/mcp-stdio-2026-07-28.md`),
   and Vibe's client speaks handshake revisions only.
2. Upstream can fix it:
   yes.
   The minimal fix is to call the vendored SDK's `negotiate_auto`
   (or probe `server/discover` then fall back)
   instead of `session.initialize()` at
   `_mcp_transport.py:275` and `vibe/core/tools/mcp/tools.py:236`;
   the SDK they already vendor ships the helper.
3. Supporting the use case:
   yes,
   MCP servers are a documented Vibe feature
   (`/mcp`, `vibe mcp add`, the `[[mcp_servers]]` config schema).
4. Contributions welcome:
   not run.
   The upstream contribution policy was not read yet.
5. Likely to fix:
   not run.
   The upstream tracker was not searched yet.
6. Minimal prototype:
   not run.
   This repository already decided,
   with the price known,
   to keep the server modern-only and to decline both a legacy
   fallback and patching clients
   (`doc/handover/mcp-stdio-2026-07-28.md`,
   scope decision re-affirmed once the cost was known).
   An upstream filing asks Mistral to modernize their client,
   which is adjacent to the declined patching path,
   so it waits for an explicit user decision.

Draft,
kept for a future session that closes constraints 4 through 6.
Do not file as-is.

~~~md
Title: MCP client only speaks handshake-era revisions; cannot connect to
servers implementing the current revision 2026-07-28

Mistral Vibe 3.0.4 registers stdio MCP servers but opens every connection
with the legacy `initialize` handshake. A server implementing the current
MCP revision `2026-07-28` (which removed `initialize` in favor of
`server/discover`) answers `-32601`, and every session warns:

    MCP server '<name>' failed to connect: mcp_transport_failed.

Reproduction: register any modern-only server:

    vibe mcp add demo --transport stdio --command node --arg /path/to/server.mjs

The vendored `mcp` Python SDK 2.3.0 already ships the fix-shaped helper:
`mcp/client/_probe.py:negotiate_auto` probes `server/discover` and falls
back to `initialize` for legacy servers, but it is only called by the
higher-level `connect()` helper in `mcp/client/client.py:465`.

Suggested fix: in
`harness/runtimes/python/python/mistralai_vibe_local_harness/vibe/_mcp_transport.py`
(`_StdioConnection.open`) and `vibe/core/tools/mcp/tools.py`
(`enter_stdio_session`), replace `await session.initialize()` with the
SDK's auto-negotiation so modern servers connect and legacy servers keep
working.
~~~

Filing puts a real person's account in front of strangers,
so the decision belongs to the user alone;
the draft above is not a recommendation,
and the tracker and policy checks in constraints 4 and 5
must run first.
