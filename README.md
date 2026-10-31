# Monochromatic

A TypeScript and Rust monorepo (148 workspace packages plus 18 standalone Rust crates)
for developer tooling,
coding-agent infrastructure,
custom linters,
desktop applications,
web applications,
and infrastructure automation.

## Highlights

**Minimal MCP server**:
[`mcp-stdio`](package/mcp/stdio/) implements Model Context Protocol
revision 2026-07-28 over stdio,
with three runtime dependencies:
one workspace module,
`valibot`,
and `@valibot/to-json-schema`.
The official `@modelcontextprotocol/sdk` pulled 5.8 MB and 17 dependencies
at audit time (Express,
Hono,
jose,
OAuth,
rate limiting,
SSE);
this package implements only JSON-RPC 2.0,
`server/discover`,
`tools/list`,
and `tools/call`.

**Coding-agent infrastructure**:
[`claude-code-plugin`](package/claude-code-plugin/) packages Claude Code hooks and plugins
(guardrail,
statusline,
bash output filtering,
session spawning,
housekeeping),
and [`pi-plugin`](package/pi-plugin/) does the same for the Pi coding agent
(advisor,
guardrail,
model selection).
Harness-agnostic logic (session discovery,
usage projection,
shell-command analysis,
terminal titles) lives in
[`agent-harness-shared`](package/agent-harness-shared/).

**Unified linter**:
[`linter/monochromatic-lint`](package/linter/monochromatic-lint/) is one Rust executable
that lints Rust,
Markdown,
and MDX,
including Rust fences in Markdown and the Markdown inside rustdoc.
It enforces the repo's per-file code-line budget (`rust/max-lines`),
rustdoc coverage (`rust/require-rustdoc`),
and the Markdown rules across the whole tree,
under the root `monochromatic-lint.config.jsonc`,
from `mise run lint` and `mise run format`.
A sibling Gradle-based Kotlin linter lives in
[`linter/kotlin`](package/linter/kotlin/).

**Policy-aware git tooling**:
[`git-policy-cli`](package/git-policy/cli/) wraps git with policy guards
(rejecting bulk staging and pathspec-less commits,
guarding worktree removal) as a native Rust executable configured by data in `cli-git.config.jsonc`.
[`forbidden-strings/cli`](package/forbidden-strings/cli/) is a gitignore-aware Rust scanner
for banned tokens and leaked credentials,
with a coverage-guided [`cargo-fuzz` harness](package/forbidden-strings/cli.fuzz/).

**Custom Oxlint plugins**:
[`oxlint-tsdoc`](package/oxlint-plugin/tsdoc/) enforces TSDoc correctness
across 20+ rules with fixture-based tests,
replacing the slow eslint-plugin-jsdoc integration that previously required ESLint.
[`oxlint-no-restricted-syntax`](package/oxlint-plugin/no-restricted-syntax/)
encodes monorepo-specific AST rules (no arrow functions,
no switch statements,
destructured params for functions taking two or more arguments)
that Oxlint's lack of AST selectors demands as dedicated rule implementations,
alongside [`stylistic`](package/oxlint-plugin/stylistic/),
[`prefer-readonly-parameter-type`](package/oxlint-plugin/prefer-readonly-parameter-type/),
and [`test-import`](package/oxlint-plugin/test-import/) plugins.

**Wayland-native desktop applications**:
[`file-manager/app`](package/file-manager/app/) is a native file manager
presenting a Niri-like horizontal strip of columns on GTK4,
with native Wayland drag-and-drop.
[`terminal/app`](package/terminal/app/) is a Slint terminal
backed by Ghostty's libghostty-vt core.
[`nested-wayland-session/cli`](package/nested-wayland-session/cli/) hosts one Wayland client
fullscreen on the GPU (dmabuf) path for GUI testing,
screenshotting the framebuffer and injecting synthetic input
over a Unix-socket control API.

**True-peak music player**:
[`music-player`](package/music-player/) normalizes playback to -1 dBTP
with an attenuate-only gain derived from Catmull-Rom inter-sample peak estimation.
The meter and policy identity live in a shared Rust core
([`truepeak-core`](package/music-player/truepeak-core/))
consumed by a Slint desktop app and a Kotlin Android app.

**Security-audited dependency selection**:
[`AUDIT.md`](doc/audit/README.md) documents source-code audits with dates and verdicts
for every non-trivial dependency.
Framework selection rejected Elysia after discovering an RCE vulnerability chain
through `new Function()` code generation
([GHSA-8vch-m3f4-q8jf](https://github.com/advisories/GHSA-8vch-m3f4-q8jf))
and measuring a 45x performance regression (3,853 vs 175,951 req/s)
when AOT compilation was disabled.
See [`PHILOSOPHY.tool-choices.md`](doc/philosophy/tool-choices.md) for the full
analysis of h3 vs Elysia vs Hono.

**Monorepo-aware CSS build tool**:
[`build-tool-css`](package/css/cli/) resolves `@import` through
`package.json` exports mappings and `node_modules`,
processes custom `@mixin`/`@apply` syntax,
and generates CSS strings for Shadow DOM injection,
all without native binaries.

**OpenTofu firewall automation**:
[`config-tofu`](package/config/tofu/) dynamically aggregates CIDR ranges
from 7 CDN sources (Cloudflare,
CloudFront,
Fastly,
GitHub,
YouTube,
Ubuntu ASN,
Coolify),
summarizes them to minimize Hetzner firewall rule count,
and caches ASN lookups for 30 days with graceful fallback to expired cache
on fetch failure.

**Custom typeface from SVG geometry**:
[`typeface-aquaticat`](package/typeface/aquaticat/) parses a master glyph strip SVG,
expands stroked outlines into filled contours using polygon offset math,
assembles an OpenType font via opentype.js,
and converts to WOFF2 through fonttools.

**Inference canary (paused)**:
[`inference-canary`](package-paused/dev-script/inference-canary/) runs five
code-generation probes against LLM models in parallel,
each executing inside a locked-down Podman container (no network,
read-only filesystem,
256 MB memory,
15 s timeout) and scored across correctness,
lint quality (oxlint),
and type safety (tsc),
with statistical threshold detection (mean - 2*stddev) flagging model degradation.

## Initial setup

### Prerequisites

Install [Mise][mise-home] as the task runner, `curl`, `jq` and `tar`.
The toolchain commands are the Rust program `mpm-shadow`
(see [Provision the toolchain](#clone-and-bootstrap) for the one prebuilt archive a fresh machine needs).
Mise runs tasks only;
every tool this repository runs (Node,
pnpm,
dprint,
Rust nightly,
Java,
Bun for explicit Bun islands,
and the rest) is provisioned from `toolchain.lock.json` by the repository
toolchain,
recorded in [`doc/decision/mpm-toolchain-adoption.md`](doc/decision/mpm-toolchain-adoption.md).
The `bin`-sourced tools resolve their releases through the GitHub API,
so `mpm-shadow install` reads `GITHUB_TOKEN` from the environment or the machine's
`gh` login;
an unauthenticated machine can exhaust the API quota mid-install.

### Clone and bootstrap

```sh
git clone https://github.com/Aquaticat/Monochromatic.git
cd Monochromatic
```

Trust the configuration so Mise evaluates environment variables,
templates,
and tasks:

```sh
mise trust
```

Trusting the monorepo root implicitly trusts all descendant `mise.toml` files
under `package/`.
See [mise trust docs](https://mise.jdx.dev/cli/trust.html)
for details on what this enables and why it is required.

A fresh machine has neither `mpm-shadow` nor `file-enforcer`.
Fetch the `mpm-shadow` archive that `toolchain.lock.json` records for your platform,
check it against the SHA-256 recorded beside it,
and put the binary on `PATH`
(the same steps are in `.github/actions/repo-tools-setup/action.yml` for CI and in the `Dockerfile`):

```sh
# Run from the repository root; platform is linux-x64 or linux-arm64.
platform=linux-x64
url="$(jq --raw-output --arg platform "${platform}" '.tools["mpm-shadow"].artifacts[$platform].url' toolchain.lock.json)"
sum="$(jq --raw-output --arg platform "${platform}" '.tools["mpm-shadow"].artifacts[$platform].sha256' toolchain.lock.json)"
curl --fail --silent --show-error --location --output /tmp/mpm-shadow.tar.gz "${url}"
echo "${sum}  /tmp/mpm-shadow.tar.gz" | sha256sum --check
mkdir --parents "${HOME}/.local/bin"
tar --extract --gzip --file /tmp/mpm-shadow.tar.gz --wildcards --strip-components=1 \
  --directory "${HOME}/.local/bin" '*/mpm-shadow'
```

Then provision the toolchain,
converging every tool to the versions `toolchain.lock.json` records
(that installs `file-enforcer` too):

```sh
mise run toolchain:install
```

The `bootstrap` and `prepare:android` tasks are aliases of `toolchain:install`,
and `mpm-shadow install` runs the same work without the task wrapper.
The install stages its own bootstrap tools first:
`bin` as a standalone release binary
(SHA-256 checked against the release `checksums.txt` values),
then `uv`,
then [mpm][mpm-home] as the PyPI wheel `meta-package-manager` 8.2.0
(SHA-256 checked against PyPI's published digest,
installed with `uv tool install`).
Then it installs the remaining tools
and rebuilds the gitignored `.cache/toolchain-bin` directory that
`.config/mise/conf.d/20-env.toml` puts first on `PATH`.
Then install dependencies and build all packages:

```sh
mise run prepareAndBuild
```

### Tool inventory and drift

- `mise run toolchain:update` re-resolves the floating requests and the
  channel stamps to their current upstream versions and rewrites
  `toolchain.lock.json` as a reviewable diff
  (`mpm-shadow update`);
  `mise run toolchain:outdated` reports what it would move without writing;
  explicit pins never move without a plan edit.
- `mise run toolchain:list` prints observed versus locked versions and exits
  non-zero on drift
  (`mpm-shadow list` runs the same check),
  and `mise run toolchain:lock:check` is the offline gate that fails when
  `toolchain.plan.jsonc` and `toolchain.lock.json` disagree.
- `mpm installed` lists what every package manager on the machine holds,
  `mpm outdated` lists available upgrades,
  and `mpm sbom` exports the inventory as SPDX or CycloneDX.
- `mpm-shadow bin --json` prints the
  exposure directory,
  PATH directories,
  and one absolute path per linked binary as JSON
  (`exposureDir`,
  `pathDirs`,
  `bins`),
  which is what IDE integrations read.

Declarations carry no `@version` pin syntax and keep floating requests;
`toolchain.lock.json` fixes what the sync installs
(the semantics `mise install` had against `mise.lock`),
so the explicit pins still install exactly.
The install installs the locked versions the way `pnpm install` honors
`pnpm-lock.yaml`;
re-resolving versions is a separate update action,
the `pnpm update` analogue.

Version bumps change the plan in `toolchain.plan.jsonc` and the lock
`mpm-shadow` writes from it (`toolchain.lock.json`),
never a hand-edited lock.
Upgrades of a single machine package name their tool
(`mpm upgrade <package>`);
a bare `mpm upgrade --all` moves every manager's packages and rewrites the
toolchain state all sessions' builds share.

## Essential commands

All builds and tasks use `mise run`.
Never invoke raw tools (`tsc`,
`tsdown`,
`bun test`,
`oxlint`,
`cargo`) or package manager scripts (`npm run`,
`pnpm exec`) directly.

```sh
# Build all packages
mise run build

# Run all tests (unit + browser + e2e)
mise run test

# Build then test (use this after editing source)
mise run buildAndTest

# Build and test a specific file
mise run buildAndTest -- package/async-time/core/src/wait.unit.test.ts

# Lint all files (oxlint, dprint, markdown, rust, detekt)
mise run lint

# Format all files
mise run format

# Full validation: format, build, test
mise run validate

# Watch mode
mise run watch:build
mise run watch:test
```

Run a task in a specific package with the monorepo path prefix:

```sh
mise run //package/async-time/core:test:unit
mise run //package/done/app:build
```

## Project structure

```text
package/
  agent-harness-shared/     Shared agent-harness plugin logic (session discovery,
                              usage projection, shell-command analysis)
  build-tool/               Build tooling (CSS processor)
  claude-code-plugin/       Claude Code plugins and hooks (guardrail, statusline,
                              bash-output-filter, claude-spawn)
  cli/                      CLI tools (mvm, vmsync, terminal-exec, fy, rgffplay,
                              markdown-lint, mutation-test, unused-export,
                              WireGuard helpers, forbidden-strings scanner,
                              nested Wayland compositor)
  config/                   Shared configurations (cosign, dotfiles, dprint, oxlint,
                              rolldown, tofu, typescript, lfs-r2-worker)
  desktop-app/              Desktop applications (GTK4/Qt/Electron file managers,
                              Slint terminal, Electron infrastructure)
  desktop-daemon/           Background services (hall-monitor)
  dev-script/               Developer utilities (file-enforcer, catalog-tighten,
                              deps-cube, page-weight, vm-builder, watch-restart)
  figma/                    Figma and Penpot tooling (kiwi parser, to-penpot converter)
  fuzz/                     Coverage-guided fuzzing harnesses (forbidden-strings)
  git-policy/               Policy-aware git wrapper (native Rust executable) and its fuzz sidecar
  intellij-plugin/          IntelliJ plugins (islands-black theme)
  kwin/                     KWin scripting (key-helper script and service)
  learning/                 Learning exercises (rust)
  linter/                   Custom linters (unified Rust/Markdown linter, Kotlin linter)
  mcp/                      Model Context Protocol servers (stdio framework, mvm)
  module/                   Core TypeScript libraries (logger, test harness, observable,
                              memoize, css/jsonc/toml editing, image diff, kv-store)
  music-player/             True-peak-normalizing music player (shared Rust core,
                              Slint desktop app, Kotlin Android app)
  ownership-marker/         Ownership boundary markers (foreign-borrowed)
  oxlint-plugin/            Custom Oxlint plugins (tsdoc, no-restricted-syntax,
                              stylistic, prefer-readonly-parameter-type, test-import)
  pi-plugin/                Pi coding-agent plugins (advisor, guardrail, statusline,
                              spawn, search-fetch)
  pi-shared/                Shared Pi plugin logic (model selection, model review)
  rolldown-plugin/          Rolldown/tsdown plugins (import-attributes)
  runtime-error/            Runtime error reproductions (bun)
  rust-module/              Rust libraries (forbidden-regex, jsonc-edit)
  shim/                     API-compatible dependency shims
  ssg/                      Static site generator for aquati.cat
  stub/                     Dependency blocklist stubs
  test-fixture/             Test fixtures for CSS imports, oxlint rules, and toml-edit
  typeface/                 Custom fonts (aquaticat geometric typeface)
  webapp-productivity/      Productivity apps (done, doodle widget, rss, wc)
package-paused/             Paused packages (inference-canary, monochromatic stylesheet,
                              webapp-forge, webapp-search, webapp-edu, editord)
package-deprecated/         Deprecated packages retained for reference
```

## Technical stack

- **Task runner**:
  [Mise][mise-home] with Node
  (`shell = "node --input-type=module-typescript -e"`)
  for cross-platform task logic
- **Toolchain manager**:
  [mpm][mpm-home] (Meta Package Manager) for the tool manifest,
  machine inventory,
  outdated detection,
  upgrade coordination,
  release-age cooldown,
  and SBOM export,
  plus the toolchain plan (`toolchain.plan.jsonc`) and the confirmed per-family install
  routes (`bin`,
  `cargo-binstall`,
  `rustup`,
  `android-sdk`),
  with versions fixed in `toolchain.lock.json`
- **Runtime**:
  [Node.js](https://nodejs.org/) for TypeScript task and CLI execution.
  Bun stays installed only for explicit Bun islands,
  such as the hall-monitor standalone compile task and runtime-error fixtures.
- **Package manager**:
  [pnpm](https://pnpm.io/) workspaces with catalog dependency management and isolated node_modules
- **Bundler**:
  [tsdown](https://tsdown.dev/) (Rolldown-based)
- **Languages**:
  TypeScript 7 with native `tsc` for type checking;
  Rust on the nightly toolchain as standalone crates without a root Cargo workspace
  (see `.out-of-scope/cargo-workspace.md`);
  Kotlin on Gradle for the Android app,
  IntelliJ plugin,
  and Kotlin linter
- **Linters**:
  Oxlint (with custom JS plugins for TSDoc and restricted syntax),
  the in-repo pluggable Rust linter plus Clippy,
  detekt,
  Harper (prose)
- **Formatter**:
  dprint (orchestrates all formatters including oxlint auto-fix)
- **Testing**:
  `@monochromatic-dev/test-harness-core` unit files executed through Node-based `mise run` tasks,
  Playwright in Podman for browser and e2e tests,
  `cargo-fuzz` for coverage-guided fuzzing
- **HTTP framework**:
  [h3](https://h3.dev/) for server applications
- **Desktop UI**:
  GTK4,
  Slint,
  and Electron
- **Infrastructure**:
  OpenTofu,
  Hetzner Cloud,
  Caddy,
  Podman

## Platform support

Development targets Linux (Fedora).
Use WSL2 on Windows:
some tools (e.g. Zellij) have no native Windows support.

## License

[LGPL-3.0-or-later](LICENSE) for code,
with root package metadata also declaring
[CC-BY-SA-4.0](LICENSES/CC-BY-SA-4.0.txt) for shareable documentation and content.

[mpm-home]: https://mpm.run/
[mise-home]: https://mise.jdx.dev/
