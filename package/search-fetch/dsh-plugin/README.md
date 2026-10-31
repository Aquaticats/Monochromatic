# @monochromatic-dev/search-fetch-dsh-plugin

A configuration-only DeepSeek Harness bundle that connects
`@monochromatic-dev/search-fetch-mcp` to a dsh profile over stdio.

## Why

dsh ships exactly one web search provider,
`@deepseek-ai/dsh-web-search-deepseek`,
and the shared base wires it as the only `ctx.web` search backend:
`searchProvider: deepseek-official` in the `dsh-base` bundle patch,
with `apiKeyEnv: DEEPSEEK_API_KEY`.
A deployment without a DeepSeek account therefore has no working `web_search`
at all,
while `web_fetch` keeps working because `@deepseek-ai/dsh-web-fetch-http` is
anonymous public HTTP.

This bundle supplies search from the repo's own provider stack instead.
It does not add a second implementation of search behavior.
`@monochromatic-dev/search-fetch-mcp` already exists to serve
"every other MCP host with the identical behavior" as the pi extension,
and both surfaces delegate to
`@monochromatic-dev/search-fetch-core`,
so DeepSeek-first search (maximum results with page content) with Exa and
Linkup fallback,
DeepSeek fetch before Linkup and Exa,
`gh` CLI routing for mapped GitHub URLs,
the global host blocklist,
and base64 data-URL image stripping all keep one implementation.

dsh is such a host,
so the MCP surface is the intended integration point rather than a new native
`ctx.web` provider,
which would create a third surface.

## What it provides

Two model-facing tools,
namespaced by the `search-fetch` server name:

- `mcp__search-fetch__web_search` searches with DeepSeek first (up to 50
  results with page content),
  then Exa fast search and Linkup standard
  fallback.
- `mcp__search-fetch__web_fetch` fetches one page,
  routing mapped GitHub URLs through the local `gh` CLI,
  then using DeepSeek fetch,
  Linkup renderJs,
  and Exa contents.

Truncated output carries the model-visible head plus a
`search-fetch://response/<id>` resource URI,
which a host reads with `resources/read`.

## Configuration

The bundle declares no plugin config.
The inserted row is `@deepseek-ai/dsh-mcp-client` with `transport: stdio`,
`serverName: search-fetch`,
and `failOnStartupError: true`,
so a server that cannot start surfaces as an activation diagnostic instead of
silently missing tools.

`command` is `!!js process.execPath`,
the host's own Node,
so the spawn does not depend on `PATH` resolution.
The single `args` entry resolves the built server entry point through this
bundle's own `package.json` with `createRequire(baseUrl)`,
then walks to `../../mcp/search-fetch/dist/final/node/index.mjs`.
That keeps the wiring relative to the checkout instead of hardcoding a
workspace location or a username.

## Credentials

No credential is copied into this bundle.
The credential source is the repo's sops/age-encrypted `.env.local.json`,
read through `@monochromatic-dev/env-local-json-core` inside the shared
search-fetch core.
That store supplies `DEEPSEEK_API_KEY`,
which is why the MCP search works
without a harness-level DeepSeek credential.
The pi config file at `$HOME/.pi/agent/extensions/pi-search-fetch.json` now
holds only the blocklist;
a config still carrying `exaApiKey` or `linkupApiKey` raises
`SearchFetchMigrationRequiredError` naming `mise run secrets:migrate`.

The store is found by an explicit path first.
The host exports `ENV_LOCAL_JSON_PATH=/var/home/user/Monochromatic/.env.local.json`
in `~/.bashrc`,
so a harness started from that shell passes the path down and the child does
not depend on its working directory.
When the variable is absent,
the reader falls back to `.env.local.json` at the nearest repository root,
then to a bounded walk from the child's working directory.
No env value is pinned in this bundle.

The harness subprocess scrub in `@deepseek-ai/dsh-subprocess` drops names
matching `KEY|PASSWORD|SECRET|TOKEN` and names beginning with `DSH_`.
That drops `SOPS_AGE_KEY_FILE` and the other environment identity names even
when the host has them,
so only the final fallback in the identity chain is reachable:
the mise default `~/.config/mise/age.txt`.
`HOME`,
`PATH`,
and `ENV_LOCAL_JSON_PATH` survive the scrub,
and `PATH` surviving is what makes both the `sops` call and the `gh` CLI route
work.

When the store is unavailable,
the server still starts with the blocklist and
GitHub-routed `web_fetch` through `gh`.
Only calls that need a missing provider key fail,
with a value-free store diagnostic in the error text.

`loadLinkupConfig()` runs once at startup,
before `serve()`,
so a store change while the server is already running is not picked up.
Respawn it by toggling the `include:search-fetch-mcp` row through
`plugin_manager`,
or restart the harness.

## Prerequisites

The server must be built,
because the row points at `dist/final/node/index.mjs`:

```sh
# from the repository root
mise run //package/search-fetch/mcp:build
```

## Install

Install through `plugin_manager` with `action: install_bundle` and this
directory as `target`.
Do not hand-write the profile's `package.json` or `cordis.patch.yml`;
`install_bundle` performs those steps.

Verify by calling `mcp__search-fetch__web_search` in a new session.
Existing sessions keep the plugin revision they started with,
so a freshly installed bundle is visible only to sessions created afterwards.

## Relationship to the native web tools

The `cordis` agent preset mounts `@deepseek-ai/dsh-tool-web`,
whose config carries independent `search` and `fetch` booleans.
The native search provider authenticates from DSH's own credentials file,
which has no DeepSeek key in this profile,
so `search: false` drops the native `web_search` rather than leaving a tool
that can only fail,
and `fetch: true` keeps the native anonymous `web_fetch`.
This bundle's MCP server reads `DEEPSEEK_API_KEY` from the repo store instead,
so its search and fetch work without re-enabling the native provider.

That leaves two fetch tools with different policies:
the native one accepts any public HTTP(S) destination,
while this bundle's enforces the repo blocklist and routes GitHub URLs through
`gh`.
Which one an agent should prefer is a deployment choice,
recorded in `doc/audit/dsh-setup-gaps-2026-10-17.md`.
