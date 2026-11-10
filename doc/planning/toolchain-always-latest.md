# Toolchain always latest

Status as of 2026-11-07T08:33:17-05:00:
the plan,
both locks and the container builds follow the newest versions;
the daily update is built,
rehearsed in a throwaway worktree,
and since 2026-11-07T08:29:48-05:00 enabled as a timer;
its first real run ended at 2026-11-07T08:33:17-05:00 with result `success` (section "First run");
since 2026-11-07 the update also checks every package before and after it and opens a fix-forward issue for each package
it newly breaks (section "Checking every package after the update").

## Owner's words

Recorded 2026-11-07,
verbatim,
in the order given:

- "Rust nightly should always be latest."
- "Move anyway,
  fix forward" (when a new nightly breaks a package).
- "Make the plan file always track latest and set up a systemd cronjob to update EVERYTHING (not just rust) daily."
- "Wait for Dylint" (the closure lint plugin `package/linter/monochromatic-lint-closure` stays as it is).
- "Everything also covers the pnpm and cargo dep locks,
  and the global toolchains too."
- Standing rules the update keeps:
  "nothing should ever wait forever or indefinitely";
  "never discard user data";
  "Commits should never fail.";
  the wrapper never starts itself at system startup or login (systemd units are fine);
  "Ignore Win and mac until I say so.";
  and the ban on synthetic host load.

This note follows
[`doc/planning/rust-nightly-always-latest.md`](rust-nightly-always-latest.md),
which moved the Rust row to the newest nightly and proposed how to keep it there.
The owner's answers replace its open question:
move anyway,
fix forward,
and update everything,
daily,
on this machine.

## Former pins

Requests in `toolchain.plan.jsonc` (repository) and the machine-global `toolchain.plan.jsonc`
under `~/.config/monochromatic-dev/toolchain` that were not the latest form on 2026-11-07,
with the reason written beside each,
what the request is now,
and whether the reason suggests that latest breaks something.
`rust@nightly` was already the latest form (a floating channel) in both plans.

### Repository plan

- `bin`:
  `github:marcosnils/bin@0.29.4`.
  No reason beside it (the global plan calls it "the pinned `bin` CLI itself",
  the tool-managed bootstrap).
  Now `@latest`;
  the newest release is still 0.29.4,
  so nothing moved.
  Nothing suggests a break.
- `mpm`:
  `pipx:meta-package-manager@8.2.0`.
  No reason beside it ("the pinned `mpm` prerequisite itself").
  Now `@latest`;
  still 8.2.0.
  Nothing suggests a break.
- `bun`:
  `github:oven-sh/bun@bun-v1.3.14`.
  Reason:
  "Pinned to 1.3.14 because the Rust rewrite's robustness invites
  serious doubts."
Now `@latest`;
  the lock moved 1.3.14 to 1.4.4.
  **Fix-forward candidate**:
  the reason is a doubt about the newer line,
  and nothing here has run on 1.4.4 yet.
- `cargo:cargo-fuzz`:
  `cargo:cargo-fuzz@0.13.2`.
  Reason:
  "the --fuzzing-workspace and libfuzzer-sys integration shifted
  across point releases;
  this is the verified rollout version."
Now `@latest`;
  the newest release is still 0.13.2.
  **Fix-forward candidate**:
  the next point release may break the fuzz crates,
  which is what the reason says happened before.
- `zig`:
  `zig@0.15.2`.
  Reason:
  "to match the terminal Containerfile's ZIG_VERSION
  (`doc/decision/desktop-app-podman-build.md`);
  bump both together."
Now `@latest`;
  the lock moved 0.15.2 to 0.17.1.
  **Fixed forward** on 2026-11-07:
  the Containerfiles of `package/terminal/app` (two) and `package/music-player/desktop-app` no longer carry a Zig version,
  they take the archive and SHA-256 of the lock's row (section "Containers follow the lock"),
  and the vendored Ghostty that needs Zig 0.15 moved to a `libghostty-rs` revision that builds with 0.17.1
  (`doc/troubleshooting/ghostty-requirezig-rejects-zig-0-17.md`).
- `llama.cpp`:
  `github:ggml-org/llama.cpp@b9829`.
  Reason:
  "llama.cpp tags before the Linux build finishes uploading,
  so a floating install can miss the ubuntu-x64 asset entirely."
  **Still pinned:
  no latest form works.**
GitHub marks every `b`-numbered release a prerelease,
  which the `github:`
  resolver skips,
  so `@latest` resolves the unrelated `v0.9.0` release (one asset,
  no Linux build) and fails the whole
  `update` (probed 2026-11-07;
  a failed lookup stops the lock from being written at all).
  The comment beside the pin now says so.
  Fix-forward:
  the resolver needs an option that takes prereleases.
- `java`:
  `java:temurin@21`.
  Reason:
  "Temurin 21:
  AGP 9 / Gradle 9 need a JDK in the 17 to 21 range."
  **Unchanged:
  the resolver takes a major line only** (`java:temurin@latest` is refused),
  and the newest release of
  major 21 is already what the lock holds.
  A newer major would break the Android build by the reason's own account.

### Machine-global plan

- `bin` and `mpm`:
  pinned like the repository plan,
  now `@latest`,
  nothing moved.
- `bun`:
  `bun-v1.3.14` for the same reason,
  now `@latest`,
  moved to 1.4.4.
  Same fix-forward candidate.
- `npm:@deepseek-ai/dsh`:
  `@next`,
  no reason beside it.
  Now `@latest`;
  both dist-tags were 0.2.2-rc.1 on 2026-11-07,
  so the lock moved 0.2.1-rc.5 to 0.2.2-rc.1 through either.
- `cargo:below`:
  `git:https://github.com/facebookincubator/below@6d62fc1f705385dfa71474e7c984b34c868ad089`.
  Reason:
  "Pin the release commit because this release has no prebuilt assets.
  Rebuild on clang 23."
  **Unchanged:
  a `git:` request takes a revision only**,
  so no latest form exists.
  A request that follows a branch needs the resolver to learn it.
- `cargo:https://github.com/slint-ui/slint` (slint-lsp):
  a pinned commit,
  no reason beside it.
  Unchanged for the same reason.
- `llama.cpp-cpu`:
  `@b9829`.
  Reason:
  b9829 was checked to load the community Xiaomi-OCR-0 GGUF files.
  Unchanged for the same prerelease reason as the repository pin.
  **Fix-forward candidate** twice over.

## What the update moved

Run in a throwaway worktree on 2026-11-07,
before the timer existed,
through the repository tasks.

- Repository lock (`mise run toolchain:update`,
  16 tools moved):
  bun 1.3.14 to 1.4.4,
  pnpm 12.16.1 to 12.18.0,
  fd-find 10.5.0 to 10.5.2,
  harper-cli 3.0.0 to 3.1.0,
  uv 0.13.6 to 0.13.7,
  caddy 2.11.7 to 2.11.9,
  cargo-nextest 0.9.150 to 0.9.151,
  zig 0.15.2 to 0.17.1,
  tombi 1.9.8 to 1.10.0,
  zizmor 1.31.1 to 1.32.0,
  opentofu 1.13.2 to 1.13.9,
  typescript-language-server 6.0.2 to 6.0.3,
  socket 1.12.2 to 1.14.1,
  wrangler 4.156.0 to 4.158.0,
  pagefind 1.5.2 to 1.6.0,
  betterleaks 1.9.0 to 1.10.0.
  The first run failed on the `llama.cpp` lookup and wrote nothing;
  one failed lookup stops the whole write.
- Machine-global lock (`mise run toolchain:update:global`,
  28 tools moved),
  among them rust nightly-2026-11-01 to
  nightly-2026-11-07,
  `cargo:monochromatic-git-policy-cli` 0.5.25 to 0.5.34,
  rclone,
  aws,
  codex,
  and the others
  listed by the task's own output.
- pnpm (`mise run deps:update`):
  `pnpm-lock.yaml`;
  then `catalog:tighten` raised catalog floors
  (oxlint 1.91.0,
  rolldown 1.3.0,
  playwright 1.65.0,
  and others) in `pnpm-workspace.yaml`.
- Cargo (`cargo-shadow update`):
  156 newly locked versions across the lock files,
  28 `Cargo.lock` files changed
  (among them `dylint_linting` and `dylint_testing` 6.3.2 in the closure lint,
  which stays as it is);
  `catalog:tighten` raised floors in `rust-catalog.toml`,
  and `sync:files` rewrote 20 manifests.

## Containers follow the lock

Seven places kept their own copy of the lock's nightly (`nightly-2026-10-29`).
None does now.

- The Containerfiles of `package/terminal/app` (two),
  `package/nested-wayland-session/cli`,
  `package/music-player/desktop-app` (two) and `package/forbidden-strings/core` (`mutants.Containerfile`)
  declare `ARG RUST_TOOLCHAIN` with no default and stop the build with a message when it is empty.
- Their callers pass it:
  `package/test-harness/core-matrix/src/bin/locked-rust-toolchain.ts` prints
  `RUST_TOOLCHAIN=<tools.rust.version of the nearest toolchain.lock.json>`,
  beside the existing `cachyos-image` command,
  and each image task (the `mise.toml` files and the translated `task.ts` files) adds it to `podman build`.
- `PINNED` in `package/cargo-shadow/cli/src/toolchain_tests.rs` is read from the repository lock.
- The same holds for Zig and `cargo-about`:
  the Containerfiles declare `ZIG_ARCHIVE_URL`,
  `ZIG_ARCHIVE_SHA256` and `CARGO_ABOUT_VERSION` with no default,
  and their callers read the values from `package/test-harness/core-matrix/src/bin/locked-tool-build-arguments.ts`
  (`zig` or `cargo-about`).
  The Zig install checks the SHA-256 and prints `zig version`.
- Images follow the lock after it changes:
  `package/test-harness/core-matrix/src/bin/container-image-build-if-inputs-changed.ts` labels each image with a digest of
  its build arguments (and the ids of the local images it starts from),
  and every task that built "only when missing" now builds when the label differs from the digest of the arguments it
  would pass now.
  The explicit `image` tasks always build and label.
  Limits:
  an image built before the label existed has none and rebuilds once;
  the digest covers build arguments and base images,
  not the contents of a Containerfile;
  the `image:windows-gnu` task and `package/nested-wayland-session/cli` still use the old exists check.
- Built on 2026-11-07 with the lock's nightly-2026-11-07:
  see "Images built".

## The daily update

### Where it is defined

- Source:
  `package/monorepo-task/core-exec`.
  The steps and the sequencing are TypeScript with injected effects
  (`src/daily-update-plan.ts`,
  `src/daily-update-run.ts`),
  run by `src/daily-update.ts`;
  the unit sources are
  `systemd/toolchain-daily-update.service` and `systemd/toolchain-daily-update.timer` beside it.
  `core-exec` already owns the `mpm-shadow` runner and the other root task helpers,
  so no new package was made.
- Tasks:
  `mise run toolchain:update:daily` runs one update now;
  `mise run toolchain:timer:install` renders the two unit
  files into `~/.config/systemd/user`,
  checks them with `systemd-analyze --user verify`,
  reloads systemd and enables
  the timer only (`-- --no-enable` leaves it off).
  It refuses a linked worktree,
  because the units name the checkout.
- The update runs the TypeScript source directly,
  never a built copy,
  so a stale build cannot be what the timer runs.

### Schedule and limits

- `OnCalendar=daily`,
  `Persistent=true` (a missed run starts at the next boot or wake),
  `RandomizedDelaySec=2h`.
- The service is `Type=oneshot` with no `[Install]` section:
  only the timer starts it,
  never a login or a boot.
- Caps from the measured run (see `Measured run`):
  `CPUQuota=200%`,
  `MemoryMax=8G`,
  `TasksMax=2048`,
  `IOWeight=10`,
  `Nice=10`,
  `TimeoutStartSec=4h` (2 hours until the package checks were added;
  section "Checking every package after the update" has the measurement).
  Every command inside also has its own time bound and its process group is stopped when
  the bound passes,
  so nothing waits forever.

### The order and why

Three groups,
run one after another.
Each group lands as a unit or not at all;
a failed group never stops the next one.

1.  Repository toolchain:
    `toolchain:update`,
    `toolchain:install`.
    Commits `toolchain.lock.json` (and the plan,
    if a
    person had changed it and left it dirty,
    the run refuses;
    see `Failure, dirty files and reporting`).
2.  Machine-global toolchain:
    `toolchain:update:global`,
    `toolchain:install:global`.
    The global lock lives outside the
    repository,
    so nothing is committed;
    on failure its text is written back.
3.  Dependency locks,
    in an order where each lock is consistent with the others when it lands:
    `deps:update` (pnpm lock,
    in range),
    `cargo-shadow update` (every `Cargo.lock`,
    under the release-age and trust policy),
    then one tail:
    `catalog:tighten` (pnpm and Rust catalog floors,
    after both updates so the floors see the new locks),
    `sync:files` (renders the Rust catalog into the manifests),
    `pnpm install` (the pnpm lock satisfies the new catalog),
    `cargo-shadow install` (each `Cargo.lock` satisfies the new manifests),
    `check:files` (nothing generated is stale).
    The two updates are soft steps:
    if one fails,
    its own files are put back,
    the other still lands,
    and the owner is told.
    The tail is a hard step:
    if it fails,
    every dependency file is put back and nothing from the group lands.

### Failure, dirty files and reporting

- A step that fails or runs past its bound lands nothing from its group;
  the files it changed are written back from the
  text read before the group began (no `git checkout`,
  so no guard is crossed and nothing else is touched).
- A group whose files already have uncommitted changes is skipped and the notification names them,
  so the update never
  mixes with someone's work.
  For the dependency group that means any dirty `Cargo.toml`,
  `Cargo.lock`,
  `rust-catalog.toml`,
  `pnpm-lock.yaml` or `pnpm-workspace.yaml`.
- A changed file outside the group's ownership is left uncommitted and named.
- Landing is `git -C <repo> commit --message ... -- <exact paths>`;
  the Git wrapper serialises it and auto-push does the
  rest.
  A commit that fails is reported and the files stay in the working tree.
- Reporting:
  a desktop notification (`notify-send`,
  as `/etc/systemd/system/mount-860.service` does) and the same text in
  the journal (`journalctl --user --unit toolchain-daily-update`).
  A run sends at most one notification,
  which holds the groups' problems and the package checks' findings together;
  the notifier is bounded to 30 seconds and is never given a session bus address the process did not have.
  A failed group makes the service exit nonzero;
  nothing in the package checks or in `gh` changes the exit status.
- One run at a time:
  an exclusive lock file `.cache/toolchain-daily-update.lock` names its holder;
  a live holder makes a
  second run exit at once,
  a dead holder's lock is replaced.

## Checking every package after the update

Added 2026-11-07 for the owner's answer "Move anyway,
fix forward"
(`doc/planning/rust-nightly-always-latest.md`,
section "What happens when a new nightly breaks a package").
The lock goes to the newest versions regardless;
this section is how a package the update breaks becomes a fix-forward issue instead of a surprise.
The first daily run (`d61dc1934a`) raised the zbus floor and moved the nightly,
`package/nested-wayland-session/cli` and `package/music-player/desktop-app` stopped compiling,
and nobody was told.

### What is checked (the proposal)

This list is a proposal made by the agent that built the check,
recorded here so the owner can change it.
Each check is the package's own `mise` task,
run from the repository root as `mise run //package/<path>:<task>`.

- Rust crates:
  `lint:clippy`,
  which compiles the crate and runs Clippy with warnings denied.
  A compile break from a compiler or dependency move shows here and nowhere sooner.
- TypeScript packages:
  `lint:types`,
  the package's `tsc` run through `task-tsc`.
- Packages with neither task are not checked.
- Not checked:
  tests,
  fuzz and mutation tasks,
  container-only builds,
  Android,
  and Windows and macOS
  ("Ignore Win and mac until I say so.").
  A failing test is a different report from a package that no longer compiles,
  and the test tasks take hours on this host.

The checks are found by `mise tasks ls --all --json` on every run,
so a package that gains one of the two tasks is checked from the next run.
On 2026-11-07 that was 31 Rust crates and 218 TypeScript packages;
14 packages have neither task and are skipped
(`config/dprint`,
`config/typescript`,
`done/app`,
`done/variant-postcss`,
`figma/kiwi`,
`figma/to-penpot`,
`linter/kotlin`,
`logger/core`,
`music-player/design`,
`ssg/aquati.cat`,
three `test-fixture` packages and
`zip-writer/core`).

### Before and after, in the same checkout

1.  Before anything is updated,
    every package is checked (the baseline pass).
2.  The groups run and commit as before.
3.  When a group committed or the machine-global lock changed,
    every package is checked again (the second pass).
    A run that moved nothing skips the second pass,
    because no check can have become red.
4.  For each check the two results are compared:
    passed then failed is newly red;
    failed then failed is already red and is never reported again (`package/linter/monochromatic-lint-closure` is red
    today and the owner said "Wait for Dylint");
    failed then passed is recovered;
    passed then passed is unchanged;
    anything else,
    including a check that timed out or never started because the pass ran out of time,
    is unknown.
    An unknown check never gets an issue.

The baseline is measured in the same checkout,
immediately before the update,
and not read from yesterday's run:
this repository's main moves many times a day,
so a stored result would blame the update for other people's commits.
The cost is one more pass over the packages every day,
measured below.

The move never waits on the check and is never undone by it:
the groups commit before the second pass starts,
nothing in the check phase restores a file,
and a failure anywhere in the check phase (the task listing,
a check,
`gh`) is logged,
told to the owner,
and does not change the exit status of the update.

### Bounds and measurements

Every value is a ceiling that stops a hung check,
not a forecast,
and cites the run that sets it.
The measured run (2026-11-07,
this worktree,
host load average between 10 and 40) used the pools and caps below,
in a
transient unit with `CPUQuota=400%`,
`MemoryMax=14G`,
`TasksMax=3000` and `IOWeight=10`.

- Rust check:
  30 minutes.
  Measured on all 31 crates in a fresh worktree:
  median about 60 seconds,
  longest passing 332 seconds,
  longest failing 452 seconds (`ide/app`,
  a container build);
  four times the longest.
- TypeScript check:
  10 minutes.
  Measured on 218 packages:
  median 3.5 seconds,
  90th percentile 6.5 seconds,
  longest 97 seconds;
  six times the longest.
- Pools:
  2 Rust checks and 4 TypeScript checks at once,
  `CARGO_BUILD_JOBS=2` for each Rust check.
  The measured run of the 64 checks not yet done after a first attempt took 19.5 minutes of wall clock,
  42 minutes of CPU
  (2.2 cores on average) and peaked at 5.5 GiB of memory including page cache,
  with 240 MiB of swap.
  The first attempt (185 checks,
  179 of them TypeScript) took 5.1 minutes at 3.3 cores.
  Both fit the service's two cores and 8 GiB.
- One pass:
  90 minutes,
  after which checks that have not started are skipped and counted as unknown.
  About twice the estimate for the service's two cores;
  a pass after a nightly change rebuilds every crate,
  which was not
  measured.
- Start timeout of the service:
  4 hours,
  up from 2 hours:
  the update (21 minutes in the rehearsal) plus two passes of at
  most 90 minutes.
  The installed unit still says 2 hours until `mise run toolchain:timer:install` runs from the main checkout.
- `gh`:
  60 seconds per call;
  `notify-send`:
  30 seconds;
  `mise tasks ls --all`:
  120 seconds (5.2 seconds cold).
- Moved versions listed in an issue: 100.
  The first daily update moved a median of 52 crates in a package's `Cargo.lock`,
  and 206,
  124 and 104 in the three
  largest.

The measured run also showed the failures a pass reports,
none of which this update caused:
`linter/monochromatic-lint-closure` (E0797,
the known "Wait for Dylint" red),
`terminal/app` (its build script runs
`git checkout`,
which the Git wrapper's linked-worktree guard refuses,
so it is red in any worktree),
`ide/app` (the same family,
in a container build;
the failure prints no line that starts with `error`,
so its issue
would say so and give the reproduce command) and `vm-builder/cli` (`lint:types`,
TS2307).
Several TypeScript packages fail in a fresh worktree only because their `dist` is not built;
the first attempt saw 13.
These are baseline reds when the daily run sees them before and after the update,
and so never get an issue.

### The issue

One issue per newly red check,
opened through `gh` in this repository (it infers the repository from the checkout).
`gh` reads its own credentials;
no token is on a command line or in the output,
the body travels on standard input,
and the process runs with `GH_PROMPT_DISABLED=1`,
`GH_NO_UPDATE_NOTIFIER=1`,
`GIT_ASKPASS` set and empty,
and `GIT_TERMINAL_PROMPT=0`.

Labels:
`needs-triage` only (`doc/agent/triage-labels.md`).
Nobody has evaluated an automatic report,
which is what that label means;
the owner or an agent adds `ready-for-agent` after reading it.

Title:
`<package>: <task> fails after the daily update (<error code or message start>)`.

Body,
in order:

- One sentence that says which check of which package passed before the update and fails after it,
  and that the update is kept and this is the fix-forward report.
- Update:
  the commits the update made (full object names),
  or the statement that only the machine-global toolchain
  moved;
  and the commit the repository was at when the check last passed.
- What moved:
  per lock,
  the versions that moved,
  one `name from to to` line each.
  For a Rust crate that is its `Cargo.lock`;
  for a TypeScript package,
  its importer block in `pnpm-lock.yaml`;
  for every package,
  the repository `toolchain.lock.json` and the machine-global lock.
  Entries whose crate or package the first error names come first and say so;
  the list is capped,
  and the rest is
  counted.
  When no lock of the package moved,
  the body says that plainly.
- First error:
  its code,
  then the error line and the lines that continue it up to the next blank line.
- Reproduce:
  `mise run //<package>:<task>` from the repository root,
  on the update commit.
- A visible sentence containing "toolchain daily update report" (what a later run searches for) and a hidden marker
  `<!-- toolchain-daily-update package=<package> task=<task> cause=<cause> -->`.

Cause is the error code (`E0433`,
`TS2307`) or,
for an error without one,
a digest of its message with the digits
removed,
so a moved line number does not make a new cause.

Deduplication:
before creating,
one `gh issue list --state open --search '"toolchain daily update report" in:body'`
finds the open reports (a search,
because this repository has thousands of open issues and a plain listing is capped),
and the marker decides.
An open issue with the same package,
task and cause gets a comment with the update commits and the first error
instead of a second issue.
Two findings with one marker in one run make one issue and one comment.
The marker names the package followed by a fixed word,
so `package/a/b` is never matched by the issue of `package/a/bc`.

A `gh` failure (offline,
authentication,
a timeout) is logged with its command,
status and error text,
reported in the notification,
and the rendered issue is kept in `.cache/toolchain-daily-update-pending-issues.json`.
The next run publishes the kept issues first,
before its baseline pass,
even when it moves nothing.
It never fails the update.

Limits:
GitHub's search index can lag by minutes,
so an issue opened a moment ago may not be found by a retry
(within one run the issues just created are remembered);
an issue is not closed when the package recovers;
and the cause of a break that the update shares with another commit in the same window is the reader's to judge,
which is why the body shows what moved.

### Notifications

At most one desktop notification per run:
the update's own problems (a skipped or failed group),
the issues opened or commented on (with their addresses,
five
listed and the rest counted),
the checks that could not be compared,
and any `gh` failure,
joined in one text.
The journal keeps the whole text.
The desktop shows the first thousand characters.
`notify-send` is bounded to 30 seconds (GLib waits at most 25 seconds for a bus reply),
and the notifier is never given
a session bus address the caller did not have.

On 2026-11-07 about eleven real desktop notifications named `toolchain-daily-update` appeared on the owner's desktop.
The cause was this change's own whole-path test:
it ran the real entry point,
whose notifier fell back to the owner's session bus at `/run/user/<uid>/bus`
when the test environment carried no address.
The timer had not fired.
The measurement run did not call the notifier.
The fixes are the removal of that fallback,
and a test environment in which the real `notify-send` can only reach a
private bus:
the whole-path test hosts the update in the nested-wayland-session notification recorder (below),
and a
probe refuses to start unless the hosted environment has a private bus address and no runtime directory.

### Tests

- `src/daily-update-package-checks.unit.test.ts`:
  which checks exist,
  the first error,
  newly red against already red,
  lock moves,
  the issue text,
  publishing and deduplication,
  pools and deadlines,
  the whole sequencing with scripted
  effects,
  one notification for a run in which thirty packages break,
  and a `gh` failure kept for the next run.
- `src/daily-update-whole-path.expensive.unit.test.ts` (run with `--all`,
  because it needs the recorder's build):
  the real entry point over a disposable repository with a recording stand-in for `gh`,
  real `mise` and `git`,
  two crates and one TypeScript package,
  one that the update breaks and one that was already red.
  The entry point runs inside `monochromatic-nested-wayland-session-notification-recorder`
  (`package/nested-wayland-session/cli`,
  a private `dbus-daemon` with no service directories and its recording
  `org.freedesktop.Notifications` service,
  no compositor),
  so the real `notify-send` sends into a bus the owner's desktop
  never sees,
  and the test asserts the notification the recorder logged.
  The probe `assertHostedOnPrivateBus` passes inside the recorder and fails for the same command run outside it.
  The repository has no remote and the stand-in `gh` is first on `PATH`,
  so no real issue can be opened.
  The recorder is built by `mise run //package/nested-wayland-session/cli:test` (or `:build`),
  or named by
  `NESTED_WAYLAND_NOTIFICATION_RECORDER`.
- Proof,
  2026-11-07:
  with the stand-in,
  one `issue list` and exactly one `issue create` were recorded,
  with the label
  `needs-triage`,
  the update commits,
  the `zbus` and toolchain moves,
  `E0433` and the reproduce command,
  and none for
  the crate that was already red;
  the recorder logged one notification,
  `Daily update needs a look`,
  naming the issue
  address;
  with `gh` failing,
  no `create` was recorded,
  the issue was kept in the pending file,
  and the notification
  carried the `gh` error.

### Limits of the check

- A package is checked by one task;
  a break that only the tests show is not seen.
- The baseline is taken immediately before the update in the same checkout,
  so a commit by someone else during the
  run can still appear in the second pass.
- The whole-path test needs a build of the recorder and so is not part of the default unit run.
- The check phase cannot tell a failure the update caused from one that another commit caused in the same window;
  the
  issue shows what moved and the reader judges.
- Container-built crates (`ide/app`,
  `terminal/app`,
  `nested-wayland-session/cli`,
  `music-player/desktop-app`) are
  checked through their own task,
  which falls back to a container on this host;
  a failure there may print no line that
  starts with `error`,
  and the issue then says so.

## Images built

On 2026-11-07,
through the repository's own image tasks,
each in a capped scope.
Every image reports `rustc 1.101.0-nightly (c980177d8 2026-11-06)`,
the lock's `nightly-2026-11-07`.

- `package/nested-wayland-session/cli:image`:
  built,
  370 s.
- `package/forbidden-strings/core:test:mutation:image`:
  built,
  239 s.
- `package/terminal/app:image`:
  built,
  320 s.
- `package/terminal/app:image:release`:
  built,
  95 s.
- `package/music-player/desktop-app:image`:
  built,
  704 s.
- `package/music-player/desktop-app:image:windows-gnu`:
  built,
  515 s (an image that only builds on Linux;
  no Windows task ran).
- Not built:
  nothing was skipped.
  `package/ide/app` images start `FROM` the terminal images and set no toolchain of their own,
  so they follow the rebuilt base.

## Zig 0.17.1 results

Run on 2026-11-07 in containers capped with `--cpus`,
`--memory`,
`--pids-limit` and `--blkio-weight` inside a `systemd-run --user --scope`
with `IOWeight` 10.

- The five images below were rebuilt by `container-image-build-if-inputs-changed` (each carried no label,
  so each was stale):
  `localhost/monochromatic/terminal`,
  `terminal-release`,
  `music-player`,
  `ide` and `ide-release`.
  `zig version` printed 0.17.1 in `terminal`,
  `terminal-release`,
  `music-player` and `ide`
  (`ide-release` starts from `terminal-release`),
  and `cargo-about --version` printed 0.9.3 in `ide`.
  A second run of the command on each image built nothing,
  so the labels match the digests the tasks compute.
- `package/terminal/app`:
  `cargo clippy --locked --release --all-targets -- --deny warnings` passed,
  and `cargo nextest run --locked --release` ran 16 tests,
  16 passed.
- `package/music-player/desktop-app`:
  `cargo zigbuild --locked --release --target x86_64-unknown-linux-gnu.2.39` finished in 5m09s on Zig 0.17.1
  after the `branches` and `zbus` repairs;
  `cargo clippy --locked --release --all-targets -- -D warnings` passed after five findings of the newest nightly's Clippy
  (a shadowed binding,
  two constant assertions and two needless borrows in tests) were fixed.
  `cargo nextest run --locked --release --no-fail-fast` with `SLINT_EMIT_DEBUG_INFO=1` ran 1594 tests,
  1593 passed,
  1 skipped,
  1 failed:
  `ui_lists_render_tests::a_search_result_title_wider_than_the_page_ends_in_an_ellipsis`
  ("the long title ends in an ellipsis",
  `src/ui_lists_render_tests.rs:227`).
  That is a text-layout assertion in the Slint renderer test;
  it was not investigated and nothing in this change touches it.
- The terminal task `mise run //package/terminal/app:test:container` ran through the rewired dispatch:
  the image was current (no build),
  and 16 of 16 tests passed.
- Found on the way,
  separate from Zig:
  the open-ended `zbus` floor and the `branches` feature unification
  (section "Fix-forward items").

## Measured run

A rehearsal ran the whole update as a transient systemd unit in a throwaway worktree
(`.claude/worktrees/toolchain-daily-rehearsal`,
branch `feat/toolchain-always-latest-code`),
with the same working directory,
caps and environment as the installed service,
from 2026-11-07T06:39:09-05:00.

- Wall clock 20 min 51 s,
  CPU time 2 min 55 s,
  memory peak 4 GiB (the cap of that rehearsal,
  page cache included),
  at most 582 tasks (sampled once a second).
- Repository toolchain group:
  2 min 39 s (`toolchain:update` 67 s,
  `toolchain:install` 92 s).
- Machine-global group:
  14 min 34 s,
  almost all of it `toolchain:install:global` building from source
  (a `go install` with cgo,
  and the Git wrapper's crate through `cargo install`),
  which is why the memory peak is high.
- Dependency group:
  3 min 36 s (`deps:update` 29 s,
  `cargo-shadow update` 100 s,
  `catalog:tighten` 8 s,
  `sync:files` 9 s,
  `pnpm install` 15 s,
  `cargo-shadow install` 45 s,
  `check:files` 9 s).
- The rehearsal committed one lock file for the toolchain group and 51 files for the dependency group in the throwaway
  worktree,
  and touched the real machine-global tools (it installed what the global lock names).
- Sizing from it:
  `CPUQuota=200%` (the mean was well under one core),
  `MemoryMax=8G` (twice the peak,
  which touched the old
  cap;
  the host has far more),
  `TasksMax=2048` (3.5 times the peak),
  `IOWeight=10` (the ceiling the rules allow),
  `TimeoutStartSec=2h` (six times the run).
  The per-command bounds in `daily-update-plan.ts` (15 min for a resolution,
  45 min for everything else) are chosen
  ceilings that stop a hung registry or download;
  the measured longest single command was the global install at 14 min.

## First run

Run 2026-11-07T08:30:04-05:00 to 2026-11-07T08:33:17-05:00,
after the crate release batch finished
(2026-11-07T13:10:36Z) and the seven repository tools moved to the `archive` route (`doc/planning/one-release-each-batch-2026-11.md`,
"Steps after the batch").

- Steps taken:
  merge `feat/toolchain-always-latest-landing` (`ea2f72c3c1`,
  2026-11-07T08:20:23-05:00);
  `mise run toolchain:timer:install` (2026-11-07T08:29:48-05:00,
  timer enabled,
  next run 2026-11-08T00:07:05-05:00);
  `systemctl --user start --no-block toolchain-daily-update.service`,
  followed with `journalctl --user --unit toolchain-daily-update`.
- Result of the service:
  `success`;
  3 min 12.6 s wall clock,
  1 min 15.3 s CPU,
  1.4 G memory peak
  (the rehearsal took 20 min 51 s because its global install compiled the wrapper;
  now every repository tool installs from an archive).
- Repository toolchain:
  unchanged,
  0 files committed (the lock had just been updated by hand through `toolchain:update`);
  `toolchain:update` 38 s,
  `toolchain:install` 3 s.
- Machine-global toolchain:
  unchanged,
  0 files committed (outside the repository);
  `toolchain:update:global` 24 s,
  `toolchain:install:global` 14 s.
- Dependency locks:
  landed,
  51 files in one commit,
  `d61dc1934a` (2026-11-07T08:33:16-05:00,
  pushed):
  `deps:update` 6 s,
  `cargo-shadow update` 76 s,
  `catalog:tighten` 6 s,
  `sync:files` 2 s,
  `pnpm install` 4 s,
  `cargo-shadow install` 16 s,
  `check:files` 3 s;
  35 lockfiles satisfy their manifests;
  `bincode` stayed on 1.3.3 in `package/git-policy/cli/Cargo.lock`
  (the `bincode-1` catalog held).
- No group failed;
  no restore and no notification were triggered.
- Seen,
  not changed:
  `toolchain:install:global` and `toolchain:list:global` disagree about 12 npm-routed global tools
  (`npm:@github/copilot`,
  `npm:@deepseek-ai/dsh` and others):
  each install reports them installed
  (`12 installed, 50 unchanged, 0 failed`,
  again on the repeated run),
  the list reports them absent
  (`12 of 62 tools drifted`) and an `unresolved bins:` line names their programs.
  The service still ends `success`,
  because the install step exits 0.

## Dependency locks: what the daily update changes and what usually breaks

What each tool changes,
all through the repository's own tasks,
never by hand:

- `mise run deps:update` is `pnpm update --recursive --no-save`:
  it moves `pnpm-lock.yaml` inside the catalog ranges.
  Exact catalog entries (the `@deepseek-ai/dsh-*` family at `0.2.1-rc.5`,
  `@deepseek-ai/cordis`,
  `schemastery`) never move.
  A same-day release under the strict release age is refused,
  and the task explains which one.
- `cargo-shadow update` re-resolves every `Cargo.lock` to the newest release older than `minimumReleaseAge` that passes the
  trust policy,
  and keeps the result only when nothing was refused;
  a refusal restores every lock file.
- `mise run catalog:tighten` raises each `>=` floor in `pnpm-workspace.yaml` and `rust-catalog.toml` to the highest locked
  version;
  `mise run sync:files` renders `rust-catalog.toml` into the generated tables of the crate manifests.
- `pnpm install` and `cargo-shadow install` then make each lock satisfy the new floors.

What usually breaks,
with what the first update showed:

- An open-ended floor (`>=1.3.3`) lets an update cross a major version or land a placeholder.
  The first update locked `bincode` 3.0.0,
  a crate whose library is a single `compile_error`
  (its message is an xkcd address,
  comic 2347),
  in seven lock files;
  `forbidden-strings/core` uses the 1.x functions,
  so the Git wrapper's
  crate and every crate that depends on `forbidden-strings-core` stopped compiling.
  Fixed on the branch:
  `[catalogs.bincode-1]` in `rust-catalog.toml` (`^1.3.3`),
  selected by `forbidden-strings/core`,
  and the seven lock files back on 1.3.3.
  Other open-ended floors can do the same on a later day;
  a daily job that lands first and fixes forward will meet them.
- A compiler update breaks crates that read compiler internals:
  `package/linter/monochromatic-lint-closure`
  (already red,
  "Wait for Dylint";
  `dylint_linting` and `dylint_testing` moved to 6.3.2 and it still stops at E0277 and E0797).
- A new `typescript` or `oxlint` plugin API breaks lint rules;
  the packages checked after the update lint clean.

Checked after the first update,
one at a time,
on the branch with the new locks:

- Passed:
  `mpm-shadow/cli` (`lint:clippy`,
  `test`),
  `file-enforcer/cli` (`lint:clippy`,
  `test`),
  `pnpm-shadow/cli` (`test`),
  `search-shadow/cli` (`test`),
  `jsonc-edit/core-rust` (`test`),
  `cargo-shadow/cli` (`test`,
  652 tests),
  `linter/monochromatic-lint` (`lint:types`,
  `test`),
  `forbidden-strings/core` (`lint:clippy`,
  after the bincode fix),
  `monorepo-task/core-exec` (`lint:types`,
  `lint:oxlint`),
  `test-harness/core-matrix` (`test`,
  `lint:types`,
  `lint:oxlint`),
  `config/oxlint` (`lint:types`).
- Failed before the fix:
  `forbidden-strings/core` (`lint:clippy`) on bincode 3.0.0.
- Failed and already failing:
  `linter/monochromatic-lint-closure` (`lint`),
  unchanged cause.
- Not run:
  the container-built desktop packages' tests,
  the Git wrapper's container gate,
  fuzz and mutation tasks,
  Android and Windows and macOS tasks.

## Fix-forward items

- bun 1.4.4 in both locks:
  nothing has run on it;
  the pin's reason was a doubt about the rewrite.
- zig 0.17.1:
  done (section "Containers follow the lock").
  The Slint and `cargo-zigbuild` builds of the music player and the terminal's Clippy and nextest ran on 0.17.1
  (section "Zig 0.17.1 results").
- `libghostty-vt` comes from a `libghostty-rs` git revision;
  return to a crates.io release once one vendors a Ghostty that builds with the current Zig.
- `branches` 0.4 and `turso_core`:
  the named catalog `branches-0-4` enables `std` on the line `turso_core` uses;
  remove it when `turso_core` requires `branches` 0.5 or newer
  (`doc/troubleshooting/branches-0-4-calls-removed-core-intrinsics-abort.md`).
- zbus 6:
  done on 2026-11-07.
  The daily update locked 6.2.1,
  which removed `zbus::blocking`;
  `package/music-player/desktop-app` and `package/nested-wayland-session/cli` are ported to `zbus::block_on`
  and the named catalog `zbus-5` is gone
  (`doc/troubleshooting/zbus-6-removed-the-blocking-api.md`).
- cargo-fuzz:
  the next point release may break the fuzz crates;
  the pin's reason says it has before.
- llama.cpp and `llama.cpp-cpu`:
  `mpm-shadow` needs a way to take prerelease GitHub releases before they can follow latest.
- Git-sourced global tools (`cargo:below`,
  `slint-lsp`):
  `mpm-shadow` needs a branch or `HEAD` request before they can follow latest.
- java:
  the resolver has no latest form;
  a newer major waits for Android Gradle Plugin support.
- Open-ended `>=` floors in `rust-catalog.toml` and the pnpm catalog can cross majors on any day;
  a bounded named catalog (as `bincode-1`) is the repair each time.
- `package/monorepo-task/core-exec:test:unit` runs every test of the package;
  `depends.unit.test.ts` ran for more than
  half an hour in one attempt and the run was stopped.
  It is not part of this change and was not investigated.
- Two files that `format:oxlint` reformatted as a side effect (`mpm-shadow.unit.test.ts`,
  `playwright-image.unit.test.ts`
  in `core-exec`) were left uncommitted in the working branch:
  the guard on `git restore` refused to put them back.

## Limits

- Windows and macOS were not touched ("Ignore Win and mac until I say so.").
- The rehearsal could not run the failure path against a real failing network step;
  the failure rules are covered by
  the unit tests with scripted effects,
  and the desktop notification was sent once as a test.
- The machine's tool installs are shared by every checkout:
  while two checkouts hold different locks,
  each
  `toolchain:install` replaces the other's versions,
  so tools flip until the locks agree.
