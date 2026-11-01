# Wire the search/fetch tools into Mistral Vibe sessions

Status: accepted 2026-10-31.
The user is building the one-shot CLI client per this recommendation;
this document records the options,
the measured evidence,
and the design facts the CLI needs.
The blocked path is documented separately in
`doc/troubleshooting/vibe-search-fetch-mcp.md`:
Mistral Vibe 3.0.4 opens MCP connections with the legacy `initialize`
handshake only,
so it cannot reach this repository's modern-only server
(revision `2026-07-28`).

## Goal

Expose `web_search` and `web_fetch` with full shared-core parity
inside Mistral Vibe sessions,
whose built-in `connector_web_search` surface measures weaker
on exactly the request classes this repository's work hits
(evidence below).

## Options and ranking

Ranking: skill + one-shot client > keep unwired > bridge shim > direct core CLI.

- Skill + one-shot client (accepted):
  a repository-owned CLI spawns the built server once per call,
  sends `server/discover` and one `tools/call`,
  and prints the result;
  a project-scope Vibe plugin skill under `.vibe/plugins/`
  teaches agents to invoke the CLI through bash.
  Beats keep-unwired because the measured gaps are structural,
  and beats the shim with a fraction of the machinery and no
  era compromise.
- Keep unwired:
  zero work,
  but sessions in this repository keep a measurably weaker
  search surface than every other host this repository uses.
- Bridge shim:
  a stdio process answering Vibe's `initialize` and translating to
  `server/discover` would buy the native tool palette,
  `/mcp` UI,
  and permission flow,
  but is the most new code and reinstates a legacy-era surface
  adjacent to what the modern-only scope decision rejected
  (`doc/handover/mcp-stdio-2026-07-28.md`).
- Direct core CLI:
  a script importing
  `@monochromatic-dev/search-fetch-core` executors
  skips MCP framing but costs the same as the one-shot client
  while bypassing the server surface the other hosts share,
  so it is dominated.

Ruled out after reading the harness source:

- Plugin `mcp.json`:
  plugin MCP servers resolve into the same
  `ResolvedMCPServerConfig` and connect through the same
  `_StdioConnection` handshake path
  (`vibe/app_server/_session_backend_port.py:152`,
  `mistralai_vibe_local_harness/vibe/_mcp_transport.py:275`),
  so the gap is identical with a different diagnostic.
- Plugin `connectors.json`:
  declares Mistral-hosted managed connectors only.
- Plugin `libraries.json`:
  materializes `NODE_PATH` and `PYTHONPATH` aliases for commands;
  not a tool surface on its own.

## Design facts for the CLI

The one-shot flow was verified live on 2026-10-31:
a single spawn that sends `server/discover` followed by
`tools/call` with the revision metadata returns real results.

Every request must carry the revision metadata:

```jsonc
// request params._meta
{
  "io.modelcontextprotocol/protocolVersion": "2026-07-28",
  "io.modelcontextprotocol/clientCapabilities": {},
  "io.modelcontextprotocol/clientInfo": { "name": "<client>", "version": "0.0.0" }
}
```

Facts that shape a per-call client:

- One spawn per call works;
  the server needs no handshake before `tools/call`.
- The `resources` index is in memory and bounded to 32 responses,
  so a one-shot client cannot read a resource after its process exits.
- The truncated head names a temp file
  (`node_modules/.monochromatic/search-fetch/pi-search-fetch-response-<id>/response.jsonl`)
  that survives the spawn,
  so the full text of a truncated `web_search` response is still
  reachable from disk.
- The parameter schemas are open objects:
  unknown keys produce a warning line instead of a rejection.
- Search responses are JSONL,
  one result per line,
  each with `title`, `site_name`, `authority`, `rerank_score`,
  `date`, `url`, `summary`, `snippet`, and full page `content`.

## Evidence

### Protocol gap

Vibe 3.0.4 is the latest release,
has no per-server protocol knob
(user-managed fields are `prompt`, `sampling_enabled`, `disabled`,
`disabled_tools`),
and both of its MCP code paths call `session.initialize()` directly,
never the vendored SDK's `negotiate_auto`.
Full chain with citations:
`doc/troubleshooting/vibe-search-fetch-mcp.md`.

### Benchmark, 2026-10-31

Method:
10 search queries and 10 fetch URLs,
identical on both arms,
one sequential run per arm.
Capability differences are structural;
latency numbers are directional (single run, arms not interleaved).

Search arm, shared core through a one-shot stdio client:

- 20 of 20 calls succeeded (all 10 searches, all 10 fetches).
- Latency 693 to 1325ms per search.
- 3 to 14 results per query (DeepSeek varies);
  full page content on nearly every result;
  dates present on most results.
- Authoritative top hit on 10 of 10 queries
  (modelcontextprotocol.io, docs.astral.sh `uv tool update-shell`,
  mise docs, cargo-nextest docs, `peps.python.org` PEP 779,
  the oxlint `Inline ignore comments` doc, the pnpm repository,
  mistral.ai, the typebox repository).

Search arm, built-in `connector_web_search` (Brave):

- Latency 1390 to 2290ms per search.
- 8 to 20 results (schema caps `limit` at 20);
  snippets only, no page content;
  `date` null on every result in every query.
- Authoritative top hit on roughly 6 of 10 queries.
  Named misses:
  `modelcontextprotocol.info` (an unofficial mirror) for the spec query,
  `xjavascript.com` (a content farm) for the oxlint query,
  a Cloudsmith blog for the PEP question.

Fetch arm, shared core `web_fetch`:

- All 10 URLs returned real content,
  including the npmjs package page (a JavaScript SPA,
  24.8s through the render chain, 9859 chars).
- GitHub blob through the `gh` CLI:
  clean markdown, 0 HTML tags, 0 page chrome, 11340 chars.
- MCP spec page: 37381 chars of full spec text.
- No page chrome on any fetch.

Fetch arm, built-in `open_url`:

- Latency 1.1 to 3.1s (likely caching).
- GitHub pages carry GitHub chrome
  (`Skip to content`, `You signed in`, `Copy path`).
- MCP spec page: 5015 chars, a docs-index stub.
- npmjs package page: 378 chars,
  a Cloudflare verification block page,
  a functional failure.
- MDN, react.dev, PEP, go.dev, Hacker News,
  and the Korean news page: comparable to the shared core.

### First head-to-head, 2026-10-31

Same-query and same-URL probes before the full corpus confirmed the
shape:
the shared core returned up to 50 results with page content
(483KB, truncated head plus temp path)
against the built-in's 20-result snippet ceiling,
and a clean `gh`-routed GitHub fetch
(79787 chars, zero chrome)
against the built-in's rendered page with GitHub UI noise.

## Caveats

- One run per arm,
  sequential,
  so latency comparisons are directional only.
- The built-in fetch latency advantage is consistent with a cache.
- Brave result count is capped by the tool schema at 20;
  the shared core returned fewer results per query on this corpus
  (3 to 14) but each carried full page content.

## Rejected ideas

- Re-registering the server in Vibe:
  blocked by the protocol gap,
  not a preference.
- The bridge shim and the direct core CLI:
  ranked out in the Options section.
- Plugin `mcp.json`, `connectors.json`, `libraries.json`:
  ruled out with source citations above.

## Next action

The user builds the CLI.
When it lands,
wire the project-scope plugin skill that invokes it,
and verify at the boundary:
a fresh Vibe session must call `web_search` and `web_fetch`
through the skill with results matching the shared core's behavior
suite.
