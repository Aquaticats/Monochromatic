# OpenCodeReview v1.13.3 MCP registration: the documented docs URL returns 404 and the bare command name does not resolve, so hosts must register the built artifact path

## Symptom

Registering our `search-fetch` MCP server with OpenCodeReview hit two dead ends
when following `package/search-fetch/mcp/README.md` as it stood before 2026-10-31:

- The docs link `https://open-codereview.ai/docs/mcp` returns HTTP 404
 (`curl -s -o /dev/null -w '%{http_code}\n' https://open-codereview.ai/docs/mcp`
 prints `404`),
 as does `https://open-codereview.ai/en/mcp`.
- Registering the bare command name fails,
 because nothing in this workspace depends on
 `@monochromatic-dev/search-fetch-mcp`:
 `command -v search-fetch-mcp` exits 1
 while the positive control `command -v mvm` resolves
 through `node_modules/.bin`.

## Root cause

Two independent causes,
one per symptom.

The upstream documentation exists in source at
`pages/src/content/docs/en/mcp.md`
([commit-pinned source][src-docs]),
and its own examples use the same field shape our README now documents:

```text
# Minimal: just a command
ocr config set mcp_servers.docs.command npx

ocr config set mcp_servers.docs.args '["-y", "@acme/docs-mcp-server"]'

# Restrict which tools are exposed to the reviewer
ocr config set mcp_servers.docs.tools '["search_docs", "get_page"]'
```

with the field table in the same file recording `command` (string,
for
`stdio`) and `args` (string array,
arguments passed to `command`).
The deployed site does not serve those pages under the URL our README cited
(both routes probed 404 on 2026-10-31),
so the repository file,
not the site URL,
is the usable source of truth.

The bare-name failure is ours,
not upstream's:
 a workspace bin name resolves only where some `node_modules/.bin` on `PATH`
links the package,
 and no package in this workspace depends on the MCP server package.

[src-docs]: https://github.com/alibaba/open-code-review/blob/216dcf513593055a70165f793e96b52589d71b1b/pages/src/content/docs/en/mcp.md#L31-L42

The link target was confirmed against that commit through the GitHub
contents API.
This harness has no `agent-browser`,
so the selection check ran against the API response rather than a browser
render.

## Verification

Version under test:
`open-code-review v1.13.3 (375f64c) linux/amd64`,
 built 2026-10-22,
 from `https://github.com/alibaba/open-code-review`.

Runnable harness:

```bash
# Failing catalog: the cited docs route and the bare command name
curl -s -o /dev/null -w '%{http_code}\n' https://open-codereview.ai/docs/mcp
command -v search-fetch-mcp
command -v mvm   # positive control: workspace bins do resolve here

# Working catalog: artifact-path registration through ocr itself
ocr config set mcp_servers.search-fetch.command node
ocr config set mcp_servers.search-fetch.args \
  '["/absolute/path/to/package/search-fetch/mcp/dist/final/node/index.mjs"]'
```

Failing catalog:
 the docs route prints `404`,
 `command -v search-fetch-mcp` exits 1.
Working catalog:
 `ocr config set` accepts the `command` plus `args` form,
 echoes the parsed array back,
 and writes it into `~/.opencodereview/config.json` in the same shape the
 upstream field table documents.
A live OpenCodeReview review run against the registered server was not
 executed;
 the verification covers registration only.

## Verified workarounds

- Register `command: node` with `args` carrying the absolute built-artifact
  path,
  exactly as upstream's own `args` example does.
  Tradeoffs:
   the absolute path breaks when the checkout moves or the artifact is
  rebuilt elsewhere,
   and each host config must be updated when the package path changes
  (this is what the 2026-10-31 product-category move required).
- Keep the `tools` allowlist
 (`["web_search", "web_fetch"]`) so only intended tools mount.
  Tradeoffs:
   new tools added later stay invisible until the allowlist grows.

## What does not work

- The bare command name `search-fetch-mcp` without a linking
  `node_modules/.bin` on `PATH`:
   exits 1 with no diagnostics.
- The site docs URL:
   404,
   so it cannot be cited as field documentation.
- `gh search code` on upstream for `mcp_servers` found the docs pages but no
  separate schema-validation path for config keys;
  this doc does not claim one exists or does not exist.

## Upstream filing artifact

`.out-of-scope/` checked 2026-10-31:
 none of the 11 exemption files covers OpenCodeReview or docs-site routing,
 so no exemption applies.

Duplicate search 2026-10-31:
 `gh search issues --repo alibaba/open-code-review "mcp"` returned ten issues,
 a positive control proving the search runs.
 None reports the docs-route 404.
 The nearest near-miss is closed #1441
 (`docs(contributing): Project Structure omits major packages`),
 a docs-completeness report rather than a routing failure.
 Open #1825 covers dotted server names in `ocr config set`,
 which does not affect the hyphenated `search-fetch` name used here.
 No in-flight fix search was run because no filing is planned.

### Upstream filing decision

- Is it upstream's fault?
   Yes for the docs route:
   their pages exist in source while the deployed site 404s.
- Can upstream fix it?
   Yes:
  routing or deployment of the Starlight pages.
- Are they supporting this use case?
   Yes:
   MCP registration with `command`,
  `args`,
  `env`,
  `tools` is documented in
  their own `mcp.md`.
- Would the repo welcome our contribution?
   Not checked (their contribution and AI-assistance policy unread);
   recorded as not run.
- Will they likely fix it?
   Not checked beyond the duplicate search;
   recorded as not run.
- Prototype?
   Not reached:
   constraints 4 and 5 were not run,
   and a docs-site route change is a deployment fix rather than a patch this
  repo can verify locally.

Outcome:
**do not file as-is**,
and no recommendation is made,
 because the policy checks were not run.
 Filing is the poster's decision,
 and any filing puts a real person's account in front of strangers;
 trust and blocklist systems exist,
 a maintainer can denounce a poster,
 and that can follow the person across projects.
 The workaround above fully unblocks this repo,
 so nothing here waits on upstream.

Draft,
kept for auditability and marked do-not-file:

~~~md
Title: docs: `https://open-codereview.ai/docs/mcp` returns 404 while `pages/src/content/docs/en/mcp.md` exists in source

Body: Following the documented MCP docs URL on 2026-10-31 returns HTTP 404
(curl harness), as does /en/mcp. The page exists at
pages/src/content/docs/en/mcp.md on main (commit 216dcf51), whose command and
args examples are what users need for mcp_servers registration.

Suggested fix: serve the Starlight docs routes on the deployed site, or
update any documentation that cites the /docs/mcp URL to the route the site
actually serves.
~~~
