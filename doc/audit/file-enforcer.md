# file-enforcer alternatives audit

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/file-enforcer` is now `package/file-enforcer/core`;
> `@monochromatic-dev/module-toml-edit` is now `@monochromatic-dev/toml-edit-core`.

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## What this audit is

`file-enforcer` keeps arbitrary repo-local derived files in sync inside this monorepo's worktree.
This document asks two questions and answers both with cited evidence:

- Replacement question:
   does any existing tool do file-enforcer's whole job,
   so we could retire the package?
- Absorption question:
   which capabilities from other tools should file-enforcer grow,
  so it becomes the single tool for repo-local derived-file work?

The conclusion is keep and extend.
No tool replaces file-enforcer,
 for a reason that survives a much wider candidate sweep than the first version of
this audit ran.
Two capabilities are worth absorbing,
 with cited reference implementations for each.

This revision supersedes an earlier draft whose method had two defects.
Both are corrected and documented below so future audits do not repeat them:

- it screened only `mise registry`,
   a binary-installer registry that structurally excludes the npm ecosystem where
  file-enforcer's closest peers live;
- it admitted configuration languages (`pkl`,
   `cue`,
   and similar) as candidates,
   when they cannot read,
   write,
   watch,
  or reconcile files at all.

## The screening gate: general-target reconciliation

A tool only counts as a file-enforcer replacement if it can manage arbitrary repo-local derived files.
A tool that can only manage its own fixed built-in domain is not good enough,
 however well it manages that domain.

This gate is the organizing principle of the audit because it decides most cases on its own.
It is what file-enforcer's root config actually exercises:
 in one run,
 the same tool generates
`CLAUDE.md` from `AGENTS.md`,
 regenerates `mise.toml` from a base plus computed workspace bin paths,
seeds and concatenates a forbidden-string deny list,
 resolves Browserslist targets through the installed package,
patches JetBrains LSP4IJ XML settings,
 mirrors skill markdown to legacy agent directories,
and asserts a forbidden root file stays absent.
Evidence:
 `file-enforcer.config.ts`.

No tool that hard-wires a single output schema can express that workload.
A tool wired to `package.json` can keep `package.json` files consistent and nothing else.
A tool wired to `$HOME` can deploy dotfiles and nothing else.
A tool that evaluates one configuration document to one data blob can emit that blob and nothing else.
Each is a non-candidate by the gate,
 and the gate is cited at the source level for every such tool below.

## The capability axes: the scoring spine

Tools that pass the gate are scored against one identical checklist.
The same checklist doubles as file-enforcer's own gap list,
 so the absorption roadmap falls out of it mechanically:
the axes where a real general-target peer scores and file-enforcer does not are exactly the features to absorb.

- A1 programmable generation:
   derived content from arbitrary code,
   not a template or declarative data only.
- A2 tracked source reads:
   the tool knows which inputs each derived file depends on.
- A3 content-stable writes:
   it skips the write when the destination already matches,
   and replaces atomically.
- A4 structured comment-preserving transforms:
   JSON,
   TOML,
   and XML edits that keep comments and unmutated whitespace.
- A5 incremental fingerprinting:
   a persisted manifest that skips unchanged work across separate runs.
- A6 watch with managed-destination protection:
   re-run on a source change,
   and react when a generated file is edited
  by hand.
- A7 check or verify mode:
   report drift and exit nonzero without writing,
   for continuous-integration gating.
- A8 marker-region management in unstructured text:
   sync a marked slice of an otherwise hand-edited file that has no
  parseable structure to scope the edit,
   so an explicit text marker delimits the region;
   structured partial edits are
  A4,
   not this.
- A9 in-repo native integration:
   config is repo TypeScript calling functions directly,
   with no separate language and no
  shell glue.

### How file-enforcer scores

file-enforcer passes the gate and scores on A1 through A6 and A9.
It does not score on A7 or A8.
Those two absences are the roadmap.

Implementation evidence for the passing axes:

- A1:
   the config is plain TypeScript with top-level `await`,
   local imports,
   and direct function calls.
  Evidence:
   `file-enforcer.config.ts`,
   `package/dev-script/file-enforcer/src/cli.ts`.
- A2:
   `cat()` records every read,
   glob expansions are tracked,
   and `addWatchedPaths()` registers extra dependencies.
  Evidence:
   `src/io/cat.ts`,
   `src/tracker.ts`.
- A3:
   `overwrite()` reads the existing destination through `readExisting()` and logs `skip (unchanged)` when content
  already matches;
   writes are atomic.
  Evidence:
   `src/io/write.ts`,
   `src/io/write-atomic.ts`.
- A4:
   structured transforms for JSON,
   TOML,
   and XML beyond byte concatenation,
   with comment-preserving TOML splice mode
  through `@monochromatic-dev/module-toml-edit`.
  Evidence:
   `src/pipeline/json.ts`,
   `src/pipeline/toml.ts`,
   `src/pipeline/xml.ts`,
   `src/io/write-toml.ts`.
- A5:
   a persisted staleness manifest under `node_modules/.cache/file-enforcer/` records read files,
   glob expansions,
  and destination content hashes,
   and lazy builders skip their content callbacks when those still match.
  Scoped: the root config contains no lazy builder and records no entries at all,
   so A5 pays nothing for this config even though the machinery works.
  See `doc/troubleshooting/file-enforcer-staleness.md`.
  Evidence:
   `src/io/staleness-manifest.ts`,
   `src/io/staleness-hash.ts`,
   `src/io/write-lazy.ts`.
- A6:
   watch mode classifies events into source,
   protected,
   and ignore,
   re-runs on source changes,
   and fires a desktop
  notification when a managed destination is edited externally.
  Evidence:
   `src/watch/watch.ts`,
   `src/watch/watch-filter.ts`,
   `src/watch/notify.ts`.
- A9:
   the config is repo TypeScript;
   there is no descriptor collection phase and no engine interpreter.
  Evidence:
   `package/dev-script/file-enforcer/README.md`.

The two gaps,
 with the package's own acknowledgement:

- A7 check or verify mode:
   `package/dev-script/file-enforcer/TODO.md` records "No dry-run mode" and frames it as
  needing a descriptor pattern.
  The roadmap section below shows why that framing overstates the cost.
- A8 marker-region management:
   file-enforcer already does partial,
   key-scoped edits for structured formats under A4,
  the TOML splice (`overwriteTomlKey`,
   `editTomlKey`),
   the XML entry replace-or-insert (`replaceOrInsertXmlEntry`),
  and the JSON key merge (`mergeFlatJson`),
   so generated and hand-edited content already coexist in structured files.
  What it lacks is a marked region in an unstructured file,
   where no key path scopes the edit.
  `TODO.md` records the absence of `appendTo` and `prependTo`;
   a managed region goes further than either.

Measured footprint,
 for scale,
 on 2026-06-15:
`package/dev-script/file-enforcer/src` holds seventy-one production TypeScript files,
fifty-one unit-test files,
 seven property-test files,
 and one container test.
The package also defines fuzz and container-isolated mutation tasks.
Evidence:
 `package/dev-script/file-enforcer/mise.toml`,
 `doc/decision/file-enforcer-fuzzing.md`.

## Method

The candidate sweep drew on four surfaces,
 not one:

- the `mise registry` capture from 2026-06-15,
   eight hundred fifteen entries,
  produced with `mise registry > /tmp/agent/mise-registry-2026-06-15.txt`;
- the npm ecosystem of scaffolders,
   workspace-consistency tools,
   region embedders,
   watchers,
   and codegen;
- general build systems that derive output files from inputs;
- desired-state and file-sync tools,
   to bound the problem from the dotfile and cross-host side.

Evidence tiers are stated per tool and are not uniform,
 by design.
Depth is proportional to how close a tool comes to passing the gate.
Tools that could change the conclusion were cloned and source-read to the same standard the incumbent is held to:
shallow clone under `/tmp/agent/`,
 commit hash and date captured,
 production source spot-read with file-path citations,
tests and continuous-integration inspected,
 and fuzz or mutation evidence found or reported absent.
Tools that the gate dismisses structurally were verified against their own README or official documentation and given a
one-line reason;
 cloning them would not change their classification.

The registry blind spot is itself a finding,
 and the reason for widening the surface.
None of file-enforcer's closest peers appears in the registry at all.
Absent from the 2026-06-15 capture,
 verified by search:
`syncpack`,
 `manypkg`,
 `sherif`,
 `knip`,
 `wireit`,
 `nx`,
 `moon`,
 `redo`,
 `tup`,
 `plop`,
 `hygen`,
 `cog`,
 `embedme`,
`c12`,
 and `onchange`.
Even the registry-present build tools `turbo` and `ninja` were never surfaced as candidates by the first screen.
A registry of installable binaries skews toward compiled,
 standalone CLIs and away from the npm-resident tools that
actually solve adjacent problems,
 so registry-only screening cannot find them.

SaaS vendor vetting does not apply.
Every candidate is an open-source local CLI or library.
No closed-source or hosted tool is recommended.

## Tier one: non-candidates dismissed by the gate

These tools fail general-target reconciliation.
Each entry names the single structural reason.
None was a replacement finalist,
 so the evidence tier is README or documentation unless the tool also carried a
capability worth citing later.

### Configuration evaluation languages

These evaluate one configuration document into one data output.
They have no concept of reading arbitrary repo sources,
 choosing destinations,
 skipping unchanged writes,
 watching,
 or
protecting outputs.
The earlier draft listed them as serious candidates;
 that was the category error this revision corrects.
They are not replacements.
They are at most A4-layer components a reconciler could call to produce one structured blob,
and even then file-enforcer already owns comment-preserving structured edits in TypeScript.

- `pkl` (`apple/pkl`):
   evaluates Pkl to JSON,
   YAML,
   or similar.
  Fails the gate:
   emits one structured output,
   orchestrates no files.
- `cue` (`cue-lang/cue`):
   evaluates and exports CUE.
  Fails the gate:
   same shape.
- `dhall` (`dhall-lang/dhall-haskell`):
   evaluates Dhall to JSON,
   YAML,
   or TOML.
  Fails the gate:
   same shape.
- `jsonnet` (`google/go-jsonnet`):
   evaluates Jsonnet,
   including multi-output writing.
  Fails the gate:
   multi-output is still pure evaluation with no source tracking,
   watch,
   or destination protection.
- `ytt` (`carvel-dev/ytt`):
   templates and overlays YAML.
  Fails the gate:
   YAML templating only,
   no reconcile loop.

Pkl is the closest of these as a language.
Adopting it would rewrite the root generator into Pkl and still leave every reconciliation concern,
 A2,
 A3,
 A5,
 A6,
A7,
 and A8,
 unsolved,
 so it would add a language without removing the enforcement layer.

### One-shot scaffolders

These generate a project or a set of files once,
 at creation time,
 from templates and prompts.
They have no reconcile loop,
 so they cannot keep a derived file in sync after the first write.

- `cookiecutter` (`cookiecutter/cookiecutter`):
   project generation from templates.
- `copier` (`copier-org/copier`):
   project generation with a versioned-template update path.
  Copier is the strongest of this group because it can re-apply template updates,
  but it is template-repository oriented and does not track arbitrary repo sources or protect destinations.
- `boilerplate` (`gruntwork-io/boilerplate`):
   folder generation from templates and a manifest.
- `plop` (`plopjs/plop`):
   micro-generator that creates files from Handlebars templates via prompts,
   then exits.
- `hygen` (`jondot/hygen`):
   project-resident template code generator,
   one-shot per invocation.
- `scaffdog` (`scaffdog/scaffdog`):
   markdown-driven multi-file scaffolder,
   invoked interactively.
- `yeoman` (`yeoman/yo`):
   runs generators to kickstart new projects.

One borderline case sits here and earns a note rather than a clean dismissal.
`mrm` (`sapegin/mrm`) is codemod-style config maintenance:
 idempotent tasks that keep files such as `package.json`
and lint configs in sync without clobbering user data,
 contrasting itself explicitly with one-shot templates.
It fails the gate on narrow,
 manually-invoked scope rather than on lack of a reconcile concept:
it manages a curated set of task-defined config files,
 not arbitrary derived files,
 and has no watch loop.

### Desired-state, dotfile, and cross-host sync

These reconcile a desired state,
 but against the wrong target.
Their destination is the user's home directory or a second host or replica,
 not this repo's worktree.

- `chezmoi` (`twpayne/chezmoi`):
   applies a source-state directory to `$HOME`.
  Excellent at that;
   it is a personal-dotfile engine,
   not a worktree derived-file tool.
- `dotbot` (`anishathalye/dotbot`):
   symlinks dotfiles into `$HOME`.
- `stow` (`aspiers/stow`):
   builds symlink trees into an external install directory.
- `rcm` (`thoughtbot/rcm`):
   deploys config to `$HOME` with host variants.
- `yadm` (`yadm-dev/yadm`):
   a Git dotfile manager whose working tree is `$HOME`.
- `unison` (`bcpierce00/unison`):
   bidirectional sync between two replicas.
- `mutagen` (`mutagen-io/mutagen`):
   real-time sync between endpoints.
- `rsync` (`rsync.samba.org`):
   incremental mirroring between locations.
- `syncthing` (`syncthing/syncthing`):
   continuous peer-to-peer device sync.

### Narrow-domain workspace-consistency enforcers

This is the cluster the first audit missed entirely,
 because none of these tools is in `mise registry`.
They are the most tempting false peers,
 because they do enforce derived and consistent file content with a fix mode.
They all fail the gate at the source level:
 every read and write target is the npm `package.json` manifest schema,
sometimes plus `pnpm-workspace.yaml`.
They cannot write `CLAUDE.md`,
 `mise.toml`,
 JetBrains XML,
 or any file outside that schema.
This cluster is the direct evidence for the project rule that a tool managing only its own workspace is not good enough.

- `syncpack` (`JamieMason/syncpack`):
   Rust,
   distributed through npm.
  Keeps dependency versions and `package.json` formatting consistent across a workspace.
  Gate failure is structural:
   the source type for a managed file is an enum of exactly `Package` and `PnpmYaml`,
  and default discovery hard-wires `package.json` patterns.
  Evidence:
   `/tmp/agent/fe-wsconsist-syncpack-20260615` at `3b2c99d793975d54438f12da4e595a803afe5b8b` (2026-06-15),
  `src/source.rs`,
   `src/source_patterns.rs`,
   `src/disk.rs`.
  It does carry a capability file-enforcer lacks:
   a `lint` check mode distinct from `fix`,
   where `lint` reports and
  returns a nonzero exit without writing,
   and `fix` mutates with a `--dry-run` no-write variant.
  Evidence:
   `src/commands/lint.rs`,
   `src/commands/fix.rs`.
- `manypkg` (`Thinkmill/manypkg`):
   TypeScript.
  Lints `package.json` files across a workspace.
  Gate failure:
   the fix writer writes `path.join(pkg.dir, "package.json")` exclusively.
  Evidence:
   `/tmp/agent/fe-wsconsist-manypkg-20260615` at `97e2ab9bc64e45af099f63f25c515f3792738cfb` (2026-06-10),
  `package/cli/src/utils.ts`,
   `package/cli/src/checks/index.ts`.
  It splits `check` from `fix`,
   the same check-mode shape syncpack has.
- `sherif` (`QuiiBz/sherif`):
   Rust,
   distributed through npm.
  Opinionated `package.json` consistency linter.
  Gate failure:
   package construction hard-codes `package.json` and errors when it is absent.
  Evidence:
   `/tmp/agent/fe-wsconsist-sherif-20260615` at `f1e6490a681165db74ce38e91dc984d8c94d64eb` (2026-04-24),
  `src/packages/mod.rs`,
   `src/rules/multiple_dependency_versions.rs`.
  Default mode checks and exits nonzero;
   `--fix` mutates and refuses to run in continuous integration.
- Yarn constraints (`yarnpkg/berry`,
   `plugin-constraints`):
   built into Yarn Berry.
  This is the most capable of the cluster and still fails the gate.
  Its rules are fully programmable JavaScript in `yarn.config.cjs`,
   which beats the declarative-data tools on A1,
  but every mutation in its API targets the `package.json` manifest:
   `set`,
   `unset`,
   `update`,
   `delete`,
  persisted through `persistManifest`.
  The official documentation states it does not support arbitrary file writing.
  Evidence tier documentation plus targeted source read at `master` (2026-06-15):
  `package/yarnpkg-types/sources/constraints.ts`,
  `package/plugin-constraints/sources/commands/constraints.ts`,
  and <https://yarnpkg.com/features/constraints>.
  The lesson here is the load-bearing one for the whole audit:
   programmable rules are not the same as general file
  management.
  A programmable rules engine bolted to a fixed output schema is still narrow.
- `knip` (`webpro-nl/knip`):
   TypeScript.
  Listed because it is often grouped with the above,
   but it fails for a different reason and is better called a
  complement.
  It edits many file types,
   but every fix is removal only:
   it deletes unused files,
   strips unused exports,
  and removes unused dependencies and catalog entries.
  It has no path that generates or derives content,
   so it cannot enforce a derived file even though it can edit one.
  Evidence:
   `/tmp/agent/fe-wsconsist-knip-20260615` at `e265d281b031783dd7f92dfc0db29f60f7138d5b` (2026-06-15),
  `package/knip/src/IssueFixer.ts`.

### Configuration loaders

These resolve configuration into memory.
Three of them write nothing at all,
 so they cannot enforce any derived file.

- `cosmiconfig` (`cosmiconfig/cosmiconfig`):
   searches for and loads config;
   writes nothing.
- `lilconfig` (`antonk52/lilconfig`):
   zero-dependency config seeker;
   writes nothing.
- `unconfig` (`antfu/unconfig`):
   universal config loader;
   writes nothing.
- `c12` (`unjs/c12`):
   borderline.
  It loads and merges config,
   and an experimental `updateConfig` path,
   backed by `magicast`,
   can edit config files,
  so the blanket "writes nothing" is false for c12.
  Its real failure is that config editing is not a reconcile loop over arbitrary derived files.

## Tier two: layer components a reconciler calls

These tools do real work,
 but they are pieces a reconciler invokes,
 not reconcilers.
A replacement must define generated content and own the reconcile loop;
 these define neither.
file-enforcer could call several of them,
 and already depends on one.

### Watchers, the A6 trigger layer

These trigger commands on filesystem changes.
They generate no content and protect no destination,
 so they cover only part of A6.

- `watchexec` (`watchexec/watchexec`):
   mature cross-platform watcher and command runner.
- `onchange` (`Qard/onchange`):
   runs a command on glob-matched changes.
- `nodemon` (`remy/nodemon`):
   watches and restarts a Node process.
- `node --watch` (Node.
  js builtin):
   process restart on change.
- `watchlist` (`lukeed/watchlist`):
   recursive directory watcher running a command.
  Note:
   the canonical repo is `lukeed/watchlist`;
   a `vercel/watchlist` repo does not exist.
- `chokidar-cli` (`open-cli-tools/chokidar-cli`):
   CLI over chokidar.
- `chokidar` (`paulmillr/chokidar`):
   the watch library file-enforcer's own watch layer builds on.
  This is a dependency file-enforcer consumes,
   not a competitor.
  Evidence:
   `chokidar` in `package/dev-script/file-enforcer/package.json`.

### Structured-data editors and TOML tooling, the A4 layer

These edit one structured file at a time.
They cannot express the whole sync graph and provide no tracked sources,
 protected destinations,
 or programmable
monorepo API.

- `yq` (`mikefarah/yq`):
   YAML and JSON evaluation with in-place writes.
- `dasel` (`TomWright/dasel`):
   selector-based reads and writes across formats.
- `jq` and `gojq`:
   JSON transforms.
- `sd`:
   string replacement.
- `taplo` (`tamasfe/taplo`):
   TOML formatting and limited DOM rewriting.
- `tombi` (`tombi-toml/tombi`):
   TOML parsing,
   formatting,
   and schema-aware editing.
  Worth watching as a TOML complement;
   its maintenance signal is stronger than Taplo's.

For TOML specifically,
 this repo already uses `@monochromatic-dev/module-toml-edit` through file-enforcer to preserve
comments and unmutated whitespace in splice mode,
 so the A4 need these tools serve is met in-repo and in TypeScript.

### Template renderers, an A1 subset

- `gomplate` (`hairyhenderson/gomplate`):
   renders templates from data sources.
  Good when the problem is pure template rendering;
   it models none of A2,
   A3,
   A5,
   A6,
   A7,
   or A8.

### Schema-driven codegen, an A1 subset

Each emits output derived from one schema or input and could be called by a reconciler.
None owns a tracked-read,
 watch,
 or destination-protection loop over arbitrary files.

- `quicktype` (`glideapps/quicktype`):
   types and serializers from JSON,
   JSON Schema,
   TypeScript,
   or GraphQL.
- `json-schema-to-typescript` (`bcherny/json-schema-to-typescript`):
   JSON Schema to TypeScript.
- `openapi-typescript` (`openapi-ts/openapi-typescript`):
   TypeScript from OpenAPI specs.
- `graphql-codegen` (`dotansimha/graphql-code-generator`):
   code from a GraphQL schema,
   with a `--watch` flag.
  The watch flag is why the failing axis is the schema-only transform scope,
   not watch support.
- `ts-json-schema-generator` (`vega/ts-json-schema-generator`):
   JSON Schema from TypeScript.

### Hook and task runners, the invocation layer

These can run file-enforcer as a commit gate,
 a continuous-integration step,
 or a watch wrapper.
They do not define generated content,
 so they cannot replace it.

- `pre-commit` (`pre-commit/pre-commit`):
   hook installation and execution.
- `lefthook` (`evilmartians/lefthook`):
   Git hook execution.
- `just` (`casey/just`):
   command runner.
- `make`,
   `cargo-make`,
   `mage`,
   `cmdx`,
   `mask`,
   `xc`:
   task runners that can invoke a generator but author no content.

## Tier three: general-target tools that pass the gate but are not replacements

These tools can produce arbitrary output files,
 so they clear the gate,
 yet none replaces file-enforcer.
This tier carries the deepest analysis because these are the only tools that could have changed the conclusion,
and because two of them are the reference implementations for the absorption roadmap.

### Build-graph reconcilers

`make`,
 `ninja`,
 `redo`,
 `tup`,
 `wireit`,
 and `turbo` derive output files from inputs and skip work when inputs and
outputs are unchanged.
That overlaps file-enforcer on A5.
The decisive separation is content generation.
Every tool in this cluster runs a user-supplied shell command to produce content;
 none has a general native transform.
The single exception is `tup`'s built-in `varsed`,
 a narrow `@VARIABLE@` substitution subprogram,
 and even it runs as a
command inside a build rule.
So adopting any of them would require writing the generators as shell glue and would still leave A2,
 A4,
 A6,
 and A8 to
build by hand.
That is the opposite of file-enforcer's design,
 where generation is TypeScript with native transforms.

Where this cluster is genuinely ahead,
 and feeds the roadmap,
 is in two places:
 a real check mode,
 and a per-target
incremental graph.

- `make` (GNU make):
   mtime-based derivation,
   shell recipes.
  It owns the canonical check mode:
   `-q` or `--question` silently checks whether targets are up to date and runs no
  recipe,
   exiting zero when current and nonzero when an update is needed.
  This is precisely the A7 shape file-enforcer lacks.
  Its mtime model has no managed-destination protection:
   a hand-edited output looks newer than its inputs,
   so make
  treats it as current and skips it.
  Evidence tier documentation:
   <https://www.gnu.org/software/make/manual/>.
- `ninja` (`ninja-build/ninja`):
   mtime DAG executor of shell commands,
   usually fed by a generator such as CMake.
  Its `-n` or `--dry-run` is a plan preview that exits zero,
   not a drift gate.
  No watch,
   no destination protection.
  Evidence:
   `/tmp/agent/fe-buildgraph-ninja-20260615` at `f735970600b5713276583589dce207d575950b14` (2026-06-14),
  `doc/manual.asciidoc`,
   `src/subprocess-posix.cc`,
   `src/build.cc`.
- `redo` (`apenwarr/redo`):
   targets built by `.do` shell scripts.
  Freshness defaults to mtime,
   size,
   and inode,
   with opt-in content checksums through `redo-stamp`.
  It has the cluster's only preserve-on-hand-edit reaction:
   when a generated target is edited externally,
  it marks the file an override and skips the rebuild rather than clobbering it.
  file-enforcer takes the opposite stance on purpose,
   notify and regenerate,
   so this is a contrast,
   not a gap.
  Evidence:
   `/tmp/agent/fe-buildgraph-redo-20260615` at `7f00abc36be15f398fa3ecf9f4e5283509c34a00` (2021-07-27),
  `redo/state.py`,
   `redo/deps.py`.
  The implementation is effectively dormant:
   its last substantive commit is from 2021 and issues are disabled.
- `tup` (`gittup/tup`):
   the closest design relative in this cluster.
  It has the one native transform (`varsed`),
   real managed-destination detection that warns and overwrites on an
  externally modified generated file,
   a config-drift check in `tup refactor`,
   filesystem-monitored freshness,
  and a `tup monitor` watch mode.
  Its general build path still runs shell commands,
   and its destination reaction clobbers rather than preserves.
  Evidence:
   `/tmp/agent/fe-buildgraph-tup-20260615` at `2867b66e7105d432dce2609538117c1e6910bc73` (2026-03-18),
  `src/tup/varsed.c`,
   `src/tup/create_name_file.c`,
   `tup.1`.
- `wireit` (`google/wireit`):
   the closest npm-native peer.
  It wraps npm scripts with SHA-256 content-hash input fingerprinting,
   a persisted per-script fingerprint cache,
  per-target freshness,
   and a watch mode.
  Its content-hash input model and per-target manifest are a sharper version of file-enforcer's A5,
  worth studying for the per-rule incremental enhancement.
  It generates nothing itself;
   content comes from the npm-script command.
  Its output-manifest protection uses an mtime-and-size heuristic and reacts by clobbering.
  It has no check mode.
  Evidence:
   `/tmp/agent/fe-buildgraph-wireit-20260615` at `3ad0bbfb6b5bc5e5fbb21c12036023ae898e839f` (2026-06-12),
  `src/fingerprint.ts`,
   `src/execution/standard.ts`,
   `src/script-child-process.ts`.
- `turbo` (`vercel/turborepo`):
   a workspace task graph with git-based content hashing of inputs and an output cache.
  Inputs and outputs are declared per task as glob arrays in `turbo.json`.
  Its `--dry` is a plan preview that exits zero,
   not a drift gate,
   and it does not protect outputs:
  freshness comes from input hashes only,
   so a hand-edited output does not trigger a rerun.
  Its per-task input and output declaration and DAG are the incremental model file-enforcer,
   which re-runs its whole
  config,
   could borrow.
  Evidence:
   `/tmp/agent/fe-buildgraph-turbo-20260615` at `a562e78a4ee598670675d5b5cd72219ce0e3cfd0` (2026-06-15),
  `crates/turborepo-task-executor/src/command.rs`,
   `crates/turborepo-scm/src/hash_object.rs`,
  `package/turbo-types/schemas/schema.json`.

Axis scoring for the cluster,
 against file-enforcer's checklist:

- A1 native generation:
   none,
   except `tup` `varsed` narrowly.
  All run shell commands.
- A2 tracked source reads:
   yes,
   via explicit input declarations (`make` prerequisites,
   `wireit` `files`,
  `turbo` `inputs`) or filesystem monitoring (`tup`).
- A3 content-stable writes:
   no native transform writes,
   so not comparable;
   `redo`'s opt-in checksum stamp is the nearest
  parallel.
- A4 structured transforms:
   none.
- A5 incremental fingerprinting:
   yes,
   and stronger than file-enforcer in `wireit` and `turbo`.
- A6 watch with destination protection:
   `wireit`,
   `turbo`,
   and `tup` watch;
   destination protection exists in `wireit`
  and `tup` (clobber) and `redo` (preserve);
   `make` and `ninja` have neither.
- A7 check mode:
   only `make` `-q` is a true nonzero-on-stale gate;
   the others are plan previews or lists.
- A8 region management:
   none.
- A9 in-repo native integration:
   none;
   all need a separate build file and shell glue.

### Region and partial-file embedders

These sync generated content into a marked region of an otherwise hand-edited file.
That is exactly A8,
 the marker-region capability file-enforcer lacks for unstructured files,
and three of the four also implement A7.
They pass the gate weakly:
 `cog` manages arbitrary text files,
 the others are markdown-bound.
None is a file-enforcer replacement,
 because none owns A2,
 A5,
 or the structured-transform and programmable monorepo
model;
 their value to this audit is as reference implementations for the roadmap.

- `cog`,
   packaged as `cogapp` (`nedbat/cog`):
   the closest analog to the missing capabilities.
  It manages arbitrary text files,
   hiding generated regions inside any host language's comments through substring
  marker matching and common-prefix stripping.
  The region body is arbitrary Python,
   so generated content is fully programmable,
   not a fixed transform.
  It guarantees idempotency and skips when regenerated output matches.
  It implements A7 cleanly:
   `--check` regenerates without writing,
   sets a failure when the file would change,
  and exits with code five,
   with `--diff` to show the drift.
  It adds an opt-in `-c` output checksum that refuses to overwrite a hand-edited region,
   a tamper-evidence model
  stronger than file-enforcer's notify-after-the-fact.
  Evidence:
   `/tmp/agent/fe-region-cog-20260615` at `c0419cf618b46af4224994a0a82759c04e27531b` (2026-06-15),
  `cogapp/cogapp.py`,
   `cogapp/options.py`,
   `cogapp/hashhandler.py`.
  Axes:
   passes the gate weakly through arbitrary text,
   plus A1,
   A3,
   A7,
   and A8;
   lacks A2,
   A4,
   A5,
   A6,
   and A9.
- `doctoc` (`thlorenz/doctoc`):
   a single-purpose markdown table-of-contents generator.
  It earns a place because its `--dryrun` is a clean,
   well-tested A7 reference:
   on an out-of-date file it sets a nonzero
  exit code and writes nothing,
   and its region markers with skip-if-unchanged are exactly the A8 shape.
  Evidence:
   `/tmp/agent/fe-region-doctoc-20260615` at `eef394097f85ef41c43a5910b5f8f3e19da7da06` (2026-06-16),
  `doctoc.js`,
   `lib/transform.js`.
  Axes:
   passes A3,
   A7,
   and A8;
   lacks A1,
   A2,
   A4,
   A5,
   A6,
   and A9;
   the gate is markdown only.
- `embedme` (`zakhenry/embedme`):
   embeds source-file contents into markdown code fences.
  Its `--verify` exits nonzero on drift without writing,
   a minor A7 reference.
  It is markdown-fence bound and a fixed transform,
   and it is stale,
   last pushed in 2024,
   with a known
  false-negative idempotency bug,
   issue 109.
  Evidence:
   `/tmp/agent/fe-region-embedme-20260615` at `3cd8692de2c905cf3c9cbc6d87c1b4220f2e6eeb` (2024-10-07),
  `src/embedme.ts`,
   `src/embedme.lib.ts`.
  Axes:
   passes A3,
   A7,
   and A8;
   lacks A1,
   A2,
   A4,
   A5,
   A6,
   and A9;
   the gate is markdown only.
- `markdown-magic` (`DavidWells/markdown-magic`):
   transforms content between comment markers across several comment
  syntaxes,
   with pluggable transforms.
  It is the weakest of the four on the axes that matter here:
   it has no drift-failing check mode,
   and at the cloned
  head its file-write path is commented out,
   so write behavior could not be verified.
  Evidence:
   `/tmp/agent/fe-region-mdmagic-20260615` at `62b616e573882208412538bb62ded4e122af1673` (2026-06-15),
  `package/core/src/index.js`,
   `package/block-parser/src/syntax.js`.
  Axes:
   passes A8 and a partial A1;
   lacks A7,
   A2,
   A4,
   A5,
   A6,
   and A9;
   the write path is unverified at the cloned head.

### Dependency-update and vendoring pipelines

These passed the earlier audit as serious complements and are re-slotted here unchanged.
They pass the gate,
 in that they can write real files,
 but they target a different problem than root generation.
Their source citations and maintenance signals are inherited from the prior revision rather than re-deepened,
since they remain complements rather than replacement finalists.

- `updatecli` (`updatecli/updatecli`):
   the most important adjacent tool.
  It runs source,
   condition,
   and target stages to automate dependency,
   version,
   and configuration updates,
  with remote sources,
   Go-template and regexp transforms,
   dry-run diffs,
   unchanged-content skip,
  and source-control checkout,
   commit,
   and push.
  It is stronger than file-enforcer for dependency-update pull requests and source-control actions,
   and weaker for root
  generation because its config is manifest data,
   not arbitrary TypeScript with local imports.
  It has no equivalent for tracked reads,
   lazy staleness manifests,
   watch-mode destination protection,
   JetBrains
  settings patching,
   or package-index helpers.
  Evidence:
   `updatecli/updatecli` at `8deb2563286f8c0388fc594e0528e2fa2523060c` (2026-06-14),
  `pkg/plugins/resources/file/target.go`,
   `pkg/core/pipeline/target/main.go`.
  Axes:
   passes the gate,
   A3,
   and a dry-run diff adjacent to A7,
   with remote sources and source-control actions
  beyond the axes;
   lacks A1,
   A4,
   A5,
   A6,
   A8,
   and A9.
- `vendir` (`carvel-dev/vendir`):
   syncs vendored external directories into a tree,
   with a lock file and per-source
  checksums.
  It should be considered only if a future file-enforcer job is actually vendoring an external tree.
  It is not a general file generator.
  Evidence:
   `carvel-dev/vendir` at `a7fb189b2a1d1be30ccdf0049ae62b17d83240bc` (2026-06-03),
  `pkg/vendir/directory/directory.go`,
   `pkg/vendir/fetch`.
  Axes:
   passes the gate for vendored trees only,
   with A3 through a lock file and checksums;
  lacks A1,
   A2,
   A4,
   A6,
   A7,
   A8,
   and A9.
- `go-task` (`go-task/task`):
   the strongest task-runner overlap.
  Its fingerprint,
   sources,
   generated-file,
   checksum,
   timestamp,
   and status checks overlap file-enforcer on A5,
  and its status checks are an A7-adjacent concept.
  Generated content still lives in shell commands or side scripts,
   so it does not provide the TypeScript transform API,
  the tracked read-and-write model,
   atomic destination protection,
   or structured helpers.
  Evidence:
   `go-task/task` at `24a3ccdf42043a2cced5b24f67cefcf902995ef3` (2026-06-07),
  `internal/fingerprint/task.go`,
   `internal/fingerprint/status.go`,
   `watch.go`.
  Axes:
   passes the gate through shell commands,
   with strong A5 (fingerprint,
   checksum,
   timestamp,
   status) and A6 watch,
  and status checks adjacent to A7;
   lacks native A1,
   A4,
   A8,
   and A9.

## Audit finding

No screened candidate replaces file-enforcer for repo-local derived-file synchronization,
 and the wider sweep makes the
reason sharper than before.

- The gate eliminates every narrow-domain tool.
  Workspace-consistency enforcers manage `package.json` only;
   dotfile and sync tools target `$HOME` or another host;
  config languages emit one blob;
   config loaders write nothing.
  A tool that manages only its own workspace cannot manage `CLAUDE.md`,
   `mise.toml`,
   JetBrains XML,
   Browserslist
  output,
   and skill mirrors in one run.
- The general-target build tools clear the gate but fail the model.
  They generate content only through shell commands,
   with no native structured transforms,
   no tracked-read API in the
  generator,
   no managed-destination protection by default,
   and no in-repo TypeScript integration.
  Replacing file-enforcer with any of them means rebuilding A1,
   A2,
   A4,
   A6,
   and A8 as glue around a build runner.
- The region embedders own the two capabilities file-enforcer lacks but nothing else,
   so they are feature donors,
  not replacements.

So the incumbent stays.
This is not a defensive conclusion.
file-enforcer already holds seven of the nine axes plus the gate,
 in one TypeScript surface,
 with property and mutation
testing behind it.
The right move is to grow it into the two axes it lacks,
 using the cited reference implementations,
so it becomes the single tool for repo-local derived-file work rather than one tool among several.

## Absorption roadmap: toward one tool

Ordered by value.
Each item names the capability,
 the cited reference implementations,
 and the concrete fit against file-enforcer's
existing source and `TODO.md`.
The list is deliberately short.
Because the config is arbitrary TypeScript,
 axis A9,
 many capabilities other tools ship as engine features are already
expressible by writing code in the config,
 so they are not engine work.
Those are catalogued after the roadmap,
 not in it.

1.  Check or verify mode,
     axis A7.
    The highest-value addition,
     and the one genuinely non-trivial item on this list.
    For a write-only config,
     which the current root config is,
     it is cheap:
     `overwrite()`,
     `overwriteEach()`,
     and the
    lazy builders all route through `writeFileAtomically`,
     and `overwrite()` already reads the destination through
    `readExisting()` and logs `skip (unchanged)` when content matches.
    So for that case a check mode is that comparison with the write suppressed,
     a nonzero exit on any mismatch,
     and an
    optional diff,
     reusing the destination hashes already in the staleness manifest.
    Evidence:
     `src/io/write.ts`,
     `src/io/write-atomic.ts`,
     `src/io/staleness-hash.ts`.
    The catch,
     and the reason `TODO.md` calls this descriptor-pattern work,
     is that the public API also has side
    effects a write-wrapper cannot neutralize:
     `exec()` runs commands,
     `ensurePackage()` installs system packages under
    `sudo`,
     and `notify` sends desktop notifications.
    A config that calls those cannot be checked safely by suppressing writes alone.
    Evidence:
     `src/pipeline/exec.ts`,
     `src/package/ensure-package.ts`.
    So the honest scope is a check mode for write-only configs,
     cheap and high-value,
     with configs that use the
    side-effecting API out of scope unless a descriptor layer is added later.
    This item is hard for the same reason axis A9 is powerful:
     the engine cannot reason about an opaque arbitrary-code
    rule without running its effects.
    Reference implementations:
     `make` `-q` with its zero-or-nonzero exit,
     `cog` `--check` with `--diff`,
    and `doctoc` `--dryrun`.
    Payoff:
     continuous integration can assert that every write-only derived file is in sync and fail the build on
    drift,
     which the repo cannot do today.

2.  Polling watch fallback for unreliable filesystem backends.
    A must-have,
     not a niche item.
    `fs.watch` misses events on some backends,
     notably NFS and FUSE mounts,
     so watch mode can silently stop
    reconciling without any error.
    `TODO.md` already lists this under watch-mode reliability,
     and `watchexec` demonstrates a cross-platform polling
    fallback.
    Fit:
     a poll-based watcher selected when the native backend is known-unreliable or when expected events stop
    arriving,
     behind the existing watch supervisor.
    Evidence:
     `package/dev-script/file-enforcer/TODO.md`,
     `src/watch/watch-supervisor.ts`.

3.  Marker-region management in unstructured text,
     axis A8.
    The blunt version of this gap does not exist:
     file-enforcer already does partial,
     key-scoped replacement for
    structured formats,
     the TOML splice that keeps unmutated regions byte-identical (`overwriteTomlKey`,
    `editTomlKey`),
     the XML entry replace-or-insert (`replaceOrInsertXmlEntry`),
     and the JSON key merge
    (`mergeFlatJson`),
     so generated and hand-edited content already coexist in those files,
     scoped by structure.
    Evidence:
     `src/io/write-toml.ts`,
     `src/pipeline/xml.ts`,
     `src/pipeline/json.ts`.
    The residual gap is the unstructured case:
     a generated slice inside a hand-written file with no key path to scope
    the edit,
     such as a section of a prose or markdown document,
     which only an explicit text marker can delimit.
    This generalizes the `appendTo` and `prependTo` operations `TODO.md` lists as missing.
    The current root config needs none of this,
     since its generated files are whole-file,
     so this is worth building
    only when a real job must place a generated slice inside a hand-edited file.
    Mechanically this is a small string splice,
     not a new engine:
     scan for the begin and end markers and replace the
    slice between them,
     the way the package's existing partial-edit primitives use a parser or string `indexOf` and
    `slice` rather than a regex,
     which the repo's `no-restricted-syntax/no-regex` policy disfavors anyway
    (`doc/handover/no-regex.md`).
    The matching is the easy part.
    The substance is the syntax boundary:
     the generated payload can itself contain the end marker,
     so a naive regex and
    a naive `indexOf` both corrupt the region on the next run.
    Handling that,
     by choosing markers the payload cannot contain,
     encoding the payload,
     or a tamper-evident checksum,
    is the work,
     and it is exactly the cross-syntax-boundary concern the repo's own rules call out.
    Design reference:
     `cog`,
     whose opt-in checksum refuses to overwrite a hand-edited region and whose idempotency
    guarantee addresses precisely this;
     `doctoc` and `embedme` show the simpler marker-and-verify shape.
    Fit:
     a string-splice write helper that reuses the existing content-stable write path and skips when unchanged,
    with adversarial boundary tests for markers appearing in the payload.

4.  Per-rule incremental rerun,
     an enhancement of axis A5.
    `TODO.md` already proposes tagging rules with their source paths so only affected rules rerun,
     and notes that warm
    full reruns are fast enough that this is low priority.
    The build-graph cluster shows the mature form:
     `wireit`'s per-target content-hash fingerprint and `turbo`'s
    per-task input and output globs.
    file-enforcer already hashes source sets and destinations in its manifest,
     so this is an enhancement of existing
    machinery,
     not new infrastructure.
    Evidence:
     `src/io/staleness-hash.ts`,
     `src/io/staleness-manifest.ts`.

### Already covered, not roadmap items

Several capabilities other tools ship as engine features are not file-enforcer gaps,
because the config is arbitrary TypeScript with top-level `await` and local imports.
They are listed here so the roadmap's short length reads as a deliberate result,
 not an oversight.

- Remote sources.
  A generator can call `fetch()` in the config directly,
   and the content-stable write path already detects when the
  fetched content changed,
   so there is nothing for the engine to add.
  This is `updatecli`'s remote-source feature,
   made unnecessary by axis A9.
- Schema validation of generated structured output.
  Import a schema library into the config,
   `valibot` for example,
   and validate the value before the write lands.
  This is the `cue` and `pkl` validation argument,
   expressed in-config rather than built into the tool.
- Vendoring external trees.
  `mise` vendors tools and `pnpm` vendors packages,
   both with lock files and integrity hashes,
   so `vendir`'s job is
  already done for the dependency shapes this repo has.
  Its one unique niche,
   vendoring an arbitrary non-package directory,
   is not a need here;
   if it arose,
   a `fetch()` and
  a checksum in the config would cover it.
- Dependency-update pull requests.
  `updatecli`'s source-control automation checks out a branch,
   commits the computed file change,
   pushes,
   and opens a
  pull request,
   the way Renovate or Dependabot do.
  That is an autonomous update-bot problem,
   not derived-file reconciliation,
   so it stays `updatecli`'s domain as a
  complement.
- Git-hook installation.
  Git hooks are being deprecated in this repo,
   so the check mode runs in continuous integration,
   not on commit,
  and there is no hook to install.
  Evidence:
   `doc/decision/cli-git-policies-platform.md`.

### Out of audit scope: engine hardening and scale

This audit ranked capabilities to absorb from peer tools.
A separate class of work has no peer donor,
 so it does not appear in the roadmap above,
 yet it is not small.
A short absorption roadmap should not be read as an empty backlog.
`TODO.md` already tracks it,
 and at least the last item is architectural rather than a helper:

- cache correctness when more than one external tool edits a source between reruns,
   where only the last edit is seen;
- unbounded in-memory cache growth on very large monorepos,
   with no eviction strategy;
- graceful shutdown on `SIGINT` and `SIGTERM`,
   so watchers and abort controllers are released cleanly;
- multiple config files,
   for example per-package configs merged at the root.

Evidence:
 `package/dev-script/file-enforcer/TODO.md`.

### What file-enforcer should not absorb

The gate cuts both ways.
file-enforcer's moat is general-target reconciliation in native TypeScript,
 and chasing capabilities that betray that
focus would weaken it.

- Not home-directory dotfile deployment.
  That is `chezmoi`'s target and a different problem;
   the worktree is the boundary.
- Not project scaffolding or initialization.
  Scaffolders run once with a different lifecycle;
   reconciliation is the point here.
- Not a general task runner.
  `mise`,
   `just`,
   and `go-task` already run tasks,
   and file-enforcer is invoked by `mise`,
   not a replacement for it.
- Not a built-in `package.json` consistency mode.
  This is the sharpest lesson of the workspace cluster.
  file-enforcer's generality already lets a config express cross-package `package.json` invariants as ordinary
  TypeScript,
   reading every manifest and writing the ones that drift.
  Baking in a fixed-schema mode like `syncpack` would trade the generality that is its whole advantage for a narrow
  feature it can already express.
  The cluster that tempted this audit as a peer is exactly the shape file-enforcer should not become.
- Not a configurable managed-destination reaction policy.
  `redo` preserves a hand-edited output and skips regeneration;
   file-enforcer notifies and regenerates on purpose,
  because a managed destination is owned by its generator,
   so an external edit to it is drift to correct,
  not content to keep.

## Maintenance snapshots

Sampled on 2026-06-15 unless noted.
Star counts and release dates are point-in-time signals,
 not endorsements.

Tier-three and feature-donor tools:

- `cog` / `cogapp`:
   406 stars,
   PyPI release 3.6.0,
   active,
   a single large test file,
   no fuzz or mutation harness.
- `doctoc`:
   4,449 stars,
   release v2.5.0 on 2026-06-12,
   very active,
   no fuzz or mutation harness.
- `embedme`:
   238 stars,
   last pushed 2024-10,
   stale,
   a known idempotency bug open,
   no fuzz harness.
- `markdown-magic`:
   864 stars,
   release 4.9.0 on 2026-05-30,
   write path disabled at the cloned head,
   no fuzz harness.
- `wireit`:
   6,410 stars,
   npm release 0.14.12 on 2025-04-10,
   actively merged,
   no fuzz or mutation harness.
- `turbo`:
   30,548 stars,
   release v2.9.18 on 2026-06-10,
   daily activity,
   `quickcheck` in one terminal crate,
  no dedicated fuzz directory found.
- `ninja`:
   13,020 stars,
   release v1.13.2 on 2025-11-20,
   active,
   manifest-parser fuzzing present.
- `redo`:
   1,845 stars,
   dormant since 2021,
   issues disabled,
   no fuzz harness.
- `tup`:
   1,255 stars,
   actively maintained in 2026,
   no fuzz harness in tup's own source.
- `make`:
   GNU make 4.4.1 from 2023-02,
   foundational and stable,
   fuzz posture not assessed at documentation tier.
- `updatecli`:
   930 stars,
   release v0.118.0 on 2026-06-02,
   maintainer-active,
   no broad fuzz,
   property,
   or mutation
  harness found.
- `vendir`:
   373 stars,
   release v0.46.0 on 2026-06-10,
   maintainer-active on sampled pull requests,
   no fuzz or mutation
  harness.
- `go-task`:
   maintainer comments and reviews on sampled issues and pull requests,
   no Go fuzz,
   property,
   or mutation
  harness found.

Narrow-domain cluster,
 recorded so the dismissal is not mistaken for staleness:

- `syncpack`:
   2,063 stars,
   release 15.3.2 on 2026-06-15,
   highly responsive owner,
   no fuzz,
   property,
   or mutation
  harness.
- `manypkg`:
   1,036 stars,
   `@manypkg/cli` 0.25.1,
   active in 2026,
   no fuzz harness.
- `sherif`:
   1,164 stars,
   release v1.11.1 on 2026-03-30,
   active owner,
   snapshot tests but no property or fuzz harness.
- Yarn constraints:
   part of `yarnpkg/berry`,
   8,078 stars on the monorepo,
   actively pushed.
- `knip`:
   11,487 stars,
   release 6.16.1 on 2026-06-06,
   active owner,
   no fuzz harness.

## Evidence and provenance

New clones from this revision,
 under `/tmp/agent/`,
 shallow unless history was needed:

- `JamieMason/syncpack`:
   `fe-wsconsist-syncpack-20260615` at `3b2c99d793975d54438f12da4e595a803afe5b8b`,
   2026-06-15.
- `Thinkmill/manypkg`:
   `fe-wsconsist-manypkg-20260615` at `97e2ab9bc64e45af099f63f25c515f3792738cfb`,
   2026-06-10.
- `QuiiBz/sherif`:
   `fe-wsconsist-sherif-20260615` at `f1e6490a681165db74ce38e91dc984d8c94d64eb`,
   2026-04-24.
- `webpro-nl/knip`:
   `fe-wsconsist-knip-20260615` at `e265d281b031783dd7f92dfc0db29f60f7138d5b`,
   2026-06-15.
- `nedbat/cog`:
   `fe-region-cog-20260615` at `c0419cf618b46af4224994a0a82759c04e27531b`,
   2026-06-15.
- `zakhenry/embedme`:
   `fe-region-embedme-20260615` at `3cd8692de2c905cf3c9cbc6d87c1b4220f2e6eeb`,
   2024-10-07.
- `DavidWells/markdown-magic`:
   `fe-region-mdmagic-20260615` at `62b616e573882208412538bb62ded4e122af1673`,
   2026-06-15.
- `thlorenz/doctoc`:
   `fe-region-doctoc-20260615` at `eef394097f85ef41c43a5910b5f8f3e19da7da06`,
   2026-06-16.
- `google/wireit`:
   `fe-buildgraph-wireit-20260615` at `3ad0bbfb6b5bc5e5fbb21c12036023ae898e839f`,
   2026-06-12.
- `vercel/turborepo`:
   `fe-buildgraph-turbo-20260615` at `a562e78a4ee598670675d5b5cd72219ce0e3cfd0`,
   2026-06-15.
- `ninja-build/ninja`:
   `fe-buildgraph-ninja-20260615` at `f735970600b5713276583589dce207d575950b14`,
   2026-06-14.
- `apenwarr/redo`:
   `fe-buildgraph-redo-20260615` at `7f00abc36be15f398fa3ecf9f4e5283509c34a00`,
   2021-07-27.
- `gittup/tup`:
   `fe-buildgraph-tup-20260615` at `2867b66e7105d432dce2609538117c1e6910bc73`,
   2026-03-18.

Earlier-revision clones retained for the re-slotted tools,
 under `/tmp/agent/file-enforcer-audit-20260615/`:
`updatecli/updatecli` at `8deb2563286f8c0388fc594e0528e2fa2523060c`,
`carvel-dev/vendir` at `a7fb189b2a1d1be30ccdf0049ae62b17d83240bc`,
`go-task/task` at `24a3ccdf42043a2cced5b24f67cefcf902995ef3`,
plus the configuration-language,
 scaffolder,
 dotfile,
 and data-editor clones cited inline above.

Documentation-tier verification,
 no clone,
 was used for the Tier-one and Tier-two structural dismissals and for GNU
make,
 each checked against the tool's README or official documentation and cited inline.

This document is an audit,
 not a migration decision.
If a future task adopts a specific tool or absorbs a specific capability,
 record that under `doc/decision/`,
and for any absorbed capability,
 vet the reference implementation's integration boundary in a disposable fixture before
landing it.
