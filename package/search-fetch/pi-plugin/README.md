# Pi Search Fetch

`@monochromatic-dev/search-fetch-pi-plugin` is a narrow Pi package for web search and page fetch.
It registers only two model-callable tools:

- `web_search`
- `web_fetch`

The package deliberately does not provide web-answer,
account-management,
or per-call search-depth controls.

## Configuration

The extension reads one optional global config file:

```json
{
  "blocklist": ["badwikipedia.invalid"]
}
```

Config path:

```text
~/.pi/agent/extensions/pi-search-fetch.json
```

The file holds only the blocklist.
Provider credentials live in the process environment or in the repo's
encrypted store,
never in this file.

The blocklist is global and is applied locally after search results,
before fetch attempts,
and to provider-supported request filters where compatible.

### Credentials

`search-fetch` resolves each provider key independently.
A non-blank `DEEPSEEK_API_KEY`,
`EXA_API_KEY`,
or `LINKUP_API_KEY` in the process environment wins.
Otherwise the key comes from the repo's sops/age-encrypted `.env.local.json`
through `@monochromatic-dev/env-local-json-core`,
the same reader the MCP
server and the dsh bundle use.

The reader locates the store in this order:

1. an explicit path,
   from `ENV_LOCAL_JSON_PATH` or a caller-supplied path,
2. `.env.local.json` at the nearest repository root,
3. a bounded walk from the working directory,
   stopping at the home directory,
   the containing repository,
   or the filesystem root.

Decryption needs the `sops` executable on `PATH`.
The identity chain mirrors mise:
`MISE_SOPS_AGE_KEY`,
`MISE_SOPS_AGE_KEY_FILE`,
`SOPS_AGE_KEY_FILE`,
`SOPS_AGE_KEY`,
then `age.txt` in the mise configuration directory
(`${MISE_CONFIG_DIR:-${XDG_CONFIG_HOME:-$HOME/.config}/mise}/age.txt`).

The reader returns only requested keys that the environment did not supply,
and it never mutates `process.env`.
When the store is unavailable,
the extension still loads the blocklist and
GitHub-routed `web_fetch` through `gh`;
only calls that need a missing provider key fail,
with the existing credential message plus a value-free store diagnostic.

The old `exaApiKey` and `linkupApiKey` fields are no longer read.
A config still carrying them,
or a legacy `pi-linkup.json`,
raises
`SearchFetchMigrationRequiredError` naming `mise run secrets:migrate`.
That migration moves both values into `.env.local.json`,
verifies the store by
decrypting it,
then removes the old fields and the legacy file.

## Tool behavior

`web_search` searches with DeepSeek first when DeepSeek credentials are configured.
It uses Exa next and Linkup last when earlier credentials are missing or the
earlier provider fails.
DeepSeek search uses fixed behavior:

- maximum result count (`count: 50`)
- page content requested (`need_content: true`)
- `includeDomains`,
  `fromDate`,
  and `toDate` mapped to `site:`,
  `after:`,
  and `before:` operators while they fit the 128 character query limit
- dropped filters enforced again on the returned results
- results deduplicated by URL host plus path

Exa search uses fixed behavior:

- `type: "fast"`
- `numResults: 10`
- `excludeDomains` set to provider-compatible configured blocklist entries

Linkup fallback search uses fixed behavior:

- `depth: "standard"`
- `outputType: "searchResults"`
- `excludeDomains` set to the normalized global blocklist

Legacy or unsupported keys such as `depth`,
`limit`,
`maxResults`,
`excludeDomains`,
`includeImages`,
and `outputType` are ignored.
The tool returns a warning text item before the response when that happens.

When a search response is exactly `{ "results": [...] }`,
or has exactly the top-level keys `requestId`,
`resolvedSearchType`,
`results`,
`searchTime`,
and `costDollars`,
and every result is an object,
the tool returns the inner results array as JSONL,
one result object per line.
Other search response shapes are returned as JSON.

`web_fetch` routes GitHub URLs through the local `gh` CLI before any paid provider.
Every other URL fetches through DeepSeek first,
then Linkup renderJs,
then Exa
contents when earlier credentials are missing or the earlier provider fails.
DeepSeek fetch failures,
empty page content,
timeouts,
and HTTP errors all fall
through to the next provider.

GitHub hosts carrying a `gh` mapping:

- `github.com` and `www.github.com`
- `gist.github.com`
- `api.github.com`
- `raw.githubusercontent.com`

Mapped GitHub URL shapes and the `gh` work each one uses:

- repository home page:
   `gh repo view` returning repository metadata and the README
- `blob`/`raw`/`blame` file URLs:
   `gh api` contents read with the raw media type
- `tree` directory URLs:
   `gh api` contents read projected to tab-separated `type`/`name`/`size` lines
- numbered `issues` URLs:
   `gh issue view` for the body plus `gh issue view --comments` for the thread
- numbered `pull` URLs:
   `gh pr view` for the body plus `gh pr view --comments` for the thread
- `pull` changed-file URLs:
   `gh pr diff`
- `commit` URLs:
   `gh api` with the diff media type
- `compare` URLs:
   `gh api` with the diff media type
- `releases` URLs:
   `gh release list`
- `releases/tag` URLs:
   `gh release view` with an explicit `--repo`
- gist URLs:
   `gh gist view`
- `api.github.com` URLs:
   `gh api` with the same path and query

GitHub shapes with no `gh` mapping fall through to the paid provider chain:

- owner and organization pages
- `actions` runs
- `wiki` pages
- `discussions`
- `projects`
- `pulse` and other graph pages
- release asset downloads

One git reference may itself contain slashes,
 so blob and tree URLs are ambiguous.
The planner builds one ordered attempt per split point with the shortest reference first,
encodes the decoded reference as one whole `ref` query value,
and the first attempt whose `gh` call exits zero answers.
A failure other than gh's ordinary exit code 1 ends the sequence instead of trying another split,
so an environmental failure costs one attempt rather than one per split point.

Each `gh` invocation runs in a temporary working directory so no ambient repository context reaches it.
One invocation carries a fixed 60-second deadline and a 32 MiB captured output ceiling.
The child environment pins `GH_HOST` to `github.com`,
 `GH_FORCE_TTY` to empty,
 `CLICOLOR_FORCE` to `0`,
and `NO_COLOR` to `1`,
so ambient values cannot redirect a fetch or switch gh to colored terminal-shaped output.
Pull request diffs also pass `--color never`.

These outcomes all fall back to DeepSeek,
then Linkup,
then Exa:

- missing `gh` executable
- unauthenticated `gh`
- exceeded deadline
- exceeded output ceiling
- non-zero `gh` exit
- unmapped GitHub shape

Tool-result details record every fallback step as `fallbackChain`.
A cancelled tool call rethrows before every provider hop instead of continuing to the next provider.

`gh` output returns in the same single-field `{ "markdown": ... }` shape Linkup uses,
 so the model sees raw text.
Output that is not renderable text returns a one-line notice naming the captured byte count instead of
replacement characters.
Font and image blobs take that path,
as does any payload carrying a NUL byte.
A failed optional comment read leaves its own one-line notice after the thread body,
so a partial thread is visible rather than silent.
Whitespace-only file content is preserved exactly.

Two consequences follow from routing through an authenticated `gh`:

- GitHub content that requires authentication is now readable,
   up to the scopes of the account `gh` is authenticated as.
   The global blocklist still applies before any `gh` invocation.
- Every mapped read spends the authenticated account's GitHub API budget instead of paid search credits.
   A session that fetches many GitHub files can reach a rate limit,
   which surfaces as a non-zero `gh` exit and then falls back to the paid providers.

DeepSeek fetch uses fixed behavior:

- `POST https://api.deepseek.com/v1/tools/fetch` with the requested URL
- raw Markdown page content normalized to the same single-field `{ "markdown": ... }` shape Linkup uses
- 30 second client timeout

Linkup fetch uses fixed behavior:

- `renderJs: true`
- `extractImages: false`
- `includeRawHtml: false`

Blocked fetch hosts throw before any `gh` invocation or provider network request is made.

After fetching,
the extension removes Markdown inline images backed by base64 `data:image/` URLs from the model-visible response.
Line-isolated images remove their complete physical lines,
while images sharing a line with prose remove only the image construct.
The untouched provider response remains available in tool-result details.

Responses that are exactly a single `markdown` string field are returned as raw markdown text,
for example `{ "markdown": "# Meow" }` becomes `# Meow`.
Other fetch response shapes are returned as JSON.

Both tools cap model-visible response text at 100KB or 2000 lines,
whichever is hit first.
When truncation happens,
the full response is written to a temporary file and the tool result names that path.

## Verification

Run package-scoped tasks:

```bash
mise run //package/search-fetch/pi-plugin:build
mise run //package/search-fetch/pi-plugin:lint:types
mise run //package/search-fetch/pi-plugin:lint:oxlint
mise run //package/search-fetch/pi-plugin:test:unit
mise run //package/search-fetch/pi-plugin:verify:extension
```
