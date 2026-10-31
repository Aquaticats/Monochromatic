# @monochromatic-dev/search-fetch-mcp

MCP server exposing the shared web_search and web_fetch tools over stdio,
with truncated responses delivered as resources.

## Why

The pi extension `@monochromatic-dev/search-fetch-pi-plugin` serves pi hosts.
This server serves every other MCP host with the identical behavior:
DeepSeek-first search (maximum results with page content) with Exa and Linkup
fallback,
`gh` CLI routing for mapped GitHub URLs,
DeepSeek fetch before Linkup and Exa,
the global host blocklist,
base64 data-URL image stripping,
and the same tool names and descriptions.
Both surfaces delegate to `@monochromatic-dev/search-fetch-core`,
so behavior has exactly one implementation.

One surface per host:
a host that loads this server should not also load the pi extension.

## Tools

- `web_search` searches with DeepSeek first (up to 50 results with page content),
  then Exa fast search,
  then Linkup standard fallback.
- `web_fetch` fetches one page,
  routing mapped GitHub URLs through the local `gh` CLI,
  then using DeepSeek fetch,
  Linkup renderJs,
  and Exa contents.

Both keep the shared open parameter schema,
so unsupported keys produce a warning instead of a rejection.

## Truncated responses as resources

Truncated tool output still carries the model-visible head plus a full-response temp path,
identical to the pi surface.
Additionally this server records the full response and adds:

- a text line naming the `search-fetch://response/<id>` resource URI,
  and
- a `resource_link` content block pointing at the same URI.

Hosts read the full text with `resources/read`,
or list what is available with `resources/list`.
Hosts without resource support can still use the temp path when their tools can read files.
Hosts that render only text content show one placeholder line for the `resource_link` block;
OpenCodeReview is such a host.

## Registration

### Generic stdio

`search-fetch-mcp` speaks newline-delimited JSON-RPC 2.0 over stdin and stdout,
so any MCP client that launches stdio servers can register it:

```jsonc
{
  "mcpServers": {
    "search-fetch": {
      "command": "search-fetch-mcp"
    }
  }
}
```

### open-codereview.ai

OpenCodeReview registers MCP servers under `mcp_servers.<name>` in
`~/.opencodereview/config.json`:

```bash
ocr config set mcp_servers.search-fetch.command search-fetch-mcp
```

Optional allowlist restricting the exposed tools:

```bash
ocr config set mcp_servers.search-fetch.tools '["web_search", "web_fetch"]'
```

See <https://open-codereview.ai/docs/mcp> for every configuration field.

## Configuration

Loads the same global config file as the pi extension
(`~/.pi/agent/extensions/pi-search-fetch.json`),
which now holds only the blocklist.
Provider keys come from `EXA_API_KEY` and `LINKUP_API_KEY` in the process
environment,
or from the repo's sops/age-encrypted `.env.local.json` through
`@monochromatic-dev/env-local-json-core`.

The reader locates the store in this order:

1. an explicit path,
   from `ENV_LOCAL_JSON_PATH` or a caller-supplied path,
2. `.env.local.json` at the nearest repository root,
3. a bounded walk from the working directory,
   stopping at the home directory,
   the containing repository,
   or the filesystem root.

Decryption needs `sops` on `PATH` and the age identity from the shared mise
chain,
which defaults to `~/.config/mise/age.txt`.
The reader returns only keys the environment did not supply,
and it never mutates `process.env`.

When the store is unavailable,
the server still starts with the blocklist and
GitHub-routed `web_fetch` through `gh`.
Calls that need a missing provider key fail with a value-free store diagnostic.

The old `exaApiKey` and `linkupApiKey` fields are rejected.
A config still carrying them raises `SearchFetchMigrationRequiredError` naming
`mise run secrets:migrate`.

## Tests

`src/index.unit.test.ts` drives the built artifact through its own dispatch surface:
discovery capabilities,
tool listing,
ignored-key warnings,
blocklist failures as tool errors,
and the truncated-response resource round trip
(oversized stub response to `resources/read`).
