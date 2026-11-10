# Dangling-link work items W2 to W15 for the repository move to the 860 drive: state and next steps

Written 2026-11-07,
after the runner fixes of the first fixture runs and the lint of the two new packages;
updated by the second agent (branch `feat/repository-move-dangling-link-items-2`,
worktree `.claude/worktrees/dangling-link-items-2`)
after W2,
W4,
W5,
W9,
W10,
W11 and the fixture-image fix.
The plan is `doc/planning/repository-on-the-860-drive.md`
(sections "Work items per tool",
"Status and order of the work items",
"The symlink design",
"The send plan");
the runner is `package/repository-move/cli` (runbook `doc/runbook/repository-on-the-860-drive-guided-runner.md`);
the shared check is `package/repository-link/check` (W1).
Branch `feat/repository-move-dangling-link-items`,
worktree `.claude/worktrees/dangling-link-items`;
everything below "Done" is merged into main with `git merge --no-ff`.

## Owner words that bind this work

Verbatim,
from the brief and the plan:

- The owner picked a symbolic link at the old path ("take care of dangling links").
- "never discard user data"
- "nothing should ever wait forever or indefinitely"
- "Commits should never fail."
- "We should never invoke ourselves recursively."
- "Principle 1:
  Lint issues need to be taken care of asap."
- "I'm authorizing you to bump and publish any updates this session."
  and "Label decides" (releases,
  as alphas,
  through the release tool,
  as the latest records in `doc/planning/one-release-each-batch-2026-11.md` did).
- Never generate synthetic host load;
  every non-trivial command in `systemd-run --user --scope` with `CPUQuota`,
  `MemoryMax`,
  `TasksMax` and `IOWeight` (never above 10);
  `GIT_ASKPASS=` and `GIT_TERMINAL_PROMPT=0` on Git and `gh`;
  never `sudo`;
  never edit `doc/decision/`;
  commit with `git commit --message ... -- <paths>`,
  tests and changes in separate commits,
  never amend.

## Done and merged

- Lint of `package/repository-move/cli` and `package/repository-link/check`:
  zero warnings,
  nothing loosened.
  The sequential awaits that are the design carry one justified suppression each,
  with
  `package/repository-move/cli/doc/lint/sequential-awaits.md`.
- Runner fixes,
  each test-first:
  stage list rebuilt before each stage;
  `f1a` resolves links;
  `f1-images` rerun-safe;
  reconcile compares resolved paths;
  fixture snapshot folders inside the source image and a shared-filesystem preflight;
  the empty-parent send plan,
  the destination check,
  `verify-first` and `verify-final`;
  placement by what is at the path;
  strict runner arguments;
  stages declare their accepted exit statuses (`f5a`);
  the clone-limit allowance in the preflight.
  Facts and sources:
  `doc/troubleshooting/btrfs-send-clone-sources-need-a-parent.md`,
  `doc/troubleshooting/findmnt-two-operands-need-source-and-target.md`,
  the plan section "The send plan".
- W8:
  `package/claude-cpu-cap/daemon` unit with `ExecStartPre` running the installed check,
  and the unit installed
  as a regular copy on this host (originals under `${HOME}/temp/agent/860-w-items/`).
  Not run:
  the dangling-link run of the unit,
  which the plan leaves to the owner.
- W1 bins reinstalled to `~/.local/share/monochromatic-dev/repository-link/` (stale chunks
  `record-identity-D7BnSfbH.mjs` and `resolve-command-auK8bmv9.mjs` remain;
  the installer deletes nothing).

### Done by the second agent (merged into main `--no-ff` from the second branch)

- W2,
  the wrapper (`package/git-policy/cli`,
  Rust):
  - `try_instance_of(git_dir) -> Result<String, GitDirUnreachable>` in `src/native/service/service_instance.rs`;
    `instance_of` keeps its signature with a doc line not to name anything new with it.
  - `choose_route` returns no route for a Git directory that cannot be resolved (checked before the socket path is made);
    `service_status` says the cause;
    `refusal_without_route` in `landing/service_commit_input_notes_and_embedded_commit.rs`
    says the cause instead of "no way to start a service" (the coordinator allowed this one file).
  - `src/native/repository_link_check/`:
    the Rust copy of the W1 check (identity reader,
    answers and messages,
    `check_repository_link`
    with a worker-thread deadline of five seconds,
    the real probe,
    the paths an invocation names,
    the gate) and the shared table
    `repository_link_shared_table.json` (the TypeScript suite `check-link.unit.test.ts` reads it by relative path and a test fails
    when the path stops resolving;
    the Rust suite reads it with `include_str!`;
    both assert `rowCount`).
    The folder is listed in `doc/planning/cli-git-source-layout-target-tree.md`,
    section "Folders added after the scan".
  - `src/native/subvolume/fs_uuid_of_source.rs`:
    the one reader of the by-uuid fact (`/dev/disk/by-uuid`),
    beside `fs_mount_options.rs`.
    `read_mount_facts` gives the mount;
    no `findmnt` anywhere on the path.
  - The gate runs at the entry (`invocation/entry_repository_link_stop.rs`,
    called in `plan_invocation` before real Git is resolved)
    on the invocation's own paths (every `-C`,
    `--git-dir`,
    `--work-tree`,
    `GIT_DIR`,
    `GIT_WORK_TREE`,
    `GIT_COMMON_DIR`,
    or the
    current directory),
    and in `startup_gate.rs` (`recover_for_command_with_home`) on the Git directory and common directory the layout
    resolved.
    `Gate::RepositoryUnreachable` carries exit status 128 (three call sites).
    Message form:
    `fatal: <plain message>`.
    Order:
    identity file absent (one failed open) then return;
    no path under the link,
    the canonical root or a sibling named
    `<link>.nvme-before-860-*` then return;
    phase `before` is one `lstat` (directory) then return;
    only phase `after` walks the link.
    Measured:
    `strace` shows two failed opens of the identity file per forwarded command (entry and startup step);
    A/B perf (three alternating pairs of the baseline binary of main `4656a37fc7` and the W2 binary,
    `native:perf`,
    files under
    `${HOME}/temp/agent/860-w-items-2/perf-ab-*`) shows forwarded added median 1.5 to 2.0 ms and commit paired delta median 20.1 to 26.4 ms in both
    arms:
    the arm difference is inside the run-to-run spread.
  - Gates:
    package lint 0 warnings,
    Clippy clean,
    `native:test:container` on the second try 5265 passed.
    The first container try had 220 failures (a lock-file stall test,
    then `fresh fixture: NotFound` in every later binary test) while
    other builds ran on the host;
    the two tests that failed first pass alone on the host;
    cause not measured.
- W4,
  `package/mpm-shadow/cli`:
  `src/toolchain/repository_link_cause.rs`.
  `ensure_launcher` stops with the check's message before any
  write when the launcher folder is under the repository and the link is not usable;
  `run_launcher` prints one warning line naming
  the cause when the lock file cannot be read under the repository,
  and still runs the tool.
  The check is the installed
  `repository-link-check`,
  started once,
  only on those paths and only when the identity file exists (a ready answer is remembered per
  process),
  with a ten second kill.
  `shadow_identity.rs` is untouched.
- W5:
  no code.
  A launcher link that dangles never starts our code (the shell reports it);
  an existing test covers a missing real program.
- W9:
  the fourth bin `repository-link-launch` in `package/repository-link/check` (table `LAUNCHED_PROGRAMS` in `src/constants.ts`).
  Forms tried:
  a link to `repository-link-run` (cannot map a link name to a program),
  a repository bin (dangles),
  a Rust bin (a second
  copy of the check),
  a mise or systemd setting (not a command name);
  a bin in the W1 package started through a link named like
  the program does it.
  Installed with `mise run //package/repository-link/check:install:outside-repository`;
  the two links in `~/.local/bin`
  now lead to it (originals in `${HOME}/temp/agent/860-w-items-2/home-originals/local-bin-links.txt`);
  `--help` of both works.
- W10,
  `package/env-local-json/core`:
  `repository-link-cause.ts`;
  `EnvLocalJsonPathError` and `EnvLocalJsonReadError` take `linkCause`;
  the reader (explicit path and unreadable store) and the writer (store behind a dangling link) say the drive is not mounted.
  New dependency `@monochromatic-dev/repository-link-check` (`workspace:*`);
  the lock diff is three lines.
- W11,
  `package/repository-move/cli`:
  `prepareUnitEdit` also replaces the unit's stop block (STOP_PATH_MARKER `held open`):
  a held mount
  prints the holders (`fuser -vm`),
  exits 1 and never runs `luksClose`;
  no lazy or forced unmount.
  Stages `m1a` and `m1b` carry it.
  The test runs the script on stand-ins with `bash --noprofile --norc`,
  a scratch mount point and a scratch mapper name:
  `bash -c` started over a pipe from Node sources `~/.bashrc` and gets the real PATH back,
  so a test that does not pass `--norc`
  runs the real `umount`,
  `fuser` and `cryptsetup`.
  The run on the real unit and the three-state report are the owner's,
  as root.
- Bash startup files in `package/repository-move/cli` (coordinator item):
  every bash the package starts passes `--noprofile --norc`
  (the runner shell,
  the copy tool's send and receive shells,
  the inner `sudo unshare ... bash` of the cut-over and loop-image stages);
  `src/bash-without-startup-files.unit.test.ts` holds a marker test and a static scan for a bash start without both flags.
  Measured 2026-11-07 for the troubleshooting doc `doc/troubleshooting/bash-c-started-by-node-reads-bashrc.md`
  (still on the unmerged W3 branch,
  to be amended there):
  the trigger needs a socket on standard input and an inherited `SHLVL` that is not a positive number
  (unset,
  `0` and `x` read `~/.bashrc`;
  `1` and `2` do not),
  so a run from an interactive terminal usually does not trip it,
  but one from systemd,
  `sudo` or a scrubbed environment does.
  The runner shell gives its child `stdin: 'ignore'`,
  so it never tripped;
  the copy tool's `execFile` shells could.
  No other package this work touched starts bash from TypeScript
  (the Rust `Command::new("bash")` calls are tests,
  started with an anonymous pipe,
  not a socket).
- Runner fix (coordinator,
  after fixture run `prepare-9` stopped at `f5a-deadline`):
  the fixture images are sized by `fixtureImageGiB()`
  from `FIXTURE_MAIN_DATA_MIB`,
  `FIXTURE_SUBVOLUME_COUNT`,
  `FIXTURE_COPY_SETS_KEPT`,
  `FIXTURE_GUARD_MIN_GIB` and the plan's quarter margin
  (six GiB each);
  the free-space stop names the folder whose filesystem it measured.
  Resume:
  unmount the old loops,
  then
  `prepare --redo=f1-images` and every later fixture stage id from the list in the runbook (a fresh image is made only when the file
  does not exist,
  so delete the old `source.img` and `dest.img` from the list the last fixture stage prints first).
- W12 to W15:
  runner stages `c23-link-check` to `c28-ide-through-link` after the release in the cut-over (as the user,
  none with sudo):
  the link check again (W15),
  `mise trust --all` and `--show` (W14),
  a fresh worktree with `pnpm install --frozen-lockfile --offline` and
  `find node_modules -xtype l` (W14),
  the first kopia snapshot of the source (W12),
  a container with a read-only bind through the
  link on the first local image (W13),
  and an assertion stage for the IDE (W13).
  The wrapper's doctor prints
  `repository link: <message>` (W15) through `filesystem_section_with_link_line` in `invocation/management.rs`.
  None of these ran for real:
  they run at the cut-over,
  and the stage data is tested by the runner's unit tests and its dry run.
- Lint fixed on the way:
  `package/repository-link/check/README.md`,
  `package/repository-move/cli/doc/lint/sequential-awaits.md`,
  the runbook,
  and the new test code of W1.

- W3,
  the work loop (`package/git-policy/cli/src/native/landing`):
  `drain_planned_with` in `service_work.rs` asks `try_instance_of` first;
  a Git directory that cannot be reached goes to `keep_plans_while_unreachable` (new,
  in
  `service_work_handlers_replanning_and_acceptance.rs`):
  it runs nothing,
  touches nothing in the directory,
  and plans every due push of
  that directory again with the next doubled pause (`planned_retry`,
  so the plan record and the journal line stay).
  A wake for such a
  directory already left its work alone (a guard test records it).
  `service/service_server_work_loop.rs` needed no edit.
  Tests in
  `service_push_retry_tests.rs`:
  the directory is renamed away and back between rounds.
  A request that finds its Git directory unreachable is answered by the wrapper before it reaches the service (the gate,
  and
  `refusal_without_route`).
- Freeze text:
  `FailureCode::PermissionDenied` (EACCES,
  wire word `repository-permission-denied`) beside `RepositoryReadOnly` (EROFS);
  `storage_failure` and `classify` keep them apart;
  `ClientFailure::PermissionDenied` and `ClientFailure::RepositoryFrozen` have their own
  sentences;
  `read_only_failure` in `service/service_client.rs` picks the frozen one only when the freeze manifest in the runner's
  state folder (`MOVE_STATE_DIR`,
  else `~/repository-move-state`) lists a subvolume holding the repository
  (`repository_link_check/repository_move_freeze.rs`).
  The thaw pass renames the manifest to `freeze-manifest.tsv.thawed-<ms>` after a
  successful thaw (a second thaw then succeeds and changes nothing),
  so a present manifest means frozen.
  The protocol version is
  unchanged:
  a new failure word reaches an older client only when a service of another release is still running,
  which the wrapper retires;
  bumping the protocol is the owner's call.
- W2 latency (coordinator's bar:
  no forwarded command slower beyond the same-binary spread).
  Method:
  the wrapper of main before the
  merge (`959402bca7`) against the final branch binary,
  alternating arms,
  three rounds of 60 runs per arm,
  `git -C <repository> status --short` and `rev-parse HEAD`,
  the real 860 as the repository's filesystem,
  three identity states
  (scripts and results under `${HOME}/temp/agent/860-w-items-2/ab-measure/`,
  `result-1.txt` before the memoizing,
  `result-3.txt` final),
  run while other sessions and a gate container kept the host busy.
  Median of the three round medians,
  in milliseconds
  (the spread of the same binary across rounds was 0.4 to 0.8):
  identity file absent,
  status:
  5.31 before and 5.34 after;
  rev-parse 4.55 before and 4.80 after;
  phase `before`,
  status:
  5.39 and 5.54;
  rev-parse 4.58 and 4.47;
  phase `after`,
  status:
  5.04 and 6.01;
  rev-parse 4.74 and 5.58.
  So absent and `before` are inside the spread.
  Phase `after` is slower by about 1.0 ms (status) and 0.8 ms (rev-parse) at the median
  and 0.5 to 0.8 ms at p95,
  beyond the spread.
  The first version cost 1.2 to 1.4 ms there;
  reading the mount table and each device's UUID once per
  check and remembering a usable repository for the rest of the process (the entry gate and the startup step meet the same
  identity) took off about a third.
  What is left is one walk of the link per command (the thread for the deadline,
  `readlink`,
  `realpath`,
  the table,
  the UUID links,
  the marker file) on every command that touches the repository once the move is done.
  Cheaper options not built:
  skip the walk for a read-only command,
  or cache the answer across commands (a file in the runtime folder);
  both change what the gate guarantees,
  so they are the coordinator's and the owner's choice.
  `native:perf` (empty home,
  so only the failed open):
  forwarded added medians 1.5 to 2.0 ms and commit paired deltas 20 to 26 ms in
  both arms of three alternating pairs,
  inside the spread.
- Footprint measurement (coordinator,
  after `prepare-10` stopped at `p3a-footprint` when a worktree was removed during the walk):
  merged into main as cf498e159e from `fix/repository-move-footprint-retries-removed-worktrees`
  (`footprint-measure.ts`,
  `measure-footprint.ts`,
  tests in `measure-footprint.unit.test.ts`).
  One whole-tree walk,
  made again at most
  three times when every line btrfs printed says an entry was not there;
  any other error or a walk past its deadline fails at once.
  It is not split by subvolume:
  worktrees share extents (3753 GiB per-worktree sum against about 264 GiB counted once),
  nested
  subvolumes would count twice,
  and a `btrfs filesystem du` call with several paths prints one independent row per path.
- The wrapper's container gate fails one test in main itself,
  not in this branch:
  `fault_limits::a_memory_limit_close_to_exhaustion_on_the_service_leaves_a_recoverable_repository` (the service aborts with
  `memory allocation of 13 bytes failed` under the address-space limit before it listens).
  Main 48c8fe6692 plus only the runner fix
  fails it in the gate image;
  this branch failed it 5 of 5 alone in the gate image;
  the older image `coordinator-tokio-062` passes it;
  the host passes it.
  The commit-stall merge's startup warm is the coordinator's lead and its author is investigating.
  This branch merges only after that fix is in main.
- Open harness item,
  container run 1 (220 failures):
  217 `fresh fixture: NotFound` (the fixture root `target/tmp` of the container
  vanished after test 5018 of 5265),
  one lock-file stall test,
  one settle test,
  one service start;
  memory peak 492 MiB of 2 GiB,
  no
  disk or process-limit line,
  other agents' gates ran at the same time;
  the identical Rust source passed on two later runs.
  What removed `target/tmp` is not identified;
  proposal:
  `fixture_in` names the state of the fixture root and its parent when
  `create_dir` fails with NotFound.
  Log:
  `${HOME}/temp/agent/860-w-items-2/container-1.log` (scratch,
  may be gone).
- Removers that can reach a path named `target` (the coordinator's lead on run 1,
  read-only;
  nothing was rerun).
  Non-test
  `remove_dir_all` and `remove_dir` call sites of `package/git-policy/cli/src/native` and `package/cargo-shadow/cli/src`,
  with what
  each takes as its root (read from its documentation and signature,
  not by a test):
  `worktree_copying/worktree_copy_journal_removal.rs` (subtrees named by a copy's journal,
  under the destination worktree),
  `worktree_copy_stage.rs` and `worktree_copy_transaction.rs` (the stage folder beside the destination worktree),
  `shadow/shadow_links.rs` and `shadow/shadow_create.rs` (shadow folders of one transaction under the Git directory),
  `foundation/scratch_location.rs` (entries older than a day in the per-user or repository scratch folder),
  `foundation/private_storage.rs`,
  `service/service_busy.rs`,
  `candidate/candidate_private_index.rs`,
  `landing/service_land_copy.rs`
  (each its own record or scratch folder),
  `worktree/worktree_removal_rescue.rs` (snapshots of one removal),
  and `cargo-shadow/cli/src/journal_done_marker.rs` (its own journal folder).
  The Cargo repair (`repair/package_repair_cargo.rs`) removes nothing itself:
  it runs `cargo clean --offline --package <member>` in
  each workspace of the new worktree.
  Its roots all come from arguments (a worktree,
  a Git directory);
  none reads `CARGO_TARGET_DIR`
  or a workspace variable,
  and the gate image sets none (`CARGO_HOME` only).
  Not checked:
  whether a fixture's Cargo workspace,
  which lives inside the container's `/work/package/git-policy/cli/target/tmp`,
  resolves a `target-dir` or `build-dir` from a
  `.cargo/config.toml` found by walking up to the container's `/work`;
  the repository's root `.cargo/config.toml` sets only
  environment values.
  That is the one lead left open;
  the new fixture-root report names the folder that went the next time.
- W13 stage `c27-container-bind` starts a fresh container with a read-only bind through the link;
  the plan's test was to start one
  existing `created` container with worktree binds (`podman ps --all --filter status=created` lists them);
  done by hand at 2026-11-08T16:21:32-05:00:
  `podman start --attach c42af057a886`,
  which binds `catalog-regenerate-every-lock/package/ide/app` through the repository link,
  exited 0 and `cargo metadata` printed its packages,
  so the bind resolves.
  The container is gone afterwards,
  which matches its created twin `7cf22bd2610a` reporting `AutoRemove` true.
  Of the five created containers,
  only those two have binds that still exist.
- The mount unit's new stop block runs `luksClose` unconditionally when the mount is not a mount point,
  so a stop with no mapper now
  fails where the old stop hid it;
  a guard `[ -e /dev/mapper/crypt_sda ]` is the owner's call.
- The startup gate and the doctor line read `HOME` from the process environment while the entry gate read the
  invocation's environment (rule HOM);
  done in `a31518394e` and `2b9567c8ad`:
  `recover_for_command` now takes the environment and resolves the home through
  `child_environment::environment_home`,
  `prepare_guarded_command`,
  `recover_before_read_only` and `plan_direct` pass it down,
  and `plan_doctor` binds the invocation's home into the doctor's filesystem section.
  The control plants an identity file in the home the invocation names,
  with a link the worktree sits under,
  and requires the stop the old process-environment read answered as `Clear`
  (`invocation/startup_gate_tests.rs`).
- The other two reads of `HOME` from the process environment,
  `route_from_environment` (`landing/service_route.rs`) and `route_for_this_worktree`
  (`landing/service_commit_input_notes_and_embedded_commit.rs`),
  are right as they stand and each now says why beside the read (commit `9d0fbcab34`):
  `main` hands the entry `std::env::vars_os()` as the invocation's environment,
  no source of the package changes the wrapper's own environment
  (`rg 'set_var|remove_var'` over `package/git-policy/cli` finds none),
  and no service chooses a route,
  because a service start goes to `service_main` before any command's route is picked
  and all three callers (`worktree_trash_dispose`,
  `auto_push_trigger`,
  `service_commit`) run in the process of the command that asked.
  So the startup gate and the route read one home for one command,
  and no read inside a serving process can answer with the manager's home instead.
  The third process-environment `HOME` read,
  in `repository_move_freeze::move_state_folder`,
  is reached from the client's rendering of a read-only failure (`service_client.rs`),
  so the same reasoning covers it and it was left as it is.
- `mise trust --show` prints `<path>: trusted` or `<path>: untrusted` (checked in this worktree),
  which `c24-mise-trust` greps.

### Blocked, with what is needed

- W6 and W7 (home configs):
  the auto-mode classifier refused every write to `~/.claude/settings.json`,
  `~/.claude.json` and
  `~/.codex/config.toml` (self-modification and credential files),
  and refused the backup copy of the last two.
  Nothing was changed.
  The owner then picked "I apply them at cutover (Recommended)"
  (`doc/handover/questions-for-the-owner.md`),
  so the coordinator makes the edits right before the cutover starts.
  The exact edits are in the section "W6 and W7 edits to apply" below.
- Releases:
  wrapper,
  mpm-shadow,
  repository-link-check and env-local-json changed;
  the coordinator wants to be asked before the first
  release and the order of the wrapper branches is theirs.
  Nothing was released or locked.

### W6 and W7 edits to apply

Run through the installed `/home/user/.local/share/monochromatic-dev/repository-link/repository-link-run`
(the absolute path;
the manager and Claude do not read the shell's PATH):

- `~/.claude/settings.json`,
  `hooks.PreToolUse[0].hooks[0].command`:
  `/home/user/.local/share/monochromatic-dev/repository-link/repository-link-run node /var/home/user/Monochromatic/package/claude-code-plugin/bash-output-filter/bundle/node/index.mjs`
- `~/.claude/settings.json`,
  `statusLine.command`:
  `/home/user/.local/share/monochromatic-dev/repository-link/repository-link-run node /var/home/user/Monochromatic/package/claude-code-plugin/statusline/src/statusline.ts`
- `~/.claude.json`,
  `mcpServers.mvm` and `mcpServers["search-fetch"]`:
  `command` becomes the `repository-link-run` path and `args`
  becomes `["--wait-seconds=20", "node", <the script it had>]` (the `env` stays).
- `~/.codex/config.toml`,
  `[mcp_servers.search_fetch]`:
  `command` becomes the `repository-link-run` path and `args` becomes
  `["--wait-seconds=20", "/home/user/.local/bin/mise", "--cd", "/home/user/Monochromatic", "exec", "--", "node", "/home/user/Monochromatic/package/search-fetch/mcp/dist/final/node/index.mjs"]`.
- Cost measured:
  `repository-link-run true` takes about 0.03 s on this host,
  added to every Bash hook and status line start.
- Pipe-tested 2026-11-08T03:43:42-05:00 with no config changed
  (scripts `${HOME}/temp/agent/860-move/pipe-test-w6-w7.ts` and `run-mcp-probes.ts`):
  the hook and the status line gave byte-identical stdout and status 0 plain and through `repository-link-run`
  (one sample each:
  hook 33 ms plain and 93 ms wrapped,
  status line 58 ms and 85 ms,
  so the added time on this loaded host was 27 to 60 ms rather than the 0.03 s measured earlier;
  no spread was taken).
  The three MCP commands (`mvm`,
  `search-fetch` and Codex's `search_fetch`) gave the same first JSON-RPC reply plain and wrapped,
  17 to 27 ms later wrapped.
  That reply is error -32601 for `initialize`
  (the servers say they implement a later MCP revision),
  so the probe shows only that stdin and stdout pass through the wrapper unchanged,
  not a full handshake.
- A running Claude session rewrites `~/.claude.json` often:
  make the edit with the sessions idle,
  or expect to redo it.

## Decisions, with their source

Coordinator answers of 2026-11-07,
after the file list for W2 and W3:

1.  Reader for W2:
    reuse what is on main,
    never copy from another worktree.
    First grep main for an existing `by-uuid` lookup;
    if none,
    add one small function beside
    `src/native/subvolume/fs_mount_options.rs`,
    with its own tests,
    and use `read_mount_facts` for the table.
    One reader of each fact in the crate.
2.  One fixture table:
    a single JSON file inside `package/git-policy/cli` (the container gate copies only a crate snapshot,
    so a path outside the crate fails there and in the packaged crate).
    The W1 TypeScript test reads it by relative path;
    a comment in both suites names the other;
    both assert the row count and iterate every row;
    add a W1 test that fails when the path does not resolve.
3.  Folder `src/native/repository_link_check/` until the layout move (issue 6261 steps 2 to 6).
    No `uses/` folder (no folder of the crate has one;
    the move adds them all at once).
    Add the folder to the list of folders to move in `doc/planning/cli-git-source-layout.md`.
4.  Gate in `startup_gate.rs`,
    `recover_for_command(context, identity)`,
    in this order:
    identity file absent,
    return;
    resolved Git directory not under the link,
    the canonical root or the stale NVMe path,
    return;
    phase `before` with one `lstat` saying directory,
    return;
    only phase `after` with a link does the full walk.
    No process start on the hot path (forwarded p95 near 2 ms):
    no `findmnt`.
    Take the `native:perf` baseline from current main before the first edit of the invocation entry
    (same-binary spread first,
    saved under `${HOME}/temp/agent/860-w-items/`) and report before and after.
5.  Frozen-move message ("the repository is frozen for the move to the 860;
    nothing was written;
    try again after the move finishes"):
    translate `ErrorKind::ReadOnlyFilesystem` at the one place where commit I/O errors become user text
    (grep for it;
    send the coordinator the file and function before editing it if it is under `service/` or `landing/`).
6.  `src/native/service/service_instance.rs` and `src/native/landing/service_route.rs` may be edited now:
    `try_instance_of(git_dir) -> Result<String, GitDirUnreachable>` (error when `canonicalize` fails;
    `instance_of` keeps
    its signature,
    with a doc line not to use it to name anything new);
    `choose_route` returns no route for an unresolvable Git directory;
    `service_status` says the cause.
7.  W3 waits:
    `service_work.rs`,
    `service_work_handlers_replanning_and_acceptance.rs` and `service_server_work_loop.rs`
    only after the coordinator says the commit-stall branch is in main.
    Keep the change to the smallest function-level edit with behavior tests,
    because tokio step T10 converts the work loop.
8.  W9:
    rule SCR forbids shell scripts,
    so do not write one and do not raise it with the owner yet.
    Find a form without a script (a link to an existing bin,
    a TypeScript bin in a repository package,
    a Rust bin,
    a mise or systemd setting) and tell the coordinator what the two lines must do and which forms were tried.
    One candidate:
    a bin in `package/repository-link/check` that,
    invoked through a link named `spawn-claude` or
    `spawn-pi`,
    runs the check and then the program that name maps to.
9.  Order of work:
    W2,
    W3 (when told),
    W4 and W5,
    W10,
    W6,
    W7 and W9,
    W11,
    W12 to W15,
    then the releases and lock moves.
    Keep this handover current after every merged item.
10. Merge gate for the wrapper:
    merge main first,
    `mise run toolchain:install`,
    the package lint (0 warnings),
    Clippy,
    one `native:test:container` unit;
    merge `--no-ff` only on a full pass.
    The repository-lint branch that reflows doc
    comments in the package merges first:
    use the recipe in `package/git-policy/cli/README.md`,
    "Merging main into a branch that edited doc comments".
    The wrapper is the one crate not released by the agent:
    report its change,
    the owner releases it.


Coordinator answers during the second agent's work (2026-11-07):

11. Second gate site:
    the same check also runs once in `plan_invocation` (before real Git is resolved) on the invocation's own paths;
    `recover_for_command` keeps the paths the layout resolves;
    both call one function in `repository_link_check/`.
    The first step is the cheapest test (identity file absent:
    one failed open).
    Red tests:
    each of the four traced invocations
    prints the plain message and exits 128,
    not Git's `fatal:`.
12. Freeze text:
    not a render-only change (errno 13 must not say a move runs);
    done properly after the commit-stall branch,
    as part of W3
    (see "Blocked,
    with what is needed").
13. `route_for_this_worktree` may be edited so a `None` route from an unreachable Git directory says that cause.
14. The wrapper release line of decision 10 is superseded by the second brief:
    alpha releases for every changed crate,
    after asking
    the coordinator first.

## Next, in order

Merged into main as `f981e1b493` (2026-11-08,
gate 5319 of 5319 in the container,
package lint 0 warnings):
the second branch with W3,
the freeze text,
W12 to W15,
the doctor line and the sibling tools.
The first gate run on it failed one test,
`lock::stale_ref_locks::tests::locks_left_by_a_killed_fetch_are_cleared_and_the_next_fetch_works`
(two locks expected cleared,
one was);
the branch does not touch `src/native/lock`,
the test passed alone and in the second full run,
and no cause or rate was measured:
an open flake for an owner.
Then the releases (alphas for the crates changed,
then `toolchain:update`,
`toolchain:install`,
commit the lock;
ask the coordinator first;
the wrapper alpha.2 is the coordinator's).
W6 and W7 stay the owner's call (see "W6 and W7 edits to apply").

Hot-path facts to keep:
phase `before` is one `lstat`;
the identity file read is a failed open when absent (measured).

## Things a fresh agent should not repeat

- `btrfs subvolume show` is refused to a user;
  `btrfs send` needs root,
  so send behavior is only checked by the coordinator
  on the fixture (`prepare`),
  through the commands this session sent as messages.
- Never merge into main while another agent's merge is in progress in the main checkout (check `git status` there).
- `git commit` through the wrapper may rewrite a file's final newline:
  check `git status` after.
