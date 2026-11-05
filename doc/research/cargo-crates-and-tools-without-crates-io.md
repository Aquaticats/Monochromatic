# Cargo crates and tools without crates.io: which of the owner's three routes work

Research note,
written at 2026-11-04T19:45:40-05:00 (the question and answers are dated 2026-11-05 in
`doc/planning/crates-io-footprint.md`).
It decides nothing:
every ranking here is a proposal for the owner,
and nothing on crates.io,
in GitHub,
in a registry
or in the repository other than this file was changed for it.

## Verdicts in one place

- A `workspace:`-like protocol is workable for uses (a),
  (b) and (d),
  and not for use (c).
  Cargo has the equivalent for the manifest already,
  and the repository already uses it (`path` plus an exact `version`).
  The protocol governs the manifest only:
  it does not decide where a tool installs from or whether a publish needs
  crates.io,
  because `cargo package` still requires every `path` plus `version` dependency to exist in the target registry.
  Tools become independent of crates.io through the `archive` route that `mpm-shadow` already installs itself with:
  the prebuilt archive and its SHA-256 come from the immutable GitHub release.
  A new `mpm-shadow` origin that reads the version from the checkout's `Cargo.toml` is only needed
  if the owner wants the lock to follow the checkout rather than the newest release.
- Floating git is workable for use (c) and for the version-field part of use (d),
  and not for uses (a) and (b).
  The dependency catalog refuses `branch`,
  and a lock file cannot name the commit it is committed in.
  Every installer that accepts a git source compiles the tool (`mpm-shadow`'s `cargo-git`,
  mise's cargo backend),
  and
  `cargo-binstall --git` fails on a repository with no root `Cargo.toml`,
  which is this repository's layout.
- The GitHub package registry,
  meaning GitHub Packages,
  is not workable:
  it has no Cargo registry.
  What on GitHub can serve a Cargo registry is a static sparse index with the `.crate` files as release assets.
  That form is workable for uses (b),
  (c) and (d) (a hand-built fixture here resolved from it,
  and `cargo-binstall` installed a prebuilt
  program from it),
  and for use (a) only after the dependency catalog and `cargo-shadow` accept a `registry` source.
- No route lets a crate that crates.io receives depend on a crate that only another registry or git holds.
  `cargo publish --dry-run` does not show this:
  two of the dry runs in the section "The `cargo publish --dry-run` trap" passed,
  and crates.io's server would refuse both.
- The route the sources show best fits the owner's intent:
  tools become `publish = false` with `path`-only dependencies and install through the `archive` route that
  `mpm-shadow` already uses for itself;
  the libraries that must stay reusable are the only crates that still go to a registry,
  and they go less often.

## The owner's words

Verbatim,
from `doc/planning/crates-io-footprint.md` (section "The owner's answers",
lines 11 to 22):

- "Can we avoid polluting crates.io too much?"
- "Not enough research.
  The best solution is obviously a workspace:
  protocol or something.
  The second best solution is floating git.
  The third best solution is GitHub package registry.
  Figure out which ones are workable first."
- "One release each.
  And we publish less to crates.io .
  That fact is given."

Other owner words quoted in the repository that bear on this:

- "did you miss how we can't really use cargo workspace?"
  and "I just realized something.
  We should not put everything in one crate."
  and "Crates obviously have to be reusable,
  otherwise there's no point."
  (`doc/planning/cli-git-source-layout.md`,
  lines 66 to 79).
- "The point of cargo-binstall is for us to not manually point to gh releases ourselves!"
  (owner decision 2026-11-04,
  `package/mpm-shadow/cli/README.md`,
  section "Cargo tool installs").

## Constraints that stay in force

- "Every Rust crate is its own Cargo workspace root" (`doc/decision/rust-dependency-catalog.md`,
  line 15).
  Internal crates "are path dependencies whose `version` the sync writes as `=<target package.version>`,
  the counterpart of `workspace:*`" (same file,
  lines 56 to 58).
  `.out-of-scope/cargo-workspace.md` gives the reason:
  `[profile.*]` is read only in a workspace root manifest.
  So `[workspace]`,
  `workspace = true` inheritance and `cargo publish --workspace` are not available.
- Rule CM7 in `AGENTS.md`:
  "Install tools only via `mise run toolchain:install` (containers,
  CI:
  the lock row's archive and
  sha256);
  never compile a locked tool this repo publishes".
- The owner chose hosting our own crates in a self-hosted registry with anonymous downloads,
  third-party dependencies staying on
  crates.io (`doc/planning/self-hosted-cargo-registry-protocol.md`,
  lines 3 to 8,
  and
  `doc/planning/self-hosted-cargo-registry.md`,
  lines 3 to 14).
  That is a proposal at a stated revision,
  not a deployed endpoint.
- The existing crates get one more release each,
  and crates.io publishing continues but less
  (the owner's answers quoted in "The owner's words").

## What was read, run and left unverified

- Repository at `187ecd2522f97d3264425b7ae3ddb37f6ba36266`.
- Cargo source and book at `d6b1b43ed13a7e12072b5f1685056719546e127a` (a shallow clone another session had left in
  `~/temp/agent/cargo-2026-11-05`;
  it was only read).
  The Cargo that ran the experiments is `cargo 1.101.0-nightly (31d3246e0 2026-10-27)`
  with `rustc 1.101.0-nightly (f9ceb4a4d 2026-10-31)`.
- cargo-binstall source at `90e4ee1101c5c9818ad29c4904d3cec4122bac0e`;
  the program that ran is the locked 1.26.2
  (`toolchain.lock.json`,
  tool `cargo-binstall`).
- mise source at `f3b85fce6faf6b0e37168bab2b360a92abe88d1c`.
- margo at `c352ce63ffd96c0c86d0f251fbf60751131dae6b`,
  margo-actions at `f65dceeae7a46c2f805d5b2e090b2a8f8444255c`,
  simple-gh-cargo-registry at `6c8ad3fb55e30db0ba639b55ce87a702254896b4`.
- crates.io source at `e0edaf98831e94b4a6d0969b957bda0094a45f3d`,
  the revision `doc/planning/crates-io-footprint.md` already cites.
- Experiments used disposable fixtures only,
  in `~/temp/agent/cargo-routes-2026-11-05/`,
  with their own `CARGO_HOME`.
  Every command ran in its own `systemd-run --user --scope` with `CPUQuota=200%`,
  `MemoryMax=2G`,
  `TasksMax=128`
  and `IOWeight=10`;
  the local registry server ran as a transient unit with `MemoryMax=256M`,
  `CPUQuota=100%`,
  `TasksMax=32` and `IOWeight=10`,
  and was stopped afterwards.
  No `cargo publish` ran without `--dry-run`,
  no token existed in the environment,
  and nothing was uploaded.
  Several `cargo` commands read the public crates.io index;
  that is a read.
- The fixture is a two-crate and a three-crate monorepo:
  `package/<name>/Cargo.toml`,
  no root manifest,
  each crate its own
  `[workspace]` root,
  as in this repository.

## Option 1: a `workspace:`-like protocol

### What pnpm does

- Resolution:
  "When this protocol is used,
  pnpm will refuse to resolve to anything other than a local workspace package."
  Source:
  [pnpm workspaces][pnpm-ws-resolve].
- Rewrite at publish:
  "When a workspace package is packed into an archive ... we dynamically replace any workspace:
  dependency" by the target's version,
  or by the range with the version for `workspace:^1.5.0`.
  The page states the point of the feature:
  "without needing intermediary publish steps".
  Source:
  [pnpm workspaces, "Publishing workspace packages"][pnpm-ws-publish].
- Cargo's contrasting behaviour,
  that packaging needs the dependency in the target registry,
  is tested in the next section.

### Cargo's equivalents, tested

- `path` plus `version` is the counterpart.
  The Cargo book:
  "The git or path dependency will be used locally (in which case the version is checked against the local
  copy),
  and when published to a registry like crates.io,
  it will use the registry version."
  Source:
  [Specifying dependencies, "Multiple locations"][cargo-multi-loc].
  This is what the repository already writes,
  with `=x.y.z`,
  through the dependency catalog.
- `cargo package` does not rewrite for free.
  Test (fixture crate `fixroutes-tool-b` with `{ path = "../lib-a", version = "=0.1.0" }`,
  library not on any registry):
  `cargo package --no-verify --allow-dirty` failed with
  `no matching package named 'fixroutes-lib-a' found, location searched: crates.io index`.
  So every internal dependency must already exist in the registry the package is prepared against.
  This is why the repository publishes dependencies first.
- `publish = false` with `path`-only dependencies builds and runs,
  and refuses packaging and publishing.
  Test:
  removing `version` and adding `publish = false` to the tool:
  `cargo build` and the program ran (it printed `1`);
  `cargo package` stopped with "all dependencies must have a version requirement specified when packaging";
  `cargo publish --dry-run` stopped with
  "`package.publish` must be set to `true` or a non-empty list in Cargo.toml to publish".
  The Cargo book says the same:
  "Crates that use dependencies specified with only a path are not permitted on crates.io."
  Source:
  [Specifying dependencies, "Local paths in published crates"][cargo-local-paths].
  Cargo source for the first message:
  `check_dep_has_version`,
  [`src/ops/mod.rs` L84 to L110][cargo-check-dep].
- `[patch.crates-io]` in the manifest points a plain `"=0.1.0"` requirement at a path.
  It built and ran,
  then `cargo package` failed with the same "no matching package" error.
  The Cargo book:
  "Cargo only looks at the patch settings in the `Cargo.toml` manifest at the root of the workspace.
  Patch settings defined in dependencies will be ignored."
  and `cargo package` removes `[patch]` from the packaged manifest.
  Sources:
  [Overriding dependencies][cargo-patch-root],
  [`cargo package`, step 2][cargo-package-strips].
  Each crate here is its own root,
  so every crate would carry its own copy of the patch table.
- `[patch]` in `.cargo/config.toml` (stable since 1.56) is the nearest thing to one central mapping.
  Test:
  a repository-level `.cargo/config.toml` with `[patch.crates-io] fixroutes-lib-a = { path = "lib-a" }`,
  the tool
  depending on plain `"=0.1.0"`.
  Result:
  the tool built;
  `cargo package --allow-dirty` including its verify build passed;
  `cargo publish --dry-run`
  reached "Uploading ... aborting upload due to dry run".
  The same crate copied outside the directory tree of that config failed with the "no matching package" error.
  The packaged manifest holds `[dependencies.fixroutes-lib-a] version = "=0.1.0"`,
  so the published crate would
  depend on a crates.io crate that does not exist.
  This hides the one guard that makes dependency-first publication fail loudly,
  so it is a trap,
  not a solution.
- Multi-package publish exists but needs a workspace.
  The Cargo changelog for 1.90:
  "Stabilize multi-package publishing.
  This allows cargo to publish multiple crates in a
  workspace,
  even if they have inter-dependencies.
  For example,
  `cargo publish --workspace` or `cargo publish -p foo -p bar`."
  Source:
  [Cargo changelog, 1.90][cargo-1-90];
  the `package-workspace` section of the unstable book now says it was
  "stabilized in Rust 1.90.0".
  Test,
  separate roots:
  `cargo publish --dry-run -p fixroutes-lib-a -p fixroutes-tool-b` from the tool's directory
  failed with "package ID specification `fixroutes-lib-a` did not match any packages".
  Test,
  one `[workspace]` with both as members:
  the same command passed in dry run,
  "Uploading" both,
  aborted.
  So it is unavailable under the "own workspace root" decision.
- Other unstable features that look related do not help:
  `path-bases` (Cargo issue 14355,
  open) only shortens `path` values;
  the cross-root inheritance issues rust-lang/cargo#5042 ("Nested workspaces")
  and #15435 ("Remote Version Definitions") are
  open,
  as `doc/decision/rust-dependency-catalog.md` line 19 already records.

### Uses (a) to (d)

- (a) A crate depending on another inside this repository:
  already served.
  `path` plus `=x.y.z`,
  generated by file-enforcer,
  is the protocol.
  Nothing in Cargo needs to change.
  The only gap is the packaging test in "Cargo's equivalents,
  tested":
  packaging needs the dependency in the target registry.
- (b) Installing our tool prebuilt,
  pinned in `toolchain.lock.json`:
  workable today through the existing `archive` route;
  new `mpm-shadow` code is needed only for a checkout-version origin.
  What exists (`package/mpm-shadow/cli`):
  the `archive` route;
  the `github:<owner>/<repo>` origin with `options.tagPrefix`,
  which already accepts an exact range,
  so
  `github:Aquaticats/Monochromatic@0.5.34` with `"tagPrefix": "monochromatic-git-policy-cli-v"` resolves release
  `monochromatic-git-policy-cli-v0.5.34` (`src/toolchain/update_github.rs`,
  line 271:
  `(Some(start), Some(range))`);
  the SHA-256 for each Linux target,
  taken from the digest GitHub reports for the asset (`asset_digest`,
  line 121);
  the postinstall links;
  the never-compile rule for repository-published tools.
  The `mpm-shadow` row in `toolchain.lock.json` (lines 689 to 729) is exactly this route with four URLs and hashes.
  Measured on the GitHub API:
  release `monochromatic-git-policy-cli-v0.5.34` is `immutable: true` and its assets carry
  `digest: sha256:...`.
  What would be new,
  and only if the lock should follow the checkout instead of the newest release:
  an origin such as `workspace:package/git-policy/cli` whose resolver reads `package.name` and
  `package.version` from that checkout's `Cargo.toml` and then does what `resolve_github` does for the tag
  `<name>-v<version>`.
  That is a small addition on top of the existing resolver,
  in the shape of "a version from the package's Cargo.toml,
  the prebuilt archive from the matching GitHub release tag".
  Compared with the `cargo-binstall` route this adds a hash per archive;
  the `cargo:` rows record `"artifacts": {}`.
  Compared with `github:...@latest`:
  `@latest` takes the newest release,
  the new origin takes the checkout's own version.
  Which one the owner wants is an open question,
  because `crate-release.yml` commits the version bump first and the release
  appears later,
  so the lock cannot be updated in the interval.
- (c) A consumer outside this repository:
  not served by the protocol.
  A `path` dependency means nothing outside the checkout.
  The consumer needs a registry or git (options 3 and 2).
- (d) Coexisting with the continuing crates.io releases:
  works only for crates whose whole dependency closure is on crates.io.
  The published manifest keeps only `version`,
  and `cargo package` needs the version on crates.io (the packaging test in "Cargo's equivalents,
  tested").
  A crate that depends on a `publish = false` crate cannot be packaged at all (the `publish = false` test there).
  Therefore the set of crates that still go to crates.io must be closed under dependencies,
  and `publish = false`
  can be set on a crate only after every crate that must stay published has stopped depending on it.
  The owner's "One release each" must land before a tool is switched to `publish = false`,
  because the extra release is a
  publish.

### Cost here

- Tools:
  `publish = false` and `path`-only internal dependencies (so no generated `version` pin;
  the catalog's
  rule for internal crates changes from "path plus `=version`" to "path only" for tools);
  `cargo-publish.yml` loses the
  `publish-crate` jobs of tools and keeps `build-binary` and `create-release`;
  `toolchain.plan.jsonc` moves each of the seven
  `cargo:` rows to the `archive` route (this part is the option A of `doc/planning/crates-io-footprint.md`).
- New code:
  one `mpm-shadow` origin (the `workspace:` form in use (b)),
  only if the owner wants the checkout's version rather than `@latest`.
- Crates that stay published are unchanged.

### Unverified

- Whether a lock updated from the checkout's version can race the release (the bump commit precedes the release).
- That every postinstall form the seven tools use works on the `archive` route (already listed in
  `doc/planning/crates-io-footprint.md`,
  "Not verified yet").
- Musl platforms:
  the existing lock row for `mpm-shadow` maps them to the GNU archive;
  the tools' release archives are
  GNU only,
  per `package/git-policy/cli/Cargo.toml`.

## Option 2: floating git

### What Cargo does with git sources

- Locating a crate in a monorepo:
  "Cargo fetches the git repository at that location and traverses the file tree to find
  Cargo.toml file for the requested crate anywhere inside the git repository."
  Source:
  [Specifying dependencies, "Specifying dependencies from git repositories"][cargo-git-traverse].
  Test:
  a consumer with `{ git = "file:///.../mono2", branch = "main" }` built `fixroutes-lib-c` from
  `package/lib-c`,
  a crate whose repository has no root manifest,
  and its `path` dependency on `../lib-a` came along.
  The lock recorded both as `git+file:///...?branch=main#<commit>`.
- Floating and pinning:
  "Cargo locks the commits of git dependencies in Cargo.lock file at the time of their addition and
  checks for updates only when you run cargo update command."
  Source:
  [same page, "Choice of commit"][cargo-git-lock].
  Test:
  after a new commit on `main`,
  `cargo run --locked` still ran the old commit;
  `cargo update` printed
  `Updating ... #444990bd -> #c30812cf` for both crates of that repository and the program's output changed.
  So "floating" means floating at `cargo update`,
  one commit hash for the whole repository.
- `version` beside `git`:
  "The version key does not affect which commit is used ... but Cargo checks the version information
  in the dependency's Cargo.toml file against the version key."
  Source:
  [same page, "The role of the version key"][cargo-git-version].
- Publishing:
  git-only is refused;
  `git` plus `version` is accepted only when the version is on the registry.
  Tests with the consumer packaged:
  git only gave "all dependencies must have a version requirement specified when packaging
  ... the `git` specification will be removed";
  `git` plus `version` gave "no matching package named `fixroutes-lib-c` found,
  location searched:
  crates.io index",
  for `cargo package`,
  and for `cargo publish --dry-run` as well.
  The Cargo FAQ promises git sources "forever" for early development and temporary patches:
  [Cargo FAQ][cargo-faq-git].
- Shallow fetch of a git dependency is unstable (`-Zgit=shallow-deps`):
  [`unstable.md`, section "git"][cargo-unstable-git].
  Without it a consumer fetches this repository's history;
  GitHub reports the repository at 714204 KB
  (`gh api repos/Aquaticats/Monochromatic`,
  size field),
  and a local clone's pack is about 709 MiB.

### Installing a prebuilt tool from a git source

- `cargo-binstall --git <url>`:
  "This skips searching crates.io and instead clones the repository specified and runs as if
  `--manifest-path $cloned_repo` is passed to binstall."
  Source:
  [`crates/bin/src/args.rs` L122 to L135][binstall-git-arg].
  The code shallow-clones and calls `load_manifest_from_workspace(dir, name)`
  ([`resolve.rs` L458 to L497][binstall-git-resolve]),
  which reads `<dir>/Cargo.toml` and then walks only
  `workspace.members` ([`cargo-toml-workspace/src/lib.rs` L67 to L125][binstall-ws-walk]).
  Test (binstall 1.26.2):
  against a fixture repository with `package/tool-b/Cargo.toml` and no root manifest,
  `--git` failed with
  "Failed to load fixroutes-tool-b from /tmp/.tmp...:
  Failed to parse cargo manifest:
  No such file or directory".
  This repository has no root `Cargo.toml` and forbids a root workspace,
  so `--git` cannot find any of its crates.
  The help text offers no revision option;
  whether the underlying `simple-git` crate takes one was not read.
- `cargo-binstall --manifest-path <dir>` skips crates.io entirely,
  and it works here.
  Test:
  `--manifest-path .../tool-b --disable-strategies compile,quick-install` fetched only the release archive named by
  `[package.metadata.binstall]` (the server log shows one `GET /assets/fixroutes-tool-b-0.1.0.tar.gz` and no index or
  `config.json` request).
  Without `--version` it used the manifest's version;
  with `--version 0.9.9` it asked for `...-0.9.9.tar.gz`,
  got 404,
  and
  stopped with "Fallback to cargo-install is disabled".
  (When a manifest path is given,
  the requested exact version replaces the manifest's:
  [`resolve.rs` L512 to L516][binstall-version-override].)
  That is the closest existing `cargo-binstall` form of "version from the package's Cargo.toml,
  archive from the release".
  It is not a floating git route,
  it needs a checkout,
  and `mpm-shadow` does not call it.
  A `cargo:` row installed this way would still record no SHA-256,
  unlike the `archive` route.
- mise's cargo backend:
  "A Git source always builds with `cargo install --git`."
  Source:
  [mise `docs/dev-tools/backends/cargo.md` L73][mise-cargo-git].
  It compiles,
  so CM7 refuses it for our own tools.
- `mpm-shadow` already has a git origin:
  `git:<url>@<revision>` with the `cargo-git` route,
  which runs
  `cargo install --git <url> --rev <rev> --root <dir> <package>`
  (`src/toolchain/install_cargo.rs`,
  lines 123 to 138).
  It compiles,
  which CM7 refuses for repository-published tools,
  and the lock records no hash.
  Its resolver does nothing:
  `Origin::Git(_) => return Ok(by_name(range))` (`src/toolchain/update_resolve.rs`,
  line 218),
  so the
  range is the revision and nothing floats.
  "A lock that follows `main` and still records a hash" therefore does not exist for git.
  What floats and hashes today is the `github:` origin with `@latest` and `tagPrefix` (the `mpm-shadow` row).
  A lock that followed `main` rather than releases would need a build in the install step,
  because only releases carry
  archives.

### Do the repository's own tools refuse git sources the way they refuse registries?

- `cargo-shadow` does not refuse a git source.
  `package/cargo-shadow/cli/src/lockfile.rs` line 122 to 123 documents that git dependencies "have a `git+` source"
  and that neither
  they nor path dependencies "go through the registry's release-age and trust rules";
  line 148 skips every source that is not
  `registry+` or `sparse+`.
  The refusal at line 169 applies only to registry sources other than the two crates.io ones.
  The cost of that skip:
  a floating git dependency is unchecked by the release-age and trust policy.
- The dependency catalog accepts `git` only with a full-hash `rev`,
  and refuses `branch` and `tag`.
  `package/dependency-catalog/core-native/src/site_classify.rs` line 38 lists the keys a use site may carry (`git`,
  `rev`
  among them,
  no `branch`,
  no `tag`);
  line 196 says "pin git dependencies with `rev`";
  lines 208 to 216 report
  "git dependency has no `rev`" and ask for `rev = "<full commit hash>"`;
  `Entry::Git` in `types.rs` line 46 has only `git`,
  `rev` and `package`.
  A `branch = "main"` use site would be reported as "declares branch,
  which the Rust catalog does not support"
  (source reading;
  the check was not run).
  Floating git between our crates therefore needs an owner amendment to `doc/decision/rust-dependency-catalog.md`
  ("git entries carry `git` and `rev`",
  line 34).
  These are source findings,
  not a reproduced file-enforcer run.

### Uses (a) to (d)

- (a) Between crates of this repository:
  not workable.
  A git dependency on the same repository resolves to a different checkout from the working tree,
  so a local edit to
  a library is not seen by its dependent without a `[patch]` override.
  The dependent's `Cargo.lock` must name a commit that already exists,
  so it can never name the commit that contains it;
  every library change would need a second commit that moves the lock.
  The catalog also refuses `branch`.
  This is reasoned from the Cargo book quotes and the lock-pinning test in this section,
  not tried in the real repository.
- (b) Installing a prebuilt tool:
  not workable.
  Every git installer compiles (`cargo install --git`,
  mise,
  `mpm-shadow` `cargo-git`),
  and the one that downloads a prebuilt
  program,
  `cargo-binstall --git`,
  cannot read this layout.
- (c) A consumer outside:
  workable.
  `{ git = "https://github.com/Aquaticats/Monochromatic" }` finds a crate by name anywhere in the tree and its `path`
  dependencies come with it;
  the consumer's `Cargo.lock` pins the commit.
  Costs:
  the consumer fetches the whole repository;
  no semantic version resolution (the `version` key is only a check);
  a
  commit,
  not a release,
  is what is pinned.
- (d) Coexisting with crates.io releases:
  `git` plus `version` is allowed in a manifest and the version part is what is
  published,
  so it coexists only where the version is on crates.io.
  Git-only dependencies make a crate unpublishable.

### Cost here

- If used for (c) only:
  no repository change;
  the consumer writes the dependency.
- For anything between crates:
  a catalog amendment,
  new keys in `LITERAL_SITE_KEYS`,
  an owner decision on whether
  `cargo-shadow` should apply trust rules to git sources,
  and a `[patch]` story for local edits.

### Unverified

- Whether `simple-git` (used by `cargo-binstall --git`) can clone a given revision.
- `-Zgit=shallow-deps` against GitHub for this repository.
- A git dependency on this repository's real layout from an outside consumer (tested on a fixture of the same shape).

## Option 3: the GitHub package registry

### GitHub Packages has no Cargo registry

- GitHub's overview names the supported registries:
  "such as npm,
  RubyGems,
  Apache Maven,
  Gradle,
  Docker,
  and NuGet".
  Source:
  [Introduction to GitHub Packages][gh-packages-intro].
  The page "Working with a GitHub Packages registry" lists the same ecosystems and the Container registry,
  and no Cargo
  ([page][gh-packages-working]).
- The request has stood open in the community discussions:
  "Are there any plans on supporting Cargo and Rust in Github
  Packages?"
  ([discussion 12067][gh-discussion-12067]);
  a GitHub staff reply there says the near-term focus is Actions
  integration,
  and commenters report no roadmap entry.
  Discussion [11926][gh-discussion-11926] asks the same.
- Authentication would not fit even if it existed:
  "GitHub Packages only supports authentication using a personal access token
  (classic)",
  and "You need an access token to publish,
  install,
  and delete private,
  internal,
  and public packages"
  (same page as the overview).
  The Container registry page adds only that
  "You can also access public container images anonymously" ([page][gh-ghcr]).
  Cargo sends a token only when the index's `config.json` sets `auth-required`.

### GHCR (OCI) as a Cargo registry

- Cargo supports two registry protocols and only these:
  "Cargo supports two remote registry protocols:
  `git` and `sparse`."
  Source:
  [Cargo registries][cargo-protocols] (the same sentence is `doc/book/src/reference/registries.md` L110 in the clone).
  Neither is OCI.
- No Cargo issue or RFC for OCI registries was found.
  Searches run with `gh search issues --repo rust-lang/cargo oci`,
  `... --repo rust-lang/rfcs "OCI"` and
  `... --repo rust-lang/cargo "container registry"` returned nothing about an OCI-backed registry
  (hits were unrelated Docker build issues).
  That is a narrow search;
  it supports "none found",
  not "none exists".
- GHCR is therefore a place to store files,
  not something Cargo can resolve from.

### What on GitHub can serve a Cargo registry

- A static sparse index.
  The Cargo book:
  the `dl` value of the index's `config.json` takes `{crate}`,
  `{version}`,
  `{prefix}`,
  `{lowerprefix}` and
  `{sha256-checksum}`;
  `api` "is optional,
  but if it is not specified,
  commands such as `cargo publish` will not work";
  the download endpoint may redirect ("Cargo supports https,
  http,
  and file URLs,
  HTTP redirects").
  Source:
  [Registry index, "Index Configuration" and "Download Endpoint"][cargo-index-config].
  So an index of plain files plus `.crate` files served by any host is a complete read-only registry.
- Existing tools:
  - margo ("an alternate Cargo registry that is built using completely static files",
    README L3 to L4):
    "Can depend on crates from
    registries other than crates.io",
    "Dependencies are not required to exist" (README L93 to L94);
    it writes `api: None`
    (`src/main.rs` L490) and its README points to margo-actions for GitHub Pages.
    [margo at c352ce6][margo-readme].
  - margo-actions:
    publishes a crate to a margo registry in Actions with `GITHUB_TOKEN`,
    then the workflow commits to the
    Pages branch ([README L35 to L53][margo-actions-readme]).
  - simple-gh-cargo-registry:
    a proof of concept that serves `.crate` files from `raw.githubusercontent.com`,
    index in
    a git repository,
    "It doesn’t have luxuries like a publish API" ([README][simple-gh-readme],
    L29).
  Their documentation,
    not their code behaviour,
    is what was read;
    the fixture in the next bullet is what was run.
- Test with a hand-written index (no margo):
  `config.json` with
  `"dl":"http://127.0.0.1:8098/dl/{crate}-v{version}/{crate}-{version}.crate"`,
  `"api":null`,
  `"auth-required":false`;
  two index files with `cksum`;
  a server that answers the `dl` path with a 302 to the file.
  - A consumer with `fixroutes-lib-a = { version = "=0.1.0", registry = "mono" }` built and ran;
    its lock recorded
    `source = "sparse+http://127.0.0.1:8098/"`.
  - The inline markers and the redirect worked,
    which is the shape of a GitHub release URL:
    `https://github.com/Aquaticats/Monochromatic/releases/download/{crate}-v{version}/{crate}-{version}.crate`
    matches the
    tag form `monochromatic-git-policy-cli-v0.5.34`.
    The real GitHub redirect to its object store was not exercised.
  - `cargo-binstall` 1.26.2 with `--index sparse+http://127.0.0.1:8098/`,
    `--disable-strategies compile,quick-install` and the crate `fixroutes-tool-b`
    read the metadata from the registry's `.crate` and installed the prebuilt program (the program ran).
    It needed `--allow-insecure-http` only because the fixture was plain http;
    GitHub Pages is https.
- Limits to design around,
  all from GitHub:
  - Pages:
    "Published GitHub Pages sites may be no larger than 1 GB" and "a soft bandwidth limit of 100 GB per month"
    ([limits][gh-pages-limits]),
    so only the index belongs there.
  - Releases:
    "Each file included in a release must be under 2 GiB.
    There is no limit on the total size of a release,
    nor
    bandwidth usage."
    ([About releases][gh-releases-quota]).
  - Immutable releases:
    "Release assets cannot be modified or deleted" ([Immutable releases][gh-immutable]).
    This is the exact-bytes retention that
    `doc/planning/self-hosted-cargo-registry.md` (lines 79 to 82) lists as an open
    decision,
    because the index records each archive's SHA-256.
    This repository's releases are already immutable (`gh api` on the newest release:
    `"immutable": true`;
    the workflow comment
    in `.github/workflows/cargo-publish.yml` says the same).
    Among the newest hundred releases only tools (and `forbidden-strings`) have release families,
    so each library would need a release per version.
- Publishing from Actions without a stored token:
  Pages deployment needs "a minimum of `pages: write` and `id-token: write` permissions"
  ([custom workflows][gh-pages-workflows]),
  the OIDC route;
  creating a release needs `contents: write` with the automatic `GITHUB_TOKEN`,
  as `create-release` in
  `.github/workflows/cargo-publish.yml` already does.
  `cargo publish` itself cannot target a static registry (no `api`),
  so a workflow would run `cargo package`,
  attach the
  `.crate` to the release,
  and regenerate the index file.
  Pages replaces the whole site on each deploy,
  so the index would be rebuilt from all releases each time (design point;
  not run).
  The repository requires signed commits on `main`;
  whether a separate index branch is exempt was not checked.

### Uses (a) to (d)

- (a) Between crates of this repository:
  only after owner-level changes.
  The registry key must reach the manifest:
  `{ path = ..., version = ..., registry = "monochromatic-crates" }`.
  Without the `registry` key,
  publishing to a non-crates.io registry would record the dependency as coming from crates.io:
  Cargo sends the dependency's registry only when it differs,
  and a key-less dependency means crates.io
  ([`cargo_publish.rs` L541 to L551][cargo-transmit-registry]:
  "In the index and Web API,
  None means 'from the same
  registry' whereas in Cargo.toml,
  it means 'from crates.io'").
  The catalog refuses `registry` (`site_classify.rs` line 196),
  and `cargo-shadow` refuses its lock entries
  (`lockfile.rs` line 169;
  reproduced in `doc/planning/self-hosted-cargo-registry-protocol.md`,
  lines 119 to 149).
  Both are the same blockers the self-hosted registry track already lists;
  the choice of host does not change them.
- (b) Installing a prebuilt tool:
  workable in principle,
  new code in `mpm-shadow`.
  `cargo-binstall --index` worked on the fixture and `mpm-shadow` already runs `cargo-binstall` for repository crates,
  but
  `src/toolchain/install_cargo.rs` lines 68 to 110 pass no `--index` or `--registry`,
  and `resolve_crate`
  (`update_registry.rs` lines 98 to 107) reads `https://crates.io/api/v1/crates/<name>` only.
  A `cargo:` row would also still record no SHA-256.
  Cheaper:
  the `archive` route (option 1) needs none of this.
- (c) A consumer outside:
  workable,
  and the best fit of the three options.
  The consumer adds a `[registries]` entry and a `registry` key;
  semantic versions,
  checksums and yanks work as on crates.io.
  `auth-required: false` gives anonymous reads.
- (d) Coexisting with crates.io:
  workable for crates in either set,
  with one hard rule.
  A crate published to crates.io cannot depend on a crate that only the static registry holds:
  the Cargo book says "crates.io does not allow packages to be published with dependencies on code published outside of
  crates.io itself" ([specifying dependencies][cargo-nodep]);
  crates.io's server code returns "Dependency `{}` is hosted on another
  registry.
  Cross-registry dependencies are not permitted on crates.io."
  for any dependency with a `registry`
  ([`publish.rs` L1002 to L1008][cratesio-cross-registry])
  and "no known crate named `{}`" for a dependency name it does not
  hold ([`publish.rs` L1058 to L1065][cratesio-no-known]).

### The `cargo publish --dry-run` trap

- The client check for other-registry dependencies lives in `verify_dependencies`
  ([`cargo_publish.rs` L487 to L521][cargo-verify-deps])
  and is skipped for any dependency that still has a `path` and a
  `version`,
  because `check_dep_has_version` returns early for them (L84 to L110).
- Tests:
  a tool with `{ path, version = "=0.1.0", registry = "mono" }` passed `cargo package` and
  `cargo publish --dry-run` to crates.io ("Uploading ... aborting upload due to dry run"),
  and the config-`[patch]` crate passed too.
  Crates.io would refuse both,
  by the crates.io server code cited in use (d) of this option.
- So a green dry run is not evidence that crates.io accepts a crate
that has a `registry` key or a patched-out dependency.
  Real acceptance was not tested and must not be (no uploads).

### Cost here

- A release or tag per library version,
  an index generator (margo or a small script owned by the repository) and one workflow;
  the index and `dl` address are public,
  no token.
- Catalog and `cargo-shadow` changes only if (a) is wanted through the registry.
- It overlaps the pnpr proposal:
  both serve exactly the same Cargo protocol.

### Unverified

- Real GitHub release redirect behaviour with Cargo,
  and `raw.githubusercontent.com` or Pages caching and `ETag` handling for sparse index
  updates.
- Whether margo's index is accepted by the current Cargo
for dependencies on a second internal registry crate (the fixture used
  `registry: null`,
  meaning the same registry).
- Margo's own code was not run.

## The route that fits the owner's intent best

The sources and tests show one combination that meets both halves of the intent (no crates.io for day-to-day work,
crates stay
reusable) with parts that already exist.
It is a proposal for the owner,
not a decision (rule DRR).

- Tools (`monochromatic-git-policy-cli`,
  `monochromatic-cargo-shadow`,
  `monochromatic-pnpm-shadow`,
  `monochromatic-search-shadow`,
  `monochromatic-file-enforcer-cli`,
  `monochromatic-nested-wayland-session`,
  and `monochromatic-lint`):
  `publish = false`,
  `path`-only internal dependencies,
  and the `archive` route in `toolchain.plan.jsonc` with the release
  archive and its SHA-256 in `toolchain.lock.json`,
  as `mpm-shadow` already installs.
  A fixture tool built and ran this way (option 1,
  "Cargo's equivalents,
  tested").
  No crates.io name,
  no first publish by hand,
  no new-crate rate limit,
  no version per merge.
  `cargo-publish.yml` keeps `build-binary` and `create-release` and loses the `publish-crate` step of these tools.
- Libraries (the reusable crates the owner wants kept reusable):
  they are the only crates that still need a registry,
  and they go there less often.
  Where they go is the owner's choice between crates.io (a closure rule applies),
  the pnpr registry the owner already chose
  to propose,
  a static index on GitHub (option 3),
  or git for outside consumers (option 2,
  uses (c) only).
  None of these changes how tools install.
- Order of work that the tests impose:
  the owner's "One release each" is a `cargo publish`,
  so it must happen before a crate is
  switched to `publish = false`;
  the lock rows must move to the `archive` route before the crates.io copies stop being the
  place the version is read from (`resolve_crate` reads `https://crates.io/api/v1/crates/<name>`,
  so a `cargo:` row would
  silently stay at the last published version).
- Some tools are also libraries of other tools:
  `package/git-policy/cli/Cargo.toml` links `monochromatic-lint` and `forbidden-strings` as path dependencies.
  A crate can be set to `publish = false` only when every crate that stays published has stopped depending on it
  (the closure rule of option 1,
  use (d));
  the owner named `forbidden-strings` and `forbidden-regex` as exceptions in
  `doc/planning/crates-io-footprint.md` (lines 76 to 77).
- What this does not give:
  a `workspace:`-style rewrite that lets a published crate depend on an unpublished one.
  No Cargo mechanism does,
  and the registry-side rules (use (d) of option 3) forbid it.
- mise's `github:` backend (digest and attestation checks,
  `version_prefix`) was weighed for one tool in
  `doc/decision/nested-wayland-session-releases.md`,
  section "Follow-up:
  the `mise` backend" (lines 179 to 213);
  it is the mise counterpart of the `archive` route and is not re-derived here.

## How the options relate to the self-hosted registry track

The track (`doc/planning/self-hosted-cargo-registry.md`,
`...-protocol.md`) proposes hosting the crates on the existing pnpr deployment.

- Option 1 complements it.
  It decides how tools install (archive route) and whether tools need a registry at all;
  the registry then holds only libraries.
  It needs no deployed endpoint,
  so it is not blocked by the track's open authentication work.
- Option 2 is neither a substitute nor blocked.
  It serves only outside consumers and only as commits.
  It stays available with or without the registry.
  Its catalog refusal is a separate amendment from the registry-key refusal the track lists.
- Option 3,
  as GitHub Packages,
  is impossible.
  Option 3,
  as a static index on GitHub,
  is an alternative host for the same registry protocol the track already chose to
  serve through pnpr.
  The track's open items map onto it:
  its Cargo workload-authentication extension is not needed (the index is built in a workflow),
  its
  archive-retention question is answered by immutable releases,
  and the catalog and `cargo-shadow` blockers are identical.
  Choosing between pnpr and the static index is a decision for the owner,
  who has already recorded a choice for pnpr;
  nothing here reopens it.
- Neither option changes the rule that applies under both hosts:
  the crates.io-published set must be closed under dependencies.

## Ranking by workability

Scored on uses (a),
(b),
(c),
(d),
with rule CM7 applied to (b).
The owner's order (workspace,
then floating git,
then GitHub package registry) is the tie-break when two are equal.

- Option 1,
  as `path` plus exact `version` for the manifest and the `archive` route for tools:
  ranks first.
  It serves (a) and (b) today or with a small origin,
  and (d) with the closure rule.
  It does not serve (c).
- Option 3 in its workable form (static sparse index with release assets):
  ranks second.
  It serves (c) best,
  (b) through `cargo-binstall --index` with new `mpm-shadow` code,
  and (d);
  (a) waits on the same owner
  changes as the pnpr track.
- Option 2,
  floating git:
  ranks third.
  It serves (c) only,
  plus the version part of (d);
  it is impossible for (a) in practice and for (b) under CM7.
- Option 3 as named,
  GitHub Packages:
  ranks last:
  no Cargo registry exists there,
  so it serves none of the uses.

Adjacent pairs:

- 1 over 3:
  option 1 covers (a),
  (b) and (d) with no new hosted service and no new trust root,
  while option 3 adds a service for what
  option 1 does through files that already exist;
  option 3 covers (c),
  which option 1 does not,
  so they complement.
- 3 over 2:
  the static index gives outside consumers versions,
  checksums,
  yanks and prebuilt installs,
  where git gives commits and
  a whole-history fetch,
  and git cannot serve (b).
  On the owner's order alone,
  floating git would outrank the GitHub package registry;
  the evidence reverses that because the
  workable form of the third option is not GitHub Packages.
  The owner should confirm that the static-index reading
  of "GitHub package registry" is acceptable before this order is used.
- 2 over GitHub Packages as named:
  floating git works for (c) and the version part of (d);
  GitHub Packages works for none.

## Facts, assumptions, limits and unknowns

### Facts (each with its evidence in the option sections)

- `cargo package` needs every `path` plus `version`
or `git` plus `version` dependency to exist in the target registry (fixture).
- `publish = false` crates build with `path`-only dependencies and cannot be packaged or published (fixture).
- Multi-package publish needs a workspace (fixture,
  Cargo changelog 1.90).
- A git dependency finds a crate by name in a repository with no root manifest;
  the lock pins a commit;
  `cargo update` moves it (fixture).
- `cargo-binstall --git` fails on this repository's layout (fixture);
  `--manifest-path` works without crates.io (fixture).
- `cargo-binstall --index` installs a prebuilt program from a static sparse registry (fixture).
- GitHub Packages lists no Cargo registry (docs);
  no OCI-backed Cargo registry was found (the `gh search` queries in the GHCR subsection).
- The catalog accepts only `rev` for git and no `registry`;
  `cargo-shadow` skips git and refuses non-crates.io registries (source).
- The newest tool release is immutable and its assets carry SHA-256 digests (GitHub API).

### Assumptions

- That "GitHub package registry" can be read as "something on GitHub that serves a Cargo registry".
- That `crate-release.yml` keeps committing the bump before the release exists.

### Limits

- The experiments used fixtures with the same layout as this repository,
  not the repository's crates.
- Nothing was published or uploaded,
  so crates.io's acceptance rules are source readings.
- Third-party source was read at the commits listed;
  behaviour of later commits is not claimed.

### Unknowns

- Whether the owner wants a tool's lock row to follow the checkout's version or the newest release.
- Real GitHub redirect and caching behaviour for a static index (option 3,
  "Unverified").
- Whether signed-commit requirements on `main` extend to an index branch.
- Whether `simple-git` can clone a named revision.
- Which crates form the crates.io-published set the owner intends,
  and so which closure rule applies.

## Link confirmation

Every reference link at the end of this file was opened in `agent-browser` (0.41.0,
one session,
closed afterwards) and the
quoted text was searched for in the rendered page body:
all of them were found.
Two limits on that check.
It tests that the text is on the page,
not that Chrome highlights the `#:~:text=` fragment.
For the commit-pinned source links the line ranges were checked against local clones at those commits instead
(a page-wide text match does not prove the range).
The driver was a scratch script that was deleted after the run.

[pnpm-ws-resolve]: https://pnpm.io/workspaces#:~:text=pnpm%20will%20refuse%20to%20resolve%20to%20anything%20other%20than%20a%20local%20workspace%20package
[pnpm-ws-publish]: https://pnpm.io/workspaces#:~:text=we%20dynamically%20replace%20any%20workspace
[cargo-multi-loc]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#:~:text=The%20git%20or%20path%20dependency%20will%20be%20used%20locally
[cargo-local-paths]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#:~:text=Crates%20that%20use%20dependencies%20specified%20with%20only%20a%20path%20are%20not%20permitted%20on%20crates.io
[cargo-git-traverse]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#:~:text=traverses%20the%20file%20tree%20to%20find
[cargo-git-lock]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#:~:text=checks%20for%20updates%20only%20when%20you%20run
[cargo-git-version]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#:~:text=does%20not%20affect%20which%20commit%20is%20used
[cargo-check-dep]: https://github.com/rust-lang/cargo/blob/d6b1b43ed13a7e12072b5f1685056719546e127a/src/ops/mod.rs#L84-L110
[cargo-patch-root]: https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html#:~:text=Cargo%20only%20looks%20at%20the%20patch%20settings
[cargo-package-strips]: https://doc.rust-lang.org/cargo/commands/cargo-package.html#:~:text=sections%20are%20removed%20from%20the%20manifest
[cargo-1-90]: https://doc.rust-lang.org/cargo/CHANGELOG.html#:~:text=Stabilize%20multi-package%20publishing
[cargo-faq-git]: https://doc.rust-lang.org/cargo/faq.html#:~:text=We%20plan%20to%20support%20git%20repositories%20as%20a%20source%20of%20packages%20forever
[cargo-unstable-git]: https://doc.rust-lang.org/cargo/reference/unstable.html#:~:text=perform%20a%20shallow%20clone%20of%20git%20dependencies
[binstall-git-arg]: https://github.com/cargo-bins/cargo-binstall/blob/90e4ee1101c5c9818ad29c4904d3cec4122bac0e/crates/bin/src/args.rs#L122-L135
[binstall-git-resolve]: https://github.com/cargo-bins/cargo-binstall/blob/90e4ee1101c5c9818ad29c4904d3cec4122bac0e/crates/binstalk/src/ops/resolve.rs#L441-L497
[binstall-version-override]: https://github.com/cargo-bins/cargo-binstall/blob/90e4ee1101c5c9818ad29c4904d3cec4122bac0e/crates/binstalk/src/ops/resolve.rs#L512-L516
[binstall-ws-walk]: https://github.com/cargo-bins/cargo-binstall/blob/90e4ee1101c5c9818ad29c4904d3cec4122bac0e/crates/cargo-toml-workspace/src/lib.rs#L67-L125
[mise-cargo-git]: https://github.com/jdx/mise/blob/f3b85fce6faf6b0e37168bab2b360a92abe88d1c/docs/dev-tools/backends/cargo.md#L73
[gh-packages-intro]: https://docs.github.com/en/packages/learn-github-packages/introduction-to-github-packages#:~:text=such%20as%20npm%2C%20RubyGems%2C%20Apache%20Maven%2C%20Gradle%2C%20Docker%2C%20and%20NuGet
[gh-ghcr]: https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry#:~:text=You%20can%20also%20access%20public%20container%20images%20anonymously
[gh-packages-working]: https://docs.github.com/en/packages/working-with-a-github-packages-registry
[gh-discussion-12067]: https://github.com/orgs/community/discussions/12067
[gh-discussion-11926]: https://github.com/orgs/community/discussions/11926
[cargo-protocols]: https://doc.rust-lang.org/cargo/reference/registries.html#:~:text=Cargo%20supports%20two%20remote%20registry%20protocols
[cargo-index-config]: https://doc.rust-lang.org/cargo/reference/registry-index.html#:~:text=Cargo%20supports%20https%2C%20http%2C%20and%20file%20URLs%2C%20HTTP%20redirects
[margo-readme]: https://github.com/integer32llc/margo/blob/c352ce63ffd96c0c86d0f251fbf60751131dae6b/README.md#L3-L4
[margo-actions-readme]: https://github.com/integer32llc/margo-actions/blob/f65dceeae7a46c2f805d5b2e090b2a8f8444255c/README.md#L35-L53
[simple-gh-readme]: https://github.com/ShoyuVanilla/simple-gh-cargo-registry/blob/6c8ad3fb55e30db0ba639b55ce87a702254896b4/README.md#L29
[gh-pages-limits]: https://docs.github.com/en/pages/getting-started-with-github-pages/github-pages-limits#:~:text=Published%20GitHub%20Pages%20sites%20may%20be%20no%20larger%20than%201%20GB
[gh-releases-quota]: https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases#:~:text=Each%20file%20included%20in%20a%20release%20must%20be%20under%202%20GiB
[gh-immutable]: https://docs.github.com/en/code-security/supply-chain-security/understanding-your-software-supply-chain/immutable-releases#:~:text=Release%20assets%20cannot%20be%20modified%20or%20deleted
[gh-pages-workflows]: https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages#:~:text=The%20job%20must%20have%20a%20minimum%20of
[cargo-transmit-registry]: https://github.com/rust-lang/cargo/blob/d6b1b43ed13a7e12072b5f1685056719546e127a/src/ops/registry/cargo_publish.rs#L541-L551
[cargo-nodep]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#:~:text=crates.io%20does%20not%20allow%20packages%20to%20be%20published%20with%20dependencies%20on%20code%20published%20outside%20of%20crates.io
[cratesio-cross-registry]: https://github.com/rust-lang/crates.io/blob/e0edaf98831e94b4a6d0969b957bda0094a45f3d/src/controllers/krate/publish.rs#L1002-L1008
[cratesio-no-known]: https://github.com/rust-lang/crates.io/blob/e0edaf98831e94b4a6d0969b957bda0094a45f3d/src/controllers/krate/publish.rs#L1058-L1065
[cargo-verify-deps]: https://github.com/rust-lang/cargo/blob/d6b1b43ed13a7e12072b5f1685056719546e127a/src/ops/registry/cargo_publish.rs#L487-L521
