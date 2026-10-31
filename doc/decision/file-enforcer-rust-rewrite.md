# Rust file-enforcer rewrite

## Status

Accepted 2026-10-29 by the user,
answering "Do you accept the Rust file-enforcer decision record?" with "Accept",
after confirming a shared understanding of the design.
The draft was `doc/planning/file-enforcer-rust-rewrite-decision.md`.
Every decision's question,
answer,
evidence,
and history is in [`doc/planning/file-enforcer-rust-rewrite.md`](../planning/file-enforcer-rust-rewrite.md);
this record states the result.

Adopted without a direct answer from the user,
and accepted with this record:

- `user` commands get their own server keyed by the per-user config path (the user answered "?").
- mpm-shadow revalidates the plan and lock revisions before publishing,
  with an OS lock per scope as an implementation detail (the user answered "?").
- `toolchain.lock.json` carries a digest of its contents that `lock --check` recomputes
  (stated in the confirmation question with the veto open).
- A mode-only rule keeps `forbidden-strings.append.local.txt` at 0600 when present,
  since the seed that used to set it is gone.

## Context

- Request:
  "Migrate to Rust + clap + config with jsonc for file-enforcer.
  Ground up rewrite.
  No need to preserve anything." (user, 2026-10-29)
- Constraint:
  "Consider the original file-enforcer bad in every way and don't treat it as Jesus." (user, 2026-10-29)
- `doc/decision/monorepo-manager-all-rust.md` put file enforcement inside meow with an HCL config;
  meow is plan-only with no code.
- Evidence:
  the research and review reports under `doc/research/file-enforcer-rust-rewrite-*.md`,
  including three outside reviews per round (two rounds),
  a source-level mise study,
  and a podman prototype of the toolchain files (branch `prototype/fe-mpm-files`).

## Decision

### Shape

- Standalone tool that meow can embed later:
  `package/file-enforcer/core-rust` (library `monochromatic-file-enforcer`,
  engine and rule kinds,
  no clap) and `package/file-enforcer/cli` (binary `file-enforcer`,
  clap).
- Config `file-enforcer.config.jsonc`,
  read with the in-repo `monochromatic-jsonc-edit` and decoded by hand.
- Fixed Rust rule kinds with structured JSONC values (literals and tagged objects such as `{"file": ...}`,
  `{"var": "dir"}`,
  `{"concat": [...]}`);
  no string expression language.
- Every line the tool writes is one JSON object,
  including clap's help,
  version,
  and parse errors.
- Exit codes:
  0 clean,
  1 drift or rule failure,
  2 bad command line or config.

### Execution

- One auto-started server per worktree serializes every write
  over an owner-only Unix socket with newline-delimited JSON-RPC 2.0,
  and exits when idle;
  `user` commands use their own server keyed by the per-user config path.
- Commands are clients:
  `write`,
  `check` (`--expected-dir <dir>` writes paired sparse actual and expected trees for standard diff tools),
  `watch` (subscribes to the server,
  which watches inputs,
  outputs,
  and glob roots,
  and warns on hand edits to generated files),
  `explain <path>`,
  `user write`,
  `user check`.
- Each run plans every rule against a virtual view of planned outputs,
  validates the whole run,
  revalidates the snapshot it read,
  then publishes file by file through hidden staging names;
  atomicity is per file,
  and a partial run reconverges on the next run.
- Server lifecycle contracts:
  a startup lock separate from the socket,
  verified stale-socket recovery,
  a handshake on protocol version,
  build identity,
  and worktree identity,
  exit when the worktree root disappears,
  a verified owner-only fallback when `$XDG_RUNTIME_DIR` is unusable,
  and detached standard streams.
- Mirror and export paths are never followed through symlinks,
  and containment is rechecked at mutation time.

### Outputs file-enforcer writes

- `CLAUDE.md` (Claude-only preamble from a content file,
  then `AGENTS.md`).
- Root `LICENSE` and every package's `LICENSES/` copies,
  pruning only the tool-owned ids (CC-BY-SA-4.0,
  GPL-3.0-or-later,
  LGPL-3.0-or-later).
- `.claude/skills` and `.factory/skills` as copies of `.agents/skills`,
  with the targets owned outright.
- Per-crate `Cargo.toml` keys (edition,
  license,
  publish,
  lints,
  repository,
  homepage,
  readme,
  profiles,
  Dylint check-cfg),
  and `rust-catalog.toml` plus `[workspace.dependencies]` through a new dependency-catalog Rust library.
- `.browserslistrc.resolved.local.json` through an in-repo implementation of the queries in use
  (changed 2026-10-30 from `browserslist-rs`; see "Consequences").
- `.config/mise/conf.d/10-bin.local.toml` (gitignored):
  the existing non-empty `package/*/*/node_modules/.bin` directories.
- Mode only:
  `forbidden-strings.append.local.txt` stays 0600 when present.
- Through `user write`:
  the JetBrains Harper LSP4IJ policy under `~/.config/JetBrains`.

### Outputs that stop being generated

- Root `mise.toml`:
  `mise.no-env.toml` becomes the hand-written `mise.toml`,
  and the static environment moves to the committed `.config/mise/conf.d/20-env.toml`.
- `.cache/forbidden-strings.rules.txt` and its seed:
  forbidden-strings and cli-git read a comma-separated list,
  each entry anchored with `{{config_root}}`,
  where a trailing `?` marks an entry that may be absent;
  cli-git's JSONC array uses `{"path": ..., "optional": true}`.
- `mpm.packages.toml` and `mpm.managers.toml`,
  in both scopes.
- Also gone:
  the scanner post-step,
  mise task-header quoting,
  the `CONTEXT.md` guard (cli-git enforces it),
  the OS package installer and its Repology pipeline,
  the staleness cache and its lock,
  and the `exec` export.

### Schema

- Committed `file-enforcer.schema.json`,
  derived with `schemars`,
  with a cargo golden test that also checks the hand-written decoder against a coverage-complete sample corpus,
  mapped in `.idea/jsonSchemas.xml`,
  with `"$schema"` kept for VS Code.

### Toolchain (mpm-shadow)

- People edit `toolchain.plan.jsonc`;
  mpm-shadow is the only writer and checker of `toolchain.lock.json`,
  which stays strict JSON for its `jq` readers,
  carries a digest of its contents,
  and keeps its frozen surface (tool ids,
  `version`,
  `artifacts.<platform>.{url,sha256}`,
  `options.androidPackages`).
- Commands:
  `sync`,
  `status`,
  `update [--check]`,
  `lock` (offline),
  `lock --check` (the toolchain drift gate in CI beside `file-enforcer check`),
  and `bin-paths`,
  each with `--global`.
- Lookups:
  every npm and PyPI version,
  first or later,
  comes from `registry.npmjs.org` and `pypi.org` directly;
  native lookups cover the rest,
  including Rust channels;
  `mpm outdated` is not used.
  Changed 2026-10-29 from "later ones from `mpm outdated`" after the gate probe
  (`doc/research/file-enforcer-rust-rewrite-gate-probes.md`) showed mpm 8.2.0 dropping failed managers with exit 0
  and reporting registry failures only in an `errors` array;
  asked again,
  the user chose "Registry lookup always".
- The plan names every tool's origin and the lock is derived (decided 2026-10-30):
  asked how a brand-new downloaded tool declares its origin,
  the user wrote "Are we misunderstanding how it should work?
  This is a problem pnpm doesn't have so we shouldn't have the problem too.",
  then chose "Origin in the request".
  Each plan entry keeps a stable id (like a `package.json` key)
  and its `request` carries origin plus version range (like a `package.json` value or pnpm's `"alias": "npm:real@^1"`),
  for example `"age": {"request": "github:FiloSottile/age@latest"}` and `"zig": {"request": "zig@0.15.2"}`.
  No resolver reads a previous lock row;
  deleting `toolchain.lock.json` and running `update` rebuilds it from the plan alone.
  `github:<owner>/<repo>` picks each platform's asset by OS and architecture naming,
  with an optional `asset` hint only for ambiguous repositories;
  other downloads use named resolvers.
  The old plan broke this:
  downloaded tools such as `age` had bare ids and their origin lived only in `plan.artifacts.data.ts`,
  rows copied from the old `mise.lock`.
- The converted plan must write `{version}` where a value equals the resolved version
  (`"packageId": "{version}"` and `"RUSTUP_TOOLCHAIN": "{version}"` for `rust`,
  `"ANDROID_HOME": "android-sdk/{version}"` for `android-sdk`),
  or the first `update` moves `version` while those values stay stale.
- A locked npm or PyPI tool missing from the machine:
  plain `update` syncs the missing tools,
  then retries the lookup once
  ("Require sync first,
  and auto-sync on such failures and retry.",
  user);
  `update --check` never installs.
- `sync` and `status` refuse when plan and lock disagree;
  offline `lock` rejects resolution-affecting edits;
  rustup channels and cargo-binstall tools install directly from the lock;
  postinstall links and commands run in sync.
- Mutations revalidate the plan and lock revisions they read before publishing.
- Bootstrap:
  a prebuilt `mpm-shadow` archive with its SHA-256 in the lock,
  or `cargo install monochromatic-mpm-shadow` after a verified rustup install.
- cargo-binstall tools install under a version-keyed directory in the toolchain root
  (for example `<toolchain root>/cargo/<crate>/<version>`),
  never the shared `$CARGO_HOME/bin`,
  and each tree links its own lock's version,
  so trees with different locks cannot overwrite each other's tools.
  Decided 2026-10-31 after a sync in a worktree with an older lock replaced the git wrapper for every checkout
  ([`doc/troubleshooting/toolchain-sync-older-lock-downgrades-shared-cargo-installs.md`](../troubleshooting/toolchain-sync-older-lock-downgrades-shared-cargo-installs.md));
  the user chose "Fix in mpm-shadow before cutover"
  over fixing it after cutover or keeping the shared install.
  Versions no lock names need a cleanup step.
- Tools this repository publishes are the exception,
  in the user's words:
  "No older repo-maintained tools should ever exist when newer is present."
  So a sync never installs a repository-published tool older than one already installed,
  and installing a newer one removes the older;
  a tree whose lock pins an older version runs the newer one,
  and `status` counts installed-newer-than-locked as satisfied for these tools,
  with a warning every time it happens
  ("and this should have a warning whenever it happens",
  user):
  each sync,
  status,
  and other toolchain command that resolves such a tool for that tree names the tool,
  the locked and the used version,
  and that the newer one runs because older repository tools are never kept.
  The warning also prints on every run of the tool in that tree
  (the user chose "Also on every tool run",
  noting "give the recourse: upgrade" and "noisy is fine"):
  it goes to stderr,
  never into a tool's stdout,
  and ends with the remedy,
  upgrading the tree's lock through `mise run toolchain:update` or by merging `main`.
  A tree's exposed link for such a tool goes through an `mpm-shadow` launcher that compares the tree's lock with the version it runs,
  since a bare link to the binary has no place to check.
  This relies on each release reading its predecessors' configuration,
  as wrapper 0.4.4 reads the string `rulesFile` of 0.4.0
  (`rules_file_takes_a_path_or_an_array_of_entries` in
  `package/git-policy/cli/src/native/config_parse_acceptance_tests.rs`).
  Version-keyed coexistence applies only to third-party tools.
- file-enforcer has no toolchain role.

### Elsewhere

- pnpr generates and checks its own `package/config/pnpr/config.yaml`.
- The bounded container runner moves to `package/test-harness/core-matrix`.
- `package/vm-builder/cli` gets a private process runner.
- The IDE launcher finds `mpm-shadow` by explicit path for `bin-paths`.

### Cutover

- One switch,
  after everything is built,
  published,
  fetched by a clean container,
  and compared with the TypeScript tool's output on this repo;
  output need not be byte-identical.
- The switch commit repoints tasks,
  CI (checking committed outputs before any regeneration,
  then local outputs separately),
  the `Dockerfile`,
  the IDE launcher and its tests,
  the scanner and cli-git configs,
  the two fixtures that still depend on the old package,
  the commands against the deleted `mpm.managers.toml` in `doc/runbook/publish-crate-first-time.md:463`
  and `doc/decision/mpm-toolchain-adoption.md:348`,
  the `mise.toml` and `mise.no-env.toml` schema mappings in `.idea/jsonSchemas.xml:26,32`,
  and deletes `package/file-enforcer/core`.
- The global plan is converted directly,
  keeping a 0600 backup beside the original until the converted global sync is verified.

## Consequences

- Amend `doc/decision/monorepo-manager-all-rust.md`:
  meow links `monochromatic-file-enforcer`,
  its file-enforcement config is JSONC,
  and the root `mise.toml` is no longer generated.
- Amend `doc/decision/mpm-toolchain-adoption.md`:
  no generated restore manifest (`:92-112`),
  and mpm-shadow,
  not file-enforcer,
  owns the toolchain policy and lock (`:201-223`).
- Gates before implementation:
  a live forbidden-strings run on throwaway rules files (including a missing optional entry),
  `browserslist-rs` on the real `.browserslistrc` feeding the rolldown consumer,
  and `mpm outdated` probes varying installed,
  locked,
  and registry versions (the reading rule stays pending until then).
- Dependencies,
  decided 2026-10-29 without completed vets:
  the user wrote "I'm approving toml_edit,
  browserslist-rs,
  schemars as-is.
  For xml editor,
  write one ourselves like what we did for jsonc."
  `toml_edit`,
  `browserslist-rs`,
  and `schemars` enter `rust-catalog.toml` as approved;
  the JetBrains rule's XML editing uses a new in-repo crate built the way `monochromatic-jsonc-edit` was
  (`doc/decision/jsonc-edit-rust-foundation.md`:
  an owned scanner,
  parser,
  and emitter with no third-party parser),
  preserving comments and formatting outside edited elements
  and rejecting malformed managed XML with a diagnostic.
  The four halted vet drafts under `doc/audit/tech-file-enforcer-*-vet-2026-10-29.md` are superseded by this
  and kept whole as knowledge ("Even if work is superseded,
  it should still become knowledge.",
  user).
  Open facts they leave for the implementation:
  `browserslist-rs` 0.21.2 hard-codes Firefox ESR as `firefox 140` where npm `browserslist` 4.29.5 gives `firefox 153`,
  and its section selection is private,
  so file-enforcer parses `.browserslistrc` itself
  (`oxc-browserslist` 5.0.1 screened as an alternative);
  `schemars` derive without serde `Deserialize` is unproven,
  and the golden test's corpus check needs a JSON Schema validator crate not yet named
  (resolved 2026-10-30:
  "I'm approving jsonschema as is.");
  `monochromatic-jsonc-edit` emits trailing commas,
  while the JetBrains files hold strict JSON.
  Resolved 2026-10-29 after the user asked whether strict JSON editing is built in or a de facto crate:
  the standard library has no JSON,
  and the de facto crate `serde_json` is already in `rust-catalog.toml:91` (`serde_json = ">=1"`)
  and used by seven repository crates;
  it writes strict JSON,
  and its insertion-order `Map` feature keeps key order through a read-modify-write.
  The JSON embedded in the JetBrains XML is therefore edited with `serde_json`,
  not `monochromatic-jsonc-edit`;
  the user approved it:
  "I'm approving serde_json as is."
- Browserslist,
  changed 2026-10-30:
  the user wrote "Change of plans,
  for browserslist,
  we retrieve the data and write something conforming to the original browserslist ourselves,
  only supporting the queries we use now,
  throw on everything non-conforming with \"not implemented\"."
  This supersedes the `browserslist-rs` approval:
  it does not enter `rust-catalog.toml`.
  The implementation conforms to npm `browserslist` for the seven lines of `.browserslistrc`
  (`last 1 Chrome versions`,
  `last 1 ChromeAndroid versions`,
  `last 1 Android versions`,
  `Firefox ESR`,
  `last 1 FirefoxAndroid versions`,
  `last 1 node major versions`,
  `not dead`)
  and fails with "not implemented" on every other query form and config feature.
  The specification is `doc/research/browserslist-subset-semantics.md`;
  the user then decided its sources (2026-10-30):
  - Data:
    "Own registry fetch".
    An update command downloads the `caniuse-lite` and `node-releases` tarballs from `registry.npmjs.org`
    over the approved HTTP stack,
    verifies npm's sha512 `dist.integrity`,
    and pins versions and hashes in a committed lock;
    writes stay offline and read the cached data.
    The option stated that choosing it approves `flate2` and `tar`;
    the user did not object,
    so both are approved.
  - Firefox ESR:
    "Mozilla product-details",
    chosen over reading browserslist's `FIREFOX_ESR_VERSION` constant.
    The update command reads `FIREFOX_ESR` from `https://product-details.mozilla.org/1.0/firefox_versions.json`
    (`"153.6.0esr"` on 2026-10-30,
    whose major 153 equals browserslist 4.29.5's constant)
    and pins it in the same lock;
    a value not shaped `<major>.<minor>.<patch>esr` fails with a diagnostic.
    When browserslist's constant lags Mozilla,
    this output differs from npm `browserslist`;
    that is accepted by this choice,
    so the conformance test compares ESR against the locked Mozilla value,
    not against npm.
  - Spelling:
    exact text only.
    Asked between exact text and browserslist's full leniency
    (case folding,
    JavaScript whitespace,
    optional plural,
    leading zeros),
    the user wrote "Whatever is easier to implement.";
    exact text is,
    since it avoids every JavaScript-against-Rust whitespace and case difference the spec lists.
    Any other spelling fails with "not implemented" naming the line.
  - Config file:
    the user chose "Accept them" with the note "Accept # comments and blank lines and reject everything else.",
    so `#` comments (with browserslist's comment rule from the spec) and blank lines are accepted;
    commas as separators,
    CR line endings,
    sections,
    and every other `parseConfig` feature fail with "not implemented".
  - Adopted from "only supporting the queries we use now",
    veto open:
    only the count 1 is supported in `last N ...` queries
    (other counts,
    including browserslist's `last 0` and large-count quirks,
    fail with "not implemented"),
    and `not dead` evaluates browserslist's own `dead` expansion
    (`Baidu >= 0`,
    `ie <= 11`,
    `ie_mob <= 11`,
    `bb <= 10`,
    `op_mob <= 12.1`,
    `samsung 4`)
    internally against the locked data rather than a hard-coded result.
  - Node pre-releases (2026-10-30):
    `node-releases` 2.0.62 lists `27.0.0-alpha.0.0.0`,
    and npm `browserslist` 4.29.5 (the newest on npm) then resolves `last 1 node major versions` to only that alpha
    (measured in a container);
    no upstream issue exists.
    First choosing filtering with the note "Isn't filtering pre-releases out common knowledge and conforming to upstream?",
    then told that upstream does not filter,
    the user chose "Filter pre-releases anyway" ("filter pre-releases anyway and write a /troubleshooting-doc").
    Pre-release Node versions are dropped before node queries are evaluated;
    the conformance test records it as a named exception like Firefox ESR.
  - Lock location:
    the user chose "Move it to the repo root",
    so the lock is `browserslist-data.lock.json` beside `toolchain.lock.json`.
  Evidence that led here:
  `doc/research/file-enforcer-rust-rewrite-oxc-browserslist.md`.
- HTTP client,
  decided 2026-10-30 without a vet:
  the user wrote "Add `hyper` to the Rust catalog.
  I'm approving it.
  If you think it's not the best choice for what we need,
  tell me why and I'll research more.",
  then "Additionally please add hyper-util,
  tokio,
  http-body-util to the rust catalog and I'm approving them too.
  Again,
  if not enough,
  tell me."
  So `hyper`,
  `hyper-util`,
  `tokio`,
  and `http-body-util` are approved.
  They enter `rust-catalog.toml` when a crate first uses them:
  `catalogPrune = true` removed an unused `hyper = ">=1.12.0"` line on the next `mise run sync:files` (measured 2026-10-30).
  Not enough for the registries,
  which are HTTPS:
  hyper's guide says "hyper allows you to bring your own IO,
  so it can work on top of any TLS implementation"
  and names separately maintained connectors (`hyper-tls`,
  `hyper-rustls`,
  `hyper-openssl`);
  `hyper-rustls` 0.27.10 defaults pull `aws-lc-rs`,
  `rustls-native-certs`,
  `tokio-rustls`,
  and `log`,
  with `ring` and `webpki-roots` as opt-in features.
  Told this,
  the user wrote "Add this too.
  https://crates.io/crates/hyper-rustls",
  so `hyper-rustls` is approved,
  used with its default features (`aws-lc-rs` and native roots) unless a reason to change them turns up.
- Follow-ups:
  process rules in issue #1817.

## Rejected

- Building it as meow's first slice,
  or standalone with no embedding path.
- The `config` crate (config-rs):
  no JSONC format,
  no error positions.
- String expressions or Rust-only named rules for the config.
- A text renderer beside JSON lines.
- No coordination,
  or flock instead of the server,
  for file-enforcer writes.
- A three-file toolchain model (plan,
  resolution,
  lock).
- file-enforcer rendering or checking the mpm files:
  the prototype showed it fails on a machine without file-enforcer and that the files mislead installs.
- Replacing `CLAUDE.md` with an `@AGENTS.md` import or the skill copies with symlinks:
  closed-source tool behavior is not a contract.
- `browserslist-rs` 0.21.2 and `oxc-browserslist` (5.0.1 and unreleased main),
  2026-10-30,
  measured on the real `.browserslistrc` against npm `browserslist` 4.29.5:
  both hard-code Firefox ESR as 140 where npm gives 153,
  and both ship older data
  (chrome 151,
  151,
  and 154 on main,
  against 156).
  oxc refreshes data on main daily but its 5.0.2 release PR has been open since 2026-08-13.
  Details:
  `doc/research/file-enforcer-rust-rewrite-oxc-browserslist.md`.
