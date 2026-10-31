# Search fetch migration handover

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/open-code-review-issue` is now `package/open-code-review-issue/cli`.

> Moved 2026-10-31 by the search-fetch product move into `package/search-fetch`
> (`doc/planning/search-fetch-cli.md`);
> paths and names below this note are historical:
> `package/pi-plugin/search-fetch` is now `package/search-fetch/pi-plugin`;
> `@monochromatic-dev/pi-plugin-search-fetch` is now
> `@monochromatic-dev/search-fetch-pi-plugin`.

## Current goal

Change the current Pi Linkup package into an Exa-first search and fetch extension:

- Use Exa whenever available.
- Fall back to Linkup when Exa is missing or fails.
- Rename the extension to `pi-search-fetch`.
- Rewire Pi to use the renamed extension.
- Keep this package handover current while the work proceeds.

Historical note:
provider order has since changed to DeepSeek first,
then Exa,
then Linkup for
search,
and `gh`,
DeepSeek,
Linkup,
then Exa for fetch.
See [`doc/decision/search-fetch-deepseek-primary.md`](../../../doc/decision/search-fetch-deepseek-primary.md).

## Required skills for the next agent

Use these when the matching work starts:

- `testing-practices`,
   when changing or reviewing package tests.
- `troubleshooting-doc`,
   when documenting provider quirks or externally verified behavior.
- `grill-me`,
   if another design branch needs user decisions.

## User decisions already made

- Public tool names become `web_search` and `web_fetch`.
- Do not keep `linkup_web_search` or `linkup_web_fetch` aliases.
- Exa unavailability means missing Exa credentials or any Exa failure.
- Fallback to Linkup must be logged and covered by tests.
- Config should use a new canonical config file only.
- Migrate the existing old config file instead of keeping `pi-linkup.json` as a fallback path.
- `web_search` should use Exa `fast` by default.
- `web_fetch` should be Linkup-first,
   despite `web_search` being Exa-first.

## Repository state at latest update

- Package path has moved to `package/pi-plugin/search-fetch/`.
- Package metadata now names `@monochromatic-dev/pi-plugin-search-fetch`.
- Public tool implementation now registers `web_search` and `web_fetch` only.
- Global config loader now uses `pi-search-fetch.json` and migrates legacy `pi-linkup.json`.
- Provider implementation is in place:
  Exa-first `web_search`,
   Linkup-first `web_fetch`,
   and fallback metadata in tool details.
- Active Pi settings now point to `/var/home/user/Monochromatic/package/pi-plugin/search-fetch`.
- Active Pi config migrated to `~/.pi/agent/extensions/pi-search-fetch.json`.
- Legacy active Pi config `~/.pi/agent/extensions/pi-linkup.json` was removed after migration.
- Existing unrelated worktree change:
   `mise.lock`.
   Do not touch unless the task requires it.

Relevant prior artifacts:

- `doc/troubleshooting/linkup-grokipedia-results.md` records the Linkup pollution diagnosis,
  the Linkup `fast` comparison,
   and the initial Exa comparison.
- Commits already made for the troubleshooting doc:
  - `f0d91077b`
  - `f66b8863a`
  - `9ca3df6c2`

## Exa search mode benchmark so far

A broader Exa `/search` `auto` versus `fast` benchmark was run across 18 documentation and debugging queries.
The query set included KDE failure cases,
 Bazzite docs,
 Node,
 TypeScript,
 Rust,
 MDN,
 Playwright,
 Vite,
pnpm,
 mise,
 TypeBox,
 GitHub CLI,
 systemd,
 and Kirigami.

Measured fields:

- first expected-source rank
- expected-source count in top ten results
- Grokipedia pollution count
- latency
- simple rank score

Aggregate result after correcting expected hosts for `mise.en.dev` and GitHub-hosted Vite docs:

- `auto`:
   rank score 72,
   expected-source hits 104,
   misses 0,
   Grokipedia hits 0,
  cache-warmed sampled latency 1815 ms,
   wins 1 case.
- `fast`:
   rank score 72,
   expected-source hits 109,
   misses 0,
   Grokipedia hits 0,
  cache-warmed sampled latency 1665 ms,
   wins 2 cases.

Interpretation:

- The corrected proxy benchmark does not show a quality penalty for `fast`.
- `fast` had more expected-source hits in the corrected sample.
- Both modes had no misses and no Grokipedia pollution.
- `auto` remains Exa's documented recommended mode for most applications.
- The benchmark is still a proxy,
   not a formal IR benchmark.
- Decision:
   use Exa `fast` as the default `web_search` mode.
  The corrected benchmark showed no measured quality penalty for `fast`.

Notable benchmark cases:

- `mise.en.dev` is live mise documentation and should count as an expected source.
- GitHub-hosted Vite docs are source documentation and should not be treated as a miss by default.
- `playwright-locators`:
   `fast` had more expected-host hits than `auto`.
- `typebox-object`:
   `fast` had more expected-host hits than `auto`.

## Implementation notes

Package rename work:

- Done:
   move `package/pi-plugin/linkup/` to `package/pi-plugin/search-fetch/`.
- Done:
   rename package metadata to `@monochromatic-dev/pi-plugin-search-fetch`.
- Done:
   update `USER_AGENT_VALUE`,
   logger tags,
   temp-file prefixes,
   tests,
   and exported docs.
- Done:
   update generated root tooling references through file-enforcer instead of hand-editing `mise.toml`.
- Done:
   update workspace lockfile importer path only.

Config work:

- Done:
   canonical config path is `~/.pi/agent/extensions/pi-search-fetch.json`.
- Done:
   legacy `pi-linkup.json` migrates once into the new file.
- Done:
   runtime does not keep using the old config path after migration.
- Done:
   config keys are `exaApiKey`,
   `linkupApiKey`,
   and `blocklist`.
- Done:
   `EXA_API_KEY` wins over config `exaApiKey`.
- Done:
   `LINKUP_API_KEY` wins over config `linkupApiKey`.
- Done:
   legacy `apiKey` migrates to `linkupApiKey`.
- Do not write any API key into this handover or other docs.

Blocklist caveat:

- Exa rejects the current bare `gov` blocklist entry when sent through `excludeDomains`.
- Keep local post-response filtering for all normalized blocklist entries.
- Send only API-compatible domain entries to Exa `excludeDomains`.
- Linkup can continue receiving the existing blocklist if its API accepts it,
  but local filtering should remain the enforcement layer.

Search behavior:

- `web_search` should try Exa first when Exa credentials are configured.
- On missing Exa credentials or any Exa failure,
   log the fallback reason and try Linkup.
- If both providers are unavailable,
   throw a clear error naming both missing or failed providers.
- Preserve existing supported parameters where practical:
  - `query`
  - `fromDate`
  - `toDate`
  - `includeDomains`
- Map date fields to Exa published-date fields unless docs or tests show another field is better.
- Keep result-count controls fixed unless the user explicitly asks for them.

Fetch behavior decision:

- `web_fetch` should be Linkup-first.
- Exa `/contents` supports URL content extraction with `urls` and `text: true`.
- Linkup `/fetch` currently uses `renderJs=true`,
   `extractImages=false`,
   and `includeRawHtml=false`.
- Exa docs do not expose an equivalent `renderJs=true` knob in the fetched docs.
- The user chose Linkup-first fetch to preserve current rendering semantics.
- If Linkup fetch is unavailable and Exa credentials exist,
   fallback to Exa `/contents` is still plausible,
  but confirm or test before implementing that fallback branch.

Output behavior:

- Rename `LinkupToolDetails` and related output helpers to provider-neutral names.
- Include the provider used in tool details.
- Include fallback metadata when Exa fails and Linkup succeeds.
- Keep current truncation behavior and JSONL rendering for exact `{ "results": [...] }` payloads
  unless tests justify changing it.

## Tests to update or add

Update existing unit tests under the package after the rename.
Add coverage for these branches:

- config loads new file name and rejects unknown keys
- old config migration writes or produces the new config shape
- Exa key comes from `EXA_API_KEY`
- Linkup key comes from `LINKUP_API_KEY`
- Exa search request body shape
- Exa search filters API-forwardable blocklist entries for `excludeDomains`
- local blocklist removes blocked result URLs from Exa and Linkup responses
- Exa success does not call Linkup
- missing Exa key falls back to Linkup
- Exa HTTP failure falls back to Linkup and records/logs the reason
- Exa network failure falls back to Linkup and records/logs the reason
- both providers unavailable throws a clear error
- tools register only `web_search` and `web_fetch`
- old `linkup_web_*` names are absent
- extension verification expects the new names

## Verification commands

After implementation,
 run package-scoped tasks through mise,
 not raw tools.
Use the renamed package path once the directory moves:

```sh
mise run //package/pi-plugin/search-fetch:build
mise run //package/pi-plugin/search-fetch:lint:types
mise run //package/pi-plugin/search-fetch:lint:oxlint
mise run //package/pi-plugin/search-fetch:test:unit
mise run //package/pi-plugin/search-fetch:verify:extension
```

## Pi rewiring notes

After build and tests pass:

- Inspect the active Pi package or extension configuration before editing it.
- Replace references to the old package or path with the renamed package.
- Ensure duplicate old and new tools are not both registered.
- Migrate the global config file to `pi-search-fetch.json`.
- Verify a real Pi extension load registers `web_search` and `web_fetch`.

## Final verification in this session

Commands passed:

```sh
mise run //package/pi-plugin/search-fetch:build
mise run //package/pi-plugin/search-fetch:lint:types
mise run //package/pi-plugin/search-fetch:lint:oxlint
mise run //package/pi-plugin/search-fetch:test:unit
mise run //package/pi-plugin/search-fetch:verify:extension
```

Boundary checks passed through the built Pi extension interface with the real migrated config:

- `web_search` registered and returned `provider: "exa"`.
- `web_fetch` registered and returned `provider: "linkup"` with normal environment.
- `web_fetch` returned `provider: "exa"` with fallback metadata when `LINKUP_API_KEY` was cleared for the process.

## GitHub URL routing through the gh CLI, 2026-09-26

User request:
when `web_fetch` receives a URL on GitHub,
use `gh` as the underlying provider instead of the paid search providers.

### Decisions

- Mapped hosts are `github.com`,
   `www.github.com`,
   `gist.github.com`,
   `api.github.com`,
   and `raw.githubusercontent.com`.
   Every other host,
   including `camo.githubusercontent.com`,
   `codeload.github.com`,
   and GitHub Enterprise Server hosts,
   keeps the existing paid provider chain.
- Mapped shapes are repository home,
   `blob`/`raw`/`blame` files,
   `tree` directories,
   numbered issues,
   numbered pull requests,
   pull request changed files,
   single commits,
   comparisons,
   release index,
   release tag,
   gists,
   and `api.github.com` endpoints.
- Deliberately unmapped shapes fall through to Linkup and then Exa:
   owner and organization pages,
   `actions`,
   `wiki`,
   `discussions`,
   `projects`,
   `pulse`,
   commit lists,
   and release asset downloads.
   Reason:
   no `gh` surface renders them faithfully,
   or the output volume is unbounded.
- Blob and tree URLs plan one attempt per reference and path split point,
   shortest reference first.
   Reason:
   a git reference may contain slashes.
   Measured evidence on `cli/cli` branch `8761/allow-multiple-items-in-nested-array`:
   the short split returns `gh: No commit found for the ref 8761 (HTTP 404)` and exits 1,
   while the long split returns the file.
- Issue and pull request threads plan two invocations:
   a required body read and an optional comment read,
   run concurrently.
   Reason:
   in piped mode `--comments` prints only comments and never the title or body.
   See `doc/troubleshooting/gh-view-comments-raw-output.md`,
   which also records upstream calling that behavior intended and recommending exactly this split.
   Measured on this host with three samples per mode on one unchanged build:
   sequential 2869 ms,
   3024 ms,
   2850 ms;
   concurrent 1846 ms,
   1571 ms,
   1630 ms.
- Release pages pass the tag and `--repo` separately.
   Reason:
   `gh release view` treats its positional argument as a tag and then resolves the repository
   from the working directory,
   so a URL argument fails outside a work tree with a Git diagnostic.
   See `doc/troubleshooting/gh-implicit-repository-git-wrapper.md`.
- Every gh child runs in `os.tmpdir()` with a fixed 60 second deadline and a 32 MiB output ceiling.
   Reason:
   a temporary directory keeps ambient repository context away from repository resolution,
   and the deadline matches the bound this repository already gives GitHub CLI work in
   `package/cli/open-code-review-issue`.
- The gh response always uses the single-field `{ "markdown": ... }` shape so the model sees raw text
   through the existing markdown-only rendering path.
- Output that is not valid UTF-8 becomes a one-line notice naming the captured byte count.
   Reason:
   measured on a real 76424 byte `inter.woff2`,
   UTF-8 decoding produces replacement characters and would fill the model context with noise.
   Detection is a byte round trip:
   re-encode the decoded text and compare with the captured bytes.
- Provider fallback details changed from a single `fallback` object to a `fallbackChain` array.
   Reason:
   three providers can now chain,
   and one from and to pair cannot state which hop failed when gh and Linkup both fail before Exa
   answers.
- A cancelled tool call rethrows instead of falling back.
   Reason:
   user cancellation must not silently turn into a paid provider fetch.

### Files

- `src/github-url-plan-constants.ts`,
   hosts,
   URL sections,
   and gh argument fragments.
- `src/github-fetch-types.ts`,
   plan,
   invocation,
   attempt,
   runner,
   and client types.
- `src/github-url-plan.ts`,
   the public planner entry point.
- `src/github-url-validation.ts`,
   URL parsing,
   host normalization,
   segment extraction,
   and argument validation.
- `src/github-section-plan.ts`,
   host and repository section dispatch.
- `src/github-content-plan.ts`,
   file,
   directory,
   and raw-host plans plus split candidates.
- `src/github-record-plan.ts`,
   issue,
   pull request,
   commit,
   comparison,
   and release plans.
- `src/gh-invocation-plan.ts`,
   attempt and argument builders.
- `src/gh-process.ts`,
   the child process boundary and its failure classification.
- `src/gh-client.ts`,
   attempt execution,
   output joining,
   and `GhFetchError`.

### Verification

Package tasks passed:
`build`,
`lint:types`,
`lint:oxlint` with zero warnings and zero errors,
`test:unit`,
and `verify:extension`.

End-to-end run through the built artifact and the model-facing `web_fetch` tool,
with the real migrated config and real gh children:

- 20 mapped GitHub URLs returned `provider: "gh"` with correct content,
   including a nested blob path,
   a multi-segment reference blob that needed the second split,
   a raw host file,
   a tree listing,
   a repository root listing,
   an issue thread,
   an issue URL that is really a pull request,
   a pull request thread,
   pull request changed files,
   a commit diff,
   a comparison diff,
   a release index,
   release notes,
   a gist,
   an `api.github.com` endpoint,
   a repository file in this workspace,
   and a binary blob that returned the notice.
- The comparison diff produced 692401 bytes,
   truncated to 92999 visible bytes,
   with the full text written to a temporary response file.
- Unmapped GitHub shapes for `actions` and an owner page returned `provider: "linkup"`.
- A non-GitHub URL returned `provider: "linkup"`.
- With `PATH` pointed at a nonexistent directory,
   the same repository URL returned `provider: "linkup"` with a `fallbackChain` step naming gh and the
   reason `gh executable gh was not found on PATH`.
- A blocklist entry for `github.com` threw before any gh child or provider request.
- An already aborted signal threw `The operation was aborted` in 4 ms with no provider request.

Live Pi wiring already loads this package from
`/var/home/user/Monochromatic/package/pi-plugin/search-fetch`,
and the package `pi.extensions` entry points at the rebuilt `dist/final/node/index.mjs`.

### Commits

- `fffd2d8d2`,
   routing,
   planner,
   child boundary,
   client,
   and the `fallbackChain` rename.
- `4a26b3f64`,
   README behavior documentation.
- `a2af1fb35`,
   planner,
   child boundary,
   client,
   and routing tests.

### Review round after the first implementation

An independent review of the first implementation found six defects,
each confirmed by measurement before fixing:

- Reference values crossed a URL syntax boundary unencoded.
   Measured:
   a blob URL for reference `topic+one&x=1` produced `?ref=topic+one&x=1`,
   which parsed as `ref` equal to `topic one` plus an injected `x` parameter.
   Fixed by decoding the reference and re-encoding it as one whole query value.
- Release tags kept URL escaping inside an argv value,
   and the leading-dash check ran before decoding.
   Fixed by decoding first and validating after.
- Cancellation protection covered only the gh hop.
   Fixed by rethrowing before every provider hop,
   with a test that cancels during the Linkup call and asserts Exa is never called.
- Split retries continued after environmental failures.
   Fixed by retrying only gh's ordinary exit code 1.
   Measured afterwards:
   a deep blob path with `gh` missing made one attempt instead of three.
- A failed optional comment read was logged but invisible to the model.
   Fixed with a one-line notice in the returned text.
- Whitespace-only file content was dropped by a trimmed-emptiness test.
   Fixed by dropping only exactly empty parts.

Three ambient-environment hazards were found by measurement while checking that review:

- `GH_HOST` redirects `gh api` to another host.
   Measured:
   `GH_HOST=github.example.invalid` made `gh api /repos/cli/cli` fail with
   `error connecting to github.example.invalid`.
- `GH_FORCE_TTY` switches gh to colored terminal-shaped output.
   Measured:
   one issue comment read grew from 477 bytes to 13990 bytes with ANSI escapes,
   and one `gh repo view` grew from 6349 bytes to 124371 bytes.
   `NO_COLOR=1` removes the escapes but not the shape,
   so the variable is pinned empty.
- `CLICOLOR_FORCE=1` adds ANSI escapes to `gh pr diff` and `NO_COLOR=1` does not override it,
   so the diff plan also passes `--color never`,
   measured ANSI-free with `CLICOLOR_FORCE=1` ambient.

The runner now pins `GH_HOST=github.com`,
`GH_FORCE_TTY` empty,
`CLICOLOR_FORCE=0`,
and `NO_COLOR=1` in the child environment.
Verified end to end with all three hostile values set in the parent process:
repository home returned 6349 raw bytes,
the issue thread returned 2194 bytes identical to the unpinned baseline,
and the pull request diff carried no ANSI escapes.

A NUL byte now counts as non-text alongside lossy UTF-8 decoding,
because a byte round trip alone accepts valid UTF-8 that carries NUL.

Provider fallback helpers moved to `src/provider-fallback.ts` to keep the router under the
300 code-line budget.

Also corrected in `doc/troubleshooting/gh-view-comments-raw-output.md`:
gh 2.101.0 rejects `--comments` together with `--json` through `cmdutil.MutuallyExclusive` at
`pkg/cmd/issue/view/view.go:60-63`,
with exit code 1 and `specify only one of --comments or --json`,
rather than silently ignoring a flag as the older upstream report described.

Commit `4e990aaf2` carries every fix in this subsection.

### Settled scope questions

Asked together on 2026-09-26 with pros,
cons,
and a ranking;
the user answered neither.

- `/actions/runs/{id}` stays unmapped and falls back to the paid providers.
   Declined.
   The summary-only form is small but rarely the wanted content,
   and the useful form,
   `gh run view --log-failed`,
   has unbounded output that would need a second byte cap.
- Commit lists at `/{owner}/{repo}/commits/{ref}` stay unmapped.
   Declined.
   A jq-projected log is bounded and cheap,
   but it is a summary rather than the page,
   and faithful coverage would need `--paginate`,
   which reintroduces unbounded output.

Settled by requirement rather than by asking:

- GitHub Enterprise Server hosts stay unmapped.
   Host detection cannot identify a GHES host from a URL alone,
   so support would need a config surface nobody has asked for.
   All mapped hosts are github.com.
- No config flag gates the gh route.
   A missing,
   unauthenticated,
   slow,
   or failing `gh` already falls back to Linkup and then Exa with the reason recorded in
   `fallbackChain`,
   so a flag would only add a way to reach behavior the fallback already provides.

## Next immediate step

No required implementation step remains for the migration or for GitHub URL routing.
Future work can rename internal `Linkup*` compatibility type names if desired,
but public tools and active Pi wiring are already provider-neutral.
Every scope question in the settled-questions subsection is closed.
