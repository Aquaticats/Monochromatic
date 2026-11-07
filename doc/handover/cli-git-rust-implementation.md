# cli-git implementation handover

Note 2026-11-04T22:23:58-05:00: the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders, logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

This page holds only open work and current state of the Git wrapper effort (`package/git-policy/cli`),
per rule H0P.
Landed and superseded work is recorded in the package docs (`SPEC.md`,
`README.md`,
`DESIGN.md`),
in `doc/decision/`,
in `doc/planning/cli-git-*.md`,
in `doc/troubleshooting/` and in the lane evidence files `doc/handover/cli-git-*.md`.
The append-only log that preceded this page ends at commit `e10b8186bc`;
`git show e10b8186bc:doc/handover/cli-git-rust-implementation.md` prints it.
Add a short entry here only for something still open;
when it lands,
move its durable facts to their canonical doc and delete it from this page.

## Current state

Installed versions and the release in flight were written 2026-11-04T14:08:00-05:00 against main `651fc7b9a5`;
the branch and agent lists below them are older and carry their own times.

### Installed versions

- Git wrapper `monochromatic-git-policy-cli` 0.5.32 (`toolchain.lock.json`,
  row `cargo:monochromatic-git-policy-cli`,
  lock commit `e6bb6fbb62`;
  `git --version` prints "monochromatic-git-policy-cli 0.5.32,
  shadowing git at /usr/bin/git" over Git 2.56.0).
  Installed by `upgrade-wrapper.ts 0.5.32`:
  the 0.5.31 service retired on the client's request,
  0.5.32 started (protocol 7),
  and the sample commit on main landed in 1680 ms in all,
  1215 ms of it provisioning the new service (landing 31 ms,
  lock hold 28 ms).
  Its commit service unit runs with systemd's defaults
  (`systemctl --user show` at 2026-11-03T10:47-05:00:
  `MemoryMax=infinity`,
  `MemoryHigh=infinity`,
  `CPUQuotaPerSecUSec=infinity`,
  `IOWeight` not set,
  `TasksMax=75462`,
  systemd's own default).
- `monochromatic-mpm-shadow` 0.2.5,
  `monochromatic-pnpm-shadow` 0.1.1,
  `monochromatic-search-shadow` 0.2.1,
  `monochromatic-file-enforcer-cli` 0.1.1,
  `monochromatic-lint` 0.5.0 (all from the same lock file).
- `monochromatic-dependency-catalog` 0.2.0 (with `trustLockfile`) was published 2026-11-04T18:58:16Z,
  after the cargo-shadow merge reached main;
  0.1.0 is the earlier version.
- `monochromatic-cargo-shadow` 0.1.0 was published 2026-11-04 with the owner's `cargo login` token
  (`mise run //package/cargo-shadow/cli:publish`);
  Trusted Publishing config 30988 (`Aquaticats/Monochromatic`,
  `cargo-publish.yml`,
  no environment,
  `repository_owner_id` 343218767) was created through the API.
  It is not yet in `toolchain.lock.json`.
- The global toolchain lock (`~/.config/monochromatic-dev/toolchain/toolchain.lock.json`) was moved to wrapper 0.5.25
  on
  2026-11-03;
  its copy from before that move is `~/temp/agent/global-lock/toolchain.lock.json.before-2026-11-03T0715`.

### Release in flight

Owner, 2026-11-04T17:50-05:00 (approximate time of reading), verbatim:
"Don't discard work, but aside from layout + linting fixes, pause everything else - but let work already in flight run to their natural conclusions. Layout + linting fixes are the biggest priority."
So only the source layout (issue 6261, principles in AGENTS.md MKY, LYD, LYF) and lint fixes go forward.
In-flight steps finish, commit and stop:
branch-write round 5 (re-gate with main, `-v2` bundle; its review not started),
the commit-protocol models round (main merge and the fix in progress; reviews A and B of step3c left for later),
cargo-shadow round 21 (the fix in progress; the rest written to its resume point),
the lean build directories proposal (finished as a proposal, nothing built),
and the claim-watch fix review already running.
Paused, not started: release 0.5.35 (one executable, claim-watch fixes and the startup wake are on main; round 5 not yet),
the mpm-shadow release,
the TypeScript port,
new review rounds,
and the `semantic-line-breaks` findings (owner: "Put the semantic line break findings on hold.").

Update 2026-11-04T15:49:27-05:00 (main `7929d95d65`):

- `monochromatic-file-enforcer-cli` 0.1.2 is published (bump `fadde01fff`,
  crates.io 2026-11-04T20:41:44Z),
  locked and installed (`7929d95d65`);
  `file-enforcer --version` prints 0.1.2.
  0.1.1 had been built with dependency-catalog 0.1.0 and rejected `rust-catalog.toml` once `912e82a0c7` set `trustLockfile`
  ("is not a catalog table or a supported setting"),
  so `mise run sync:files` failed on main;
  `bce22f7456` had moved the cli source to `=0.2.0` without a version bump.
  CLAUDE.md is regenerated and carries SUF.
- The installed wrapper 0.5.34 is not the release archive's binary:
  the archive `monochromatic-git-policy-cli-0.5.34-x86_64-unknown-linux-gnu.tar.gz` holds only `cli-git-native`
  (13,728,056 bytes) and `README.md`,
  while the install directory `~/.local/share/monochromatic-dev/toolchain/cargo/monochromatic-git-policy-cli/0.5.34/bin/`
  holds `cli-git-native` (13,710,088 bytes) and `cli-git-service`.
  Cause:
  the crate declares two `[[bin]]` targets since `e9eaf63d14` (2026-11-01),
  the release job builds and packages only `cli-git-native` (`cargo-publish.yml:2506`),
  cargo-binstall needs every `[[bin]]` in the archive,
  and mpm-shadow runs `cargo binstall` without `--disable-strategies compile`
  (`package/mpm-shadow/cli/src/toolchain/install_cargo.rs:61-84`),
  so it compiled the crate from source.
  The same archive fails CI's `catalog-specifiers` job (`catalog-specifiers.yml:58` passes `--disable-strategies compile`:
  "bin cli-git-service is not found");
  that job's last success was 2026-11-01T14:34:19Z (`72541fe39c`).
- Owner,
  2026-11-04:
  "Bazel doesn't need to distribute as 2 bins."
  The service becomes a mode of `cli-git-native`;
  agent `a26d7ef8229ffbeb2` on branch `fix/git-policy-cli-one-executable`
  (worktree `~/temp/agent/cli-git-one-executable`);
  then gate,
  merge,
  release 0.5.35,
  and check that `catalog-specifiers` goes green.
- Owner,
  2026-11-04:
  "mpm-shadow should still be able to compile tools that are not controlled by this repo."
  Queued (the five agent slots were full at 15:49):
  mpm-shadow disables cargo-binstall's compile strategy for crates this repository publishes
  (its existing `monochromatic-` prefix test,
  `cargo_installs.rs:70`,
  or the crate-release registry)
  and keeps the fallback for third-party crates;
  red test first;
  a failing repository crate names the crate,
  version,
  binstall's reason and the remedies;
  audit every `cargo-binstall` row with a dry run;
  release only after wrapper 0.5.35 is locked,
  because a fresh install of 0.5.34 would then fail.
  Owner's clarification:
  "It should still BE ABLE TO compile tools this repo doesn't control when needed."
  CM7 rewritten as the owner chose ("Rewrite CM7 (Recommended)",
  `5f03ec99b3`).
  Agent `a97a9a4124bdb9776` started 2026-11-04T15:52-05:00 on branch
  `fix/mpm-shadow-cli-never-compiles-a-repository-crate`
  (worktree `~/temp/agent/mpm-shadow-no-compile`).
- Models agent (`a9c775c9196f1d836`) finished the step-2b fixes and plan step 3 behind `--lane`
  (models tip `45bee5ba00`,
  design tip `1fc759ba80`;
  `native:test:model` 605 passed,
  4 leaky;
  map 66 covered,
  20 narrower,
  9 historical,
  13 not yet).
  Cross-vendor reviews running:
  `~/temp/agent/reviews/commit-models-step3-sol-a1.txt` (code against whole files),
  `-a2.txt` (component tests),
  `-b.txt` (models,
  map,
  handover).
  Its `--lane` options in `service_main.rs` will conflict with the one-executable change;
  that agent was told to keep the service options a pass-through.
  The `-a2.txt` review (component tests) returned "Sufficient after named additions, not sufficient as is"
  with one product risk (blocking doorbell sends can wait without a bound);
  sent to the models agent at once,
  which resumed even with five other agents running
  (resuming a finished agent was not refused by the five-agent limit).
  `-a1.txt` ("not correct as is": a disposal revalidation hole and the doorbell blocking clients) and `-b.txt`
  ("correct after named fixes") were sent the moment each landed.
  The fix round is done (models tip `f98a499e8d`;
  `native:test:model` 625 passed,
  none leaky;
  both clippy tasks clean;
  `source_guard` 39 of 39):
  revalidation reads the newest record's whole retained set;
  the doorbell is a socket file `service.ring` in the lane directory with non-blocking sends and one owner by `service.lock`;
  a double close in `launch_detached_preparing` (a core dump per returned helper in debug builds) fixed;
  every false-green window of `-a2.txt` checked with its guard removed;
  one surviving mutation ("a second service may own the lane") stated as redundant with the live-socket refusal.
  Not done (stated in its handover):
  the flusher lock spans an until-empty sweep loop.
  Cross-vendor reviews:
  `~/temp/agent/reviews/commit-models-step3b-sol-a.txt` (diffs of code,
  tests,
  models,
  handover) "correct after named fixes—not approved as-is" ("The primary disposal fix is convincing.")
  and `-sol-b.txt` (whole current files) "correct after named fixes—not correct as-is";
  both sent to the models agent,
  which is fixing them.
- cargo-shadow 0.1.1 as `cargo` blocked every wrapper gate:
  `bin/native-scanner-snapshot.ts` runs `cargo vendor --manifest-path` on a staged copy outside the work tree,
  and the shadow counted all 281 locked versions of an unchanged lockfile as new and refused 4 on trust level
  (log `~/temp/agent/gate/gpc-gate-branch-write-round4-retry-20261104T155841-0500.log`).
  The `cargo` link is off (`8917a3f221`;
  the install left the old symlinks,
  removed by hand in the main checkout and in the branch-write worktree;
  that stale-link defect went to the mpm-shadow agent).
  The cargo-shadow agent has the vendor defect as top priority;
  the link returns after its fixed release passes the launcher trace again.
- Branch-write round 4 (`aa0fee0c2cf70bb31`,
  tip `07076dc23b`,
  host suite 4874/4874) re-gated without the link:
  unit `gpc-gate-branch-write-round4-no-cargo-link-20261104T160436-0500`;
  review `~/temp/agent/reviews/branch-write-r4-sol.txt` returned "correct after named fixes" (B,
  the side-ref reflog proof,
  still lacks a valid safety proof).
  That gate failed 4873/4874:
  `fault_limits::a_memory_limit_close_to_exhaustion_on_the_service_leaves_a_recoverable_repository`,
  with `thread 'cli-git-durable' ... panicked at library/std/src/thread/scoped.rs:206:46: failed to spawn thread: ... Resource temporarily unavailable`
  and the same panic on `main`;
  the test passed on the round 3 and startup-wake gates.
  Not a round 4 change (its diff adds no scoped spawn):
  `landing/service_durable.rs` `sync_batch` (line 60) and `force` (line 132) call `scope.spawn`,
  which panics when a thread cannot start;
  shipped since `eaad11a3a6` (0.5.28 and later).
  The fix and an audit of every production `std::thread::scope` site go out in the next release.
  Sent to the branch-write agent first,
  before the review fixes,
  on its round 5 branch (`fix/git-policy-cli-side-ref-proof-round5`,
  worktree `~/temp/agent/branch-write-round5`);
  round 4 and round 5 merge together once that gate passes.
  Owner decision on the earlier single service,
  2026-11-04 (question tool):
  "A: fail, retry (Recommended)",
  which round 4 built;
  record it in `doc/planning/cli-git-no-unbounded-waits.md` and `SPEC.md` when the branch merges.
- Service startup wake (`aca6e78c5ffbd5680` done;
  branch `fix/git-policy-cli-service-recovers-claims-without-a-wake`,
  gate 4847/4847 on the Rust tip `f2dc8833f8`,
  e2e kill scenarios green on seeds 1 to 3):
  main merged in cleanly,
  gate `gpc-gate-service-startup-wake-with-main-20261104T160613-0500` running,
  review `~/temp/agent/reviews/service-startup-wake-sol.txt` returned "correct after named fixes"
  (fixes on `fix/git-policy-cli-claim-watch-identity-and-errors`);
  the gate passed 4866/4866 and the branch is merged to main.
  Main now holds it together with the leaked one-executable commit,
  first gated together by the one-executable branch's gate.
- QWD tightened as the owner chose ("Tighten QWD (Recommended)",
  `3abcbf585e`).
- Leak to main:
  the one-executable and mpm-shadow branches were created with `git worktree add -b <branch> <path> origin/main`,
  which set `origin/main` as their upstream,
  and the wrapper's auto-push sends commits to the upstream when the names differ,
  so `9b360150c0` (service as a mode of `cli-git-native`,
  not gated),
  the mpm-shadow never-compile commits (`ae2f2210bc`,
  `0154f65dea`,
  package tests 546/546) and its link retirement (`5b7213a46a`,
  `c71ad82ba1`) reached origin/main.
  Both upstreams were removed at 2026-11-04T16:20-05:00.
  Owner (question tool):
  "A: keep, gate first (Recommended)":
  the one-executable branch is gated before merge and before 0.5.35.
  Future briefs start branches from a commit hash or local `main`,
  or pass `--no-track`.
- Owner on cargo-binstall's quick-install (question tool,
  after a clarification):
  "A: one flag, our crates (Recommended)":
  crates this repository publishes get `--disable-strategies compile,quick-install`,
  third-party crates keep the defaults;
  owner's note:
  "The point of cargo-binstall is for us to not manually point to gh releases ourselves!"
  Sent to the mpm-shadow agent.
  Its audit:
  only wrapper 0.5.34 among repository rows fails with compile disabled (archive lacks `cli-git-service`);
  release mpm-shadow after 0.5.35 is locked.
- cargo-shadow round 19 merged to main (`bc671de431`;
  gate 648/648;
  staged copies in ignored directories go to cargo untouched;
  running-image identity from `/proc/self/exe`;
  K1 unreadable inventory reported as unknown);
  next:
  release 0.1.2,
  relock,
  launcher trace,
  put the `cargo` link back,
  then the fifth certification pass (bundles `~/temp/agent/reviews/cargo-shadow-cert5-*.txt`,
  15 groups).
  0.1.2 published (crates.io 2026-11-04T21:23:13Z,
  publish run 41915321457 built from `b96ca268de`),
  locked and installed without the `cargo` link (`3a59c6cc1c`).
  The launcher trace (`~/temp/agent/cgs-installed-launcher-trace.ts`) passed at 2026-11-04T16:40-05:00:
  `cargo --version` and `cargo metadata` exited 0,
  the link and the shadow each ran once per command,
  and the real cargo ran only through `.cache/toolchain-bin/cargo`
  (three times for `metadata`: the shadow's own lockfile checks plus the command).
  The `cargo` link is back (`c5e3d0d268`);
  in the repo environment `cargo --version` prints `cargo-shadow 0.1.2, shadowing cargo at .../.cache/toolchain-bin/cargo`.
  Not yet seen: a wrapper gate's `cargo vendor` step through the link (the next gate an agent launches shows it).
- Fifth cargo-shadow certification pass:
  all 15 groups "not certified" (`~/temp/agent/reviews/cargo-shadow-cert5-sol-<n>.txt`).
  Groups 1,
  2,
  4,
  7,
  9 and 14 ran without the manifest (packet plus manifest over 250 KB),
  and several say the bodies the manifest assigns were not in the packet;
  the rest are findings.
  Group 15 (launcher):
  the replacement test injects a fixture identity (false green);
  the chosen candidate stays a pathname,
  so a path swapped to the shadow between the check and the exec starts the shadow again;
  the no-record launcher test allows one to six launcher runs.
  All sent to the cargo-shadow agent (`ae0fbdf78c31bd757`) with the packet-splitting suggestion.
  Owner decisions,
  2026-11-04 (question tool):
  re-entry through an unrecorded launcher,
  option A with the note "A, but make the help message descriptive enough that even a monkey can understand and execute it."
  (the second cargo-shadow exits 2 before any work,
  like the Git wrapper's arrival stop in `package/git-policy/cli/SPEC.md`,
  and the bounded fallback goes away);
  the nine delegated picks of 2026-11-03 (`doc/planning/cargo-shadow-update.md`,
  "Delegated picks of 2026-11-03"):
  "Keep all nine (Recommended)",
  so they are owner decisions now.
  Both sent to the agent,
  which records them in the planning doc.
- cargo-shadow round 20 (tip `7d66717bb6`,
  gate 652/652,
  clippy clean) merged to main (`fd6a3cd183`):
  the second start stops at once with exit 2 (`src/arrival.rs`),
  and a test runs both printed fixes as printed;
  the chosen real cargo is checked again before each start.
  0.1.3 published (crates.io 2026-11-04T21:59:45Z,
  publish run 41921008620 built from `a9901d3503`),
  locked and installed as `cargo` (`9c51ac07f2`) after the launcher trace passed again
  (`~/temp/agent/cgs-trace-013.log`).
  Sixth certification pass (14 groups,
  every packet with its manifest,
  `~/temp/agent/reviews/cargo-shadow-cert6-sol-<n>.txt`):
  groups 2 and 4 "certified after named fixes",
  the other 12 "not certified";
  all sent to the agent.
  Round 21 (agent `ae0fbdf78c31bd757`):
  the check-to-start window is closed for a compiled real cargo by starting the checked file through its descriptor
  (`execveat` with `AT_EMPTY_PATH`;
  the real cargo here is rustup's ELF proxy);
  scripts keep the pathname and the re-check,
  because `man 3 fexecve` and `man 2 execveat` say a script started through a close-on-exec descriptor fails with
  `ENOENT`,
  and a hit there ends at the arrival stop.
  Coordinator's reading,
  not an owner question:
  the owner's rule "We should never invoke ourselves recursively. That's bad design. Audit, remove, and ban that design aggressively."
  determines it.
  Then the open list of the fifth pass and every sixth-pass group's named fixes,
  groups 2 and 4 first.
- Worktree adds into `~/temp/agent` take over 15 minutes and fill the disk queue:
  `~/temp/agent` is `/var/mnt/encrypted/temp/agent`,
  another filesystem than the main checkout,
  so the wrapper's add falls to rung 3 (Git's checkout,
  then the ignored-state copy of `node_modules`,
  `target/`,
  `dist/`,
  local configuration and `.cache/`,
  README "worktree add" section) as a byte copy.
  Measured 2026-11-04T16:30-05:00 with `/proc/<pid>/io`:
  the branch-write round 5 and claim-watch adds had each read about 52 GB and written about 49.5 GB,
  three ran at once,
  two waited in `blk_mq_get_tag`,
  and `/proc/pressure/io` full avg300 was 22%.
  The same add under `.claude/worktrees/` was a snapshot in seconds (trace worktree `cgs-trace-892d1fa0`).
  The trace script and new briefs now put worktrees under `.claude/worktrees/`.
  Measured with `du` (`~/temp/agent/worktree-copy-sizes.txt`, 1 KiB blocks):
  `package/ide/app/target` 40,582,652,
  `package/nested-wayland-session/cli/target` 14,144,844,
  `package/git-policy/cli.fuzz/target` 3,991,544,
  `node_modules` 3,256,040,
  `.cache` 52,148.
  Owner decisions,
  2026-11-04 (question tool,
  after asking what "wrapper fix" meant):
  on the copy,
  "No lefting out build outputs. The real fix is to make sure every build dir is always lean. That's not the wrapper's responsibility, but I'm assigning this to you to.";
  on an AGENTS.md rule for worktree placement,
  "No exposing underlying machinary to always-loaded context.";
  then,
  "(debuginfo level, incremental off for dependencies - no messing with these."
  and
  "other profile settings (split-debuginfo, strip, codegen-units) should be off-limits too".
  So the wrapper's copy stays,
  no AGENTS.md rule is added,
  and no build profile setting changes (debuginfo level,
  incremental,
  split-debuginfo,
  strip,
  codegen-units).
  Agent `a0cd49729007486f4` measures what each build directory holds,
  researches mechanisms that keep them lean without changing what a build produces,
  and writes `doc/planning/lean-build-dirs.md` with ranked options and a cross-vendor review;
  it deletes nothing;
  the owner picks before anything is built.
- One executable merged to main (`ab40458516`,
  branch tip `4a46ac09bc`,
  gate 4880/4880;
  main was ahead only by doc commits).
  It fixed main's red `lint:clippy` (`service_executable` unused) and the three main-only test failures that blocked the
  branch-write round 5 and claim-watch gates;
  `lint:clippy` on main after the merge exits 0.
  Both agents were told to merge main and re-gate;
  round 5 restores `service_executable` and regenerates its review bundle (`-v2`).
- Owner, 2026-11-04,
  on the source layout (after stopping the issue 6261 child session:
  "I told the layout subagent to stop because it couldn't ever produce something good enough. Lets do this ourselves."):
  "Principle 1: Lint issues need to be taken care of asap.
  Principle 2: every name of a dirent must be understandable to a monkey. If jargon is unavoidable, the file's top doc must explain everything, in a way that would make any monkey understand.
  Principle 3: Who depends on who must be immediately visible from looking at the file structure."
  and "Consider monkey someone who only knows what git commit and git push are."
  Then "Put the semantic line break findings on hold. They aren't the most serious issues right now."
  Question tool answers:
  TypeScript in the Rust package "A: port all to Rust (Recommended)";
  structure "B: numbered layers (Recommended)" with the note "Don't be afraid of using extremely long names. B, but make sure a monkey even just looking at one thing know what it means.";
  granularity "Files too, checked (Recommended)";
  scope "Repo-wide rules (Recommended)",
  written as AGENTS.md MKY,
  LYD and LYF (`b1955e0b49`,
  CLAUDE.md regenerated).
  Measured on main (`~/temp/agent/issue-6261/scc/`, comments and strings excluded):
  554 production modules,
  2,296 references between them,
  266 in one dependency loop (plus loops of 4 and 2);
  the best order found leaves 82 references pointing backwards (a heuristic upper bound, mostly inside one folder:
  15 in `landing/`,
  7 in `push/`,
  6 in `worktree/`).
  The flat glob re-exports in `lib.rs` (`pub use <folder>::*;`) have to go so every `use` names its folder.
  Lint counts on main before the fixes (`~/temp/agent/issue-6261/lint-now.log`, `mlint-now.log`):
  package `lint` had 1 clippy error and 6 oxlint errors with 14 warnings (now 0 errors and 1 pre-existing warning,
  `8f475bf9e9`);
  the repo linter reports 31,009 `markdown/semantic-line-breaks` (on hold),
  2,528 `rust/avoidable-closure` warnings,
  37 `rust/max-lines` and 3 `rust/tests-in-sidecars` errors.
  The package holds 142 TypeScript files (34,270 lines: 79 in `e2e/` from 2026-09-25,
  63 in `bin/`),
  29 of them TypeScript tests,
  all to be ported to Rust.
  Update 2026-11-04T18:40-05:00 (main `7663284896`):
  a group order with no upward reference is found and committed with its group map (`bb9d8f3505`,
  `doc/planning/cli-git-source-layout.md`, section "Group order found, from the bottom");
  a rescan of main after the branch-write merge gives the same result;
  27 loops between files inside groups remain (79 files).
  Branch-write round 5 is merged (`de36a92311`, gate 4927/4927 at `258e0a2e14`).
  The three `rust/tests-in-sidecars` errors are fixed on main (`7663284896`):
  the inline test-only helper modules moved to `*_test_support.rs` files under the same module names.
  Four lint agents work from `7663284896`, one disjoint folder set each, on `rust/avoidable-closure` and
  `rust/max-lines` (brief `~/temp/agent/issue-6261/lint-brief.md`):
  `refactor/git-policy-cli-lint-worktree-folders` (worktree, worktree_copying),
  `refactor/git-policy-cli-lint-service-landing-and-runner-folders` (landing, service, transaction, invocation,
  process, foundation),
  `refactor/git-policy-cli-lint-reader-push-and-lock-folders` (repo_read, subvolume, push, index, lock),
  `refactor/git-policy-cli-lint-command-and-policy-folders` (every other folder).
  Oversized files are split by these agents, with each folder's file-loop list so a split breaks a loop rather than
  adding one; the layout pass renames and moves files afterwards.
  The coordinator merges the four branches and gates the result once.
  The owner then added AGENTS.md VGW (no vague word such as `native` alone in a name; "helper_<specific> is fine")
  and answered the layout questions: plain-sentence folder names (the draft in
  `doc/planning/cli-git-source-layout.md`, `src/native/` gone), "hand back 'run this next'" in place of the two
  in-process recursions, and all four smaller call changes.
  A fifth agent builds the LYD and LYF checks into `monochromatic-lint`, off by default
  (branch `feat/monochromatic-lint-folder-uses-and-file-loops`, brief `~/temp/agent/issue-6261/layout-lint-brief.md`).
  Update 2026-11-05 (main `1942f0de53`): the fifth agent's rules `rust/folder-uses-links` and
  `rust/file-order-in-folder` are merged to main (78 layout tests in the container, 810 native),
  off unless a config block selects them; measured on the wrapper at `8c9eb6e035`: 364 findings
  (288 missing `uses/` links, 27 glob re-exports, 1 folder loop, 21 file loops, 27 top docs without a file list).
  Open to veto from that agent: `mod.rs` and other crate roots are left out of the in-folder loop check.
  Asked the owner async on 2026-11-05 (owner: ask "async", not with the question tool), pick adopted at once:
  index files (`mod.rs`, `lib.rs` as index) hold only the top doc, `mod` lines and re-exports, anything else is a
  finding; a name reached through a re-export counts as a use of the file that defines it.
  An agent builds it on `feat/monochromatic-lint-index-files-hold-only-declarations`
  (worktree `.claude/worktrees/lint-index-files-only-declarations`, from `bec267799f`); the owner's veto stays open.
  Merged to main 2026-11-05 (840 tests, 108 in the container). A use through a re-export counts for both the
  re-exporting file and the defining one (replacing the edge lost a real 15-file loop in `landing/`).
  The entry-point check for `main.rs` beside `lib.rs` and `src/bin/*.rs` is merged to main too (842 tests,
  110 in the container); repo-wide survey: 74 entry-point and 107 index-file findings outside the wrapper, none in it.
  The `exclude = ["**/uses"]` for the wrapper's `Cargo.toml` is still to add through file-enforcer.
  The worktree lint branch is done (`refactor/git-policy-cli-lint-worktree-folders` at `8d19c13a28`):
  `worktree/` and `worktree_copying/` have zero `rust/avoidable-closure` and `rust/max-lines` findings,
  11 closure statements carry an `expect` documented in
  `src/native/worktree/worktree_closure_lint_suppressions.md`, host tests 582 passed, clippy clean,
  no container gate run; it waits for the other three branches, then one merge and one container gate.
  The reader, push and lock branch is done too (`refactor/git-policy-cli-lint-reader-push-and-lock-folders` at
  `0b422a9115`): zero `rust/avoidable-closure` and `rust/max-lines` in repo_read, subvolume, push, index and lock;
  19 `expect`s in 11 files, documented in `package/git-policy/cli/doc/closure-lint-suppressions-reader-push-lock-subvolume-index.md`;
  host tests 1178 passed, clippy clean, no container gate.
  Open from that branch: one run of `lock::` and `push::` failed three
  `landing::service_index_lock::claim_tests::*` tests ("took the whole ceiling") that passed on rerun;
  the agent called it host load without measuring, which the repository treats as a contention or ordering defect
  until measured; the merged gate must check these three, and a failure there is a defect to find.
  Its seam closures (`dyn Fn` parameters in subvolume, push and `start_retry`) stay suppressed;
  turning them into traits changes signatures used by `worktree/worktree_clone.rs` and test sidecars.
  The command and policy branch is done too (`refactor/git-policy-cli-lint-command-and-policy-folders` at
  `0595efcd4b`): zero findings of both rules in its 17 folders, 10 `expect`s documented in
  `doc/planning/cli-git-closure-lint-suppressions.md`, 1507 host tests passed with the same test names as before,
  clippy clean, no container gate.
  All three finished branches are merged into `refactor/git-policy-cli-lint-all-folders`
  (worktree `.claude/worktrees/lint-all-folders`, from main `bec267799f`) without conflicts;
  clippy passed after the first two.
  Next, after the service, landing and runner branch lands: merge it, one container gate, merge to main.
  Update 2026-11-05: that branch is done too (`refactor/git-policy-cli-lint-service-landing-and-runner-folders` at
  `c97766b11a`; 662 closure and 12 max-lines findings to 0; 17 `expect`s; lib 4248 passed, binary 678 passed,
  no container gate). All four branches and `main` are merged into the integration branch, plus the probe-test move
  (`5983d1603d`) and the `repo_read_hook` import that breaks its loop with `git_metadata` and `transaction_git`
  (`a8b41b0d62`); clippy is clean there.
  The container gate runs on `a8b41b0d62` as unit `git-policy-cli-lint-all-folders-gate`
  (log `~/temp/agent/issue-6261/lint-all-folders-gate.log`).
  The suppression pass runs in parallel from the same commit on `refactor/git-policy-cli-fewer-closure-suppressions`
  (worktree `.claude/worktrees/fewer-closure-suppressions`), gated on its own after it lands.
  Gate result on `a8b41b0d62`: `4930 tests run: 4930 passed (1 slow), 8 skipped` in the container
  (the slow one is `push_slow_server_tests::a_push_over_http_waits_past_the_old_low_speed_cutoff_and_lands`, which
  waits past a 60 s cutoff by design); all 27 `landing::service_index_lock::claim_tests` passed.
  The earlier one-time host failure of three of them stays unexplained; it is still open,
  to be measured if it recurs.
  Merged to main 2026-11-05 (483 files). The five lint worktrees are removed; their branches stay.
  Layout next: an agent drafts the whole target tree on main `977e697ad5` (rescan, crate split, every folder and
  file old to new with `uses/` lists, every change the moves need, move order, cross-vendor review) into
  `doc/planning/cli-git-source-layout-target-tree.md`; nothing moves until the owner has seen it.
  Suppression pass done on `refactor/git-policy-cli-fewer-closure-suppressions` (`cbaa95e9b9`, main merged in at
  `9829d05863`): `expect(avoidable_closure)` sites 56 to 3, the three in `src/native/foundation/`
  (`side_by_side_threads.rs` for `std::thread::scope`, `scoped_start.rs` for `spawn_scoped`,
  `call_catching_panics.rs` for `catch_unwind`); callees take traits on named structs, guards, or an enum instead;
  blanket closure impls only in test sidecars; lib 4259 and binary 678 passed on the host; 8 new tests, none renamed.
  The four suppression notes become one, `package/git-policy/cli/doc/closure-lint-suppressions.md`
  (moved 2026-11-05 from `src/native/worktree/worktree_closure_lint_suppressions.md`,
  `doc/closure-lint-suppressions-reader-push-lock-subvolume-index.md`,
  `doc/closure-lint-suppressions-service-landing-and-runner-folders.md`
  and `doc/planning/cli-git-closure-lint-suppressions.md`, which are deleted on that branch).
  Planted controls: of the 46 in `bin/native-planted-controls-{commit,entry,landing}.ts`, 3 had a `from` line
  that matched nothing before the lint merge (checked at `7663284896`) and 7 after it, on main and on the branch
  (checker `~/temp/agent/issue-6261/planted-from-lines.ts`); an agent repoints the seven on the suppression branch,
  proving each fails its test when planted. Then: container gate on that branch, `native:mutation:planted` over
  its image, merge to main.
  Correction: two of the seven were checker artifacts (it could not read a `from` written as an array `.join` or
  `String.raw`); the real breaks were one before the lint merge and four through it, and two controls named
  binary tests with a wrong folder prefix (from `9533c30e80`) so they could never count as noticed.
  All fixed at `a7e32e118e` (48 controls, 0 problems; each planted and seen failing on the host).
  Container gate running on `a7e32e118e` as unit `git-policy-cli-fewer-closure-suppressions-gate`.
  Target tree drafted and reviewed cross-vendor (`doc/planning/cli-git-source-layout-target-tree.md` and
  `-files.md`, last `0912bd4661`); its owner questions are in `doc/handover/questions-for-the-owner.md`.
  The layout rule's file-list marker now reads across a line break (branch
  `fix/monochromatic-lint-file-list-marker-may-wrap` at `dd600cb33b`, merged to main; 854 tests, 122 in the
  container), because `markdown/semantic-line-breaks` demands a break after its comma.
  Suppression branch gate on `a7e32e118e`: `4938 tests run: 4938 passed (1 slow), 8 skipped`;
  planted controls over its image run as unit `git-policy-cli-planted-controls-after-suppressions`
  (log `~/temp/agent/issue-6261/planted-controls-after-suppressions.log`); merge after they pass.
  Next for the layout:
  break the file loops with named changes,
  split the large groups into folders,
  name every folder and file plainly with its `uses/` list,
  rescan after the lint branches land,
  and put the whole tree before the owner before anything moves.
  The fuzz crate `package/git-policy/cli.fuzz` does not compile on main:
  `src/owners.rs` imports `git_policy_cli::owner_lock_record`, deleted in `ee30d244f6`
  ("worktree add takes no settlement lock", 2026-11-03T02:36:31-05:00); no gate compiles that crate, so nothing
  noticed. The crate split changes every `git_policy_cli::` import it uses, so it is fixed as part of the split:
  each move keeps it compiling, and its compile joins the container gate at the first move.
- Program rename (task #165; plan `doc/planning/git-policy-cli-program-rename.md`):
  step 1 built on `feat/git-policy-cli-program-rename` (worktree `.claude/worktrees/git-policy-cli-program-rename`,
  tip `9dbcb5ce92` with 22 commits on `a7e32e118e`; host lib 4305 and binary 690 passed, clippy clean,
  planted checker 48/0; container gate not yet run).
  Two follow-ups sent back to its agent at 2026-11-05T03:58:47Z:
  (A) the root settings file keeps the name `cli-git.config.jsonc` until step 3, because the installed old program
  reads only that name (the branch's `f781971973` renamed it, and four later commits on the branch ran with no
  policies);
  (B) an open batch an old helper left in the old service state folder must still land after the switch.
  A done at `0c840d0e17` (root file restored in `d513d1cadd`; host lib 4306 and binary 690 passed, clippy clean,
  planted checker 48/0; the installed wrapper's `check` over the paths changed since `f781971973` found nothing).
  Container gate on `0c840d0e17` passed: `4997 tests run: 4997 passed (1 slow), 8 skipped`
  (unit `git-policy-cli-program-rename-gate`, worktree `.claude/worktrees/git-policy-cli-program-rename-gate`,
  log `~/temp/agent/issue-6261/program-rename-gate.log`).
  Final tip `cbb5b463e8` (main merged in as `dfa6798f1f`; 13 oxlint findings fixed by restructuring) passed the
  container gate: `5005 tests run: 5005 passed (1 slow), 8 skipped`.
  Merged to main as `ca6cd899b3` (2026-11-05T04:41:34Z), with the suppression branch it contains; per the owner
  ("Mutation runs don't block publish and release", "Treat reviews as async") the planted run and the cross-vendor
  review of B did not hold the merge; a miss or a finding is fixed forward.
  Review of B: two cross-vendor runs (10 files, then 4 files with the diff) each timed out after 3600 s with no
  output; a one-line control prompt answered `ok`, so the reviewer works and the input was too heavy.
  A third run over only the three fixed source files on main (846 lines) also timed out after 3600 s with no output
  (`~/temp/agent/issue-6261/sol-review-after-fixes/`). The fourth run answered (see the release blockers below) and
  its findings are fixed on `fix/git-policy-cli-earlier-helper-review-fixes`; no cross-vendor review of that fix has
  run. A same-vendor second pass
  (`claude-second-pass.md` beside it) found three P1s, no P0:
  the `/proc` look misses helpers started by units older than `9b360150c0` and the shared service;
  the adoption's `Drop` can remove the intent of a landing in flight, leaving a landed commit with no record;
  `current_value` reads a failed Git run as "branch not moved", which also leads to `Drop`.
  Fixed with red tests on `fix/git-policy-cli-earlier-helper-adoption`
  (worktree `.claude/worktrees/earlier-helper-adoption-fixes`, from main `35310097267f`, tip `c4b6812564`):
  all three P1s and the five P2s (unreadable process counts as "may be running", the `/proc` scan bounded by
  `PROCESS_LOOK_CEILING` 10 s after a measured 0.64 s for 899 processes, same-user filter, each adopted batch logged,
  earlier journals read-only); `current_value` now answers "no answer" on a Git failure for both the adoption and
  the service's own recovery. Host lib 4327, binary 691, clippy and oxlint clean.
  Container gate on `c4b6812564` passed: `5019 tests run: 5019 passed (1 slow), 8 skipped`; merged to main.
  The rename, hand-over-fix and gate worktrees are removed; the suppression worktree stays until its planted run ends.
  Still uncovered (plan doc, "Open batches of the old helper"): a helper in another PID namespace; an intent left on
  disk is never settled if the old units are removed; the Windows build of the new module was not compiled (owner:
  Windows paused).
  The rename note added to 117 old records said "is now" and sat in 6 `doc/decision/` records; on the owner's
  "Run the fix as described" (2026-11-07T02:03:01-05:00) `31f3223d25` removed it from the decision records and
  reworded it in the other 111.
  Release blockers at 2026-11-07T02:03:01-05:00 (owner: the one release goes out "After the test gap is fixed"),
  all four fixed on `fix/git-policy-cli-earlier-helper-review-fixes` (worktree
  `.claude/worktrees/earlier-helper-review-fixes`; each with a red test committed first), recorded 2026-11-07T03:01:27-05:00:
  - The fourth cross-vendor run (`~/temp/agent/issue-6261/sol-review-retry-4/review.txt`) found three defects:
    (1) an old helper can start between the `/proc` scan and the adoption.
    The helper installed here (tag `monochromatic-git-policy-cli-v0.5.34`) holds no lock on its journal
    (`service_main.rs`), so no exclusion both sides honour exists; the adoption now defers while a socket of the earlier
    units listens (`/proc/net/unix`, read before the processes), and deferred batches stay on disk until the earlier helper or
    the next start of this service settles them (plan doc, "Changes after the fourth review");
    (2) journal arguments are compared by device and inode, relative ones in the process's working directory, and an
    unresolvable one means "may be running";
    (3) a command line longer than the bytes read means "may be running".
  - The planted control "worktree copy installation batches" went unnoticed because the batch size grew from 512 to 4096
    in `f538c41379` (2026-11-02) while the test kept 1,100 files, so one batch held the whole tree; the closure-lint
    refactor did not touch the split. The test now installs two full batches plus one file, derived from the constant.
    Rerun over the fixed image: that control and the six the shutdown never reached (the log reached 42 of 48 with 41 noticed, not the 38 and 37 first recorded)
    were all noticed. The `fewer-closure-suppressions` worktree can be removed.
  Fifth cross-vendor run (`~/temp/agent/issue-6261/sol-review-fix-5/review.txt`, recorded 2026-11-07): the command line and
  folder identity fixes hold; one P1 left, the old-release wrapper that starts the old helper between the look and the
  adoption. Owner pick "Stop the old socket" built on `fix/git-policy-cli-stop-earlier-helper-socket` (worktree
  `.claude/worktrees/stop-earlier-helper-socket`): the commit that sets up the service also stops the same worktree's
  `cli-git@<instance>.socket` (same instance text; job mode `ignore-dependencies`; a running old helper is not stopped; the
  socket file is never unlinked; failures are logged and never fail a commit). Masking that unit would make an old-release
  wrapper exit 1 on its commit (tag `service_commit.rs` 386 to 388), so it was not built. Owner pick "Off, settle on request"
  (2026-11-07): the service has no adoption path or option; `git monochromatic-git-policy-cli service settle-earlier-batches`
  finishes the worktree's leftover batches when asked, after the look passes, saying first that it is safe only when no
  old-release wrapper is in use in that worktree; `doctor` reports the count. Sixth review (`sol-review-socket-6/review.txt`):
  both findings fixed (option removed; no unlink, retry read from `/proc/net/unix`). Cost while an old helper runs: about one
  millisecond per commit. Plan doc, "Changes after the fifth review" and "Changes after the sixth review".
  Seventh cross-vendor run (`sol-review-settle-7/review.txt`, 2026-11-07): three defects in the settle command, fixed on
  `fix/git-policy-cli-settle-command-review-fixes` (condition write failure refuses; unexamined batches are named and exit 1;
  the hold keeps which socket or helper caused it and the remedy fits; plan doc, "Changes after the seventh review").
  The rename audit (plan doc, "Earlier names after the rename audit") fixed three names that had been renamed along with the
  program but name the earlier program's artifacts: `LEGACY_SOCKET_RELATIVE` (the single service's socket), the first lines of
  the earlier push records, and the first line of the earlier claim files. Open for the owner: the earlier `cli-git-scratch`
  folders in Git directories are not moved or swept.
  After the one release, every later release carries `-alpha.N` and skips crates.io, but this machine installs our
  seven tools through crates.io (`cargo-binstall`); until they install from GitHub releases (the `archive` route
  `mpm-shadow` uses, task #167), no later fix can be installed here. The owner has not yet said whether that route
  change comes right after the release or after layout and lint.
  B done at `7fe6ac14ff`: `landing/earlier_service_journal_adopted.rs` finishes, at service start, batches left open
  in the old journal (`<state>/cli-git/service/<instance>` and the shared `<state>/cli-git/service`) when no service
  of that instance shows in `/proc`, and counts their finished identities as landed (host lib 4313, binary 691).
  Not covered: a helper in another PID namespace; the old helper restarted by its unit during the adoption.
  Cross-vendor review of B running (`~/temp/agent/issue-6261/sol-review-part-b/review.txt`).
  oxlint on the branch: 10 warnings and 3 errors in `bin/` files A touched (main is at 0 since `995cab8d1c`);
  sent back to the agent with a main merge; then the container gate reruns on the final tip.
  Release and lock (steps 2 and 3) wait for the owner's release-timing answer.
  Step 3 also renames the root settings file and the link folder (to `.cache/monochromatic-git-policy-cli-bin`).
- crates.io footprint (owner 2026-11-05: "Can we avoid polluting crates.io too much?"):
  measured and ranked in `doc/planning/crates-io-footprint.md` (`dd61b20559`), owner answers recorded there.
  Decided by the owner: "One release each. And we publish less to crates.io".
  Research done: `doc/research/cargo-crates-and-tools-without-crates-io.md` (option 1 works; floating git and
  GitHub Packages do not; a static index on GitHub Pages would) and `doc/research/tests-inside-published-rust-crates.md`
  (distributions and crater run `cargo test` from a published crate); summary in the planning doc,
  section "Research results (2026-11-05)".
  Owner questions open in `doc/handover/questions-for-the-owner.md` (kept open in Zed): where the libraries go,
  and when the one release of each crate happens.
  Chosen by the session (non-destructive, owner's likely pick): tools move to the `archive` route and
  `publish = false` after their one release, not started until layout and lint land; test files stay in.
  The wrapper's crate with test files left out packs and verifies (1.6 MiB compressed against 3.1 MiB for the
  published 0.5.34; details in the planning doc); the throwaway worktree is removed.
  The one release of each crate is held until the route research is answered by the owner and,
  for the wrapper and the crates it pins, until the four lint branches are merged and gated,
  so all of them ship in one batch; the automatic yank step (task #166) is on hold for the same answer.
- Issue 6261 reopened by the owner at 2026-11-04T21:30:23Z
  (https://github.com/Aquaticats/Monochromatic/issues/6261#issuecomment-6642606105):
  "The current layout is still completely unacceptable and far from enough to consider this issue complete."
  The five agent slots were full,
  so a child session (`spawn-claude`,
  spawn id `7ee83419-2ce0-4ac8-a4f7-36f0731b095c`,
  brief `~/temp/agent/issue-6261/brief.md`) writes the target layout proposal
  (`doc/planning/cli-git-source-layout.md`, a file-by-file mapping, the crate-root map, the placement guide,
  the `native` layer decision) and a cross-vendor review,
  reports in `~/temp/agent/issue-6261/report.md`,
  and waits:
  the owner sees the target tree before any file moves,
  then a re-runnable move tool applies it on the latest main.

Git wrapper 0.5.33 is installed:
branch-write round 2 (`6fc72e8abe`,
gate r7 4839/4839 with main,
`native:clippy` on main clean),
release run 41896129005,
publish run 41896943949 built from `6a922adfee`,
crates.io 2026-11-04T19:28:15Z;
`upgrade-wrapper.ts 0.5.33`:
the 0.5.32 service retired on the client's request,
0.5.33 started (protocol 7),
and the sample commit on main landed in 1980 ms in all,
1271 ms of it provisioning the new service (landing 34 ms,
lock hold 31 ms).

cargo-shadow 0.1.1:
round 17 merged to main (`a439de1578`;
branch tip `a3c703025d`,
release gate 618/618,
clippy clean;
launcher fix:
a PATH candidate whose `<launcher>.launch` record leads back to the shadow is skipped,
chains up to 8 hops;
the live check behind a copy of the real launcher ran the shadow once per command);
dispatched as run 41900374975,
`release:wait` following;
then relock (`toolchain:update -- cargo:monochromatic-cargo-shadow` after restoring the plan row from `efe10d9745`),
install,
and the launcher check before the `cargo` link stays.
The branch's auto-push was refused once with GH009 (`doc/troubleshooting/github-lfs-integrity-rejection.md`);
a later push reached origin (`git ls-remote` shows `a3c703025d`).

Agents and reviews at 2026-11-04T15:05:00-05:00:
- Branch-write round 3 (`aa0fee0c2cf70bb31`):
  owner pick B for foreign-owner patience plus every round-2 review finding.
- Models (`a9c775c9196f1d836`):
  step-1 review findings fixed and step 2 built as callables (`43fbb6041f`;
  `native:test:model` 543 passed;
  nothing in `git commit` calls it;
  the shipped service only gains `--flush-lane`);
  the decision-22 route is `systemd-run --user`,
  recorded as within the decision ("Systemd unit + fallback (Recommended)";
  `systemd-run` creates "a transient .service or .scope unit");
  now closing the step-2 leftovers (`WAIT_DRAIN` measured,
  forcing I/O errors,
  disposal crash injection,
  establishment record,
  spool and attempt durability).
  Step-2 reviews running:
  `commit-models-step2-sol-a.txt` (code and component tests),
  `commit-models-step2-sol-b.txt` (models and map).
- cargo-shadow third certification pass,
  seven groups in parallel (`run-cert3-reviews.ts`,
  outputs `cargo-shadow-cert3-sol-1.txt` to `-7.txt`);
  the agent (`ae0fbdf78c31bd757`) is idle until they return.
- e2e `sigkill-service-during-work` (`aca6e78c5ffbd5680`,
  branch `fix/git-policy-cli-e2e-sigkill-service-during-work`):
  diagnose whether the product or the scenario is wrong,
  fix,
  gate.

cargo-shadow 0.1.0 is done:
both proving runs passed (dry run 41893310452;
run 41893318423 authenticated through Trusted Publishing,
skipped the upload and created the immutable release `monochromatic-cargo-shadow-v0.1.0`),
and the consumer checks passed
(runbook section "monochromatic-cargo-shadow 0.1.0").
Owner action open:
revoke the `cargo login` token,
then `cargo logout`.

Update 2026-11-04T15:35:00-05:00:
cargo-shadow 0.1.1 is live as `cargo` for every session of this checkout
(lock `cac4766b11`,
link `.cache/cargo-shadow-bin/cargo` to the toolchain launcher).
Traced in the repository's own mise environment from a throwaway worktree:
`cargo --version` ran the link,
the shadow and the real cargo once each;
`cargo metadata` ran the shadow once and the real cargo three times;
the worktree stayed clean.
cargo-shadow round 18 merged to main (`22a3c5b812`;
hard-link identity skip,
`.done` completion barrier counted as mandatory,
`u128` kept-size sum);
its release gate on main failed:
`a_killed_wrapper_with_an_escaped_writer_stops_the_recovery` (fixture `ready` did not appear in 30 s)
and `a_descendant_is_stopped_with_its_parent_at_the_bound` (`pid never held a process id`),
both passing alone in 0.1 s and 0.5 s;
not attributed to load (owner:
"I don't see any load, and once again, load is not an excuse.");
the cargo-shadow agent is diagnosing the block;
0.1.2 waits for a green gate on main.
Fourth certification pass (eleven groups) running.
Branch-write round 3 merged (`e75a2549c9`) and released as 0.5.34 (run 41905476029);
its review returned four P1s (pruning withdrawal confirmations,
existing reflogs,
foreign refusal after mutations,
an unbounded receive on the review timeout path),
sent as round 4.

Before that:
cargo-shadow as `cargo` for every session was not live.
It was locked (`efe10d9745`) and installed,
then reverted (`65f5733aec`) and its link `.cache/cargo-shadow-bin/cargo` removed the same minute:
the install links `cargo` to the toolchain launcher (`.cache/toolchain-launchers/cargo-shadow`),
and through it the shadow runs itself once before the real cargo
(strace:
launcher,
shadow,
launcher again,
shadow again,
rustup proxy,
real cargo;
`cargo metadata` ran the shadow 4 times for 3 real cargo runs),
against "We should never invoke ourselves recursively.".
Started from its binary it ran once per command.
Sent to the cargo-shadow agent (`ae0fbdf78c31bd757`) as the first 0.1.1 fix,
the way pnpm-shadow 0.1.1 learned to skip its launcher;
then 0.1.1,
relock,
and the launcher check (`/var/home/user/temp/agent/cgs-recursion-check.ts <launcher> <throwaway worktree> <toolchain-bin>`)
before linking again.

Older:
0.5.28 was published to crates.io at 2026-11-04T03:09:07Z (publish run 41764751088,
built from `7b41c215ec`) and installed by `upgrade-wrapper.ts 0.5.28`
(`git --version`:
"monochromatic-git-policy-cli 0.5.28,
shadowing git at /usr/bin/git").
Its lock commit on main took 1743 ms in all,
1140 ms of it provisioning the new service (the 0.5.27 service refused the protocol-7 request line,
then retired after its batch);
the 0.5.28 service's landing took 44 ms (`lock_hold_ms=25`,
`sync_ms=22`).
`sync_ms` is not a disk sync:
it times removing the conclusion state and building and installing the new index (`service_timing.rs:84`);
queued:
rename it by its role.
History of the 0.5.28 release:
base gate `gpc-gate-base-0-5-28-final-20261103T214518-0500` passed 4706 of 4706 (7 skipped) on `d9e8907ae8`;
the base was merged into main as `920fdce60d` (2026-11-03T21:57:04-05:00,
body `Closes #6260`);
dispatched as run https://github.com/Aquaticats/Monochromatic/actions/runs/41764055048;
`native:release:wait` is following it;
next:
`upgrade-wrapper.ts 0.5.28`,
then time commits on main against native Git.
Before 0.5.28:
0.5.27 was dispatched 2026-11-03T10:28-05:00 from main after its gate passed (4264 of 4264,
6 skipped),
run https://github.com/Aquaticats/Monochromatic/actions/runs/41663034226,
published to crates.io at 2026-11-03T15:44:38Z,
and installed by `upgrade-wrapper.ts 0.5.27`.
Its lock commit on main took 1625 ms in all,
of which 1169 ms was provisioning the new service (the 0.5.26 service retired after its batch);
the service's landing took 49 ms (`lock_hold_ms=28`,
`sync_ms=25`).
0.5.26 was published to crates.io at 2026-11-03T13:21:01Z.

### Unmerged branches and worktrees of this effort

Tips were read from `git for-each-ref` at 2026-11-03T08:16:08-05:00 and keep moving while the owning agents work.

- `fix/git-policy-cli-commits-never-fail` `f160209787`,
  worktree `.claude/worktrees/commits-never-fail`.
- `fix/git-policy-cli-plain-commit-finishes-conflicted-merge` `a06af6102c` (finished),
  worktree `.claude/worktrees/plain-commit-conflicted`.
- `fix/git-policy-cli-no-inner-quiet-and-slow-server-waits` `e38a7dea6a` (finished),
  worktree `.claude/worktrees/no-inner-quiet-slow-server`.
- `feat/git-policy-cli-word-level-second-merge-pass` `8cc36cd4b6`,
  fifteen commits ahead of the merge base,
  worktrees `.claude/worktrees/word-level-second-merge-pass` and the gate copy
  `.claude/worktrees/gate-word-level-merge-pass`.
- `feat/cargo-shadow-cli` `504a82aa3f`,
  worktree `.claude/worktrees/cargo-shadow`,
  fifty commits ahead,
  all on origin.
- `worktree-agent-ab3dd6294f9d91284` `fe2081f2a1`,
  the copy-in-service design that was never merged (see the P1 section).
- `e2e-trace-diagnosis-20261101` `a0126e571c`,
  unmerged and kept on purpose.
- `integrate/git-policy-cli-deadline-stop-and-conversion-real-git` `2ac6da873e` is the stopped first 0.5.21 candidate;
  both of its parts shipped by other routes,
  so it is stale and can be deleted once the owner agrees to branch deletions.

Many detached worktrees under `.claude/worktrees/` (`integrate-0NN`,
`gate-*`,
`survivors-*` and others) remain from merged work;
their files and mutation evidence were kept deliberately,
and removing them is open (see "Housekeeping").

### Agents and gates running

At 2026-11-04T03:27:12-05:00 (the owner's cap is five delegate agents at once),
two delegate agents and four cross-vendor reviews running:
- Design revision 16,
  `fix/git-policy-cli-commits-never-fail` (from revision 15,
  `4a6ad9d34a`):
  records the owner's revision-15 answers (in "Owner decisions in force") as decisions 15 to 18
  and answers the fifteenth review (`commits-never-fail-sol-15.txt`,
  R15-1 to R15-8;
  protocol "approve continued model work only",
  questions "accurate after named edits").
  Two of its findings bear on owner rules:
  R15-1,
  accurate attribution of the post-success exception is not yet shown to be the owner's approval;
  R15-6,
  start-only materialization adds an unbounded foreground phase the owner never approved.
  Done:
  tip `f8d83c760f` (2026-11-04T03:30:19-05:00);
  R15-2 to R15-5 closed,
  R15-6 closed by mechanism (only a terminating,
  size-limited,
  child-free class runs inline),
  R15-7 a unit configuration obligation (stop timeout infinity,
  drain on upgrade),
  R15-1 narrowed to owner question 8 (a lost success receipt and a replay after a power loss;
  recommended B,
  force the receipt before success only when no service runs)
  in `commits-never-fail-questions-f8d83c760f.md`;
  not asked until review 16 checks it.
  The design is 254 KB (history moved to `commits-never-fail-history-f8d83c760f.md`).
- Models round 11,
  `feat/git-policy-cli-commit-protocol-models` (from `ffc98d1eae`,
  293 of 293):
  merges main (the subsystem folders of #6261),
  puts the `commit_*` component files in `commit/` and the `model_*` files in a new `model/` folder,
  fixes components review 7 ("Not correct"),
  models review 5a ("sound after named fixes") and 5b ("Not sound":
  the discharge green is tautological and its checkpoint rule is unsafe),
  applies decisions 14 to 18,
  and adds the fifteenth review's model controls.
  Done in part:
  tip `57c0fb3c9b`,
  319 of 319,
  both clippy tasks clean;
  main merged (`eaacff58bc`;
  components in `native/commit/`,
  models in `native/model/`,
  the `native_models` test root at `model/mod.rs`);
  decision 14 as `Rules::Fifteen` in the barrier and register models (barrier stage exempt from the wait bound,
  `BARRIER_ENTRIES` FOREGROUND,
  COMMITTING,
  DEPART,
  STOP);
  decisions 16 to 18 in the watch model (its one reachable violation is the accepted 7a concession);
  discharge rebuilt as a crash-state machine (`UnsafeCheckpoint` red,
  `Fifteen` green;
  R2 the only live dependency modelled);
  components review 7 items 2.1,
  2.2,
  3.1,
  3.2,
  3.4,
  4.1,
  4.3,
  4.4,
  4.6,
  4.7 and a `Departure` order.
  Not done:
  models 5a #2A,
  #3,
  #4,
  #5,
  #6,
  #8;
  the 5b pipe,
  T1 and merge items;
  components 2.4,
  3.3,
  4.2,
  unknown-version preservation,
  the wall-clock replay test;
  fifteenth-review controls 1,
  3 to 6,
  8 to 12 and 16.
  The component tests were written with their fixes and not run against the old code.
  Reviews:
  `commit-components-sol-8.txt` "Not correct" (checkpoint dependencies are paths,
  not validated replacement-root authority,
  and their ancestry is not made durable;
  a candidate is recovery-authoritative before its dependencies are forced;
  departure is a caller-assembled flush list;
  quarantine is an unowned destructive mutation;
  selection races reclamation;
  "Non-reclaiming component and model work can continue");
  `commit-models-sol-6a.txt` "sound after named fixes" (the register has an unchecked route around its barrier stages;
  final forcing cannot repair a failed disposition creation;
  planned shutdown can still end a protected hook);
  `commit-models-sol-6b.txt` "Not sound" (a checkpoint can erase a still-live R1 and the invariant accepts it).
  Design revision 17 done:
  tip `34a3c22f2f` (2026-11-04T04:14:48-05:00),
  design 244 KB;
  R16a-1 to R16a-6 and R16a-8 to R16a-10 closed in the rule,
  R16a-7 narrowed (receipts queued for a flusher;
  with no service a detached one-shot flusher through the manager,
  else a detached child;
  if neither starts the command exits success saying the receipt is not yet forced);
  components 2.1 to 2.5 and 3.x and the models' checkpoint finding closed in the rule,
  open in the component;
  question 8 rewritten (recommends A,
  accept the residual window;
  option B dropped) in `commits-never-fail-questions-34a3c22f2f.md`.
  Review 16b failed at 2026-11-04T04:28:19-05:00 after 57 minutes ("OpenAI Responses stream ended before a terminal response event",
  input 258 KB);
  not rerun,
  since 17b reads the whole revision 17 design that replaces revision 16.
  If 17b (250 KB) fails the same way,
  split the design into two halves for review.
  Review 17a (`commits-never-fail-sol-17a.txt`):
  R16a closure "not closed";
  question 8 "accurate after named edits" (R17a-10,
  ten points);
  P1s R17a-1 (receipt forcing at L4 brings back a foreground wait),
  R17a-2 (a running service or a started child is not yet durable responsibility),
  R17a-3 (detached flushers need descriptor,
  ownership and resource-lifetime rules),
  R17a-4 (equal content in some predecessor is not enough support),
  R17a-5 (optional verification cannot become unprotected by declaration).
  Review 17b (the whole design) running.
  Revisions 12 to 17 each drew new P1s;
  the next round narrows its scope before it starts.
  Review 17c (`commits-never-fail-sol-17c.txt`,
  the convergence question):
  R17a-1,
  R17a-2 and R17a-4 are rule gaps;
  R17a-3 and R17a-5 mixed (rule parts:
  the detached flusher's survival and handoff contract;
  abandoned-worker late-effect fencing and the blanket slow-I/O exemption);
  "Yes:
  a finite,
  conditional acceptance contract is possible",
  given as properties and controls by transition group,
  with "A new counterexample may reopen one of these properties;
  it should not silently introduce a new acceptance criterion.";
  the three literal rule conflicts (a foreign lock that never frees,
  a departure sync that never returns,
  a power cut before the only copy is durable) are already ranked by decisions 4,
  5 and 14 and the outside-cause list,
  so no new owner question.
  Revision 18 done:
  tip `21b8042205` (2026-11-04T04:55:40-05:00),
  design 246 KB (the first ten reviews moved to the new tracked `doc/planning/cli-git-commits-never-fail-review-history-archive.md`);
  the five rule gaps fixed (L4 queues receipts without forcing;
  seal protocol with `lane/sealed`,
  a doorbell and a final sweep;
  a one-shot flusher from an internal entry point under `lane/flusher.lock`;
  support as the fold of live `install` entries in tail order;
  the command creates the side ref,
  the helper only reads and flushes);
  "Acceptance target" and "Rule conflicts and the decisions that rank them" sections added;
  question 8 rewritten (`commits-never-fail-questions-21b8042205.md`,
  recommends A,
  ranking A > C > B labelled a policy preference).
  R17a-6 to R17a-9 untouched on purpose.
  Scoped review running (`commits-never-fail-sol-18a.txt`:
  the five gaps,
  the copied target,
  the flusher against the recursion rule and the lock rule,
  question 8).
  Models round 13 started (the models agent resumed),
  building to the acceptance target with a coverage map.
  Review 18a (`commits-never-fail-sol-18a.txt`):
  "Rule gaps:
  closed after named fixes" (the first design verdict since revision 12 with no new P1;
  R17a-1 and R17a-4 closed,
  R17a-3 closed subject to R17a-2,
  R17a-2 needs recurring enumeration independent of the doorbell,
  R17a-5 needs level 2 disabled on that path or bounded whole);
  the copied acceptance target "substantively preserved";
  "Internal helper launch:
  no inherent recursion conflict";
  "`lane/flusher.lock`:
  no demonstrated violation";
  question 8 "accurate after named edits".
  Coordinator's pick (veto open):
  disable level 2 on that path.
  Revision 19 started (the two named fixes and the question 8 edits only).
  Models round 12 started (the models agent resumed):
  the P1s of components 8,
  models 6a and 6b,
  built on revision 17's rules;
  P2 and P3 queued for round 13.
- cargo-shadow sixth review,
  four parts (`cargo-shadow-sol-6a.txt`,
  `-6b.txt`,
  `-6b2.txt`,
  `-6c.txt`),
  of round 6 (tip `d206fd5321`,
  472 of 472);
  round 6 is described in "Cargo shadow and the Rust catalog tools".

Done since the last list:
- Issue #6261 (subsystem folders):
  merged into main as `fec77945dd` (parents `5435980375` and `ac5414810a`);
  the issue closed at 2026-11-04T08:22:56Z through `186e264e06`,
  and main matches origin.
  29 subsystem folders;
  `lib.rs` re-exports each folder (`pub mod <folder>; pub use <folder>::*;`),
  so old paths keep working.
  Removing the `native` layer is deferred (option A,
  coordinator's pick under the delegation,
  veto open).
- Issue #6260 (nested-aware source scans):
  shipped in 0.5.28 and closed at 2026-11-04T02:57:10Z.
- 0.5.28:
  see "Release in flight";
  its measurements are in `doc/planning/cli-git-commit-latency-under-disk-pressure.md`.
  It carries the batch-number fix (`fix/git-policy-cli-service-batch-numbers-unique` tip `9c3d279e1a`,
  merged as `d9e8907ae8`):
  services sharing a journal (development runs and the test harness;
  production gives each instance its own journal) could give two landings one number,
  and one landing's records then overwrote the other's;
  each number is now claimed with an `O_EXCL` claim file.
- Design revision 15 (`4a6ad9d34a`) and its review:
  the owner answered questions 5,
  7a,
  7b and 7c (all recommended).
- cargo-shadow fifth review:
  all four parts "not correct";
  round 6 answered it.
- Every wait has a bound:
  fast-forwarded into the 0.5.28 base (tip `f89fa63f10`).
  Step ceilings with measurements:
  `PLUMBING_STEP_CEILING` 120 s (longest read step of 1,651 landings 9,663 ms),
  `OBJECT_STEP_CEILING` 600 s (longest client preparation 250,280 ms),
  `REFERENCE_STEP_CEILING` 60 s (longest of 1,651 reference updates 3,438 ms);
  a reference update ended at its ceiling is followed by a read of the branch:
  moved means landed,
  not moved means withdrawn as at `LOCK_CEILING`.
  The push stall prints that the push is queued (not yet seen through a real service).
  The coordinator's picks on its option sets are in `doc/planning/cli-git-no-unbounded-waits.md`,
  "Picks" (veto open).
- Index writer round 5,
  `fix/git-policy-cli-index-recovery-never-overwrites-staging` tip `b47467ebd1`,
  gate 4641 of 4641:
  a restart replays a batch only if the live index is still the very file the landing saw (inode,
  size,
  times,
  checksum tail);
  otherwise it resets nothing and logs a notice.
  Visible change:
  any rewrite of the index between a failed install and the restart,
  even a `git status` refresh,
  leaves the commit's paths un-updated in the index (nothing is lost).
  Not closed:
  the `HEAD` race after the last check (the same window native Git has);
  physical-index reclamation is mitigated,
  with options A (one reclaimer per physical alternate index:
  refuse to finish an index outside the instance's own Git directory) > B (document);
  coordinator's pick A under the delegation,
  veto open,
  queued after the integration.
  The weaker options it offered (per-path replay,
  content-only witness) are not taken:
  both discard ABA staging.
- Design revision 12 (`18ca87c579`) and the owner's four answers.
- Models round 7,
  `feat/git-policy-cli-commit-protocol-models` tip `7b87e79d3d`,
  188 of 188 model tests,
  both clippy tasks clean:
  components on rule V (`commit_inbox.rs`:
  one checksummed request file linked from `inbox/tmp/`,
  identity the hash of content and lane instance;
  `commit_state.rs`:
  `state.a`/`state.b` written in place without a sync);
  red test first counted the revision 11 path at exactly 22 syncs,
  the new waiting path makes 0,
  a departure syncs request file,
  inbox directory and newest state file in that order before returning.
  `LedgerOwner` is a non-blocking exclusive lock on `<ledger>.owner`
  (the standalone `repair_tail` and `append` are still public);
  amendments and gitlink changes are refused before acceptance and go to real Git (a test shows real Git commits them).
  The explorer gained `explore_eventually` (cycles with no exit).
  Not done:
  the order and ledger lines at the departure barrier (not built yet);
  F2,
  F1 reflog,
  F5,
  E1,
  persistence intermediate states,
  M1 compare-and-unlink;
  deferred to revision 13:
  the register's cumulative deadlines,
  the merge edit-loss invariant.
  The revision 11 binding,
  spool and record modules are unused by the new path;
  coordinator's pick (veto open):
  remove what revision 13 still leaves unused,
  after it lands.
  Reviews:
  `commit-components-sol-5.txt` "Not correct as submitted" (the state pair loses its checkpoint across successive unsynced transitions;
  the departure barrier omits state-name durability and the order and ledger lines;
  equal payloads get one identity),
  each answered by a revision 13 mechanism (state log,
  D-sets,
  invocation nonce) and sent to models round 8;
  `commit-models-sol-3a.txt` "sound after named fixes" (liveness semantics and truncation of `explore_eventually`;
  historical greens kept out of approval evidence);
  `commit-models-sol-3b.txt` "Not sound as verification of revision 12" (recovery still infallible,
  child progress owner-dependent,
  the register still assumes timeout completion;
  F7 `EEXIST` and F9 not retired by rule V).
  Both were sent to models round 8.
- Models round 8 (tip `8b31735dd3`,
  198 of 198,
  both clippy tasks clean),
  partly done before that agent ran out of room:
  `commit_state_log.rs` (hash-chained log,
  longest valid prefix,
  rotation safe at every step,
  0 syncs while waiting);
  `repair_tail` and `append` crate-private behind `LedgerOwner`;
  the departure forces request files,
  inbox directory,
  order file,
  state log and ledger with their directories;
  a 16-byte invocation nonce in each request (`REQUEST_VERSION` 4);
  `explore_eventually` stated as AG EF and truncation no longer makes violations.
  A fresh agent took the rest (models round 9,
  same branch):
  the W1 and register models rebuilt for revision 13 with fallible recovery,
  the twenty controls,
  F7 and F9 re-opened,
  the leftovers,
  historical greens labelled,
  components items 3.5 to 3.7 and removal of unused legacy modules.
- #6260 merged into the base.

Started before 2026-11-04T03:34:02-05:00 (the commit `372828457c` that records them) from main `05c16d73d2`,
each in its own worktree:
- Latency round 3,
  `perf/git-policy-cli-less-foreground-work`:
  the sections "Git's pack sync in `fast-import`",
  "Less work in the client and the landing",
  "The service unit and the host's swap" and "The queue and the next commit"
  of `doc/planning/cli-git-commit-latency-under-disk-pressure.md`,
  and the `sync_ms` rename.
  The plan's option B (loose objects in process) conflicts with `doc/decision/cli-git-concurrent-commits.md` lines 512 to 514
  ("Landing migrates the shadow's new objects into the real store as a pack kept with a `.keep` file
  until the compare-and-swap succeeds");
  the agent builds only an option that keeps that guarantee,
  else it writes an owner question.
- Index reclamation,
  `fix/git-policy-cli-one-reclaimer-per-physical-index` (tip `dd37f0c462` at its first report):
  the race shown red (`landing::service_index_lock::reclaim_tests::a_live_lock_taken_aside_by_a_late_reclaimer_is_never_open_to_a_third_program`,
  "a third program placed its lock while the live holder's lock was off its name");
  pick A reversed on its evidence:
  three call sites reclaim (`remove_stale` and `try_acquire` in `settle`,
  `wait_and_place` in the live landing),
  A covers only the restart,
  and extending it to the live landing makes a dead service's lock on an alternate index fail the commit at `LOCK_CEILING`.
  Coordinator's pick under the delegation (veto open):
  B,
  a non-blocking exclusive `flock` held only for the reclaim sequence
  (on the stale lock's own inode,
  or a sidecar,
  whichever the agent shows has fewer moving parts);
  options in `doc/planning/cli-git-one-reclaimer-per-index.md`.
  Built as B1 (the `flock` on the stale lock's own inode;
  ranking B1 > B2 > A > C):
  tip `9db8794047`,
  gate 4714 of 4714 (7 skipped);
  `reclaim_if` opens,
  reads the marker,
  takes `LOCK_EX|LOCK_NB`,
  checks device and inode against the name and unlinks;
  no `.aside-*` files any more.
  Left open:
  a person's `rm index.lock` between the inode check and the unlink;
  file systems without `flock` (reclaim unguarded,
  logged);
  reclaimers on different hosts;
  `.aside-*` files 0.5.28 already left are not swept (queued);
  `native:clippy:windows` not run (paused by the owner).
  Cross-vendor review (`one-reclaimer-sol-1.txt`):
  "correct after named fixes" (P1 a failed `flock` permits unlinking a live lock,
  through the no-`flock` fallback;
  P2 the `Busy` test does not show a wait;
  P3 the regression control does not probe the old aside window;
  "Can the lock be avoided?
  Not with an equivalent sequence of ordinary pathname operations under the current protocol.").
  Fix round sent to the agent,
  with the coordinator's pick (veto open):
  where `flock` is unavailable,
  never remove the lock;
  the landing waits as for a held lock (bounded by `LOCK_CEILING`) and its notice names the stale lock,
  the dead holder and how to remove it;
  plus the sweep of 0.5.28's `.aside-*` files.
  Merge into main after its gate.
  Fix round done (`de34a39194`,
  gate 4719 of 4719);
  its re-review (`one-reclaimer-sol-2.txt`) "correct after named fixes":
  the original three closed,
  two new P1s in the `.aside-*` sweep (an incomplete process view authorizes deleting a live aside;
  prefix matching does not show the file is disposable).
  Coordinator's pick (veto open):
  remove the sweep,
  document the leftovers and a manual removal ("Allow the rare leftover");
  plus the notice's evidence and deterministic `Busy` controls.
  Sent to the agent.
  The older index review's P2 and P3 findings (`private_index_path` aliasing,
  `head_names` revalidation,
  `Interrupted` writes) are already fixed on main,
  each with named tests.
- Test containers to CachyOS,
  `build/git-policy-cli-cachyos-test-containers`:
  `e2e/concurrent-commits.Containerfile` (Debian `node:24.11.0-slim`)
  and the `native:test:container` base,
  per `doc/decision/container-base-distro.md`.
- Design review 16 in two parts (`commits-never-fail-sol-16a.txt`:
  R15 closure,
  decisions 15 to 18 and question 8 from the diff;
  `-16b.txt`:
  the whole design),
  before question 8 is asked.

- Mutation campaign over what 0.5.28 changed (no recorded campaign covers it;
  161 product files and 14,935 changed lines between the `monochromatic-git-policy-cli-v0.5.27` and `-v0.5.28` tags,
  list in `~/temp/agent/mutation-scope-0528.txt`):
  gate of main (`690e0eb89d`,
  worktree `.claude/worktrees/mutation-main-0528`) running as `gpc-gate-mutation-main-0528-20261104T042223-0500`;
  then `native:mutation:parallel` with `GIT_POLICY_NATIVE_IMAGE_TAG` set to that unit's name,
  first landing,
  index,
  lock,
  push and invocation files,
  then the merge passes.

State at 2026-11-04T08:25:42-05:00 (after the Claude Code restart the owner asked for at about 07:30-05:00):
- Models round 14 done:
  `feat/git-policy-cli-commit-protocol-models` tip `6c4446d2de`;
  items 1 to 6 of `commit-models-sol-8.txt` section 4 done;
  full `native:test:model` 411 passed (`~/temp/agent/r14/full-model2.log`),
  both clippy variants clean;
  coverage map 70 covered,
  11 narrower,
  9 historical,
  17 not yet (107 rows);
  the author's findings for the reviewer:
  only the double-forked helper launch is built (no manager-started unit route,
  fork refusal not injected),
  the helper runs Rust after a bare `fork` (safe only single-threaded),
  Git experiments on one version and two backends,
  the reference-interpreter row's Covered is the author's call.
  Cross-vendor reviews returned:
  `commit-models-sol-9a.txt` "does not meet the target items it claims" (component authority,
  departure prefixes,
  `state_covers` succession,
  launch success on a failed second fork,
  the single-threaded fork precondition,
  helpers bounded only at the lock,
  a receipt joining after the error snapshot,
  an unforced directory entry counted as forced)
  and `commit-models-sol-9b.txt` "no, as submitted" (the receipt error handoff still atomic;
  two false-green decision 19 schedules:
  a notice left Scheduled after positive authority survives,
  and reclamation on an earlier attempt's delivery;
  map rows to mark narrower);
  both say no acceptance criterion is added.
  Round 15 running on both (same branch and worktree);
  it cuts `commit-models-10-part-*.diff` when done.
- cargo-shadow round 9 done:
  `feat/cargo-shadow-cli` tip `16249ffe94`;
  debug suite and release gate 561 passed,
  3 skipped (ignored helper entries other tests start),
  clippy clean after one rename;
  two stale contract names fixed;
  README and "Round 9" written;
  all 16 rows "shown".
  Cross-vendor reviews returned:
  `cargo-shadow-sol-9a.txt` "does not meet the list for this part" (D1:
  an unsettled scope upgraded by a later `exists()` check,
  in two schedules;
  D2:
  a malformed boot identity read as a reboot;
  D3:
  discovery cleanup past its shared deadline;
  the boot rule needs an authoritative kernel boot identity or a stated premise)
  and `cargo-shadow-sol-9b.txt` "Meets it after named fixes" (seven false-green test assertions,
  the missing same-namespace bind-mount schedule);
  no row added.
  Round 10 running on both (same branch and worktree);
  it cuts `cargo-shadow-10*.diff` when done.
- Latency round 3 merged to main (`b0d9708b17`;
  gate `gpc-gate-less-foreground-r3-fixes-20261104T073150-0500` passed 4751/4751;
  `native:clippy` on main clean);
  0.5.29 published (crates.io 2026-11-04T13:32:49Z,
  run 41837674379) and installed (`d4979bab1b`);
  the sample commit took 2665 ms,
  1254 ms of it provisioning the upgraded service,
  as earlier upgrades did (1016 to 1491 ms);
  the landing took 36 ms.
- Agents running:
  index-lock reclaim without `flock` (`fix/git-policy-cli-index-lock-reclaim-without-flock`,
  worktree `.claude/worktrees/index-lock-without-flock`);
  models round 15;
  cargo-shadow round 10;
  test images from pacman (`feat/git-policy-cli-test-images-from-pacman`,
  worktree `.claude/worktrees/test-images-from-pacman`).
  Not done from that round:
  a built root-commit control.
- Planted controls and the `service_durable.rs` survivors merged to main (`3248a86254`;
  branch tip `c279110576`,
  gate 4720/4720,
  `gpc-gate-planted-control-and-durable-survivors-20261104T070548-0500`);
  two stale controls fixed (self-exclusion by identity and content;
  the deleted `worktree_copy_lock.rs` control replaced by a live-owner journal control in `worktree_copy_recovery.rs`);
  two tests kill the `index_is_hardened` survivors,
  but the function has no production caller,
  so deleting it may beat testing it;
  resume point in `package/git-policy/cli/doc/planted-control-and-durable-survivors-resume.md` on that branch.
- Mutation campaigns paused (2026-11-04 at about 07:40-05:00),
  after the owner asked "We're not close to doing mutation runs though, right? Don't we still have a lot of things to fix before doing them?":
  the commit design is still at step 1 (models only;
  "Nothing past the red controls and the conflicted-file changes is built",
  `doc/planning/cli-git-commits-never-fail.md`,
  "Verification plan and build order"),
  and its live protocol replaces the commit path,
  whose directories hold most of the 0.5.28 scope (measured with `~/temp/agent/mutation-scope-overlap.ts` and a per-directory sum of `~/temp/agent/mutation-scope-0528.txt`:
  of 14,758 changed lines in 160 files,
  `index` 2,515,
  `landing` 2,261,
  `service` 1,098,
  `lock` 828;
  `merge` 2,571 also changes with the word-level pass;
  the latency branch rewrites 7 more scope files holding 621 lines);
  which of those files the live protocol rewrites is the coordinator's reading,
  not measured.
  The planted-control run started for the resume point (unit `mutctl-planted-0528b`) was stopped after its first control and its container removed.
  Resume the campaign after the commit protocol is built and the lint-debt pass (task 88) lands:
  first `native:mutation:planted` on that tree to show the two edited controls noticed,
  then a fresh scope.
  The planted-control branch still merges on its own (it fixes a crash on main:
  `node bin/native-planted-controls.unit.test.ts` stops with ENOENT on the deleted `worktree_copy_lock.rs`).
  Open:
  `index_is_hardened` has no production caller,
  so delete it rather than keep the two new tests (to decide when the branch merges).
- No agent process or unit is left in any of the four worktrees (checked 2026-11-04 at about 07:35-05:00).
- Decision record corrected (`05dc3800a8`,
  owner's "Correct it now."):
  `doc/decision/cli-git-concurrent-commits.md` now says landing writes objects with one `fast-import` (small batches loose,
  large ones a pack,
  measured with Git 2.56.0),
  protected by the scratch ref `refs/worktree/cli-git-service/<seq>` until the compare-and-swap.

Veto batch to ask the owner after the restart (the owner said "I'm fully back now, ask me."),
through the question tool in groups of four,
each with context,
the pick,
the alternative and a ranking:
- cargo-shadow:
  delegated transient scope as the default containment;
  the weaker path promises what pnpm promises and its recovery waits for an acknowledgement;
  assumptions a (no deliberate same-uid escape),
  b (pre-existing directories are durable),
  c amended (same mount and inode at recovery,
  else refuse),
  d (storage progress for blocking calls),
  e (one machine at a time,
  no checkpoint and restore);
  a verified-absent scope is proof of no members;
  same machine id plus a different boot id clears liveness;
  "Force at leaving,
  else background" read as the exit barrier only;
  the `user.delegate` route dropped.
- Commit design:
  level 2 disabled on the execution path where its eligibility cannot be shown.
- Answered after the restart,
  see "Answers after the restart (2026-11-04)" under "Owner decisions in force":
  the decision record's prune limit,
  reference-lock removal (shipped lock removal question 5),
  saved conflict copies that can be the only copy (shipped-limits pick 10's question),
  and the mutation pause.
- Index reclamation:
  the `flock` on the stale lock's own inode;
  without `flock` the lock is never removed;
  `.aside-*` leftovers documented,
  not swept.
- Latency:
  pack-sync option C;
  `sync_ms` renamed `index_update_ms`.
- CachyOS containers:
  `latest` resolved once per run and cached by the pulled id;
  registry unreachable means a run on the local copy with a warning;
  Git 2.56.0 from source;
  Rust from pacman;
  Node from the tarball;
  strace pinned.
- #6261:
  the `native` layer removal deferred (option A).
- Older picks still marked veto open in "Picks made under the owner's delegation" and in the planning records (shipped limits,
  waits,
  fallback to the real Git,
  word-level merge pass,
  cargo-shadow's nine questions):
  list them for the owner in the same way.

At 2026-11-04T06:55:05-05:00 (main `a72b474e1d`):
- Merged into main:
  index reclamation as `5d23a0f800` (branch tip `5ae971d1ec`,
  gate 4718 of 4718;
  the `.aside-*` sweep removed and its leftovers documented with a manual removal,
  coordinator's pick,
  veto open)
  and the CachyOS test containers as `a72b474e1d` (branch tip `976efc2fca`,
  gate 4707 of 4707 with passing names identical to main's control;
  e2e unchanged including the pre-existing `sigkill-service-during-work` failure,
  not investigated).
  Container agent's picks (veto open):
  resolve `latest` once per run and key a derived base on the pulled image id plus the Containerfile hash;
  Git 2.56.0 from source in the gate base;
  with the registry unreachable the run continues on the local CachyOS copy with a loud warning (the one pick near "Must be latest");
  Rust from pacman;
  Node 24.11.0 from the tarball;
  strace 6.13 by checksum.
  Other packages' images still on Debian:
  `cli.fuzz/bin/fuzz-images.ts`,
  `forbidden-strings/cli/bin/snapshot-constants.ts`,
  `linter/monochromatic-lint/test.Containerfile`.
- Latency round 3 done (`perf/git-policy-cli-less-foreground-work` tip `e24d0cd017`,
  gate 4727 of 4727):
  warm wrapped commit 108 to 78 ms unsigned and 143 to 109 ms signed (native 53 and 57);
  pack sync option C (the one `fast-import` child gets `core.fsync` minus `pack` and `pack-metadata` unless loose objects are synced),
  pack-first object reads,
  cache-tree comparison,
  built-commit checks from the object store,
  `sync_ms` renamed `index_update_ms`;
  swap (0 kB over 25 minutes) and the queue link (not shown) need no change.
  Decision-record drift for the owner:
  `doc/decision/cli-git-concurrent-commits.md` lines 118 to 121 and 512 to 514 describe a kept pack per landing,
  but the code keeps the scratch ref `refs/worktree/cli-git-service/<seq>` as the protection and unpacks small batches;
  the record needs an owner-accepted correction.
  Cross-vendor review running (`less-foreground-sol-1.txt`);
  merge and release 0.5.29 after it.
- cargo-shadow round 8 done (tip `089585a116`,
  540 passed,
  3 skipped):
  all sixteen list rows have tests,
  nine fully shown,
  seven partly;
  the delegated-scope route built (`systemd-run` with an escaped command line,
  since `systemd-run` rewrites `$`;
  the slice started before discovery);
  the `user.delegate` route dropped (needs `clone3` with `CLONE_INTO_CGROUP`);
  amended c's boot-scoped mount id makes every killed run's journal wait for acknowledgment after a reboot,
  so the coordinator proposes recording the boot id and treating a different boot as proof the run's processes are gone
  (in the review prompt).
  Review:
  8a "Does not meet the list as submitted" with "No additional acceptance row is needed" (F1 an absent scope accepted while its launcher can still create it;
  F2 manager discovery not fully bounded;
  F3 internal locks wait instead of returning busy;
  F4 an incomplete acknowledgement can count as valid;
  K1 an unreadable beside-lockfile directory suppresses the warning);
  8b "Meets it after named fixes" with "No additional acceptance rows or deeper adversarial model are required" (seven false-green or mis-scoped tests).
  On the boot id:
  "A genuine reboot of the originating kernel proves that its old local processes are gone",
  but a different boot id alone does not (a second host sharing the workspace).
  Coordinator's pick (veto open):
  record boot id and machine id;
  same machine id and a different boot id discharge process liveness only,
  every other recovery check kept;
  otherwise refusal and acknowledgement;
  assumption added that a workspace is used from one machine at a time.
  Round 9 started with both parts.
  Earlier review prompt note:
  the boot-id idea was in the round-8 prompt.
- Models round 13 done (tip `bf786b2004`,
  368 model tests):
  the round-12 findings closed red first;
  revision 18's support fold and receipt protocol modelled;
  coverage map 62 of 96 target items covered,
  9 only by historical controls,
  25 not yet.
  Review against the acceptance target (`commit-models-sol-8.txt`):
  "no,
  as submitted",
  with "None of these requests adds an acceptance criterion";
  a six-item path (component authority boundaries;
  departure promises;
  overstated controls;
  a corrected coverage map;
  current evidence for historical and not-yet rows;
  decision 19's fresh attempt with a stored notice);
  the author's two design findings are implementation obligations.
  Round 14 started on that path.
- Design:
  decision 19 recorded (`6d053403db`),
  no open owner question;
  review 17b timed out at its 3600 s bound (superseded by 18a and 19a).
- Mutation campaign:
  `landing/service_durable.rs` pass 1 scored 19 mutants (11 caught,
  6 unviable,
  2 missed:
  `index_is_hardened` replaced with `true` and with `false`);
  pass 2 did not run because the planted control "self-exclusion by identity and content" matches 0 times in `invocation/real_git_candidate.rs`.
  An agent fixes the stale control and kills both survivors (`fix/git-policy-cli-planted-control-and-durable-survivors`);
  campaign tags need one `podman tag` of the gate image per unit (`m0528-durable0` was added by hand).

Flagged for the owner (measured,
not forecasts):
cargo-shadow has had seven review rounds with no part passing,
and the commit protocol design six revisions (12 to 17) each drawing new P1s,
with the protocol verdict "approve continued model work only" since revision 12.
Cause found:
no written contract says what is promised and what is assumed,
so each review moves the adversary one level deeper (threads,
mount views,
a same-uid process that leaves its group).
Remedy taken:
cargo-shadow writes `doc/planning/cargo-shadow-guarantees.md` (guarantees,
assumptions,
behavior where an assumption fails) before more code,
and the reviewer turns it into a finite acceptance list;
the design reviewer is asked (`commits-never-fail-sol-17c.txt`) which of R17a-1 to R17a-5 are rule gaps,
for the finite property set that would move its verdict,
and which owner rules cannot all hold together for some schedule.
The cargo-shadow assumptions are coordinator's picks (veto open):
no deliberate same-uid escape (such a process can write `Cargo.lock` after the run anyway);
directories that existed before the transaction are durable;
the journal resolves to the same mount and inode at recovery,
else recovery refuses;
the weaker path promises what pnpm promises and its recovery waits for a person's acknowledgment.
cargo-shadow contract (`doc/planning/cargo-shadow-guarantees.md` on `feat/cargo-shadow-cli`,
`29af137137`):
cross-vendor review `cargo-shadow-sol-contract-1.txt` "fit after the named edits",
with a finite acceptance list (sixteen rows) and "No remaining round-7 safety finding inherently requires going another adversarial level deeper.";
the delegated-scope probe works (`systemd-run --user --scope --property=Delegate=yes`;
strace showed no cgroup file opened by cargo-shadow;
`user.delegate` appears only on `user@1000.service`).
Coordinator's picks for the two choices it left open (veto open):
a verified-absent scope is proof of no members (systemd removes a scope only when empty or after stopping its members),
scope names carry a random token and are never reused while a journal can name them;
every wait cargo-shadow controls has a deadline,
and blocking kernel and file-system calls rest on a stated storage-progress assumption mirroring "Assume the final disk save would never be cut short.".
Round 8 started:
the contract edits,
then every row of the acceptance list with a coverage map.
cargo-shadow P1 re-review (round 7):
7a "not closed" (F1 to F6:
the spawn lock does not stop capturing another thread's child;
discovery outside the spawn lock;
failure to leave the group returns an ordinary I/O error;
a completed process-group record cannot authorize recovery;
the boundary is writable by the payload;
recovery checks the cgroup namespace,
not the mount view);
7b "not closed" (a mount boundary is not a durable anchor;
`begin` turns an entry-level `NotFound` into "no earlier journals");
7c "not closed" (weaker-path recovery without quiescence;
the documented same-uid escape).

Queue,
in order:
- Design revision 17 (started by 2026-11-04T04:05:21-05:00,
  commit `1e301bfdc1`),
  from review 16a (`commits-never-fail-sol-16a.txt`:
  R15 closure "not closed";
  P1s R16a-1 level 2 still lacks the promised durable representation (the ref store need not share the object store's file system),
  R16a-2 the no-manager materialization path brings back the forbidden wait,
  R16a-3 an indefinite foreground upgrade or restart,
  R16a-4 `not-submitted` must be final for the whole attempt,
  R16a-5 same-path re-keying underspecified;
  question 8 "accurate after named edits",
  R16a-10)
  and review 16b when it is in.
  Question 8 is asked only after revision 17 settles what it depends on.
- Models round 13 after revision 18 lands,
  building to its "Acceptance target" section,
  with the round-12 reviews:
  `commit-components-sol-9.txt` "not closed" (the replacement authority is still caller-asserted;
  later appends escape its dependency check;
  completion can delete a dependency it just validated and forced;
  full-file identities lack a lifetime contract for mutable dependencies;
  departure validates argument presence,
  not the disposition's promise)
  and `commit-models-sol-7.txt` "P1 set:
  not closed" (one P1:
  ordinary successful completion still bypasses the register's final-save obligation;
  P2s on the shutdown green's monitor and `FifteenDropsLive` also removing R2's guard).
  Models round 12 tip `99bdd7f48b`,
  333 of 333.
  Its reviews then judge against the acceptance target,
  not a new open search.
- Lint task 88 once the open branches of this package merge.
- Native-layer removal (#6261 option A,
  veto open).
- Ask the owner for the measured span values (decision 17) before that part ships.
- Final re-review.

Older entries,
from the 2026-11-03T10:00-05:00 list,
are history to prune into canonical docs:
- Shipped-limits pick 1's classing with the measured citations 7,
  8,
  13,
  14 and 15,
  branch `fix/git-policy-cli-limits-cite-measurements`,
  worktree `.claude/worktrees/limits-cite-measurements`.
- Shipped-limits picks 4 and 9,
  branch `feat/git-policy-cli-steps-end-only-on-no-progress` from the 0.5.28 base `f58510c538`,
  worktree `.claude/worktrees/steps-end-only-on-no-progress`.
- Done and merged into the 0.5.28 base:
  the wrapper never runs itself as Git,
  `fix/git-policy-cli-never-runs-itself-as-git` `7ed8e9abe5`.
  Cause:
  `classify_candidate` (`real_git_candidate.rs`) compared a `PATH` candidate with the running file by inode,
  path and bytes,
  so the toolchain launcher counted as real Git,
  and the `forwarded_to_self` guard (`entry.rs`) named the launcher,
  not the binary behind it.
  Fix:
  candidates with a `<name>.launch` record of this tool are skipped,
  and every real-Git start sets `CLI_GIT_STARTED_AS_GIT` (starter pid,
  birth,
  mode),
  so a wrapper that arrives as its own starter's Git stops with exit 2 and a message naming every candidate.
  Coordinator's picks under the delegation,
  veto open:
  a user script named `git` that runs the wrapper without `exec` stays a documented limit
  (a depth cap would need an uncited number;
  a script that uses `exec` is caught);
  falling back to the real Git the wrapper was installed over when `PATH` holds none is queued after 0.5.28.
  Found by the cargo-shadow agent's bench on 0.5.26:
  `PATH` held `cli-git-bin/git` (a symlink to the toolchain launcher `.cache/toolchain-launchers/cli-git-native`)
  and no real Git;
  `git rev-parse ... @{upstream}^{commit}` became a parent-to-child chain of wrappers,
  each adding its `-c hook.*.enabled=false` options,
  exactly 100 at the scope's `TasksMax=100`,
  all sleeping,
  until the scope was stopped;
  `git --version` still printed "shadowing git at /usr/bin/git".
  Against "We should never invoke ourselves recursively."
  (`doc/planning/cli-git-no-self-invocation.md`);
  without a cap it can exhaust the host's process limit.
- Done and merged into the 0.5.28 base (`177f9ebc50`):
  shipped-limits picks 2 and 6,
  `feat/git-policy-cli-index-lock-and-service-waits` `008b74d3a3`.
  Its follow-ups,
  picked under the delegation (veto open) and queued:
  an unproven lock owner is waited for like a proven one,
  because an editor-held `git commit` would otherwise make ours fail after 1 s;
  poll when no inotify watch is available;
  ask the service manager to start an inactive unit after `START_WAIT`;
  a scratch file can be aged out during a wait longer than a day.
  The 150 s hold-age margin is for the owner to approve on return.
- Done and merged into the 0.5.28 base (`cfd30ccd18`):
  the two shipped lock-deletion P1s from the eighth design review (S-1,
  S-2),
  `fix/git-policy-cli-never-removes-a-live-lock` `9f92dae98c` (from `177f9ebc50`).
  The stopped-child sweep no longer deletes anything itself
  (it deleted a live foreign writer's empty lock,
  and a lock made after an old pid file);
  it calls the same judgement as every fetch and push.
  The boot-time proof is gone (a clock step fooled it);
  a pid number proves a holder gone only from the initial PID namespace,
  without `hidepid`,
  and when no process carries that number inside a nested namespace (`NSpid:`).
  Companion files (pid file,
  `packed-refs.new`) go through the same rename,
  compare and remove-or-put-back step as the lock;
  that step now checks identity twice and moves with `RENAME_NOREPLACE`.
  7 red tests first,
  98 library and 10 binary tests,
  clippy clean.
  Its four questions are in "Picks made under the owner's delegation".
  Conflict to expect:
  the citations follow-ups set `BOOT_SLACK` to 3607 s,
  which this branch removes.
- Done:
  the second cross-vendor review of cargo-shadow,
  split in three after the whole 733 KB diff timed out at 3600 s with no output
  (2026-11-03T11:23:48-05:00),
  each part from `504a82aa3f` to `09bb7820ea` with the first review attached.
  None of the three parts passes.
  Part 2a (the Git reader,
  `cargo-shadow-sol-2a.txt`,
  2026-11-03T11:38:43-05:00) says "not correct",
  with three P1s that can pick `HEAD` where Git picks an older upstream:
  a `\` at the end of a comment line joins the next line,
  hiding an `[include]`;
  `extensions.worktreeConfig = yes` (or `on`,
  or a nonzero number) is not read as true;
  the last `branch.<name>.merge` entry is used where Git uses the first.
  P2s:
  the section header parser (dotted section with a quoted subsection,
  `]` inside quotes);
  trailing whitespace trimmed from quoted values;
  a bare key read as the string `true`;
  worktree-private refs read from the common directory;
  `refs/remotes/<name>/HEAD` missing from short-name lookup;
  a hexadecimal ref value taken as a commit without reading the object;
  `/etc/gitconfig` assumed as the system file;
  `gitdir:` path whitespace trimmed and `commondir` read errors taken as absence;
  unknown extensions and invalid refspecs answered natively.
  P3:
  the fallback can start four Git processes,
  not one.
  It asks for a differential suite with Git as the oracle.
  Part 2b (the journal and restore,
  `cargo-shadow-sol-2b.txt`) says "Not correct":
  output attributed by reading after Cargo returns,
  an exchange-then-unlink race,
  a fallback race,
  predictable temporary names,
  read errors taken as empty,
  no fsync ordering,
  an unreadable manifest index read as success,
  a global `kept_a_lock`,
  no recovery claim;
  P2s for lossy paths and CSV memory.
  Part 2c (install,
  verify,
  policy and the planning record,
  `cargo-shadow-sol-2c.txt`,
  2026-11-03T11:37:58-05:00) says "not ready for 0.1.0",
  with seven P1s:
  a Cargo call that writes a lockfile and then exits nonzero leaves it unrecorded,
  and the `--locked` fallback builds it;
  a lookup error after an age refusal erases the refusal and builds;
  `cargo update` checks only versions this command added,
  so an unchanged `--precise` target is never judged;
  the same post-Cargo read attribution as 2b;
  placeholder cleanup reads,
  then deletes,
  so a lockfile published in between is lost;
  a sync-failure rollback writes the old catalog over another writer's edit;
  a malformed historical timestamp sorts first and gives a new publisher earlier history.
  P2s:
  frozen non-strict age warnings are dropped;
  `cargo update -m` without a value becomes a full update;
  an option value equal to the verb is taken as the verb;
  partial sync edits are not recorded.
  P3:
  seven comments say the opposite of the code.
  Before those reviews,
  the first review's follow-ups were done:
  294 of 294 tests,
  the trust base read from Git's files with one `git` start only in the cases they cannot answer,
  saving 1.5 to 4 ms per call,
  `file-enforcer write` from source reproduces the generated line,
  and about 4300 comment lines,
  partly from a regex tool,
  which the review checks for false statements).
- Done,
  not merged anywhere (design code,
  nothing live calls it):
  step 1 of the commits-never-fail build,
  `feat/git-policy-cli-commit-protocol-models` `22e0ba1f37`,
  worktree `.claude/worktrees/commit-protocol-models`.
  A hand-written schedule explorer (stateright is not in the offline registry;
  loom and shuttle cannot model system calls)
  and models F1 to F9,
  each red under revisions 6 and 7 with literal replays and green for revision 8 or a candidate;
  step 3 components (ledger,
  identity binding,
  request,
  spool) and the step 4 pure replay,
  differential against Git 2.56.0;
  105 of 105 tests (`native:test:model`),
  clippy clean.
  Four findings against revision 8,
  sent to the revision 9 agent:
  a resend answered from a visible but not yet durable inbox record (F7);
  an owner wedged with no gate cannot be ended by a catchable signal (F3,
  L7);
  L4 must raise the capture generation to odd first (F6);
  a recorded swap-failed replays silently after the post-anchor reflog lines are erased (F1,
  R4).
  The eighth review's section-4 controls are not built;
  they follow revision 9.
  Its comment density is below the dum-dum skill's in the F2 to F9 models and three component files.
- Done and merged into the 0.5.28 base (`afc9a8621d`,
  both merges clean,
  clippy clean on the result):
  the fallback to the real Git the wrapper was installed over (task 150),
  `feat/git-policy-cli-falls-back-to-installed-git` `6453c380b1`.
  When `PATH` offers no real Git,
  it uses the `--git` path from the commit service's unit file under `$XDG_RUNTIME_DIR/systemd/user`,
  checked as every `PATH` candidate is,
  and prints one line naming the path and the directory to add to `PATH`;
  no usable record keeps the stop.
  7 new binary tests and 17 new unit tests,
  red first;
  its four questions are in "Picks made under the owner's delegation".
- Done and merged into the 0.5.28 base (`afc9a8621d`):
  the shipped-limits follow-ups after the citations (task 151),
  `fix/git-policy-cli-limits-cite-measurements` `e314460f79`
  (2026-11-03,
  measured under the load other sessions put on the host;
  figures in the audit doc's "Follow-up measurements" section).
  Lock age `MARGIN` 7 s (twice 3438 ms,
  rounded up),
  `BOOT_SLACK` 3607 s;
  8 local queries at once (the earlier measurement,
  not repeated) and 4 remote;
  batch cap 256,
  the size at which the per-request landing cost (0.16 ms) had not yet doubled,
  from a burst of up to 1024 clients against a throwaway service;
  claim lease kept at 120 s (a killed claim recovered 119.55 to 119.57 s after the kill;
  a live holder losing its claim was not measured);
  copy,
  install and clone go parallel from 16 entries (`PARALLEL_BREAK_EVEN_ENTRIES`;
  the old 4096,
  1024 and 512 left up to 145,
  50 and 12 ms unused);
  removal split depth 3,
  install batch 4096 (12% slower than one batch with a stand-in recorder;
  the owner's trade) and 4 index copies kept as measured.
  Red tests first for each changed value;
  688 unit tests,
  8 binary tests and clippy clean;
  the full gate was not run.
  The claim-kill probe's scope showed inactive before the tests ran,
  so that run was bounded only by the tests' five-minute limit.
- Done and merged into the 0.5.28 base (`a840964c97`):
  the third round on the in-process index writer,
  `perf/git-policy-cli-follow-up-index-in-process` `01e4d4c610`.
  Findings 1 to 5 of the third re-review fixed,
  each red first;
  6 (the reclaimer's check-then-unlink) recorded in SPEC,
  not fixed.
  The shipped P1:
  `IndexLock::release_with_record` installs the index,
  records `<batch>.indexed` (the commit name,
  synced) while the lock is held,
  then unlocks;
  recovery that finds the record replays only the conclusion state.
  The HEAD check is asked again once the lock is held;
  recovery that finds another branch checked out leaves its index alone.
  `PrivateCopy` is made with `O_EXCL` and never reused,
  so copies a killed run left can pile up (no cleanup yet).
  720 library and 70 binary tests,
  clippy clean.
  The fourth cross-vendor re-review (`~/temp/agent/reviews/follow-up-index-sol-4.txt`,
  2026-11-03T12:08:52-05:00):
  "Safe after named fixes,
  not safe to ship as shown".
  P1s:
  recovery can still overwrite later staging (a build or install error unlocks with a replayable intent;
  recovery unlinks a stale lock before taking its own;
  a panic in `record` unlocks),
  and an unreadable `.indexed` record is taken as absent.
  P2s:
  durability ordering of the install,
  record and done mark;
  the second HEAD check is still apart from the ref update;
  the private copy is filled at default permissions;
  the private name is not owned for the whole operation;
  branch-change recovery drops an alternate index's update and pending cleanup;
  HEAD read from the wrong worktree;
  sixteen abandoned copies exhaust the names for good.
  P3:
  a long alternate index name overflows `NAME_MAX`.
- Done:
  fourth round on the index writer,
  `perf/git-policy-cli-follow-up-index-in-process` `2dd575cfc1`,
  all ten findings red first,
  774 library and 70 binary tests,
  clippy clean.
  Recovery resets only paths whose entries still equal the `<seq>.start` record taken under the lock before the ref moves;
  `.indexed` is three-valued and recovery stops on an unreadable one;
  the index directory is synced before the record,
  and the journal directory before `.indexed` goes;
  the release disarms before syncing,
  so an error or a panic keeps the lock;
  an alternate index is still finished after a switch;
  HEAD is read from the request's own worktree Git directory;
  private copies live in an owner-only `cli-git-private-<uuid>` directory with an owner marker,
  swept only when its pid is dead.
  Finding 4 differs from the pick:
  Git 2.56.0 refuses `symref-verify HEAD` beside an update of its referent ("multiple updates for 'HEAD' ...
  are not allowed",
  probed),
  so HEAD is read again after the ref moved,
  under the lock,
  and a worktree that left the branch keeps its index;
  accepted under the delegation,
  veto open.
  The gate's ACL test now sets the default ACL by the `system.posix_acl_default` xattr (passed in the existing image).
  Present in 0.5.27 (tag `monochromatic-git-policy-cli-v0.5.27`):
  1(A),
  1(B),
  1(C),
  3,
  4,
  6 (worse),
  8;
  not 2,
  5,
  9,
  10;
  7 inverted (0.5.27 replays onto a switched worktree).
  Not fixed:
  the `remove_stale` check-then-unlink;
  a batch with no start record (a commit Git built) replays every path;
  two extra fsyncs per landing,
  unmeasured.
  It merged the base `14c26dbcc9` with no textual conflicts (`f6be7b42e2`);
  one semantic conflict found at compile time,
  the carried conflict stages against the private-copy directory,
  fixed in `e7e304ffd6` (stages written into the copy's own `index` before the release);
  794 of 795 library tests (`service_index_lock::tests::a_foreign_lock_is_described_with_the_process_that_has_it_open` failed once,
  then passed alone and in its 64-test module:
  a load flake to confirm in the gate)
  and 95 of 95 binary tests,
  clippy clean.
  The base was fast-forwarded to `e7e304ffd6`.
  Running,
  not an agent:
  the fifth cross-vendor re-review (`follow-up-index-sol-5.txt`).
- Check the container image `native:test:container` builds:
  the index agent reports a Debian trixie base,
  against the repository's rule of the newest CachyOS image (`doc/decision/container-base-distro.md`).
  Picked under the delegation (veto open):
  recovery resets a path only while its index entry still equals the one the landing started from,
  so a later `git add` survives however the lock handoff goes ("never discard user data");
  the HEAD expectation moves into the ref transaction (`update-ref --stdin` `symref-verify`,
  to be checked against the installed Git).
- Done:
  design revision 9 of `doc/planning/cli-git-commits-never-fail.md`,
  `a7b917e47e` to `5ff9706edc` (2026-11-03T11:44:10-05:00 to 11:54:15),
  answering all of the eighth review and the four model findings;
  the three choices it left were picked in `7afb60a024` (see "Picks made under the owner's delegation").
  The ninth cross-vendor review (`~/temp/agent/reviews/commits-never-fail-sol-9.txt`,
  2026-11-03T12:09:51-05:00):
  "Approve continued executable-model work and disposable backend experiments only.
  Do not approve the live protocol."
  Closed:
  the ungated restart,
  the shared ledger repair,
  shared candidates,
  the suppression example,
  the signal choice.
  P1s:
  C0 releases the lane lock while its client lives,
  so the next owner kills a healthy client (R9-1);
  a resend acknowledges a binding whose directory entry is not durable (R9-2);
  M1's journaled inode number is not deletion authority (R9-3);
  MG2 and MG3 erase unrelated staging and ignore a completed switch (R9-4);
  T1 assumes reachability holds through a maintenance run's earlier view (R9-6);
  and the shipped lock-removal step (R9-S,
  see "Shipped lock removal").
  P2s:
  merge staging captured before publish-begin and a reverse carry that is no inverse (R9-5),
  R4 cannot tell an erased suffix from none so the F1 closure is withdrawn (R9-7),
  E1 retries a refusal after a foreign change (R9-8),
  B3 with path-set changes (R9-9),
  undefined exits (R9-10);
  P3 rule S's arithmetic (R9-11).
- Done:
  design revision 10,
  `7afb60a024` to `dc6e809eca` (2026-11-03T12:22:02-05:00 to 12:27:13),
  answering all of the ninth review;
  new rule D (whoever relies on a visible name makes it durable first),
  rule N (the lane removes only names no foreign program creates),
  a recovery helper for C0,
  delta merge installs with attachment authority,
  a hold ref rooting each recovery copy,
  E1 by exit status first,
  amendments as fresh identities,
  a `resume` point on every waiting request,
  a durable hook result file.
  It states that shipped code should stop removing reference locks until a primitive that cannot revoke a replacement lock is shown;
  that is the open owner question in "Shipped lock removal".
  Its five choices picked in `eed561efaa` (see "Commits never fail,
  revision 10").
  The tenth cross-vendor review (`~/temp/agent/reviews/commits-never-fail-sol-10.txt`,
  2026-11-03T12:41:05-05:00):
  "Approve continued executable-model work,
  pure components without reclamation,
  and disposable backend experiments only.
  Do not approve the live protocol."
  P1s:
  an absent `index.lock` does not prove M1 installed the index (R10-1);
  W-detect and MG6 cannot restore after a detected switch (R10-3,
  so W-detect is not approved as specified);
  G-root's invariant is not established (R10-4).
  P2s include a recovery helper that can wait without end (R10-5)
  and P-wait being possibly indefinite,
  both now against the owner's no-unbounded-wait rule.
  Done:
  design revision 11,
  `eed561efaa` to `1430d7ddfa` (2026-11-03T12:48:49-05:00 to 12:54:14),
  answering the tenth review and applying the owner's bound answers:
  rule B registers every blocked process with its bound;
  W1 at a command's bound withdraws the request durably,
  keeps the commit on `refs/cli-git/<instance>/withdrawn/<id>`,
  leaves staging alone and exits non-zero;
  P2 is `update-ref --stdin` prepare,
  decide,
  commit,
  and W1 races the decision by `link` to `lane/decision/<id>`,
  so a request with a client lands or never lands;
  bounds START,
  FOREGROUND,
  MARKER,
  ACQUIRE,
  REAP,
  KILL,
  COMMITTING,
  STALL and PUSH start from shipped constants,
  labelled unmeasured.
  Its choices (`9c2b2ec0ab`,
  `e6199d4e4f`):
  the owner decided P-carry ("Merge anyway,
  unstage x") and W-detect with its residue ("Allow the rare leftover");
  the coordinator picked G-order,
  N-silent and H-episode.
  Running,
  not an agent:
  the eleventh cross-vendor review (`commits-never-fail-sol-11.txt`).
  Running:
  step 1,
  fifth round (revision 11 models with `explore_bounded` over every registered wait,
  the W1-versus-P2 race,
  the tenth review's controls,
  the components' third-review fixes,
  the per-site comment pass).
- Done,
  not merged:
  step 1,
  third round,
  `feat/git-policy-cli-commit-protocol-models` `27fa595034`,
  149 of 149 model tests,
  clippy clean.
  Revision 10 modelled for F1 (an earlier "green" relied on an oracle and is corrected),
  E1,
  F8 and M1 (each red under revision 9,
  green under 10);
  most of the ninth review's controls are not built.
  The components re-review's P1s are fixed (a durable chain through an anchor on every call;
  a canonical manifest checked against the bytes returned),
  with the P2s and most P3s;
  not done:
  a populated-submodule fixture,
  deterministic fault points,
  the comment-density pass.
  The components' third review (`commit-components-sol-3.txt`,
  2026-11-03T12:51:01-05:00):
  "Ready after named fixes".
  A and B are repaired provided the anchor is durable;
  to do:
  an initialized-root constructor that makes the root durable through a known-durable parent
  (P1 integration prerequisite),
  bare relative ledger filenames and ancestor durability before a repair (P2),
  gitlink and amendment eligibility and retention contracts (P2),
  the trace gated out of production (P3),
  a fully canonical manifest syntax or an accurate claim (P3),
  deterministic filesystem-ordering tests,
  and bumping `REQUEST_VERSION` for the new mandatory line.
  In step 1's fifth round.
  Done:
  step 1,
  fourth round (`bc3ff62cd4`,
  157 of 157 model tests,
  clippy clean):
  `explore_bounded` flags a wait longer than N of the protocol's own steps,
  or a wait with no own step left;
  `model_observe.rs` separates what a reader sees from what a power loss keeps;
  deterministic fault points before the link in `bind`,
  `publish` and the spool;
  a real populated-submodule fixture for `GitlinkNeedsAncestry`.
  The comment pass was a script adding one generic block per concept from a fixed list
  (F2 and F6 got none);
  a real per-site pass is still owed.
  The models' first review waits for revision 11's models
  (the models diff is 386 KB and must be split).
- Done,
  not merged anywhere:
  step 1,
  second round,
  `feat/git-policy-cli-commit-protocol-models` `52ccb3e38e`.
  Revision 9 modelled:
  each changelog schedule red under revision 8 and green under 9
  (F1,
  F3,
  F4,
  F6,
  F8,
  F9,
  plus new two-lane ledger,
  M1 kill-point and merge models),
  135 of 135 tests.
  It found one revision 9 defect:
  C4 answers `handed over` when its no-replace link finds a visible but not durable record
  (fix proposed as variant `NineFsyncOnExists`,
  sent to revision 10).
  Components review P1s fixed
  (type-aware replay,
  per-lane ledgers,
  durable directory creation,
  resends sync the name,
  request format version 2 with the base bytes in the spool);
  not fixed:
  byte paths,
  gitlink fast-forwards,
  `merge-tree -z`,
  race-test barriers,
  and the comment-density pass.
  Some red states were compile errors rather than failing assertions.
  The components re-review (`~/temp/agent/reviews/commit-components-sol-2.txt`,
  2026-11-03T12:21:53-05:00):
  "not correct".
  The type-merge P1 and the `.torn` ordering are fixed;
  per-lane ledgers hold under revision 9's single-appender contract.
  P1s:
  `create_dir_all_durable` stops at the first existing ancestor,
  so an ancestor another creator made but never synced can vanish after acceptance;
  the spool manifest's order and indices are unchecked,
  so a correctly hashed manifest can make `read_closure` return old and new bytes swapped,
  and an index such as `../other` is read as a path.
  P2s:
  an agreed type change (symlink to regular file on both sides) is refused;
  a relative path fails (`create_dir("")`);
  an empty commit and a message-only amendment are indistinguishable (no operation kind);
  v1 requests are rejected (fine before release).
  P3s:
  the oracle strips legitimate `~<hex>` names;
  the test sync counter runs in production code;
  the counter tests prove neither the target nor the order.
- Step 1,
  second round:
  its agent is resumed to model revision 9 (red under revision 8,
  green under 9 for each changelog schedule),
  build the eighth review's section-4 controls,
  raise the comment density,
  and then fix the components review:
  "not correct" (`commit-components-sol-1.txt`),
  P1s for a mode and object merged across file types (a symlink made from an edited file),
  ledger repair racing an append (answered by revision 9's per-lane ledgers),
  new directory names never synced into their parents,
  a resend acknowledging a link whose directory entry was never synced,
  and a spool that cannot be replayed once the base objects are pruned.
- cargo-shadow fixes for all three second-review parts,
  its agent resumed;
  it writes the three diffs for the third review to `~/temp/agent/reviews/cargo-shadow-3a.diff`,
  `-3b.diff` and `-3c.diff`.
- The word-level second merge pass (task 114),
  `feat/git-policy-cli-word-level-second-merge-pass` at `2a37bb7682`:
  the 0.5.28 base merged in (`9763599f3a`;
  in `merge_isolation.rs` a clean-index conflict still goes to native Git first,
  then the pass runs on the stopped merge);
  autostash and `branch.<name>.mergeOptions` verified;
  a pull that is stated to merge now runs as `git fetch` plus the isolated `git merge FETCH_HEAD`
  (`pull_isolation.rs`),
  because native `git pull` starts its merge as Git's own child,
  out of the wrapper's sight.
  Second round done (`411e98e732`) and merged into the 0.5.28 base (`9e0b06ad83`):
  the three picks built
  (an autostash whose apply would conflict leaves the merge stopped as Git does,
  the work kept in `MERGE_AUTOSTASH`;
  the pull's working-tree update waits for the index lock);
  a fast-forwarding pull skips the isolated attempt.
  Measured under the load there was (load average 9 to 11,
  debug build,
  nine alternating rounds,
  native-versus-native spread 2 ms or less):
  a merging pull 367 ms against native 14 to 16 ms,
  a fast-forwarding pull 128 ms against 11 to 12 ms;
  most of the merging cost is the isolated merge itself (a typed merge 302 ms).
  Answering the pull's queries in process is open.
  179 library and 83 binary tests,
  clippy clean.
  Its agent is resumed on `fix/git-policy-cli-conflict-warning-reads-the-content` from `9e0b06ad83`:
  the false conflict-marker warning in "Known flakes and failures not yet fixed",
  then carry-back A.
- Passed:
  a second early gate of the 0.5.28 base at `afc9a8621d` (after the waits,
  citations follow-ups and fallback merges),
  unit `gpc-gate-candidate-0-5-28-fallback-citations-20261103T114530-0500.service`,
  4468 of 4468,
  6 skipped,
  one slow.
- Failed,
  2 of 4575 (8 skipped):
  a third early gate at `9e0b06ad83` (after the index writer's third round,
  the lock-removal fix and the word-level pass),
  unit `gpc-gate-candidate-0-5-28-index-r3-locks-word-level-20261103T120421-0500.service`.
  `index_reset_file::tests::the_file_mode_under_a_default_acl_equals_git`:
  the gate's container has no `setfacl` (`NotFound`);
  sent to the index round 4 agent (add the ACL tools to the image).
  `git_stop_lock_tests::a_stop_without_grace_sweeps_the_locks_on_a_thread_of_its_own`:
  a regression from the lock-removal fix,
  passing on the host;
  the stopped child's lock and its pid file stay,
  read as the general judge declining inside the container's own PID namespace,
  so inside any container every Git child the wrapper ends would leave a lock.
  Fixed and merged into the base (`3c7aaa1122`):
  `fix/git-policy-cli-sweeps-its-own-reaped-childs-locks` `9176f9302c`.
  The reading was confirmed (the judge reports a PID namespace of its own and returns Unproven).
  The sweep passes the judge the ended members of the tree it reaped (`Judge.own_ended`);
  a pid file born at or after the lock,
  a lock born after the child started (`COARSE_CLOCK_SLACK` 50 ms back),
  and no process with that number again prove it gone in any process view,
  without the 4 s margin.
  Container run 2 red then 93 of 93;
  host 104 of 104;
  six negative controls for the container view.
  It adds occasions for the move-aside residue (R9-S) without changing the step;
  SPEC now states that residue beside the removal step.
- Failed,
  7 of 4596:
  a fourth early gate at `3c7aaa1122` (after the waits follow-ups and the regression fix),
  unit `gpc-gate-candidate-0-5-28-waits-own-child-locks-20261103T122526-0500.service`.
  The ACL test as expected (no `setfacl` in the image,
  with the index round 4);
  the stop-lock test now passes;
  and six `budget::` binary tests:
  "starts of git check-attr: observed 1,
  budget 0" on every ordinary commit,
  a regression from the conflict-warning fix (`conflict_markers.rs` asks for `conflict-marker-size` even when no path was conflicted).
  Its agent is fixing it on `fix/git-policy-cli-conflict-check-only-for-conflicted-paths`
  (start nothing new for an ordinary commit;
  budget constants unchanged).
- Done and merged into the 0.5.28 base (fast-forward to `d300bb806d`):
  `fix/git-policy-cli-conflict-warning-reads-the-content`.
  The marker claim reads the committed blob (`conflict_markers.rs`,
  honouring `conflict-marker-size`,
  default 7,
  a lone `=======` not enough);
  any named-path commit during a merge or cherry-pick prints one line saying how to finish it.
  Carry-back A:
  a conflicted autostash's `stash@{0}: autostash` and its unmerged stages reach the real repository
  (objects already land in the real store;
  `autostash_carry.rs`,
  stored once so a restart does not store it twice);
  removing either half turned its control red.
  134 library and 90 binary tests,
  clippy clean.
- Done:
  the index-lock and service waits follow-ups,
  `feat/git-policy-cli-index-lock-and-service-waits` `82633304d0`,
  each pick red first:
  unproven index-lock owners waited for with no bound (`UNPROVEN_OWNER_BACKOFF` and `unproven_backoff.rs` deleted);
  polling every 50 ms (`POLL_PAUSE`,
  classed exempt) where no inotify watch can be set;
  an `inactive` unit asked to start after each `START_WAIT`;
  a waiting command renews its scratch directory once an hour so the sweep keeps it.
  903 library and 92 binary tests,
  clippy clean;
  the real manager calls (`active_state`,
  `start_unit`) were not run against a real user manager.
  Open audit question:
  the heartbeat proves the service and its connection,
  not that the landing thread makes progress.
  It merged the base `d300bb806d` with no conflicts (`db0e92df64`),
  dropped its handover edit (`afbf402146`;
  the base branch's copy of this handover is stale,
  and main's copy wins when the base merges into main),
  and the base was fast-forwarded to `afbf402146`
  (260 wait,
  68 carry-back and conflict,
  29 binary tests;
  clippy clean).
- Passed:
  an early gate of the 0.5.28 base at its tip after the index writer's second round
  (unit `gpc-gate-candidate-0-5-28-early-20261103T111639-0500.service`,
  4424 of 4424,
  7 skipped,
  one slow).
- Done:
  the eighth cross-vendor design review (`~/temp/agent/reviews/commits-never-fail-sol-8.txt`,
  2026-11-03T11:33:50-05:00),
  with the shipped stale-lock code attached.
  Recorded in `doc/planning/cli-git-commits-never-fail.md` on `fix/git-policy-cli-commits-never-fail` (`a7b917e47e`,
  section on the owner questions):
  the shipped stale-lock rule does not survive (S-1,
  S-2,
  fixed on their own branch);
  B2 is withdrawn for B1,
  because a restarter's correct "no program running" observation can be followed by a hook starting,
  which the restart then ends (R8-1);
  A2 stays as an accepted narrowing:
  ending a stalled Git child that ignores the terminate signal can leave a lock only the person clears.
  The other findings need design revision 9:
  the ledger has several writers,
  C2 is not immutable,
  generations do not cover every state,
  X1's classification,
  M2 brings back a removed path,
  T1 conflicts with G3,
  and the state machine has gaps,
  merge among them (R8-2 to R8-8).
  Revision 8 `833b4ef80a` to `3f49d76ec6` (2026-11-03T11:04:37-05:00 to 11:06:22) is the normative state machine
  (L1 to L7,
  C1 to C5,
  B1,
  B2,
  P1 to P4 with recovery R1 to R5,
  M1,
  M2,
  T1,
  X1,
  G1 to G4);
  its two owner questions were picked in `3beecf2368`:
  A2 (kill a Git child that ignores the terminate signal,
  the owner's "End Git") and B2 (restart a quiet owner that holds the gate,
  against handing our own wedge to the person);
  a lock either leaves goes through the stale-lock rule shipped since 0.5.26,
  which revision 8's "the wrapper never removes a ref lock" contradicts,
  so the eighth review judges that shipped rule too.
- Done and merged into the 0.5.28 base (`a7a19b590a`,
  audit doc conflict resolved by keeping both sections):
  shipped-limits picks 4 and 9,
  `feat/git-policy-cli-steps-end-only-on-no-progress` `1d30940265`
  (Git steps,
  queries,
  the linter,
  the review gateway and the repair tool end only after a labelled unmeasured quiet window;
  `QUERY_CEILING` gone;
  every step now records its longest quiet gap in the timing records and the service journal).
  Picked under the delegation:
  collect quiet gaps from ordinary use and set each window above the longest by a margin the owner approves on return;
  redo a quiet step that is safe to run twice once;
  `update-ref` waited for,
  not ended,
  until the eighth review settles the lock question;
  the review's local log write left without a judge.
- Done and merged into the 0.5.28 base (`fc19017f67`,
  audit doc conflict resolved by keeping both agents' sections):
  shipped-limits pick 1 and the measured citations 7,
  8,
  13,
  14,
  15,
  `fix/git-policy-cli-limits-cite-measurements` `9b65660e95`
  (free space from the entry count,
  about 1 KiB of btrfs metadata per entry measured;
  ref-lock `MARGIN` 4 s from the longest journal `ref_ms` 3438 ms;
  `BOOT_SLACK` 3604 s;
  `REPOSITORY_MEMORY` and `ENTRY_MEMORY` 128 MiB;
  35 constants classed exempt;
  `MAX_BATCH` and `CLAIM_LEASE` have no data).
  Its follow-up questions,
  picked under the delegation (recorded in the audit doc,
  "Picked for these follow-ups"):
  measure the batch size and claim lease against a throwaway service;
  keep the 3 h idle exit;
  8 concurrent local queries and 4 remote;
  measure the copy-stage and install thresholds on the real stage;
  patience 1 h now,
  derived again when the index-lock wait pick lands;
  lock margin 7 s (twice the sample maximum);
  the preflight as built.
  Those follow-ups are queued for the next free slot (resume that agent).
  The gate owes its `native_binary` run:
  `binary_stale_ref_lock_tests.rs:151` now sleeps the margin plus 0.5 s.
- Not an agent:
  the cross-vendor re-review of the in-process index fixes (`pi`,
  output `~/temp/agent/reviews/follow-up-index-sol-2.txt`);
  the fixes are done (`dac87edc5e`,
  merged into the 0.5.28 base):
  the helper refuses to run unless the live `index.lock` marker names this process and batch,
  the landing holds that lock from the copy through the install (`service_land.rs:359`,
  `147-153`,
  `service_index_hold.rs:140-165`),
  one exclusively created temporary per run,
  Git's mode under `core.sharedRepository`,
  EWAH bitmaps checked and bounded,
  every protected-name spelling declined;
  the intent-to-add finding does not reproduce on Git 2.56.0 (a byte-for-byte differential pins it).
Queued,
in this order:
- Done (`cb2b9fe766`,
  merged into the 0.5.28 base with the no-`--quiet` test conflict resolved by keeping both sides),
  third re-review running (`~/temp/agent/reviews/follow-up-index-sol-3.txt`,
  with `service_land.rs`,
  `service_index_hold.rs`,
  `service_index_lock.rs`,
  `service_land_follow.rs` and `index_reset_publish.rs` attached whole):
  second round on the in-process index writer,
  from its re-review (2026-11-03T10:51:00-05:00,
  `~/temp/agent/reviews/follow-up-index-sol-2.txt`:
  "safe after fixing the remaining NTFS alias,
  correcting permission ordering,
  resolving the test-build/isolation issues,
  and verifying the cited outer locking and installation code";
  the temporary-file race and EWAH are fixed,
  and the intent-to-add finding is withdrawn):
  decline the hashed NTFS short name of `.gitmodules` (`GI7EBA~1`) with a real-reset differential;
  set the file mode before writing any byte,
  tested with a mode narrower than the umask gives;
  pass a borrow of the held `IndexLock` guard to the helper instead of a marker check;
  run umask-changing tests in a child process;
  assert the intent-to-add entry's flags,
  not the index version bytes;
  tests for injected write and sync failures,
  concurrent publication with distinct payloads,
  and an unselected change staged before the lock that must survive.
  The non-Linux test imports stay paused ("Ignore Win and mac until I say so.").
  Then a third re-review with `service_land.rs`,
  `service_index_hold.rs` and the lock-marker code attached whole.
- Third round on the in-process index writer and its outer landing code,
  first in line,
  starting now that the index-lock and service waits branch is merged (`177f9ebc50`);
  resume its agent with the findings in this entry.
  From the third re-review (2026-11-03T11:29:26-05:00,
  `~/temp/agent/reviews/follow-up-index-sol-3.txt`:
  "Safe after named fixes,
  not safe to ship as shown";
  the second round's fixes hold):
  - P1,
    also in shipped 0.5.27's outer code:
    `release_with` (`service_index_hold.rs` 168-172) installs the index and removes `index.lock` before `mark_done`,
    and recovery replays an open intent's reset,
    so a `git add` made between the unlock and a service death is overwritten;
    `follow_up` also installs and unlocks when conclusion cleanup returned false.
    Record index completion durably while the lock is held,
    and keep index completion apart from cleanup completion.
  - P2:
    the mode calculation `0o666 & !umask` (`index_reset_publish.rs` 98-110) ignores an inherited default ACL;
    create the file as Git would,
    read its mode,
    adjust before the first byte.
  - P2:
    `private_index_path` (`service_land_follow.rs` 173-176) can equal the live index under `GIT_INDEX_FILE`
    or another client's index;
    reserve the private copy by exclusive creation and refuse aliasing.
  - P2,
    also in shipped outer code:
    `head_names` (`service_land.rs` 233-243) is checked before waiting for the index lock;
    revalidate after acquisition and before the ref move,
    and give recovery a rule for a branch no longer checked out.
  - P3:
    retry `Interrupted` writes;
    restore the deleted `later_status_and_diff_are_right_after_a_reset_without_a_refresh` coverage and the racy-entry
    fallback assertion;
    a reader-start handshake in the concurrent publication test.
  - Not a blocker for this Linux-only release:
    the reclamation check-then-unlink in `service_index_lock.rs` 285-289 needs a per-physical-index single-reclaimer
    guarantee or a test;
    the non-Linux test imports stay paused.
- The fifth index re-review once the fourth round writes `follow-up-index-fixes-4.diff` and its `index-r5-` files.
- Running:
  every wait in the wrapper gets a bound (the owner's "nothing should ever wait forever or indefinitely"),
  the waits agent on `fix/git-policy-cli-every-wait-has-a-bound` from `14c26dbcc9`:
  an inventory in `doc/planning/cli-git-no-unbounded-waits.md` first,
  then each bound with a red test,
  values measured from the timing logs and journals,
  unclear end behaviors left as choices for the coordinator.
  A blocker for 0.5.28.
- Done and fast-forwarded into the 0.5.28 base (`14c26dbcc9`):
  `fix/git-policy-cli-conflict-check-only-for-conflicted-paths`;
  an ordinary commit starts no `check-attr` again (the six `budget::` tests red on the base,
  green after;
  16 of 16 with the `commit_during_merge` controls).
- GitHub issue #6260 (2026-11-03T16:23:05Z,
  no comments):
  the source scans must find nested modules and key identities and exceptions by relative path.
  Running:
  a new agent on `fix/git-policy-cli-source-scans-cover-nested-modules` from `14c26dbcc9`.
- GitHub issue #6261 (2026-11-03T16:23:43Z,
  blocked by #6260):
  group the 1,012 flat files under `src/native/` into subsystem directories and weigh dropping the `native` layer,
  keeping the file-size rule.
  Queued after the 0.5.28 release and the open branches merge,
  because it moves every file;
  then the lint-debt pass (task 88).
- Retire option B of the autostash pick now that carry-back A exists,
  so the wrapper's finish ends a conflicting autostash as Git does
  (merge commit made,
  stash kept,
  `UU` entries),
  instead of leaving the merge stopped (picked under the delegation:
  "Auto merging should never fail.").
  Done and fast-forwarded into the 0.5.28 base:
  `fix/git-policy-cli-finish-applies-a-conflicting-autostash` `2b5e99f5a8`
  (`autostash_blocks_finish` removed;
  merge and pull controls compare with a twin repository on which Git stops,
  the person resolves and `git commit` concludes;
  the wrapper had ended with status 1 where Git's commit gives 0;
  6 of 6 autostash controls plus one for unsaved changes to a stash file;
  44 library and 41 binary tests,
  clippy clean).
- A gate after the regression fix,
  the index round 4 and the waits follow-ups merge,
  then release 0.5.28.
- After step 1's third round:
  the components' third review from `commit-components-3.diff`,
  and the models' first cross-vendor review from `commit-models-1.diff` (split if it exceeds about 250 KB).
- After the tenth design review:
  revision 11 if it needs one.
- Done:
  the cargo-shadow fix round for all three second-review parts,
  `feat/cargo-shadow-cli` `ce9a17d7d7`,
  354 of 354 tests,
  clippy and `monochromatic-lint` clean (20 new `avoidable-closure` warnings that capture variables).
  Ownership of an output file now comes from one place (`file_owner.rs`):
  a changed file whose inode change time falls in a window around the cargo call is this run's,
  any other is another writer's and is never restored over
  (blind spot:
  a writer within about one scheduler tick of the call,
  which loses no data because every displaced file is kept).
  The restore is a preservation protocol (`RENAME_EXCHANGE`,
  a hard link of what came out before any unlink,
  undo on a mismatch,
  no destructive fallback).
  The Git reader is a lexer after Git's `config.c`,
  proves commits from loose objects or pack entries,
  delegates everything else,
  and falls back with one `git cat-file --batch-check` start;
  a 20-case differential suite against `/usr/bin/git`.
  Not fixed:
  the system config path is assumed to be `/etc/gitconfig`;
  power loss is untested.
  The third cross-vendor review,
  in three parts (all finished within the hour):
  3a,
  the Git reader (`~/temp/agent/reviews/cargo-shadow-sol-3a.txt`):
  "still not equivalent to Git";
  P1s:
  a symbolic-link `HEAD` is read as detached,
  remote `"."` is not the merge spelling returned literally;
  P2s:
  trailing path characters,
  the object reader proves a type marker rather than a readable commit,
  a fatal batch error can drop an existing upstream,
  configuration validation narrower than Git's,
  the system config path,
  Git's dubious-ownership rejection bypassed;
  P3 an unreaped child on a pipe-write failure.
  3b,
  journal and ownership (`-3b.txt`):
  seven P1s
  (the undo deletes an inode another writer can have open;
  per-root manifest recovery fails open;
  `begin` can truncate an earlier transaction's only snapshots;
  a `/proc` lookup failure can call a live owner dead;
  claim-by-rename does not stop directory-name reuse or a live transaction;
  durability ordering incomplete;
  a missing manifest index still authorizes deleting manifest snapshots);
  P2s:
  `ctime` cannot establish ownership,
  `atomic_write` uses a non-exclusive temporary,
  a concurrent deletion is undone.
  3c,
  install and policy (`-3c.txt`):
  "not ready for 0.1.0";
  fix the refusal-preservation path,
  the verification scope of ordinary updates and the placeholder race first.
- Running:
  cargo-shadow's fourth fix round from the three third-review parts,
  with the owner's "nothing should ever wait forever or indefinitely" and the error-at-the-bound answers applied to its waits;
  it writes `cargo-shadow-4a.diff`,
  `-4b.diff` and `-4c.diff` for the fourth review.
- Gate the 0.5.28 base once the index round and the lock-deletion fix are merged into it
  (and the word-level merge pass,
  if it is ready by then;
  it does not hold the release),
  then release 0.5.28,
  install it and time commits against native Git.

0.5.27 ships without the in-process index work
(coordinator's pick under the delegation,
veto open:
the cross-vendor review of it,
2026-11-03T10:09:59-05:00,
`~/temp/agent/reviews/follow-up-index-sol.txt`,
says "safe after named fixes,
not safe as-is",
and its measured wall-clock gain was nil,
a median 139 ms against 137 ms before and 38 ms native on a throwaway copy of main's shape,
so holding back the rest of 0.5.27 for it buys nothing).
Its branch is `integrate/git-policy-cli-0-5-27-without-index-in-process` at `00259fff5e`,
worktree `.claude/worktrees/integrate-0-5-27-without-index`,
gate unit `gpc-gate-release-0-5-27-without-index-20261103T101115-0500.service`
(log `~/temp/agent/gate/gpc-gate-release-0-5-27-without-index-20261103T101115-0500.log`).
The gate on the combined tip `3848f9581d` was stopped by the coordinator at 3908 of 4338 tests passed.

The review's findings on `perf/git-policy-cli-follow-up-index-in-process` (`48d6abb28c`),
queued first for the next free slot,
with red tests first:
- P1:
  `index_reset_file.rs` writes every run to the same `<file>.cli-git-reset` name and removes it unconditionally,
  so two runs can publish each other's partial file;
  the writer takes no `index.lock`,
  so the outer protocol must be shown to hold the live `index.lock` from the read through the install.
- P2:
  an intent-to-add entry whose blob equals the tree's (an empty file) is kept with its flag,
  where `git reset --no-refresh` writes an ordinary entry (`index_reset.rs` 212-215).
- P2:
  `path_is_plain` accepts names Git refuses under `core.protectNTFS`
  (for example `.gitmodules::$DATA` as a link);
  port Git's checks or decline those settings and names.
- P2:
  a malformed EWAH bitmap of a few bytes expands without a bound and can exhaust memory before the fallback
  (`index_ewah.rs` 175-224);
  validate and bound by entry or directory count.
- P2:
  the rewritten file takes the process umask,
  losing `core.sharedRepository` modes.
- P3 (non-Linux test builds,
  paused by the owner's "Ignore Win and mac until I say so."):
  Linux-only fixture imports.
- Also asked:
  SHA-1 reference vectors (empty,
  `abc`,
  lengths around block edges,
  over 512 MiB),
  and tests of later `status` and `diff` after `--no-refresh` and zero stat data.
It found no mismatch in the quiet check for ordinary entries,
and no SHA-1 arithmetic or padding defect.

The combined branch `integrate/git-policy-cli-plain-commit-no-quiet-index-in-process` (tip `3848f9581d`,
worktree `.claude/worktrees/integrate-plain-commit-no-quiet-index`) becomes the 0.5.28 base once those fixes land.
It adds to 0.5.27's content
the follow-up `reset` and `diff-tree` and the `diff-index --cached --quiet` checks in process
(the remaining Git `reset` calls pass `--no-refresh`;
merge conflict with the no-`--quiet` branch resolved as `reset --no-refresh` with both branches' tests kept).
0.5.27's content:
plain commit over unmerged files,
no inner `--quiet`,
the service unit without limits,
the audit rotation fix,
the flaky-test fixes,
the doctor fsmonitor line (`072ac84868`)
and main up to `a9e81617a1`.
0.5.27 waits for its gate,
then merges into main,
is released,
installed,
and measured on main against native Git.

## Open work

Each item names what remains,
where it lives and its evidence.

### Verify 0.5.26 against native Git

Measured with 0.5.27 installed (`~/temp/agent/tools/observe-main-commit.ts`,
host under its usual load):
the handover commit `90f5d99c36` on main at 2026-11-03T15:47:03Z took 324 ms wall,
with no memory-cap hit in the service cgroup and one new major fault;
native Git on a throwaway copy of main's shape takes a median 38 ms
(in-process index branch's measurement),
so the wrapper adds about 0.3 s against the owner's five-second bound.
One sample so far;
the next commits on main add more.

The owner's standing complaint (2026-11-03T07:38:40-05:00 entry):
wrapped commits on main must not take five seconds longer than
native Git.
Before 0.5.26:
on main a commit waited over 50 s in the client's `git add --all -- <path>`,
20 s in `git read-tree --reset`,
over 50 s in a merge's `diff-index`,
and 120 s in a full commit (two Git children of about 60 s inside a 119 s `index.lock` hold;
why each took 60 s is unexplained).
The perf branch changed this (throwaway of main's shape:
wrapped commit median 651 ms to 437 ms,
native 63 ms).
Measured with 0.5.26 installed (2026-11-03T08:27:22-05:00):
the lock commit `c76a6b6d3e` of one file took 66.8 s,
of which the service's follow-up `git reset -q <commit> --pathspec-from-file=-` ran 59 s inside the `index.lock`
hold (`lock_hold_ms=59122`,
`sync_ms=59118`);
the next `git merge` waited 21 s for that commit and then over 40 s in the wrapper's
`git diff-index --cached --quiet HEAD`.
Owner: "Can't we skip the git reset?"
Answer:
the index entries must be updated (or the next commit reverts the committed paths),
but no Git needs to run for it.
Agent on `perf/git-policy-cli-follow-up-index-in-process` writes the committed entries in process,
as `builtin/commit.c` does for a partial commit,
and answers the merge's `diff-index` check in process,
with differential tests against real Git and a measured before and after.
Service journal 2026-11-03T08:55 to 09:20 (main instance):
eight one-file commits landed in 43 to 52 ms each (`sync_ms` 20 to 27,
so the service's `git reset` took milliseconds);
only `3d138d5bc6` (09:19:18) was slow,
with `sync_ms=50370` in the service's `git reset`,
and in the same commit the client's `git read-tree --reset` ran over 50 s and its `git add --all -- <path>` over
40 s.
Three separate Git processes each stalled about 50 s in one commit,
so the cause is shared by them in that window,
not a slow step of the wrapper;
the observer now records each Git process's current system call (`/proc/<pid>/syscall`) to find what they block on.
Observed 2026-11-03T09:21:59-05:00 (commit `53f96ecc24`,
7.7 s wall):
from 0.5 s to 7.0 s the wrapper's Git child (`/usr/bin/git -c core.hooksPath=...`) sat in `wait4` on its own
child `git fsmonitor--daemon start`,
which was on the CPU with a `fsmonitor--daemon run --detach` beside it;
the commit finished half a second after that start ended.
A Git process of another session showed the same wait.
Stopped there,
because the owner's rule is to never chase Git's fsmonitor as a cause
(see "Git's own fsmonitor");
the observation is raised with the owner.
An observed one-file commit on main at 2026-11-03T08:42:37-05:00 took 343 ms
(`~/temp/agent/tools/observe-main-commit.ts`),
with no new cap hit and one new major fault in the service cgroup,
so the slow case is conditional;
the 66.8 s commit was the first after the service started,
at host load 18 to 57.
Found 2026-11-03T08:38:42-05:00 to 08:40:51-05:00 (open, being observed):
plain `/usr/bin/git` on main runs `diff-index --cached --quiet HEAD --` in 12 ms and
`--no-optional-locks status --porcelain=v1 -z --untracked-files=no` in 10 ms,
both inside and outside the shared Bash cgroup
(`~/temp/agent/tools/main-git-timing.ts`),
so the 40 s and 30 s wrapped runs are not Git's own work.
Index 2.8 MB with 20,720 entries;
3 packs and 8,506 loose objects.
Main's commit service cgroup
(`cli-git@fc85fdf7a58f866b7f7a84b44fe0b69a.service`,
started 08:26:19)
had hit its `MemoryMax=1G` (`src/native/service/service_units/cli-git@.service`) 274,736 times,
with 16,125,924 major page faults;
the service slice as a whole 582,767 and 29,926,443.
Inference until observed:
the service's Git children thrash against the 1 GiB cap.
The client-side slow steps ran in `claude-code-bash`,
which never hit its cap (50,897 major faults),
so they need another explanation.
Git's `builtin/reset.c` lines 500-504 refresh the whole index unless `--no-refresh`,
and its "It took N seconds to refresh the index" advice (line 507) is suppressed by `-q`.
The main service's `memory.peak` read 1,073,766,400 bytes,
at the cap.
Owner, 2026-11-03:
"Why did you cap the commit service to the magic number of 1GB of memory w/o evidence?"
Answer:
an agent wrote `MemoryMax=1G`,
`CPUQuota=200%`,
`TasksMax=512` and `IOWeight=50` in `e95a43090c` (2026-11-01T08:26:09-05:00) with no measurement,
apparently carrying the agent-command cgroup rule over to the shipped unit;
its test checked only that the lines existed;
the coordinator merged it without catching that.
The owner chose "Remove all three (Recommended)" for the other limits and "Add it (Recommended)" for a rule.
Done:
branch `fix/git-policy-cli-service-no-memory-cap` (red `45a7590037` and `b539a588bf`,
fix `741d42932e`,
SPEC `474c987ae2`),
unit-template and provisioning tests 31 of 31,
merged into the 0.5.27 integration branch;
rule `LQM` on main `708fbd7e47`.
The shipped-limits audit is on branch `docs/git-policy-cli-shipped-limits-audit` `7387f29a52`
(`doc/planning/cli-git-shipped-limits-audit.md`):
73 limits,
4 cited,
7 partly,
58 uncited,
one dead constant (`DEFAULT_UNPROVEN_OWNER_TIMEOUT_MS`);
conflicts with decisions:
the 1800 s live-holder wait against `doc/decision/cli-git-concurrent-commits.md` lines 151-158 and "Same as the ref
lock",
`COMMIT_WAIT`'s message naming a setting that does not exist,
main's push window and ceiling against "Wait while connected",
retry counts on causes the owner put on the designed-out list.
Fifteen owner questions are in the doc;
not yet asked.
Remaining:
merge the audit doc,
ask its questions,
and measure the service's commits on main once 0.5.27 is installed.
Remaining after that:
measure commits and merges on the real main checkout against native Git,
under the load there is,
and report counts.
Plan:
`doc/planning/cli-git-main-commit-within-native-plus-five.md`.
The gate step still owed for 0.5.26:
confirm the container branch of
`a_fetch_after_a_killed_fetch_clears_the_locks_where_the_holder_is_provably_gone` ran.

### Commits, merges and pushes that never fail

Branch `fix/git-policy-cli-commits-never-fail`,
design `doc/planning/cli-git-commits-never-fail.md` (on the branch).
Design summary:
remove the class-bound kill and move the slow follow-up steps in process;
the client lands a commit itself when the service is absent;
heartbeats instead of `COMMIT_WAIT`;
the service is the one writer of its worktree's branch;
move the ref first and reconcile the index when the lock is free;
a per-worktree hold ref keeps captured blobs from pruning.
The cross-vendor review returned:
`~/temp/agent/reviews/commits-never-fail-sol.txt`,
verdict "not ready to implement as written",
four Critical and five High findings:
a deferred merge checkout can overwrite work created after the merge was checked;
a deferred index update can erase another program's staged content;
the client fallback creates another coordinator without an ownership protocol;
foreign lock reclamation lacks a sufficient proof;
moved-head stacking leaves content replay underspecified;
hold refs do not establish the pruning guarantee;
some items remove an error return but not the liveness failure;
the native fallback is not an end-to-end escape path;
additional refusals and native failures are not classified.
The revised design had a second cross-vendor review:
`~/temp/agent/reviews/commits-never-fail-sol-2.txt`,
verdict "Not ready for approval",
four Critical and five High findings (file headings:
an epoch check does not fence an obsolete worker;
carry-forward does not protect writes through an already-open descriptor;
the off-lock index build can erase unrelated foreign staging;
current ancestry cannot show whether publication already happened;
acknowledgement and transaction completion are conflated;
pending requests need a logical head;
the spool does not establish recoverability after pruning;
the side-ref path does not remove every lock dependency;
remaining Git hangs are acknowledged,
not designed out;
pending-ref non-transmission is an assertion without enforcement).
Foreign-lock reclamation is closed,
and captured-change replay and the refusal classes are substantively corrected.
Third cross-vendor review:
`~/temp/agent/reviews/commits-never-fail-sol-3.txt`,
verdict "Not ready to build as the live transaction protocol",
four Critical and eight High findings
(receipt does not prove publication;
receipt deletion loses deduplication;
the ref fence does not cover obsolete worktree,
index and cleanup work;
index checksum is not provenance;
every displaced inode needs retention;
capture and takeover need ref writes a foreign lock blocks;
the fence ref is repository-wide;
`queue/<n>` is not fenced acceptance;
logical head needs a logical-index model;
index extensions cannot be kept byte for byte;
spool reclamation needs an overlapping-prune proof;
push enforcement,
hooks and deterministic hangs).
Revision `a7bfea3b4a` and `d428b27385` (2026-11-03T08:47:23-05:00 and 08:48:42) removes the features instead:
no live takeover,
receipt,
fence ref,
numbered queue,
checksum provenance or base hold ref;
one kernel lock per worktree held on an open file description that Git children inherit;
a systemd watchdog ends a wedged owner;
acceptance writes no ref;
publication is a compare-and-swap `update-ref`;
retirement writes an fsynced ledger line first.
The agent's five questions for the owner,
held until the fourth review answers whether each narrowing is needed:
lock per worktree or a service-only lane;
a signed commit whose parent moved waits for a new signature;
a Git child that hangs twice on identical input leaves the commit pending with exit 0;
a prune that overlaps a request and deletes after the lane's check is treated as corruption by another tool;
after a crash on a repository with no branch reflog,
a published commit removed by a non-descendant rewrite reappears once.
Fourth review returned 2026-11-03T09:05:02-05:00:
`~/temp/agent/reviews/commits-never-fail-sol-4.txt`,
verdict "not ready to build as the live transaction protocol";
one Critical (carry-forward crash recovery deletes an editor's newer inode)
and five new High (kill-before-acquire and unsupervised inline owners;
the watchdog is a time bound that can repeat a hook;
lockfile contents do not prove ownership;
dry-run expansion differs from the real push;
reachability does not make an old index intent current).
It judges the lock-versus-service question an implementation choice,
and only the signed-commit,
prune-boundary,
deterministic-Git-bug and no-reflog cases owner decisions.
Fifth revision `6d727bcab0` and `42aff9b39c` (2026-11-03T09:08:32-05:00 and 09:09:29):
no kill-before-acquire,
no unit watchdog or takeover,
no lock sweeping after an owner crash,
a permanent ledger,
an unconditional `^refs/cli-git/*` on every push,
deletion only of inodes the lane created and journaled,
a per-path consumption map,
worktree attachment and generation on each request.
Its own trade-off,
not an owner decision:
a wedged owner has no automatic recovery and the person restarts the unit,
which the coordinator flagged against "never stuck".
Its four owner questions:
signed commit with a moved parent;
a Git hang on identical input (the owner's "embed instead of spawning" may already decide it);
an overlapping destructive prune;
no-reflog ambiguity.
Fifth review returned 2026-11-03T09:23:28-05:00 (`~/temp/agent/reviews/commits-never-fail-sol-5.txt`):
not ready;
open:
publication inferred from an incomplete reflog or a reused commit id,
staging intent inferred from stat equality (a tree-created zero-stat entry can be recreated exactly),
the attachment check not synchronized with the mutation,
deferred intents against a foreign successor commit,
descendants of an inline owner holding the lane lock forever,
age cleanup deleting the only saved copy of a carried edit,
maintenance admitted at an idle lane but not covering its lifetime,
a hook's external effect repeated by recovery after an owner death.
Its judgments:
no construction recovers a silent wedge without a time bound and also never repeats a hook's effect,
and manual restart cannot silently satisfy "never stuck";
the deterministic Git hang is settled by "embed instead of spawning";
corruption by another tool after release is on the outside list,
but the wrapper's own maintenance must be coordinated over its lifetime;
signed replay and no-reflog ambiguity are owner choices.
Coordinator adoptions,
veto open:
re-sign a replayed signed commit in the background with the configured signer,
as the service already signs its reconcile merges;
saved copies of carried edits are kept until explicit disposal ("never discard user data").
Owner answers 2026-11-03 (question tool):
stuck step "End Git, hold hooks (Recommended)" (a Git child with no progress for a measured interval is ended and its
step redone;
the person's hook is never ended,
its commit is set aside as pending with a notice and the others keep landing on top);
deferred staging "Update the index (Recommended)" (a deliberate restage to exactly the old entry inside a foreign
lock's window is replaced,
accepted as undetectable);
unclear publication "Land it (Recommended)" (landed with a message naming it and how to drop it).
Sixth revision `766f623dd0`,
`851aa5b446` and `d09c58e7d1` (2026-11-03T09:41:10-05:00 to 09:42:05):
a closure line for every fifth-review finding;
publication evidence only after a reflog anchor recorded before the swap
(no anchor:
owner answer "Land it");
staging targets the current tip's entry;
the attachment check brackets reads of `HEAD` and `logs/HEAD` around the index read under `index.lock`;
inline owners move into a transient scope;
`hook-dispatch` journaled before `update-ref`;
wrapper maintenance runs recorded in `lane/maintenance`.
Still open in it:
the embedded path for a deterministic Git hang,
the exit inventory,
and four owner questions,
picked under the delegation in `75cf58f345`
("Picks made under the owner's delegation").
Sixth cross-vendor review started 2026-11-03T09:50-05:00
(`~/temp/agent/reviews/commits-never-fail-sol-6.txt`,
prompt `commits-never-fail-prompt-6.txt` beside it).
Sixth review returned 2026-11-03T10:33:37-05:00 (`~/temp/agent/reviews/commits-never-fail-sol-6.txt`):
"Ready to build step 1:
executable models and disposable crash fixtures only.
Not ready to build the live protocol."
Its findings (P1 unless marked):
- F1 (P2):
  an intact reflog anchor does not prove the entries after it are complete;
  treat missing evidence as ambiguous ("Land it" with the notice),
  never as "not published".
- F2:
  a dead inline owner's descendant can hold the lane lock,
  and the next owner may stop that scope only after acquiring it,
  so cleanup can deadlock;
  a closed-description descendant can still mutate after a new owner acquires;
  needs a lifecycle authority that reaps a proven-dead owner's scope without the lock.
- F3:
  restarting a quiet owner (question 2,
  A) races a hook's dispatch,
  and a delayed restart can hit a replacement owner;
  restart must be serialized with hook admission and the owner's incarnation.
- F4:
  stopping `update-ref`,
  or restarting its owner,
  leaves a branch lock the wrapper caused and recovery refuses to sweep;
  "stop and redo" and "never reclaim" do not compose.
- F5 (P2):
  the HEAD bracket is a guard,
  not a fence;
  reflog growth is neither a complete switch detector nor proof of a switch.
- F6:
  reading the same state version twice allows a mixed snapshot
  (a materializer pausing before it advances the version makes a plain commit capture `x=1` and revert `R2`);
  needs a seqlock-style in-progress marker before every relevant mutation.
- F7:
  renaming a temporary request over `inbox/<id>` lets a second client replace an accepted request with the same id;
  bind each id to its content without overwrite across inbox,
  queue and ledger.
- F8:
  maintenance records per worktree miss the shared object store
  (a prune started from worktree A deletes objects of B's request);
  needs an object-store-wide,
  crash-safe registry.
- F9 (P2):
  hooks held while others land need an explicit ordering exception,
  including the held request later refusing or succeeding.
- On question 1:
  keep A;
  the coordinator's "record restaged paths" is not adopted as a closed solution;
  versioned restaging events (resulting entry,
  operation,
  consumed or not) are an optional later improvement,
  with its listed cases (`git add` of unchanged content,
  `add -p`,
  `restore --staged`,
  `stash`,
  `commit -a`,
  aliases,
  bypassing writers fall back to A).
- Questions 2 to 4:
  no conflict with the owner's rules,
  given F3 and F4 for question 2.
- P3:
  several operative sections still describe superseded behavior;
  one authoritative rule each for cancellation,
  inline-owner lifetime,
  optional index locking,
  background re-signing and attachment generations.
Revision 7 `e2c2d9d8cc`,
`2b65f37d52` and `b658f5f01f` (2026-11-03T10:45:46-05:00 to 10:47:38):
a closure for each finding
(evidence classes with everything unproven landed with the notice;
children never hold the lane lock and the next owner reaps the recorded scope before mutating;
a `lane/gate` lock shared around every person's program and ref transaction,
taken exclusively by the restarter,
which ends the owner through a pidfd;
one cancellation protocol,
terminate first,
then remove only a lock absent before launch and born inside the child's life;
a seqlock for capture;
a permanent `ids/<xx>/<id>` binding created with `O_EXCL` under `$GIT_COMMON_DIR/cli-git/`;
one maintenance registry per object store;
held requests out of the logical tail);
no new owner questions.
Seventh review returned 2026-11-03T11:00:08-05:00 (`~/temp/agent/reviews/commits-never-fail-sol-7.txt`):
"step 1 remains the right first build.
R7 is not ready for the live protocol,
and passing step 1's models alone would not make it ready."
New P1s from revision 7's own mechanisms:
reaping a recorded service cgroup that may hold the replacement owner (it kills itself);
deleting a lock born inside a stalled child's life (it can be a foreign writer's;
"Do not implement this deletion rule.");
an `O_EXCL` identity binding that can be partial,
or complete with no request published;
an empty or absent maintenance scope counted as finished,
and process-group escape by `setsid`;
the capture seqlock's odd generation held across a hook that may wait for ever;
the gate's restarter dying with an asynchronous stop outstanding,
a wedged shared holder,
and a PID read across namespaces.
Also:
F1 positive evidence must outrank a recorded `swap-failed`;
F5 immediate materialization needs the attachment guard;
F9 one materialization-aware consumption release and versioned completion for superseded held requests;
seven operative contradictions (its section 3).
Its section 4 lists what the models must express and the properties they check.
Revision 8 running:
the design agent writes one compact normative state machine with named transitions,
removes the birth-time deletion rule,
and states narrowed guarantees plainly where only waiting on something outside our control would close them,
with any real owner choice as a question.
The step 1 agent has the seventh review's schedules as additional red controls.
Step 1 building:
branch `feat/git-policy-cli-commit-protocol-models` from the 0.5.28 base `5e8c96de21`,
worktree `.claude/worktrees/commit-protocol-models`;
its first output is the choice of model-checking approach
(a new crate is reported for a pick before it is added).
Step 1 is:
failing models for F2 to F8 with the review's schedules as red controls,
the hook-bypass ordering model,
immutable request,
spool and permanent ledger components with no reclamation,
pure tree comparison and replay with no live mutation.
Not enabled until the models pass:
automatic restart,
production pending publication,
live carry-forward,
retention deletion.
The exit inventory stays a gate.
Interim code in 0.5.26 (index-lock wait and a bounded moved-head loop) stays until the by-construction design replaces
it
(owner:
"Ship interim now (Recommended)").
The 2026-11-03 question-tool answers listed in `SPEC.md`,
"Merges and commits never fail",
still need building:
"Save on a side ref" (commit meets a live foreign ref lock),
"Same as the ref lock" (live foreign `index.lock`:
a commit lands and exits 0 and the index catches up when the lock frees,
a merge saves on a side ref and finishes the checkout when the lock frees),
"Allowed refusals" (the person's hooks and required filters),
"Carry edits forward" (merge over unsaved edits),
and moved head stacks a new commit (amend,
merge conclusion,
revert).
Unexplained and still open:
why `diff-tree` and `reset -q --pathspec-from-file` took about 60 s each in the 120 s commit
(read Git's source and cite file and line,
per the owner's "git itself being OSS" rule under "Owner decisions in force").
Candidate cause,
not confirmed:
with `core.fsmonitor = true`,
a Git command that finds no daemon starts one and waits up to `fsmonitor.startTimeout`,
60 s by default (`builtin/fsmonitor--daemon.c` lines 39-40 in 2.56.0),
the same length as each step.
Confirming it needs a trace of such a step waiting on the daemon,
in a disposable repository with fsmonitor set in its own config;
fsmonitor is now off globally,
so main no longer shows it.

### Plain commit with unmerged files finishes the merge

Branch `fix/git-policy-cli-plain-commit-finishes-conflicted-merge`,
for 0.5.27.
Owner answer:
"Warn and finish the merge (Recommended)":
a plain `git commit` with unmerged files lands as the merge commit with
the markers and warns per file.
Finished at `a06af6102c` (2026-11-03T08:16:26-05:00),
host suite 3606 and binary suite 605 passed on its agent's run.
Remaining:
merge into the 0.5.27 candidate,
gate,
release.

### No wrapper `--quiet` anywhere, and pushes that wait while connected

Branch `fix/git-policy-cli-no-inner-quiet-and-slow-server-waits`.
Owner answers:
"Drop it everywhere (Recommended)" (no `--quiet` on any Git run the wrapper starts:
maintenance,
auto-push fetch,
fast-forward merges,
reset,
clean,
init) and "Wait while connected (Recommended)" (count bytes on the push's network socket as
progress through TCP counters,
because `push_progress.rs` notes `rchar` and `wchar` miss `send` and `recv`;
once the pack is sent keep waiting while the connection is open,
with a line naming the wait).
Built and finished by its agent at tip `e38a7dea6a` (2026-11-03T08:24:13-05:00),
main merged in as `0fec1a97e6`.
The wrapper's own `git push` no longer sets `GIT_HTTP_LOW_SPEED_LIMIT` and `GIT_HTTP_LOW_SPEED_TIME`
(`lfs_bounded_push.rs`),
and since `feat/git-policy-cli-pushes-wait-while-connected-lfs-5xx` (in the 0.5.28 base) the auto-push reconcile fetch
sets none either.
A value the person set reaches Git unchanged,
and Git reads `http.lowSpeedLimit` and `http.lowSpeedTime` itself
(Git 2.56.0 `http.c` lines 453-460,
1477-1478 and 1218-1223).
Red `f0c0ad7112`:
a push whose hook sleeps past the old 60 s cutoff failed with curl's "Operation too slow";
fix `fb1e093809` makes it land;
re-adding the call made it fail again.
Control:
a person's 2-second setting still ends a 12-second silent wait with Git's error.
Checks on the tip:
clippy clean,
host suite 3634 of 3634;
the binary suite had 3 failures at host load average 57,
two passed alone,
and `a_commit_that_waits_for_a_foreign_index_holder_is_attributed_to_that_wait` failed twice alone at load 38-44
and passed at load 11
(see "Known flakes and failures not yet fixed").
Unknown:
whether GitHub sends keepalives (Git's `receive.keepAlive` default is 5 s).
Not exercised:
IPv6 sockets,
real GitHub keepalives,
real `ssh://` or `git://` pushes.
Remaining:
merge into the 0.5.27 candidate,
gate,
release.

### Word-level second merge pass

Branch `feat/git-policy-cli-word-level-second-merge-pass` `8cc36cd4b6`,
not merged.
Built:
`word_merge.rs` on `similar` 3.2.1 (Myers),
equal to Jujutsu 0.46.0 on the stored corpus;
runs in a person's merge,
rebase,
cherry-pick,
revert and `git pull`,
in the isolated merge,
in the auto-push reconcile merge
and in the commit replay;
last recorded checks host 3442 and binary 595 passed,
clippy clean.
Remaining:
the last gate copy failed two tests that main had already fixed
(`worktree_add_plan` dash-h test and a `supervisor_guard` harness race),
so merge current main,
regate,
merge,
release.
Follow-ups the branch records as done (commit `8bc4b70b7a`,
"finish autostash and mergeOptions...",
and the former log),
not verified by me:
finishing `--autostash` and `branch.<name>.mergeOptions` cases as Git does.
Follow-up not known to be done:
`git pull` that merges goes through the isolated merge.
Decisions:
see "Merge auto-resolution" under owner decisions in force.

### Cargo shadow and the Rust catalog tools

Current state at 2026-11-04T03:27:12-05:00:
round 6,
tip `d206fd5321` (2026-11-03T22:26:12-05:00),
472 of 472 tests.
Round 6 answered the fifth review (all four parts "not correct").
Sixth cross-vendor review (`~/temp/agent/reviews/cargo-shadow-sol-6a.txt`,
`-6b.txt`,
`-6b2.txt`,
`-6c.txt`):
all four parts "not correct" again.
Its P1 findings:
6a R1 (a zombie leader can have live threads;
the `/proc` parent map is not a coherent snapshot),
R2 (crash recovery cannot establish that escaped writers stopped),
R3 (the process-tree ownership heuristic can kill unrelated work);
6b 1 (the stopping directory of the directory-entry sync is presumed,
not shown,
durable);
6b2 1 (workspace-wide survivor discovery fails open),
2 (missing group and readiness records still authorize conflicting work),
5 ("outputs durable" treated as "only cleanup failed");
6c R1 (an operational install error bypasses the finish classification),
R2 (group-only recovery unsafe for escaped descendants).
Round 5's process group and round 6's subreaper plus `/proc` scan both failed the same proof,
that every writer cargo started has stopped.
Coordinator's reversal of the agent's round-6 pick,
on the reviews' counterexamples:
containment moves to a delegated control group (membership survives `setsid` and double forks;
`cgroup.kill` and `cgroup.events` `populated 0` make stop and quiescence atomic;
a survivor after a crash is a membership query of a recorded group).
Coordinator's picks under the delegation (veto open,
for the next batched owner question):
where no delegated control group can be had (some CI runners and containers),
cargo-shadow runs with a stated weaker guarantee and a one-line notice naming the missing capability,
never claiming quiescence it has not shown;
"Force at leaving,
else background" has no background carrier in a command that exits,
so cargo-shadow forces what must be durable at its exit barrier and syncs nothing else;
each limit's comment cites the delegation record ("Picks made under the owner's delegation") and the owner's quote,
or a measurement.
Round 7's bar is the P1 set only,
each closed with the reviewer's counterexample as a red-then-green test;
the P2 and P3 findings and the missing-test lists of the four files wait for the round after,
then a re-review of the P1 set alone.
Round 7 done:
tip `94226a1385`,
508 passed and 1 skipped (the ignored escaped-writer fixture helper);
probes,
red tests and the fix in `e543782a4a`,
`939dcb31d1` (11 red),
`a47e630e12`,
`ba620f9e1a`.
Probe findings:
a child of the process's own group works where that group is user-owned (not in a root-owned login session scope);
`cgroup.kill` alone left a thread group whose leader thread exited `populated 1` for 2 to 5 s,
so the stop adds SIGKILL through a re-checked pidfd until `populated 0`;
a `setsid` plus double-fork escapee stays a member,
a same-uid member writing its id into the parent's `cgroup.procs` leaves;
children inherit membership with no `pre_exec` hook.
A run took 6.8 ms median with a group against 6.4 ms in the weaker path (a pidfd wait replaced a 10 ms sleep poll).
systemd.io/CGROUP_DELEGATION (read 2026-11-04):
"Never create your own cgroups below arbitrary cgroups systemd manages,
i.e cgroups you haven't set `Delegate=` in.",
and systemd marks delegated groups with the `user.delegate` extended attribute;
the built default is therefore unsupported.
Coordinator's pick under the delegation (veto open),
for round 8:
a transient user scope with `Delegate=yes` by default (7.0 ms median to start,
15 runs),
a child of the own group only where that group carries `user.delegate`,
else the weaker path.
P1 re-review running in three parts (`cargo-shadow-sol-7a.txt`,
`-7b.txt`,
`-7c.txt`);
round 8 (the queued P2 and P3,
in `doc/planning/cargo-shadow-update.md` "Queued for round 8",
plus the scope pick and a cgroup namespace for same-uid escapes) follows it.
Picks open to veto,
the agent's under the coordinator's delegation:
one workspace lock (`.cache/cargo-shadow/lock`,
non-waiting `flock`,
close-on-exec) instead of a lock per journal;
a child subreaper plus a `/proc` scan,
not a control group (`cgroup.kill` was available here),
to stop every descendant;
one bounded run per process;
runner constants labelled as the implementer's picks (the scan cost 7.9 to 14.6 ms with 894 processes);
HTTP limits unmeasured.
Kept copies follow the owner's warning (see "cargo-shadow kept files" in "Owner decisions in force").
A pnpm 12.16.1 oracle confirmed that pnpm also refuses an unchanged update that leaves a too-young version locked.
Not built:
the background half of "Force at leaving,
else background".
Not done:
a hard-link fault matrix,
a power-loss model,
concurrent directory publication,
the three-writer undo,
the "refused version already in the initial lockfile" counterexample,
network tests for trickling,
stalled and oversized bodies,
add and remove dispatch tests,
and a pnpm oracle for an unrelated refused package during a targeted update.
The rest of this section is the history of the first rounds;
its "Remaining" list is what follows once the reviews pass.

Branch `feat/cargo-shadow-cli` `504a82aa3f`:
182 tests at the last report,
clippy clean,
`monochromatic-lint` 0 errors
(26 `avoidable-closure` warnings left,
18 fixable).
Cross-vendor review returned 2026-11-03T08:40:49-05:00:
`~/temp/agent/reviews/cargo-shadow-sol.txt`,
verdict "Do not rely on this implementation as a policy-enforcement boundary yet",
with P1 findings (a refused version still runs through the `--locked` fallback;
the freshness stamp skips the policy and base checks;
non-frozen installs skip existing entries;
unapproved lockfiles are visible to other invocations;
registry identity is dropped;
the lazy owner lookup misses a downgrade;
unreadable timestamps grant the age exemption;
rollback and recovery can overwrite another writer's lockfile or lose the last copy;
`/proc` liveness;
`--breaking` edits outside the transaction;
the missing-lockfile placeholder;
the recursion-guard kind;
`-m` and `--package=` spellings),
P2 findings (CSV EOF and row width,
unbounded parser memory,
the global cutoff defeating exemptions,
set-aside range) and a P3 README contradiction.
Owner answers of 2026-11-03 (question tool):
"Try asking better. pnpm nowadays just warn and runs when a dep could be updated to something published within a day."
Adopted from that and pnpm 12.16.1 (`pickRespectingMinReleaseAge`;
no message found in its source):
the install step sets a newly needed release under the age limit aside for the newest older passing release and runs.
For a committed Cargo.lock holding a release under a day old (case 1) and one holding a trust downgrade (case 2),
both answered "Set it back, build (Recommended)":
the shadow changes that entry to the newest older release that passes,
prints one line,
and builds;
it stops only when nothing older fits.
Scoped to local runs;
CI keeps the frozen behavior;
operational failures keep warn-and-run with `--locked`.
Done on the branch,
tip `569596c984` (243 of 243 tests,
clippy clean):
every review finding and both answers,
red tests first.
Its nine questions for the owner,
not yet asked:
under `trustLockfile = true` with base `HEAD` a committed merge's lockfile is already base,
so cases 1 and 2 do not fire locally (compare with upstream,
check every entry locally,
or keep);
release age versus trust under `trustLockfile`;
CI still warns and runs `--locked` on a refusal;
the window between cargo writing the live lockfile and the checks;
the restore read-then-rename gap;
strict cutoff with exemptions;
`--breaking` sync failure;
a killed `--breaking` run's manifests;
whether the trust line must name the publisher account.
Remaining,
in the order the agent's report adopted:
1. the review fixes and the two answers (agent running), then a second cross-vendor review;
2. publish `monochromatic-dependency-catalog` 0.2.0 (publishes itself on merge through `cargo-publish.yml`);
3. rebuild file-enforcer-cli with it and run `toolchain:update`,
   then land `912e82a0c7` (`trustLockfile = true` in `rust-catalog.toml`;
   the installed file-enforcer rejects that key until it is rebuilt);
4. first hand-publish of `monochromatic-cargo-shadow` 0.1.0 (`doc/runbook/publish-crate-first-time.md`),
   its entry in
   `toolchain.plan.jsonc`,
   `toolchain:update`,
   check the launcher-backed link;
5. workflow commit:
   move `advisory-core.yml:170` (`task-run rust:verify`) to the shadow's own command and drop the `rust:*` mise
   tasks and `task.ts` entries (owner:
   workflows are editable like any file;
   workflows must not use mise);
6. frozen-install trust check with `trustLockfile` true checks only versions new since `CARGO_SHADOW_TRUST_BASE`
   (unset:
   `HEAD`);
   0 findings over 35 lockfiles at base `9ef2c198cf`,
   5.6 s warm.
The first design used pnpm's default `trustLockfile: false` and refused 22 existing versions;
`doc/decision/rust-dependency-catalog.md` lines 89 to 95 (amendment of 2026-10-29) say only new versions are checked,
the counterpart of `trustLockfile: true`.
A bug found by using the tool (a single-workspace `cargo update` pruned all `trustPolicyExclude` entries) was fixed in
`6a9afd9f59`.

### Conversion of the remaining worktrees and its small fixes

`git cli-git convert-worktree-to-subvolume` ran for the real repository twice (0.5.20,
0.5.21).
At the last dry run with 0.5.21 (`~/temp/agent/convert-dry-run-0.5.21.log`):
183 looked at,
164 subvolumes,
18 would convert,
1 skipped for real content (`follow-ups-059`:
`package/forbidden-strings/core/Cargo.lock` differs from the commit).
The second real run converted 17 and skipped 3:
`fast-snapshot-add` (edited file,
running tests),
`follow-ups-059`,
and a throwaway.
Remaining:
- convert the worktrees still ordinary once their holders are done (busy worktrees are refused by the tool itself);
- the dry-run summary reads "0 converted or completed" beside "would convert" lines;
  it should count what would convert (small fix,
  not done);
- sweep the conversion journal's `cli-git-convert.tmp.<pid>.<nanos>` leftovers,
  and read the migration journal without following
  a link (`Journal::load` uses `read_to_string`);
- count existing worktrees whose `node_modules` links are broken (made from main after main moved to pnpm's global
  virtual store,
  before the package repair shipped in 0.5.21) and ask the owner before relinking worktrees that belong to other
  sessions.

### Worktree add follow-ups

An add of about 60 s is acceptable to the owner and sets no target to stop at ("60s is fine for adding worktrees").
Not done:
overlap the inline trash disposal with the package repair;
on a post-snapshot mismatch of a main source,
keep the snapshot and take the restore path instead of the full copy;
make the snapshot rung settle racily clean entries by content (as the conversion does) instead of falling to the slow
route;
parallelism audit of loops over independent items (branch `perf/git-policy-cli-parallel-independent-work` merged the
first round;
the audit's remaining findings are not recorded here).
Unmeasured:
whether a later `pnpm install` in a copy made without `storeDir` writes into the source's in-tree store.
The owner's statement of accepted risk for races with other programs ("That is not an our problem...",
see "Worktree add and package managers" under owner decisions in force) is to be stated
plainly in command help,
in a short notice when a destructive command runs,
and once for the whole program in `README.md` and
`SPEC.md`;
a search of both files on 2026-11-03 found no such statement yet.

### Service follow-ups

Pre-existing,
from the lock-removal pass,
probably moot if the commits-never-fail design lands (unsure):
the scratch ref `refs/worktree/cli-git-service/<seq>` that a killed landing leaves,
and the `claimed/` scan that is quadratic for a drain of N claims.
`started` and `ready` service lines now carry distinct journal times;
lines of a retiring older service still share one stream timestamp (nothing to fix).

### Gate and tooling

- Gate-slot and campaign-lock claim files are merged,
  with the 2026-11-03 liveness and signal fixes (see "Open review findings" for what remains).
- Mutation campaigns for wrapper code merged since the last recorded campaigns (conversion,
  snapshot add,
  package repair,
  lock
  removals,
  push patience,
  stale ref locks,
  word-level pass,
  commit speed) have no recorded campaign in this page's sources
  (unsure;
  check `package/git-policy/cli/target/verification/` and the sibling handovers before assuming none ran).
  Campaigns are run by the main agent only.
- Controls for `subvolume_platform.rs:76` and `:80` that need no btrfs landed (`b9826f2d3f`);
  other btrfs-only tests still skip in the gate container,
  so a btrfs host run of the ladder is a standing gap.
- Older umbrella items from October,
  still unchecked in the former queue,
  with no later closing record found (all unsure):
  the scanner pathname mutation scope on Windows without a timeout
  (`doc/handover/scanner-native-verification.md`);
  the Windows checks and release-artifact performance gates that trail the cutover;
  long fuzz campaigns;
  mutation campaigns of non-core modules (hooks plumbing,
  auto-push,
  automatic maintenance,
  signals);
  the linter cutover task,
  the slice 6 convergence mutation rerun and the post-cutover non-core campaigns
  (the coordinator's task list names them as 24,
  25 and 29);
  reconcile each against Git state and either close it with evidence or move it here.
- macOS is not a release target (the GitHub release carries Linux assets only);
  the `native_binary` tests still fail there (`/dev/full` is absent,
  several fixture spawns fail with NotFound);
  the lib tests pass on `m1`.
- Android check (`ANDROID_HOME`) now gets its variables from the mpm env fragment;
  the Android-target `cargo check` is blocked by a compile error in `turso_sync_engine` 0.8.2 under nightly-2026-11-01
  ("cycle detected when computing when turso_parser::ast::Select needs async drop"),
  not yet reported upstream or worked around.
- `package/mpm-shadow/cli/tests/shim.rs::passes_arguments_and_exit_code_through_then_prunes` was a pre-existing
  failure when last
  recorded (unsure whether fixed).
  The nested-launch guard and the EAGAIN retry of issue 6259 are covered by package tests but were never exercised
  live.

### After the release: agents at work (2026-11-07, resumed after a usage-limit stop)

- Post-release steps are done (lock on the archive route `c048f3a566`, live commit through 0.6.0, 14 yanks, daily timer on,
  first run `d61dc1934a`); records in the batch plan, `crates-io-footprint.md` and `toolchain-always-latest.md`.
- Decision record `doc/decision/mpm-toolchain-adoption.md`, section "Our seven tools install from GitHub release archives"
  (`0c7b081fbd`), holds the owner's answer: "GitHub, but I'm still not seeing the research I asked you to do on some sort of
  workspace protocol. ... I'm also wondering if pnpm's new rust management capabilities can help."
- Running, each told to report back:
  - research on a `workspace:`-like protocol for Cargo and pnpm's Rust support (doc under `doc/research/`, cross-vendor
    reviewed before hand-back); it answers the owner's 2026-11-05 request that the earlier research missed;
  - the wrapper folds the old `cli-git` data names into the new ones losslessly, so the six "both exist" notices stop
    (`fix/git-policy-cli-fold-earlier-names`; no release yet, and an `-alpha` would not reach this machine through `@latest`);
  - the tokio continuation on `feat/git-policy-cli-on-tokio` (main merge allowed; coordinator merges after its final gate).
- Rename step 3 landed (merge `71498b658f`, record `3af147fcfb`, 2026-11-07T09:01:50-05:00): the settings file is
  `monochromatic-git-policy-cli.config.jsonc`, the link folder `.cache/monochromatic-git-policy-cli-bin`,
  `changeset:version` starts the program by its own name, `earlierExecutables` is gone; details and the names kept on
  purpose are in the rename plan's "Step 3 as built". In main, `command -v git` gives the new link and the
  settings-file notice no longer prints; the six "both exist" notices wait on the fold branch.
- Open incident, separate from the rename: main's `.cache/cli-git-bin`, `mpm-shadow-bin` and `cargo-shadow-bin` were
  found empty (folder times 08:42) before step 3 installed anything, so `git` in main fell through to the global
  `~/.local/bin/git` (also wrapper 0.6.0) until 09:00:59. The fold worktree's link folders were filled at 08:42:49,
  which makes an install in one checkout removing links in another the first suspect; not yet shown.
- Also running since about 09:10 (-05:00): the masking of Git output in `diagnostic_log::debug` under
  `MONOCHROMATIC_VERBOSE=true` (`fix/git-policy-cli-mask-verbose-git-output`), and the diagnosis of the emptied link
  folders. Tried and refused at the same time by the five-agent limit, so they start at the next free slot:
  bounding `core-exec`'s half-hour `depends.unit.test.ts` (`fix/monorepo-task-core-exec-depends-test-time`), and the
  terminal and music-player Containerfiles taking Zig from the lock row instead of `ZIG_VERSION=0.15.2`
  (`fix/containers-zig-from-lock`); both briefs are ready in the coordinator's session.
- Left for later: the old `cli-git@` units of each worktree still run 0.5.34 until their three-hour idle exit (0.6.0 neither
  stops nor removes them, `package/git-policy/cli/README.md` lines 40 to 41); old `cargo/<crate>/<version>` install folders
  stay on disk; `toolchain:install:global` and `toolchain:list:global` disagree on 12 npm-routed tools (not investigated).

### Release batch finished (2026-11-07T13:10:36Z, wrapper 0.6.0 last)

- All 16 crates published in wave order; per-crate runs and times in `doc/planning/one-release-each-batch-2026-11.md`,
  "Release record". One dispatch was refused by the origin-holds-main check and re-dispatched; no other failure.
- Nothing yanked yet (owner: "All after the lock moves"). A post-release agent now does, in order: merge
  `feat/toolchain-always-latest-landing`, switch the seven tools to the archive route, regenerate and install the lock,
  verify wrapper 0.6.0 live with a real commit, yank the 14 older versions (not `forbidden-strings`, `forbidden-regex`),
  enable the daily timer and follow its first run.
- The tokio agent was told the hold on main is lifted; it merges main into its branch, and the coordinator merges the
  branch into main after its final gate.
- Still queued after this: rename step 3 (settings file, link folder `.cache/monochromatic-git-policy-cli-bin`).

### Release batch waiting for a free agent slot (2026-11-07)

- The one-release-each batch (`doc/planning/one-release-each-batch-2026-11.md`) is ready to run:
  registration merged (`a6104c0f64`), the `jsonc-edit` dry run passed (run 42363884326),
  the settle fixes, rename audit and never-at-login guards are merged (`2fa2df5d68`, gate 5069/5069).
- Its agent could not start: five subagents were already running (the limit).
  It starts when one finishes, with the 15 crates other than the wrapper in wave order.
- The wrapper's two preconditions are met as of merge `73173e0f41` (2026-11-07T06:30:33-05:00):
  the cross-vendor review of `2fa2df5d68` (`~/temp/agent/issue-6261/sol-review-audit-8/review.txt`)
  found one test race (closed-stderr settle test), fixed by closing the pipe's read end before the wrapper starts;
  that fix and the owner's "Pair the names" scratch sweep merged to main from
  `fix/git-policy-cli-pair-earlier-scratch-name` (tip `7e2284d7d9`, container gate 5071/5071).
  The wrapper (0.5.34 to 0.6.0) can go after the other 15 crates.
- The earlier per-user scratch (`<state>/cli-git/scratch`) is still not swept; the owner's pick covered the repository scratch only.
- The daily toolchain timer must not run for real until the batch ends (its agent was told).
  Its agent finished: code merged as `c6daf6161e` (plus doc commits `baac0829e2`, `798360d773`);
  units `toolchain-daily-update.service` and `.timer` installed but disabled; rehearsal 20 min 51 s in a throwaway worktree.
  After the batch, in this order: merge `feat/toolchain-always-latest-landing` (plan and lock to latest, plus the
  `bincode-1` catalog that keeps the placeholder bincode 3.0.0 out), run `mise run toolchain:timer:install`,
  start `toolchain-daily-update.service` once and read its journal. Main's lock stays on bun 1.3.14 until then
  (installed bun matched it when checked shortly before 2026-11-07T07:14:14-05:00).
  Deviation: it updated the real global plan and lock (outside the repo) during the rehearsal, so the global toolchain
  already moved. A guard refused `git restore` of two formatter side effects; it saved the diff to
  `~/temp/agent/tal/format-side-effects.patch` and force-removed the worktree.
  Follow-ups it found: terminal and music-player Containerfiles still hard-code `ZIG_VERSION=0.15.2`
  while the lock moves to 0.17.1; `llama.cpp` stays pinned until `mpm-shadow` can pick prereleases;
  `core-exec`'s `depends.unit.test.ts` ran over half an hour (stopped, not investigated);
  scratch branches `feat/toolchain-always-latest` and `feat/toolchain-always-latest-code` remain.
- The batch agent started after merge `73173e0f41` with the 15 crates; "go wrapper" waits for the background-prompt fix below.
  It runs on the default subagent model because an `opus` override was refused by a permission rule;
  the owner, asked, answered "Keep it running (Recommended)".
- While tracing the dialog, a pattern read of another process's environment (`CLAUDE[A-Z_]*`) printed an API key and a
  messaging token into the session log; the owner answered "I'll rotate it" and, asked whether to add an `AGENTS.md`
  rule for exact-name environment reads, "Don't add it".

### Tokio branch: owner answers (2026-11-07)

- jiff newer than the locked 0.2.24 was refused by the trust policy through optional `defmt-parser` 1.1.0
  (publisher change). Owner, verbatim: "I'm approving the publisher change for jiff."
  The tokio agent records it in `doc/planning/git-policy-cli-full-tokio.md`, adds a narrowly scoped trust entry
  with that sentence as its reason, and moves jiff to the newest allowed version on `feat/git-policy-cli-on-tokio`.
- G1 of `doc/audit/tech-safe-file-saving-published-crates-vet-2026-11-07.md` (checked-rename fallback where a filesystem
  has neither `RENAME_NOREPLACE` nor hard links): facts gathered first (no in-kernel filesystem read lacks both;
  s3fs-fuse does; with the library only, merge-concluding commits fail with "Operation not supported").
  Asked, the owner picked "Keep the fallback (Recommended)" (10 lines plus a 15-line test).
- The first tokio agent ran out of context with T1 to T3, the crate swaps and part of T4 done (head `8383a12bf7`,
  gate 5094/5094). A second agent continues on the same branch from T4's remainder through T13 and the
  `private_storage` replacement on `tempfile::persist_noclobber` with that fallback, and fixes the defect it found:
  the service journal publishes by hard link with no fallback (`service/service_busy.rs:149`), so a state directory
  that refuses links fails service commits today. It was told not to repeat the rules the first agent bent
  (tests and fixes in one commit, a direct `cargo fetch`, one `pkill -f`, hand-reverting edits after a guard refused
  `git checkout --`).

### Background Git never prompts (owner pick 2026-11-07)

- Cause and probe: `doc/troubleshooting/git-askpass-dialog-despite-terminal-prompt-off.md`.
  The dialogs the owner saw came from the LFS agent's test with `/usr/bin/git`, not from the wrapper.
- Owner: "Never prompt (Recommended)" for the wrapper's background Git, and "Neither, rule pkg-local." for agents.
- Built on `fix/git-policy-cli-background-git-never-prompts` (tip `6b01de5bf0`, gate 5078/5078, Clippy clean):
  `never_prompt_overlay()` in `child_environment.rs` covers the service's Git and git-lfs runs; forwarded commands
  are unchanged; package rule `NDG` in `package/git-policy/cli/README.md`. Merged to main; "go wrapper" sent, so
  0.6.0 carries it. The push-record leak needed no fix: its writer was deleted in `69d275d92b` (2026-11-01), the
  3 records holding a credential predate that, and a guard test now checks no wrapper file keeps one.
  Open, minor: `diagnostic_log::debug` writes local Git output to stderr unmasked under `MONOCHROMATIC_VERBOSE=true`.
- `package/config/lfs-r2-worker` gets the same package-local rule after "unpause lfs".
- Second fix on the same branch (sent by 2026-11-07T07:11:35-05:00 by the coordinator after the LFS report):
  push records (`.git/monochromatic-git-policy-cli/push/*.last-pushed.json`) store failed push output with URL userinfo
  unmasked while the terminal relay masks it; red test, shared masking, existing records untouched.

### LFS credential work: finished and paused (owner: "pause the lfs thread until I say unpause lfs")

- The LFS agent's report and full inventory: `doc/troubleshooting/lfs-credential-in-url.md`,
  merged to main as `05199a8d8e` (credential guard task, `url-userinfo-credential` scanner rule, credential-helper docs).
- Not done, by the pause: local migration off `lfs.url`, rotation, Worker auth change.
  Its open owner questions (auth design A to F, anonymous reads, credential helper, copy classes, wiring the guard
  into the wrapper) wait for "unpause lfs"; ask only the real decisions among them (issue #6262).
- It printed a config key name that embeds the old token into its own transcript: one more copy for the inventory.
- It left an unbounded `du` over home folders running for 68 minutes (PIDs 3421138, 3421137); the coordinator stopped it.
  Its worktree `.claude/worktrees/lfs-credential-fix` is still there.

### Repository on the 860: plan revised, owner questions next

- The symlink plan is committed (`4f430a0683`, `837fc9a6eb`) in `doc/planning/repository-on-the-860-drive.md`; nothing moved.
- A cross-vendor review of it runs (`~/temp/agent/860-review/sol-review.txt`); fold its findings in.
  Then ask the owner only the items among Q1 to Q9 that are real decisions: the answer changes what happens,
  no owner decision or measurement settles it, and it is destructive or a matter of preference.
  Settled or reversible items with a clear ranking are adopted and recorded in the plan, not asked.
  Owner, 2026-11-07: "Overloading me with non-question questions will only make things worse."
  Notices and disclosures go in plain text, never as question-tool options.
  Which questions to ask and how to word them is open in issue #6262
  (owner: "This is about how you ask questions and which questions to ask. Your wording is bad.");
  the owner settles it in another session, so no rule change until then.
- The review is folded in (`b89d384cca`, `1a462e1eb5`): 21 findings fixed, narrowed or marked root-only unverified;
  the former Q1 to Q9 are adopted in the plan with reasons, open to the owner's veto.
  One real owner decision remains and is not asked until the move is scheduled:
  the cut-over needs a bounded window with commits refused, against "Commits should never fail"
  (plan's ranking: a bounded window at an owner-chosen idle time with a stated maximum, after timing the first pass).
- Incident: the plan agent ran `btrfs property set -ts /var/home/user/Monochromatic ro true` on the real main checkout
  by mistake at about 06:51 and cleared it (`ro=false` confirmed 2026-11-07T06:52:28-05:00).
  Every worktree's Git dir lives in that subvolume, so commits anywhere would have been refused for about a minute.
  The coordinator re-checked after the report: `ro=false`, no `*.lock` files under `.git` dated 06:50 to 06:54, no probe
  processes left, main's `git status` clean. A refused attempt in that minute can be neither confirmed nor excluded.
- The plan's scratch subvolumes in `/var/home/user/temp/agent/dangling` (`svfix`, `svfix-snap`, `svfix-snap-ro`,
  `svfix-asm` and two nested ones) are deleted, without sudo: the owner said "I don't think they need sudo.",
  and emptying then `rmdir` works (kernel feature `rmdir_subvol`). The plan's reason for adding
  `user_subvol_rm_allowed` is corrected to the wrapper's one-step subvolume removal.
- Its guard fixture mounted a tmpfs at `/dev/shm/dangling-world` inside `podman unshare`, outside the scratch rule;
  the mount ended with the namespace and the empty folder was removed.

### Known flakes and failures not yet fixed

- Shipped in 0.5.27,
  seen live 2026-11-03T12:00 on the 0.5.28 base:
  `git commit -m <subject> -- <path>` during a merge,
  with the conflict resolved in the file but not staged,
  landed a one-parent commit of that path (`f1f78f7920`,
  as SPEC designs:
  "Named paths during a merge: Git refuses; the wrapper lands them and the merge is carried on")
  and printed that the path "still has conflict markers,
  and they were committed",
  though the committed file has none.
  `service_prepare.rs:445` lists every path that was unmerged in the index before the commit,
  whatever the content holds.
  Its advice (another `git commit -- <path>`) also leaves the merge open.
  The merge was concluded with a plain `git commit` (`cfd30ccd18`);
  a corrective comment is on `f1f78f7920`
  (<https://github.com/Aquaticats/Monochromatic/commit/f1f78f7920e379b3d45893e25739a689be60331e#commitcomment-207965325>).
  Fix running with the word-level agent.
- `a_commit_that_waits_for_a_foreign_index_holder_is_attributed_to_that_wait` failed twice alone at load 38 to 44
  (`objects_ms=6760`,
  `attempts=2`:
  the stand-in holder let go before the commit's wait) and passed at load 11;
  an agent is making the holder release only once the commit is observably waiting,
  on branch `fix/git-policy-cli-foreign-index-holder-test-waits-for-the-wait` (started 2026-11-03).
- `lfs_hang::two_worktrees_committing_at_once_with_a_slow_upload_both_finish_without_waiting_for_each_other` failed
  once in a full
  binary run,
  then passed 15 of 15 alone and in later full runs.
- `fault_limits::a_descriptor_limit_hit_at_any_open_leaves_a_recoverable_repository` ran over 60 s in two separate
  runs
  (passed;
  not looked at).
- Found 2026-11-07 by the published-crates study (`doc/research/wrapper-git-free-jobs-and-published-crates.md`):
  `the_runner_waits_for_a_slow_job_and_contains_a_panic` fails (`Busy` instead of `Finished(7)`) in a selection run of
  the `systemd_dbus`, `standards_review` and `source_guard` tests on unchanged code, and passes alone twice;
  an ordering or contention defect of the suite, not yet looked at (the full container gate passes it).
- Found 2026-11-07 by the same study:
  `zbus` 5.19.0 and 6.x do not compile on the pinned `nightly-2026-11-01`
  (`E0391: cycle detected when computing when ...::signature::child::Child needs async drop`)
  and do compile on `nightly-2026-10-28` and stable 1.99.0 (scratch crate).
  `package/nested-wayland-session/cli` locks `zbus` 5.19.0, so it may not build on the pinned toolchain;
  not built yet. The study's agent is bisecting the nightly and checking newer ones.
- Rule for all of these:
  the owner's "Fix flaky tests properly."
  (recorded 2026-11-02T21:49:28-05:00).
  Never rerun to chase one.

### Questions waiting for the owner

- Container disk weight:
  about twenty other packages and two workflows start containers at the default weight;
  the weight constant has several copies;
  `vm-builder`'s rootful privileged bootc-image-builder run has no CPU,
  memory or process limits.
  The six packages in the owner's answer "Both flags in all six" are done.
- Delete the thirteen merged branches with nondescript names
  (`fix-push-semantics`,
  `fix-review-log-common`,
  `fix-takeover-hook-race`,
  `integ-20261101`,
  `follow-ups-059`,
  `normal-conditions-gaps`,
  `fix-service-queue`,
  `integrate/cli-git-native-command-parser`,
  `perf-harness-service`,
  `test-hang-bounds`,
  `fix-lfs-hermetic-20261101`,
  `integrate-slices`,
  `lint-lockfiles`),
  locally and on origin:
  the auto-mode classifier refused ("Git Destructive").
  Eighty-eight other nonconforming branches belong to other work (audit `~/temp/agent/branch-name-audit.txt`).
- Whether an operator escape hatch should force the native route when the commit service misbehaves
  (the owner refused install switches;
  a debug override would be a different thing).
  Not built.
- Windows and macOS CI effects of the shared pnpm `storeDir` and `virtualStoreType: global` are paused per the owner.
- Whether the 2026-10-13 standing authorization to bump and publish the wrapper still holds beyond that session (see
  working rules).

### Housekeeping

- Remove the stale detached worktrees listed by `git worktree list` under `.claude/worktrees/` when their evidence is
  no longer
  needed (needs the owner's word for deletions);
  the stale integration branch named under "Unmerged branches and worktrees of this effort" goes with them.
- Test services that crash on purpose left hundreds of coredumps a day;
  fixtures expected to abort were told to set `RLIMIT_CORE` 0
  (test harness only).
  Whether that landed is unconfirmed:
  check `coredumpctl` after 0.5.26.

### Final re-review

The owner's words,
2026-11-01T09:22:48-05:00 (when the entry was written;
the owner spoke earlier that day) on the original entry ("Final re-review"):
"You yourself do a re-review of the entire thing when all tasks are drained.
During the re-review:
Assume nothing on disk ever makes sense and I answered every question with via magic 8 ball."
Reading,
revisable by the owner:
it starts when every delegated agent has reported and its branch is merged or rejected,
gates are green,
no release is in flight
and every task-list and handover item is done or recorded as blocked with its reason.
The coordinator session does it,
not a delegate (a cross-vendor pass may run alongside,
never instead).
Scope:
the `git-policy/cli` wrapper and everything built for it (crate,
service,
embedded linter seam,
fuzz and mutation sidecars,
gate and release tooling,
planted controls,
docs).
Every comment,
doc,
test name,
commit message,
config and recorded result is an unverified claim checked against behavior run in a
throwaway fixture or against the deciding source;
a test counts only after a planted fault shows it failing;
recorded owner decisions carry no evidence of being right,
so judge each on its merits and ask the owner with options,
pros,
cons
and a ranking where judgment differs;
never change decided behavior without asking.
Output:
findings report in `doc/audit/` with evidence per finding,
clear bugs fixed with proofs,
flagged decisions to the owner.

## Open review findings

Source:
`/var/home/user/temp/agent/reviews/p1-reconciliation.md` (read-only,
160 P1 findings across 40 returned reviews;
131 fixed in main,
8 fixed only on an unmerged branch,
6 withdrawn or void,
1 accepted limit,
11 open,
3 unknown).

The audit-rotation race and the four gate-slot findings were fixed with red tests on 2026-11-03:
the wrapper fix `fix/git-policy-cli-audit-rotation-race` `2eb2cf1bfc` (lock-free:
one rename takes the name,
a mistaken take goes back by `hard_link`,
a rotation error never costs the record;
host 3611,
binary 603) is merged into the 0.5.27 integration branch and ships with it;
the gate fix `fix/mutation-test-cli-cargo-mutants-claim-liveness-foreign-namespace` `db01385349`
(no age rule,
foreign-namespace owners found in `/proc` by inner id and start time,
termination forwarded to the whole process group,
the legacy `flock` held beside the claim) is merged into main;
its trade-offs are in `package/mutation-test/cli-cargo-mutants/README.md`.
Still open from them:
- a kill between the rotation's two renames leaves an orphan `.moving-*` file that audit readers do not read
  (no line lost);
- the next rotation still replaces `.1`,
  dropping older audit history as before;
  a question for the owner (does "never discard user data" cover the audit history?),
  folded into the shipped-limits audit;
- a non-file at `slot-N.lock` makes `holdLegacyLock` return `absent`,
  which the gate treats as an earlier-release holder forever.

Accepted limit with a documented remedy,
no code fix:
downgrading to 0.5.5 revives the per-user writer
(`baefc60463`;
remedy:
stop `cli-git@*.service` and `cli-git@*.socket` first).

On the unmerged copy-in-service branch `worktree-agent-ab3dd6294f9d91284` (head `fe2081f2a1`,
19 commits not in main):
9 open (`review-wtc` findings 3,
4 and 6 and `review-wtc/v3` findings 1 to 6) and 8 fixed only there.
The branch may be moot:
the btrfs ladder replaced the copy on btrfs hosts,
but main still has the older lock-based copy modules,
which no review covers.
No later decision to merge or drop it is recorded.
The v3 decision "do not fix these on the current
repository-wide layout" (2026-11-01) stands until then.

Not P1-labelled:
the four Critical findings of the commits-never-fail review are open by construction (see "Commits,
merges and pushes that never fail").

## Picks made under the owner's delegation

Owner, 2026-11-03 (about 09:45-05:00):
"I have to be away for around 8 hours. Pick options as if I had picked them and invite veto when I say I'm back."
Clarified by the owner:
"\"As if I had chosen\" means what I would have picked most likely if I'm always here."
So each pick is the option the owner would most likely have chosen:
the ranked-first option where its ranking follows the owner's standing rules,
which matches every answer the owner gave on 2026-11-03.
Every pick below is the coordinator's,
made under that delegation;
each stays open to the owner's veto and is listed for the owner on return.

### Shipped-limits audit (`doc/planning/cli-git-shipped-limits-audit.md`, fifteen questions)

1. Which limits need a citation:
   B,
   limits that can change a command's result or duration;
   polls,
   display caps,
   chunk sizes and reader fallbacks are classed once as exempt with the reason (they still follow MGN).
2. Index lock holder:
   A now (a proven-alive holder is waited for with no bound and a line naming it;
   an unproven owner gets 1000 ms of backoff and then a diagnostic;
   the dead constant goes),
   then B with the commits-never-fail design (the owner's "Same as the ref lock").
   Follow-up pick (built on `feat/git-policy-cli-index-lock-and-service-waits` `82633304d0`):
   an unproven owner is waited for with no bound like a proven one,
   with a repeating line that says what is known and that removing the named file lets the command run
   if no program uses the repository;
   a dead owner (a pid file naming a gone process) still ends the wait at once.
   Its cost:
   a lock left by a killed program with no pid file now waits until a person removes it.
   Built 2026-11-03 on `feat/git-policy-cli-index-lock-and-service-waits`, with three differences the owner may veto:
   the 1000 ms is cited from a measurement of Git's own commands (at most 100 ms, ten times that),
   a dead owner ends the wait at once,
   and the service's own hold keeps a measured 150 seconds (`UNASKABLE_HOLD_AGE`)
   because live holds ran up to 120 seconds.
3. Retry counts:
   C with the commits-never-fail design (stack on the new head for every moved branch);
   the interim loops stay until then ("Ship interim now").
4. Git child steps:
   B,
   end only on no progress with the push's signs and no wall clock,
   matching the owner's "End Git, hold hooks".
5. Pushes:
   A,
   "Wait while connected" whole:
   no stall window for a connected push,
   no ceiling,
   no low-speed abort.
   Built:
   `feat/git-policy-cli-pushes-wait-while-connected-lfs-5xx` `3ae733155f`,
   merged into the 0.5.28 base (`5299b6d102`);
   connected means a process of the push holds an `ESTABLISHED` TCP socket (read through sock_diag).
   Its three questions,
   picked:
   the 30 s window of a push with no open connection stays and is measured once logging records quiet time
   (its option C over A,
   because A leaves the value uncited);
   its own extension of the connected wait to the reconcile fetch and the LFS upload stays
   (both are part of the push,
   and a clock there brings back the failure the owner's answer removed);
   `QUERY_CEILING` (a wall-clock stop on read-only queries such as `git status`) moves to picks 4 and 9,
   and the stop grace and the 4 MiB output cap are measured once logged.
   Cost of the rule,
   from its agent:
   a live remote that holds the connection without answering holds that repository's pushes until the remote or TCP
   ends it.
6. Waits on the service:
   A,
   heartbeats and no client wall-clock bound;
   the `COMMIT_WAIT` message naming a nonexistent setting goes.
   Built 2026-11-03 on `feat/git-policy-cli-index-lock-and-service-waits`:
   a heartbeat a second, five seconds of silence asks again with the same request,
   and the start waits on the service manager's `failed` state where the client can ask it
   (the open questions are in the audit doc).
7. Foreign-owner patience:
   C,
   measured from the logs.
8. Parallelism caps and memory estimates:
   A,
   measured.
9. Deadlines of tools that are not Git:
   A,
   end only on no progress.
10. Saved conflict copies:
    C,
    defaults set from the measured growth.
    Outcome:
    no saved copy exists yet in any of the 11 repositories under the home directory,
    so there is no growth to measure;
    the agent's O3 is taken
    (keep the owner's 2026-11-02 "Prune by both age and size." with 30 days and 1 GiB,
    measure,
    then set the numbers),
    because the owner chose pruning knowing a saved copy can be the only copy
    (`doc/handover/cli-git-logging.md`,
    "Saved conflict copies");
    nothing can be pruned for 30 days.
    Question for the owner on return:
    "never discard user data" was first recorded 2026-11-03T00:08:39-05:00 (`d4aab88c70`,
    snapshot leave-out),
    after the pruning decision;
    does it override pruning a saved copy that is the only copy of a merged file?
    Options and ranking:
    `doc/planning/cli-git-shipped-limits-audit.md`,
    "Only-copy finding".
11. Audit and timing logs:
    C,
    audit generations kept up to a measured size and never dropped while they name a saved copy that exists;
    timing logs keep one generation.
    Built:
    `feat/git-policy-cli-saved-copies-and-audit-generations-measured` `5f453574e5`,
    merged into the 0.5.28 base;
    4 MiB derived from the measured busiest day (about 103 KB a day) times the 30-day copy age.
12. LFS server errors:
    HTTP 5xx classed as outside our control (a server fault,
    like "Network down"),
    retried by the service's push retry plan alone (option B's single mechanism).
    Built on the same branch as pick 5;
    a typed `git push` has no plan,
    so it now ends at the first 5xx with the explanation.
13. Service housekeeping:
    A,
    measured from its journal and timing logs.
14. Lock-removal margins:
    A,
    measured on this host.
15. Free-space preflight:
    A,
    computed from the entry count with a measured cost per entry.
Implementation queue (agents start as the five slots free;
measurements under the load there is,
never synthetic):
pushes and LFS (5,
12);
Git child and tool no-progress ends (4,
9);
index lock interim and service waits (2,
6);
audit generations and saved copies (10,
11);
the measured citations (1,
7,
8,
13,
14,
15).

### Commits never fail, revision 6 (four questions)

The questions,
the reasons and the coordinator's candidate are in `doc/planning/cli-git-commits-never-fail.md`,
"Questions for the owner",
on `fix/git-policy-cli-commits-never-fail`.

1. Staging a path back to its consumed entry while commits are pending:
   A (the commit lands without that path;
   a notice says so).
   The coordinator's candidate goes to the sixth review:
   record paths the person explicitly restages through the wrapper while requests are pending,
   so a plain commit gives native Git's result.
2. A quiet service owner of our own making:
   A (the waiting command restarts the unit after the measured interval),
   shipped only once the interval is measured.
3. A hook with an external effect on "prepared",
   after an owner death:
   A (run `update-ref` again;
   the hook may post twice).
4. The instant inside Git's `checkout` between the index write and the `HEAD` change:
   A (accept a rare staged change on the new branch).

### Commits never fail, revision 8 (two questions)

In `doc/planning/cli-git-commits-never-fail.md`,
the owner-questions section,
on `fix/git-policy-cli-commits-never-fail` (`3beecf2368`,
`a7b917e47e`).
These are options of the owner questions,
not the state-machine transitions of the same names;
revision 9 renames them.

1. A Git child that ignores the terminate signal:
   option A2 (end it with the kill signal,
   the owner's "End Git"),
   kept as an accepted narrowing after the eighth review:
   it can leave a lock that only the person clears.
2. A quiet owner that holds the gate:
   option B2 (restart it) picked first,
   then withdrawn for option B1 (hand it to the person with a line naming it),
   because the eighth review showed a restart can end a person's hook that starts after the "no program running"
   observation (R8-1).

### Commits never fail, revision 9 (three choices)

In `doc/planning/cli-git-commits-never-fail.md`,
"Questions for the owner",
on `fix/git-policy-cli-commits-never-fail` (`7afb60a024`).
Options of the revision 8 questions are renamed A-keep,
A-kill,
B-hand and B-restart.

1. What the stall rule (X1) may cancel:
   C-kind (by command kind,
   `update-ref` exempt) over freezing the dispatch inputs and over re-checking at cancel time,
   because of "End Git,
   hold hooks";
   a stalled `update-ref` with no hook,
   which needs a Git bug,
   stays named and durable for the person.
2. When a held request's hook runs:
   once per version (H-version) over once per build,
   so a slow hook is not rerun each time another program moves the branch,
   and nothing waits without end.
3. Merge order:
   check out first,
   then swap (M-A),
   over swap first,
   whose window lets a plain commit capture the reverse of the merge ("never discard user data").

### Commits never fail, revision 11 (five choices)

On `fix/git-policy-cli-commits-never-fail` (`9c2b2ec0ab`,
dated in `e6199d4e4f`).

1. A merge touching a path the person has staged:
   decided by the owner (question tool,
   answered before 12:56:51-05:00):
   "Merge anyway,
   unstage x" (P-carry;
   the staged content becomes an unstaged edit merged onto the result),
   over refusing at once like Git.
2. A `git switch` finishing after the lane's last check during merge file writes:
   decided by the owner:
   "Allow the rare leftover" (W-detect;
   the owner was told a `git commit -a` in that window could commit the merge's files to the wrong branch).
3. Retention against late maintenance:
   G-order (the connectivity check,
   then a re-read of the maintenance registry,
   then the release;
   the hold ref only as extra protection),
   coordinator's pick.
4. A live owner's retry after a proven store failure:
   N-silent (no notice on an ordinary held-lock retry),
   coordinator's pick,
   recorded as a concession.
5. A hook rerun after repeated owner deaths:
   H-episode (one rerun per recovery episode,
   with a notice),
   coordinator's pick.

### Commits never fail, revision 10 (five choices)

In `doc/planning/cli-git-commits-never-fail.md`,
"Questions for the owner",
on `fix/git-policy-cli-commits-never-fail` (`eed561efaa`).

1. A stranded `index.lock` the lane's own dead process made:
   never removed (R-never),
   reported with the incarnation that made it,
   over pinning the inode and unlinking,
   which keeps the pathname race of the shipped sweeper.
2. A merge that touches a path the person has staged:
   it changes nothing and waits (P-wait,
   Git itself refuses this merge),
   over turning the staging into an unstaged edit.
3. A `git switch` finishing between the merge's guard and its file writes:
   detect after and restore from pinned files (W-detect),
   over never writing merge files automatically,
   because of "Auto merging should never fail."
4. Keeping the recovery copy against a late `git gc`:
   a hold ref (G-root),
   over making maintenance wait on a live lane;
   the prune view including `refs/cli-git/*` must pass as a step-1 experiment first.
5. Merge order:
   check out first,
   then swap (M-A),
   kept after the ninth review's repairs.

### Shipped lock removal (four questions)

On `fix/git-policy-cli-never-removes-a-live-lock` (`9f92dae98c`).

1. A lock a reboot left without a pid file is now reported with the `rm` line,
   not cleared:
   kept (picked),
   because the wrapper sets `core.lockfilePid` for every Git it starts,
   so only a kill between the lock and its pid file,
   or a foreign Git without the setting,
   leaves one;
   a rule keyed to a recorded boot identifier could not cover locks made before the record.
2. The sweep after a stopped child judges every reference lock of the repository (picked),
   as each fetch and push already does,
   over only those born during the child's life.
3. Its notice goes to the diagnostic log on standard error (picked,
   unchanged),
   because the sweep runs on a background thread.
4. Its new lines over 120 characters:
   left for the lint-debt pass (task 88).
5. The ninth design review's R9-S:
   the removal step (rename the lock to a private name,
   compare,
   remove or put back),
   shipped since 0.5.26,
   can break mutual exclusion:
   if between the judgement and the move another cleaner removes the judged lock and a live Git makes a new one,
   the move displaces that live lock,
   and a third Git can take the path before the put-back,
   so two Git processes hold the same ref lock.
   No Linux call removes or moves a path only if it still names a given file.
   Picked:
   keep removal only on a proof about the holder,
   with the second identity check just before the move,
   and ship it in 0.5.28,
   because it removes strictly fewer locks than 0.5.27 does;
   over never removing a reference lock,
   which makes every Git the wrapper ends leave a lock the person must `rm` before the next fetch,
   push or commit ("Commits should never fail.").
   Question for the owner on return:
   accept this residue,
   or never remove reference locks (the design's X1 already says the wrapper never removes one).

### Shipped-limits follow-ups

- After the citations:
  picked in `doc/planning/cli-git-shipped-limits-audit.md`,
  "Picked for these follow-ups";
  built (`e314460f79`).
- After the index-lock and service waits (`008b74d3a3`):
  an unproven lock owner is waited for like a proven one,
  because an editor-held `git commit` would otherwise make ours fail after 1 s;
  poll when no inotify watch is available;
  ask the service manager to start an inactive unit after `START_WAIT`;
  keep a scratch file from being aged out during a wait longer than a day.
- For the owner to approve,
  not picked:
  the margin above the longest recorded quiet gap for each quiet window (picks 4 and 9),
  and the 150 s hold-age margin (pick 2).

### Fallback to the installed real Git (four questions)

In `doc/planning/cli-git-no-self-invocation.md`,
"Falling back to the Git the commit service was set up with";
built on `feat/git-policy-cli-falls-back-to-installed-git` (`6453c380b1`).

1. Fall back at all:
   yes (picked) over keeping the stop,
   because a found wrapper that cannot run Git is a failure the owner's rules name,
   and the one-line notice keeps the cause visible.
2. Which record names the real Git:
   the commit service's unit file,
   whose `ExecStart` carries `--git <path>` (picked),
   over a new record written on every successful `PATH` search
   (survives logout but adds a write to every command),
   the version-cache entries (several,
   unordered)
   and a fixed `/usr/bin/git` (a guess).
   Limit:
   the unit lives in the runtime directory,
   so before the first commit of a login,
   and after the last logout,
   the stop stays.
3. The service's own checks of its `--git`:
   left as they are (picked);
   the wrapper always writes `--git` from its checked path,
   so tighter checks guard only older or hand-written units.
4. How often the notice prints:
   on every command (picked),
   because printing once per login needs a file that outlives the process and a rule for when it expires.

### Word-level second merge pass and pull (three questions)

In `doc/planning/cli-git-merge-auto-resolution.md`,
the section dated 2026-11-03T11:45:26-05:00,
on `feat/git-policy-cli-word-level-second-merge-pass` (`2a37bb7682`).

1. A stash whose autostash apply conflicts is lost:
   the shadow repository keeps `refs/stash` and the unmerged entries,
   where Git keeps `stash@{0}: autostash` and `UU` entries.
   This also happens to a person's own `git commit` after `merge --autostash`,
   which has shipped.
   Picked:
   B now (a dry-run of the stash apply hands the wrapper's finish back to native Git when it would conflict),
   then A after the index writer's third round (carry the stash,
   its objects and reflog entry,
   and the unmerged stages back into the real repository under the landing lock),
   which also fixes the shipped path;
   over leaving it documented,
   because of "never discard user data" and "Fix even pre-existing issues of the pkgs you own."
2. A pull with nothing stated,
   and `--ff-only`:
   left to native Git's whole pull (picked),
   because its refusals are deliberate and the isolated merge already forwards fast-forwards.
3. The working-tree update after a fetch that moved the checked-out branch:
   wait on the index lock before it (picked),
   over leaving native Git's race,
   which a loaded host hits and the never-fail rules cover.

### Release contents

- 0.5.27 ships without the in-process index work,
  which waits for its review's fixes and ships in 0.5.28;
  reasons under "Agents and gates running".
- The fallback to the installed real Git was queued for after 0.5.28;
  it finished first and is merged into the 0.5.28 base (`afc9a8621d`),
  so it ships in 0.5.28 (coordinator's pick:
  the owner's "Ship interim now" favours shipping a finished fix).

### cargo-shadow (agent's nine questions)

1. Committed lockfile under `trustLockfile = true`:
   local runs compare with the upstream branch (`@{upstream}`,
   else `origin/<default>`) instead of `HEAD`,
   so cases 1 and 2 fire for versions not yet on the upstream;
   CI keeps its base.
2. Release age and trust both apply to versions new since the base,
   on both the local and the frozen path.
3. CI stops on a policy refusal (pnpm's frozen install fails on a lockfile entry the policy rejects).
4. The window in which a cargo that is not the shadow builds on an unapproved lockfile is accepted and documented
   ("Use as few locks as possible").
5. Restore uses `renameat2` with `RENAME_EXCHANGE` and an undo,
   through a safe wrapper (no lock,
   no lost writer).
6. Strict cutoff with exemptions:
   keep resolving without the cutoff and enforcing it on non-exempt crates.
7. A `--breaking` sync failure after the catalog write restores the catalog (transactional).
8. A killed `--breaking` run's manifests go in the journal.
9. The trust-downgrade line names the earlier and the new publisher account when known,
   as the owner's chosen option said ("naming the publisher change").

## Owner decisions in force

Timestamps are the commit times at which the decision was first written into the former handover
(`git log -S` over that file);
the owner's own time of speaking is recorded only where the entry gave a date.
A decision already recorded in a canonical doc is a pointer,
not a copy.

### Answers after the restart (2026-11-04)

Through the question tool,
answered before 08:22:29-05:00 (`b0d9708b17`, the next commit):
- Mutation runs:
  "Pause all (Recommended)":
  no mutation runs until the commit protocol is built and the lint cleanup (task 88) lands,
  then one fresh campaign on the new code,
  starting with `native:mutation:planted`.
- The decision record's scratch-ref sentence:
  "Add the limit now (Recommended)":
  written into `doc/decision/cli-git-concurrent-commits.md` (immediate-expiry prune,
  `gc --prune=now` and `repack -a -d` started while a commit lands are not supported);
  no code change.
- Reference-lock removal (the ninth design review's R9-S):
  "Keep narrowed removal (Recommended)":
  0.5.28's removal on a proof about the holder,
  with the second identity check just before the move,
  stays until the new commit protocol (whose X1 never removes a reference lock) replaces it;
  the rare two-holders race is accepted for that time.
- Saved conflict copies that can be the only copy:
  "Keep pruning by age and size",
  against the coordinator's recommendation (keep all and warn past a size);
  the owner was told a copy older than 30 days or past 1 GiB is deleted even if it is the only copy.
  "Prune by both age and size." therefore stands over "never discard user data" for saved copies;
  the numbers stay as cited and are set from measurement once saves exist (option O3).
- cargo-shadow's six picks (delegated scope with the acknowledged weaker path,
  verified-absent scope as proof,
  same-machine reboot clears liveness,
  assumptions a to e,
  forcing at exit only,
  the `user.delegate` route dropped):
  "Keep all (Recommended)".
- Commit design level 2 on the path where its eligibility cannot be shown:
  "Keep level 2 off (Recommended)".
- Index reclamation:
  the owner answered in the question's free text,
  "Must work perfectly even when flock isn't available. Do better."
  So the shipped behavior on a file system with no `flock` (never remove,
  wait to the lock limit,
  stop naming the file) is rejected:
  reclaiming a stale `index.lock` must work there as it does with `flock`.
  The other two picks (the `flock` on the stale lock's own inode with the device and inode check;
  `.aside-*` leftovers documented,
  not swept) were not addressed and stand.
  Work started on `fix/git-policy-cli-index-lock-reclaim-without-flock`.
  The owner then clarified:
  "I'm not banning flock. I'm just saying our wrapper must work perfectly even when flock isn't there."
  So the `flock` on the stale lock's own inode stays where it works,
  and the fallback must not depend on `flock` anywhere (a `flock` in `/run/lock` is at most an intermediate tier).
- Commit-speed picks (no `fast-import` pack flush unless `core.fsync` names loose objects;
  `sync_ms` renamed `index_update_ms`):
  "Keep both (Recommended)".
- CachyOS test containers:
  "Make as many pkgs as possible come from podman.",
  then,
  asked which was meant:
  "Sorry, typo. Yes I meant from pacman. And I believe pacman can give a specific Git version. If not possible, bring me the evidence and re-ask."
  So Git,
  Node and strace move to pacman inside the CachyOS image wherever pacman can supply what the tests need,
  Git at the pinned version through pacman if pacman can install it;
  if it cannot,
  bring the evidence and ask again.
  Work on `feat/git-policy-cli-test-images-from-pacman`.
  The owner then added:
  "I just realized something. We don't need to pin git because my current system is already using git latest."
  So the test image takes pacman's current `git`,
  with no version pin.
  Then:
  "We don't need to pin Node too. And what else?"
  Asked through the question tool about the tools `toolchain.lock.json` locks (Node,
  cargo-nextest,
  Rust,
  pnpm) against CM7's "in containers and CI use its row's archive and sha256",
  the owner chose "Pacman for everything it has"
  (the option named the CM7 exception);
  so these test images take every tool pacman carries from pacman,
  and pnpm from the lock row only if pacman lacks it.
  Open:
  propose to the owner the AGENTS.md wording of that exception (CM7),
  not yet asked.
  Asked about the end-to-end image's Git 2.55.0 (the oldest Git the wrapper accepts),
  the owner answered:
  "Raise minimum, drop the test, merely print a warning on older git and not refuse."
  So the 2.55.0 build and its scenarios go,
  the minimum rises to the tests' Git,
  and an older Git gets a warning and the command runs.
  Both sent to the agent on `feat/git-policy-cli-test-images-from-pacman`.
  CM7's wording:
  the owner chose "Leave CM7 unchanged" with the note "Record this carve-out for whatever the git wrapper needs.";
  so `AGENTS.md` stays as it is,
  and the package docs of `package/git-policy/cli` record the carve-out (its test images take tools from pacman).
  The other container picks (resolve `latest` once per run,
  run on the local copy with a loud warning when the registry is unreachable) were not addressed and stand.
- After the check of the older delegated picks (report of 2026-11-04,
  live picks listed under "Older picks still live"):
  - Branch writes:
    "Never cut a branch write (Recommended)":
    the shipped 60 s ceiling on `git update-ref` goes;
    steps that can run twice keep their ceilings;
    the command still returns at its own bound with the "Short extra wait" text.
  - Non-Git tools during `git worktree add` (package repair,
    Markdown linter,
    standards review):
    "Silence plus a ceiling (Recommended)":
    keep ending on silence and add a measured overall ceiling per tool that ends it with an error naming the tool.
  - The commit design's R-never (the lane never removes an `index.lock` its own dead process left):
    "Reclaim it (Recommended)":
    the new protocol reclaims a proven-stale lock of its own dead incarnation through the shipped mechanism once that passes review;
    revision 19 is to be amended (coordinator),
    and the models agent was told.
  The first two are built on `fix/git-policy-cli-branch-write-and-tool-ceilings`,
  with the never-run follow-up of shipped-limits pick 7 (re-derive the one-hour namespace-record block from the 10-minute lock ceiling).
  Built (tip `607e568e7d`,
  gate r3 4780/4780,
  clippy clean,
  not merged,
  no cross-vendor review yet):
  `update-ref` and the other in-place writers have no step ceiling;
  queued commits are told what the service is stuck in and withdrawn at their own ceiling;
  `FOREIGN_OWNER_PATIENCE` is 1200 s (twice `LOCK_CEILING`);
  repair tool 18 min (whole process group),
  Markdown linter 4 min,
  review call 1 min,
  whole review 2 min,
  each five times the longest measured run (warm stores only).
  Follow-up sent to the agent (`aa0fee0c2cf70bb31`),
  decided by the owner's own wording:
  the commit inside the stuck write returns at its own bound plus the short extra wait ("a few more seconds,
  a constant to be measured"),
  not at the 10-minute `WITHDRAW_CEILING`;
  a step keeps its ceiling exactly when it can run twice,
  decided per step by cutting it on a throwaway repository and comparing with an uncut run;
  every step without a ceiling still lets the person's command return at its own bound with a not-known error,
  the step left running;
  the service never restarts itself out of a stuck write;
  cold-store `pnpm install` and Cargo stage measured before the ceilings are fixed.
  Owner answers (question tool,
  2026-11-04):
  tool ceilings "5x (Recommended)" (five times the longest measured run,
  rounded up to whole minutes,
  recomputed with the cold-store case if longer);
  short extra wait "7 s (Recommended)" (twice the longest measured branch write,
  3,438 ms of 2,093);
  both sent to the agent.
  Follow-up done (tip `bafebbd512`,
  main `64031356d8` merged in;
  gate r5 4801/4801;
  clippy clean):
  `SHORT_EXTRA_WAIT` 7 s with a backward-readable `InsideWrite` notice (one extra field on the `waiting` line);
  the cut experiment (Git 2.56.0,
  kill at every mutating system call,
  rerun,
  compare) decided which steps keep a ceiling,
  and the first build's list was wider than the criterion;
  a person's command stops waiting at the step's class bound (120 s quick,
  10 min bulk) plus 7 s with a not-known error and leaves the step running;
  only the landing thread waits without a bound;
  cold `pnpm install` 20.9 s and `cargo fetch` 7.6 s were shorter than the warm maxima,
  so the ceilings stand.
  Cross-vendor reviews running:
  `branch-write-sol-1.txt` (step bounds,
  135 KB) and `branch-write-sol-2.txt` (tool ceilings and docs,
  127 KB).
  Owner answer (question tool,
  2026-11-04) on a person's command inside a step that cannot run twice:
  "Class ceiling + 7 s (Recommended)" (as built:
  2 min 7 s for quick steps,
  10 min 7 s for bulk steps,
  then a not-known error with the step left running);
  to be cited in `doc/planning/cli-git-no-unbounded-waits.md` with the review fixes.
- Index lock claims always plus machine id:
  built on `fix/git-policy-cli-index-lock-claims-always` (tip `69650ced17`,
  gate 4780/4780,
  clippy clean,
  red first);
  `Reclaimed::Foreign` leaves another machine's lock or claim and names it.
  Two cross-vendor attempts by the agent timed out empty (exit 124);
  the third (`claims-always-sol-3.txt`) returned "Merge after named fixes":
  legacy foreign markers must fail closed,
  a previous-release `flock` reclaimer and a claim reclaimer can still race on one lock,
  a permanently unjudgeable claim reads as a transient `Busy`,
  and a lost placement answer can strand this process's own claim;
  sent to the agent with the test fixes.
  Branch-write reviews (`branch-write-sol-1.txt`,
  `branch-write-sol-2.txt`):
  both "merge after named fixes";
  P1s:
  a commit can be answered withdrawn after it landed,
  failed markers count as a withdrawal,
  an early withdrawal is one-shot,
  a step "left running" dies of SIGPIPE when the command exits,
  and the classifier fails open on forms the experiment never ran;
  also the 7 s path misses a write that starts within 60 s of the answer ceiling,
  repair output collection is unbounded,
  a timed-out review keeps the review log,
  and the 20-minute foreign-owner patience can authorize reclaiming a still-live owner now that writes may run past any ceiling
  (the agent designs options;
  owner question to follow).
  All sent to the agent.
  Shipping under the owner's rule that reviews are async (2026-11-04):
  branch-write `bafebbd512` merged to main (`9cc1060ecb`),
  clippy clean,
  release 0.5.31 dispatched (run 41886727366),
  published to crates.io 2026-11-04T18:30:46Z (built from `e55cb35b07`)
  and installed:
  the 0.5.30 service retired on the client's request,
  0.5.31 started (protocol 7),
  and the sample commit on main landed in 31 ms (lock hold 28 ms);
  its review findings follow as fixes.
  Known in 0.5.31 and being fixed:
  a withdrawal can be answered after the commit landed,
  a step left running can die of SIGPIPE when the command exits,
  and the 20-minute foreign-owner patience is shorter than main's one hour
  (`branch-write-sol-2.txt` section 1 C;
  options then an owner question).
  Index lock `69650ced17` merged onto main in the integration worktree
  `.claude/worktrees/index-lock-claims-on-main` (branch `fix/git-policy-cli-index-lock-claims-on-main`);
  combined gate `gpc-gate-index-lock-claims-on-main-20261104T132517-0500` running;
  on pass,
  merge to main and release 0.5.32.
  Combined gate passed (4812/4812) and merged;
  the agent's review fixes (`7280815a34`,
  gate 4824/4824 on main) merged too.
  Owner answer (question tool,
  2026-11-04) on the open pick:
  "Own format only (Recommended)"
  (each release reclaims only locks of its own marker format;
  an older-format stale lock is left and named with its `rm`).
  Release 0.5.32 dispatched (run 41890150737).
  cargo-shadow:
  owner answered "Yes,
  add the wiring (Recommended)" for the three release-wiring edits the permission check had refused the agent
  (`releasable-crates.json`,
  the crate list in `crate-release.yml`,
  `release` and `release:wait` tasks),
  and "Done." for the first-publish token (`cargo login` in the owner's own terminal,
  per `doc/runbook/publish-crate-first-time.md`);
  the branch (`39b67c8748`,
  main merged,
  610 passed) merges after 0.5.32 is built;
  its one `doc/decision/rust-dependency-catalog.md` hunk (`c3602e69f1`,
  the update now living in the cargo shadow):
  owner "Approve the edit (Recommended)" (question tool,
  2026-11-04).
  cargo-shadow 0.1.0:
  the agent merges main into `feat/cargo-shadow-cli` (799 commits behind),
  wires it into `crate-release` as `7036dca934` did for mpm-shadow,
  then the coordinator merges and dispatches.
  Then the cross-vendor review of the whole branch,
  merge,
  and release 0.5.31.
  Done 2026-11-04:
  0.5.32 installed;
  cargo-shadow merged to main (`624e4ea5d2`,
  without waiting for 0.5.32,
  owner:
  "you don't have to wait for that.");
  release wiring committed (`651fc7b9a5`;
  the crate-release cli test now reads the expected crate list from the registry);
  0.1.0 published and Trusted Publishing config 30988 created
  (state in "Release in flight").
  Branch-write round 2 (agent `aa0fee0c2cf70bb31`, finished):
  all twelve follow-up items done except item 10,
  which is analysis only
  (foreign-owner patience options B > C > D > A in `doc/planning/cli-git-no-unbounded-waits.md`,
  section "Foreign-owner patience: what its expiry authorizes";
  owner question after the round-2 review checks the analysis);
  gate r6 passed 4827/4827 on `d2f74ce55a`.
  Main (with the index-lock change) merged into the branch as `bbf3317aff`;
  gate `gpc-gate-branch-write-and-tool-ceilings-r7-20261104T140212-0500` running;
  on pass,
  merge to main and release 0.5.33.
  Cross-vendor round-2 reviews running:
  `branch-write-r2-sol-1.txt` (landing record,
  protocol,
  client,
  step classifier,
  output capture)
  and `branch-write-r2-sol-2.txt` (tool ceilings,
  review log,
  the foreign-owner analysis).
  Known gap the agent left:
  `RERUNNABLE` names `symbolic-ref` and `init` in every form,
  and the cut experiment never ran them.
  Round-2 reviews returned 2026-11-04,
  both "correct after named fixes".
  sol-1:
  a provisional `withdrawn` decision can be acknowledged by a second withdrawer,
  so the commit lands while the command says it was withdrawn;
  digit-only side refs do not rule out a reflog or a symbolic ref;
  `fast-import` is allowed by name;
  reading captured output has no bound;
  `.tmp` decision files leak on a failed write.
  sol-2:
  a review that panics after its timeout leaves service reviews refused until restart;
  admission is not atomic;
  the timeout path still waits on `child.wait()`;
  `has_shadow_log` runs outside containment.
  Round 2 still merges after gate r7 (it narrows the 0.5.31 and 0.5.32 races;
  owner rule "Treat reviews as async.");
  every finding went to the agent as round 3.
  Owner answers (question tool,
  2026-11-04,
  verbatim choices):
  - Foreign-owner patience:
    "Refuse and name it (Recommended)"
    (option B:
    an expired record from another PID namespace stops the command,
    changes nothing,
    and names the record,
    its owner's PID and namespace,
    its age and the exact `rm`;
    the `rm` condition also covers Git processes the wrapper left running,
    the reviewer's addition;
    applies to merge intent recovery,
    claim takeover and worktree-copy journals).
    Sent to agent `aa0fee0c2cf70bb31` with round 3.
  - Commit notices (commit protocol plan question 4):
    "Line plus what was kept"
    (option B,
    not the ranked-first one-line option:
    every outcome prints its first line,
    then the indented `kept:` lines and their siblings from the drafts;
    command names stay placeholders).
    Sent to the models agent `a9c775c9196f1d836` as decision 24.
  Models agent:
  step 1 (the real state document as `LaneDocument`,
  `commit_lane_state.rs`) built;
  `native:test:model` 499 passed,
  both clippy tasks clean,
  `source_guard` 39 of 39;
  decisions 21 to 23 recorded (`e2df97729c`);
  next:
  the reclamation-during-capture and disposal models,
  then a review bundle for the coordinator.
  - Every other live pick in `doc/planning/cli-git-delegated-picks-status.md`,
    asked in seven grouped questions,
    each answered "Keep all (Recommended)":
    waits and bounds (150 s connected-push quiet window,
    10-minute foreign `index.lock` ceiling,
    a quiet Git step redone once,
    the 20-minute,
    60 s,
    10-minute and 150 s numbers,
    to be replaced by measured values);
    the commit design's stuck-Git and hook picks (A-kill,
    B-hand,
    C-kind,
    H-version,
    H-episode);
    its edge-case picks (rev 6 Q1,
    Q3,
    Q4,
    G-order,
    N-silent);
    cargo-shadow's first-round picks with its own-call limits and the 1,000-file warning;
    reference-lock removal Q1 to Q3;
    the merge and pull picks;
    and the smaller shipped limits (picks 1,
    2 remnants,
    3,
    5 remnants,
    8,
    11,
    12,
    13,
    14,
    15).
    No delegated pick is left open.
- cargo-shadow round 10 (tip `5be23af4b7`;
  debug and release 571 passed,
  5 skipped helpers;
  map 14 rows shown,
  2 partly) flagged two choices:
  - A killed run whose start was not fully recorded:
    "Record the launcher (Recommended)":
    the launcher's process id,
    start time and namespace are recorded,
    recovery proves it gone before reading the scope,
    and the acknowledgement stays only for what that cannot prove;
    built as round 11 (tip `2c20baf0e6`;
    578 passed,
    6 skipped helpers,
    debug and release;
    clippy clean;
    map 14 shown,
    2 partly).
    Cross-vendor reviews returned:
    `cargo-shadow-sol-11a.txt` "Does not meet the list for this part" (the launcher check's `kill` and `/proc/<pid>/stat` can address different processes when the mounted procfs serves another PID namespace;
    the boot-id authority check follows symlinks and races with the read;
    a `try_wait` error counted as a reap;
    the `view` header not validated)
    and `cargo-shadow-sol-11b.txt` "Meets it after named fixes" (D3 tolerance,
    K2 barrier order,
    real-launcher identity not compared,
    missing gone-launcher-with-member case,
    reboot control,
    byte-prefix cuts,
    discovery handshake);
    no row added.
    Round 12 done (tip `a8c7fbf8c2`;
    584 passed,
    8 skipped helpers,
    debug and release;
    clippy clean;
    map 14 shown,
    2 partly;
    the writer-side sync fault and a real interrupted publication now tested).
    Reviews returned:
    `cargo-shadow-sol-12a.txt` "Does not meet the list for this part" (the pidfd fdinfo check is not bound to the later stat read: a `/proc` replaced or overmounted between them makes a live launcher read as gone;
    a changed positive number after a failed stat read as gone;
    boot-id,
    reap and view-header fixes accepted)
    and `cargo-shadow-sol-12b.txt` "Meets it after named fixes" (scope comparison circular,
    procfs fixture not discriminating,
    publication control without completion);
    no row added.
    Round 13 running (suggested fix:
    one pinned procfs directory descriptor and `openat2` with `RESOLVE_NO_XDEV`).
    Owner,
    asked whether deliberate tampering with `/proc` and mounts by a program running as the user is in scope (rounds 10 to 12 each found a deeper such schedule):
    "Out of scope, keep cheap guards (Recommended)";
    assumption a is extended to exclude it,
    cheap guards stay,
    and the reviews judge against that premise.
    Round 13 done (tip `f463a87151`;
    588 passed,
    10 skipped helpers,
    debug and release;
    clippy clean;
    map 16 shown,
    0 partly:
    real kills after each step of `begin` for V3/V4 and a real shim run with a user-command sentinel for A2/R4;
    pinned procfs reads with `openat2`;
    only fdinfo `-1` reads as gone).
    Review `cargo-shadow-sol-13.txt`:
    "Does not meet the list":
    12a findings closed or excluded,
    12b repairs closed;
    a new counterexample needing no tampering:
    `/proc/<pid>/stat` field 22 is offset by the reader's time namespace,
    so a recovery in another time namespace reads a live launcher as gone;
    the shim test does not prove the delegated route and the intended fault;
    no row added.
    Round 14 done (tip `2530e617f4`,
    2026-11-04T10:02:46-05:00;
    590 run,
    590 passed,
    11 skipped;
    clippy clean):
    the launcher line carries the writer's time namespace inode and recovery answers Unknown on a mismatch,
    an unreadable side or an older line without one
    (measured: `unshare --time --boottime 7200` moves the same process's field 22 by 720000 ticks);
    the shim test now asserts the delegated scope,
    the consumed `ContainmentStopStuck` fault and the not-quiescent refusal.
    Change review `cargo-shadow-sol-14.txt`:
    "Does not meet the list for the changes; not ready for the certification pass":
    the distinct-time-namespace schedule and the shim evidence are closed;
    namespace inode numbers are recycled,
    and a writer's current time namespace can differ from its children's,
    so a recorded number can name a later namespace and a live launcher reads as gone;
    the shifted-start prerequisite accepts a read error,
    and the recovery assertion does not name its refusal.
    Round 15 sent to the agent (`ae0fbdf78c31bd757`):
    a namespace-independent launcher identity (suggested:
    the pidfs inode of the launcher's pidfd,
    to be verified from the kernel source),
    else Unknown on any start mismatch whose basis is unproven;
    both test assertions tightened.
    Round 15 done (tip `946615ef52`,
    2026-11-04T10:23:39-05:00;
    592 run,
    592 passed,
    12 skipped;
    clippy clean):
    the review's recycled-number schedule was built unprivileged and went red first;
    fix:
    Gone only when no process has the id in the recorded pid namespace,
    any other start time is Unknown (a reused pid now needs `cargo-shadow acknowledge`),
    and the time-namespace number left the record;
    pidfs inode rejected after reading `fs/pidfs.c` (an `ida`,
    reused,
    from 6.9 to 6.13);
    both assertions tightened.
    Change review `cargo-shadow-sol-15.txt`:
    "Meets the list for the changes; ready for the separate certification pass";
    no defect,
    two wording qualifications (namespace recycling after the last reference;
    "blocks" applies to a launcher classified Alive).
    Sent to the agent:
    those wording fixes and the certification bundles (row groups under 250 KB each,
    whole named test files plus the support they call,
    manifest `cargo-shadow-cert-manifest.md`).
    Wording fixed (tip `d5a8753488`);
    bundles ready (three groups of about 232 KB:
    group 1 rows A1,
    V1,
    V2 amended c,
    V6;
    group 2 A1/V3,
    A1/A2,
    A2/R4,
    V3/V4;
    group 3 A3/A4,
    R1,
    R2,
    R3/R5,
    K1,
    K2,
    V5,
    weaker path).
    Certification reviews of groups 1 and 2 running (`cargo-shadow-cert-sol-1.txt`,
    `cargo-shadow-cert-sol-2.txt`);
    group 3 queued for the next free review slot (prompt `cargo-shadow-cert-prompt-3.txt` written);
    group 2 `cargo-shadow-cert-sol-2.txt`:
    "not certified" (scope lifecycle shown;
    spawn isolation,
    post-start failures and recording partly shown:
    the overlap misses the real spawn boundary,
    no post-spawn startup-completion failure,
    fault arming not told apart from consumption,
    no write-sync-launch ordering,
    no proof of the containment kind attempted);
    no production defect;
    sent to the agent as evidence fixes;
    group 1 `cargo-shadow-cert-sol-1.txt`:
    "not certified" (V1 shown;
    A1,
    V2 amended c and V6 partly:
    direct-child reaping not observed,
    zombie fixture not run through the timeout,
    malformed and survivor tests accept another blocking cause,
    view tests only classify,
    no stalled scope-startup or shutdown deadline,
    lock tests do not time the answer,
    the pipe-holder `alive` oracle fails open,
    one timeout test checks only text);
    sent to the agent as well;
    group 3 `cargo-shadow-cert-sol-3.txt`:
    "Not certified" (all eight rows partly shown:
    completion failures with an operational error,
    kept files not read back,
    one inode not inventoried,
    rollback refusal cause unasserted,
    combined kept size,
    the barrier after the kept link,
    active recovery's own finish,
    weaker-path capability classes,
    lock and two-journal schedules;
    and the shared `journal_dirs` helper reads an unreadable directory as empty);
    no production defect;
    all three groups' fixes sent;
    done (tip `d5ab71ea62`,
    609 run,
    609 passed,
    13 skipped;
    clippy clean;
    16 of 31 items with a planted-regression red run;
    gaps:
    no real uninterruptible-member shutdown,
    no plant for five tests);
    bundles rebuilt in four groups (`cargo-shadow-cert-1-a1-v2amendedc-weakerpath.txt` 239 KB,
    `-2-a1v3-r1-r2-v3v4-v5` 235 KB,
    `-3-a1a2-v6` 230 KB,
    `-4-a2r4-a3a4-r3r5-k1-k2-v1` 235 KB);
    prompts `cargo-shadow-cert2-prompt-1..4.txt` written (group 1 without the manifest to stay under 250 KB);
    second certification pass running (outputs `cargo-shadow-cert2-sol-1..4.txt`);
    group 4 returned "not certified":
    A2/R4 and V1 shown,
    A3/A4,
    R3/R5,
    K1 and K2 partly (five evidence fixes,
    no production defect);
    group 1:
    A1,
    V2 amended c and weaker path all partly (process and thread oracles that can still pass falsely,
    fixtures and unsettled-scope bodies missing from the bundle,
    lost-record evidence stopping at classification,
    a notice formatted but not observed,
    writer-side acknowledgement durability untested);
    group 2:
    R1 shown;
    A1/V3,
    R2 (circular EROFS inventory,
    conflict and undo preservation failures unexercised),
    V3/V4 and V5 partly;
    group 3:
    A1/A2 shown;
    V6 partly (pipe draining has no bounded-return assertion;
    helper bodies missing from the bundle);
    4 of 16 rows shown across the pass,
    no production defect;
    all four groups' fixes sent to the agent,
    with a cutter that fails when a named test's helpers are not in its group,
    and as many smaller groups as needed;
    after they pass,
    the owner's approval for the first crates.io publish.
    Queued after a change review with no defect:
    a certification pass that attaches the test bodies the map names,
    split by row group under 250 KB per review,
    because the sources total 2.36 MB and every change review so far called the unchanged rows unavailable.
- Test images from pacman merged to main (`85984b790e`;
  branch tip `4bafaa418d`;
  gate 4754/4754 with passing names identical to a main control except the tests the owner's older-Git decision changed;
  e2e identical to main,
  same seed,
  the known `sigkill-service-during-work` failure unchanged;
  `native:clippy` on main clean):
  Git 2.56.0,
  Node 26.12.1,
  pnpm 12.17.0,
  strace 7.3,
  cargo-nextest 0.9.150 and Rust 1.99.1 all from pacman with signatures checked;
  the wrapper warns on a Git older than `MINIMUM_GIT_VERSION` 2.56.0 and continues.
  Follow-ups:
  `bin/native-nextest-snapshot.ts` stays because the fuzz sidecars import it;
  the minimum is raised by hand when `test:git-image` reports a newer Git (no automated check yet).
  Ships in the next release with the branch-write and tool-ceiling fixes.
- 0.5.30 published (crates.io 2026-11-04T14:27:24Z,
  run 41846701765) and installed (`5c2fb3a5b2`):
  it ships the claim chain;
  the sample commit took 1690 ms,
  1197 ms of it provisioning the upgraded service;
  the landing took 29 ms.
- Models round 15 review part 10b:
  "no, as submitted":
  the fresh-attempt model binds delivery to the current attempt counter,
  so an older stored notice printed after a new attempt starts authorizes reclaiming that attempt;
  bind notice,
  printing and removal to the producing attempt;
  the reclaim row should point to the claim-chain evidence;
  no criterion added.
  Part 10a:
  "does not meet the claimed items as submitted" (2A closure still caller-declared;
  2B adoption of an existing read-only inode;
  2E ancestry is not a covering relation;
  3A the launcher's lock copy blocks the handover;
  3B helpers unbounded after release;
  3C descriptors 0 to 2;
  3D ready byte with an unreaped first child);
  no criterion added.
  Round 16 done (tip `93dbc86b80`;
  451 model tests passed;
  map unchanged at 64,
  17,
  9,
  17);
  every 10a and 10b finding fixed except 3B,
  recorded as a coordinator pick (veto open):
  helpers stopped from outside after their lock release are not bounded,
  because no release-then-recheck protocol found bounds them without missing a receipt or adding a wait;
  the launch row is Narrower.
  Disclosed:
  a round 15 test had been cut by a later round 15 edit and is restored;
  the map check only tests names in its test column.
  Review `commit-models-sol-11.txt`:
  "no, as submitted":
  2B,
  3C and 10b closed;
  2A closure still lets a dependency outside the scanned places (an order file without its needs line) reclaim a predecessor;
  2E head equality credits every record in the chain,
  so one checkpoint can drop an acknowledged promise whose hash lands in the lineage;
  3A a handover timeout still runs the helper,
  and the intermediate child still holds a lock copy when GO is sent;
  the forced cleanup ignores `kill` and `waitpid` results;
  descriptor normalization leaks on an `EMFILE` path;
  3B as a Narrower row does not meet the target's resource bound,
  and the reviewer names a bounded protocol with no wait and no second lock:
  keep the admission token until process exit and hand a contender's receipt to the persisted syncer retry path.
  Coordinator pick,
  decided by the fixed acceptance target:
  build that protocol,
  so the 3B narrowing is withdrawn.
  Round 17 done (tip `8425cf376f`;
  462 model tests passed;
  both clippy variants clean;
  map unchanged at 64,
  17,
  9,
  17):
  the handover wait and the intermediate child fixed (reap,
  close the launcher copy,
  then the byte,
  then an acknowledgment);
  checked kill and reap;
  every descriptor owned on every path (EMFILE tests);
  only a checked `LaneDocument` checkpoint reclaims,
  and lineage names only reclaimed records.
  Coordinator direction,
  from the review's two options and the bounded log:
  extend `LaneDocument` with whatever the real lane state needs once that state is built;
  only states not yet represented stay non-reclaiming.
  3B not built:
  the model checker finds two schedules where the reviewer's token-to-exit protocol misses a receipt
  (a planned shutdown with no owner left to retry,
  and a client that cannot start one during the shutdown);
  the launch-bound row stays Narrower.
  Cross-vendor review running:
  `commit-models-sol-12.txt` (prompt `commit-models-prompt-12.txt`),
  asked whether those misses are real under the target and whether any bounded protocol avoids them.
  Review `commit-models-sol-12.txt`:
  "no, as submitted";
  2A,
  2E,
  the successful-launch windows,
  the unchecked calls and the descriptor leaks closed;
  the two 3B misses are genuine but disprove only reliance on the service's retry,
  and the reviewer names a singleton service-independent retry owner per lane holding the token for its lifetime;
  new defects:
  launcher death before handover leaves joined receipts unowned,
  a failed grandchild can still hold the lock after the failure was recorded,
  and `LaneDocument` dependencies are not validated on append admission or recovery.
  Round 18 sent to the models agent:
  fix those three,
  and model the bound's candidates (the reviewer's singleton owner,
  a sealed-lane inline force by the client,
  any other,
  the built one as comparison) with misses,
  helper count,
  waits,
  locks,
  recursion and foreground cost each,
  building none;
  the owner question on the bound goes out with that table.
  Round 18 done (tip `e07d9e2fbe`;
  471 model tests passed;
  clippy clean;
  map unchanged):
  A,
  B and C fixed red first (only the launcher carries the lock during a launch,
  the helper takes it after the handover;
  the launcher's drop is the last release before receipts are recorded;
  checked documents validated at admission,
  checkpoint start and recovery).
  Bound comparison (`model_helper_bound_r20.rs`):
  built release-recheck-exit no miss,
  helpers grow with clients;
  singleton owner no miss,
  two processes,
  but a lasting owner needs the unbuilt unit route (logout ends the fallback scope);
  inline only while sealed misses;
  inline whenever the token is held,
  token to exit:
  no miss,
  one helper,
  the client syncs its own receipt only when a flusher is at work (median 11.4 ms,
  max 77.1 ms,
  20 samples at load 14);
  baton no miss but needs the unit route;
  the agent ranks inline-whenever-held first.
  Review `commit-models-sol-13.txt`:
  "no, as submitted";
  C closed;
  A and B reopen through deeper timings (a handover timeout then launcher death;
  the intermediate child's inherited lock copy before its first close with failed cleanup);
  protocol 4 misses a receipt when the client dies before its own force,
  its one-helper bound breaks while launched helpers are delayed before taking the lock,
  and no time bound is shown for its foreground sync;
  the comparison omits client death,
  worker death,
  launch interleaving and mixed versions,
  and its crash exemption is too broad;
  new plain bug:
  a helper that fails to take the lock is acknowledged as a launched executor.
  Third "no" in a row where each closed finding is replaced by a deeper interleaving of the same family
  (launch,
  handover,
  death,
  cleanup);
  the coordinator stopped sending fix-everything rounds (second opinion concurred).
  The helper-count bound is the reviewer's acceptance target ("launch is non-recursive and resource-bounded"),
  not an owner rule;
  the supported scope was fixed by the coordinator,
  veto open.
  Sent to the models agent now:
  the plain bug and the map and comparison corrections only.
  Owner answer (question tool,
  2026-11-04):
  "Keep built,
  state limit (Recommended)"
  (keep release,
  recheck and exit;
  the helper count is bounded only by the commits that arrive during those milliseconds,
  overriding the reviewer's limit;
  the plain launch bugs still get fixed).
  Plain bug fixed (tip `2ba400d898`,
  472 model tests passed:
  a helper that cannot take the lock is a failed launch with every receipt recorded,
  and it records them itself too;
  `record_errors` writes each line in one write after interleaved lines were seen);
  map and comparison claims narrowed,
  the round 18 ranking withdrawn.
  Round 19 done (tip `e514bb736d`,
  473 model tests passed):
  the launcher holds no lock for the launch,
  it probes the lock and forks only when it was free,
  and the helper takes the lock itself;
  the handover byte and its timeouts are gone.
  Review `commit-models-sol-14.txt`:
  "no, as submitted":
  the probe itself is a false owner
  (a client holding its probe makes another client return `LockHeld`,
  then dies before launching,
  and nobody forces the second receipt);
  claims about who holds the lock and for how long overstated.
  Round 20 sent:
  only a process that lists and forces after acquiring may hold the flusher lock
  (drop the probe or test without acquiring),
  the reviewer's four regressions red first,
  claims corrected;
  parts to `commit-models-15-part-<n>-<topic>.diff`.
  Round 20 done (tip `924f8edd76`,
  476 model tests passed):
  the probe is dropped,
  every client forks a helper (launch to acknowledgment median 0.66 ms,
  20 samples),
  and only processes that list after acquiring hold the lock.
  Review `commit-models-sol-15.txt`:
  "Yes after named fixes,
  for the models/components' claimed coverage—not approval of the live protocol";
  fixes:
  the late-receipt argument (release and recheck,
  not acquisition order),
  the three probe controls do not establish their interleavings,
  reap and timing wording,
  and a cutover obligation for old probe-capable clients.
  Round 21 done (tip `be89e6a319`,
  479 model tests passed,
  map 64,
  17,
  9,
  18 not yet,
  108 rows):
  the release-and-recheck argument with a new control,
  three production acquisition sites listed,
  negative controls rebuilt on a test-only legacy probe seam,
  reap and timing wording corrected,
  and one Not-yet cutover row for old probe-capable clients.
  Review `commit-models-sol-16.txt`:
  "Yes—for the models/components' claimed,
  explicitly limited coverage.
  Not approval of the live protocol."
  The model rounds have converged
  (limits kept:
  `assert_never_recorded` observes 1.5 s;
  `the_fixed_launch_leaves_no_receipt_to_a_recheck_that_loses_the_lock` does not force a failed recheck acquisition).
  Next phase sent to the models agent:
  the declared disposable Git experiments for the supported scope,
  and an ordered plan for the remaining live-approval gates (cutover fence,
  lifecycle and forcing at real call sites,
  recovery and preservation,
  installation and latency) in the design file,
  owner decisions marked as questions.
  Done:
  Git experiments (tip `40c8ff7baa`,
  490 model tests;
  files and reftable,
  packed refs,
  symbolic refs,
  side ref,
  linked worktrees,
  kill at each boundary;
  under reftable one `tables.list.lock` blocks every ref) and the seven-step plan (`a1308f6d8c`,
  section "Ordered plan to the live protocol (round 22)").
  Owner answers (question tool,
  2026-11-04):
  state document "Checked format (Recommended)";
  launch route "Systemd unit + fallback (Recommended)" (lifts the "manager-started route,
  until built" exclusion);
  cutover fence "New lock name (Recommended)".
  The notice-text question waits for drafted example texts;
  step 1 (the real state document) is the next build.
  - Trusting the boot id:
    "Check and state premise (Recommended)":
    procfs magic and no mount on top of `boot_id`,
    plus the stated premise that the serving kernel ran the old run.
- Index reclamation without `flock`:
  merged to main `38de589e7a` (branch tip `ceafded580`,
  gate 4769/4769,
  its own cross-vendor review `claim-chain-sol-1.txt` folded in);
  the claim chain of placed files where `flock` is refused.
  Owner,
  asked about the mixed-mount race it left (a `flock` reclaimer and a claim reclaimer on one file):
  "Claim files always (Recommended)";
  so every reclaimer takes the claim chain and the `flock` path for this is retired.
  Follow-up running on `fix/git-policy-cli-index-lock-claims-always`:
  that change,
  plus the machine id in markers and claims (a different machine is unknown,
  never gone;
  same machine with a different boot is gone),
  the coordinator's application of the rule the owner kept for cargo-shadow.
- Models round 15 done (tip `d650198dcf`;
  440 model tests passed;
  map 64 covered,
  17 narrower,
  9 historical,
  17 not yet);
  coordinator picks under the owner's rules sent back before its review (veto open):
  saved content made immutable by its creator (O_TMPFILE or a private name,
  read-only before it is named) instead of depending on read leases;
  the running syncer retries recorded receipt errors;
  linked worktrees,
  packed refs,
  the side-ref transaction and create,
  delete and symbolic-ref updates are obligations to demonstrate,
  not exclusions (exclusions:
  level 2,
  a threaded launcher,
  the memory or process cap,
  the transient-unit route until built).
  Applied at tip `249589f623` (442 model tests passed;
  map unchanged at 64,
  17,
  9,
  17);
  the scope paragraph and the two picks are in the design (`f8f129af86` on `fix/git-policy-cli-commits-never-fail`,
  section "Acceptance target").
  Cross-vendor reviews running:
  `commit-models-sol-10a.txt` (components,
  launch and flusher) and `commit-models-sol-10b.txt` (models and map),
  prompts `commit-models-prompt-10a.txt` and `-10b.txt`.
- #6261 native-layer move:
  "Defer until branches merge (Recommended)".
- Fallback to the installed real Git (four picks):
  "Keep all (Recommended)".

### Nothing waits forever

The owner,
2026-11-03 at about 12:37-05:00 (said in the session,
not yet in SPEC):
"nothing should ever wait forever or indefinitely".
The owner added that they are not completely back,
so the delegation continues.
What happens at a bound,
answered by the owner through the question tool at about 2026-11-03T12:45-05:00:
"Return with an error"
(the option read:
"The command stops waiting and exits non-zero,
naming what blocked it;
the saved work stays saved."),
then,
asked whether the saved commit should still land by itself later,
"No,
keep it for a retry (Recommended)"
(the option read:
"Nothing lands later by itself.
Your staged changes stay staged,
and the commit is also kept on a side ref.
The error names the blocker and the command to run again.").
This replaces the coordinator's first reading (return as success and land later),
and the owner chose it over "Commits should never fail." for a command that hits a bound.
Pushes,
same question tool:
"Bound it,
keep retrying (Recommended)"
(a connected push with no progress for a measured time is ended,
the service's retry plan keeps it,
the command says it is queued),
replacing "Wait while connected".
The rule also overrides these coordinator picks:
pick 2 (a proven-alive index-lock holder waited for with no bound),
its follow-up (an unproven owner waited for with no bound,
built in `82633304d0` and merged into the 0.5.28 base),
pick 6 (no client wall-clock bound on the service),
`update-ref` waited for,
and the design's B-hand,
P-wait and R-never.
0.5.28 does not ship until every wait in it has a bound.

### Commit protocol revision 12 answers

The owner,
through the question tool,
answered before the commit that records this entry
(design `doc/planning/cli-git-commits-never-fail.md` revision 12,
section "Owner questions for revision 12"):
- A branch write already started when the time runs out:
  "Short extra wait (Recommended)"
  (the command waits a few more seconds,
  a constant to be measured,
  for the write it started;
  if the disk is still stalled it exits non-zero saying the result is not known,
  naming `git log -1` and the side ref).
- Merge order:
  "Branch first, files after (Recommended)"
  (the branch moves first and the files follow;
  at the bound the folder and staging are untouched;
  the owner was told a direct `/usr/bin/git commit -a` in the lag window would commit the merge's reverse).
  This replaces the coordinator's delegated pick M-A.
- The person's own programs before the commit exists (editor,
  hooks,
  signer,
  forwarded `gc`):
  "No limit; Ctrl-C ends it (Recommended)".
- How soon an accepted commit must survive a power cut:
  "Force at leaving, else background (Recommended)"
  (no forced write while the person waits;
  what a command promises when it leaves is forced to disk first;
  a power cut while it still waits loses that attempt,
  as with native Git).
  The coordinator applies the same rule to today's landing (the integration agent in "Agents and gates running").

### Commit protocol question 8 answer

Asked at 2026-11-04T05:07-05:00 through the question tool,
after the review of revision 19 (`commits-never-fail-sol-19a.txt`) found it "accurate after one small precision edit",
which was applied (`b98b3cad18`).
The owner picked "Retry it,
leave a notice (Recommended)":
when recovery after a power loss finds no decisive positive authority for a request that had returned success,
it makes a fresh attempt and the next wrapper command in that worktree prints an outcome-specific stored notice
(published with the new commit's id and the command to drop it,
or why it stopped).
The rare case where a commit removed on purpose comes back,
not promised to happen only once,
is accepted.
Recorded as decision 19 in the design.

### Commit protocol revision 15 answers

The owner,
through the question tool,
before the commit that records this entry,
answered the four questions of revision 15 (`4a6ad9d34a`) as the coordinator worded them after applying the fifteenth review's named edits
(`commits-never-fail-sol-15.txt` section 3;
the edited wording was not re-reviewed):
- Question 5:
  "Carry the file, save staged (Recommended)".
- Question 7a:
  "Clean up automatically (Recommended)"
  (by value tests within 7b's window under `index.lock`,
  current version saved first;
  "possible leftover" when any test fails).
- Question 7b:
  "A measured span (Recommended)"
  (starts when the merge's files are written;
  at its end no new cleanup starts;
  the value comes back to the owner before shipping).
- Question 7c:
  "Keep a reminder (Recommended)"
  (dismissing deletes no saved copy).
These supersede the provisional decisions 12 and 13.
Sent to the revision 16 agent.

### The final disk save is never cut short

The owner,
in the session,
after being told the fourteenth review found question 6a omitted fail-closed recovery:
"Assume the final disk save would never be cut short."
Meaning for the commit protocol design:
the departure barrier (stop receipt and saved set) runs to completion with no time limit;
questions 6a and 6b are dropped;
the provisional decision 12 ("Short wait, then say unconfirmed") is superseded;
a disk that never finishes the sync falls under the owner's outside-our-control list ("disk I/O errors").
Sent to the revision 15 agent.

### cargo-shadow kept files

The owner,
through the question tool:
"Warn past a count, never refuse (Recommended)"
(past a threshold every run prints how many kept files there are,
their total size and the folder to review;
cargo keeps working;
nothing is deleted).
Replaces the agent's refusal at `MAX_KEPT_FILES`.

### Commit protocol revision 13 answers

The thirteenth review (`commits-never-fail-sol-13.txt`) found questions 6 and 7 framed inaccurately:
question 6 omitted that a departure cut short by its limit can lose the record that forbids further attempts (so a withdrawn request could still land after a power cut),
and question 7's option A claimed it never undoes a later intentional change,
which a `reset --soft` plus switch disproves.
Answers 6 and 7 are therefore provisional until revision 14 asks them again;
answer 5 stands,
with its missing option to be shown.


The owner,
through the question tool,
answered before the commit that records this entry
(design revision 13,
`cc5e0822dc`,
section "Owner questions for revision 13"):
- Staged content differing from the file at a merge:
  "Carry the file, save staged (Recommended)"
  (the file as seen is carried;
  the staged-only version is saved to `saved/<path>.staged` and named in a line).
- A slow force at leaving:
  "Short wait, then say unconfirmed (Recommended)"
  (wait up to a named,
  measured limit,
  then exit saying the saved copy is not yet confirmed on disk while the wrapper keeps forcing it).
- How long to watch for a late `git switch` after a merge:
  "A measured span, then stop (Recommended)"
  (after the span the merge's files stay as changes,
  nothing lost,
  and the merge's message names how to undo them).

### Issues the owner raised are active work

The owner,
in the session,
before commit `25c4b39f58` (2026-11-03,
the commit that first recorded it):
"Why are you still not doing gh issue 6260 and 6261?"
#6260 was fixed and merged into the 0.5.28 base but the issue had no comment;
#6261 had been queued by the coordinator behind 0.5.28 without asking.
Both are now active:
#6260 has a status comment,
and #6261 runs on `refactor/git-policy-cli-subsystem-directories` (from `f89fa63f10`) in slices,
families no running branch touches first.

### Host pressure is never an excuse

The owner,
2026-11-03 between about 13:05 and 13:17-05:00 (said in the session),
after the coordinator offered disk pressure as the reason commits took 8.99 s,
16.6 s and 35.7 s:
"Disk pressure being high is not an excuse.",
then
"I've said something to the effect of "disk pressure / host / load is not an excuse" like 10 times now.",
then,
on the coordinator's "remove it or move it off the waiting path":
"sometimes the work itself can be optimized.
And we control every part of the wrapper,
including everything it embeds.",
then
"Think optimization can't do much?
No!
Look at our forbidden-strings and forbidden-regex.
Much faster than anything else.
At least 2x faster in every way than official `regex` crate.
And I did it with a much weaker model."
Then,
asked through the question tool how AGENTS.md should carry it,
"Tighten NHR, add SLW (Recommended)",
followed in the session by
"Host pressure might sometimes be the cause but it's never an excuse.
I also told you if you think the 5s standard cannot be always met even if we put in all the work,
you can ask me to raise it with evidence.",
and
"I told you that a long time ago.
And you never used it."
(that earlier permission was not written into this handover,
SPEC or any planning doc;
a search for it just before the commit that records this entry found nothing).
Meaning for this effort:
a command slower under load,
disk pressure,
swap or a slow SSD is a defect in the command,
even where the pressure is a cause.
The fix is found by measuring,
under that same pressure,
what the wrapper does beyond native Git,
and then removing it,
making the work itself cheaper (embedded code included),
batching it,
or moving it off the path the person waits on.
Standing permission:
when the evidence shows the 5 s standard ("commits vs native git should not take 5s longer.") cannot always be met even after all that work,
ask the owner to raise it,
with the measurements;
never accept a miss silently,
and never offer the pressure as the reason instead.
Offering the pressure as the excuse is retracted wherever this handover did it
(the 1996 ms commit was split by step against the load average,
and the slowdown was then attributed to load).
The precedent for how far optimization goes is `package/forbidden-strings/core` and its bench `package/forbidden-strings/core.bench`.
AGENTS.md now carries NHR ("sometimes a cause,
never an excuse or a reason to wait") and SLW.

### Never fail, and what is outside our control

All in `package/git-policy/cli/SPEC.md`,
"Merges and commits never fail":
the three rules ("Auto merging should never fail.",
"Manual merge should never fail too.",
"Commits should never fail."),
the correction "never fail doesn't primarily mean retries!",
the settled list of causes outside our control
(settled by the question tool,
recorded 2026-11-03T07:31:44-05:00),
the allowed refusals,
"Committing a conflicted path is fine.
Warn and commit anyway,
even if conflict markers will be committed.",
"Stack on the new head (Recommended)" and the later
answers.
Also:
"Offline is retried,
but auth and server hook declines stop until your next commit."
(SPEC,
same section).
Answered by the question tool on 2026-11-03 and not yet in main's SPEC
(they are on `fix/git-policy-cli-commits-never-fail` until it merges):
"Same as the ref lock (Recommended)" for a live foreign `index.lock`;
"Save on the side ref (Recommended)" for a branch a foreign program keeps moving;
"Leave out wrapper refs (Recommended)" for `git push --mirror` (everything except `refs/cli-git/*`,
with a line saying so);
"Amend the pending commit (Recommended)" for an amend while commits are pending.

### Design stance

The commit service unit sets no resource limits of its own
(owner, 2026-11-03,
"Remove all three (Recommended)" after the 1 GiB memory cap was found unmeasured;
recorded in `package/git-policy/cli/SPEC.md`).
Every shipped limit cites a measurement or an owner decision (`AGENTS.md` rule `LQM`).

`package/git-policy/cli/DESIGN.md`:
"we control all the code" and "Good is not perfect" (2026-11-01),
concurrent committers keep
growing,
locks are an anti-pattern with a resident single-writer service,
and "socket activation (OS starts it) or a loud,
clear
failure - no,
never leak internal things that could be resolved to the user."
(2026-11-01T07:37:44-05:00).
`AGENTS.md` rule NLK carries the lock rule;
the owner's earlier "Use as few locks as possible."
(2026-11-01T11:05:47-05:00) and "Invoke real git as rarely as possible."
(2026-11-01T01:28:19-05:00) are quoted in `SPEC.md` and `doc/planning/cli-git-btrfs-worktree.md`.
"We should never invoke ourselves recursively."
is in `doc/planning/cli-git-no-self-invocation.md`.

Recorded here only (no other home found):
- "We also have the advantage of git itself being OSS so we're not just wrapping a black box."
  (2026-11-03T07:36:28-05:00).
  Practice:
  for every Git step the wrapper runs or replaces,
  read Git 2.56.0's own source for what it locks,
  waits on and can exit
  nonzero for,
  cite file and line,
  and where a step moves in process port Git's logic so the result equals Git's.
  A v2.56.0 clone is at `~/temp/agent/git-v2.56.0-2026-11-03`.

### Git's own fsmonitor

Superseded 2026-11-03 by the owner:
"If git's fsmonitor is fundamentally broken and cannot be easily or meaningfully worked around,
disable git's fsmonitor globally entirely and make doctor say so."
The coordinator found that it is
(Git 2.56.0's Linux daemon answers only after registering a watch for every directory,
`compat/fsmonitor/fsm-listen-linux.c` lines 402-439 and 702;
a client waits up to 60 s for it,
`builtin/fsmonitor--daemon.c` line 40;
main's daemon watched 2,269,377 directories because the linked worktrees live inside main,
and cost 117.7 s of CPU and 734 MB to start;
a running daemon saved about 7 ms per `git status` on main),
and at 2026-11-03T09:38-05:00 set `core.fsmonitor = false` in `~/.gitconfig`
(backup `~/temp/agent/gitconfig-backup/gitconfig.before-fsmonitor-off-2026-11-03T0938`;
it was the only scope that set it)
and stopped all 261 running daemons (253 with `git fsmonitor--daemon stop`,
8 with SIGTERM,
none left;
`~/temp/agent/tools/fsmonitor-stop-all.ts`).
Done:
the doctor line (`feat/git-policy-cli-doctor-says-fsmonitor-is-off` `072ac84868`,
shipping in 0.5.27)
and `doc/troubleshooting/git-fsmonitor-daemon-start-wait.md` with a prototype patch beside it
(`7ea0414e80` to `da42fdb07b`;
the waiting `fsmonitor--daemon start` busy-spins,
concurrent clients each start a daemon,
a cold start measured 1.88 s at 100,000 directories and 0.013 s patched,
`t7527` passed 69 with 6 skipped;
the upstream draft is not for filing as is,
because Git's `SubmittingPatches` rejects AI-generated patches and needs a human sign-off;
filing is the owner's decision).
The earlier statements stay in `package/git-policy/cli/README.md`,
"Git's own fsmonitor",
as history.

### Worktree add and package managers

All in `doc/planning/cli-git-main-worktree-ignored-copy.md`:
"node_modules must be copied."
and "Builds must be copied" (2026-11-02),
"Skip nested worktrees and caches.
The point is to make
new worktrees immediately functional.",
"Skip run records (Recommended)",
"Convert once + snapshot (Recommended)",
"Set the shared store (Recommended)",
the dirty-main answer "B:
snapshot,
then reset",
"Reuse deps,
rebuild own" with
"Run cargo clean -p",
"Install online during the add",
"Run scripts as allowBuilds says (Recommended)",
the owner's pnpm
requirements ("Our wrapper must work with pnpm w/o a storeDir set...",
"Compatibility with all pnpm configurations is required.")
and the widening "I'm widening the pnpm spawning exemption during worktree copy ops to all pkg managers."
Races with other programs (2026-11-03T00:15:33-05:00),
verbatim:
"That is not an our problem.
By using this command or this program,
make the user understand and accept the risk."
Adopted reading (veto open):
races caused by other processes changing files or names while the wrapper works are outside what it
defends against;
it states that risk plainly and keeps fixing its own deterministic bugs.
Speed (2026-11-03T04:03:31-05:00):
"60s is fine for adding worktrees.
And make sure we're running everything that can be run in
parallel in parallel."
and "No,
I didn't say speed is unnecessary!"
Completion marker:
"Drop the fsync (Recommended)" (`SPEC.md`).
Pass `checkout.workers=0` "only if unset" (`SPEC.md`).
Per-worktree service (2026-11-01T16:28:21-05:00):
"Per worktree (Recommended)"
(`doc/planning/cli-git-resident-service.md`);
"Properly support overlayfs too."
(`doc/planning/cli-git-btrfs-worktree.md`);
"there is never a 5s bound on wrapped commands" (`README.md` and `SPEC.md`).

### Merge auto-resolution

`doc/planning/cli-git-merge-auto-resolution.md`,
"Decisions":
word-level second pass for merges a person runs and for the wrapper's unattended merges,
"Own merge on similar (Recommended)",
"doc/handover/*.md only (Recommended)" for the union driver (`.gitattributes`),
"Finish like a clean merge (Recommended)".
Conflict sides:
"save,
then allow" (`SPEC.md`).
Pruning of `cli-git/discarded/`:
"Prune by both age and size."
(`doc/handover/cli-git-logging.md`).

### Hooks and pushes

Only hooks the wrapper declares,
run natively;
no external hook commands ("Stop supporting them and port anything in.",
chosen label "B:
Only hooks we declare,
natively",
2026-11-01T02:13:03-05:00):
`SPEC.md`,
"Real Git starts with traditional
hooks off".
The wrapper never appends `--quiet` or `-q`:
"No the wrapper should never append --quiet.
Letting native git emit whatever it wants
to is expected."
(2026-11-03T04:50:57-05:00,
`SPEC.md`;
inner uses are the open `--quiet` branch).
Push stop rule "Stop only on no progress (Recommended)" (`SPEC.md`,
"push patience");
"Login failures final" (`SPEC.md`).
"Keep LFS on the fast path (Recommended)" (`doc/planning/cli-git-btrfs-worktree.md`).

### Tooling and repository policy

- Lockfiles are never hand-edited:
  "You're not meant to manually edit lockfiles!
  You're meant to fix the mpm set of tools we control and run it!
  And it is meant to follow pnpm!"
  (2026-11-03T05:15:12-05:00);
  `AGENTS.md` rule FOT and LFW.
- Cargo shadow:
  "Make a cargo shadow like what we did for pnpm shadow.
  Put the existing rust catalog tools inside it instead of
  making them separate.";
  `cargo update` equals `pnpm update` ("I said follow pnpm semantics.",
  2026-11-03T05:30:00-05:00);
  "I also said to remove the Mise tasks that would then be handled via the wrapper.";
  "PnPM doesn't have the reconcile keyword.
  Our Rust catalog tooling should follow pnpm and shouldn't have that keyword too."
  (2026-11-03T04:30:37-05:00).
  These live on the unmerged cargo-shadow branch;
  `doc/decision/rust-dependency-catalog.md` records the catalog rules.
- Workflows are editable like any file ("Who said you can't touch workflows?
  No one said that.
  Just do it.")
  and "Workflows
  should never use mise at all."
  (2026-11-03T05:31:50-05:00).
  CI runs tasks through `task-run`.
- mpm shadow:
  "I want pnpm style verbs.
  We talked about this!"
  (`package/mpm-shadow/cli/README.md`);
  the repair "should start whatever pnpm is first on path."
  (`doc/planning/cli-git-main-worktree-ignored-copy.md`);
  global tool environment "Repository only (Recommended)";
  the owner is "migrating away from mise soon TM"
  (the hint fix of 2026-11-03 removed a mise task name from a user-facing remedy).
  Mise is a task runner here,
  not a package or tool manager;
  the repository's package managers are pnpm,
  Cargo,
  Gradle,
  uv,
  Bun,
  npm,
  Zig and the `mpm-shadow` toolchain wrapper.
- Disk weight:
  `IOWeight` never above 10 (`AGENTS.md` RXS);
  containers get `--blkio-weight=10 --cgroup-conf=io.weight=10`.

## Working rules for this effort

Rules not in `AGENTS.md` (each recorded at the time shown in the former handover).

- Standing authorization (2026-10-13):
  "bump and publish any updates of the native wrapper crate and its `forbidden-strings`
  dependency during this session,
  after the gate,
  a dry-run publish and a real-credential commit have passed."
  It was given for that session;
  releases since have gone through `crate-release` under it.
  This session (2026-11-04) the owner said:
  "I'm authorizing you to bump and publish any updates this session."
  It covers every crate,
  a crate's first publication included;
  the owner,
  2026-11-04,
  on an earlier coordinator line that required approval for a first publication:
  "Didn't I always give you approval to publish?"
  and "By "first" I meant "first in this session of any crate"",
  and "didn't I also say reviews and mutation testings don't block publishing?".
  Earlier (session transcript): "Mutation runs don't block publish and release."
  So reviews and mutation campaigns never block a merge or a publish;
  their findings become follow-up fixes and later releases.
- Release order for the wrapper:
  gate passes (`gate-exit-status=0` and the nextest summary),
  merge to main,
  `native:clippy` clean on main,
  `mise run //package/git-policy/cli:native:release` (dispatches the `crate-release` workflow;
  it refuses while origin lacks local main commits),
  `native:release:wait`,
  then `node ~/temp/agent/upgrade-wrapper.ts <version>`
  and a sample commit.
  Owner (2026-11-02T21:58:07-05:00):
  "This is expected and fine,
  the correct answer is to release again combining all the
  changes."
  So merge gated work as it passes and dispatch again with everything combined.
  A release builds from origin's main:
  check `main...origin/main` and that fetch works before trusting a release's contents.
- Gates:
  `node ~/temp/agent/gate/launch-gate.ts <worktree name or path> <label of lowercase letters, digits and hyphens>`
  starts the container gate as its own systemd unit `gpc-gate-<label>-<timestamp>`;
  its log in `~/temp/agent/gate/` ends with
  `gate-exit-status=<n>`.
  Read the verdict from that line and the log's nextest summary,
  never from `systemctl show` on the unit
  (`doc/troubleshooting/systemd-run-collect-reports-success-for-a-failed-unit.md`).
  Host checks:
  `mise run //package/git-policy/cli:native:test:host -- <filter>`,
  `native:clippy`,
  `native:perf`
  (30 paired samples against direct Git,
  limit 5000 ms per delta;
  last measured commits median +40.2 ms).
- Reviews and merging:
  "If we have already decided on something just do it.
  Treat reviews as async."
  (2026-11-01T07:21:50-05:00);
  decided work merges after its gate,
  review findings become follow-up fixes.
  "Figure it out amongst yourselves who should own or do what.
  You have the tools."
  "You should (almost) never be waiting.
  There is always more work."
  and
  "Avoid wasting compute on stuff that would be shortly superseded."
- Cross-vendor reviews run through `pi` (see `CLAUDE.md`),
  each in its own scope (1 CPU,
  2 GiB),
  as many at once as there is work:
  the owner asked "make sure we're running everything that can be run in parallel in parallel".
  An earlier "keep at most three running" was the coordinator's own unmeasured guess at the cause of the timeouts
  (added 2026-11-02,
  `f0780fff47`),
  never an owner rule;
  removed 2026-11-04 after the owner asked who set it.
  Whether concurrency affects the timeouts is unmeasured;
  record each run's concurrency and outcome before limiting again.
  Several have timed out at 3600 s with no output and an empty session folder
  (`~/.pi/agent/sessions/--var-mnt-encrypted-temp-agent-review-*--/`):
  the run stalled inside `pi` before sending.
  A one-word probe answers in about 5 s;
  relaunch with the original question,
  narrower input and a word cap.
- Mutation campaigns are run by the main agent,
  never a subagent
  (owner,
  2026-11-01:
  "Because of how a few Claude Code quirks work together,
  long-running mutation runs when handed to subagents
  would burn abnormally amount of compute,
  so they'd have to be done by the main agent.";
  scope "Limit it to mutation runs only for now.").
  Record:
  `doc/troubleshooting/claude-code-subagent-background-wait-context-dump.md` and
  `package/mutation-test/cli-cargo-mutants/README.md`,
  "Who runs a campaign".
  Container gates,
  fuzz campaigns and the e2e suite may run in a subagent.
  A campaign starts as a background unit and never holds a release or the next item;
  survivors are killed or justified later.
  Parallel campaigns use `--passes 2` (owner-accepted).
- Briefs to wrapper agents quote `package/git-policy/cli/README.md`,
  "Git's own fsmonitor",
  and the component's recorded
  `doc/decision/` owner decisions (rule DQB),
  and the synthetic-load ban.
- Keep all lines going:
  the owner chose "Keep all lines" (2026-11-03T07:38:56-05:00):
  continue the parallel lines and release
  whenever a gate passes.
- Timestamps in notes come from `date` or `git log --format=%cI`,
  never guessed;
  waits are on PIDs,
  never on a process name.
- Queue items are written here the same turn they are deferred (rule QWD);
  finish an item by moving its facts out and deleting it.
