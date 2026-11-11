# @monochromatic-dev/crate-release-cli

Release helpers for Rust crates that are published to crates.io by a GitHub Actions workflow and
ship tarballs as GitHub release assets.
A release runs on a GitHub runner,
not on a developer machine:
the person releasing dispatches the `crate-release.yml` workflow and watches it.
Nothing compiles locally.
The decision,
with the owner's words,
is `doc/decision/releases-run-in-ci.md`.

The package is private and has no build output:
its executables run as TypeScript,
and tests import the sources
directly.

## Why this location and shape

- `package/crate-release/cli` follows the layout rule `package/<category>/<name>` with singular segments;
  the package name is `@monochromatic-dev/crate-release-cli`.
  The category is the activity (releasing crates),
  not a product,
  because the code serves every crate that publishes
  through `.github/workflows/cargo-publish.yml`.
- The commands are product-neutral:
  anything specific to one crate (its name,
  directory,
  commit scope,
  pins,
  check binaries,
  lock directories) is an option of the release command,
  never a constant in the code.
- The arguments of every releasable crate live in one data file,
  [`releasable-crates.json`](releasable-crates.json),
  because several readers need them and a copy in each would drift:
  the release command on the runner,
  the dispatch command and the await command on the developer machine,
  and the `crate` choices of the workflow,
  which a unit test compares with the file.
  The alternatives were to read the arguments out of each package's `mise.toml` release task,
  and to keep a data file in each package.
  The first parses a shell command line out of TOML,
  and stops working once the local task only dispatches.
  The second needs a scan of the repository to find a crate,
  and the workflow's `choice` list needs one enumeration of the crates anyway.
- The package reuses `shellQuote` and the character-set helpers of `@monochromatic-dev/shell-argument-core`
  through its `./ts/*` subpath.
  That neutral package is shared with `@monochromatic-dev/mutation-test-cli-cargo-mutants`,
  so neither tool depends
  on the other.

## How a release runs

1.  The person runs the crate's `release` mise task with `<from> <to> <note>`.
    It runs the dispatch command,
    which starts `.github/workflows/crate-release.yml` with `gh workflow run` and prints the run URL.
2.  The `release` job of the workflow checks out `main`,
    installs Node,
    pnpm and the Rust toolchain without mise,
    and runs the release command with `--host runner`.
    The job fails at once,
    naming the manifest,
    when the crate does not hold `<from>`.
3.  The job appends the bump to `main` with the push command,
    through the GraphQL API,
    so GitHub signs the commit:
    `main` requires signed commits,
    and a runner has no signing key.
    `main` moves every few minutes and a release takes long,
    so the commit is usually refused once for a moved `main`.
    The push command then compares the new head with the checkout over every path the release depends on,
    and sends the same commit again on the new head when none changed
    (see "Push command").
4.  The `publish` job dispatches `.github/workflows/cargo-publish.yml` on `main` for the crate with its `dry-run` input off.
    A push made with the repository's `GITHUB_TOKEN` starts no workflow
    ([GitHub documents][token-trigger-rule] that events triggered by the `GITHUB_TOKEN` create no workflow run,
    except `workflow_dispatch` and `repository_dispatch`),
    so the dispatch is what starts the publish,
    and the push of the bump runs no other workflow either.
    `cargo-publish.yml` publishes the crate named by a dispatch whatever changed,
    reading the version from the commit the dispatch runs on,
    which is `main` after the bump.
5.  The person runs the crate's `release:wait` task with the run URL and the version.
    It follows the release run to its end,
    finds the publish run that it started
    (the earliest run built from a commit that holds the released version),
    follows that one to its end too,
    and checks crates.io and the GitHub release.
    For a version that never goes to crates.io (see "Labels and crates.io") it leaves crates.io out and says so.

A release therefore runs two workflows,
and both matter to whoever waits for it:

- `.github/workflows/crate-release.yml`,
  the run `release` dispatches.
  Its `release` job bumps the version,
  checks the crate,
  dry-runs the publish and pushes the bump to `main`;
  its `publish` job dispatches the second workflow and does no publishing itself.
- `.github/workflows/cargo-publish.yml`,
  the run the `publish` job dispatches.
  It publishes the crate to crates.io
  and its per-crate jobs build the tarballs and create the GitHub release,
  so the archives a released version ships come from this run,
  never from the release run.
  It also runs a `detect` job for every crate of the repository,
  so most of its jobs of another crate are `skipped`:
  a skipped job is no failure,
  and a run is a pass only when the run itself concluded with success.

A release that fails before the push leaves `main` as it was.
A push that finds a path the release depends on changed on `main` pushes nothing,
names the paths,
and fails the run:
dispatch the release again.
A release that fails after the push,
or a publish that fails,
leaves the bump on `main`.
When the publish run never uploaded the crate,
start it again with
`gh workflow run cargo-publish.yml --ref main -f crate=<crate> -f dry-run=false`.
Once the upload succeeded,
re-run the failed jobs of that publish run with `gh run rerun <run id> --failed` instead:
the `forbidden-strings` block of `cargo-publish.yml` has no skip for a version that is already on crates.io,
so a fresh dispatch for it fails at `cargo publish`,
unlike the blocks of the other crates.
The release command refuses to bump a crate that already holds `<to>`
when it runs with `--host runner`,
so dispatching the release again cannot repeat it.

Add `--dry-run` to the dispatch to run everything except the push and the publish.
A dry run is the way to prove a new runner setup before a real release,
and it is followed with `gh run watch <run URL>`,
because no publish run follows it.

## Labels and crates.io

The owner's rule of 2026-11-07,
verbatim:
"My thinking is to deploy all releases on pnpr,
but only publish rc and stable releases on crates.io",
with the pick "Label decides"
(`doc/planning/crates-io-footprint.md`):

- a version ending `-alpha.N` or `-beta.N` never goes to crates.io;
- a version ending `-rc.N` and a plain number do.

pnpr does not serve crates yet,
so for now an alpha or a beta goes to its GitHub release only.
A GitHub release made by `cargo-publish.yml` is never flagged as a prerelease,
whatever the version,
because the `@latest` resolution of `mpm-shadow` (`github:<owner>/<repo>@latest` with a `tagPrefix`) skips flagged and draft releases,
and the lock of this repository installs its tools that way.

The label is the first dot-separated identifier of the pre-release part,
and only the exact shape `<label>.<N>` with one number counts.
Build metadata after a plus sign is ignored.
Every other form is called unlabelled:
`-alpha` without a number,
`-rc.1.2`,
`-RC.1`,
`-dev.1`,
and a text that is no version.
An unlabelled version does not go to crates.io either,
because a crates.io publish cannot be undone and a form the rule does not name must not reach it.
The decision is made twice and a unit test compares the two over one list of versions
(`src/crates-io-version-label-cases.ts`):

- in TypeScript by `src/crates-io-version-label.ts`
  (`releaseLabel`,
  `goesToCratesIo` and `cratesIoRoute`),
  which the release command,
  the dispatch command and the wait commands use;
- in shell by the composite action `.github/actions/decide-whether-a-version-goes-to-crates-io`,
  which every `detect` job of `cargo-publish.yml` runs on its own crate's `Cargo.toml`
  (a runner has no Node installation step in those jobs,
  and an action keeps the shell in one place).

What a version that does not go to crates.io skips:

- In `cargo-publish.yml`,
  every step that touches crates.io:
  `cargo package` (it verifies against crates.io),
  the provenance attestation of the `.crate`,
  the Trusted Publishing login,
  both `cargo publish` steps,
  the wait for a `forbidden-strings` version in the index (`gpc-publish-crate`),
  and the index check of a push of `forbidden-strings`.
  Each reads `needs.<crate>-detect.outputs.goes_to_crates_io` of its own crate.
  The `publish-crate` job itself still runs,
  because each `create-release` job needs it,
  and a skipped job would skip the GitHub release that an alpha exists for.
  The build of the binaries and the GitHub release (with the assets) run as before,
  and its notes say the version is not on crates.io.
- In the release command,
  the `cargo publish --dry-run` step,
  because it asks crates.io.
- In the wait commands,
  the crates.io lookup:
  the report says `crates.io <version>: skipped, <reason>` and the result depends on the workflow run and the GitHub release.

A release that does go to crates.io is refused,
by the release command on the runner before any file is edited and by the dispatch command before it spends a run,
when a crate of the registry that it depends on is not at such a version.
The dependency's own manifest and the exact pin in the released crate's manifest are both read,
a dependency written as an inline table or as a table of its own counts,
and a dependency without a version does not (a published manifest drops it).
The refusal names each dependency,
its version and the manifests that hold it.
An alpha or a beta release has no such need.

## Starting a release

```bash
# mise task of the crate: dispatches the workflow and prints the run URL
mise run //package/forbidden-strings/cli:release -- 0.6.0 0.6.1 "what changed"

# follows the release run, then the publish run it started, then crates.io and the GitHub release
mise run //package/forbidden-strings/cli:release:wait -- <run URL or id> 0.6.1
```

The git-policy wrapper has the same two tasks under the names `native:release` and `native:release:wait`.
`mise run //package/crate-release/cli:release -- --crate <name> ...` does the same for any crate in the registry.

## The registry

`releasable-crates.json` is an object of crates by their crates.io name,
which is also the `crate` choice of `cargo-publish.yml` and of `crate-release.yml`.
Each entry holds

- `crateDir`,
  the repository-relative directory of the crate with its `Cargo.toml` and `Cargo.lock`;
- `scope`,
  the commit scope of the release commit;
- `pinFiles`,
  the manifests that pin the crate exactly,
  such as a fuzz crate,
  each moved with the crate and committed with the `Cargo.lock` beside it;
- `checkBins`,
  the binaries the check builds,
  empty to check the whole crate;
- `lockDirs`,
  the directories whose lockfile records the changed version of a workspace member,
  each refreshed with `cargo update --workspace` and committed with its `Cargo.lock`
  (a directory that only reaches the crate through another crate,
  such as `package/file-enforcer/cli` for
  `monochromatic-jsonc-edit`,
  belongs here and not in `pinFiles`);
- `dispatchGate`,
  optional:
  the command a real release of the crate waits for,
  as its program and then its arguments,
  run from the crate's own directory.
  Every word must hold text and the list must not be empty,
  because a release that waits for nothing is nobody's check.

Only `crateDir` and `scope` are required.
Unknown fields are refused.

Every crate this repository publishes is registered:
the libraries
`monochromatic-jsonc-edit`,
`monochromatic-dependency-catalog`,
`monochromatic-browserslist`,
`monochromatic-xml-edit`,
`monochromatic-catalog-specifier`,
`monochromatic-file-enforcer` and
`forbidden-regex`,
and the tools
`forbidden-strings`,
`monochromatic-search-shadow`,
`monochromatic-nested-wayland-session`,
`monochromatic-lint`,
`monochromatic-pnpm-shadow`,
`monochromatic-file-enforcer-cli`,
`monochromatic-cargo-shadow`,
`monochromatic-git-policy-cli` and
`monochromatic-mpm-shadow`.
A library has no GitHub release,
so the `release:wait` check of a library ends with `PUBLISH CHECK FAILED` that names only the missing release assets,
and its `crates.io` line is the result.
The release job installs the Wayland build libraries only when the crate is `monochromatic-nested-wayland-session`,
which links them.
A pin and a lock directory are found by scanning every tracked `Cargo.toml` and `Cargo.lock` for the crate
(a dependency line,
a `[dependencies.<crate>]` table,
a lockfile package without a `source`);
the unit test `release-workflow.unit.test.ts` checks the pin files against the repository,
but only a scan finds a lock directory that is missing.
To make another crate releasable,
add its entry,
add its name to the `crate` choices of `crate-release.yml`
(a unit test fails until both match),
and give its package `release` and `release:wait` tasks like those of `package/forbidden-strings/cli`.

## Release command

```bash
# what the workflow runs on the runner
node bin/release.ts --crate <name> --host runner -- <from> <to> <note>

# the same with every argument spelled out, for a crate that is not in the registry
node bin/release.ts --crate-name <name> --crate-dir <dir> --scope <scope> \
  [--pin-file <Cargo.toml>]... [--check-bin <name>]... [--lock-dir <dir>]... \
  [--host <local|runner>] <from> <to> <note>
```

The command moves a crate from `<from>` to `<to>` and records the bump as a local commit.
It never pushes and never publishes.
In order,
it

1.  edits the `version` of the `[package]` table in `<crate-dir>/Cargo.toml`;
2.  edits the exact pin (`version = "=<from>"`) of the crate in each `--pin-file` manifest;
3.  runs `cargo check` (with one `--bin` per `--check-bin`) in the crate directory;
4.  runs `cargo update --workspace` in each `--lock-dir`,
    in order,
    which records the new version of the workspace member without moving any other locked version;
5.  runs `cargo publish --dry-run --locked --allow-dirty` in the crate directory;
6.  commits the manifests and the `Cargo.lock` beside each of them,
    and the `Cargo.lock` of each `--lock-dir`,
    with the message
    `chore(<scope>): release <to>: <note>`.

Every edit is computed before the first file is written,
so a refused manifest leaves all files as they were.
The first failing step stops the run with `FAILED <step> (<status>)` and exit status 1.
Each step is printed as `== <step>` and a `$ ...` line that can be pasted into a shell.

### Hosts

`--host` says where the release runs.

- `runner` is a fresh CI checkout.
  Cargo may use the network,
  because the registry cache is empty,
  and the dry run runs directly.
  A manifest that already holds `<to>` is refused with the manifest named,
  because the bump is already on the branch and committing it again would find nothing to commit.
- `local` is the default.
  Cargo works with `--offline`,
  the dry run is capped in a transient `systemd-run --user --wait --pipe` unit
  that carries all four limits the owner's rule asks of a capped unit:
  `--memory-max` (default `12G`),
  `--cpu-quota` (default `800%`),
  `--tasks-max` (default `2048`,
  threads included),
  and a fixed disk weight of 10 (`IOWeight`),
  which no option raises.
  The defaults come from a cold dry run of the largest releasable crate,
  and the evidence sits beside each value in `src/constants.ts`.
  `--unit-name` names the unit (default `<crate-name>-publish-dry`),
  and a manifest that already holds `<to>` is reported and not rewritten,
  which makes a run that failed at a later step safe to run again.
  It compiles the crate on the machine that runs it.
  The owner's decision is that releases do not run there,
  so the tasks do not use it.

### What the edits accept

- The crate version is the `version` key of the `[package]` table only.
  A `rust-version` key,
  a `version` key in any other table and a dependency's version are never touched.
- The current value must equal `<from>` exactly,
  or `<to>` (no change,
  accepted on a local host only).
  Any other value,
  including a longer one that starts with `<from>` such as `0.4.21` for `0.4.2`,
  is refused with the manifest and both
  values named.
- A pin is the `version = "=<version>"` of a dependency on the crate,
  written either as the inline table
  `<crate> = { ..., version = "=<version>" }` or as a table of its own,
  `[dependencies.<crate>]` (also `[dev-dependencies.<crate>]` and `[build-dependencies.<crate>]`) with the
  `version` key on a line of its own before the next table header
  (the fuzz crates of `jsonc-edit`,
  `xml-edit` and `forbidden-strings/core` use the second form).
  Every dependency of either form in a manifest is edited together,
  and a refusal in one leaves the other unedited.
  A loose requirement such as `"0.4.2"` is refused,
  a crate whose name only starts with this crate's name is not
  matched,
  and so are an array of tables,
  a `[workspace.dependencies.<crate>]` table and a nested table of the crate.
  Measured 2026-11-07T05:17:51-05:00:
  this edit applied to every registered crate at once (each one patch version up)
  left `file-enforcer check` with no drift beyond the local `.config/mise/conf.d/10-bin.local.toml`,
  so a release leaves nothing for `file-enforcer write` to correct.
- Notes are one line without control characters,
  at most 200 characters.
  Quotes,
  `$`,
  `;`,
  backticks and `--looks-like-an-option` are kept verbatim,
  because the message reaches Git and `gh` as one
  argument and no shell reads it.
  Put a lone `--` before a note that starts with a hyphen.

### Release options

Run the command without arguments,
or with `--help`,
for the same list with defaults.

- `--crate <name>` reads the crate directory,
  scope,
  pins,
  check binaries and lock directories from the registry.
  It cannot be combined with the options that spell them out.
- `--crate-name`,
  `--crate-dir` and `--scope` are required without `--crate`:
  the crate name,
  its repository-relative directory,
  and the
  scope of the commit subject.
- `--pin-file <Cargo.toml>`,
  repeatable,
  names a manifest that pins the crate exactly.
- `--check-bin <name>`,
  repeatable,
  narrows the check to those binaries.
- `--lock-dir <dir>`,
  repeatable,
  names a directory whose lockfile is refreshed after the edit.
- `--host`,
  `--memory-max`,
  `--cpu-quota`,
  `--tasks-max` and `--unit-name` are described under hosts.
- `--root <dir>` overrides the repository root,
  which is otherwise found by walking up from the package to the nearest
  `pnpm-workspace.yaml`.

## Dispatch command

```bash
node bin/dispatch-release.ts --crate <name> [--dry-run] [--repo <owner/name>] <from> <to> <note>
```

The command refuses what the workflow would refuse,
before it spends a run:
a crate that is not in the registry,
versions that are not release versions or are equal,
and a note that cannot be a commit subject for the crate's scope.
Then it checks that origin holds the local commits (see "Unpushed commits"),
and only then runs `gh workflow run crate-release.yml` with the inputs as `--raw-field` arguments,
shows what `gh` printed,
and ends with `RELEASE STARTED <run URL>`.
The branch is the default branch of the repository.
When `gh` prints no run address,
the closing line says so and shows `gh run list --workflow=crate-release.yml --limit=1`.

### The dispatch gate of a crate

A crate whose registry entry names a `dispatchGate` gets one more check,
which is the last thing between the person and a run nothing can take back:
the command runs after every refusal that costs nothing,
from the crate's own directory,
with what this machine has and a GitHub runner does not.
The dispatch prints `Waiting for <the command>` and hands the gate its own streams,
so everything the gate prints stays visible for as long as it takes;
when the gate exits nonzero the dispatch refuses,
shows the line that names the gate and its status,
and never calls `gh`.
A `--dry-run` skips the gate and prints that it did,
because a dry run runs nothing the gate's evidence would cover.
Only `monochromatic-git-policy-cli` names one:
`node bin/native-release-host-controls.ts`,
the btrfs controls that cannot run in the container gate
(`package/git-policy/cli/README.md`).

### Unpushed commits

The workflow builds the default branch of the repository on GitHub,
not this checkout.
Auto-push can lag or stop,
and a release of a crate version cannot be taken back from crates.io,
so a dispatch made while the local default branch holds commits origin does not have
would publish old source under the new version.
Before it calls `gh`,
the command therefore

1.  asks origin for its default branch with `git ls-remote --symref origin HEAD`,
    which names the branch `gh workflow run` builds when it gets no branch,
    so no branch name is written into the tool;
2.  fetches that branch into `refs/remotes/origin/<branch>` with `git fetch`;
3.  runs `git merge-base --is-ancestor refs/heads/<branch> refs/remotes/origin/<branch>`:
    status zero means origin holds the local branch.

It refuses,
exits with status 1 and never calls `gh` when

- origin lacks commits.
  The message gives how many local commits origin lacks,
  the short hash and subject of the newest,
  and the two ways out:
  wait for auto-push,
  or run `git push origin <branch>`;
  then run the same release command again.
- a step gave no answer (offline,
  no credentials,
  no `origin`,
  no local branch of that name).
  The message names the `git` call that failed and shows Git's own message,
  because publishing unchecked is irreversible.
  Fix the cause and run the same release command again.

Each `git` call is printed as a `$` line first.
A `--dry-run` is refused the same way:
it builds origin's branch too,
so a dry run on old source would prove nothing about the release.

The check covers the whole default branch,
not only the directory of the crate (`crateDir`).
A check of the crate directory would pass while a shared input the release reads was unpushed:
a `Cargo.lock` of a `lockDirs` entry,
a pinned manifest,
the `Cargo.lock` of a workspace,
or `.github/workflows/crate-release.yml` and `cargo-publish.yml` themselves.
Listing those inputs would need the same registry data and the same rules as the push command's path comparison,
and would fail open when the list is out of date,
while "origin holds every local commit" has no list to keep.
The price is that an unrelated unpushed commit blocks a release until it is pushed,
which auto-push does within moments in normal use.

The check compares with the remote named `origin`.
A dispatch with `--repo` for a repository other than `origin` still compares with `origin`.

## Await command

```bash
node bin/await-release.ts --crate <name> [options] <run URL or id> <version>
```

The command

1.  follows the release run to its own end,
    reading the answer of `gh run view --json=createdAt,status,conclusion,jobs`,
    and never the exit status of a watcher:
    `gh run watch --exit-status` exits 1 both for a run that failed and for a failure of the watcher itself,
    which reported a release that passed as failed once
    (`doc/troubleshooting/gh-run-watch-exit-status-conflates-failures.md`).
    A run that ended with success passes and is reported as such;
    a run that failed stops the wait with `RELEASE RUN FAILED`,
    the job and the step that did not pass,
    and the `gh run view <id> --log-failed` command,
    because a failed release run starts no publish;
    a run that has not ended says which jobs it is still running and is looked at again,
    for a bounded number of looks;
2.  reads when the release run was created,
    from the same answer,
    and finds the publish run:
    the earliest manually started run of `cargo-publish.yml` created at or after it
    and built from a commit whose `Cargo.toml` of the crate holds `<version>`,
    read with `gh api` at each candidate commit,
    oldest first.
    A publish run that someone else dispatched in the same window on a commit before the bump
    holds the previous version,
    so it is never taken for this release's run.
    Before the bump lands no commit holds `<version>`,
    and the release dispatches the publish on `main` after it,
    so the commit is the bump or a later one.
    The run listing does not say which crate a run publishes,
    so a dispatch for another crate made on `main` after the bump and before the release's own dispatch cannot be told apart.
    When no run matches,
    the last result lists every run seen and why it was passed over;
3.  follows the `cargo-publish.yml` run to its own end the same way,
    then checks crates.io (unless the version never goes there) and the GitHub release for the expected assets,
    as the wait command does.
    A run that is still building keeps the wait going,
    with a line per look naming the jobs it is still running,
    until the look limit.

It exits 0 only when every part succeeded,
and its last line names the release it found,
for example `PUBLISHED monochromatic-git-policy-cli-v0.6.1-alpha.9`.

Its publish check options (`--workflow`,
`--repo`,
`--attempts`,
`--poll-interval`,
`--run-limit`,
`--tag-template`,
`--asset-template`,
`--user-agent`,
`--max-time`) are those of the wait command.

## Push command

```bash
node bin/push-release.ts --crate <name> --repo <owner/name> --branch <branch> --scratch-dir <dir> [--max-attempts <count>]
```

The release workflow runs this after the release command.
It reads the head commit of the checkout:
its parent,
which must be the only parent and is the base the release verified,
its subject,
and the files it changes.
It writes a `createCommitOnBranch` GraphQL request that adds exactly those files,
with their whole contents,
on top of that parent,
and sends it with `gh api graphql --input`.
GitHub signs a commit made through that mutation,
which a branch that requires signed commits accepts.
The environment must hold `GH_TOKEN`.

### A branch that moves during the release

GitHub refuses the commit when the branch is no longer at the head the request expects,
and `main` moves every few minutes while a release takes about half an hour.
A plain refusal would make a re-dispatch lose the same race,
so the command settles it instead of giving up.

Before it sends anything,
it records every path the release depends on:

- the files the release commit changes;
- the directory of the crate and the directories whose lockfile the release refreshed (`lockDirs` in the registry);
- every path dependency in the dependency closure of the crate and of each lock directory,
  read with `cargo metadata --locked` at the checkout,
  which is the tree the check and the dry run built;
- the manifest and the lockfile of each workspace root.

A package from a registry or a git source is not watched,
and neither is a path package outside the repository.
A path package at the repository root is refused,
because its directory is every file.

When a send is refused with GitHub's words for a stale expected head,
the command

1.  reads the new head of the branch with `gh api repos/<repo>/git/ref/heads/<branch>`;
2.  compares the checkout base with the new head through the compare endpoint,
    `repos/<repo>/compare/<base>...<head>`,
    and keeps the changed paths that are one of the watched files,
    or are or lie below one of the watched directories.
    The test is exact:
    `package/ab` is not below `package/a`,
    and a renamed file counts by its old and its new path;
3.  sends the same commit again,
    expecting the new head,
    when no watched path changed,
    and starts again from step 1 if the branch moved once more,
    comparing from the checkout base each time;
4.  pushes nothing,
    names the changed paths and fails the run when one did change.
    It does the same when the new head cannot be read,
    when the comparison fails or the branch is not ahead of the checkout,
    and when the branch moved before every allowed send (`--max-attempts`,
    default six,
    the first send included).

The compare endpoint lists at most three hundred files for a whole comparison.
A list that long may have been cut and cannot show a path unchanged,
and on this repository a half-hour window has reached it.
The command then decides by object names instead:
one GraphQL query reads the object name of each watched path at the base and at the new head,
and equal names prove a path unchanged whatever else changed on the branch,
because the name of a tree covers everything below it.

Every step is printed:
the base,
the watched paths,
each new head,
the comparison,
and the decision.
Files that a build script or `include_str!` reads from outside the crate and path dependency directories are not seen,
because `cargo metadata` does not list them.

## Wait command

```bash
mise run //package/crate-release/cli:release:wait:commit -- --crate-name <name> [options] <sha> <version>
```

The command waits for the publish workflow run of commit `<sha>` (a prefix of at least seven hexadecimal digits),
follows the run to its own end,
then checks crates.io for `<version>` (unless the version never goes there) and the GitHub release for the expected assets.
It is for a publish that something other than the release workflow started,
such as a push to `main`.
It prints

```text
cargo-publish.yml run 40991272842: completed with success
crates.io 0.4.3: 2026-10-30T02:26:00.989331Z
release monochromatic-git-policy-cli-v0.4.3 assets: ...x86_64-unknown-linux-gnu.tar.gz, ...
PUBLISHED monochromatic-git-policy-cli-v0.4.3
```

and exits 0 only when the run ended with success,
crates.io has the version and the release holds every expected asset.
A run that has not ended is reported as such and looked at again:

```text
cargo-publish.yml run 40991272842 is still going: gpc-build-binary (in_progress); waiting up to 40 minutes
```

A run that failed names the job and the step that did not pass,
for example `job gpc-publish-crate concluded failure at step "Publish the crate to crates.io"`.
Otherwise the report ends with `PUBLISH CHECK FAILED: ...` naming each reason,
and exits 1.
A failed or unfinished run does not skip the checks,
so the report shows everything that is and is not in place.
When no run appears it prints the bound that was waited,
for example `within 15 minutes`,
and the last answer it got.

The runs are listed as JSON and filtered in code;
the hash is never placed in a `gh --jq` program.

### Wait options

- `--crate-name` is required.
- `--workflow <file>` (default `cargo-publish.yml`) and `--repo <owner/name>` (default:
  read by `gh` from the working
  directory,
  which is the repository root).
- `--attempts <count>` (default 45),
  `--poll-interval <seconds>` (default 20) and `--run-limit <count>` (default 10)
  bound the wait and each listing;
  the defaults wait at most 15 minutes for a run to appear.
- `--follow-attempts <count>` (default 120) and the same poll interval bound how long one run is followed to its end,
  so the defaults follow one run for 40 minutes.
  Raise it for a crate whose archive build is slower than the two releases measured on 2026-11-11,
  which took 4 minutes 38 seconds and 7 minutes 28 seconds.
- `--tag-template <text>` (default `{crate}-v{version}`) and `--asset-template <text>`,
  repeatable (default the `x86_64`
  and `aarch64` Linux tarballs,
  `{crate}-{version}-<target>.tar.gz`).
  Templates must hold `{version}`.
  Add `--asset-template '{crate}-{version}-sha256sums.txt'` to require the checksums file as well.
- `--user-agent <text>` (default `crate-release-check`) and `--max-time <seconds>` (default 30) shape the crates.io
  request.
- `--root <dir>` as for the release command.

## Source layout

- `bin/` holds the executables:
  `release.ts`,
  `dispatch-release.ts`,
  `push-release.ts`,
  `await-release.ts` and `wait-publish.ts`.
  They only read the process state and spawn;
  they hold no logic.
- `src/` holds the pure builders and parsers,
  each with a unit test:
  the reader of a run's own answer,
  which is the only verdict a wait accepts
  (`run-outcome.ts`,
  tested against the answers of real runs recorded under `src/fixture/`),
  version and name validation (`version.ts`,
  `validate.ts`),
  the registry (`crate-registry.ts`),
  the manifest edits (`manifest-edit.ts`,
  `dependency-pin-sites.ts`,
  `toml-line.ts`),
  the commit message (`commit-message.ts`),
  the commit request of the push command (`commit-request.ts`),
  the program arguments (`release-steps.ts`,
  `service-arguments.ts`),
  the result parsing and formatting
  (`publish-results.ts`,
  `run-results.ts`),
  and the option parsers (`release-options.ts`,
  `dispatch-options.ts`,
  `push-options.ts`,
  `await-options.ts`,
  `wait-options.ts`).
- `crates-io-version-label.ts` gives a version its label and says whether it goes to crates.io,
  `crates-io-dependency-check.ts` refuses a crates.io release whose dependencies do not,
  and `crates-io-label-gate-workflow.unit.test.ts` checks the decision action,
  that each of the sixteen crates of `cargo-publish.yml` decides and gates its crates.io steps,
  and runs the real `detect` script of every crate for alpha,
  beta,
  rc and plain versions
  (`workflow-text.ts` holds the readers of workflow files that the workflow tests share).
- `origin-containment.ts` builds the `git` calls of the origin check,
  parses origin's default branch and writes the refusals.
- `release-run.ts`,
  `dispatch-run.ts`,
  `push-run.ts`,
  `await-run.ts` and `wait-run.ts` orchestrate through injected readers,
  writers,
  program runners and sleepers,
  so
  tests drive them without touching a file,
  a network or a real program.
  `cli.unit.test.ts` starts the real executables with stand-in `cargo`,
  `git`,
  `gh`,
  `curl` and `systemd-run`
  programs first on the search path,
  in a disposable directory under `<home>/temp/agent`.
- `release-workflow.unit.test.ts` reads the workflow files and the repository:
  it compares the `crate` choices of `crate-release.yml` with the registry and with `cargo-publish.yml`,
  checks that each registry entry names real manifests,
  an exact pin at the crate's current version and the files a release commits,
  and keeps the workflow within the repository's shell budget and permission rules.
- No program is started through a shell.
  The only shell-facing text is the `$ ...` line printed for people,
  and it is built with `shellQuote`.

## Tasks

[`mise.toml`](mise.toml) defines this package's tasks.
Run one with `mise run //package/crate-release/cli:<task>`:
`release` dispatches,
`release:wait` follows a release,
`release:run` and `release:push` run the release and push commands,
and `release:wait:commit` waits for the publish run of a commit.

[token-trigger-rule]: https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow#:~:text=events%20triggered%20by%20the%20GITHUB_TOKEN%20will%20not%20create%20a%20new%20workflow%20run
