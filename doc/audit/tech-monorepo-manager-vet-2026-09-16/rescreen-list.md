# Re-screen list after dropping requirement (c)

Classification of every G3 exit in the eight screening inputs
(`screening-primary-chunk-{1,2,3}.md`, `screening-registry-chunk-{1,2,3}.md`, `screening-promoted-runners.md`,
and the `## Screening` section of `discovery-expansion.md`).
G1 exits, G2 exits, and user exclusions are ignored.
Machine-readable RESCREEN list: `rescreen-list.json`.

## Counts

- G3 exits read: 151
  - primary chunk 1: 23; primary chunk 2: 16; primary chunk 3: 17
  - registry chunk 1: 36; registry chunk 2: 14; registry chunk 3: 18
  - promoted runners: 10; discovery expansion: 17
- RESCREEN: 20 (6 of them borderline)
- STAYS OUT: 131

## RESCREEN

- folke/ultra-runner (primary 1): (c) only. Borderline: (a) annotated "partial" but not listed as failing.
- colcon/colcon-core (primary 2): (c) only, extension registration undocumented.
- chrismatix/grog (primary 2): (c) only.
- meslzy/outdo (primary 3): (c) only.
- Bolt (registry 1): (c) only. Borderline: exit text also says config section documents only `bolt.workspaces`.
- drkns (registry 1): (c) only.
- @enspirit/emb (registry 1): (c) only, built-in plugins without authoring docs.
- workspace-utils (registry 1): (c) only.
- rman (registry 1): (c) only.
- lattice (registry 1): (c) only, extension only by editing source.
- monist / monist-tools (registry 1): (c) only.
- cargo-flux (registry 3): (c) only.
- guild (registry 3): (c) only.
- Watchman (registry 3): (c) only.
- dev-process-manager (registry 3): (c) only. Borderline: notes CLI-to-daemon channel not documented as an interface.
- baton-run (Baton) (registry 3): (c) only.
- clier-ai (clier) (registry 3): (c) only. Borderline: notes Unix socket has no protocol reference.
- fbecart/zinoma (promoted runners): (c) only. Borderline: sample output shows 0.19.0 vs latest 0.19.6,
not recorded as confusion.
- Invincible (discovery expansion): (c) only.
- affected, Rani367 (discovery expansion): (c) only. Borderline: (a) is one annotated example,
which counted as an (a) gap elsewhere.

## STAYS OUT, flags worth a second look

These stay out under the literal rule, but the other failure is thin or tied to (c).

- Confusion located only in extension or plugin docs: Yarn (hook list missing from plugin tutorial),
Vite+ task runner (integration client linked off-site, same fact as its (c) gap),
Live Test Runner (custom adapter loading undocumented), pantsbuild/pants (plugin debugging sent to Slack),
thought-machine/please (plugin authoring path unclear).
- Plugin-docs confusion plus a separate non-plugin signal: gradle/gradle (also three dependency-declaration entry
points), sportradar/elixir-workspace (also ExDoc-only navigation structure signal),
BindPort (also "docs site is a work in progress").
- devtooie: otherwise (c) only; the block is one GitHub HTTP 429 abuse-detection response on a blob page,
while a later GitHub probe returned 200.
- smorsic/pacwich: its own subsection records (c) only, but the bun-workspaces subsection (registry 1) records
confusion on the same pacwich.dev docs.

## STAYS OUT

### Primary chunk 1

- lerna/lerna: confusion, commands reference sends options to per-command GitHub READMEs.
- vercel/turborepo: confusion, `daemon` config entry backs `turbo watch` but watch page never mentions it.
- nrwl/nx: coverage (e) one-line MCP running-task listings; confusion, watch only in Knowledge Base,
four module-boundary pages.
- bazelbuild/bazel: confusion, duplicate sidebar entries and BEP filed under Remote Execution.
- gradle/gradle: confusion, duplicate plugin-authoring page sets; three dependency-declaration entry points.
- teambit/bit: no-JS, reference pages are JS app shells.
- facebook/buck: confusion, frozen docs for a dead project.
- microsoft/rushstack: confusion, watch docs defer to GitHub PR #2298 and issue #1202.
- sbt/sbt: confusion, server protocol contradicts between 2.x and 1.x docs.
- pantsbuild/pants: confusion, rule graph debugging sent to Slack #plugins.
- com-lihaoyi/mill: confusion, "Reference" section mixes Scaladoc, changelog link, and talks.
- thought-machine/please: confusion, plugin authoring location unclear and flagged experimental.
- blade-build/blade-build: confusion, "Accessibility" page holds auxiliary commands.
- invertase/melos: confusion, Medium post as guide; raw MDX served without JS.
- guigrpa/oao: coverage (a), no configuration reference.
- lerna-lite/lerna-lite: confusion, config in wiki vs commands in package READMEs; YouTube reference.
- benchkram/bob: docs unreachable, bob.build NXDOMAIN.
- simplesurance/baur: confusion, wiki home mixes outdated drafts and logos with versioned docs.
- ojkelly/yarn.build: confusion, supported Yarn versions contradict.
- mbtproject/mbt: confusion, README "Visit Github issues for support".
- seansfkelley/yerna: confusion, README "a hack-based stopgap/overgrown experiment".
- giltayar/bilt: confusion, broken link to source for config type definition.

### Primary chunk 2

- paularmstrong/onerepo: confusion, `taskConfig` keys vs `tasks` examples.
- sportradar/elixir-workspace: confusion, "third party plugin" referenced with no plugin docs;
ExDoc-only navigation signal.
- smorsic/pacwich: confusion recorded under bun-workspaces (registry 1) on the same pacwich.dev docs.
- run-z/run-z: coverage (a) no options reference; confusion, API docs link is a README copy.
- electrode-io/fynpo: confusion signals, config page never names the config file.
- Farfetch/garment: confusion, CLI.md usage lines contradict and `tasks` section duplicated.
- AmbitionEng/qik: coverage (a) no configuration reference; confusion, broken runner anchor.
- bazurbat/jagen: confusion, configuration reference self-declared obsolete.
- leostera/warp: docs site DNS failure; repo docs are title-only stubs.
- rnza0u/blaze: confusion, "two kinds of executors" lists three; duplicate CLI nav link.
- abuob/yanice: confusion, config example link targets an image; plugin API only in fixtures.
- tylerbutler/trellis: structure signal (exit in chunk 2), sidebar "Full README" link-out
and design only in `docs/DESIGN.md`.
- chgibb/mono-surveyor: coverage (a), `surveys.json` only in prose.
- pnordahl/monorail: confusion, README `#tutorial` anchor missing.

### Primary chunk 3

- IKatsuba/runx: coverage (a); confusion, stale pinned install version.
- kristofferlind/knega: coverage (a); confusion, stale install version.
- egladman/magus: confusion, count-only API index; daemon and status pages contradict.
- Grevix/Rivox: coverage (a) example-only; confusion, "four CLI commands" vs nine.
- hjosugi/frost-build: coverage (b); confusion, quick start pinned to v0.8.0.
- nshkrdotcom/blitz: coverage (b) usage stubs; confusion, options only in source.
- Ashutosh0x/hyperblaze: coverage (a), (b) placeholders; confusion, watcher diagram vs "No file watcher daemon".
- chaitanya-archive/obelisk.build: no config or CLI reference; confusion, docs links and license disagree.
- jdarais/cobble: coverage (b), CLI reference "coming soon".
- Buck2: coverage (a) self-declared incomplete; confusion, `[project].ignore` missing from reference.
- Rush: confusion, `watchOptions` under experimental `watchForChanges` heading.
- pnpm workspaces and pnpm pipeline: confusion, contradictory `--no-bail` semantics.
- pitchfork: confusion, SSE vs NDJSON log streaming.
- systemd user units: confusion in navigation; HTTP 418 bot block on man pages.
- Aspect CLI: confusion, builtins on Types page; duplicate llms.txt titles.
- microsoft/lage: coverage (d) watch undocumented; confusion, `--no-deps` alias vs text.

### Registry chunk 1

- pnpm: confusion, task-running docs scattered across unlinked nav sections.
- Yarn: confusion, plugin hook list missing ("Our new website doesn't support generating the hook list yet").
- wsrun: coverage (a), no configuration file reference.
- Vite+ task runner (vp run): confusion, integration client documented by an off-site npmx.dev link.
- @visulima/vis: confusion, hook table contradicts plugin guide; caching guide 404.
- bun-workspaces: confusion, duplicated page title and leaked authoring text.
- mono-vir: coverage (a).
- yakumo: no configuration, CLI, or extension docs.
- bun-spaces: coverage (a).
- @forklaunch/bunrun: coverage (a).
- @aklinker1/buildc: no configuration or CLI reference.
- @nu-art/build-and-install: no configuration reference, incomplete CLI; npm README missing.
- neex: coverage (a); confusion, README names `neexp`.
- calviche: no configuration or CLI reference.
- laoban: confusion, defaults only in raw GitHub JSON; stray `</div` text.
- crowd: README "Documentation will follow."
- mrpm: coverage (a).
- @jakehamilton/titan: coverage (a).
- vx: confusion, unrendered template href returns 404.
- monopkg: docs site does not resolve.
- Fabr: docs site curl connection timeout (recorded as weaker exit).
- monox: coverage (a), `monox.toml` without field reference.
- @varlabs/monorun: confusion, `CacheRow` and `ctx` undocumented.
- bun-manage-workspace: coverage (a).
- just-build-tools: coverage (a), one example only.
- mono-runner: confusion, truncated `--no-wait` sentence.
- pmnps: confusion, `run` command missing from command options.
- @webeferen/buildable: coverage (a), example file only.
- mrdr: coverage (a).

### Registry chunk 2

- workspace-builder: confusion, watch mode claimed with no way to enable; coverage (b).
- @monoloom/cli: confusion, CI example uses undocumented `workspace-tools`.
- nasti-task: confusion, undocumented `-r` and `#` target syntax; coverage (a), (b).
- npm-mono-repo: confusion, undefined `packagesDir`.
- Rex: confusion, empty docs index and dead wiki link.
- bdep: coverage (a); confusion, build runner unclear.
- npm-ws: confusion, configuration deferred to another project's package.json.
- sail: confusion, `--workspace` described as release group.
- @anzerr/mono.cli: confusion, undocumented `restore` and `--count`.
- hwan: confusion, README is "// TODO".
- @messman/node-mono-builder: confusion, pseudocode config; CLI example contradicts command list.
- quimera: confusion, README truncated after first heading.
- ghidora: confusion, contradictory `dependsOn` comments.
- linkctl: confusion, getting-started output contradicts `dependsOn`; coverage (b).

### Registry chunk 3

- cuenv: confusion, cross-project deps only in ADR; internal test notes in user docs.
- fyrer: confusion, undefined `concurrency` key.
- haz: Anubis bot challenge on the forge.
- repoctl-runner: coverage (a), no manifest field reference.
- breach: confusion, `dependencies` and `mode` fields undefined.
- braid: confusion, plugin availability contradicts; config reference deferred to `src/types.ts`.
- @fervon/launchpad: coverage (e), (f) API only a curl example; naming confusion.
- devtooie: HTTP 429 GitHub abuse-detection block on `docs/control-api.md`.
- devmux: confusion, session naming convention contradicts.
- BindPort: confusion, hook environment variable formats undefined; docs site unpublished.
- bizi: coverage (a), (e), (f); config example text absent.
- park: confusion, "Intended Interface" and restart-policy status contradict.

### Promoted runners

- nadlejs/nadle: confusion, `CopyTask` option `to` vs `into`.
- OctaHive/octa: confusion, wrong-repository license badge; coverage (a), (b).
- no0dles/hammerkit: confusion, "Guides" is both nav section and a page.
- tuist/once: confusion, Target Kinds sidebar omits indexed kinds.
- kraken-build/kraken: confusion, docs say components "not very well documented"; duplicate nav label.
- gregnazario/must: confusion, docs site unlinked; footer license contradicts.
- Jomy10/beaver: confusion, README keys absent from guide and reference.
- BobBuildTool/bob: confusion, docs labeled unreleased 1.3.dev999; wrong axis name.
- oxequa/realize: confusion, broken "Get started" anchor; coverage (e).

### Discovery expansion

- fwatchd: coverage (a), (b).
- gw: no-JS, docs site is a JS shell.
- fx: docs domain fx.dev does not resolve.
- mono-cli: confusion, install command fetches an unrelated npm package.
- lazyrepo: coverage (a), full config shape only in `index.d.ts`.
- Jazelle: confusion, README anchors missing.
- Live Test Runner (`@livetest/cli`): confusion, custom adapter docs point to tests; daemon loading undocumented.
- Monex: confusion, `context-daemon` vs `monex-daemon`.
- Wireit: confusion, `service` reference row links to cleaning section.
- fluid-build: confusion, task list and config deferred to source.
- Shadowdog: confusion, MCP auto-start vs "add the MCP plugin" step.
- Tilt: confusion, editor temp-file list undocumented ("please file a bug").
- process-compose: confusion, launcher details deferred to a GitHub issue comment.
- @sohaha/zzz: no-JS, homepage is a loading shell.
- rnme / runme: confusion, MCP mode only in design doc with `rnme` binary name.
