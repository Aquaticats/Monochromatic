# cli-git native cutover runbook

Note 2026-10-29: the runner scripts this document names under `package/git-policy` (`cli/bin`,
`cli/native-interop` and `cli.fuzz/bin`) are TypeScript files now. Read every `.mjs` file name here as the same
name ending in `.ts`; the driver `old-format-fixtures` is `old-format-driver`.

## Purpose and how to use this document

This document is the runbook for the cutover of the Git policy wrapper:
installing the published native crate as a `cargo:` mise tool,
migrating consumers,
documentation,
and retirement of the TypeScript implementation.
The user's standing decision is that there is no hybrid and the old TypeScript goes
(`doc/handover/cli-git-rust-implementation.md`,
 queue item "Commit path,
 run as parallel lanes",
 and the section `User decisions 2026-10-05`).

State on 2026-10-13:
the crate `monochromatic-git-policy-cli` 0.1.1 is published on crates.io,
the GitHub release `monochromatic-git-policy-cli-v0.1.1` carries the Linux gnu x86_64 and aarch64 archives,
and Trusted Publishing is configured for the crate.
Version 0.1.0 is also published;
 it must not be pinned (see `Why the pin is 0.1.1`).
The install lane (branch `feat/cli-git-native-install`,
 from `main` at `0f79ccf033`,
 merged with `main` at `4e3eda3798`) made every repository edit
that does not switch the live `git`:
the tool entry pinned to 0.1.1,
the `_.path` entry,
the lock entry,
the retirement of the hk cleanup and of the performance harness,
the release build as the package `build` task,
the CI jobs that need the wrapper,
and the native recognition in `package/git-executable/core`.
Nothing in the real installation was installed,
 replaced,
 or published by that lane;
 its probes ran against a disposable `HOME`,
 mise directories and `PATH` (see `Evidence basis`).
What remains is in `Steps that remain`,
 `User-boundary verification` and `Order of the retirement commits`.

Retirement state on 2026-10-14:
the retirement commits are on the branch `feat/cli-git-native-retire`
(record,
 gates,
 leftovers and the cleanup list for the main checkout:
 `doc/handover/cli-git-native-retire.md`).
The sections below describe the plan and the facts that led to it;
where they say a file "will be deleted" or "stays",
 the branch is the outcome.

Facts measured on 2026-10-12 carry that date;
 facts measured on 2026-10-13 say so.
The scout of 2026-10-12 read `main` at `e1890fb471`.

Terms:
the incumbent is the TypeScript wrapper (every file of `package/git-policy/cli` outside `src/native/`);
native is the Rust executable `cli-git-native` built from `package/git-policy/cli/src/native/`.
Paths starting with `src/`,
 `bin/`,
 `e2e/`,
 `perf/` or `native-interop/` are relative to `package/git-policy/cli/`;
every other path is repository-relative.
`file:line` citations were read in a checkout at the time of writing and move with edits;
re-read before relying on one.

How to use it:
`Decisions` lists what is settled and what is open to veto.
`Install the published tool` describes the tool entry and its isolated verification.
`Installation sequence with rollback` and `Order of the retirement commits` give the order of the remaining work.
`Not established` lists the facts no probe settled.

## Decisions

### Decided by the user on 2026-10-13

- Install route:
  the native crate is published to crates.io and installed as a `cargo:` mise tool,
   like the published linter `monochromatic-lint` (root `mise.no-env.toml`,
   `package/linter/monochromatic-lint`).
  Done:
   the crate is published (see `Publication state`).
- Hk cleanup:
  the task `cleanup:hk-git-config`,
   `doc/runbook/remove-retired-hk-git-config.md` and the task `test:hk-config-cleanup` are retired.
  Done on the install branch;
   `src/maintenance/hk-config-cleanup*.ts` and its fixture `src/trust/fixture/hk-config-cleanup-consumer.ts` stay until
   `src/` goes.
  `doc/troubleshooting/cli-git-hk-retirement-stale-hook-config.md` now gives the manual removal.
- Rollback window:
  it ends right after the installed wrapper is verified.
  The incumbent's shim,
   `dist` and dependency links stay only until that verification passes.
  Afterwards rollback is versioned (pin the previous published crate version);
  returning to the incumbent then means a fresh install from the pinned revision (see `Rollback`).
- First native workflow:
  Linux only.
  Hosted macOS and Windows runners wait for the trailing Windows checks.

### Adopted without asking on 2026-10-13, open to veto

- Stale worktrees:
  switch after the worktrees in active use have merged `main`,
   run `mise run sync:files` and run `mise install`;
  stale worktrees fail with the migration message quoted under hazard 3.
- Verification tools with the incumbent as reference:
  pin the last incumbent revision (its hash goes in the retirement commit and the behavior ledger);
  delete the commit,
   configuration and dependent-version differentials;
  keep `native:interop` and give it an incumbent-provisioning step from the pinned revision
  (superseded: the matrix ran once against a rebuilt incumbent and the user then had the incumbent tooling deleted,
  `doc/handover/cli-git-native-interop.md`).
- Release and CI jobs:
  build the crate from the checkout with `cargo build --release --locked`.
  Done for `final-newline.yml` and for the `version` job of `npm-release.yml`.
  Deviation from the adopted wording "put `git` first on `PATH` for the step":
   the release job puts `package/git-policy/cli/target/release` on `PATH` under the executable's own name,
   and `changeset:version` runs `cli-git-native cli-git fix ...`,
   because the changesets action runs `git add .` and `git commit` in the same environment as its script,
   and the wrapper refuses bulk staging.
- The four authoring and policy packages (`package/git-policy/{api,forbidden-strings,markdown-lint,repository}`):
  delete after their consumers have moved.
- Performance:
  `perf/*.json` stays;
   the harness (`perf/*.ts`),
   its tasks and `cli-git-performance.yml` are deleted.
  Done on the install branch.
- Package `build`:
  is the release build of `cli-git-native` (`cargo build --release --locked --offline --bin cli-git-native`).
  Done on the install branch.
  `pack:npm` and the supported-runtime job of `cli-git-trust.yml` call `build:js` instead,
   so the incumbent's bundle still builds until `src/` goes.
- Crate name:
  `monochromatic-git-policy-cli` (rationale and cost under `Name`).
- Location of the `git` link:
  a gitignored per-worktree directory `.cache/cli-git-bin`,
   first in `PATH` (rationale under `Install the published tool`).

## Evidence basis

Read-only probes run for this document,
 all on 2026-10-12:

- `command -v git`,
   `type -a git`,
   `git --version`,
   `git cli-git --help`,
   `git cli-git status`,
  `mise env --json` (PATH only),
   `mise exec -- command -v git` and `mise exec -- git --version` in this worktree.
- `ls`,
   `sha256sum` and `rg` over the main checkout's `package/git-policy/cli/dist`,
   `node_modules` and `.git`.
- `find` over `.git` for transaction directories and locks,
   and `pgrep` for running wrapper processes.
- `gh api` GET calls for branch protection and rulesets of `Aquaticat/Monochromatic`.
- A scratch repository under `${HOME}/temp/agent` driven through a private PATH entry that linked the integration
  worktree's debug `cli-git-native` as `git`;
   it was deleted afterwards and nothing production was shadowed.
  It produced the exact refusal texts quoted under `Coordination hazards and detection`.
- Reads of the handover,
   planning and decision documents named in the citations.

Read-only or throwaway probes run on 2026-10-13,
 after the user's answers:

- `GET https://crates.io/api/v1/crates/<name>` for the candidate names and for the dependency crates.
- Downloads of `forbidden-strings` 0.4.1,
   `forbidden-regex` 0.1.1 and `monochromatic-jsonc-edit` 0.1.1 from crates.io
  into a scratch directory,
   compared with the local sources.
- `cargo package`,
   `cargo package --list`,
   `cargo publish --dry-run` and `cargo check` in the package and in
  throwaway copies under `${HOME}/temp/agent`,
   with `CARGO_TARGET_DIR` outside the repository.
  No upload happened.
- `cargo build --release --locked --offline --bin cli-git-native` into a scratch target directory.
- `mise` 2026.10.0 in a throwaway `MISE_DATA_DIR`,
   state directory and project,
   with a real install of
  `monochromatic-lint` 0.1.1 standing in for the crate (the crate was not yet published then).
  `mise env --json`,
   `mise install`,
   `mise exec`,
   `mise reshim`.
- `git diff` between the gated commit and the current head for `package/git-policy/cli`.

Probes run on 2026-10-13 by the install lane,
 all in disposable state under `${HOME}/temp/agent`:

- `mise lock cargo:monochromatic-git-policy-cli` (dry run first) in the install worktree:
   it added one `[[tools."cargo:monochromatic-git-policy-cli"]]` block to `mise.lock` and rewrote nothing else.
- `mise install cargo:monochromatic-git-policy-cli` from the install worktree,
   with a disposable `HOME`,
   `XDG_*` directories,
   `MISE_DATA_DIR`,
   `MISE_CACHE_DIR`,
   `MISE_CONFIG_DIR`,
   `MISE_STATE_DIR`,
   `MISE_GLOBAL_CONFIG_FILE`,
   `MISE_TRUSTED_CONFIG_PATHS` limited to the worktree and `MISE_CEILING_PATHS` at the worktree's parent.
  The tool's install dependencies `rust` and `cargo-binstall` were provided as copies of the live mise install
  directories (links to the live `rustup` proxies,
   no download).
- A throwaway repository with the linked executable first on `PATH`:
   `git --version`,
   `git cli-git --help`,
   `git cli-git status`,
   `git worktree add`,
   `git add`,
   `git commit` of a text file and of a Markdown file,
   the two refusals,
   and the leftover search of hazard 1.
- `strace` of the Markdown commit with 0.1.0 (system calls `execve`,
   `statx` and `newfstatat` only) to locate the post-commit failure described under `Why the pin is 0.1.1`.
- The same throwaway-repository run again with the 0.1.1 archive installed from the published release through the
   throwaway-project block of `Steps that remain`,
   plus `git worktree add` from a repository that has a gitignored `.cache/forbidden-strings.rules.txt`.
- The dependent-version release step on the real repository:
   two clones of the install branch with the version of `package/async-time/core` raised,
   one through `node package/git-policy/repository/src/bump-dependents.ts`,
   one through the installed executable's `cli-git fix --all --policy mono/dependent-version-bump`.
- The new `bin/final-newline-workflow.mjs` against the installed executable,
   in a clone of the install branch.
- `actionlint` 1.7.12 over `final-newline.yml`,
   `npm-release.yml` and `cli-git-trust.yml`.

## How the wrapper is installed and run today

### The installed incumbent

The `git` that a session in this repository resolves is a pnpm shim,
 not a wrapper binary:

- `command -v git` prints `/var/home/user/Monochromatic/node_modules/.bin/git`;
  `type -a git` then lists `/usr/bin/git` and `/bin/git`.
   `git --version` prints `git version 2.55.0`.
- That shim is a 2,314-byte POSIX script (dated Sep 22) whose last lines `exec node` on
  `node_modules/@monochromatic-dev/git-policy-cli/dist/final/node/index.mjs`.
  `node_modules/@monochromatic-dev/git-policy-cli` is a symlink to `../../package/git-policy/cli`.
- So the program every session runs is `package/git-policy/cli/dist/final/node/index.mjs` of the main checkout:
  1,439,660 bytes,
   built Oct 10 14:48,
   SHA-256 `d7c26d53eb463d8f3d0137905931d59bac5164915e81301a7e8206fb83dc0b47`.
  `dist/` is ignored (`.gitignore:53`),
   so the artifact is not in any commit.
- The bundle is not standalone.
   `dist/final/node/index.mjs:18` to `:21` import `valibot`,
   `yuku-parser`,
  `yuku-ast` and `ignore` as bare specifiers.
  They resolve through the symlinks in `package/git-policy/cli/node_modules/` (into `node_modules/.pnpm`).
  The incumbent therefore runs only while the cli package's own `node_modules` links exist.
- The shim exists because the root manifest depends on the package (`package.json:36`
  `"@monochromatic-dev/git-policy-cli": "workspace:*"`) and the package declares `bin.git`
  (`package/git-policy/cli/package.json:17`).
   `pnpm install` links that bin into `node_modules/.bin/git`;
  the shim is an install-state file,
   not a tracked file.

### `git cli-git trust` and the trust registry

- The incumbent's management commands are `trust`,
   `untrust`,
   `status`,
   `check`,
   `fix` (`git cli-git --help`).
  `git cli-git status` in this worktree prints a JSON `trust-status` event with `"trusted":true` for the worktree's
  `cli-git.config.ts`.
- Trust is consent to run the executable configuration `cli-git.config.ts`.
  Records live outside the repository in `~/.local/state/cli-git/trust/v1` (the directory exists;
  its contents were not read).
- A changed config needs `git cli-git trust --yes` before the incumbent runs it again.
  The linter cutover did exactly that after the user approved it (`doc/handover/unified-linter-cutover.md`,
  section `Landing choreography`,
   step 3).
- The native executable retires these commands:
   `git cli-git trust` and `git cli-git status` print
  "`git cli-git trust is retired. cli-git now reads cli-git.config.jsonc as data ...`" and exit 0,
   change nothing,
  and leave the old records in place and unread (observed with the integration worktree's debug build;
  decision in `doc/handover/cli-git-rust-implementation.md`,
   `User decisions 2026-10-05`).
  This difference is the cheapest way to tell which wrapper a `git` is:
   the incumbent prints JSON for `status`.

### Tasks that build or install the incumbent

- `mise run //package/git-policy/cli:build:js` (template `build:js` in `mise.no-env.toml`,
   then `build:js:node`,
   which runs `rolldown --configLoader native --config rolldown.node.config.ts`) produces
  `dist/`.
   The task `build` of this package is no longer that:
   it is the release build of `cli-git-native`.
   The root `mise run build` fans out to every package's `build` (`vars.fanout_packages`),
   and `prepareAndBuild` is `prepare` then `build`.
- `pnpm install` (root task `prepare:pnpm:install`) creates the `node_modules/.bin/git` shim.
- `mise run //package/git-policy/cli:pack:npm` (`package/git-policy/cli/mise.toml:141`) builds the unpublished tarball
  `dist/pack/monochromatic-dev-git-policy-cli-0.0.1.tgz`;
  the incumbent e2e image and the packed-trust test depend on it.
- The previous wrapper replacement was manual:
   build `dist` in a branch worktree,
  then copy it over the live one by rename (`doc/handover/unified-linter-cutover.md`,
   `Landing choreography`,
   step 3).

### PATH mechanics on this host

- The root `mise.toml` is generated from `mise.no-env.toml` by file-enforcer (`mise.toml:1-5`);
  the `[env]` block with `_.path` is written by `generateMiseToml` in `file-enforcer.config.ts:656-720`.
  `_.path` lists `node_modules/.bin` first (`file-enforcer.config.ts:680`) and then every
  `package/*/*/node_modules/.bin` found on disk,
   all relative to the config root.
- `mise env --json` run in this worktree gives a PATH whose first mise entries are this worktree's
  `node_modules/.bin`,
   then the other package bin directories,
   then the mise tool directories,
  so `_.path` entries precede tool directories and `/usr/bin`.
Probed on 2026-10-13 in a throwaway project (mise 2026.10.0):
   `{{env.HOME}}` and `~` in `_.path` both expand to the home
directory,
   a relative entry resolves against the config root,
   and `_.path` entries precede tool `bin` directories.
- This session's PATH instead names the main checkout's `node_modules/.bin` first,
  because the session was started from the main checkout.
   PATH is fixed when a process starts;
  changing `mise.toml` does not change running sessions.
- A worktree has its own `node_modules/.bin/git` shim pointing at its own `package/git-policy/cli/dist`.
  In this worktree no `dist` exists,
   so `mise exec -- git --version` fails with
  `Cannot find module '.../cli-git-lane-cutover-scout/package/git-policy/cli/dist/final/node/index.mjs'`.
  The same failure is recorded for `gh` in `doc/troubleshooting/gh-implicit-repository-git-wrapper.md`.
  Sessions in worktrees only work because they inherit the main checkout's PATH.
- Real Git is `/usr/bin/git` 2.55.0.
   The incumbent's debug output in this session shows it resolving `/usr/bin/git`
  through common-platform-path priority.

### The native executable and how it is built

- The crate is `package/git-policy/cli/Cargo.toml` (`name = "monochromatic-git-policy-cli"`,
   `version = "0.1.1"`,
   published;
   see `Publication state`).
  Its binary is `cli-git-native` (`[[bin]]`,
   path `src/native/main.rs`),
   deliberately not named `git`;
  the manifest comment says the repository's mise tool entry links it as `git` after installing it.
  `Cargo.toml` is managed by file-enforcer (`generateCargoManifests`,
   `file-enforcer.config.ts:1777`),
  so comment edits are followed by `mise run sync:files`.
- Host build:
   `mise run //package/git-policy/cli:native:build` runs
  `cargo build --locked --offline --bin cli-git-native`,
   a debug build for differentials
  (`package/git-policy/cli/mise.toml:36-38`).
   The integration worktree's debug binary is 93,671,240 bytes.
  No task builds or installs a host release binary.
  The host Rust toolchain is `rust = nightly` (`mise.no-env.toml:101`).
  The user's `~/.cargo/config.toml` sets `build-dir` (`doc/decision/cargo-build-dir-host-cache.md`),
  so intermediates are outside the package's `target/`.
- Container build:
   `mise run //package/git-policy/cli:test:e2e:concurrent:native:image`
  (`package/git-policy/cli/mise.toml:413`,
   `bin/e2e-native-container.mjs`) compiles
  `cargo build --offline --locked --release --bin cli-git-native` in `rust:1.97-bookworm` from a mount-free snapshot
  of `Cargo.toml`,
   `Cargo.lock`,
   `README.md`,
   `src/native` and the path dependencies,
  then adds it to the image `localhost/cli-git-concurrent-e2e-native:<tag>` as `/opt/native/cli-git-native` with
  `/opt/native/bin/git` linked to it.
   That image starts from `localhost/cli-git-concurrent-e2e:latest`,
  whose build packs the incumbent (`package/git-policy/cli/e2e/concurrent-commits.Containerfile:56-59`).
  Image `localhost/cli-git-concurrent-e2e:latest` has id `b7cf6d8c1786`;
  `localhost/cli-git-concurrent-e2e-native:integration` has id `d04ea26e5350`.
- The container gate (`native:test:container`) and the end-to-end suite passed on `0ff1379f7d`
  (`doc/handover/cli-git-native-integration.md`,
   `Results`).
  `git diff 0ff1379f7d e1890fb471 -- package/git-policy/cli` is empty,
   so current `main` has the gated crate.
- Release artifacts exist since 2026-10-13:
   the release `monochromatic-git-policy-cli-v0.1.1` (and `-v0.1.0`) carries `x86_64-unknown-linux-gnu` and
   `aarch64-unknown-linux-gnu` archives built by `gpc-build-binary` in `.github/workflows/cargo-publish.yml`.
  The installed x86_64 file is 9,300,536 bytes for 0.1.1 (9,313,296 for 0.1.0),
   dynamically linked against `libgcc_s.so.1` and `libc.so.6` (`ldd`),
   and needs symbol version `GLIBC_2.39` at most (`objdump -T`),
   so a host older than glibc 2.39 (for example Debian 12 or Ubuntu 22.04) cannot run it.
  The host has glibc 2.43.
  A `git`-named symlink to the executable is the form the binary tests use (`src/native/binary_support.rs:74-75`),
  so a symlinked install is a tested shape.
- Runtime prerequisites the native wrapper finds itself:
   real Git on PATH after the wrapper (floor 2.55.0,
  `src/native/git_version.rs:26`),
   `monochromatic-lint` on PATH (a mise shim at
  `~/.local/share/mise/shims/monochromatic-lint`;
  pinned `"cargo:monochromatic-lint" = "0.1.1"` at `mise.no-env.toml:120`),
  `cli-git.config.jsonc` at the repository top level,
   and `.cache/forbidden-strings.rules.txt` in each worktree
  (`cli-git.config.jsonc` names it through `rulesFile`).
   The scanner is linked in-process and needs no lookup.

### What installing replaces

Replaced:
 the `git` that sessions resolve (today the shim to `dist/final/node/index.mjs`),
 the `trust` workflow,
the executable configuration `cli-git.config.ts`,
 and every Node process the incumbent starts per Git command.
Not replaced:
 `/usr/bin/git`,
 the user's trust records (left inert),
 the `monochromatic-lint` shim,
the scanner source,
 and `.git/cli-git*` state,
 which both wrappers read
(`doc/handover/cli-git-rust-implementation.md`,
 `Adopted without a question`,
 first item).

## Consumer inventory

Every reference found with `rg --hidden` over the checkout (which honors `.gitignore`),
 excluding `.git`,
`node_modules`,
 `target` and `pnpm-lock.yaml`,
 then classified.
Searched patterns:
 `git-policy-cli`,
 `git-policy/cli`,
 `cli-git.config.ts`,
 `cli-git/ts`,
 `hk-git-config`,
`hk-config-cleanup`,
 `CLI_GIT_`,
 `git cli-git`,
 `git-executable`,
 `dist/final/node/index.mjs`.
Hidden directories searched:
 `.github`,
 `.changeset`,
 `.idea`,
 `.agents`,
 `.pi`,
 `.serena`,
 `.opencodereview`,
`.out-of-scope`,
 `.claude-plugin`.
 No hits in `.idea`,
 `.agents`,
 `.pi`,
 `.serena`,
 `.opencodereview`,
`.out-of-scope`,
 `.claude-plugin`,
 `tsconfig.json`,
 `oxlint.config.ts`,
 `socket.json`,
 `.pnpmfile.mjs`,
`.pnpmfile.policies.json`,
 `forbidden-strings.append.txt`,
 `pnpm-workspace.yaml` (its `packages` glob is
`package/*/*`,
 `pnpm-workspace.yaml:420-421`,
 so no workspace entry names the package).
The main checkout's `.claude/settings.json` has no hit;
 `*.local.*` files were not read.

### Root configuration and tasks

- `package.json:36`,
   the root devDependency on `@monochromatic-dev/git-policy-cli`:
   delete the line.
  This is the line that makes `pnpm install` create `node_modules/.bin/git`.
  Land it only in the final retirement commit (see `Order of the retirement commits`),
  because a later `pnpm install` in the main checkout removes the incumbent's shim.
- The root task `cleanup:hk-git-config` is deleted from `mise.no-env.toml` and the generated `mise.toml`
  (install branch).
- `mise.no-env.toml`,
   task `changeset:version`:
   runs `changeset version`,
   then `cli-git-native cli-git fix --all --policy mono/dependent-version-bump`
   (`doc/handover/cli-git-native-dependent-version.md`,
    section `Wiring into the native executable`,
    item "The release task"),
   then `pnpm install --lockfile-only`.
  The task's optional base-revision argument has no equivalent;
   the base is `HEAD`.
  Proof on the real repository on 2026-10-13:
   with the version of `package/async-time/core` raised in two clones,
   the TypeScript planner (`package/git-policy/repository/src/bump-dependents.ts`) and the native fix changed 77
   manifests with byte-identical diffs (40,744 bytes);
   an earlier native run without `git-lfs` on `PATH` exited 2 and changed one file,
   so the comparison can see a difference.
  The release job needs `git-lfs` on `PATH` (the clone's `git add` runs the LFS filter);
   hosted Ubuntu runners have it,
   which was not checked.
  The planner (`package/git-policy/repository`) is deleted in retirement commit 2.
- `cli-git.config.ts` (root):
   delete after the native wrapper is installed and verified.
  It imports `@monochromatic-dev/git-policy-cli/ts` (`cli-git.config.ts:11-12`),
   the package's `./ts` export,
  which is `src/authoring.ts` and disappears with `src/`.
  `cli-git.config.jsonc` is its translation (all four optional policies and `rulesFile`).
  While both exist,
   native `git cli-git check` prints a `legacy-config-ignored` configuration warning
  (`doc/handover/cli-git-native-config.md`,
   `Facts the integrator must know`);
   deleting the TypeScript file ends it.
- `cli-git.config.jsonc:2`,
   the comment "Translated from `cli-git.config.ts`":
  reword to state the file is the configuration,
   so the comment does not name a deleted file.
- `.changeset/config.json:21` lists `@monochromatic-dev/git-policy-cli` under `ignore`:
  keep while the package name is kept.
   If the name goes,
   delete the line;
  `@changesets/config` 4.0.1 filters `ignore` through `globMatch`
  (`node_modules/.pnpm/@changesets+config@4.0.1/node_modules/@changesets/config/dist/index.mjs:708-719,776`) and does
  not reject unmatched names.
- `pnpm-lock.yaml` importers:
   root `.` (`:460-462`) and `package/git-policy/cli` (`:2044` onward) change with the
  manifests.
   Regenerate through the package manager,
   inspect the diff,
   never hand-edit (`LFW`).
- `package/config/pnpr/config.yaml:78` lists `@monochromatic-dev/git-policy-cli`;
  the file is generated by `generatePnprConfig` (`file-enforcer.config.ts:2062`) from manifests that have an entry
  point,
   version and no exclusion.
   Removing `main`,
   `types`,
   `exports` and `bin` from the cli manifest drops the entry
  on the next `mise run sync:files`.
   `PNPR_EXCLUDED_PACKAGES` (`file-enforcer.config.ts:1884`) names no git-policy
  package,
   so no exclusion edit is needed.
- (Removed on 2026-10-15 in `91bdc9a9f9`.)
  `slopo.ignore.txt:45,50,69,200,309,324,373` carry duplicate-code dismissals
  whose subjects are deleted TypeScript or
  `perf/` files;
   `:298` names `src/native/policy_registry.rs` and stays.
  The file's own header (`slopo.ignore.txt:40`) already counts "10 dead pending a decision",
  so dead dismissals exist in it today and removal is cleanup,
   not a blocker.
- `README.md:65` ("behind a side-effect-free policy authoring API") and `README.md:262` ("policy authoring API,
  forbidden-strings policy"):
   reword;
   the authoring API is retired.
- `AGENTS.md:1366` and `CLAUDE.md:1415` say "Never preemptively bypass `git-policy-cli` guards":
  the name is still the crate and package name,
   so keep.
   `CLAUDE.md` is generated from `AGENTS.md`.
- `monochromatic-lint.config.jsonc:58-66` scopes the native Rust path (`package/git-policy/cli/src/native/**/*.rs`):
  keep.
   Its comment cites a manual task `native:lint:rust` that no longer exists in
  `package/git-policy/cli/mise.toml`;
   that staleness predates the cutover.

### File-enforcer

- `file-enforcer.config.ts:2276-2341`:
   three `Promise.all` blocks (`mirrorRepositoryPolicy`,
  `mirrorForbiddenStringsPolicy`,
   `mirrorMarkdownLintPolicy`) copy
  `package/git-policy/{repository,forbidden-strings,markdown-lint}/src` into `package/git-policy/cli/src/optional/`.
  Delete all three in the same commit that deletes `src/optional/`;
  otherwise `mise run sync:files` regenerates the deleted files.
- `package/file-enforcer/core/src/generated-policy-contracts.unit.test.ts` (76 lines,
  only these mirror fixtures at `:49-55`):
   delete with the mirrors.
- `file-enforcer.config.ts`,
   `generateMiseToml`:
   `.cache/cli-git-bin` is the first `_.path` entry (install branch),
   before `node_modules/.bin`;
   never edit `mise.toml` by hand.
- `generateCargoManifests` (`:1777`) and `generatePackageLicenseTexts` (`:534`) keep working:
   the crate stays,
  its `LICENSES/` follow `Cargo.toml`'s license expression.

### Continuous integration

- `.github/workflows/cli-git-trust.yml`:
   delete.
  It builds the TypeScript package at the Node floor,
   runs `verify:supported-runtime`,
  and exercises trust on three operating systems (ubuntu,
   macos and windows in the job matrix at `:194-197`;
  also `:178`,
   `:187`,
   `:222-232`);
   every job tests the retired trust subsystem.
  Its `paths` filter `package/git-policy/**` (`:7,17`) also means no native change ever triggered a native job:
  there is none.
- `.github/workflows/cli-git-performance.yml` is deleted (install branch),
   with the performance harness;
   `perf/*.json` stays.
- (Removed on 2026-10-14 at the owner's request,
  together with `package/git-policy/cli/bin/final-newline-workflow.mjs`:
  the check rebuilt the wrapper on every pull request.)
- `.github/workflows/final-newline.yml` builds the crate from the checkout (`cargo build --release --locked`) and runs
  `package/git-policy/cli/bin/final-newline-workflow.mjs <executable>`,
  which clones the checkout,
   writes a core-only `cli-git.config.jsonc` (`final-newline` at `error`,
   because a `warn` finding exits 0),
   runs `cli-git check --policy final-newline --all`,
   then stages a file without a final newline and requires the policy to refuse it.
  Run on 2026-10-13 against the installed 0.1.0 and 0.1.1 executables in a clone of the install branch:
   exit 0,
   the clean pass and the refused control (a `legacy-config-ignored` warning prints until
   `cli-git.config.ts` is deleted).
  The job was not run on GitHub.
- `.github/workflows/npm-release.yml`:
   the `version` job builds the crate the same way,
   puts `package/git-policy/cli/target/release` on `PATH`,
   and its `mise run changeset:version` runs the native ripple (see `Root configuration and tasks`).
  `MISE_AUTO_INSTALL` is `false` for the workflow (`:41`),
   so the new tool entry is not installed there and the built executable is the one that runs.
  The job was not run on GitHub.
- `.github/workflows/cli-git-trust.yml` still builds the incumbent:
   its supported-runtime job runs `build:js`.
- No workflow runs the native tests on macOS or Windows
  (`doc/handover/cli-git-rust-implementation.md`,
   `Adopted without a question`,
  item on hosted runners,
   is not implemented).
  No workflow runs the native tests or the container gate on Linux either;
   the first native test workflow covers Linux only (see `Decisions`) and was not written by the install lane.
- `.github/workflows/cargo-publish.yml` has the job group `gpc-*` for the new crate;
   it ran for the first publication (see `Publication state`).
- Branch protection of `main`:
   `required_status_checks` has empty `contexts` and `checks`,
   required signatures are on,
  and the only ruleset is "Copilot review" (`gh api repos/Aquaticat/Monochromatic/branches/main/protection` and
  `.../rulesets`).
   Deleting workflows therefore strands no required check.

### Sibling packages

- `package/git-executable/core` (the shared real-Git resolver,
   `@monochromatic-dev/git-executable-core`):
  done on the install branch (commit `1e5c35a886`).
   `isGitPolicySelfShim` in `src/self-shim.ts` returns "not a shim" for any ELF,
   PE or Mach-O header,
   and its markers match only the incumbent's package name and `dist` path,
   so the resolver would have selected the native `git` as "real Git" for its other consumers
  (`resolveRealGit` is called by `package/pi-plugin/auto-mode/src/git-worktree-read-allowlist.ts`).
   On this host the common-path rule masks it (`src/resolve-real-git.ts`
  promotes `/usr/bin/git` when PATH exposes it),
   but not on a host whose Git is elsewhere.
  Reading whole binaries is not an option (`doc/troubleshooting/starship-cli-git-command-timeout.md`).
  The function now also returns true for a candidate whose `realpath` basename is `cli-git-native` or
   `cli-git-native.exe`,
   which holds because the installed `git` is a symlink to that file;
   the script markers stay,
   so a stale incumbent shim is still rejected.
  Two tests in `src/native-executable.unit.test.ts` cover it (a link skipped for a later native Git,
   and a PATH holding
   only the link reporting `RealGitNotFoundError`);
   both fail with the check disabled (run on 2026-10-13).
  `README.md` documents it.
  The native side already rejects the incumbent's markers (`src/native/real_git_candidate.rs:91-94`,
  test `src/native/binary_resolution_tests.rs:206`);
   keep them while stale shims can exist.
- `package/git-policy/api`,
   `package/git-policy/forbidden-strings`,
   `package/git-policy/markdown-lint`,
  `package/git-policy/repository` (the four authoring and policy packages):
   delete after the config,
  the release task and the cli mirrors are gone (see `Decisions`).
  Their only consumers are `cli-git.config.ts`,
   the cli mirrors,
   `package/git-policy/cli/bin/dependent-version-*.mjs`
  (through `@monochromatic-dev/git-policy-repository/ts`),
   `changeset:version`,
  and their own READMEs (`package/git-policy/repository/README.md:14-15`,
  `package/git-policy/markdown-lint/README.md:70-74`).
- `package/forbidden-strings/cli/README.md:118`,
   `:788-789`,
   `:799`,
   `:809`:
   migrate text.
  They say root `cli-git.config.ts` gives the policy the scanner path and that "the policy invokes the
  repository-built scanner directly";
   the native wrapper links the scanner in-process and reads `rulesFile` from
  `cli-git.config.jsonc`.
   `package/forbidden-strings/cli/PERF.md:159,199-212` cite incumbent `perf/` files and a
  packed tarball path:
   historical,
   keep.
- In the package `package/oxlint-plugin/prefer-readonly-parameter-type`,
  the comment at line 392 of `src/prefer-readonly-parameter-types/effect-member-result-relation.ts` cites
  `package/git-policy/cli/src/policy-engine/manual-push-candidates.ts` in a comment:
  replace the citation with the explanation it supports,
   since the file will not exist.
- `package/ssg/aquati.cat/TODO.generalize-ssg.draft.md:286` cites `package/git-policy/cli/package.json:8` as a model
  for a `bin` entry:
   cite another package that keeps a `bin`.
- `package/pi-plugin/auto-mode/README.md:371`:
   no edit,
   it names `git-executable` only.
- `package/git-policy/cli.fuzz`:
   keep.
   It depends on the crate by path (`Cargo.toml:23`) and copies `Cargo.toml`,
  `Cargo.lock`,
   `README.md` and `src/native` (`bin/planted-controls.mjs:21-24`);
  `src/controls.rs:30` holds TypeScript pseudocode in a comment only.

### Documentation

- Live and wrong after the cutover:
   `package/git-policy/cli/SPEC.md` (4,305 lines).
  It describes plugins,
  executable configuration and trust,
  and mentions `cli-git.config.jsonc` nowhere (measured on 2026-10-12:
   zero matches).
  The package README was rewritten for the native wrapper in `4f5bc1d30c` and needs no further change for the
  cutover.
  SPEC sections that exist for the retired surface,
   by heading and line:
   `Public authoring declarations` (`:77`),
  `Authoring example` (`:628`),
   `Trust registry schema version 1` (`:1638`),
   `Trust consent protocol` (`:1796`),
  `Recursive enrollment and revocation` (`:1823`),
   `Relaxed-mode parser` (`:1895`),
  `TypeScript build contract` (`:1931`),
   `Package and user-boundary verification` (`:4268`) and
  `Contract fixture verification` (`:4299`).
   `Configuration validation` (`:718`) needs the JSONC schema.
  The native behavior ledger cites SPEC line numbers (`doc/planning/cli-git-rust-behavior-ledger.md`),
  so edit the SPEC in one commit and re-cite the ledger in the next,
   or cite by heading.
  The README must keep existing:
   `Cargo.toml` `readme` and `include`,
   `bin/test-native-container.mjs:62`,
  and `package/git-policy/cli.fuzz/bin/planted-controls.mjs:23` read it.
- `package/git-policy/cli/e2e/README.md:42-44`:
   describes `E2E_WRAPPER` defaulting to `incumbent` and the
  ELF-versus-script check.
- `doc/runbook/remove-retired-hk-git-config.md` is deleted (install branch),
   and `doc/troubleshooting/cli-git-hk-retirement-stale-hook-config.md` gives the manual removal
   (`git config --show-scope --show-origin --get-regexp '^hook\.hk-'` to list,
    `git config --local --unset-all <key>` or `--global` to remove) and keeps its root cause.
  Historical records that name the retired task or runbook carry an in-place note (`WR6`):
   `doc/handover/cli-git-policies-platform.md`,
   `doc/handover/cli-git-native-signals.md`.
- `doc/philosophy/agents.md:362` cites `DEFAULT_ALLOWED_WORKTREE_DIRS` in
  `package/git-policy/cli/src/allowed-worktree-dirs.ts`:
   repoint to `default_allowed_worktree_dirs` in
  `package/git-policy/cli/src/native/effective_target.rs:118`.
- `doc/handover/cli-git-rust-implementation.md` queue items at `:1556-1558`,
  `:1593` and the sentence "Current production tools remain active until cutover" (`:1599`),
  and `doc/handover/cli-git-native-integration.md` `Remaining`:
   update when the cutover lands;
   replace,
   do not append.
- Historical records that cite incumbent paths and stay as written:
   44 files in `doc/handover`,
   45 in `doc/planning`,
  62 in `doc/troubleshooting`,
   17 in `doc/audit`,
   8 in `doc/decision`,
   2 in `doc/research`,
   1 in `doc/artifact`.
  Apply the repository's moved-identifier rule (`WR6`) only where a document is still read as current;
  put the retired incumbent's last commit hash (see `Order of the retirement commits`) at the top of
  `doc/planning/cli-git-rust-behavior-ledger.md` so its `src/...ts:NN` citations stay resolvable through Git history.
- `package/git-policy/cli/doc/concurrent-commits-implementation-plan.md` (1,388 lines,
  about 135 TypeScript path citations):
   historical implementation plan;
   keep.
- `doc/runbook/forbidden-strings-out-of-band-rules.md:152` (`git cli-git check`) and
  `doc/todo/forbidden-strings.md:18`:
   still correct,
   no edit.

## Retirement list

### Delete inside `package/git-policy/cli`

- `src/` except `src/native/`:
   478 tracked files (188 in `policy-engine/`,
   112 in `trust/`,
   56 at the top level,
  25 in `worktree-copy/`,
   22 in `parser/`,
   21 in `optional/`,
   17 in `index-lock/`,
   10 in `rule/`,
  9 in `shadow-repository/`,
   6 in `hook-dispatch/`,
   6 in `api/`,
   4 in `owner-lock/`,
   2 in `maintenance/`),
  of which 110 are `*.unit.test.ts`.
   `src/native/` keeps its own `index_lock_restart_manager.ps1` and
  `dependent_version_fixtures/unit_cases.json`.
- `rolldown.node.config.ts`.
- `dist/` (ignored,
   on disk in the main checkout):
   remove after the verification passes,
   when the rollback window ends (see `Decisions`).
- `src/maintenance/` goes with `src/`;
   the hk cleanup is already retired (see `Decisions`).
- Tasks in `package/git-policy/cli/mise.toml`
  (list them with `rg --line-number "^\[tasks" package/git-policy/cli/mise.toml`):
   `run`,
   `build:js`,
   `build:js:node`,
   `watch:build`,
  `watch:build:js`,
   `watch:build:js:node`,
   `verify:supported-runtime`,
   `pack:npm`,
  `test:built:trust`,
  `test:e2e:concurrent:image` (the incumbent image),
   `test:e2e:concurrent` (the incumbent run) and
  `test:e2e:concurrent:native:surface` (compares incumbent with native).
  The task `build` is not in this list:
   it is the release build of `cli-git-native` and stays.
- Incumbent-bound verification bins (all need `src/index.ts`,
   `cli-git.config.ts`,
  the packed incumbent or the planner in `package/git-policy/repository`):
  `bin/differential-commit.mjs` (runs `src/index.ts`,
   line 388),
   `bin/differential-config-*.mjs` and
  `bin/differential-repository-config.mjs` (`bin/differential-config-fixture.mjs:116-120,248-268` copy
  `cli-git.config.ts`,
   run `src/index.ts`,
   and trust the config),
   the 21 `bin/dependent-version-*.mjs` files (import
  `@monochromatic-dev/git-policy-repository/ts`,
   `bin/dependent-version-incumbent.mjs:23`),
  `bin/interop-native-container.mjs` and `native-interop/*.mjs` (copy the main checkout's `dist` and `/opt/cli-git`
  into the image,
   `bin/interop-native-container.mjs:55-79,146,211-212`).
  The three differentials (commit,
   configuration,
   dependent-version) are deleted (see `Decisions`).
No surviving bin,
   e2e file or native test imports them (searched `bin`,
   `e2e`,
   `native-interop`,
   `mise.toml` and
`src/native`:
   only the three tasks `native:config:differential`,
   `native:differential:commit` and
`native:differential:dependent-version` run them).
The interop bin `bin/interop-native-container.mjs` and `native-interop/*.mjs` stay,
   but copy the main checkout's `dist`
and `/opt/cli-git` into the image (`bin/interop-native-container.mjs:55-79,146,211-212`),
   so they cannot run
unchanged until an incumbent-provisioning step from the pinned revision exists.

Note (2026-10-13): deleted at the user's request and readable at commit `5d61d7b442`:
The incumbent-provisioning step and the incumbent drivers.
The results are in `doc/handover/cli-git-native-interop.md`.

### Keep inside `package/git-policy/cli`

- The crate:
   `Cargo.toml`,
   `Cargo.lock`,
   `src/native/**` (including `dependent_version_fixtures/unit_cases.json`,
  which the native tests read and which stays as the recorded incumbent truth),
   `README.md`,
   `SPEC.md`,
   `LICENSES/`.
- The end-to-end runner `e2e/` (TypeScript run by Node inside the image) with `e2e/commit-shape-trace.json`.
  Edit `e2e/wrapper-selection-fixture.ts:16-26,60-80,167-219` and `e2e/wrapper-selection.unit.test.ts:26-57,68-87` so
  the only wrapper kind is native and an unset `E2E_WRAPPER` no longer selects the incumbent;
  delete `e2e/wrapper-surface-probe.ts` and `e2e/wrapper-surface-steps-fixture.ts` with the `surface` task;
  edit `e2e/concurrent-e2e-consumer.ts:12`.
   Eight unit-test files remain under `e2e/` and keep running under
  `test:unit`.
- The native bins:
   `bin/test-native-container.mjs`,
   `bin/mutate-native-container*.mjs`,
  `bin/native-planted-controls.mjs`,
   `bin/native-*-snapshot.mjs`,
   `bin/native-verification-process.mjs`,
  `bin/build-git-test-image.mjs`,
   `bin/e2e-native-container.mjs` (minus the `surface` subcommand and the base-image
  coupling).
- `perf/*.json` (the incumbent's measured baselines):
   the accepted performance bar is "faster than the incumbent per
  scenario" (`doc/handover/cli-git-rust-implementation.md`,
   `Adopted without a question`),
  and the baselines are the incumbent side of that comparison.
  The `perf/*.ts` harness ran a packed npm tarball and was deleted (see `Decisions`);
   the baselines have no runner in the repository.
- Tasks:
   every `native:*` task,
   `test:git-image`,
   `e2e:concurrent:trace`,
   `test:e2e:concurrent:native:image`,
  `test:e2e:concurrent:native`,
   `native:lock:scanner`,
   and the lint tasks named in
  `What the manifest and tasks must become`.
- The fuzz sidecar `package/git-policy/cli.fuzz` is untouched.

### What the manifest and tasks must become

- `package/git-policy/cli/package.json` stays,
   because `e2e/` and `bin/` still import workspace packages and the lint
  tasks need a manifest,
   but it stops being a published package:
   `private: true`;
  remove `main`,
   `types`,
   `exports`,
   `bin`,
   `files`,
   `engines`,
   `publishConfig`,
   `sideEffects`,
  and all runtime `dependencies` that only `src/` used (`ignore`,
   `rolldown`,
   `type-fest`,
   `typescript`,
   `yuku-ast`,
  `yuku-parser`);
   keep the name so `SGD` holds and the `.changeset/config.json` entry stays valid.
  Imports measured in the surviving `e2e/`,
   `bin/` and
   `native-interop/` (after the performance harness was deleted):
   `@monochromatic-dev/test-harness-core`,
  `@monochromatic-dev/caught-value-core`,
   `@monochromatic-dev/logger-core`,
  `valibot` (only `bin/differential-config-compare.mjs:11`),
   plus `@monochromatic-dev/config-typescript` for
  `tsconfig.json` and `@types/node`.
   `nano-spawn` is now imported only by `src/` (`rg nano-spawn` on 2026-10-13).
   Drop `nano-spawn` and `valibot` together with the files that use them.
  The `@monochromatic-dev/git-executable-core`,
   `...-git-policy-repository`,
   `...-config-rolldown`,
  `...-module-async-time`,
   `...-module-fs-id`,
   `...-module-fs-path` and `...-ownership-marker-foreign-borrowed`
  devDependencies go with `src/`.
   Removing `valibot`,
   `yuku-*` and `ignore` from this manifest also removes the
  incumbent's runtime closure (hazard 5).
- `tsconfig.json` (`package/git-policy/cli/tsconfig.json`):
   `include` loses `src/**/*.ts` and `*.config.ts`;
  `e2e/**/*.ts` remains (`perf/**/*.ts` is already gone from `include`).
  `package.json` stays in `include` only if JSON modules are imported.
- `lint` is a template fan-out to direct child tasks (`mise.no-env.toml:611-613` template `lint`,
   `vars.fanout`);
  it throws when no child exists.
   Keep `lint:oxlint`,
   `lint:types` and `lint:oxlint:paths` (`:99-131`),
  and add `lint:clippy` running `cargo clippy --locked --offline --all-targets -- -D warnings`,
  matching the sibling Rust CLI `package/forbidden-strings/cli/mise.toml` (`lint:clippy`).
  Today `lint` does not run Clippy at all:
   the crate is checked only by `native:clippy` and the container gate.
  Adding `lint:clippy` makes the root `mise run lint` compile the crate;
   its duration was not measured.
- `test:unit` (`:134`) keeps globbing `**/*.unit.test.ts` inside the package;
  after retirement it runs the eight `e2e/` unit tests.
   The authority for Rust remains `native:test:container`;
  there is no `test` task for the crate.
- `build` (done on the install branch):
   root `mise run build` runs every package's task named `build` and throws only when no package has one.
  `build` is the release build of `cli-git-native` (`cargo build --release --locked --offline --bin cli-git-native`,
   as the sibling `package/forbidden-strings/cli` does),
   so `prepareAndBuild` keeps producing an artifact;
   every root `mise run build` now compiles the crate (about 71 seconds in one run on this host on 2026-10-13,
   with a shared cargo `build-dir` from `doc/decision/cargo-build-dir-host-cache.md`,
   so not a clean-machine figure).
   The templates `build:js` and `build:js:node` still run `rolldown` and are deleted with `src/`.
- `test:e2e:concurrent:image` (`:349`) loses `depends = ["pack:npm"]`,
  the `TARBALL` build argument and the context `dist/pack`;
  `e2e/concurrent-commits.Containerfile:13,56-59` loses `ARG TARBALL`,
   `COPY ${TARBALL}` and `npm install`.
  The image keeps Node at the floor,
   Git 2.39.5,
   2.40.0 and 2.55.0,
   `ssh-keygen` and the `caught-value` link.
  Its `engines.node` comment (`:17`) must name the Node version itself,
   since `engines` is gone.
  The tag `localhost/cli-git-concurrent-e2e:latest` and the task names with `:native` can stay (renaming them touches
  the rerun commands of every handover);
   do not rename in the cutover commits.

### Places where deleting the TypeScript breaks something not obviously covered

1.  Root `mise run watch:build` and `lint` fan out by task name;
    the cli package's `watch:build*` and `build:js*` templates run `rolldown` and a fan-out
    (root `build` no longer reaches them:
     the package `build` is the cargo build).
    Removing `src/` and `rolldown.node.config.ts` without removing or replacing those tasks makes `mise run watch:build`
    fail in this package.
2.  `test:e2e:concurrent:image` and the native e2e image depend on `pack:npm`.
    The native image requires `localhost/cli-git-concurrent-e2e:latest` and refuses to build without it
    (`bin/e2e-native-container.mjs`,
     `buildImage`).
3.  `E2E_WRAPPER` unset means the incumbent (`e2e/wrapper-selection-fixture.ts:73-80`).
4.  File-enforcer regenerates `src/optional/**` unless its three mirror blocks go (`file-enforcer.config.ts:2276-2341`).
5.  `pnpm install` after the manifest edits prunes
    `package/git-policy/cli/node_modules/{valibot,yuku-parser,yuku-ast,ignore}`,
     which the installed `dist` needs:
    the incumbent stops working for every session still using the shim.
    The same `pnpm install` removes `node_modules/.bin/git` in that checkout once `package.json:36` is gone.
6.  `mise run //package/git-policy/cli:native:config:differential` needs `cli-git.config.ts` and the incumbent;
    `native:differential:commit` needs `src/index.ts`;
    `native:differential:dependent-version` needs `package/git-policy/repository`;
    `native:interop` needs the incumbent `dist` and the packed `/opt/cli-git`.
    All four are verification tools whose reference side is the incumbent
(see `Decisions`:
     the three differentials are deleted,
     `native:interop` stays and needs provisioning from the pinned
revision).
7.  The trailing gates in `doc/handover/cli-git-rust-implementation.md` (`Gates that trail the cutover`) include the
    old-and-new interop matrix,
     which needs a runnable incumbent after the retirement.
    Note (2026-10-13): deleted at the user's request and readable at commit `5d61d7b442`:
    The runnable incumbent and its provisioning step.
    The results are in `doc/handover/cli-git-native-interop.md`.
8.  `package/git-executable/core` treats native executables as real Git (see `Sibling packages`).
9.  `cli-git.config.ts` imports the `./ts` export that the retirement deletes,
    so after the deletion nothing can load it:
     the native wrapper never reads a `.ts` configuration,
    and the incumbent could not import the missing export.
     It must be deleted.
    Once it is gone,
     any session still on the incumbent runs only the core policies (hazard 6).
10. The hk cleanup command imports `@monochromatic-dev/git-executable-core/ts` and `@monochromatic-dev/logger-core/ts`
    (`src/maintenance/hk-config-cleanup-command.ts:14`,
     `hk-config-cleanup.ts:7`);
    its disposable-fixture test lives in `src/trust/fixture/hk-config-cleanup-consumer.ts` and is run by
    `test:hk-config-cleanup`.
     Both go with `src/`.
11. `final-newline.yml`,
     `npm-release.yml` (through `changeset:version`) and `cli-git-trust.yml` call deleted paths.
12. `package/git-policy/cli/Cargo.toml:9-10` ("the TypeScript executable remains active meanwhile") and `:17-18` ("at
    cutover") become false comments;
     reword and run `mise run sync:files`.

## Publication state

The crate `monochromatic-git-policy-cli` 0.1.1 (and 0.1.0) is on crates.io,
the GitHub release `monochromatic-git-policy-cli-v0.1.1` lists the `x86_64-unknown-linux-gnu` and
`aarch64-unknown-linux-gnu` archives,
and Trusted Publishing is configured for the crate (state reported to the install lane on 2026-10-13;
the install lane read the release only through `cargo-binstall`'s download described under `Isolated verification`).
Archives are named `monochromatic-git-policy-cli-<version>-<target>.tar.gz` and hold
`monochromatic-git-policy-cli-<version>-<target>/cli-git-native` and `README.md`.

A fix to the crate (as 0.1.1 was for the defect under `Why the pin is 0.1.1`) is a new version through the same path:
raise `version` in `package/git-policy/cli/Cargo.toml` (and the lock entries that follow it),
land the change on `main`,
and let `.github/workflows/cargo-publish.yml` run.
Its `gpc-detect` job notices the changed version on the push,
`gpc-publish-crate` publishes through Trusted Publishing,
and `gpc-build-binary` and `gpc-create-release` build the archives and the release
`monochromatic-git-policy-cli-v<version>`.
The group can also be dispatched:
`gh workflow run cargo-publish.yml --ref main -f crate=monochromatic-git-policy-cli -f dry-run=true`
(the dry-run input was read in the workflow,
 not run by the install lane).
The `forbidden-strings` dependency must already be on crates.io at the version `Cargo.toml` requires
(0.5.0 is);
`gpc-publish-crate` waits for it.
The container gate runs on the source,
 not on the release archive,
so the user-boundary verification of the installed file is the only proof for the installed bytes.

### Name

- The crate is `monochromatic-git-policy-cli`.
  Repository precedent was mixed:
   `monochromatic-jsonc-edit`,
   `monochromatic-lint` and `monochromatic-nested-wayland-session` carry the prefix,
   `forbidden-strings` and `forbidden-regex` do not (`doc/research/unified-linter-naming.md`,
   section `Repository facts`).
  A published name cannot be renamed,
   the bare name was generic,
   and the prefix matches the newest releases.
- The library name `git_policy_cli` keeps every `git_policy_cli::` path in `package/git-policy/cli.fuzz/src/*.rs`.
  The mise tool name is `cargo:monochromatic-git-policy-cli`.
- The binary is `cli-git-native`.
  The word "native" loses its contrast with the incumbent after retirement,
   but the installed `git` link hides it.

## Install the published tool

How the `cargo:` backend names things (mise 2026.10.0,
 observed with the installed `monochromatic-lint` and with this crate):

- The installed file is `<mise data>/installs/cargo-<crate>/<version>/bin/<bin name>`,
   so the executable is
  `cli-git-native`,
   not `git` and not the crate name.
  The `cargo:` backend has no option to rename it,
   and a `bin_path` option is ignored for this backend.
- Never make the crate's binary or a link in that `bin/` directory be named `git`.
  A probe with a `git` entry in an installed tool's `bin/` made `mise reshim` create `<mise data>/shims/git`;
  on this host that directory comes before `/usr/bin` in every shell,
   including outside the repository
  (positions 131 and 135 of the probing session's `PATH`).
- The wrapper behaves as the wrapper under any file name.
  It recognizes itself through `current_exe` (`src/native/real_git.rs:453`,
   `src/native/commit_command.rs:167`),
   and
  hook entries are symlinks to the canonical file,
   not to the `git` link (`src/native/hook_dispatch_entries.rs:80`).
  A relink of `git` mid-transaction is therefore harmless;
   removing the version directory it resolves to
  (`mise prune`,
   an uninstall,
   or reinstalling the same version) is not.
- `cli-git-native cli-git <command>` works without a `git` link
  (the release smoke step and the CI jobs of the install branch use it).

### What the repository has

On the install branch (commit `8f0d946a24`,
 pin raised to 0.1.1 in `35887bf67c`):

- `mise.no-env.toml`:
   the entry `"cargo:monochromatic-git-policy-cli"` with `version = "0.1.1"` (pinned,
   with the
  reason in a comment),
   `os = ["linux"]` (only Linux archives are released),
   and a tool-level `postinstall` with `when = "always"`.
  The postinstall creates `$MISE_CONFIG_ROOT/.cache/cli-git-bin`,
   writes the symlink `.git.$$` there to `$MISE_TOOL_INSTALL_PATH/bin/cli-git-native`,
   then `mv --no-target-directory` moves it over `git` (atomic relink).
  It uses coreutils only,
   because the postinstall's `PATH` is not guaranteed to hold a working `node`
   (a first attempt with an inline Node program failed in a throwaway data directory with "node is not a valid shim").
- `file-enforcer.config.ts`,
   `generateMiseToml`:
   `'.cache/cli-git-bin'` precedes `'node_modules/.bin'` in the `_.path` list,
   and the comment above `_.path` in the generated file says so.
- `mise.toml` (generated):
   the entry above,
   the `_.path` entry,
   and the other `_.path` lines unchanged.
- `mise.lock`:
   one block `[[tools."cargo:monochromatic-git-policy-cli"]]` with `version = "0.1.1"`,
   from `mise lock cargo:monochromatic-git-policy-cli` (a dry run first);
   the run rewrote no other entry.
  It printed the warning "preserving unavailable dependency sidecar ... .mise/locks/pipx-slopo/0.9.0",
   which concerns
  the `pipx:slopo` entry and not this tool.

The `_.path` list in the committed `mise.toml` is the list of a full install,
not the list this lane's `sync:files` run produced.
The generator globs `package/*/*/node_modules/.bin` from disk (the comment above `_.path` says the list moves with
install state).
The install worktree had a partial install (34 such directories;
 for example `package/git-policy/cli/node_modules/.bin` did not exist),
 and `mise run sync:files` there dropped 14 existing entries.
The main checkout's glob (48 entries) equals the committed list exactly (compared with `diff`).
So the committed `mise.toml` is the previous committed list plus the one new first entry,
 produced by running the generator and then restoring the list to the 48 entries of a full install;
 the diff of `mise.toml` against the previous commit is the new tool entry,
 its comment and the new first `_.path` entry.
Reproduce it on a checkout with a full `pnpm install`:
 `mise run sync:files` there must leave `mise.toml` unchanged;
 any other result is install-state drift to restore,
 not to commit.

Why per worktree and not one user-level directory:
 the pin and the link live in the same checkout,
 so a worktree on an older pin keeps its own version,
 and one worktree's `mise install` never repoints the link of another.
The cost is that `.cache/cli-git-bin` of the main checkout serves every session started from the main checkout,
 in whatever worktree they later enter,
 because the session's `PATH` is fixed at its start.
`os = ["linux"]` was probed:
 on this Linux host `os = ["macos"]` made `mise install` install nothing ("all tools are installed"),
 and `os = ["linux"]` made `mise install --dry-run` report "would install".

Each checkout needs `mise install` once after it merges the change,
 because the link directory is per worktree;
until then its `PATH` entry points at a missing directory and `git` resolves to `/usr/bin/git` without the wrapper.
That is the same failure mode the incumbent has when `node_modules/.bin/git` is missing.
The root config sets `auto_install = true` (`mise.no-env.toml:251`),
 and `mise run <task>` installs every missing configured tool (measured in a throwaway project:
 a task that names no tool
 installed the configured `cargo:` tool and ran its postinstall),
 so the first `mise run` in a worktree also creates the link.
`.github/workflows/npm-release.yml` sets `MISE_AUTO_INSTALL: 'false'`;
 other workflows that do the same do not install the entry;
 a workflow that leaves auto-install on installs it.
Rollback is pinning the previous version,
 running `mise install` in each worktree (the postinstall relinks),
 and
`mise run lock:mise`.

### Isolated verification

Run on 2026-10-13 against the real releases.
The install itself ran from the install worktree (0.1.0) and from a throwaway project (0.1.1,
 the pinned version).
The environment was a disposable `HOME`,
 `XDG_*` directories and
`MISE_DATA_DIR`,
 `MISE_CACHE_DIR`,
 `MISE_CONFIG_DIR` and `MISE_STATE_DIR` under a `mktemp --directory` in
`${HOME}/temp/agent`,
 and,
 from the worktree,
 `MISE_TRUSTED_CONFIG_PATHS` limited to the worktree and `MISE_CEILING_PATHS` at the
worktree's parent.
Facts the first attempts showed:

- Without `MISE_CEILING_PATHS`,
   every `mise` command in a worktree under `.claude/worktrees` of the main checkout failed
  with "failed to decrypt sops file":
   mise also loads the main checkout's `mise.toml` (an ancestor directory) and its `_.file` of the encrypted
   `.env.local.json`.
  A throwaway project outside the main checkout (the block under `Steps that remain`) is not affected.
- The install from the worktree failed with "tool ... requires configured install dependency 'rust@nightly',
   but its selected version is not installed"
   because the root config declares `rust` and `cargo-binstall`,
   which the `cargo:` backend needs first.
  A fresh data directory would install the nightly toolchain with its components and Android targets;
   the probe copied the live `installs/rust` links and `installs/cargo-binstall` into the disposable data directory
   instead.
  The throwaway project of `Steps that remain` declares `cargo-binstall` itself and no `rust`,
   and needs neither copy.

Results for 0.1.1,
 from the throwaway project:

- `mise install`:
   exit 0,
   two tools (`cargo-binstall` and the wrapper) in 16.1 seconds,
   17.1 seconds wall.
  `cargo-binstall` reported "The package monochromatic-git-policy-cli v0.1.1 (x86_64-unknown-linux-gnu) has been
  downloaded from github.com" and "Done in 8.15s":
   the prebuilt archive path,
   not a source build.
  The installed file is 9,300,536 bytes,
   dynamically linked against `libgcc_s.so.1` and `libc.so.6`,
   and needs
  `GLIBC_2.39` at most (host glibc 2.43).
- `.cache/cli-git-bin/git` is a symlink to
  `<data>/installs/cargo-monochromatic-git-policy-cli/0.1.1/bin/cli-git-native`
  (0.1.0 from the worktree:
   the same shape,
   and a second `mise install` from `package/git-policy/cli` reported "already installed" and left the link at the
   repository root,
   because `$MISE_CONFIG_ROOT` is the root that declares the tool).
- Throwaway repository (disposable `HOME`,
   `GIT_CONFIG_NOSYSTEM=1`,
   `commit.gpgsign false`,
   the repository's `cli-git.config.jsonc` committed in it,
   a committed `.gitignore` with `.cache/`,
   `.cache/forbidden-strings.rules.txt` copied into the source repository,
   `PATH` = link directory,
   the installed `monochromatic-lint` directory,
   `/usr/bin`,
   `/bin`):
  `command -v git` printed the link;
   `git --version` printed `git version 2.55.0` (the real Git,
   resolved through the wrapper);
   `git cli-git --help` printed the usage and exited 0;
   `git cli-git status` exited 0 and printed the "is retired" sentence on standard error;
   `git worktree add -b work <path>` through the wrapper,
   run in the source repository,
   exited 0 and printed
   "cli-git:
   copied 1 ignored filesystem entry from ... into 1 new worktree.";
   the new worktree had `.cache/forbidden-strings.rules.txt` (14,482 bytes,
   mode 600);
   `git add -- a.txt` exited 0;
   `git commit --message ... -- a.txt` exited 0 and `git log --oneline` listed the commit;
   `git status --short` printed nothing;
   `git add .` exited 1 with `add-explicit/bulk-add-rejected`;
   `git commit -a -m x` exited 1 with `commit-only/all-flag`;
   `git add -- doc/x.md` and `git commit --message 'docs: x' -- doc/x.md` exited 0 with nothing on standard error
   (no `engine-failure` event and no `commit-landed` event,
   so no blocked outcome);
   `git cli-git check --all` exited 0.
- Leftovers after those commands,
   in the throwaway's common git directory:
   no directory matching `cli-git-transactions/<uuid>`,
   no `landing.lock`,
   `reservation.lock`,
   `capture.lock` or `hook.lock` directory,
   no `index.lock`,
   no `*.lock` and no `*.pending` entry;
   `cli-git/shadow` and `cli-git-worktree-copy/v1` existed and were empty.
- Signing was disabled in the throwaway (a hermetic `HOME` has no key) and is not verified there.
- Auto-push was not exercised.
- `bin/final-newline-workflow.mjs` against this 0.1.1 executable:
   exit 0 (clean clone passed,
   staged control refused).

### Why the pin is 0.1.1

Update 2026-10-14: the user decided that nothing this repository publishes needs a pin
("You don't need to pin anything we control"), so `mise.no-env.toml` now says `latest` for the wrapper and the
linter and `mise.lock` records the resolved version (0.1.2).
The section below stays as the record of why 0.1.0 must never be installed.

Version 0.1.0 landed a Markdown commit and then exited 2:
 in the throwaway repository with the repository's configuration (which lists `markdown/autofix`),
 `git commit --message 'docs: x' -- doc/x.md` printed an `engine-failure` event
 (`"code":"policy-incomplete"`,
  `"trigger":"post-commit"`,
  message "cli-git could not find monochromatic-lint in any absolute directory on PATH ...")
 and a `commit-landed` event with `"outcome":"post-commit-blocked"`,
 so the automatic backup was blocked.
`monochromatic-lint` was on `PATH`;
 `strace` showed the pre-commit stage finding it and the post-commit stage making no lookup.
The cause was in `package/git-policy/cli/src/native/post_commit.rs`:
 the post-commit checks were built without the invocation's environment,
 so their Markdown linter had no `PATH` (commit `b9fab0f843` on `main` gives the stage the environment and the
 configured options).
Never pin below 0.1.1.
The container gate of 0.1.0 had no test for this:
 `src/native/binary_markdown_tests.rs` had no match for `post-commit`
 and `src/native/post_commit_tests.rs` none for `markdown`.

## Installation sequence with rollback

### Preconditions

1.  The pinned crate version is published with its release archives and passed the isolated verification of
    `Steps that remain`,
     step 2.
    Version 0.1.1 qualifies (2026-10-13);
     0.1.0 does not (see `Why the pin is 0.1.1`).
2.  The container gate passes on the head that is pinned and landed,
     with
    `GIT_POLICY_NATIVE_IMAGE_TAG=cutover` and every container command through
    `node "${HOME}/temp/agent/gate-slot.ts"`.
    On the install branch head `76a70b8fb9` (crate sources equal to `main` at `4e3eda3798`,
     which is 0.1.1) the gate
    with `GIT_POLICY_NATIVE_IMAGE_TAG=install` exited 0 on 2026-10-13:
     image `localhost/git-policy-native-test:install` id
    `b15ea7a8690b6438c88d3533f9a7c405768948b500c0c748c0a44f27937b23fe`,
    library 1657 passed,
     0 failed,
     6 ignored;
     `native_binary` 116 passed,
     0 failed,
     1 ignored;
    `native_candidates` 1 passed;
     the `cli-git-native` unit target 0 tests;
    the gate's Clippy (`-D warnings`,
     all targets) finished without a diagnostic.
    The host task `native:clippy` was run on the earlier head `d39c883c4d` (crate sources of `main` at `0f79ccf033`) and
    exited 0;
     it was not rerun after the merge.
    The installed bytes come from the release workflow's toolchain,
     not from that gate,
     so the user-boundary
    verification is the only proof for the installed file.
3.  The incumbent identity is recorded:
    main's `HEAD`,
     the `dist/final/node/index.mjs` SHA-256,
     and the id of `localhost/cli-git-concurrent-e2e:latest`
    (values on 2026-10-12 are in `How the wrapper is installed and run today`).
    The incumbent is not a single file (hazard 5);
    the rollback artifact before verification is the shim plus its install state,
     and afterwards the pinned revision.
4.  The Git on the host is 2.55.0 or newer (it is 2.55.0;
     `src/native/git_version.rs:26`).
5.  Every worktree that will commit during or after the switch has merged `main`,
     run `mise run sync:files` and run
    `mise install` (hazards 3 and 4).
    Stale worktrees are not waited for (see `Decisions`).

### Coordination hazards and detection

Detect first,
 in this order,
 all read-only;
 repeat the in-flight check immediately before the switch.

1.  In-flight incumbent transactions and locks.
    One `find` per class,
     from the repository's common git directory (here `/var/home/user/Monochromatic/.git`):
    ```sh
    # run from the common git directory
    find . -maxdepth 4 -type d -regextype posix-extended \
      -regex '\./(worktrees/[^/]+/)?cli-git-transactions/[0-9a-f-]{36}'
    find . -maxdepth 4 -type d -regextype posix-extended \
      \( -name landing.lock -o -name reservation.lock -o -name capture.lock -o -name hook.lock \)
    find . -maxdepth 4 -name 'index.lock'
    ls cli-git/shadow cli-git-worktree-copy/v1
    ```
    A published transaction directory (a UUID,
     not `*.pending`),
     any `*.lock` directory,
    a `shadow/<id>` repository or an `index.lock` means a commit is running or was killed (layout:
    `package/git-policy/cli/SPEC.md:2085-2135`).
    Names ending in `.pending` are abandoned staging directories kept for diagnosis.
    State on 2026-10-12 from this host:
     one published transaction,
     a live `capture.lock` in
    `worktrees/cli-git-lane-rules-cache/cli-git-captures/` that was gone on the next probe,
    three `cli-git/shadow` entries,
     two stale `cli-git-transactions/*.pending` (Oct 3 and Oct 4),
    four stale `cli-git-worktree-copy/v1/settlement.lock.*.pending` (Oct 6),
    39 `cli-git-add-policy-*` directories (Jul to Oct),
     and no `cli-git-transaction` legacy directory.
    The probe is racy by nature:
     another session can start a commit right after it.
2.  Running wrapper processes:
     `pgrep --list-full --full '^node .*git-policy[-/]cli/dist/final/node/index\.mjs'` lists
    live incumbent Git commands.
     The character class matters:
     the main checkout's shim reaches the bundle through
    `node_modules/@monochromatic-dev/git-policy-cli/dist/...` (hyphen),
    while a package directory path has `git-policy/cli` (slash).
    The `^node` anchor keeps the shell that runs `pgrep` from matching itself.
    Positive control on 2026-10-12:
     a scratch `node` process started on a `git-policy-cli/dist/final/node/index.mjs`
    path was listed,
     and so was a live incumbent `commit` of this session through the main checkout shim.
    There is no native counterpart:
     for commands that do not wait on policy,
    the native wrapper replaces itself with `/usr/bin/git` by `exec`,
     so native processes look like plain Git.
    The published `cli-git-transactions` directories and `*.lock` files in the common git directory are the
    authoritative signal for in-flight native work.
3.  Stale and active worktrees.
     `git worktree list --porcelain` shows 89 worktrees (23 under
    `/var/mnt/encrypted/temp/agent/tr-fan`,
     18 under `/var/home/user/worktrees`,
     18 under `/var/home/user/temp/agent`,
    17 under `/var/home/user/Monochromatic/.claude/worktrees`,
     and others).
    A scratch script (not committed) over that list measured,
     at HEAD of each:
     72 have `cli-git.config.ts`,
    5 have `cli-git.config.jsonc`,
     7 have `.cache/forbidden-strings.rules.txt`.
    Active means a process whose `/proc/<pid>/cwd` is inside the worktree or a reflog or index modified recently;
    read those two signals,
     do not guess.
     Under native,
     in a worktree whose HEAD has only the TypeScript config,
    a config-loading command such as `git add` ends with exit 2 and an `engine-failure` event of code `config-invalid`
    (observed).
     Its message starts with "Legacy configuration",
     then the worktree's `cli-git.config.ts` path,
    then "is not executed or read by the native cli-git.
    Translate its settings into",
     then the worktree's `cli-git.config.jsonc` path
    (`src/native/config_file.rs:152-170,303-351`);
     read-only `git status` still works.
    In a worktree with the JSONC file but no `.cache/forbidden-strings.rules.txt`,
    `git add` ends with exit 2 and an `engine-failure` event of code `policy-incomplete` (observed).
    Its message starts "cli-git could not load the forbidden-strings rules,
     so no file was scanned:
     read rules",
    then the worktree's `.cache/forbidden-strings.rules.txt` path,
     then ":
    No such file or directory" (`doc/handover/cli-git-native-config.md`,
     `Rules file in a linked worktree`).
    Under the incumbent a missing `.cache` file did not stop commits in a worktree because its variable pointed at the
    main checkout's file;
     that is new fail-closed behavior.
    Remedy per worktree:
     merge `main`,
     then `mise run sync:files`,
     then `mise install` (the link directory of the native `git` is created
there).
4.  Sessions with the old PATH.
     PATH is inherited at start,
     so after the switch,
    sessions that started before the switch keep the TypeScript shim until restarted.
    Read,
     per process of the same user,
     only the PATH value from `/proc/<pid>/environ` (never print other variables;
    one environment contains tokens) and count those whose PATH names `node_modules/.bin`
    before the worktree's `.cache/cli-git-bin` directory.
    Both wrappers share `.git/cli-git*` state (adopted:
     locks and journals interoperate both ways),
    so a mixed fleet is the designed state,
     but only the incumbent-writes,
    native-reads direction has a recorded test (`doc/handover/cli-git-native-transactions.md:629-631`,
     trailing gate).
5.  The incumbent's runtime closure (see `The installed incumbent`) lives in install state:
    do not run `pnpm install` in the main checkout with the cli manifest reduced
    until the verification has passed and the hazard 4 count of sessions
    that still start the shim is zero.
6.  The incumbent without `cli-git.config.ts` runs only core policies:
    optional policies "remain disabled until trusted config registers them"
    (the former `package/git-policy/cli/README.md:210-212`,
     replaced in `4f5bc1d30c`),
     so rolling back after deleting the file silently drops
    `security/forbidden-strings`,
     `markdown/autofix`,
     `mono/forbidden-root-context` and `mono/dependent-version-bump`
    until the file is restored from Git history and trusted again.
    The same loss hits every session that still has the old PATH (hazard 4) the moment the file is deleted in the
    checkout it uses.
7.  The generated hook entries are symlinks from each transaction's `hooks/` directory to the wrapper executable
    (`doc/handover/cli-git-native-hooks.md`,
     `hook_dispatch_entries.rs`),
    so the installed file must keep a stable path for the length of any transaction.
The executable that hook entries target is the canonical file under the mise `installs` directory,
     because
`current_exe` (`src/native/real_git.rs:453`,
     `src/native/commit_command.rs:167`) resolves the `git` link.
Relinking `git` during a transaction is harmless;
     `mise prune`,
     an uninstall,
     or reinstalling the same version over
that file during a transaction is not.

### Steps that remain

All commits go through the installed wrapper with explicit pathspecs (`CLG`,
 `CPN`);
commit each step before the next (`GCE`).
The install branch already holds the tool entry,
 the `_.path` entry,
 the lock entry,
 the hk and performance
retirements,
 the `build` task and the CI jobs (see `Decisions` and `Install the published tool`).

1.  Remove the dead duplicate-code dismissals of the deleted `perf/*.ts` files from `slopo.ignore.txt`
    (lines 45,
     50,
     69,
     200 and 373 on 2026-10-13,
     hashes `fd8e709c61cb`,
     `ca593422f3b5`,
     `5e87bcf79430`,
    `9dec8705a47d` and `24a1be12cae0`).
    The install lane did not do it:
     the file is an inventory keyed to a `slopo analyze` report of 2026-10-07 with per-section live and dead counts
     (header and section titles),
     and its own convention keeps or annotates dead keys instead of deleting them,
     so the edit needs a fresh report to recount and a decision on annotate-or-remove.
    It does not block the switch.
    The other documentation edits of `Consumer inventory` change what is true only after the switch and belong to
    the retirement commits.
    Keep `cli-git.config.ts`,
     the root dependency,
     the cli manifest and `src/` untouched.
2.  Verify the pinned published artifact in isolation,
     with a throwaway data directory and project,
     before the switch.
    This ran end to end for 0.1.1 on 2026-10-13 (see `Isolated verification`);
     repeat it whenever the pin changes.
    `PROJECT` holds a `mise.toml` with `cargo-binstall = "latest"` (the `cargo:` backend installs through it) and the
    tool entry copied from `mise.no-env.toml`,
     made with:

    ```sh
    # run from the repository top level
    printf '[tools]\ncargo-binstall = "latest"\n%s\n' \
      "$(rg '^"cargo:monochromatic-git-policy-cli"' mise.no-env.toml)" > "$PROJECT"
    ```

    Then:

    ```sh
    # run by hand in a throwaway project; nothing production is touched
    V=$(mktemp --directory "${HOME}/temp/agent/cutover-isolated.XXXXXX")
    mkdir --parents "$V/proj" "$V/state" "$V/cfg" "$V/home"
    cp "$PROJECT" "$V/proj/mise.toml"
    cd -- "$V/proj"
    export HOME="$V/home" XDG_CONFIG_HOME="$V/cfg" XDG_DATA_HOME="$V/data" \
      XDG_CACHE_HOME="$V/cache" XDG_STATE_HOME="$V/state" \
      MISE_DATA_DIR="$V/data" MISE_STATE_DIR="$V/state" MISE_CONFIG_DIR="$V/cfg" \
      MISE_GLOBAL_CONFIG_FILE="$V/cfg/none.toml" MISE_CACHE_DIR="$V/cache" MISE_YES=1 MISE_TRUSTED_CONFIG_PATHS="$V"
    mise trust "$V/proj/mise.toml"
    mise install
    ls -la "$V/proj/.cache/cli-git-bin"
    ```

    Expected:
     `mise install` takes the prebuilt archive (about ten seconds;
     a compile that takes a minute or more means the release archives were not found,
     see `doc/troubleshooting/mise-cargo-binstall-falls-back-to-compiling.md`);
    `.cache/cli-git-bin/git` is a symlink to
    `$V/data/installs/cargo-monochromatic-git-policy-cli/<version>/bin/cli-git-native`.
    Then run items 1 and 2 of `User-boundary verification` with
    `BIN="$V/proj/.cache/cli-git-bin"` (skip its `ROOT` line) and `MAIN` set to the main checkout,
     and delete `$V`.
    A run from inside a worktree of this repository instead of a separate project also needs `MISE_CEILING_PATHS` set to
    the worktree's parent (see `Isolated verification`).
3.  Land the install branch on `main`,
     made through the incumbent (the branch holds the tool entry,
     the first `_.path`
    entry,
     the regenerated `mise.toml` and `mise.lock`,
     and the other edits).
    Check that `mise env --json` prints `<root>/.cache/cli-git-bin` before any `node_modules/.bin` entry
    (it did,
     in the install worktree on 2026-10-13,
     after an inherited `/home/user/.elan/bin` entry).
4.  In the main checkout and in each active worktree:
     merge `main` (the main checkout pulls it),
     run `mise run sync:files` (and restore a partial-install `_.path` drift,
     see `What the repository has`),
     run `mise install`,
     then check
    `command -v git` (identity item of `User-boundary verification`).
    Sessions started before this step keep the shim until restarted (hazard 4),
     and a session started from the main checkout takes its link from the main checkout,
     so `mise install` there is what switches such sessions;
    the shim and `dist` stay until step 7.
5.  Repeat the hazard 1 detection immediately before the first native commit.
    The switch is atomic per process start,
     so no quiet period is required by design
    (`doc/planning/cli-git-rust-open-decisions.md`,
     `Lock and journal interoperability across versions`,
     option A),
    but the reverse direction is not tested,
     so prefer a moment with no transaction directories.
6.  Run `User-boundary verification` through the real `PATH`.
7.  When it passes,
     the rollback window is over (see `Decisions`).
    Delete `cli-git.config.ts` and reword `cli-git.config.jsonc:2`,
     only when the hazard 4 count of sessions on the
    old `PATH` is zero (hazard 6 explains what those sessions lose otherwise).
    Commit through the native wrapper.
8.  Record the installation in `doc/handover/cli-git-rust-implementation.md` and
    `doc/handover/cli-git-native-integration.md` by replacing the superseded text;
    that commit is also the doc-only verification commit.
9.  Start the retirement (`Order of the retirement commits`) at once;
    no waiting period follows the verification.

### User-boundary verification

Both checks use the installed `git` on the resolved PATH,
 not an explicit executable path.

1.  Identity.
     `command -v git` prints `<root>/.cache/cli-git-bin/git`,
     where `<root>` is the checkout whose `mise.toml` set the session's `PATH`
     (a session started from the main checkout uses the main checkout's link
     in every worktree it enters;
     `mise exec -- sh -c 'command -v git'` shows a worktree's own environment);
    `readlink --canonicalize "$(command -v git)"` ends in `cli-git-native`;
    `git --version` prints `git version 2.55.0`;
    `git cli-git status` prints the "is retired" sentence,
     where the incumbent prints a JSON `trust-status` line.
2.  A real commit in a throwaway repository (sequence run end to end against a native debug build on 2026-10-12,
    and,
     with the additions listed under `Isolated verification`,
     against the installed 0.1.0 executable on
    2026-10-13,
     with these results).
     Everything runs in a subshell with a disposable `HOME` and no system Git configuration,
    so no user configuration is read;
     `/usr/bin/git` is used only for fixture setup,
    and the wrapper is found through `PATH`:

    ```sh
    # run by hand; MAIN is the main checkout; the wrapper policies require a linked worktree and explicit paths
    ROOT=$(git rev-parse --show-toplevel)
    BIN="${ROOT}/.cache/cli-git-bin"
    V=$(mktemp --directory "${HOME}/temp/agent/cutover-verify.XXXXXX")
    mkdir --parents "$V/home"
    (
      export HOME="$V/home" GIT_CONFIG_NOSYSTEM=1
      export PATH="$BIN:${MAIN}/node_modules/.bin:/usr/bin:/bin"
      command -v git
      /usr/bin/git init --initial-branch=main "$V/repo"
      /usr/bin/git -C "$V/repo" config user.name Verify
      /usr/bin/git -C "$V/repo" config user.email verify@example.invalid
      /usr/bin/git -C "$V/repo" config commit.gpgsign false
      /usr/bin/git -C "$V/repo" commit --allow-empty --message seed
      /usr/bin/git -C "$V/repo" worktree add -b work "$V/wt"
      cd -- "$V/wt"
      echo hello > a.txt
      git add -- a.txt
      git commit --message 'verify native wrapper' -- a.txt
      git log --oneline
      find "$V/repo/.git" -name 'cli-git-transactions*' -o -name '*.lock'
    )
    /usr/bin/git -C "$V/repo" worktree remove --force "$V/wt"
    rm --recursive --force "$V"
    ```

    Observed:
     `git add -- a.txt` exits 0,
     `git commit ... -- a.txt` exits 0 and `git log --oneline` shows the new
    commit,
     `git add .` is refused with `add-explicit/bulk-add-rejected`,
    and `git commit -a -m x` with `commit-only/all-flag` (both exit 1 with one JSONL finding on standard error;
    run them inside the subshell before the cleanup to see them).
    The final `find` must print nothing after the commit:
     no published transaction directory and no lock (the 2026-10-13 run found none,
    with the four searches of hazard 1).
    Also commit a Markdown file there (copy the repository's `cli-git.config.jsonc` and
    `.cache/forbidden-strings.rules.txt` into the throwaway first,
     with `monochromatic-lint` on `PATH`):
     it must exit 0 with nothing on standard error (version 0.1.0 exits 2,
     see `Why the pin is 0.1.1`;
     0.1.1 exits 0,
     2026-10-13).
     The throwaway disables signing on purpose,
    because a hermetic `HOME` has no signing key;
     signing is checked by item 3 instead.

3.  A doc-only commit in this repository,
     from a linked worktree that has merged `main`
     and has run `mise run sync:files` and `mise install`
     (so `cli-git.config.jsonc`,
     `.cache/forbidden-strings.rules.txt` and `monochromatic-lint` on
    PATH all exist):
     edit one Markdown file under `doc/`,
     `git add -- <that file>`,
    `git commit -m '<type>(<scope>): <subject>' -- <that file>`.
    Expect exit 0 and,
     if the Markdown autofix policy rewrites the file,
     the wrapper's `fix` outcome;
    then confirm `git status --short` is clean,
     `git log -1` shows the commit,
    and the common git directory has no new transaction directory or lock.
    Then check the signature:
     `git log -1 --format='%h %G? %GK'` must print `G` and the same key id as
    `git log -1 --format='%GK' main`,
     because the branch protection on `main` requires signatures and the native
    wrapper must leave signing to Git,
     which the throwaway check cannot show.
    A failure here with `config-invalid` is hazard 3,
     not a defect;
     `policy-incomplete` with trigger `pre-forward` and a missing rules file is hazard 3 too;
     `policy-incomplete` with trigger `post-commit` and the message about `monochromatic-lint` is the 0.1.0 defect
     and means a version below 0.1.1 is installed.
4.  Auto-push:
     with auto-push enabled (`APG`) the commit pushes the branch;
    a push failure is a result to read,
     not noise.

### Rollback

Before the verification passes (step 6 of `Steps that remain`),
 the incumbent is still installed and rollback is a
change of `PATH`:

1.  Make sure no native transaction is in flight (hazard 1 detection returns nothing),
    because the incumbent reading native-written records is not tested.
    Let running native commits finish,
    or run any native read-only Git command in the affected repository,
    which runs recovery for interrupted transactions before it forwards.
2.  Remove or rename `.cache/cli-git-bin/git` in each affected worktree.
    Every process whose `PATH` contains that directory then falls through to the next `git`.
    For a session started from the main checkout that is the main checkout's `node_modules/.bin/git`,
    the incumbent,
     immediately and without a restart.
    A shell activated inside a worktree falls through to that worktree's own shim,
    which already fails today for lack of a `dist` (see `PATH mechanics on this host`),
    so it loses nothing it had.
    Removing the main checkout's link rolls back every session started from the main checkout at once.
     Then revert the switch commit and run `mise run sync:files`,
     so new sessions stop adding the directory and the
    tool entry is gone.
3.  Verify with `git cli-git status` (JSON `trust-status`,
     `"trusted":true`) and one commit in a throwaway
    repository with the incumbent.

After the verification passes the window is over,
 by decision.
`cli-git.config.ts` is deleted,
 and the shim and `dist` go in the retirement commits.
Rollback then has two shapes:

- To an earlier native version:
  pin the previous published version in `mise.no-env.toml`,
   run `mise run lock:mise`,
   and run `mise install` in each
  worktree (the postinstall relinks `git` atomically).
  The first published version has no predecessor,
   so a fix after it is a new version through the same publication
  steps;
   the registry's yank operation was not examined.
- To the incumbent:
  create a worktree at the pinned incumbent revision,
   install its dependencies with the package manager,
  build `dist` with `mise run //package/git-policy/cli:build:js:node`,
  put that worktree's `node_modules/.bin` first on `PATH`,
  restore `cli-git.config.ts` from Git history,
   and run `git cli-git trust --yes` for it with the user's approval
  (precedent:
   `doc/handover/unified-linter-cutover.md`,
   `Landing choreography`).
  That path was not exercised here and its duration was not measured.

### Order of the retirement commits

Start right after the verification of step 7.
Each step is its own commit,
 made through the native wrapper:

1.  Record the pinned incumbent revision:
    the last commit that contains the TypeScript,
    in the commit body of step 2,
    in `doc/planning/cli-git-rust-behavior-ledger.md` and in the handover queue.
2.  `changeset:version` already runs the native `fix` (install branch);
    prove it on the release workflow's input (the first run of the `version` job of `npm-release.yml` after the
    switch,
     compared with the planner's output for the same input) before deleting the planner;
    delete the three differentials (`bin/differential-*.mjs`,
     the 21 `bin/dependent-version-*.mjs` files and their
    tasks),
     because those bins import the planner and the TypeScript source;
    then delete `package/git-policy/repository` and the other three authoring packages
    with `file-enforcer.config.ts:2276-2341`,
    the file-enforcer test,
    and `src/optional/`.
3.  Delete `src/` (except `src/native/`),
    `rolldown.node.config.ts`,
    the tasks and bins listed under `Retirement list`,
    reduce `package.json` and `tsconfig.json`,
    edit `e2e/` and the Containerfile,
    delete `cli-git-trust.yml`,
    `src/maintenance/` (with `src/`),
    rewrite the `SPEC.md` sections of `Documentation`.
4.  Remove `package.json:36` and regenerate `pnpm-lock.yaml`,
    `package/config/pnpr/config.yaml` and `mise.toml`
    (`pnpm install --lockfile-only` or `mise run prepare:pnpm:install`,
     then `mise run sync:files`),
    inspecting each diff.
    Only now does `pnpm install` remove `node_modules/.bin/git` and the incumbent's closure.
5.  Remove the main checkout's ignored leftovers (`package/git-policy/cli/dist`,
     `dist/pack`,
     `dist/perf`):
    list them first with `git clean --dry-run -d -X` (`GCL`,
     `GC2`) and check stray git-dir entries with `ls -d` and
    `git check-ignore --verbose` (`GCR`).
    Remove them by explicit path,
     never with `git clean -X`:
     its listing includes `.cache/cli-git-bin`,
     the link every session depends on,
     and `.cache/forbidden-strings.rules.txt`.
     The rollback-window decision covers deleting them;
    the hazard 4 count of sessions still on the shim must be zero first.
6.  After the trailing gates are recorded (interop matrix,
    long fuzz,
    non-core mutation,
    `pathBytes`,
    Windows,
    release performance) and verified complete,
    the queue item "retirement of the old TypeScript implementation is verified complete" in
    `doc/handover/cli-git-rust-implementation.md` can be closed.

## Not established

- Whether the `version` job of `npm-release.yml` works on GitHub:
  it was checked with `actionlint` 1.7.12 and its commands were run locally against the installed 0.1.0 executable,
   but it has never run on a hosted runner, because every push so far found no pending changeset and skipped it
  (runs 38676475783, 38689708686); a run needs a pending changeset and opens or updates the Version Packages pull
  request.
  Measured on 2026-10-15 from the runner image notes (`images/ubuntu/Ubuntu2404-Readme.md`, image 20261012.334.1):
  hosted Ubuntu 24.04 has Git 2.55.0 (exactly the floor), Git LFS 3.8.0 and glibc 2.39, so the executable's needs are
  met there.
  `final-newline.yml` was deleted on 2026-10-15.
- Hosts older than glibc 2.39:
   the release archive's x86_64 file needs `GLIBC_2.39` (`pidfd_spawnp` and `pidfd_getpid`, measured on the 0.1.4
  archive with `objdump -T`); no older host was tried, and the README says the archives do not start there.
  The `ubuntu-latest` label moves to Ubuntu 26.04 in November 2026, so the wrapper's x86_64 build job names
  `ubuntu-24.04` to keep the floor.
- Whether `security/forbidden-strings` reports a finding at the post-commit trigger with 0.1.1:
   the throwaway commits had the rules file present and exited 0,
   and no commit with a forbidden string was made.
- Whether `mise run sync:files` leaves the committed `mise.toml` unchanged on a checkout with a full `pnpm install`:
  inferred from the main checkout's directory listing (48 entries,
   equal to the committed list),
   not run there.
- The prebuilt archive path of `mise install` was exercised for 0.1.0 and 0.1.1 on x86_64 Linux only;
  the aarch64 archive and any macOS or Windows host were not.
- Whether the unanchored `include` entries of `package/linter/monochromatic-lint/Cargo.toml` package a stray
  `README.md` from a subdirectory (the new crate's list is anchored;
   that package was not inspected).
- Whether the worktree setup flow of this repository runs `mise install`.
  The link directory is created by `mise install` (`postinstall` with `when = "always"`) and,
   with the root
  `auto_install = true`,
   by the first `mise run` of a worktree;
  `mise exec`,
   `mise env` and shell activation were not shown to run it.
- Which of the 89 worktrees are active,
   and which sessions hold the old `PATH`:
  the commands are given,
   not run against processes.
- Whether `pnpm install` prunes the cli package's links the moment its manifest drops them:
  inferred from the symlink layout,
   not run.
- The reverse interoperability direction (incumbent reads records native wrote):
   untested by decision (trailing gate).
- Whether the native wrapper tolerates the four Oct 6 `settlement.lock.*.pending` directories in
  `.git/cli-git-worktree-copy/v1`;
  the transaction staging names are ignored by design (`package/git-policy/cli/SPEC.md:2085-2095`).
- The contents of `*.local.*` files,
   `~/.local/state/cli-git/trust/v1`,
   and whether any of them name the shim or `dist`.
- Windows and macOS behavior of anything native,
   which is why the release matrix of the new crate is Linux only.
- The duration of the Clippy addition to `lint` (not yet added;
   see `What the manifest and tasks must become`).
- Baseline results of `mise run //package/git-policy/cli:lint` and `lint:types` before and after the manifest
  reduction (not run:
   the package is large).
- The exact `package.json` field list after reduction is derived from import counts,
   not from a type check of the
  reduced package.
