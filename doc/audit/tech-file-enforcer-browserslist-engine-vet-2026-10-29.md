# File-enforcer browserslist engine vet

Superseded 2026-10-29 before a verdict:
the user approved `browserslist-rs` without a completed vet
("I'm approving toml_edit, browserslist-rs, schemars as-is."),
recorded in `doc/decision/file-enforcer-rust-rewrite.md`, "Consequences".
This halted draft is kept whole as knowledge ("Even if work is superseded, it should still become knowledge.", user).
Its findings stay open facts for the implementation:
`browserslist-rs` 0.21.2 hard-codes Firefox ESR as `firefox 140` while npm `browserslist` 4.29.5 gives `firefox 153`;
section selection is not public API, so file-enforcer parses `.browserslistrc` itself and calls `resolve`;
`browserslist-data` 0.3.1 was published from a dirty working tree;
and `oxc-browserslist` 5.0.1 survived screening as an alternative.

Status:
 in progress,
 halted 2026-10-29 for handoff.
The coordinator stopped this audit before finalist validation was complete.
No recommendation is made.
A later agent continues from the "Steps not yet done" section.
Nothing was committed (the task forbade commits),
 so the skill's "update and commit the scoped report" steps are recorded here as not performed.

Lifecycle phase:
 serious alternatives screened,
 hard-gate evidence partly collected,
 finalist validation started (source-to-artifact mapping done for two candidates,
 container runs prepared but not executed).

Subject:
 file-enforcer browserslist engine.

Decision scope:
 select the Rust crate that resolves `.browserslistrc` production queries
 into `.browserslistrc.resolved.local.json` for the Rust file-enforcer.

Start date:
 2026-10-29.

Last updated:
 2026-10-29.

Governing skill path:
 `.agents/skills/choosing-technology/SKILL.md`
 (mirrored to `.claude/skills/choosing-technology/SKILL.md`).

Governing skill commit:
`37117e36397e30233062828581a5e38217da9458`
 (`git log -1 -- .agents/skills/choosing-technology/SKILL.md`, 2026-09-29).

Governing skill SHA-256:
`552ac9955299b65a9f921a4e836b60a3fabc15d3477eeb8ec2bfb3f400647f9b`
 (identical for the `.agents` and `.claude` copies).

Compatibility fingerprint:
`1ea93c29b25d0c0eed1c05ec9c57d27d018afa4f74af4617226dc583717d0edd`.
Input (JSON, keys sorted, compact, SHA-256 over the UTF-8 bytes):

```json
// fingerprint input
{"baseCategories":["inspectable open-source local technology"],"decisionScope":"Select the Rust crate that resolves .browserslistrc production queries into .browserslistrc.resolved.local.json for the Rust file-enforcer","deployment":{"ci":"github-actions","platforms":["linux-x64"],"runtime":"native binary"},"hardConstraints":["Rust crate usable in-process from the Rust file-enforcer (owner choice, no Node at generation time)","MIT-compatible license","Resolves the queries in the repository .browserslistrc","Output consumable by package/config/rolldown/src/browserslist-targets.ts","Config and stats discovery disabled","Dependencies from rust-catalog.toml with workspace = true"],"incumbent":{"name":"npm browserslist via file-enforcer.config.ts","version":"4.29.5"},"overlays":["incumbent dependency replacement","native, Wasm, prebuilt binary, or generated-code boundary","multi-platform or browser-baseline claim"],"schemaVersion":1,"subject":"file-enforcer browserslist engine","trustBoundary":"build-time tool reading repository files and writing one gitignored JSON file; bundled generated browser data"}
```

Active audit owner:
 the vet subagent session of 2026-10-29,
 now halted.
Its report lock was released after the write; the next agent takes a new lock per the skill.

Prior compatible report:
 none.
`rg` over `doc/audit` for the slug finds no earlier file.
The nearby 2026-10-29 reports for the TOML editor and the XML editor
 (`doc/audit/tech-file-enforcer-toml-editor-vet-2026-10-29.md`,
 `doc/audit/tech-file-enforcer-xml-editor-vet-2026-10-29.md`)
 are other subjects.

Unresolved preferences:
 one candidate question is held until sensitivity shows it controls the order (see "Open question").

## Context

The accepted decision `doc/decision/file-enforcer-rust-rewrite.md` (2026-10-29) lists
 `.browserslistrc.resolved.local.json` "through `browserslist-rs`"
 and gates implementation on a choosing-technology vet of `browserslist-rs`
 (section "Consequences", gates before implementation).
The owner chose a Rust rule kind over resolving live in rolldown:
 "But rolldown needs to go fast in the common case and we don't have 10ms to wait every build."
 (`doc/planning/file-enforcer-rust-rewrite.md`, section "browserslist: the generated file stays").
The owner accepted that the bundled browser data versions with the crate, not the pnpm lock.

Measured facts about the incumbent:

- Generator: `file-enforcer.config.ts:936-967` (function `generateResolvedBrowserslistTargets`).
  It parses `.browserslistrc` with `browserslist.parseConfig`,
  picks `config.production ?? config.defaults` (`selectBrowserslistQueries`, `file-enforcer.config.ts:659`),
  and calls `browserslist(queries, { path: false, stats: {} })`.
- `.browserslistrc` has no section header, so the whole file is the `defaults` group.
  Its queries are `last 1 Chrome versions`,
  `last 1 ChromeAndroid versions`,
  `last 1 Android versions`,
  `Firefox ESR`,
  `last 1 FirefoxAndroid versions`,
  `last 1 node major versions`,
  and `not dead`.
- Current output (18 entries):
  `and_chr 156`,
  `and_ff 158`,
  `android 156`,
  `chrome 156`,
  `firefox 153`,
  and thirteen `node 26.x.0` entries.
- Installed npm data in this repository:
  `browserslist@4.29.5`,
  `caniuse-lite@1.0.30001819`,
  `electron-to-chromium@1.5.467`,
  `node-releases@2.0.60`,
  `baseline-browser-mapping@2.11.34`
  (`node_modules/.pnpm`).
- Only consumer: `package/config/rolldown/src/browserslist-targets.ts`.
  It splits each entry at the first space,
  maps the browser id through `ROLLDOWN_BROWSER_NAMES`
  (`and_chr`,
  `and_ff`,
  `android`,
  `chrome`,
  `edge`,
  `firefox`,
  `ios`,
  `ios_saf`,
  `node`,
  `op_mob`,
  `opera`,
  `safari`,
  `samsung`),
  throws on any other id,
  and keeps the lowest version per family.
- Dependency policy: `rust-catalog.toml` has `minimumReleaseAge = 1440` minutes
 and `trustPolicy = "no-downgrade"`.
  Crates enter through the catalog and are used with `workspace = true`.

## Hard constraints

- H1:
 the engine is a Rust crate usable in-process from the Rust file-enforcer
 (owner choice; the TypeScript incumbent that needs Node is kept in the ledger and exits on this constraint).
- H2:
 the license is compatible with the repository.
- H3:
 every query in `.browserslistrc` resolves without error.
- H4:
 the output array is consumable by `browserslist-targets.ts`
 (browser ids all present in `ROLLDOWN_BROWSER_NAMES`).
- H5:
 config discovery and stats discovery are disabled,
 so the result depends only on the committed `.browserslistrc` and the engine's bundled data.
- H6:
 build provenance of the bundled data maps to source.

Soft criteria (frozen when scoring starts):
 not yet frozen;
 the planned criteria are listed under "Steps not yet done".

## Base components and overlays

- Base: inspectable open-source local technology (every Rust crate candidate).
- Overlay: incumbent dependency replacement (the npm `browserslist` call).
- Overlay: generated-code and prebuilt-data boundary (bundled browser data compiled into the crate).
- Overlay: browser-baseline claim (the output defines build targets).
- Human-auditability overlay: the tool is repository-owned build tooling that runs on developer machines and in CI,
  not a plugin, hook, or credential holder.
  Marked partly applicable: the crate parses text from a checked-in file only,
  but `browserslist-rs` can spawn `node` for an `extends` query (see "Source findings").
  The line-count and dependency measurements below serve that overlay.

## Query schedule and discovery ledger

Schedule recorded after the fact for the first crates.io round
 (the run happened before the schedule was written down;
 the literal queries and results are in the log
 `/var/home/user/temp/agent/bl-vet-crates-queries-2026-10-29.log`).
This is a deviation from "freeze before executing";
 the next agent should treat the crates.io class as run but re-confirm that no query was chosen after seeing results.

### Source class 1: category package registry (crates.io API)

Provider: `https://crates.io/api/v1/crates`,
 `per_page=50`,
 no include or exclude filters,
 pages 1 to 3,
 sort `relevance` and sort `downloads` for each query.
Queries and totals:

- `browserslist`: 17 results,
 page 2 and 3 empty (provider exhausted).
- `caniuse`: 5 results,
 exhausted.
- `caniuse-lite`: 0 results.
- `browser targets`: 7991 results,
 pages 1 to 3 inspected (150 rows per sort).
- `browserslist rust`: 13 results,
 exhausted.
- `browser support query`: 2827 results,
 pages 1 to 3 inspected.

Screening reads were by name and description.
That filter can hide a crate whose name and description omit browserslist and caniuse words.
For the two broad queries the pages held only unrelated crates
 (terminal UI crates, sparse matrix solvers),
 so pages 2 and 3 added no survivor
 and the two-empty-pages rule is met for them.
The cap of three pages is therefore a stop by the saturation rule,
 not by a provider cap,
 for those two queries.

Rows that matched the topic:

- `browserslist-rs` 0.21.2 (2026-09-16).
- `browserslist-data` 0.3.1 (data half of `browserslist-rs`).
- `oxc-browserslist` 5.0.1 (2026-08-12).
- `rspack_browserslist` 0.103.0 (2026-10-29).
- `lightningcss` 1.0.0-alpha.72 (CSS tool whose `browserslist` feature wraps a browserslist engine).
- `css_feature_data`,
 `omena-transform-target`,
 `nargo-config`,
 `grimoire_css`,
 `parcel_css`,
 `vize_lightningcss`,
 `lightningcss_rs`,
 `sort-package-json`,
 `turf`,
 `css_build_system`,
 `omena-wasm`,
 `omena-napi`
 (CSS or unrelated tools that mention browserslist; none exposes a standalone query engine).
- `caniuse`,
 `caniuse-db`,
 `caniuse-serde`,
 `caniuse-rs`,
 `css-autoprefix`
 (last updated 2016 to 2021; data-only or CLI crates, no query engine).

### Source class 2: repository host (GitHub)

Not yet run.
Planned literals: `gh search repos browserslist --language rust`,
 `gh search code browserslist --language rust --filename Cargo.toml`,
 and the `browserslist` and `oxc-project` organization repository lists.
Evidence so far from GitHub:
 `browserslist/browserslist-rs` (152 stars,
 4 open issues,
 not archived,
 pushed 2026-10-24)
 and `oxc-project/oxc-browserslist` (62 stars,
 2 open issues,
 not archived,
 pushed 2026-10-24).

### Source class 3: broader web

Not yet run.
Planned literals: `rust browserslist crate`,
 `browserslist rust alternative`,
 `oxc-browserslist vs browserslist-rs`,
 `docs.rs browserslist`.

### Source class 4: this repository

Partly run.
`rg` over `rust-catalog.toml` and the workspace found no existing Rust browserslist dependency
 (no match for browserslist in the catalog head).
The incumbent and the decision records were read (see "Context").
Still to run: `rg browserslist` over `doc/audit`,
 `doc/decision`,
 every `Cargo.toml`,
 and `Cargo.lock` files.

### Expansion round

Not yet run.
Taxonomy terms collected so far: `caniuse`,
 `baseline`,
 `lightningcss targets`.

### Discovery result

Not saturated:
 classes 2 and 3 are not run and class 4 is partial.
Candidates so far are listed below.

### Candidate ledger

#### browserslist-rs 0.21.2 (owner's choice)

- Source: crates.io search `browserslist`.
- Base: inspectable open-source local technology.
 Overlays: replacement, generated-data boundary,
 browser-baseline.
- Screening: pass.

#### oxc-browserslist 5.0.1

- Source: crates.io search `browserslist`.
- A fork of `browserslist-rs` by the oxc project (README:
 "Rust port of Browserslist, forked from browserslist-rs").
 It is a standalone Rust engine,
 so it is a category survivor,
 not a mismatch.
- Differences stated in its README:
 no config-file loading,
 no `BROWSERSLIST*` environment variables,
 no `extends`,
 no custom usage;
 fewer dependencies (`nom`,
 `chrono`,
 `ahash`,
 `serde_json`,
 and others removed);
 588.3K versus 3.2M binary size as claimed by its authors (not measured here).
- Screening: pass.
 The unsupported features are unused by this repository,
 but file-enforcer must then parse `.browserslistrc` itself
 (it must also do so for `browserslist-rs`; see "Source findings").

#### rspack_browserslist 0.103.0

- Source: crates.io search `browserslist`.
- Dependencies from the crates.io API: `browserslist-rs ^0.21.2`,
 `lightningcss ^1.0.0-alpha.72`,
 `regex ^1.13.1`.
 It is a thin downstream wrapper (11.6 KB crate) over `browserslist-rs`.
- Screening result: exit as category mismatch with the decision (a wrapper, not an independent engine;
 it brings the same data and adds `lightningcss`).
 Its `src/lib.rs` was not read.
 The advisor asked to read it before closing the exit,
 so this exit is provisional until that read.

#### lightningcss (browserslist feature)

- Source: crates.io search `browserslist`.
- A CSS transformer that consumes an engine;
 category mismatch.
 Its engine is `browserslist-rs`,
 so it adds no separate engine.
 Exit.

#### Incumbent: npm browserslist 4.29.5 called from `file-enforcer.config.ts`

- Source: repository incumbent.
- Kept in the ledger per the replacement overlay.
- Screening: exits on H1 (owner's Rust constraint).
 The Rust file-enforcer would need Node and an installed npm tree at generation time.
 It remains the reference oracle for output comparison.

#### Hand-written resolver

- Not a candidate:
 the skill allows a custom implementation only when every existing tool fails a named hard constraint.

## Evidence records

Notation:
 status is pass,
 fail,
 pending,
 or finding.
 Access date for every URL and registry call is 2026-10-29.

### E1: browserslist-rs 0.21.2 release facts

- Claim: version,
 date,
 license,
 owner,
 publish route.
- Source: `https://crates.io/api/v1/crates/browserslist-rs` and `/owners`.
- Result: 0.21.2 published 2026-09-16,
 MIT,
 checksum `8ca3b73a7c87da0bcca071227be2c28527af6ecd927060a1d75beef8bc4868b8`,
 downloaded crate matches (`sha256sum`).
 Published by GitHub trusted publishing,
 run 35071591549,
 sha `7503606a19d9503c64738c827ee8d8657133e383`.
 One crates.io owner:
 the user `g-plane` (Pig Fang),
 although the repository sits under the `browserslist` organization.
 Earlier releases 0.19.0 (2025-07-02),
 0.20.0 (2026-08-01).
- Status: pass for license,
 finding for owner concentration (single crates.io owner).

### E2: browserslist-rs 0.21.2 source equals tag

- Clone: `/var/home/user/temp/agent/browserslist-rs-2026-10-29`
 (full history after `git fetch --unshallow --tags`),
 HEAD `716680cf612cf300d4d9fcacbd1d2c0db8567872` (2026-10-24).
- Tag tree exported to `/var/home/user/temp/agent/bl-tag-0.21.2` from `7503606`.
- Command: `cmp` of every `src/` and `examples/` file of the extracted crate
 (`/var/home/user/temp/agent/crate-dl-bl-2026-10-29/x1/browserslist-rs-0.21.2`) against the tag tree.
- Result: same file list,
 zero differences.
 `.cargo_vcs_info.json` records sha `7503606` and no dirty flag.
 `build = false` in the packaged `Cargo.toml` (no `build.rs`).
- Status: pass.

### E3: browserslist-data 0.3.1 is generated at publish time (provenance finding)

- Source: crates.io API for `browserslist-data`;
 `.github/workflows/ci.yml` at `7503606`;
 `data/src/generated/.gitignore` at `7503606`;
 `generate-data/src/main.rs` at `7503606`.
- Result:
 the data crate 0.3.1 (checksum `58f90c231428ed81dbb8f39dcc14021335c3cc4f46ffdffde278638ca832b579`,
 downloaded and matched) has `.cargo_vcs_info.json` with `"dirty": true`.
 In the repository the `generated` directory holds only a `.gitignore`.
 CI runs `pnpm install` (action `pnpm/action-setup@v4.0.0` with `version: latest`),
 `cargo run -p generate-data`,
 the tests,
 and then `cargo publish -p browserslist-data --allow-dirty`
 (`ci.yml` job `test`, steps in that order).
 The bundled data is therefore built in CI from npm packages at the versions in `pnpm-lock.yaml`:
 `caniuse-lite 1.0.30001810`,
 `electron-to-chromium 1.5.428`,
 `node-releases 2.0.55`,
 `baseline-browser-mapping 2.11.23`,
 `browserslist 4.28.9`.
 The published `info.txt` is 0 bytes because the generator writes the output of `git submodule`
 (`main.rs`, function `build_info`) and the repository has no submodules.
 Lock integrity values for those five packages match the registry (checked with `npm view dist.integrity`
 and with the installed `node_modules/.package-lock.json`).
 No Cargo.lock is committed in the repository (`.gitignore`), so CI resolves transitive crates at run time.
- Status: finding.
 Source-to-artifact mapping is possible (lock plus `generate-data`) but is not yet verified by regeneration.
 It is a hard gate (H6) until the regeneration comparison passes.

### E4: browserslist-rs query behavior from source (answers the caller's questions)

All from the extracted crate 0.21.2,
 `/var/home/user/temp/agent/crate-dl-bl-2026-10-29/x1/browserslist-rs-0.21.2/src/`.

- Low-level API `resolve(queries, &Opts)` (`lib.rs`, fn `resolve`, lines 78-120):
 takes strings only,
 reads no file and no environment variable itself.
- High-level API `execute(&Opts)` (`lib.rs:153-159`) calls `config::load`.
 The module is declared `mod config;` (private),
 so the config parser is not reachable from outside the crate.
 `config::load` (`config/mod.rs:52-`) returns the `BROWSERSLIST` environment variable first when set,
 then `opts.config` or `BROWSERSLIST_CONFIG`,
 then searches the directory for `browserslist`,
 `.browserslistrc`,
 and `package.json`.
 `get_env` (`config/mod.rs:182-188`) takes `opts.env`,
 then `BROWSERSLIST_ENV`,
 then `NODE_ENV`,
 then `"production"`.
 `pick_queries_by_env` falls back to `defaults` unless `throw_on_missing`.
- Consequence for env selection:
 to select the production section with ambient variables unable to change the result,
 file-enforcer must parse `.browserslistrc` itself
 (a few dozen lines: `#` comments,
 `[section]` headers,
 queries before the first header belong to `defaults`)
 and call `resolve(queries, &Opts::default())`.
 Using `execute` with `Opts { env: Some("production"), config: Some(..) }` would still let a stray `BROWSERSLIST`
 variable override everything.
 This mirrors the incumbent's `parseConfig` plus `selectBrowserslistQueries`.
- Disabling discovery:
 `resolve` has no discovery.
 Stats discovery does not exist (custom usage `in my stats` is unsupported by the README,
 section "Limitations", and `percentage.rs` reads only global data).
- `extends` query: `queries/extends.rs:19-45` spawns `node -p "JSON.stringify(require('<pkg>'))"` when a query starts with `extends`
 (the name is checked against a `browserslist-config-` prefix unless `dangerous_extend` or
 `BROWSERSLIST_DANGEROUS_EXTEND` is set).
 The repository `.browserslistrc` has no `extends`,
 and a Rust caller can reject such lines before calling `resolve`.
 This is a process-spawn path inside a library that otherwise has none.
- Query syntax used by `.browserslistrc`, mapped to files under `src/queries/`:
 `last N X versions` (X = Chrome,
 ChromeAndroid,
 Android,
 FirefoxAndroid) to `last_n_x_browsers.rs`;
 `Firefox ESR` to `firefox_esr.rs`;
 `last N node major versions` to `last_n_node_major.rs`;
 `not dead` to `dead.rs`.
 Each has test cases in its file
 (`last_n_x_browsers.rs` includes android and alias cases;
 `firefox_esr.rs` lists the `firefox esr`,
 `ff esr`,
 `fx esr` forms).
 Whether the aliases `ChromeAndroid` and `FirefoxAndroid` resolve in 0.21.2 is not yet run
 (the probe agent covers the real config).
- Rust-side dependencies of the result type: `Distrib::to_string()` gives `name version`
 (`lib.rs` rustdoc), the same shape the consumer splits on.

Status: pass for env selection (with caller-side parsing),
 pass for discovery being off via `resolve`,
 pending for the exact aliases.

### E5: Firefox ESR is hard-coded stale in 0.21.2 (material finding)

- `src/queries/firefox_esr.rs` in 0.21.2 returns `Distrib::new("firefox", "140")`.
 Repository HEAD (`716680c`,
 commit "derive Firefox ESR version from browserslist data (#58)", 2026-10-24) replaces it with a generated constant
 `browserslist_data::FIREFOX_ESR_VERSION`,
 filled by `generate-data` from `require('./node_modules/browserslist').versionAliases.firefox.esr`.
 That change is not in any published crate (the latest crates.io version is 0.21.2).
- Incumbent: `node_modules/browserslist/index.js:15` sets `FIREFOX_ESR_VERSION = '153'` (browserslist 4.29.5).
- Effect: the incumbent's file has `firefox 153`;
 the crate would emit `firefox 140`.
 `browserslist-targets.ts` keeps the lowest version per family,
 so the Rolldown target would drop from `firefox153` to `firefox140`
 (more downleveling than the owner has today).
 This is a version-currency difference of the kind the owner accepted ("data versions with the crate"),
 but the ESR value is source code, not data,
 so it does not even follow the crate's own data release.
 A published fix requires upstream to release after 2026-10-24.
- Status: finding (scored concern for data currency;
 not a hard failure unless the owner treats exact equality with the incumbent as a requirement).
 To be confirmed by a run of the real crate (the probe agent).

### E6: browserslist-rs data currency against the repository

- Crate data built from `caniuse-lite 1.0.30001810`,
 `node-releases 2.0.55`
 (E3);
 the repository has `caniuse-lite 1.0.30001819`,
 `node-releases 2.0.60`.
 `browserslist-data` released on 2025-07-02,
 2025-08-08,
 2026-01-15,
 2026-08-01,
 2026-09-16
 (crates.io API),
 so gaps of up to about seven months occurred (2026-01-15 to 2026-08-01).
 `last 1 node major versions` is data-driven (`last_n_node_major.rs` reads `node::versions()`),
 so a new Node release shows up only after a data release.
- Status: scored concern (data currency),
 accepted in principle by the owner.

### E7: oxc-browserslist 5.0.1 release facts and source mapping

- Source: crates.io API for `oxc-browserslist`.
- Result: 5.0.1 published 2026-08-12,
 MIT,
 crate checksum `456720daa870e4f69f8d29fded1c8deea82da4780a6b8553baa160eb0cd8d864`
 (downloaded to `/var/home/user/temp/agent/crate-dl-bl-2026-10-29/oxc-browserslist-5.0.1.crate` and extracted to `x3`).
 One crates.io owner, user `Boshen`.
 Release flow: `release-plz` action in `.github/workflows/release.yml`.
 The crates.io `trustpub_data` shows sha `a9b49a18042671c91811dc1074e01b9c08331327`.
 Tree exported to `/var/home/user/temp/agent/oxc-tag-5.0.1`;
 `cmp` of all 67 files under `src`,
 `benches`,
 and `examples` against that tree found zero differences
 (the generated data in `src/generated` is committed, so there is no dirty publish).
 `.cargo_vcs_info.json` has no dirty flag.
 `rust-version = "1.95.0"`.
 `rust-toolchain.toml` pins `1.97.1`.
 Data at that tag: `caniuse-lite 1.0.30001809`,
 `electron-to-chromium 1.5.403`,
 `node-releases 2.0.53`,
 `baseline-browser-mapping 2.11.13`,
 `browserslist 4.28.8`
 (`pnpm-lock.yaml`),
 integrity values matching the registry.
- Status: pass for provenance mapping by comparison;
 regeneration by `cargo codegen` (xtask) not yet run.
 Its data is thus older than `browserslist-rs` 0.21.2 (2026-08-12 versus 2026-09-16 inputs).
- Open check: how oxc-browserslist handles `Firefox ESR` and `not dead` and whether `ChromeAndroid` aliases resolve is unread.

### E8: dependency surface from the crates.io dependency API

- `browserslist-rs` 0.21.2 normal dependencies:
 `ahash` (serde),
 `browserslist-data ^0.3`,
 `chrono` (std,
 clock,
 oldtime),
 `either`,
 `itertools`,
 `nom 8`,
 `serde` (derive),
 `serde_json`,
 `thiserror 1`.
 Data crate: `ahash`,
 `chrono`,
 `miniz_oxide` (optional through `deflate`).
 `chrono` `clock` pulls `iana-time-zone`:
 the "today" for `dead`-adjacent date queries uses the local time zone.
 HEAD adds a default `local` feature to gate that, unreleased.
 `src/` in the crate totals 4008 lines in 49 files (`wc -l`).
- `oxc-browserslist` 5.0.1 normal dependencies:
 `miniz_oxide`,
 `postcard`,
 `rustc-hash`,
 `serde`,
 `thiserror 2`,
 `libc` on unix and windows,
 and wasm-only extras.
 `src/` totals about 4332 lines (`wc -l` over `src/*.rs`,
 `queries/*.rs`,
 `data/*.rs`; generated files not counted).
- Lifecycle scripts: none in either crate (`build = false`;
 oxc has no `build.rs`
 in the extracted tree list).
- Status: pass (no lifecycle scripts);
 transitive trees through `cargo tree` not yet run.

### E9: single-owner concentration

- `browserslist-rs`, `browserslist-data`: owner `g-plane` only.
- `oxc-browserslist`: owner `Boshen` only.
- `rspack_browserslist`: owners `hardfist`, `chenjiahan`.
- Status: finding for both finalists;
 the contributor and commit-author split is not yet measured.

## Execution manifests

No third-party command tree has been executed yet except these inspected steps,
 all inside `podman run --rm --memory=2g --cpus=2 --pids-limit=512` with no credential mounts and no home mount:

- Image `docker.io/library/node:26.12.0-trixie-slim`
 (index digest `sha256:d62f13676d530f7edf97782729df69713c8ccfe6631e7f62f9ccab65968719a8`,
 image id `7a0d2c0c3081...`).
 Command: `npm install --ignore-scripts --no-audit --no-fund --no-package-lock <five pinned packages>`
 in `/var/home/user/temp/agent/bl-repro-2026-10-29` and `/var/home/user/temp/agent/oxc-nm-2026-10-29`.
 Network on for this dependency fetch only.
 Deviations: `--user 0:0` (rootless root maps to the host user) and `--security-opt label=disable`
 (SELinux denied writes on the bind mount).
 Lifecycle scripts disabled;
 exit 0;
 integrity values of the five packages match the lock files.
- The same node image copied `/usr/local/bin/node` to `/var/home/user/temp/agent/nodebin-2026-10-29/node`
 (network none);
 `rust:1.97.0` ran that binary and printed `v26.12.0`.

Prepared, not yet executed (all in `docker.io/library/rust:1.97.0`,
 digest `sha256:baddee7381d4ec97de8a627040d9c50c3c61e8702e86985199583b1616686fad`,
 Debian 13, rustc 1.97.0, cargo 1.97.0):

1. browserslist-rs data regeneration.
 In `/var/home/user/temp/agent/bl-repro-2026-10-29` (checked out at `7503606`, node packages installed):
 network phase `cargo generate-lockfile` and `cargo fetch` with `CARGO_HOME` and `CARGO_TARGET_DIR` on the scratch volume;
 then `--network=none`: `cargo run -p generate-data --offline` with `PATH` including `/n` (the node binary).
 The generator runs `git submodule` (needs the directory to be a git repository, which it is),
 and `node -e` scripts reading `node_modules/caniuse-lite`, `node_modules/baseline-browser-mapping`.
 Then `cmp` each `data/src/generated/*` against `x2/browserslist-data-0.3.1/src/generated/*`.
 Pass criterion: byte-identical, or every difference explained.
2. browserslist-rs CI-equivalent: `cargo test`, `cargo test --no-default-features`,
 `cargo clippy --all-features`,
 `cargo clippy --example inspect`,
 `cargo clippy --tests`.
 Expected writes: tests create `/tmp/browserslist-test-pkgs` and temp files, set process environment variables,
 and spawn `node` in `extends.rs` tests.
3. oxc-browserslist data regeneration:
 in `/var/home/user/temp/agent/oxc-repro-2026-10-29` (checked out at `a9b49a1`, `node_modules` copied in),
 `RUSTUP_TOOLCHAIN=1.97.0 cargo codegen` (xtask; runs `node -e`, `cargo fmt`),
 then `git diff --exit-code -- src/generated`.
4. oxc CI-equivalent: `cargo check --all-targets --all-features`, `cargo test` (includes `tests/proptest.rs`),
 `cargo clippy --all-targets --all-features -- -D warnings`.
 Fuzz target `fuzz/fuzz_targets/fuzz_resolve.rs` and `benches/resolve.rs` are to be read and recorded as omitted
 with their commands (`cargo +nightly fuzz run fuzz_resolve`, `cargo bench`).
5. Consumer boundary: a scratch crate depending on each engine calling `resolve` with the `.browserslistrc` queries,
 printing JSON in the incumbent's shape;
 then `node` (26.12.0, type stripping) running `browserslistTargets` from
 `package/config/rolldown/src/browserslist-targets.ts` with an injected `importJson` (no repository edit),
 compared with the incumbent file contents.
 The separate probe agent covers the real-config run for `browserslist-rs`;
 the oxc run and the consumer-boundary run for both remain.

## Hard-gate exits and survivors so far

- Exit: incumbent npm `browserslist` (H1, owner's Rust constraint).
- Exit (provisional): `rspack_browserslist` (wrapper over the same engine).
- Exit: `lightningcss` and the CSS tools (category mismatch).
- Exit: `caniuse`,
 `caniuse-db`,
 `caniuse-serde`,
 `caniuse-rs`,
 `css-autoprefix` (data or CLI crates with no maintained query engine; last updates 2016 to 2021).
- Survivors: `browserslist-rs` 0.21.2 and `oxc-browserslist` 5.0.1.
 Both are open-source,
 MIT,
 inspectable,
 Linux x64 capable by pure Rust.
 H6 stays pending for `browserslist-rs` until the data regeneration (item 1) is compared.
 H6 shows a source mapping for `oxc-browserslist` by byte comparison;
 its regeneration is pending.

## Steps done, in the skill's order

1. Read the skill fully, the decision record, the planning record, the consumer, the generator, and `.browserslistrc`.
2. Context forks: measurable facts gathered; no user question needed so far.
3. Classification of base and overlays (above).
4. Candidate discovery: source class 1 complete (crates.io); class 4 partial; classes 2 and 3 not run.
5. Screening: five exits recorded; two survivors.
6. Report created and compatibility fingerprint computed (this file).
7. Open-source gates started: both crates cloned or extracted, source equals tag checked, dependencies listed,
 lifecycle scripts checked.
8. Provenance overlay: findings E3 and E7.
9. Source reading for the caller's three questions (E4) and the Firefox ESR finding (E5).

## Steps not yet done

1. Run discovery source classes 2 and 3 and finish class 4; run the one expansion round; freeze and record the saturation result.
2. Read `rspack_browserslist` source and close its exit.
3. Complete the license and transitive dependency audit (`cargo tree` per candidate, in a container).
4. Production-source reading of both finalists for parser, error handling, and date handling
 (`parser.rs`, `dead.rs`, `last_n_node_major.rs`; oxc's equivalents including how it derives Firefox ESR and today's date).
5. Tests, CI, fuzz and mutation search for both (oxc has `fuzz/` and `tests/proptest.rs`;
 `browserslist-rs` has no fuzz or property tests found in the tree listing; confirm with `rg`).
6. Maintenance audit for both repositories per the skill's sampling rule
 (issues and pull requests updated since 2025-10-29, maintainer comments versus actions, merge latency, release reach,
 commit authors in twelve months).
7. Execution manifests approved and the five prepared runs above executed, one pass each (no repeated runs).
8. Consumer-boundary run through `browserslist-targets.ts` for both finalists and comparison with the incumbent output
 (expect a Firefox 140 versus 153 difference for `browserslist-rs`; measure oxc).
9. Platform claims:
 Linux x64 only is available.
 macOS arm64 and Windows x64 are not available on this host;
 record whether the repository targets them for the Rust file-enforcer
 (the decision record uses Unix sockets) and mark omissions with evidence.
10. Freeze criteria and weights before rating.
 Planned criteria:
 query fidelity to the incumbent on the repository's real queries (weight to be set),
 data currency and release cadence,
 provenance and reproducibility of the data,
 determinism (environment variables and time-zone or clock dependence),
 dependency surface and human auditability,
 maintenance and ownership concentration,
 built-in `.browserslistrc` section parsing (default weight 1).
11. Rate, compute scores, run the complete one-at-a-time sensitivity matrix, order the finalists with adjacent reasons.
12. Write pros and cons per finalist, the recommendation or terminal result, and the final fully sorted ranking.
13. Release the report lock after durable completion.

## Open question

None is open yet.
A candidate question exists but must not be asked until sensitivity shows it controls the order:
 whether built-in `.browserslistrc` section parsing should outrank dependency leanness.
 Both engines make file-enforcer parse the file itself
 (the `config` module of `browserslist-rs` is private,
 E4),
 so the built-in-parsing advantage of `browserslist-rs` may not exist at all;
 the next agent should confirm that before spending a question on it.
If asked, the question and ranked options are:
 "Should exact agreement with the npm incumbent on `Firefox ESR` be required?"
 Options: (1) no, accept the crate's data and constants as the owner already accepted (recommended,
 it follows the owner's stated acceptance);
 (2) yes, require equality, which fails `browserslist-rs` 0.21.2 until upstream releases the 2026-10-24 fix
 and favors waiting or the other engine if it matches.

## Scratch paths created (all under `/var/home/user/temp/agent`)

- `browserslist-rs-2026-10-29`: full clone of `browserslist/browserslist-rs` at `716680c` with tags.
- `bl-tag-0.21.2`: exported tree of tag `7503606`.
- `bl-repro-2026-10-29`: copy of the clone checked out at `7503606` with `node_modules` (five pinned packages).
- `oxc-browserslist-2026-10-29`: full clone of `oxc-project/oxc-browserslist` at `6297028` with tags.
- `oxc-tag-5.0.1`: exported tree of `a9b49a1`.
- `oxc-repro-2026-10-29`: copy checked out at `a9b49a1` with `node_modules` copied in.
- `oxc-nm-2026-10-29`: npm install directory for oxc's pinned data packages.
- `crate-dl-bl-2026-10-29`: the three `.crate` files and their extractions `x1` (browserslist-rs), `x2` (browserslist-data), `x3` (oxc-browserslist).
- `nodebin-2026-10-29/node`: node 26.12.0 binary copied from the node image.
- `bl-vet-crates-queries-2026-10-29.log`: crates.io query log.

Containers: every `podman run` used `--rm` and exited;
 no process or `systemd-run` unit of this audit is running
 (the containers listed by `podman ps` belong to other sessions).

## Recommendation

None.
The audit is incomplete,
 and the skill forbids a recommendation before discovery saturation,
 finalist validation,
 scoring,
 and sensitivity.
Material risks found so far,
 stated here and not buried:
 the `Firefox ESR` constant in `browserslist-rs` 0.21.2 differs from the incumbent (140 versus 153),
 the `browserslist-data` crate is published from a dirty working tree
 whose provenance is pending a regeneration check,
 and both engines have a single crates.io owner.
