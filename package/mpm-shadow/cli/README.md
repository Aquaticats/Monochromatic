# mpm-shadow

Owns this repository's toolchain plan and lock,
and shadows `mpm` ([Meta Package Manager](https://mpm.run)).

- As `mpm-shadow` it runs the toolchain commands,
  named after pnpm's
  (see "Command names follow pnpm"):
  `install` (alias `i`),
  `update`,
  `outdated`,
  `list` (alias `ls`),
  `bin`,
  `prune`,
  and `verify-archive`.
- As `mpm` (a link the toolchain plan creates) it finds the real `mpm` on `PATH`,
  runs it with the arguments untouched and the standard streams inherited,
  waits for it,
  deletes the superseded cached versions of this repository's own crates from cargo's registry,
  and exits with mpm's own status.

## Design rule: pnpm answers first

Where pnpm's behaviour answers a toolchain question
(the plan,
the lock,
`update`,
`install`,
`install`,
or what happens in CI and when frozen),
mpm-shadow follows it,
and the change is recorded,
even when it overrides an earlier decision line.
Only what pnpm leaves open goes to the owner.
The user's words:
"Please go with pnpm style instead of blocking questions that can be resolved by it on me."
The decisions themselves are in `doc/decision/file-enforcer-rust-rewrite.md`,
section "Toolchain (mpm-shadow)".

## Command names follow pnpm

The commands are pnpm's,
with the meaning pnpm gives them,
checked against the pages on <https://pnpm.io> on 2026-10-31
and against pnpm 12.16.1 in disposable directories.
The old names (`sync`,
`lock`,
`lock --check`,
`update --check`,
`status`,
`bin-paths`,
`--frozen-lock`
and `--no-frozen-lock`)
no longer exist and have no aliases,
because pnpm has none.

- `install`:
  <https://pnpm.io/cli/install>,
  "Aliases:
  `i`",
  `--frozen-lockfile`,
  `--no-frozen-lockfile`,
  `--lockfile-only` ("only updates `pnpm-lock.yaml`...
  Nothing gets written to the `node_modules` directory")
  and `--offline`.
  It replaces `sync`.
- `install --lockfile-only --offline`:
  replaces `lock`.
  pnpm's `--offline` resolves from its metadata cache;
  mpm-shadow has no such cache,
  so an offline run rewrites only the declaration fields of rows the lock already holds
  and refuses an entry that needs resolving.
- `install --lockfile-only --frozen-lockfile`:
  replaces `lock --check`.
  The pnpm page says nothing about this combination;
  measured with pnpm 12.16.1,
  it exits 0 when the lockfile matches the manifest and 1 (`ERR_PNPM_OUTDATED_LOCKFILE`) when it does not,
  and writes nothing,
  which is the check this command did.
- `outdated`:
  <https://pnpm.io/cli/outdated>,
  "exits with code 1 when any dependency is outdated",
  and takes patterns the way `update` does.
  It replaces `update --check` and exits 1 when any selected tool would move or a lookup fails.
  pnpm's `--json`,
  `--format`
  and `--long` are not offered:
  the report is the same text `update` prints.
- `list`:
  <https://pnpm.io/cli/list>,
  "Aliases:
  `ls`".
  It replaces `status`.
  The pnpm page leaves exit codes open,
  so `list` keeps exiting 1 on drift.
  pnpm's `--json`,
  `--parseable`
  and `--depth` are not offered.
- `bin`:
  <https://pnpm.io/cli/bin>,
  "Prints the directory into which the executables of dependencies are linked",
  with `--global`.
  Plain `bin` prints the exposure directory.
  The page has no `--json`;
  `bin --json` prints the document the IDE launcher reads (`exposureDir`,
  `pathDirs`
  and `bins`),
  following the `--json` that pnpm's `list` and `outdated` pages document.
  It replaces `bin-paths`.
- `--global` also has pnpm's short form `-g`.
- Unchanged:
  `update` (<https://pnpm.io/cli/update>),
  `prune`,
  which keeps its name,
  and `verify-archive`,
  which has no pnpm counterpart.

## Toolchain plan and lock

People edit `toolchain.plan.jsonc` (JSONC:
comments and trailing commas are allowed).
mpm-shadow is the only writer and the only checker of `toolchain.lock.json`,
which stays strict JSON because `jq` readers in CI and Dockerfiles select
`.tools.<id>.version`,
`.tools.<id>.artifacts.<platform>.{url,sha256}`
and `.tools.<id>.options.androidPackages`.

The machine-global toolchain has the same two files in `${XDG_CONFIG_HOME:-$HOME/.config}/monochromatic-dev/toolchain`;
every command takes `--global` to act on them.
The repository scope is found from the working directory by the nearest `toolchain.plan.jsonc`,
or from `MONOCHROMATIC_WORKTREE_ROOT`.

### The plan

Every entry has a stable id (an alias,
like a `package.json` key)
and a `request`,
like a `package.json` value:
`<origin>@<range>`.
The origin says where versions come from,
the range says which are wanted
(`latest`,
an exact version or tag,
a channel such as `nightly`,
or a Java major line such as `21`).

```jsonc
// toolchain.plan.jsonc
{
  "schemaVersion": 1,
  "tools": {
    // Why this pin exists belongs here, beside it.
    "bin": { "request": "github:marcosnils/bin@0.29.4", "route": "archive", "bins": ["bin"] },
    "age": { "request": "github:FiloSottile/age@latest", "route": "archive", "bins": ["age", "age-keygen"] },
    "zig": { "request": "zig@0.15.2", "route": "archive", "bins": ["zig"] },
    "cargo:fd-find": { "request": "cargo:fd-find@latest", "route": "cargo-binstall", "bins": ["fd"] },
    // `{version}` stands for the resolved version in `packageId` and in `env` values.
    "rust": {
      "request": "rust@nightly", "route": "rustup-toolchain", "packageId": "{version}",
      "bins": ["rustc"], "env": { "RUSTUP_TOOLCHAIN": "{version}" }
    }
  }
}
```

- A tool is `request`,
  `route`,
  optional `manager`,
  `packageId`
  and `os`,
  `bins`,
  `env`
  and `options`.
- The origins are
  `github:<owner>/<repo>` (a generic release resolver),
  `npm:`,
  `cargo:`,
  `pipx:`,
  `conda:`,
  `go:<module>`,
  `git:<url>` (the range is the revision),
  and the named resolvers
  `node`,
  `go`,
  `zig`,
  `rustup`,
  `aws`,
  `rust`,
  `android-sdk`
  and `java:temurin`.
- The `@` that separates the range is the last one not preceded by `:` or `/`,
  so `npm:@actions/languageserver@0.3.61` reads as expected.
- The routes are
  `bin-sourced`,
  `archive`,
  `cargo-binstall`,
  `cargo-git`,
  `mpm-pnpm`,
  `mpm-uvx`,
  `go-install`,
  `rustup-toolchain`,
  `rustup-components`
  and `android-sdk`.
  The route is how a tool installs;
  the request is where its versions come from.
- An unknown field or origin is an error naming its path.
  `version` and `artifacts` are not plan fields:
  only `update` writes them.
- Tools install in the order written,
  except that `bin`,
  `uv`,
  `mpm`,
  `rustup`,
  `rust`,
  `cargo-binstall`,
  `go`,
  `node`
  and `pnpm` come first.

#### The `github:` resolver

- It takes the newest release (or the release of the range as a tag,
  with and without a leading `v`),
  and for each of the four Linux platforms
  (`linux-x64`,
  `linux-x64-musl`,
  `linux-arm64`,
  `linux-arm64-musl`)
  the asset whose name says Linux and the platform's CPU
  (`x86_64`,
  `amd64`,
  `x64`,
  `aarch64`,
  `arm64`).
- It skips checksums,
  signatures,
  installer packages,
  scripts,
  other operating systems and CPUs,
  baseline,
  debug,
  source,
  `.full.`
  and GPU (`vulkan`,
  `rocm`,
  `cuda`)
  builds;
  an `options.asset` glob that matches a GPU build's name lets that build through,
  and is the only way to ask for one.
- It prefers the platform's libc (a `musl` name for a musl platform,
  otherwise one without),
  then `.tar.gz`,
  `.tar.xz`,
  `.zip`,
  `.gz`,
  and a bare file.
- Two equally good assets are an error that names both.
  `options.asset` (a glob with `*`) settles it,
  for example `"asset": "harper-cli-*"` for a release that ships two programs.
- A platform with no matching asset is left out of the lock
  and fails at `install` with a message naming it.
- The version is the tag without its `v` or `name-v` prefix.
- `options.tagPrefix` picks one program's releases in a repository that tags several
  (this repository tags `monochromatic-mpm-shadow-v0.2.0`,
  `monochromatic-lint-v0.4.0`,
  and so on).
  With `"request": "github:Aquaticats/Monochromatic@latest"`
  and `"tagPrefix": "monochromatic-mpm-shadow-v"`,
  the resolver reads the release list
  (100 per page,
  at most 10 pages,
  newest first)
  and takes the first release that is neither a draft nor a prerelease
  and whose tag is the prefix followed by a version
  (a digit,
  or `v` and a digit);
  the version is the rest of the tag.
  A range names a tag:
  the prefix and the range as written,
  then (unless the prefix ends in `v`) the prefix,
  `v` and the range.
  A list with no such release is an error that names the prefix.
  Changing the prefix is a resolution-affecting edit:
  it needs `mpm-shadow update`.
  Characters other than letters,
  digits,
  `.`,
  `_`
  and `-` are refused.

#### Lock rows

- A downloaded tool is recorded for the four Linux platforms only
  (the repository's Linux-only stopgap).
- Zig,
  Go,
  Node,
  rustup,
  conda
  and GitHub publish checksums,
  which the lock records.
  Java uses the release asset's digest.
  The Android command-line tools and the AWS CLI publish none,
  so `update` downloads each once and records the hash of what it got.

### The lock

- The lock is derived from the plan and nothing else.
  Deleting `toolchain.lock.json` and running `update` rebuilds it
  (the same versions for exact requests,
  the newest for `latest`).
  No resolver reads a previous lock row to learn a repository,
  a tag shape,
  an asset
  or a platform list.
- Tool entries keep the key order `request, version, route, manager, packageId, bins, os, env, options, artifacts`.
- A top-level `digest` (`sha256:` and the hash of the compact JSON of everything else) covers the whole content.
  `install --lockfile-only --frozen-lockfile` recomputes it,
  so a hand edit of a resolved version,
  a URL or a hash fails the gate.
  A reformatted file keeps its digest.

### What the cutover converter must write

For every entry the converter writes a `request` string.
Package-manager entries keep their specifier:
`cargo:<crate>@latest`,
`npm:<name>@latest`,
`pipx:<name>@latest`
(`@<version>` where the old request was `pinned`,
for example `cargo:cargo-fuzz@0.13.2`).
`manager` and `packageId` become optional because the request names the package;
`routes`,
`bins`,
`os`
and every `options` field except `cargoGit` (replaced by `git:<url>@<revision>`)
and `assetPattern` (renamed `asset`) carry over unchanged.

**The `{version}` placeholder is required.**
Where a field must equal the resolved version,
the converter must write `{version}`,
never today's literal:

- `rust`:
  `"packageId": "{version}"` and `"env": { "RUSTUP_TOOLCHAIN": "{version}" }`.
- `java`:
  `"env": { "JAVA_HOME": "java/{version}" }`.
- `android-sdk`:
  `"env": { "ANDROID_HOME": "android-sdk/{version}", "ANDROID_SDK_ROOT": "android-sdk/{version}" }`.

A literal such as `nightly-2026-10-30` there would make the first `update` move `version`
while `packageId` keeps the old date,
and `rustup toolchain install` would install the stale channel.

Requests for today's downloaded tools,
each checked by a live `update` in a disposable home against the current lock
(the four Linux platforms resolve to the same files,
except where a musl platform now gets the musl build the release offers):

- `bin`:
  `github:marcosnils/bin@0.29.4` (pinned;
  keep the `bootstrap` option).
- `mpm`:
  `pipx:meta-package-manager@8.2.0`
  (pinned;
  keep `route: mpm-uvx` and the `bootstrap` option with `distribution: wheel`).
- `bun`:
  `github:oven-sh/bun@bun-v1.3.14` (pinned tag;
  keep `binAliases`).
  The baseline platforms (`linux-x64-baseline` and the musl baseline) are no longer recorded.
- `pnpm`:
  `github:pnpm/pnpm@latest` (keep `binAliases`).
- `ripgrep`:
  `github:BurntSushi/ripgrep@latest`.
- `node`:
  `node@latest`.
- `harper-cli`:
  `github:Automattic/harper@latest` with `"asset": "harper-cli-*"`.
- `dprint`:
  `github:dprint/dprint@latest`.
- `uv`:
  `github:astral-sh/uv@latest`.
- `hyperfine`:
  `github:sharkdp/hyperfine@latest`.
- `caddy`:
  `github:caddyserver/caddy@latest`.
- `watchexec`:
  `github:watchexec/watchexec@latest`.
- `sops`:
  `github:getsops/sops@latest`.
- `age`:
  `github:FiloSottile/age@latest`.
- `cargo-binstall`:
  `github:cargo-bins/cargo-binstall@latest`.
- `cmake`:
  `github:Kitware/CMake@latest`.
- `zig`:
  `zig@0.15.2` (pinned).
- `conda:bubblewrap`:
  `conda:bubblewrap@latest` (keep `os: ["linux"]`;
  the package is a `.conda` file,
  which `install` unpacks with `unzip` and `tar --zstd`).
- `taplo`:
  `github:tamasfe/taplo@latest` (keep `binAliases`).
- `tombi`:
  `github:tombi-toml/tombi@latest` with `"asset": "tombi-cli-*"`.
- `marksman`:
  `github:artempyanykh/marksman@latest`.
- `rumdl`:
  `github:rvben/rumdl@latest`.
- `github:Feel-ix-343/markdown-oxide`:
  `github:Feel-ix-343/markdown-oxide@latest`.
- `zizmor`:
  `github:zizmorcore/zizmor@latest`.
- `superhtml`:
  `github:kristoff-it/superhtml@latest`.
- `github:docker/docker-language-server`:
  `github:docker/docker-language-server@latest`.
- `llama.cpp`:
  `github:ggml-org/llama.cpp@b9829` (pinned).
- `opentofu`:
  `github:opentofu/opentofu@latest`.
- `hcloud`:
  `github:hetznercloud/cli@latest`.
- `betterleaks`:
  `github:betterleaks/betterleaks@latest`.
- `java`:
  `java:temurin@21` (a major line that moves;
  the version is `temurin-<release>.0.LTS`).
- `android-sdk`:
  `android-sdk@latest` (keep `androidPackages`).
- `rust`:
  `rust@nightly` (keep `rustComponents` and `rustTargets`).
- The machine-global plan also holds `go: go@latest`,
  `aws: aws@latest`,
  `rustup: rustup@latest`
  and `go:github.com/charmbracelet/crush@latest` on the `go-install` route.

## Commands

- `mpm-shadow install --lockfile-only --offline`:
  rewrites the lock's declaration fields from the plan without the network,
  keeping every resolved version and artifact.
  It refuses a new tool or a resolution-affecting edit
  (`request`,
  `route`,
  `manager`,
  `packageId`,
  `os`,
  `asset`,
  `bootstrap`)
  and names `mpm-shadow update <id>` for just those tools.
- `mpm-shadow install --lockfile-only --frozen-lockfile`:
  the toolchain drift gate for CI,
  beside `file-enforcer check`.
  Writes nothing.
  Exits 1 when the plan and lock disagree or the digest does not match.
  In CI the frozen default applies when a lock exists,
  so `install --lockfile-only` alone is the same check there.
- `mpm-shadow install --lockfile-only`:
  resolves the entries the lock lacks or holds for an older request,
  writes only their rows,
  and installs nothing.
- `mpm-shadow update [<tool>...]`:
  resolves every tool and writes the whole lock,
  or,
  given tool ids,
  only those (see "Updating named tools").
  A brand-new downloaded tool is just a new plan entry:
  every origin has a resolver,
  and none reads the previous lock.
- `mpm-shadow outdated [<tool>...]`:
  reports what `update` would move,
  writes nothing,
  installs nothing,
  and exits 1 when something would move.
- `mpm-shadow install [--frozen-lockfile | --no-frozen-lockfile]`:
  first resolves the plan entries the lock lacks or holds for an older request
  (see "Installing a plan that is ahead of the lock"),
  then installs every locked tool that is not at its locked version,
  makes the postinstall links,
  rebuilds the exposure directory and runs the postinstall commands.
- `mpm-shadow list`:
  compares the installed tools with the lock.
  Exits 1 on drift.
- `mpm-shadow prune --toolchain [--global] [--dry-run] [--lock <file>]...`:
  deletes the cargo tool versions no known lock names (see "Cleanup").
  Plain `mpm-shadow prune` is unchanged:
  it cleans cargo's registry.
- `mpm-shadow verify-archive <tool> <file> [--platform <id>]`:
  hashes a downloaded file and compares it with the lock's SHA-256 for that tool and platform.
- `mpm-shadow bin [--json]`:
  prints the directory the executables are linked into;
  with `--json`,
  `exposureDir`,
  `pathDirs`
  and `bins` as JSON for the IDE launcher.
  It reads only the plan and the file system.

`list` refuses when the plan and lock disagree,
and names `mpm-shadow install --lockfile-only --offline` or `mpm-shadow update <ids>`;
`install` does the same when frozen,
and otherwise updates the lock itself:
it resolves the new entries and writes the plan's other declaration changes.

### Updating named tools

`mpm-shadow update <tool>...` mirrors `pnpm update <pkg>...`:
it looks up and moves only the tools it selects,
and every other lock row keeps its version and artifacts
(and takes the plan's current `bins`,
`env`
and `options`,
as offline `install --lockfile-only` would give it).
Adding a tool to the plan and running `mpm-shadow update <new id>` locks that tool alone,
the way `pnpm install` after adding one dependency resolves only that dependency.

- A name is a tool id,
  or a pattern in which `*` stands for any run of characters
  (`mpm-shadow update 'npm:*'`);
  a leading `!` excludes
  (`mpm-shadow update 'npm:*' '!npm:wrangler'`),
  and only exclusions select everything else.
  This is the pattern grammar of `pnpm update "@babel/*" "!@babel/core"`.
- A name that matches no tool of the plan is refused with exit code 2,
  before any lookup,
  naming it and listing every valid id.
  pnpm 12.16.1 accepts such a name silently and reports "Already up to date";
  that hides a typo here,
  where a name selects what to resolve.
- A plan entry the lock cannot hold as it is
  (no lock row,
  or an edit of `request`,
  `route`,
  `manager`,
  `packageId`,
  `os`,
  `options.asset`
  or `options.bootstrap`)
  that the names left out is refused,
  before any lookup,
  with the ids to add to the command.
  Writing a lock with such a row kept would fail `install --lockfile-only --frozen-lockfile`,
  and resolving it unasked would move what the command line did not name.
  With no lock file every entry is such a row,
  so a named update cannot create the lock;
  plain `update` does.
- `outdated` takes the same names:
  it reports only the selected tools and exits 1 when one of them would move.
- Not mirrored:
  `--latest` (a request's range already says what is wanted),
  `<tool>@<range>` (people edit the plan,
  so a range changes there),
  `--interactive`,
  `--depth`,
  and the workspace flags have no counterpart.
  `update` installs only to retry a failed npm or PyPI lookup,
  so it already behaves as `--lockfile-only` does.
- `install --lockfile-only --offline` stays offline.
  `install` resolves a plan that is ahead of the lock,
  as the next section says.

### Installing a plan that is ahead of the lock

`mpm-shadow install` mirrors `pnpm install` after a dependency was added to `package.json`:
the entries the lock cannot hold as they are
(the same set the named `update` refuses to leave out:
no lock row,
or an edit of `request`,
`route`,
`manager`,
`packageId`,
`os`,
`options.asset`
or `options.bootstrap`)
are resolved,
only their rows are written to the lock,
and then the install runs.
No other row moves,
exactly as with `mpm-shadow update <those ids>`,
and the report prints `toolchain install: locked <id> <version> ...` for each.

- **Frozen:**
  with `--frozen-lockfile`,
  or when a CI environment is detected,
  `install` resolves nothing into an existing lock:
  it refuses as before,
  names `mpm-shadow update <ids>` for exactly those entries,
  and says why it did not resolve.
  pnpm's `--frozen-lockfile` is "true by default in CI environments"
  and "pnpm detects a CI environment with the same rules as the ci-info package.
  Set `CI=false` to turn detection off"
  (<https://pnpm.io/cli/install>).
  mpm-shadow ports those rules (`src/toolchain/ci_detect.rs`):
  `CI=false` means not CI,
  otherwise `CI`,
  `BUILD_ID`,
  `BUILD_NUMBER`,
  `CI_APP_ID`,
  `CI_BUILD_ID`,
  `CI_BUILD_NUMBER`,
  `CI_NAME`,
  `CONTINUOUS_INTEGRATION`,
  `RUN_ID`,
  or a CI vendor's own variable (for example `GITHUB_ACTIONS`) means CI.
- **`--no-frozen-lockfile`:**
  resolves even in CI,
  like pnpm's `--no-frozen-lockfile`.
  The two switches cannot be combined.
- **A lock that does not exist** is resolved in full and written by `install`,
  frozen by CI or not,
  and then the install runs.
  pnpm's `--frozen-lockfile` defaults to "true,
  if a lockfile is present" in CI,
  so `CI=true pnpm install` with no `pnpm-lock.yaml` resolves and writes one
  (checked with pnpm 12.16.1 in a disposable directory).
  `--frozen-lockfile` given explicitly refuses instead,
  as `pnpm install --frozen-lockfile` does with no lockfile
  (it fails with `ERR_PNPM_NO_LOCKFILE`),
  and names `mpm-shadow update`.
- **A failed lookup** stops `install` before anything is written or installed,
  naming the entry and the URL.
- **Offline-fixable edits:**
  a plan that differs from the lock only in what `install --lockfile-only --offline` fixes
  (`bins`,
  `env`,
  the install options,
  a removed entry,
  the order)
  is written into the lock by a non-frozen `install`,
  without a lookup and without moving any version,
  and the report prints `toolchain install: lock updated from the plan: ...` for each change.
  This is pnpm's non-frozen install,
  which updates the lockfile for any manifest change.
  The same write also carries these edits when a new entry is resolved in the same run.
  Frozen (`--frozen-lockfile`,
  or CI with a lock present)
  refuses instead,
  writes nothing,
  and names `mpm-shadow install --lockfile-only --offline`.
- **A lock whose digest does not match**
  is refused naming the restore,
  under either policy.
  `list` never resolves and never writes.

## How it installs

- `rustup toolchain install --no-self-update <channel>`,
  then `rustup component add` and `rustup target add`,
  straight from the lock.
- `cargo binstall --no-confirm --root <toolchain root>/cargo/<crate>/<locked> --version <locked> <crate>`
  (see "Cargo tool installs").
- `pnpm add --global <package>@<locked>`
  and `uv tool install --force <package>==<locked>`,
  or a wheel whose SHA-256 the lock records.
- `archive` tools are downloaded,
  checked against the lock's SHA-256,
  and only then unpacked with `tar` or `unzip`.
  A file that does not match is deleted and nothing is installed from it.
- `bin-sourced` tools go through the `bin` CLI.
- mpm is never asked to install or look up a version:
  mpm 8.2.0 ignores a pinned version for cargo and pnpm,
  and reports registry failures only in an `errors` array while exiting 0.

## Cargo tool installs

Each `cargo-binstall` tool installs into a cargo root of its own,
`<toolchain root>/cargo/<crate>/<version>`:

```text
# shape of the command install starts (the paths are examples)
cargo binstall --no-confirm --root <toolchain root>/cargo/fd-find/10.4.2 --version 10.4.2 fd-find
```

- Binaries land in `<root>/bin` and cargo's own record in `<root>/.crates.toml`.
  Nothing is written to `$CARGO_HOME/bin` or `$CARGO_HOME/.crates.toml`;
  only cargo's registry cache and `binstall.toml` still live below `$CARGO_HOME`.
- `--root` rather than `--install-path`:
  cargo-binstall 1.26.1 (the locked version) documents `--install-path` as having no effect on a package built from source,
  and its source (`crates/binstalk/src/ops/resolve/resolution.rs`) fails with `CargoInstallDoesNotSupportInstallPath`
  when the build-from-source fallback runs with it.
  `--root` is passed on to the fallback's `cargo install --root`,
  so one flag isolates both ways of installing.
- Tracking stays on (no `--no-track`):
  the root's `.crates.toml` is the record `list` and `install` read,
  and cargo-binstall's own lock then covers two trees installing one version at once.
- A directory without a record for the crate is an unfinished install and counts as absent.
  The legacy `$CARGO_HOME/.crates.toml` and `cargo binstall --list` are never read for this route.
- The `cargo-git` route already installed with `cargo install --git ... --root <tool install dir>` and is unchanged.
- Third-party crates coexist per version:
  each tree's `.cache/toolchain-bin` links the version its own lock names.
- A tool whose crate name starts with `monochromatic-` (the prefix the registry prune already uses for this
  repository's own crates) is repository-published.
  The plan and lock carry no second list of them.
  See the next section.

### Repository-published tools keep only the newest version

Older repository-published tools are never kept once a newer one is installed.

- `install` never installs a version older than the newest finished one in the toolchain root,
  and after installing a version it deletes every older one.
- `<toolchain root>/cargo/<crate>/current` is a link to the newest finished version.
  Every tree's exposed link goes through it,
  so deleting an older version never leaves a dangling link,
  and two trees syncing at once converge on the newest.
- `list` counts an installed version that is newer than the locked one as satisfied,
  not drift.
- A tree whose lock pins an older version than the one that runs is warned,
  in the same words everywhere:

  ```text
  <tool id> is locked at <locked> in this tree but <used> runs, because older repository tools are never kept once a newer one is installed; upgrade this tree's lock with `mise run toolchain:update` or merge main
  ```

  `install` and `list` print it as `toolchain <verb>: warning: ...` in their report on standard output;
  `install --lockfile-only`,
  `update`,
  `outdated` and `bin` print the same line on standard error
  (`bin` writes JSON on standard output);
  the shim path (`mpm`) resolves no tool versions and prints none.

### The launcher

A bare symlink to the binary has nowhere to check the tree's lock,
so the exposed link of a repository-published tool (`.cache/toolchain-bin/<bin>`)
and its postinstall links (`.cache/cli-git-bin/git`) point at a launcher in `.cache/toolchain-launchers/<bin>`.

- The launcher is a hard link of the `mpm-shadow` that ran `install` (a copy across file systems),
  with a `<bin>.launch` record beside it naming the tool id,
  the tree's lock file and the real binary.
  The kernel resolves a symlink chain to its final file,
  so a hard link per tree is what lets the running process learn which tree's link started it
  (`/proc/self/exe`).
- Started under that path,
  `mpm-shadow` reads the record and the lock,
  prints the warning above on standard error when the version behind `current` is newer than the lock pins,
  then replaces itself with the real binary (`exec`):
  argv,
  environment,
  standard streams,
  exit code and signals are the real program's own,
  and standard output is never written.
  An unreadable lock or record never blocks a run.
- Measured 2026-10-31 with `hyperfine` 2.0.0,
  400 runs per arm,
  `cli-git-native --version`
  (the git wrapper handing `--version` to git),
  order bare,
  launcher,
  copy of bare,
  launcher with warning,
  bare,
  launcher:
  median wall time 1.47 ms bare against 2.04 ms to 2.09 ms through the launcher,
  so the launcher adds about 0.6 ms (about 0.5 million instructions) to a call that takes 1.5 ms.
  Bare against a bare copy and a repeat of bare differed by at most 0.04 ms.

### Cleanup

`mpm-shadow prune --toolchain [--global] [--dry-run] [--lock <file>]...` deletes what no known lock names.
It runs only when asked;
`install` never deletes another tree's version.

- Known locks are this scope's lock,
  the `toolchain.lock.json` of every worktree `git worktree list --porcelain` reports,
  and each file given with `--lock` (a clone that git does not list).
- A third-party version is kept when any known lock names it;
  a repository-published crate keeps only its newest finished version,
  and nothing when no known lock names the crate.
- It refuses,
  deleting nothing,
  when git cannot list the worktrees,
  when a listed worktree's directory is missing and not marked prunable,
  or when a known lock exists but cannot be read.
  A lock in the TypeScript tool's format is read for `tools.<id>.version`,
  `route` and `packageId` only.
- A `--lock` is the only way to protect a tree that git does not list.

### Migration from the shared `$CARGO_HOME/bin`

The first `install` with this layout finds no version directory,
installs every `cargo-binstall` tool into its own,
links the tree's exposure and postinstall links to it,
and leaves `$CARGO_HOME/bin` and `$CARGO_HOME/.crates.toml` exactly as they were:
trees still on the TypeScript toolchain keep linking the files there.
`list` reports such a tool as absent until that first `install`.
Nothing copies,
moves or deletes the legacy files;
removing them is the cutover's decision.

## Network

Every npm and PyPI version,
first or later,
is looked up directly at `registry.npmjs.org` and `pypi.org` over HTTPS.
crates.io,
`static.rust-lang.org`,
`nodejs.org`,
`go.dev`,
`proxy.golang.org`,
`ziglang.org`,
`api.adoptium.net`,
`dl.google.com`,
`api.anaconda.org`
and the GitHub API are asked the same way.
The client is `hyper` with `hyper-rustls` (native root certificates),
HTTPS only,
with bounded timeouts and sizes,
and every failure names the URL.
`curl` is never started.

When an npm or PyPI lookup fails for a tool that is in the lock but not installed,
plain `update` installs just the missing tools (and `node`,
`pnpm`
or `uv` when they need them),
then looks each one up again once.
A second failure is final and says the lookup was made twice.
`outdated` never installs.

## Concurrency

`install --lockfile-only` and `update` read the plan and lock,
remember their content hashes,
take an exclusive OS lock for the scope,
read the hashes again,
and publish the new lock (a temporary file renamed over the old one) only if neither file changed.
A changed file refuses the publish and names the file.

## Bootstrap

A fresh machine needs one prebuilt `mpm-shadow` archive whose SHA-256 is in the lock,
or `cargo install monochromatic-mpm-shadow` after a verified `rustup` install.
`mpm-shadow verify-archive` checks a downloaded archive against the lock before it is run.
Containers and CI can use the same `jq`,
`curl`
and `sha256sum` pattern they use for Node and pnpm.

## mpm shadow mode

mpm 8.2.0 has no cleanup for its `cargo` manager and removes no previous version on install or upgrade
(`doc/decision/mpm-toolchain-adoption.md`,
section on pruning the repository's own crates).
Plain `cargo install` fills `registry/cache/<index>/<crate>-<version>.crate`
and `registry/src/<index>/<crate>-<version>/` with every version it ever fetched.

- Runs the real `mpm` as a child,
  not by replacing itself,
  because the prune has to run after mpm finishes.
- Passes arguments through byte for byte,
  including ones that are not valid UTF-8,
  and adds nothing to standard output.
- Exits with mpm's exit code,
  or `128` plus the signal number when mpm died from a signal.
- Prunes after every run that started,
  whatever the exit code and whatever the verb.
  A run that could not start prunes nothing.
- Touches only crates whose name starts with `monochromatic-`.
  For each,
  it keeps the newest version an install record names as installed,
  or the newest cached version when no record names the crate,
  and deletes every other cached archive and extracted source.
  The records are the version-keyed ones `install` writes under `<toolchain root>/cargo/<crate>/<version>/.crates.toml`,
  read in both the repository root and the machine-global root
  (`<data home>/monochromatic-dev/toolchain` and `toolchain-global`,
  `XDG_DATA_HOME` or `~/.local/share`),
  and cargo's own `<cargo home>/.crates.toml`,
  which trees still on the TypeScript toolchain keep writing;
  the newest across them wins,
  so a stale legacy file cannot keep an old version or delete a newer cached archive.
  A version directory without a record is an unfinished install and names nothing.
- Reports each deletion on standard error as `mpm-shadow: deleted stale <path>`.
- Skips every candidate that resolves to its own executable when it looks for the real `mpm`.
- Shows that it is a shadow in the two places a person looks,
  after mpm's own output and only after a successful run:
  a top-level `mpm --version` prints mpm's version text,
  then one line on standard output,
  `mpm-shadow <version>, shadowing mpm at <path of the real mpm>`;
  a top-level `mpm --help` (or `-h`) prints mpm's help,
  then a blank line and a short section naming the shadow,
  its version,
  the real mpm's path,
  and the prune it runs after every run.
  This is how `hub` does it:
  `hub --version` runs `git version` and then prints `hub version <X>`
  (<https://github.com/mislav/hub/blob/master/commands/version.go>),
  and `hub help` appends the commands hub adds to git's help.
- Detects only top-level options,
  the way mpm's parser reads them:
  the words before the subcommand,
  in order,
  skipping the values of the options that take one
  (`--verbosity`,
  `--config`,
  `-t`,
  `-j`
  and the others read from `mpm --help` of mpm 8.2.0;
  a bare `--color` takes none).
  The first eager option wins,
  as in mpm.
  `mpm install --help`,
  `mpm help install`,
  `mpm --help-format json`,
  `--man`,
  `--tree`,
  `--params`,
  and a failing run print exactly what mpm printed,
  with mpm's exit status.
  mpm has no `-V`.
  A new top-level mpm option that takes a value must be added to `VALUE_OPTIONS` in `src/shadow_notice.rs`.

A cached version that a project's `Cargo.lock` still pins is deleted too when it is not the kept one;
cargo downloads it again on the next online build,
but an offline build of that project fails until then.

The artifact is deliberately not named `mpm`,
so a `target/*/mpm` or a tool `bin/mpm` never shadows mpm by accident.
The toolchain plan links it as `mpm` into a gitignored directory that comes first on `PATH`.

## Exit codes

- `0`:
  success.
- `1`:
  the plan and lock disagree,
  the lock's digest does not match,
  an install,
  download
  or lookup failed,
  or `list` found drift.
- `2`:
  a bad command line,
  or a plan or lock file that cannot be read.
- `126`:
  (shadow mode) the real `mpm` was found but could not be started.
- `127`:
  (shadow mode) no real `mpm` on `PATH`.
- Anything else in shadow mode:
  mpm's own exit code.

## Checks

```bash
mise run //package/mpm-shadow/cli:lint
mise run //package/mpm-shadow/cli:lint:clippy
mise run //package/mpm-shadow/cli:test
```
