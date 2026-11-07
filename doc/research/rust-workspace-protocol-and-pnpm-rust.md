# Rust crates and tools after GitHub releases: the workspace protocol, Cargo's mechanisms and pnpm's Rust support

Research note, written at 2026-11-07T09:07:32-05:00.
It decides nothing:
every ranking here is a proposal for the owner,
and nothing in the repository other than this file was changed for it.

The owner asked twice for research on "a workspace: protocol or something"
(2026-11-05 and 2026-11-07, verbatim in `doc/planning/crates-io-footprint.md` and
`doc/decision/mpm-toolchain-adoption.md`, section "Our seven tools install from GitHub release archives"),
and on whether "pnpm's new rust management capabilities can help".
This note answers both from primary sources and from probes run here.

## Answers in one place

### What pnpm's `workspace:` protocol does

It has two halves.
Locally, `workspace:` makes pnpm resolve a dependency only to a sibling package of the workspace
and refuse anything else ([pnpm.io, Workspace protocol][pnpm-ws-resolve]).
At pack and publish time, pnpm rewrites each `workspace:` dependency to a plain version
(`workspace:*` to `1.5.0`, `workspace:~` to `~1.5.0`, `workspace:^` to `^1.5.0`, `workspace:^1.5.0` kept as the range),
"without needing intermediary publish steps" ([pnpm.io][pnpm-ws-nosteps]).
That last clause only holds because the npm registry does not check that a dependency exists when a package is
published.

### Which Rust mechanisms give the same

Cargo has the local half and a partial publish half, in one form: `path` plus `version`,
plus `registry` for any registry other than crates.io.
Packaging drops `path`, keeps `version`, and turns `registry = "name"` into `registry-index = "<url>"`
(probe in "Probe results: what `cargo package` writes").
Cargo does not have the part npm gets for free: the dependency's version must already exist in the target registry
when the dependent is packaged, so crates still publish dependencies first.
Nothing in Cargo's stable or nightly features removes that, and `-p a -p b` publishing needs one Cargo workspace.
Every other mechanism tried (`[patch]`, `[replace]`, `paths`, directory sources, local registries, git)
is either a local-only
override or hides the packaging check.
One of them, a directory source that replaces a named registry,
makes a committed `Cargo.lock` unusable under `--locked` without the replacement.

### What pnpm now offers for Rust

pnpm 12.4.0 (2026-09-08) added Cargo dependency management behind `cargo.enabled`,
and 12.11.0 (2026-10-09) added Rust toolchain management.
This repository's pnpm is 12.18.0, so both exist here, and `pnpm-workspace.yaml` does not enable either.
It installs and vendors third-party crates and the toolchain.
It has no `workspace:`-like protocol for crates,
cannot generate a lock for a path dependency between two independent Cargo roots,
cannot install a crate as a tool (`crate:` cannot be installed globally),
and applies no release-age or trust policy to crates.
It does not serve use (a) or use (b).

### GitHub's package registry

Still no Cargo registry as of 2026-11-07.

### Where to go

The best-supported route is pnpr as the Cargo registry for the crates (run end to end here)
with the seven tools staying on GitHub archives.
Whether the lock selects a tool by an exact version taken from the checkout instead of `@latest` is an owner decision:
it is proposed only because `@latest` skips releases GitHub flags as prerelease
and `compare_versions` orders `-alpha.10` before `-alpha.9` (both source readings, not run).
Section "Options and ranking" gives the ranking with a reason for every adjacent pair and marks the owner decisions.

## What was read, run and left unverified

- Repository at `05a529bb22d3b39bc1dfbad8b002afdf9b10d62b`.
- pnpm source and docs at `0470052291b2ba369be93a9b696e0a0097722ba5` (tag `v12.18.0`, shallow clone read only).
  The pnpm that ran is the repository's locked 12.18.0 (`.cache/toolchain-bin/pnpm`,
  `pnpm --version` printed `12.18.0`).
- Cargo source and book at `c07ac2c4d12237effc4b94f4543769b421790045` (2026-11-06).
  The Cargo that ran is `cargo 1.101.0-nightly (111443f3d 2026-11-05)` with
  `rustc 1.101.0-nightly (c980177d8 2026-11-06)`.
- pnpr `0.1.0-alpha.18` from the `pnpm/pnpm` release `pnpr@0.1.0-alpha.18`,
  `pnpr-linux-x64.tar.gz`; the extracted binary's SHA-256 equals the `pnpr-linux-x64.sha256` asset (`5201f665...`).
- All pnpm GitHub releases (1225 release records) were downloaded and searched for `cargo`, `crate`, `rust`,
  `rustup`, `Cargo.lock`.
- Disposable fixtures only, under `~/temp/agent/pnpm-rust-2026-11-07/`.
  Every command ran in `systemd-run --user --scope` with `CPUQuota=200%`, `MemoryMax=2G`, `TasksMax=128`
  and `IOWeight=10` (the browser check used `MemoryMax=3G` and `TasksMax=256`).
  The two local servers ran as transient units (`MemoryMax=256M` and `1G`, `CPUQuota=100%`, `TasksMax=32` and `128`,
  `IOWeight=10`)
  and are stopped.
- Disclosure for the caller to judge against "never publish anything":
  the pnpr probe ran a real `cargo publish` (not `--dry-run`)
  of two fixture crates named `fixrust-lib-a` and `fixrust-tool-b`,
  against a throwaway pnpr bound to `127.0.0.1:17677` with its own storage and a throwaway account.
  Nothing was uploaded to crates.io or any host off this machine,
  no credential of the owner was read (`gh` made read-only API calls),
  and the throwaway token file was deleted.
- Not run: any `cargo publish` to crates.io, `cargo binstall` against pnpr,
  `cargo publish -p` against an unpublished dependency
  inside a pnpr batch, and a pnpm `cargo.enabled` install in the real repository.

## Where the 2026-11-05 note was wrong or incomplete

`doc/research/cargo-crates-and-tools-without-crates-io.md` (`54c6511df7`, `62ccbc6fa5`) stays right on:
`path` plus exact `version` is how crates use each other,
`cargo package` needs the dependency in the target registry,
floating git is blocked inside the repository,
GitHub Packages has no Cargo registry,
and a static sparse index on GitHub works with new code.
Its Option 1 did cover pnpm's `workspace:` text and Cargo's equivalents.
Plainly, where it fell short:

- It answered the manifest question and the install question as one verdict ("workable for uses (a), (b) and (d)"),
  so the reader could not see which half of `workspace:` Cargo has and which it lacks.
  This note separates them.
- It tested `{ path, version, registry }` only for a publish to crates.io, and called it "a trap".
  That is true for crates.io, which refuses other-registry dependencies.
  It did not test the case that matters for `workspace:`-like behaviour:
  publishing to the registry that dependency names.
  That case works and is the workspace-protocol rewrite (see "Probe results: what `cargo package` writes").
- It never covered pnpm's Cargo support.
  pnpm 12.4.0 had been released on 2026-09-08, two months before the note.
- Its claim that no Cargo mechanism lets a crate on crates.io depend on an unpublished crate stands.
  It also did not probe pnpr itself with Cargo; this note did.
- The session's question file calling the archive route the owner's "first pick" was wrong.
  The owner's first pick was a `workspace:` protocol, and this note is the research that pick needed.

## Question 1: what pnpm's `workspace:` protocol does exactly

- Resolution:
  "When this protocol is used, pnpm will refuse to resolve to anything other than a local workspace package."
  With `"foo": "workspace:2.0.0"` and no local `foo@2.0.0`, install fails
  ([pnpm.io, Workspace protocol][pnpm-ws-resolve]).
- Forms of the right-hand side:
  `workspace:*`, `workspace:~`, `workspace:^`, an explicit range such as `workspace:^1.5.0`,
  a bare `workspace:` (treated as `workspace:*`; [pnpm.io][pnpm-ws-bare]),
  a relative path `workspace:../foo`,
  and an alias `workspace:foo@*`.
- Rewrite at pack and publish:
  "we dynamically replace any workspace: dependency by" the target's version (for `workspace:`, `*`, `~`, `^`)
  or by the associated range (for any other range) ([pnpm.io][pnpm-ws-replace]).
  With foo, bar, qar, zoo all at 1.5.0:
  `workspace:*` becomes `1.5.0`, `workspace:~` becomes `~1.5.0`, `workspace:^` becomes `^1.5.0`,
  `workspace:^1.5.0` stays `^1.5.0`.
  Path and alias forms are converted to ordinary version specs or `npm:` aliases.
- The point of the feature:
  publish the packages "without needing intermediary publish steps"
  ([pnpm.io][pnpm-ws-nosteps]).
- What the protocol assumes of the registry:
  none of this checks that the dependency's version exists in the registry at publish time.
  That is a property of the npm registry protocol, not of the `workspace:` protocol,
  and it is exactly where Cargo registries differ (see "Probe results: what `cargo package` writes").

## Question 2: Rust mechanisms, each against uses (a) and (b)

Use (a): our crates depend on each other inside the repository.
Use (b): install our tools on this machine without compiling them
(repository rule: never compile a locked tool this repository publishes).

### Probe results: what `cargo package` writes

Fixture: two crates in separate `[workspace]` roots, `package/lib-a` and `package/tool-b`,
`tool-b` depending on `lib-a` with `path` plus `version = "=0.1.0-alpha.1"`.
A static sparse index (`api: null`) and a pnpr alpha.18 hosted Cargo registry served as targets.

- With `registry = "name"` on the dependency, `cargo package` (static index)
  wrote this dependency table into the packed manifest:
  `[dependencies.fixrust-lib-a]` with `version = "=0.1.0-alpha.1"` and
  `registry-index = "sparse+http://127.0.0.1:18431/"`.
  The `path` is gone, the version stays, and the registry name became its URL,
  so a consumer needs no configuration for the transitive dependency.
- Without the `registry` key, `cargo package --registry fix` and `cargo publish --registry pnpr` both failed with
  "no matching package named `fixrust-lib-a` found, location searched: crates.io index".
  So the key is required on every internal dependency whose target registry is not crates.io.
- Against pnpr alpha.18, in this order:
  `cargo publish --registry pnpr` of `lib-a` `0.1.0-alpha.1` succeeded,
  then of `tool-b` (dependency with the `registry` key) succeeded.
  Cargo accepted the `-alpha.1` label and pnpr stored it.
  A new project whose Cargo configuration only names the registry's index, with no token set,
  built a dependency on `tool-b` with `cargo build`:
  Cargo downloaded both crates from pnpr anonymously, and its `Cargo.lock` recorded
  `source = "sparse+http://127.0.0.1:17677/cargo/~crates/index/"` and a `checksum` for each.
- Bumping `lib-a` and `tool-b` to `-alpha.2` and running `cargo publish --registry pnpr --dry-run` for `tool-b` first
  failed with "failed to select a version for the requirement `fixrust-lib-a = "=0.1.0-alpha.2"` ... candidate
  versions found which
  didn't match: 0.1.0-alpha.1".
  Cargo itself enforces dependency-first publishing against pnpr, not only crates.io's server.
- `cargo publish --registry pnpr --dry-run -p fixrust-lib-a -p fixrust-tool-b` from the tool's directory failed with
  "package ID specification `fixrust-lib-a` did not match any packages".
  Multi-package publishing needs one Cargo workspace ([Cargo book, unstable,
  `package-workspace`][cargo-package-workspace]:
  stabilized in 1.90.0), which the owner's decision rules out.
- A `Cargo.lock` for `tool-b` with the `path` plus `version` plus `registry` dependency was generated with
  `cargo generate-lockfile --offline`: the internal crate appears as a path package with no `source` line,
  and nothing contacted the registry.
  `--locked` therefore works in a clean checkout, and `cargo-shadow` (which reads only `registry+` and `sparse+` sources
  in `package/cargo-shadow/cli/src/lockfile.rs`, source reading) has no internal entry to refuse.

For use (a), `path` plus `version` plus `registry` is the Cargo counterpart of `workspace:`:
local edits are seen at once, the lock holds no registry source for internal crates,
and packaging produces a registry dependency without a hand-written step.
What it lacks:
the `registry` key must be in the manifest (the dependency catalog's accepted keys, `LITERAL_SITE_KEYS` in
`package/dependency-catalog/core-native/src/site_classify.rs`, are `default-features`, `features`, `git`, `optional`,
`package`, `path`, `rev`, `version`, so a site with `registry` is not accepted today; source reading),
and a manifest cannot serve two targets: the same file must have the key for pnpr and not have it for crates.io
(crates.io refuses a dependency with a `registry`, per the prior note's reading of crates.io's server code).
For use (b) it gives nothing: it concerns manifests, not installing programs.

### `[patch]` and `[replace]`

- `[patch]` in a manifest is read only at the root of a Cargo workspace;
  "Patch settings defined in dependencies will be ignored"
  ([Cargo book, Overriding dependencies][cargo-patch-root]).
  Each crate here is its own root, so each carries its own table.
- `[replace]`: "is deprecated" ([Cargo book][cargo-replace-deprecated]).
- A `[patch.crates-io]` path entry (manifest or `.cargo/config.toml`) makes local builds use the path
  but the packed manifest keeps a plain `version`, so the published crate depends on a registry crate.
  The prior note measured that `cargo package` passes inside the config's tree and fails outside it,
  which hides the dependency-first guard.
- Use (a): works locally, hides the guard when packaging; use (b): nothing.

### `.cargo/config.toml` `paths`

- Cargo book: "You cannot use this feature to tell Cargo how to find local unpublished crates"
  ([Cargo book][cargo-paths-note]), and the graph may not change.
- Probe: with `paths = ["package/lib-a"]` and a dependency `{ version = "=0.1.0-alpha.1", registry = "fix" }`,
  the build used the local edit (it printed `2` where the published copy returned `1`).
  Changing the requirement to `=0.9.9` failed:
  "failed to select a version ... candidate versions found which didn't match:
  0.1.0-alpha.1".
  So the version must already exist in the registry.
- Use (a): only for a crate already published at that version; use (b): nothing.

### Directory, local-registry and vendor sources

- Cargo has a `directory` source (unpacked crates, each with `.cargo-checksum.json`, "to protect against accidental
  modifications"; [Cargo book][cargo-dirsource]) and a `local-registry` source (`.crate` files plus an index).
  Replacing a registry's source with one makes that registry's dependencies resolve from disk.
- Probe, directory source replacing a named registry with the registry server stopped:
  the dependency `{ version = "=0.1.0-alpha.1", registry = "fix" }` built from `internal/fixrust-lib-a`,
  a symlink to the crate directory that holds a `.cargo-checksum.json` of `{"files":{},"package":null}`.
  The program printed the local edit.
  `cargo package` under the replacement wrote the same `registry-index` rewrite.
- Probe, the committed lock: the `Cargo.lock` Cargo wrote under the replacement had `source = "sparse+..."` for the
  internal crate
  and no `checksum` line.
  After moving the replacement away, `cargo build --locked` against the running pnpr failed:
  "cannot update the lock file ... because --locked was passed to prevent this".
  So a replacement works for a local session only, and the committed lock must come from a registry resolution,
  which for an unpublished alpha does not exist yet.
  This is also the mechanism pnpm itself uses for third-party crates
  ([pnpm.io, Cargo dependencies][pnpm-cargo-nocompile]: vendored `.pnpm/crates` with a source replacement).
- Use (a): possible but worse than `path` for the lock; use (b): nothing.

### Alternative registries

- pnpr alpha.18 hosts Cargo crates (see "Probe results: what `cargo package` writes"), serving `config.json`,
  the sparse index, `cargo publish`, `cargo yank` and anonymous reads when the registry allows.
  Its Cargo `packages:` map accepts only an exact crate name or `**`:
  a pattern such as `fixrust-*` made pnpr refuse to start with
  "unsupported cargo registry pattern "fixrust-*": use an exact name, nothing narrower, or `**`".
  The published crate set therefore needs one entry per crate or `**`.
- pnpr also documents a batch publish endpoint with Cargo entries,
  "Every entry is authorized and verified before any of them is written" and "a batch that fails a check publishes
  nothing"
  ([pnpr docs, One publish for a workspace that spans ecosystems][pnpr-batch]).
  No client found sends a Cargo entry:
  `pnpm publish` has no Cargo mention in its docs,
  `cargo publish` sends one crate,
  and a source search of `pnpm/crates` for a Cargo publish path found none.
  Whether the server checks a Cargo entry's dependencies against the same batch was not probed.
  If it does, the release tool could publish a dependency closure as one transaction, which is the closest registry-side
  counterpart of "no intermediary publish steps".
- A static sparse index (prior note):
  works for reading (a fixture resolved from it and `cargo-binstall` installed a prebuilt program),
  but `cargo publish` is refused: `registry fix does not support API commands` (probe, `api: null`).
  A generator and a whole-site Pages deploy are new code.
- `cargo-local-registry` style tools: a directory of `.crate` files; no live editing.
- Use (a): pnpr and a static index both carry the same `registry` key; use (b):
  `cargo-binstall --index` reads the crate's binstall metadata
  from the registry but downloads the program from the address in that metadata (the prior note's fixture),
  so the bytes still live elsewhere.

### Git dependencies

The prior note's tests stand: `git` plus `rev` pins one commit for the whole repository;
a moving `branch` floats only at `cargo update`;
the dependency catalog accepts `git` with `rev` and refuses `branch`;
every git installer compiles.
Not re-run here.
pnpm 12.5.0 vendors git crates at their locked commit
([pnpm.io, Git-sourced crates][pnpm-cargo-git]),
which does not make a branch float.
Use (a): blocked; use (b): blocked.

### Unstable and newer Cargo features

Read from the Cargo book at the pinned commit
([`unstable.md`][cargo-unstable]):
`path-bases` ([Path Bases][cargo-unstable-pathbases]; tracking issue 14355, open) only prefixes `path` values,
and its built-in `workspace` base needs a workspace;
`package-workspace` is stabilized;
`-Zgit` shallow fetch is unstable;
`-Zscript` and `builtin-dependencies` are unrelated.
rust-lang/cargo issues 5042 ("Nested workspaces", open, updated 2026-08-04),
15435 ("Remote Version Definitions", open) and 5332
("Allow to share versions of dependencies between unrelated Cargo packages", open) are the cross-root requests;
none is implemented.
Searches for a path-dependency rewrite for alternate registries returned no issue,
so the `registry` to `registry-index` rewrite is the behaviour, not a proposal.

## Question 3: pnpm's Rust management

### What exists

From the pnpm release notes and docs
([release 12.4.0][pnpm-12-4-0], [12.5.0][pnpm-12-5-0], [12.11.0][pnpm-12-11-0], [12.17.0][pnpm-12-17-0],
[12.18.0][pnpm-12-18-0]):

- 12.4.0 (2026-09-08): with `cargo.enabled: true` in `pnpm-workspace.yaml`, `pnpm install` installs Cargo dependencies;
  `pnpm add crate:<name>` edits `Cargo.toml`; sparse registries through `registries` with `ecosystem: cargo`.
  The docs call it experimental: "Multi-ecosystem support is experimental" ([pnpm.io][pnpm-cargo-experimental]).
- 12.5.0 (2026-09-18): `pkg:cargo/` URLs; `[patch]` and `[replace]` path and git overrides when generating `Cargo.lock`;
  git dependencies and submodules vendored at the pinned commit.
- 12.11.0 (2026-10-09): Rust toolchain management from `rust-toolchain.toml`, `pnpm add -g rust@<channel>`,
  `pnx --package=rust@<channel>`
  (12.11.1), and the vendoring include written into `.pnpm/crates/config.toml`.
- 12.17.0 (2026-10-30) and 12.18.0 (2026-11-05): `pnpm outdated` and `pnpm update` for Cargo dependencies.
- pnpr 0.1.0-alpha.11 (2026-09-10) added the Cargo registry, with `cargo publish`, `cargo yank` and downloads.
- Mechanism ([pnpm.io, Cargo dependencies][pnpm-cargo-nocompile]):
  pnpm resolves against the sparse index (or with Cargo, for overrides),
  writes `Cargo.lock`, verifies each `.crate`, unpacks it into its store and links a Cargo directory source;
  "pnpm does not compile anything itself".
- This repository's lock holds pnpm 12.18.0, so all of this is available; `pnpm-workspace.yaml` has no `cargo:` block.

### What it does not do, from source and probes

- No `workspace:` protocol for crates.
  Searches that returned nothing for it:
  `rg '"workspace:|workspace:[*^~]|workspace:\.\.'` over `pnpm/crates/cargo-resolver`,
  `pnpm/crates/cli/src/cargo_deps` and
  `pnpm/crates/cli/src/cargo_deps.rs`;
  `gh search issues --repo pnpm/pnpm` for "cargo workspace: protocol", "crate path dependency publish",
  "cargo independent workspace roots" and "Cargo path dependency version rewrite publish"
  (the first returned four unrelated closed pnpr issues);
  the Cargo lines of the 1225 release records, grepped for `workspace`, which showed only mentions of Cargo workspaces.
  Positive controls: the same `rg` pattern hits `pnpm/crates/versioning` and `pnpm/crates/catalogs-resolver`,
  and the release records yield the Cargo lines of 12.4.0 to 12.18.0 cited here.
- Two independent Cargo roots in one pnpm workspace (fixture `fixture-multi-root`, pnpm 12.18.0, `cargo.enabled`):
  pnpm finds each root ([`discover_workspace_roots`][pnpm-discover-roots]) but
  without a `Cargo.lock`, `pnpm install` failed with
  "resolve Cargo dependencies ... workspace dependency fixrust-lib-a is not a workspace member"
  ([`cargo-resolver/src/lockfile.rs`][pnpm-not-member]:
  only members of the same Cargo workspace are accepted as path dependencies).
  With a `Cargo.lock` generated by Cargo in each root beforehand,
  `pnpm install` succeeded and wrote `.pnpm/crates/config.toml`
  and `.cargo/config.toml` in every root.
  A single Cargo workspace with both crates as members succeeded without a lock (positive control).
  So pnpm tolerates independent roots and cannot generate their locks.
- `[patch.crates-io]` with a path entry (fixture `fixture-patch`): `pnpm install` generated a lock through Cargo.
  This needed the project's own `git` in `PATH` to run `git config` from the crate directory;
  with a `.git` directory in the fixture,
  this repository's `git` wrapper refused ("not at the root of the git repository")
  and the install failed with "read Cargo Git transport policy".
  The install succeeded only after removing the fixture's `.git`.
  In this repository every crate lives inside the root checkout,
  so enabling `cargo.enabled` here would hit that refusal for any override-using crate.
- `crate:` is a dependency prefix, not a tool installer:
  global-package docs list "`crate:` and Python dependencies" among
  what "cannot be installed globally" ([pnpm.io, Global packages][pnpm-global]).
  `pnpm add -g rust` installs a toolchain, not a crate.
- Third-party registry on a dependency: "A dependency that names a third-party registry of its own ... is rejected:
  pnpm resolves one index per workspace" ([`pnpm/docs/cargo.md` L75][pnpm-cargo-registry-line]).
  An internal dependency with `registry = "pnpr"` would be rejected by `cargo.enabled`
  unless pnpr is the workspace's one declared index,
  in which case crates.io dependencies would need pnpr's upstream proxy.
- No release-age, trust-policy or catalog for crates was found:
  `minimumReleaseAge` appears in `pnpm/crates/config`, but a search of `pnpm/crates/cargo-resolver/src` and
  `pnpm/crates/cli/src/cargo_deps` for release-age, publish-time and catalog terms returned nothing,
  and the settings and Cargo docs name no such setting for crates.
  That is "none found in these searches", not a trace of every settings path into Cargo resolution;
  a test with the policy enabled and a too-new crate was not run.
  The repository's `rust-catalog.toml` and `cargo-shadow` provide those today
  (`doc/decision/rust-dependency-catalog.md`).
- No Cargo publish: `pnpm publish` and `pnpm pack` docs mention no crates,
  and pnpr's batch endpoint is documented for Cargo entries
  but no pnpm client code builds one.
- The owner's fork `Aquaticat/pnpm`: `main` is 0 commits ahead and 442 behind `pnpm/pnpm:main` on 2026-11-07;
  its other branches are `docs/no-bail-skips-dependents` (docs fix) and `pnpr-oidc-workload-discovery`
  (one commit, "verify GitHub Actions workload tokens without login metadata", `pnpr/crates/auth`).
  None touches Cargo or Rust management.

### Could it serve (a) or (b)?

- (a): not served, though parts overlap.
  pnpm provides materialization (a content-addressed store, vendored sources, offline builds)
  that `cargo-shadow` does not,
  and `cargo-shadow` provides update and admission policy that pnpm was not found to provide for crates;
  they are complementary rather than redundant.
  What (a) needs, the links between our own crates and their publication, is not something pnpm does:
  cross-root path dependencies need a Cargo-generated lock first
  (and the `git` wrapper collision applies where Cargo must generate it; see "What it does not do,
  from source and probes").
- (b): no.
  The seven tools are programs installed by `mpm-shadow`; pnpm's Rust support installs a toolchain and library crates.
- Worth watching, not adopting now:
  Rust toolchain provisioning overlaps the `rust` row of `toolchain.plan.jsonc` (`rust@nightly`),
  already served by `mpm-shadow`.
  Conditions that would change this:
  a cross-root path dependency accepted without a prior lock,
  a `workspace:`-style rewrite for crates,
  crate tools installable globally,
  a trust policy for crates.

## Question 4: GitHub's package registry, re-checked

- The current docs list the supported registries as "npm, RubyGems, Apache Maven, Gradle, Docker, and NuGet"
  ([Introduction to GitHub Packages][ghp-intro]),
  and "Working with a GitHub Packages registry" lists the Container registry,
  RubyGems, npm, Maven, Gradle and NuGet; no Cargo.
- On authentication the same page says "GitHub Packages only supports authentication using a personal access token
  (classic)" ([same page][ghp-auth]); it does not say whether a public read needs one, so no conclusion about
  anonymous Cargo reads is drawn from it.
- Community discussion 12067, "Cargo (Rust) registry support for Github Packages",
  is category Packages, 16 comments, no accepted answer, last updated 2026-08-28T09:41:16Z
  (last comment 2026-08-28: "When are we getting this?"); discussion 11926 last updated 2023-05-11
  ([GitHub discussion 12067][gh-discussion-12067]).
- The documented "linked artifacts" page "does not host the package or image files themselves".
- The Container registry holds OCI artifacts, but Cargo supports only the git and sparse registry protocols
  (prior note, with the Cargo book quote), so it cannot be a Cargo registry.

## Options and ranking

Two axes matter: how crates depend on and publish to each other (use a),
and where the bytes of an installed tool come from (use b).
"Label decides" is the owner's rule: `-alpha.N` and `-beta.N` to pnpr only,
`-rc.N` and plain versions also to crates.io.

### Option 1: pnpr as the crate registry, tools stay on GitHub archives

- Use (a): works, run end to end (path plus version plus registry; alpha publish; anonymous consumer with checksums;
  dependency-first enforced).
- Use (b): unchanged archives from the immutable GitHub release; no step downloads or compiles a crate.
- Needs built:
  - dependency catalog (`package/dependency-catalog`)
    and `file-enforcer` sync accept and write `registry` on internal sites
    (an owner amendment to `doc/decision/rust-dependency-catalog.md`);
  - the release tool (`package/crate-release/cli`) publishes in dependency order with the key present for pnpr
    and a copy of the manifest without the key for the crates.io publish of rc and stable;
  - pnpr: the Cargo workload login that `doc/planning/self-hosted-cargo-registry.md` already plans,
    a named hosted registry,
    and one `packages:` entry per crate (exact names or `**`);
  - `mpm-shadow`: optionally an origin that reads the version from the checkout's `Cargo.toml`
    (the prior note's `workspace:`-like origin, a convenience over an exact request, not a requirement).
  `cargo-shadow` needs no change for internal dependencies, since their lock entries have no registry source.
- How "Label decides" fits: pnpr takes every label;
  crates.io takes rc and plain versions after the crate's whole dependency closure is on crates.io.
  Two checks are involved.
  The client check (Cargo resolving the dependency in the target registry's index while packaging)
  ran here against pnpr.
  The server check (crates.io refusing a dependency that names another registry)
  is the prior note's reading of crates.io's
  code; `cargo publish --dry-run` does not exercise it and no real publish to crates.io was run.
- How an `-alpha` release reaches this machine:
  today `github:Aquaticats/Monochromatic@latest` with `options.tagPrefix` skips releases GitHub flags as `prerelease`
  (`latest_release_with_prefix` in `package/mpm-shadow/cli/src/toolchain/update_github.rs`, lines 162 to 164,
  source reading).
  `gh release create` in `.github/workflows/cargo-publish.yml` passes no `--prerelease`,
  and each of the twelve newest releases listed on 2026-11-07 has `prerelease: false`,
  so alphas are not skipped today, and are skipped the day a workflow marks them.
  Separately, `compare_versions` in `package/mpm-shadow/cli/src/version.rs` compares pre-release tails as text
  (`left_pre.cmp(right_pre)`),
  so `0.6.1-alpha.10` is ordered before `0.6.1-alpha.9`;
  `latest_release_with_prefix` calls it (source reading, not run:
  the built selector was not exercised with these inputs).
  An exact request (`github:...@0.6.1-alpha.10` with the prefix) goes through `release_by_tag`,
  which in the source does no comparison and no prerelease filtering,
  so the predicted effect is that an exact request, and an origin that builds that request from the checkout's version,
  avoids both.
  An exact request in the lock plan already exists today as the baseline;
  the new origin only makes the lock follow the checkout instead of a hand-written version.
  Its contract would need: a missing release is an error that names the tag (the archive route never compiles),
  the release tag is created by the workflow after the version-bump commit, so a checkout with edits since that commit
  or ahead of the release has no matching bytes, and the lock then says nothing about those edits.
- Risks: the pnpr Cargo workload patch is not written; "No copies, like npm" means a lost store breaks outside locks
  (accepted by the owner for npm and crates);
  a crate rc needs everything it uses to be rc or stable on crates.io first;
  per-target manifests change the packed bytes, so one version has two checksums, one in each registry.

### Option 2: Option 1 with the tool bytes also on pnpr, as npm packages

- Use (a): as Option 1.
- Use (b): each tool ships per-platform binary packages on pnpr (pnpm itself ships as `pnpm` and `@pnpm/exe` whose
  bins are placeholders
  replaced by the host's `@pnpm/exe.<platform>` binary: [pnpm 10.34.5 release notes][pnpm-10-34-5]),
  installed through the existing `mpm-pnpm` route.
- Needs built: per-platform package builds and publishing from `cargo-publish.yml`;
  `mpm-shadow` configurable npm registry
  (its README, section "Network", says every npm version is looked up at `registry.npmjs.org`);
  a hash record (the `mpm-pnpm` rows in `toolchain.lock.json` carry `"artifacts": {}`, unlike the `archive` rows).
- How "Label decides" fits: npm dist-tags, with the version label deciding the tag.
- Risks: duplicates the release assets, more machinery than the archive route, and not probed here.

### Option 3: static sparse index on GitHub (Pages plus release assets)

- Use (a): same `registry` key, same catalog and release-tool changes as Option 1;
  the index is a generated artifact instead of a server.
- Use (b): `cargo-binstall --index` worked on a fixture, but `mpm-shadow` passes no `--index`,
  and the lock would record no hash.
  The prior note saw `--disable-strategies compile` stop with "Fallback to cargo-install is disabled" when the
  archive was
  missing (404), so a missing archive fails without compiling there; that was a fixture, not our tools.
- Needs built: an index generator and a workflow; Pages replaces the whole site on each deploy;
  `cargo publish` is refused (`api: null`).
- How "Label decides" fits: any label, but it ignores the owner's decision to deploy all releases on pnpr.
- Risks: release assets are immutable (per the prior note), Pages limits, and a second registry beside pnpr.

### Option 4: stay as now (GitHub archives for tools, `path` plus `version` only, no registry for libraries)

- Use (a): works today inside the repository (28 `path` plus exact `version` lines in 20 of 35 tracked manifests on
  2026-11-07).
- Use (b): works (the seven tools on the `archive` route; install measured 31.7 s with none compiled,
  per `doc/planning/crates-io-footprint.md`).
- Gap: under the owner's rule that crates.io takes only rc and stable,
  no registry resolves an `-alpha` version for anyone outside the repository
  (a git dependency still reaches any commit, prior note),
  and the owner's "deploy all releases on pnpr" is unmet.

### Option 5: pnpm `cargo.enabled` and the pnpm Rust toolchain

- Use (a): not served (cross-root path dependencies need a prior lock; one index per workspace;
  `git` wrapper collision).
- Use (b): not served (`crate:` cannot be a global tool).
- Fit: Rust toolchain provisioning overlaps `mpm-shadow`;
  pnpm's vendoring and `cargo-shadow`'s update and admission policy are complementary, not redundant.
- Marked experimental by pnpm; revisit under the conditions listed in Question 3.

### Option 6: floating git

- Use (a): blocked (the catalog refuses `branch`; a lock cannot name its own commit).
- Use (b): blocked (every git installer compiles;
  `cargo-binstall --git` cannot read a repository with no root `Cargo.toml`).
- Works only for outside consumers (prior note).

### Option 7: GitHub Packages

- Holds no crates (Question 4).

### Ranking

The two axes rank separately.
Crate registry: pnpr (Options 1 and 2) > static index (Option 3) > no registry (Option 4).
Tool bytes: GitHub archives (Option 1) > npm packages on pnpr (Option 2).
Option 3 conflicts with the recorded decision to deploy all releases on pnpr,
so it competes only if the owner reopens that
decision; it is ranked as reference material, not as a live choice.
Pnpm's Cargo support, floating git and GitHub Packages serve neither axis.
The combined order is:

Option 1 > Option 2 > Option 3 > Option 4 > Option 5 > Option 6 > Option 7.

- Option 1 over Option 2: Option 1's registry half was run end to end (publish of an alpha, anonymous build,
  checksums, dependency order)
  and its tool half already exists as the `archive` route plus an optional origin;
  Option 2 adds per-platform packaging and a configurable npm registry in `mpm-shadow` that were not probed.
  Option 2 would move the tool bytes off GitHub, which Option 1 does not.
- Option 2 over Option 3: this is a judgment about the owner's recorded decision, not a measured gap.
  Option 2 follows "deploy all releases on pnpr" and needs no index generator;
  Option 3 needs a generator and whole-site deploys for the registry protocol pnpr already serves,
  and `cargo publish` cannot target it.
- Option 3 over Option 4: Option 3 lets outside consumers resolve any label from a registry anonymously (a fixture
  resolved
  and installed from it);
  Option 4 offers no registry for the labels that crates.io does not take under the owner's rule.
- Option 4 over Option 5: Option 4 works today with measured installs;
  Option 5 failed on cross-root path dependencies without a prior lock,
  collided with the `git` wrapper on the Cargo-generated-lock path, and installs no tool.
- Option 5 over Option 6: Option 5 ran on independent roots once the locks existed;
  Option 6 is blocked by the catalog and compiles.
- Option 6 over Option 7: Option 6 serves outside consumers (prior note's test); Option 7 holds no crates.

### Owner decisions

- Whether internal dependencies carry a `registry` key,
  which needs an amendment to `doc/decision/rust-dependency-catalog.md`
  and a per-target manifest rule in the release tool (the key for pnpr, none for crates.io).
- Whether `@latest` stays for the seven tools or an origin follows the checkout's `Cargo.toml` version;
  and whether alpha releases are flagged `--prerelease` (flagged ones are skipped by `@latest`).
- Whether `compare_versions` should order numeric pre-release identifiers numerically; this is a fix, not a choice,
  but it changes what `@latest` selects.
- Whether tool bytes stay on GitHub (Option 1) or move to pnpr (Option 2).
- Whether `cargo.enabled` is wanted at all, given the `git` wrapper collision (the wrapper rule is the owner's).
- Whether a pnpr `packages:` entry per crate or `**` is wanted.

## Cross-vendor review

A review by `openai/gpt-6.1-sol` (no tools, whole note supplied, run 2026-11-07, output kept in
`~/temp/agent/pnpm-rust-2026-11-07/sol-review.txt`) raised findings.
Each was fixed or recorded as follows.

- Dependency-first conclusion too broad (high): narrowed to the tested `cargo publish` workflow;
  the batch route needs a release tool that builds the archive without Cargo resolving against the registry;
  both unprobed.
- Version substitution omitted: added the property-by-property comparison and a local version-validation probe;
  Cargo does not substitute versions.
- Claim about what the npm registry checks: removed, since the pnpm page does not support it and it was not probed.
- Two dependency checks conflated: named the client check (run) and the server check (prior note's source reading,
  not run).
- Source-replacement lock conclusion too broad: narrowed to the tested checksum-less lock,
  with the untried variants listed.
- pnpm summary contradicted the seeded-lock result (high): rewritten; the fixture had no third-party crates,
  which is now stated.
- Git-wrapper collision promoted without inventory: inventory added (no `[patch]`,
  two manifests with `git` dependencies,
  every manifest has a lock), and the collision is now conditional.
- Policy absence overstated: reworded to "none found in these searches", with the missing test listed.
- Vendoring and policy treated as one capability: now complementary.
- `--locked` in a clean checkout: ran `cargo build --locked --offline` with the generated lock and narrowed the claim.
- Checkout-version origin had no contract: contract and exact-request baseline added; "small" removed.
- Source-reading conclusions stated as behaviour: labelled predicted.
- Binstall fixture and compile fallback: cited the prior note's observed failure message; not run for our tools.
- Status quo overstated: reworded to registry resolution under the owner's rule, with git noted.
- Ranking mixed axes and a binding decision: per-axis ranking added;
  Option 3 treated as reference unless the decision reopens.
- "No credential read" claim: replaced by exact statements about `gh`, `CARGO_HOME` and the throwaway token.
- Local publication versus "never publish anything":
  kept as a disclosed deviation from the brief for the caller to judge.
- GitHub authentication excerpt: inference about anonymous reads removed.
- "sorts below": reworded.

## Not verified

- A batch publish with Cargo entries and whether the server checks dependencies inside the batch.
- `cargo-binstall` against pnpr, and `cargo publish` of rc versions to crates.io (never run).
- npm-packaged tools (Option 2) and the real GitHub redirect for a static-index `dl` address.
- `compare_versions` and `latest_release_with_prefix` behaviour by running the built `mpm-shadow` (source reading only).
- A pnpm `cargo.enabled` install in the real repository (fixtures only).

[pnpm-ws-resolve]: https://pnpm.io/workspaces#:~:text=pnpm%20will%20refuse%20to%20resolve%20to%20anything%20other%20than%20a%20local%20workspace%20package
[pnpm-ws-replace]: https://pnpm.io/workspaces#:~:text=we%20dynamically%20replace%20any%20workspace%3A%20dependency%20by
[pnpm-ws-nosteps]: https://pnpm.io/workspaces#:~:text=without%20needing%20intermediary%20publish%20steps
[pnpm-ws-bare]: https://pnpm.io/workspaces#:~:text=A%20bare%20workspace%3A%20without%20a%20version%20range%20is%20treated%20as%20workspace%3A*
[pnpm-cargo-experimental]: https://pnpm.io/cargo#:~:text=Multi%2Decosystem%20support%20is%20experimental
[pnpm-cargo-nocompile]: https://pnpm.io/cargo#:~:text=pnpm%20does%20not%20compile%20anything%20itself
[pnpm-cargo-git]: https://pnpm.io/cargo#git-sourced-crates
[pnpm-global]: https://pnpm.io/global-packages#:~:text=crate%3A%20and%20Python%20dependencies
[pnpm-cargo-registry-line]: https://github.com/pnpm/pnpm/blob/0470052291b2ba369be93a9b696e0a0097722ba5/pnpm/docs/cargo.md#L75
[pnpm-not-member]: https://github.com/pnpm/pnpm/blob/0470052291b2ba369be93a9b696e0a0097722ba5/pnpm/crates/cargo-resolver/src/lockfile.rs#L216-L223
[pnpm-discover-roots]: https://github.com/pnpm/pnpm/blob/0470052291b2ba369be93a9b696e0a0097722ba5/pnpm/crates/cli/src/cargo_deps/lockfile.rs#L108-L139
[pnpr-batch]: https://github.com/pnpm/pnpm/blob/0470052291b2ba369be93a9b696e0a0097722ba5/pnpr/docs/ecosystems.md#L200-L257
[pnpm-12-4-0]: https://github.com/pnpm/pnpm/releases/tag/v12.4.0
[pnpm-12-5-0]: https://github.com/pnpm/pnpm/releases/tag/v12.5.0
[pnpm-12-11-0]: https://github.com/pnpm/pnpm/releases/tag/v12.11.0
[pnpm-12-17-0]: https://github.com/pnpm/pnpm/releases/tag/v12.17.0
[pnpm-12-18-0]: https://github.com/pnpm/pnpm/releases/tag/v12.18.0
[pnpm-10-34-5]: https://github.com/pnpm/pnpm/releases/tag/v10.34.5
[ghp-intro]: https://docs.github.com/en/packages/learn-github-packages/introduction-to-github-packages#:~:text=such%20as%20npm%2C%20RubyGems%2C%20Apache%20Maven%2C%20Gradle%2C%20Docker%2C%20and%20NuGet
[ghp-auth]: https://docs.github.com/en/packages/learn-github-packages/introduction-to-github-packages#:~:text=GitHub%20Packages%20only%20supports%20authentication%20using%20a%20personal%20access%20token%20(classic)
[gh-discussion-12067]: https://github.com/orgs/community/discussions/12067
[cargo-patch-root]: https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html#:~:text=Cargo%20only%20looks%20at%20the%20patch%20settings
[cargo-replace-deprecated]: https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html#:~:text=%5Breplace%5D%20is%20deprecated
[cargo-paths-note]: https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html#:~:text=You%20cannot%20use%20this%20feature%20to%20tell%20Cargo%20how%20to%20find%20local%20unpublished%20crates
[cargo-dirsource]: https://doc.rust-lang.org/cargo/reference/source-replacement.html#:~:text=Each%20crate%20in%20a%20directory%20source%20also%20has%20an%20associated%20metadata%20file
[cargo-unstable]: https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/doc/book/src/reference/unstable.md
[cargo-unstable-pathbases]: https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/doc/book/src/reference/unstable.md#L1715
[cargo-package-workspace]: https://github.com/rust-lang/cargo/blob/c07ac2c4d12237effc4b94f4543769b421790045/doc/book/src/reference/unstable.md#L2313
