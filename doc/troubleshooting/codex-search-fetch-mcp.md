# Codex 0.163.0 needs protocol opt-in for the search/fetch MCP server

## Symptom

With the repository's search/fetch server registered but Codex's modern MCP feature disabled,
Codex reports `MCP startup failed: handshaking with MCP server failed` and JSON-RPC error `-32601`:

```text
Method not found: initialize. This server implements MCP revision 2026-07-28,
which removed the initialize handshake: call server/discover and declare the
revision in each request's params._meta instead
```

This was reproduced with a temporary client override on October 17, 2026.
The saved global configuration remains enabled and working.

## Root cause

The repository implements revision `2026-07-28` exclusively.
`package/mcp/stdio/src/server-response.ts:188` returns the diagnostic for an older client's handshake:

```ts
// package/mcp/stdio/src/server-response.ts
code: JSON_RPC_METHOD_NOT_FOUND,
message:
  `Method not found: initialize. This server implements MCP revision `
  + `${SUPPORTED_PROTOCOL_VERSIONS.join(', ')}, which removed the initialize handshake: `
    + `call server/discover and declare the revision in each request's params._meta instead`,
```

The shallow `openai/codex` source checkout was inspected at
`ce0323f2f2d3b25e82806cd555237da3db4e1897`.
Its `codex-rs/rmcp-client/src/protocol_mode.rs:36` selects modern stdio only when both inputs opt in:

```rust
// openai/codex: codex-rs/rmcp-client/src/protocol_mode.rs
match (self, requested_version) {
    (Self::Legacy, _) => Ok(Self::Legacy),
    (_, None) => Ok(Self::Legacy),
    (Self::V20260728, Some(version)) if version == OsStr::new("2026-07-28") => {
        Ok(Self::V20260728)
    }
```

The inputs are the Codex feature `mcp_2026_07_28` and this server's
`CODEX_MCP_PROTOCOL_VERSION=2026-07-28` environment entry.
The source checkout is an audit reference;
the installed binary was separately verified as `codex-cli 0.163.0`.

## Verified configuration

The user's global Codex configuration now registers `search_fetch`.
The setup used Codex's own `features enable` and `mcp add` commands,
with current-user paths derived from the runtime home directory.

The registered command is the user's `.local/bin/mise`,
with arguments selecting this repository through `--cd`,
then `exec -- node package/search-fetch/mcp/dist/final/node/index.mjs`,
using an absolute artifact path.
This selects the repository's Node runtime independently of the caller's working directory.
The server environment includes:

```toml
# Global Codex config.toml
[mcp_servers.search_fetch.env]
CODEX_MCP_PROTOCOL_VERSION = "2026-07-28"
MISE_AUTO_INSTALL = "false"
```

`MISE_AUTO_INSTALL=false` prevents this server launch from installing missing repository tools.
The tradeoff is dependence on this checkout,
its built server artifact,
and its already installed runtime.
An updated source server requires rebuilding that artifact.
Codex currently labels the modern MCP feature as under development.

Existing configuration was compared with a private backup.
The CLI reordered MCP environment entries,
omitted an empty default argument list,
and normalized `120` to `120.0` for an existing startup timeout.
The comparison allowed those equivalent representations and confirmed the other settings were preserved.

## Verification

The runnable local harness is
`${HOME}/temp/agent/codex-search-fetch-20261017/mise.toml`.
It starts a fresh Codex app-server,
uses the real MCP status and tool-call APIs,
and creates an ephemeral thread without starting a model turn.

The passing cases were:

- Temporary configuration:
  discovery returned `web_search` and `web_fetch`.
- Temporary configuration:
  both tools completed real provider calls.
- Saved global configuration from outside the repository:
  discovery and both provider calls passed.
- Search returned documentation URLs from `developers.openai.com`.
- Fetch returned the actual MCP documentation content.

The failing control disabled only `mcp_2026_07_28` for a temporary app-server process.
Discovery then returned the handshake error quoted in the symptom.
This control did not alter the saved configuration.

Private transcripts,
the registration snapshot,
and verification summaries are retained beside the harness.
Neither passing fetch response was truncated,
so the resource-read continuation path was not exercised.
This verifies fresh Codex clients;
it does not establish that an already running conversation refreshes its tool catalog.

## What does not work

Registering the command and its protocol marker while leaving the Codex feature disabled
reproduces the legacy `initialize` error.
The marker alone is insufficient.

## Upstream filing decision

The maintenance statement in `.out-of-scope/codex-harness.md` is not a prohibition on this work
or an automatic exemption from investigating a Codex issue.
The user clarified that meaning on October 17, 2026.

- Upstream fault:
  no demonstrated defect remains;
  the required opt-in configuration works.
- Fixability:
  an upstream code change is unnecessary for the verified configuration.
- Supported use case:
  the audited source explicitly supports this stdio protocol combination.
- Contribution policy:
  not assessed because no upstream contribution is proposed.
- Upstream movement:
  an issue search for `mcp_2026_07_28 stdio` found other MCP reports,
  but this investigation has no unresolved defect to report.
- Prototype:
  configuration controls reproduced failure and success without an upstream patch.

There is nothing to file for this configuration finding.
The official configuration reference is [MCP configuration](https://developers.openai.com/codex/mcp).
