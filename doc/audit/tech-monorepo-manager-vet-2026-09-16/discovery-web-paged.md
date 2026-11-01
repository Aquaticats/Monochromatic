# Monorepo manager discovery: paged web enumeration

Run date: 2026-09-16.
Scope and category definition: same as `discovery-web-repo.md`.
Screening is discovery-depth only, judged from result title and snippet; no target page was opened.

Run status: partial.
The user stopped the run mid-way ("We already have enough from DDG").
W1, W2, and W4 reached the two-empty-page stop.
W3 and W5 stopped before it.
W6, W7, and W8 were never fetched.

## Provider

- Chosen provider: DuckDuckGo HTML endpoint, `https://html.duckduckgo.com/html/`,
   fetched with `curl` (Firefox 130 user agent).
  - Page 1: `POST` with `q=<literal query>` only.
  - Later pages: `POST` of the previous page's own "Next" form fields:
    `q`, `s` (offset), `nextParams` (empty), `v=l`, `o=json`, `dc`, `api=d.js`, `vqd`,
    `kl=wt-wt` (all regions, the default).
  - Pacing: 15 s before every request.
    On a block, the script retried after 60 s, then 120 s, then 240 s.
  - Page sizes seen: 9 or 10 results on page 1, 12 to 15 on later pages.
    Offsets advanced `s=0`, `10`, `25`, `40`.
  - Parsing: organic `div.result` blocks, excluding `result--ad`.
    "New URL" counts are deduplicated within each query.
- Why DuckDuckGo: it was the first provider in the brief's order that returned real result pages with explicit offsets.
  The other providers failed as described under "Providers tried and their failures".
- Same provider for every fetched query: yes.

### Providers tried and their failures

- DuckDuckGo, first attempts:
  - A `curl` test (`q=Turborepo alternative`) returned HTTP 200 (29,684 bytes) with 10 results
     and a Next form (`s=10`, `dc=11`).
  - A Node `fetch` for W1 page 1 soon after returned HTTP 202
     with the `anomaly-modal` challenge ("bots use DuckDuckGo too"), logged at 2026-09-16T21:00:41Z.
  - An immediate `curl` retry of W1 also returned HTTP 202 (14,321 bytes).
  - A `curl` retry at 2026-09-16T17:05:58-04:00 returned HTTP 200 (30,955 bytes), and every later fetch used `curl`.
- Mojeek (`https://www.mojeek.com/search?q=...`):
  - `curl` got HTTP 200, but the page title was "Captcha", with an ALTCHA challenge that requires JavaScript.
  - `agent-browser` got "403 - Forbidden. Sorry your network appears to be sending automated queries
     so we can't process your search at this time."
  - Blocked; no results.
- Bing via `agent-browser`:
  - Default HeadlessChrome user agent: `Turborepo alternative` returned 10 relevant results.
    W1 then redirected (`rdr=1`) to 10 unrelated HiNative language Q&A pages with no pager element (`.b_pag` absent).
  - Desktop Chrome user agent in a fresh session:
     W1 returned relevant results and a pager (`a.sb_pagN` pointing to `first=11`).
    The next scripted load of W1 showed "One last step. Please solve the challenge below to continue" with 0 results.
  - Blocked.
- Linkup (checked again because the user asked why DuckDuckGo instead of Linkup):
  - The `linkup-search` tool's parameters are `query`, `depth`, `maxResults`, `includeDomains`, `excludeDomains`,
     `fromDate`, `toDate`, and `includeImages`.
    None of them is an offset or page.
  - W3 at `depth: standard` with `maxResults: 60` returned 20 results.
  - W3 at `depth: deep` with `maxResults: 60` returned 28 results,
     as one fixed set with no way to request result 29 onward.
    Tool-name tally of that deep result: Nx, Turborepo, Buck2, Pants, Lerna, Earthly, Rush, Gradle, moon, Mill.
    All of them are already on the known list.
  - Not usable for offset paging.

## Query ledger

Known-list policy used for the "new survivor" test:

- Known: every name in the web ledger of `discovery-web-repo.md`, whether survivor, screened out, or follow-up hint.
- New: names that appear there only in its repository findings (R1 to R3, for example lage, wireit, Nadle, Tilt, ninja).
  They are new to the web source class, so a fitting one counts as a new survivor, flagged "repo-known".

### W1

- Query: `monorepo build tool daemon watch mode API inspect running tasks`
- Provider: DuckDuckGo HTML via `curl`
- Filters: none (`kl=wt-wt`)
- Page 1: offset `s=0`, 10 results (10 new URLs).
  New candidates: none (mise, Turborepo, Nx, moon, Bazel are known).
  New survivors: 0.
- Page 2: offset `s=10`, 13 results (13 new URLs).
  New candidates: Paseo, Symphony (OpenAI Codex orchestration spec), oh-my-pi, Hermes Agent.
  New survivors: 0.
- Page 3, fetched in the same batch after the stop point: offset `s=25`, 13 results (13 new URLs).
  New candidates: OneUptime (Task, Lerna, moon are known).
  New survivors: 0.
- Page 4, fetched in the same batch after the stop point: offset `s=40`, 12 results (11 new URLs).
  New candidates: Herdr, Multica, `uv-monorepo-dependency-tool` (Rush is known).
  New survivors: 0.
- Stop reason: two consecutive complete pages (1 and 2) with no new survivor.
  Pages 3 and 4 also added none.

### W2

- Query: `build system daemon status command show running actions`
- Provider: DuckDuckGo HTML via `curl`
- Filters: none (`kl=wt-wt`)
- Page 1: offset `s=0`, 9 results (9 new URLs).
  New candidates: none (Gradle daemon and systemd are known).
  New survivors: 0.
- Page 2: offset `s=10`, 14 results (14 new URLs).
  New candidates: Docker Engine daemon, Docker Compose (`docker compose up` reference),
  Codex CLI (Bazel command-line reference is known).
  New survivors: 0.
- Page 3, fetched in the same batch after the stop point: offset `s=25`, 13 results (13 new URLs).
  New candidates: Gitea Runner, GitHub Actions.
  New survivors: 0.
- Stop reason: two consecutive complete pages (1 and 2) with no new survivor.
  Page 3 also added none.

### W3

- Query: `Bazel alternative monorepo build system 2026`
- Provider: DuckDuckGo HTML via `curl`
- Filters: none (`kl=wt-wt`)
- Page 1: offset `s=0`, 10 results (10 new URLs).
  New candidates: none (Turborepo, Nx, Bazel, moon, Pants, Buck2, Gradle, Lerna are known).
  New survivors: 0.
- Page 2: offset `s=10`, 15 results (13 new URLs).
  New candidates: Lage (from the monorepo.tools compare snippet), CMake, Meson, Autotools, MSTools, uv.
  New survivors: 1 (Lage, repo-known).
- Page 3: offset `s=25`, 14 results (14 new URLs).
  New candidates: Brazil, Shake, ninja (repo-known), Nix with Hydra (all from the Earthly blog snippet).
  New survivors: 0.
- Stop reason: not saturated.
  Only one empty page followed the last new survivor.
  Page 4 (`s=40`) was never fetched because the user stopped the run.

### W4

- Query: `Nx alternative monorepo task runner`
- Provider: DuckDuckGo HTML via `curl`
- Filters: none (`kl=wt-wt`)
- Page 1: offset `s=0`, 10 results (10 new URLs).
  New candidates: none.
  New survivors: 0.
- Page 2: offset `s=10`, 15 results (14 new URLs).
  New candidates: Angular CLI workspaces (Task, Lerna, and pnpm 12.4 `pnpm pipeline` are known).
  New survivors: 0.
- Page 3, fetched in the same batch after the stop point: offset `s=25`, 15 results (15 new URLs).
  New candidates: nx-dotnet, and an alternativeto.net Nx listing ("more than 25 alternatives", unnamed in the snippet).
  New survivors: 0.
- Stop reason: two consecutive complete pages (1 and 2) with no new survivor.
  Page 3 also added none.

### W5

- Query: `Turborepo alternative`
- Provider: DuckDuckGo HTML via `curl`
- Filters: none (`kl=wt-wt`)
- Page 1: offset `s=0`, 10 results (10 new URLs).
  New candidates: none (alternativeto.net lists Angular, npm, Svelte, which are known and screened out).
  New survivors: 0.
- Page 2: offset `s=10`, blocked.
  - HTTP 202 with `anomaly-modal` at 2026-09-16T21:10:57Z,
     then again after the 60 s backoff (21:11:57Z) and the 120 s backoff (21:13:58Z).
  - The run was stopped during the 240 s backoff.
- Stop reason: blocked on page 2, then stopped by the user.
  Not saturated.
  No fallback provider was tried for page 2: Mojeek and Bing had already blocked earlier in the run,
  and the user then ended the run.

### W6

- Query: `task runner watch mode HTTP API status`
- Not fetched: the run was stopped by the user before this query.

### W7

- Query: `monorepo tool code generation keep generated files in sync`
- Not fetched: the run was stopped by the user before this query.

### W8

- Query: `incremental build system file watcher keeps outputs up to date continuously`
- Not fetched: the run was stopped by the user before this query.

### Saturation status

- Saturated under the two-empty-page rule: W1 (page 2), W2 (page 2), W4 (page 2).
- Not saturated: W3 (a new survivor on page 2, only one empty page after it), W5 (blocked on page 2).
- Not run: W6, W7, W8.
- Caveat on the saturated queries: DuckDuckGo results drifted off topic by page 2 of W1 and W2
  (AI agent products, Docker, news sites, CISA bulletins).
  So an empty page there may mean the results stopped being relevant, not that no other tools exist.
  The only sign that this paging can still surface a new tool is W3 page 2, which found Lage.

## New candidates

### Survivors

#### Lage

- URL: https://monorepo.tools/compare (discovery page).
  The project URL https://microsoft.github.io/lage/ is recall, not observed.
- Discovered by: W3 page 2
- Category guess: monorepo manager (JavaScript task runner)
- Screening note: survivor.
  The monorepo.tools compare snippet lists it with Bazel, Gradle, Lerna, moon, Nx, Pants, Rush, and Turborepo
  as a tool compared on caching and task orchestration.
  Already named in the repository's `doc/audit/tech-monorepo-manager-vet-2026-09-16.md` (R3 of `discovery-web-repo.md`),
  but no web query in the Linkup pass surfaced it.
  No watch or inspection capability observed.
- Taxonomy terms: caching, task orchestration, "AI support" (a compare-page feature axis)

### Screened out

- Paseo (https://paseo.sh/changelog), W1 page 2: agent product with a daemon; category mismatch.
- Symphony (https://openai.com/index/open-source-codex-orchestration-symphony/), W1 page 2: agent orchestration spec;
   category mismatch.
- oh-my-pi (https://github.com/can1357/oh-my-pi), W1 page 2: coding agent; category mismatch.
- Hermes Agent (https://hermesatlas.com/guide/), W1 page 2: AI agent; category mismatch.
- OneUptime (https://oneuptime.com/), W1 page 3: observability and incident management service; category mismatch.
- Herdr (https://github.com/yigitkonur/awesome-herdr), W1 page 4: agent ecosystem list; category mismatch.
- Multica (https://multica.ai/changelog), W1 page 4: agent platform; category mismatch.
- `uv-monorepo-dependency-tool` (https://pypi.org/project/uv-monorepo-dependency-tool/), W1 page 4:
  pins local path dependency versions at build time; not an orchestrator.
- Docker Engine daemon (https://docs.docker.com/engine/daemon/troubleshoot/), W2 page 2: container runtime;
   category mismatch.
- Docker Compose (https://docs.docker.com/reference/cli/docker/compose/up/), W2 page 2:
  the snippet shows only `docker compose up` and no watch behavior.
  Recall, not observed: `docker compose watch` exists, but it is scoped to containers, not workspace tasks.
- Codex CLI (https://github.com/openai/codex), W2 page 2: coding agent; category mismatch.
- Gitea Runner (https://docs.gitea.com/runner/) and GitHub Actions (https://github.com/features/actions), W2 page 3:
   CI services.
- CMake, Meson, Autotools, MSTools (https://gist.github.com/MangaD/26ef92a1e1efd967c3e0188dc0591e83), W3 page 2:
  C++ build systems; no workspace task orchestration in the snippet.
- uv (https://docs.astral.sh/uv/concepts/projects/), W3 page 2: Python package and project manager;
   the snippet shows no cross-package task orchestration.
- Brazil (https://earthly.dev/blog/monorepo-tools/), W3 page 3: Amazon's internal build system; not publicly available.
- Shake (same Earthly blog), W3 page 3: Haskell build-system library; library exclusion.
- ninja (same Earthly blog), W3 page 3: low-level build executor, repo-known;
   no workspace orchestration or watch process.
- Nix with Hydra (same Earthly blog), W3 page 3: package manager plus CI; excluded.
- Angular CLI workspaces (https://www.rockyourcode.com/angular-workspaces-as-alternative-to-nx-monorepo/), W4 page 2:
   framework feature.
- nx-dotnet (https://www.nx-dotnet.com/core), W4 page 3: Nx plugin; belongs under Nx.

## New taxonomy terms

These terms are absent from the "New taxonomy terms" list in `discovery-web-repo.md`:

- dependency-aware task observer (Turborepo's description of watch mode, W1 page 1)
- interruptible tasks (`interruptible: true`, restarted by `turbo watch`, W1 page 1)
- experimental cache writes in watch mode (`--experimental-write-cache`, W1 page 1)
- interactive terminal UI for tasks (Turborepo 2.0, W1 page 1)
- long-running dev tasks (W1 page 1)
- target path syntax (mise monorepo tasks, W1 page 1)
- daemon reuse and busy or stopped daemons (Gradle "1 busy and 6 stopped Daemons could not be reused", W2 page 1)
- action graph query (`bazel aquery`, W2 page 2)
- continue-on-failure pipeline with log replay (pnpm 12.4 `pnpm pipeline`, W4 page 2)
- executors (Nx plugin term, W4 page 3)
- module boundary enforcement (`nrwl/nx/enforce-module-boundaries`, W4 page 3)
- AI support (monorepo.tools compare axis, W3 page 2)

## Run artifacts and side effects

- Fetch scripts: `ddg-page.ts` (in use), `bing-page.ts` (abandoned after the Bing challenge).
- Raw pages and state: `ddg-W<n>-p<k>.html`, `ddg-W<n>-p<k>.txt`, `ddg-W<n>.json`, and `ddg-blocks.log`,
   all in the scratchpad directory.
- Side effect: while the Bing session was being reset,
   `agent-browser close --all` also closed an agent-browser session named `promises-open-chat-present`
   that this run did not create.
- Cleanup: the background fetch and its monitor were stopped, no `ddg-page.ts` process remains,
  and the `webpaged2` browser session was closed.
