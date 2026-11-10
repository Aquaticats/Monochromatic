# Renaming the Git wrapper program and its command word

Status:
decided by the owner;
step 1 is built on the branch `feat/git-policy-cli-program-rename` (see "Step 1 as built"),
unmerged and unreleased.
The work waited for the four lint branches of the crate (`refactor/git-policy-cli-lint-*`) to merge,
because the command word is read in `invocation/management*.rs`,
which one of them was editing.
They merged on 2026-11-05;
the branch started from `a7e32e118e`,
the gated suppression branch
(worktree `.claude/worktrees/git-policy-cli-program-rename`).
The proposed handling of the settings file and data folders is adopted as written,
owner veto open.
Steps 2 and 3 are done:
the release is 0.6.0 and step 3 is recorded in "Step 3 as built".

## What the owner decided

2026-11-04,
after `AGENTS.md` VGW ("a word that fits any code (`native`,
`core`,
`helpers`) never stands alone")
named the program `cli-git-native` as a vague name:

- "Rename it too."
  (the installed program `cli-git-native`)
- Program name,
  through the question tool,
  verbatim note:
  "follow the actual crate's name."
  So the program becomes `monochromatic-git-policy-cli`,
  the crate's name.
- The command word,
  through the question tool:
  "B:
  rename the word too".
  So `git cli-git <command>` becomes `git monochromatic-git-policy-cli <command>`
  (`doctor`,
  `check`,
  `fix`,
  `migrate-to-subvolume`,
  `convert-worktree-to-subvolume`,
  `service run`,
  and the rest).
- The old spelling,
  through the question tool:
  "A:
  still works,
  with a one-line notice (Recommended)":
  `git cli-git ...` keeps working in the new release and prints one line naming the new spelling.
- Package folders,
  through the question tool:
  "A:
  exempt package folders (Recommended)";
  VGW now ends "Package folders are exempt."

## What changes

- `package/git-policy/cli/Cargo.toml`:
  the `[[bin]]` name (the file is managed by file-enforcer,
  `CLAUDE.md` WC2:
  edit its source and run `file-enforcer write`).
- The command word in the management grammar (`invocation/management_arguments.rs` and its users),
  the help texts,
  the service mode the service unit starts (`... service run`),
  and every test that types the word.
- The old word:
  accepted,
  with one line on standard error naming the new spelling,
  then the same behaviour.
- The crate's tasks (`mise.toml`:
  `--bin`,
  `target/release/<name>`),
  its tests that name the built file
  (`CARGO_BIN_EXE_...`,
  `release_bins_tests.rs`,
  launcher fixtures),
  README,
  SPEC and the e2e TypeScript.
- Other packages:
  `package/git-executable/core` (`self-shim.ts` lists the names it must never pick as the real Git:
  add the new name,
  keep the old one for older installs),
  and the references in `mpm-shadow`,
  `crate-release`,
  `mutation-test`,
  `pnpm-shadow` and `dependency-catalog`
  fixtures (each read before changing:
  some are comments,
  some are behaviour).
- Workflows:
  `cargo-publish.yml` (build `--bin`,
  the archive layout cargo-binstall reads,
  the `--help` check),
  `npm-release.yml`,
  `catalog-specifiers.yml`.
- The root `mise.toml` task that runs `cli-git-native cli-git fix ...`.
- `toolchain.plan.jsonc` and `toolchain.lock.json` (the `bins` list and the postinstall `sourceBin` that links the
  program as `git` under `.cache/cli-git-bin`):
  only after the release that carries the new name is published,
  because the lock must name a file the archive holds.

## The settings file and the data the wrapper keeps

The word `cli-git` also names the settings file (`cli-git.config.jsonc`),
data folders (`cli-git/discarded`,
`cli-git/notices`),
log files (`cli-git-audit.jsonl`),
the link folder `.cache/cli-git-bin`,
the service units and the
`cli-git:` prefix of messages:
162 string literals in 65 files.
Told that renaming the settings file or the data folders would leave existing ones behind,
the owner answered,
2026-11-04:
"rename them."

Proposed handling,
following the owner's choice for the command word (the old spelling keeps working with a notice);
open to veto:

- The settings file becomes `monochromatic-git-policy-cli.config.jsonc`.
  In this repository the committed file is renamed in the same change.
  In a repository that still has only the old file,
  the wrapper reads it and prints one line naming the new name;
  it never renames a committed file by itself.
  When both exist,
  the new one is read and the line names the old one as ignored.
- The data folders and files the wrapper keeps inside the Git directory (saved copies,
  notices,
  logs,
  records)
  take the new name.
  The first time the wrapper needs one and finds only the old name,
  it renames it in place (one atomic rename in the
  same folder),
  so nothing is copied and nothing is left behind.
  When both exist,
  the new one is used and the old one is left untouched and reported,
  never deleted
  (owner:
  "never discard user data").

Asked about the link folder `.cache/cli-git-bin`,
the service unit names and the `cli-git:` message prefix,
the owner picked "A:
rename them too (Recommended)".
The new units replace the old ones when the old helper is replaced;
a helper still running under an old unit keeps
landing commits until then.

## Order

1.  One branch:
    program file,
    command word with the old word accepted,
    tests,
    docs,
    workflows,
    `git-executable` and the other packages;
    gated.
2.  Release the crate;
    the archive holds `monochromatic-git-policy-cli`.
3.  Lock the release:
    `toolchain.plan.jsonc`,
    `toolchain.lock.json`,
    the root `mise.toml` task;
    install;
    check that `git` resolves to the new file and that a commit,
    a push and `git cli-git doctor` (old word,
    with the
    notice) work,
    and that the running background helper started by the old program still lands commits.

## Older installs

Worktrees still on an older lock keep `cli-git-native` and the old word;
the new release accepts the old word,
and `git-executable` keeps the old name in its list.
A background helper started by an older program keeps running from its versioned install folder until it is replaced.

## Step 1 as built

Recorded 2026-11-04T22:36:12-05:00 on the branch `feat/git-policy-cli-program-rename`;
the sections from `What the owner decided` to `Older installs` stay as decided.

### Mechanism

- One pair per renamed name lives in `package/git-policy/cli/src/native/foundation/wrapper_names_old_and_new.rs`
  (program,
  command word,
  settings file,
  message prefix,
  data folder,
  four logs,
  merge records,
  ignored-state copy records,
  service unit);
  no other file spells a pair.
- Where the wrapper first learns the Git directories of a command (`GitFacts` in `repo_read/repository_facts.rs`,
  once per
  process),
  `foundation/kept_names_in_the_git_directories.rs` moves each earlier name to the current one with
  `renameat2(RENAME_NOREPLACE)` (`foundation/earlier_name_moved_to_current_in_place.rs`):
  no lock,
  nothing copied,
  a lost race
  or a crash between the look and the rename leaves a state the next run finishes.
  The audit log's numbered older copies move with it.
  Readers that visit another worktree's Git directory (`path_for_reading`) never move anything.
- The settings file:
  `LoadedConfig.earlier_name_notice`,
  said once per process on standard error.
  An invalid earlier file is an error that names its path,
  without the notice.
- The earlier command word:
  `EARLIER_COMMAND_WORD_NOTICE` in front of the same action;
  `EARLIER_SERVICE_WORDS` starts the
  service mode too,
  so a unit file an older program wrote still starts a helper.

### Open batches of the old helper

Recorded 2026-11-04T23:07:31-05:00.
The service journal holds one file per batch:
`<seq>.intent` while the batch may be unfinished (the branch moves only after
the intent is on disk),
`<seq>.done` after it,
and the request identities a done batch remembers.
A helper killed between the branch move and the index update,
or whose session ended,
leaves an `.intent` in its own state
folder;
the unit of this release keeps the journal in another folder,
so nothing would have looked at it,
the person's index
would have shown the landed commit as reverted,
and a client the old helper never answered would have had its commit landed
twice on a retry.

- `landing/earlier_service_journal_adopted.rs`:
  when the service of this release starts for an instance,
  and its journal is
  where the unit puts it,
  it runs the existing recovery over the earlier journal of that instance and the first releases' shared
  journal above it,
  in place (nothing is moved or copied;
  a finished batch is marked done where it was),
  before it accepts a
  connection.
- One owner:
  it does so only when no process of the same user has the start arguments of a service of that instance
  (`/proc`) and no socket of the earlier units listens (see "Changes after the fourth review"),
  so a running helper finishes
  its own batches and two services never settle one batch.
  A look that cannot be made counts as "may be running";
  the adoption is tried again at the next start of the service
  (it exits when idle and the socket starts it again),
  so a false "may be running" defers it until then;
  nothing waits for it,
  nothing takes a lock.
  The start arguments are recognised in every shape an earlier release wrote,
  see "Changes after the second pass".
- The service of this release also counts the identities in the earlier journals as landed,
  so a retried request is answered
  as already landed.
- Not covered:
  a helper of another PID namespace (a container) is not seen by the look,
  and neither is a helper of another
  user;
  the earlier helper restarted by its own unit (`Restart=on-failure`) while the adoption runs could settle the same
  batch,
  which the recovery's checks of the branch and the index make a repeat of the same step,
  not a second landing.
- The audit log split across two files in a mixed-install window loses nothing;
  it is documentation only.

#### Changes after the second pass

Recorded 2026-11-05T00:54:16-05:00,
after a same-vendor second pass (a cross-vendor review of the same change had not returned).

- The look recognises a service by its journal as well as by its mode words:
  `--journal` followed by the earlier per-instance
  folder or the shared folder.
  Units written before `9b360150c0` started the helper as `<program> --journal <state>/cli-git/service/<instance> --git <git>
  --instance <instance>` and the first releases' shared service had no instance at all;
  a running process keeps the arguments it
  was started with,
  and the idle exit is three hours,
  so both can be alive.
  The mode words with `--instance` stay as the second accepted shape.
- The adoption never drops an intent.
  A live landing writes its intent,
  waits for the index lock and only then moves the branch,
  so "the branch has not moved" can mean
  "about to move";
  removing the intent then left a landed commit with no record.
  The adoption finishes what landed (a finish never moves the branch,
  its index replay needs the lock and the records,
  and two
  finishes end in one `.done`) and leaves an intent that looks unlanded on disk with its records,
  reported as left,
  for the earlier
  helper's own start-up recovery.
  If the earlier units are removed,
  such an intent is never settled;
  it costs a file and one log line at each start.
- A Git question about the branch that gets no answer keeps the batch open for the service's own recovery as well as the adoption:
  a Git that cannot be started,
  any exit but "no such reference" (`rev-parse --verify --quiet` exit status 1 with nothing on the
  standard error),
  and an ancestry question that fails other than with "not below".
  Before,
  the failure was read as "the branch has not moved" and the intent of a landed batch was dropped.
- The look counts only processes of the same user (the owner of the `/proc/<pid>` folder),
  as this plan always said.
  A listing,
  open or read error other than the process being gone (`ENOENT`,
  `ESRCH`) counts as "may be running".
- The look has a ceiling (`PROCESS_LOOK_CEILING`).
  Reading `/proc/<pid>/cmdline` of another process takes that process's address-space lock with `mmap_read_lock_killable`
  (Linux `mm/memory.c`,
  `__access_remote_vm`,
  reached from `fs/proc/base.c`,
  `get_mm_cmdline`;
  read from `torvalds/linux`
  `05441d697f7f` on 2026-11-05),
  so a process stuck in the kernel can hold the read for good,
  and only a fatal signal to the reader
  ends it.
  The scan runs on its own thread,
  and no answer in time counts as "may be running";
  the stuck thread is left behind.
- The earlier journals are read without removing anything:
  the listing that tidies old `.tmp` and `.claim` files is not used on
  them,
  because they may belong to a live helper.
  The adoption does not prune finished batches there either.
- The adoption logs each batch (finished,
  left open,
  intent left on disk) and the skip of the whole adoption,
  naming the folder.

Still uncovered:

- The earlier journals' finished batches are read in full at each start and never pruned (a cost of time,
  not of data).
- The uid filter and the ceiling have tests that need the new look signature,
  so they could not be red on the old code;
  each was shown to fail by removing its guard once (2026-11-05) and restoring it.
- The Windows build of the new look module was not compiled:
  the Windows target is not installed here,
  and the module's
  Unix-only parts are behind `cfg(unix)`.
- The look sees only this PID namespace and this user;
  a helper run by another user for the same worktree is not waited for.
- The retried-request path (`earlier_commits_landed`) matters little while the earlier units still exist:
  a client uses a fresh
  request identity for each invocation,
  an old-release client retries against the old socket,
  and `Restart=on-failure` restarts a
  killed old helper within a fraction of a second,
  so the old helper settles its own batches before the new service reads anything.
  The adoption matters mainly when the earlier units are disabled or removed;
  the second pass's risks (all addressed above)
  weigh against that.

#### Changes after the fourth review

Recorded 2026-11-07T02:25:36-05:00,
after the fourth cross-vendor run
(`~/temp/agent/issue-6261/sol-review-retry-4/review.txt`) found three defects in the look.

- What the earlier helper holds,
  read at tag `monochromatic-git-policy-cli-v0.5.34` (the release installed on this machine,
  `monochromatic-git-policy-cli 0.5.34`):
  nothing this program could also take.
  `service_main.rs` creates the journal folder,
  binds the socket the manager passed,
  recovers the shared journal,
  and serves;
  it takes no `flock`,
  writes no pid file,
  and opens no singleton.
  The `O_EXCL` file in `service_journal.rs` (`claim_batch`) is per batch number,
  and the `flock` in `service_land_copy.rs`
  is on a private copy folder.
  The only gate on its start is the manager's socket activation:
  the unit `cli-git@<instance>.service` `Requires` and runs
  after `cli-git@<instance>.socket` (`ListenStream=%t/cli-git/%i.sock`),
  and the first connection starts the helper.
  So the reviewer's first choice (an exclusion both sides honour) does not exist,
  and its second (defer) is taken.
- The adoption defers while a socket of the earlier units listens:
  the worktree's `<runtime>/cli-git/<instance>.sock` and the
  first releases' `<runtime>/cli-git/service.sock`.
  The look reads `/proc/net/unix` and takes a line whose flags column has `__SO_ACCEPTCON` (`1 << 16`,
  Linux
  `include/linux/net.h`;
  `net/unix/af_unix.c`,
  `unix_seq_show`) and whose path ends with one of those names.
  It does not stat the socket file (a stopped socket unit leaves the file unless `RemoveOnStop=` is set,
  and the unit text does not set it) and
  never connects (a connection starts the helper).
  The sockets are read before the processes,
  so once none listens,
  no helper can be started by a connection and the
  processes seen afterwards are all there can be.
  A table that cannot be read counts as "may be running".
  Measured on this machine at 2026-11-07:
  with the three earlier sockets listening,
  the look answers yes for one of
  their instances and no for an instance that has none,
  over a `/proc` of 744 processes,
  in 9 ms (a test that called the look on the real `/proc`,
  removed afterwards).
- Deferred batches stay on disk,
  discarded by nobody.
  They are settled by the first of:
  the earlier helper (its start recovers the shared journal;
  it recovers its own journal
  before each batch it lands),
  or this service at its first start after no earlier socket listens (the earlier units are
  stopped or removed,
  or the machine restarted:
  they live under the runtime directory).
  While an earlier unit stays active and no earlier client connects,
  an open batch of the earlier helper stays open and the
  person's index shows the landed commit as reverted;
  this is the cost of not racing a helper the look cannot see coming.
- A journal argument is compared by the folder it names (device and inode),
  not by spelling:
  on this machine `/home` is a link
  to `/var/home`,
  so a helper started with `--journal /home/user/...` is the same service as the folder `/var/home/user/...`
  the service derives.
  A relative argument is resolved in the working directory of the inspected process (`/proc/<pid>/cwd`);
  an argument that
  cannot be resolved counts as "may be running";
  the same-spelling check stays as the first test.
  All of it runs inside the bounded scan.
- A command line is read to the limit plus one byte;
  one that is longer counts as "may be running",
  because the words past
  the limit were not looked at (an `exec -a` program name of 70 KiB hides the service words).
  A trailing NUL no longer makes an empty last word.
- Still uncovered:
  a wrapper of the earlier release run in the same worktree writes the earlier units again and starts their
  socket,
  which can start the earlier helper after the look and before the adoption ends;
  nothing the earlier release left
  behind can stop that,
  and the recovery's checks of the branch and the index make the second settling a repeat of the same
  step.

#### Changes after the fifth review

Recorded 2026-11-07,
after the fifth cross-vendor run (`~/temp/agent/issue-6261/sol-review-fix-5/review.txt`) found one defect
and closed the other three fixes (the process command line and the folder identity).

- Owner decision,
  2026-11-07,
  picked option "Stop the old socket",
  verbatim:
  "When the new program sets up its own service for a
  worktree,
  it stops and removes the old cli-git@<worktree>.socket unit of that same worktree (these are runtime units,
  gone at
  reboot anyway).
  Pros:
  half-done work is finished at once;
  no old units linger.
  Cons:
  the new program touches the old
  program's units;
  an old-release wrapper still used in that worktree would set its unit up again."
- Built in `service_provision.rs` (`stop_earlier_instance_socket`),
  run by `provision_with` after this release's socket has
  started,
  through the wrapper's own bus client (`systemd_dbus.rs`,
  new method `stop_unit_alone`).
  - The old and the new instance name are one text.
    Tag `monochromatic-git-policy-cli-v0.5.34`,
    `service_instance.rs` lines 47 to 56 hash the canonical path of the worktree's
    own Git directory with the 128-bit FNV-1a constants of lines 31 to 35;
    main's `instance_of` in the same file is the same body
    (the diff of the two bodies is only the `unwrap_or_else` spelling),
    and both callers pass the same Git directory
    (`service_route.rs` line 108 at the tag and on main).
    The old units are `cli-git@{instance}.socket` and `.service` (tag lines 69 to 75);
    main spells the prefix
    `SERVICE_UNIT.earlier`.
    So the old unit of a worktree is exactly `cli-git@{host.instance}.socket`,
    and another worktree's has another hash.
  - What it does:
    ask the manager for the state of the old service,
    then of the old socket;
    stop the socket only when the
    service is `inactive`,
    `failed` or not loaded.
    The socket file `<runtime>/cli-git/<instance>.sock` is never unlinked (changed after the sixth review,
    below):
    the manager
    leaves it behind after a stop (measured with throwaway units:
    file present after `StopUnit`),
    and an earlier-release wrapper
    can make its unit listen again at that path between any answer and an unlink.
  - The stop is sent in the job mode `ignore-dependencies`.
    Measured with throwaway units shaped like the old pair (`Requires=` and `After=` the socket):
    `systemctl --user stop` in
    the default mode stopped the running service with the socket;
    with `--job-mode=ignore-dependencies` the service stayed
    active.
    `systemd.unit(5)` says the same for `Requires=`:
    when the required unit "is explicitly stopped" the requiring unit is stopped
    too.
    So a plain stop would cut a running helper's batch;
    the state check skips a running helper,
    and the job mode keeps a helper
    that started between the check and the stop.
  - What is left alone:
    any other worktree's old units (nothing names them);
    the old templates `cli-git@.socket` and
    `cli-git@.service` in `$XDG_RUNTIME_DIR/systemd/user`,
    which the old program wrote as its only unit files
    (tag `service_provision.rs` lines 421 and 154) and which every worktree's instance is made from;
    the old state folders.
    There is no per-worktree unit file to remove:
    "remove" is the manager unloading the stopped instance (measured:
    the unit is
    gone from `list-units --all` after the stop);
    the socket file stays.
  - A running helper:
    not stopped,
    its socket kept,
    nothing logged;
    the helper exits when idle (`IDLE_EXIT`,
    three hours,
    tag
    `service_server.rs` line 70) and the next commit that sets the service up stops the socket.
  - Failure:
    any bus failure,
    a refusal,
    a timeout or a missing unit is logged as a warning naming the unit (a missing unit is
    not logged) and provisioning carries on;
    the commit never fails for this.
    The calls share one exchange's time (`EXCHANGE_WAIT`,
    20 s,
    the budget the reload and the start already have).
    After the sixth review `in_place` is false while a socket of the earlier release for the worktree listens (read from
    `/proc/net/unix`,
    only when the old socket file is there),
    so a skipped or failed stop is tried again by the next commit,
    and a stopped socket with a leftover file costs no bus trip.
  - Cost:
    measured after the sixth review,
    see `Cost of the stop while an earlier helper runs`.
- The review's defect:
  an old-release wrapper in the same worktree can write its units again and start its socket between the
  adoption's look and the end of the recovery,
  and the module already said so ("Still open").
  The reviewer asked for exclusion at the launch paths,
  "Merely stopping the currently listening sockets,
  or adding another
  scan,
  leaves the race."
  - Candidate guard:
    mask `cli-git@<instance>.socket`,
    which systemd enforces whatever the old program does.
    Documentation:
    `systemctl mask` links the unit to `/dev/null` so that "any start operation" fails
    (`systemctl(1)`,
    `mask`),
    in `~/.config/systemd/user` or with `--runtime` below `$XDG_RUNTIME_DIR`
    ("Similar restrictions apply for `--user` mode"),
    and fails when a file of that name already exists there.
    `systemd.unit(5)`,
    "Unit File Load Path":
    the user directory outranks `$XDG_RUNTIME_DIR/systemd/user`,
    and systemd "will first
    search for the literal unit name" and only then for the template,
    so an instance-named mask beats the template the old
    program writes;
    the old program writes only templates,
    so its `write_atomically` never touches a file of the instance's name.
    Measured with a throwaway template pair:
    `systemctl --user start` of the masked instance fails with "Unit ... is masked",
    of another instance succeeds,
    and the service of the masked instance fails with "Unit ...socket is masked".
  - What the old wrapper does when its start is refused (tag `monochromatic-git-policy-cli-v0.5.34`):
    `provision` returns
    `provision_with` (`service_provision.rs` line 457);
    the start at line 453 gets the masked refusal;
    line 435 returns it
    through `refusal` (line 385:
    `CouldNotStart`);
    `service_commit.rs` lines 386 to 388 turn it into
    `TransactionStop::Message`;
    `commit_command.rs` lines 105 to 108 print it and exit 1.
    There is no fallback to Git or to landing in process (the matches for "fallback" in `invocation/` are about finding the real
    Git).
    So a mask makes every commit of an old-release wrapper in that worktree fail,
    against "Commits should never fail."
    (it would not write the old journal,
    because it never reaches the service).
  - Other ways the old helper starts:
    none outside systemd.
    `service_main.rs` line 4 says it is "never started by the wrapper";
    the client's start watch (`service_client.rs` lines 136
    to 138 and 291) asks the manager to start the service unit,
    which a mask refuses too;
    `management_arguments.rs` line 429
    accepts `service status` only;
    a development run needs `--abstract` or `--path`.
  - Result:
    the second condition (the old wrapper neither fails a commit nor writes the journal) fails,
    so the mask was not built.
- What was done instead (superseded by the sixth review,
  below):
  the adoption was off in the unit this release writes,
  behind
  the start option `--adopt-earlier-journals`.
- Options for the owner:
  1.  Stop the socket,
      adoption off (built):
      no race,
      no failing commit,
      open batches stay open until the owner runs a recovery.
  2.  Stop the socket,
      adoption on (the owner's pick as worded;
      one unit file change,
      the option in `ExecStart`):
      finishes batches
      at once;
      the race stays for an old-release wrapper used in the same worktree,
      and the recovery's checks of the branch and
      the index make a second settling a repeat of the same step.
  3.  Stop and mask:
      the race is closed,
      and every commit by an old-release wrapper in that worktree fails with the refusal.
  4.  Adoption on demand:
      finish the batches when the person asks (`doctor` or `fix`) after saying no old wrapper runs.
      The owner picked this one (below).
  5.  Status quo before the stop:
      batches stay open until a reboot,
      because the old units live in the runtime directory.

#### Changes after the sixth review

Recorded 2026-11-07,
after the owner's pick and the sixth cross-vendor run
(`~/temp/agent/issue-6261/sol-review-socket-6/review.txt`) on the socket change.

- Owner decision,
  2026-11-07,
  picked option,
  verbatim:
  "Off,
  settle on request".
  The option text it was offered:
  "Automatic handover stays off.
  A new command (in `doctor` or `fix`) finishes leftover batches
  when you ask,
  after it checks that no old socket listens and no old helper runs.
  Pros:
  nothing races by itself;
  a leftover
  batch can still be finished.
  Cons:
  you have to run it;
  it is only safe when no old-release wrapper is in use in that worktree
  at that moment."
- Built:
  `git monochromatic-git-policy-cli service settle-earlier-batches` (also under the old command word,
  with its
  one-line notice like every management command,
  `invocation/entry.rs` lines 163 to 183),
  in `invocation/management_service_settle_earlier_batches.rs` and
  `landing/earlier_service_journal_adopted.rs` (`settle_on_request`).
  - Why under `service` and not under `fix`:
    `fix` applies policies and requires a file scope (`--all` or pathspecs);
    this
    works on the commit service's journals.
    `doctor` only reports,
    with documented exit codes,
    so it gets a read-only note instead.
  - It acts only on the current worktree:
    the per-instance earlier journal and the batches of that worktree in the shared
    earlier journal (found by the Git directory each intent records,
    as the recovery selects them).
  - Order:
    find the worktree and the state folder (`XDG_STATE_HOME`,
    else `$HOME/.local/state`);
    count the open batches (read
    only);
    with none,
    say so and stop;
    look (no earlier socket listening in `/proc/net/unix`,
    no process with the earlier
    helper's start arguments,
    the folder-identity and long-command-line rules,
    one ceiling);
    when the look holds,
    settle
    nothing and say why,
    how many batches,
    and what to do (stop the earlier units,
    or wait for the helper's three-hour idle
    exit);
    when it passes,
    write the condition to standard error,
    then run the same finish-only recovery the adoption used
    (`recover_instance_without_drops`:
    a batch that looks unlanded keeps its intent on disk),
    logging each batch.
  - The condition,
    said before it acts and in the help and the README:
    it is safe only when no wrapper of the earlier release
    is in use in that worktree at that moment,
    because such a wrapper sets its commit service up again and can start the
    earlier helper while the command runs.
  - Not on the commit path:
    no commit waits for it or fails for it.
    The look is bounded by `PROCESS_LOOK_CEILING`,
    the recovery by the lock ceiling.
  - `doctor` adds a read-only note on standard error with the count and the command;
    its status and standard output do not change.
- Review finding P1 (`service_main.rs`,
  the `--adopt-earlier-journals` option exposed the same unsafe adoption):
  the option and
  the automatic path are removed;
  the service has no start option that runs it (a binary test starts the service with that
  option and others and finds the batch untouched).
- Review finding P2 (`service_provision.rs`,
  the socket-state answer and an unlink are separate steps):
  the socket file is
  never unlinked,
  and `in_place` reads the table of listening sockets instead of the file.
  A control makes the file again after every answer and every stop,
  as an earlier-release wrapper would,
  and finds it whole.
- Found while running the command against a real wrapper:
  the look took this release's own service
  (`<program> monochromatic-git-policy-cli service run --journal <new folder> --instance <instance>`) for an earlier helper,
  because the mode words and the instance option match;
  a person who ran a commit minutes before would have been held back
  for the service's whole idle time.
  The mode words behind the current command word no longer count.
- Cost of the stop while an earlier helper runs (no `native:perf`:
  it times whole commits against a service it starts itself,
  where an earlier helper cannot be set up;
  the added work was timed alone,
  on the real user manager with throwaway units that
  carry the same call mix,
  75 samples after 5 warm-ups,
  two arms of the same code alternated as the control,
  2026-11-07):
  - connect and hello:
    median 0.10 ms.
  - the whole trip (connect,
    hello,
    two resets,
    the start,
    one pair of state questions,
    which is where it ends when the helper
    runs):
    median 1.13 ms and 1.12 ms for the two arms,
    95th percentile 1.32 ms and 1.24 ms;
    the arms differ by 0.01 ms in the
    median.
  - the read of `/proc/net/unix` (1277 lines on this machine) when the old socket file is there and nothing listens:
    median
    1.22 ms.
  - Against current main both cost nothing in the steady state (one `stat` of a file that is not there).
    While a helper runs,
    each commit pays about one millisecond more.
    After a stop the leftover socket file is never unlinked,
    so every commit in that worktree reads the table once,
    about the
    same millisecond,
    until the runtime directory is cleared at logout or reboot.
  - The command looks twice (once to decide whether to say the condition,
    once inside the settling that acts),
    each look bounded by
    `PROCESS_LOOK_CEILING`;
    when the second look holds,
    the condition has been said and nothing was settled.
- Not covered:
  the command against a real systemd and a real earlier helper (the machine's `cli-git@` units are live and were
  not touched);
  a helper in another PID namespace;
  the Windows build.

#### Changes after the seventh review

Recorded 2026-11-07,
after the seventh cross-vendor run (`~/temp/agent/issue-6261/sol-review-settle-7/review.txt`) on the settle
command found three defects,
fixed on `fix/git-policy-cli-settle-command-review-fixes`,
each with a red test committed first.

- The safety condition's write and flush errors were ignored,
  so a closed standard error let the command change the journal
  without the condition being delivered.
  A failure now refuses before any recovery starts (unit test with a failing writer;
  binary test with the reader of standard error gone:
  the batch stays open).
- A batch Git could not answer for was left out of the recovery's result,
  and the command noticed only an empty result.
  The
  settling now reconciles its result against the open batches it selected:
  each one that is neither reported nor gone is an
  `UnansweredBatch`,
  named with its journal in the output,
  left on disk,
  and the exit status is 1 (also for a batch left open).
- The hold kept only "a socket listens" and the remedy always named this worktree's units.
  The hold now says which one:
  this worktree's own socket,
  the single service's socket of the first releases (`cli-git/service.sock`),
  a helper started with
  this worktree's journal or mode words,
  a helper started with the shared journal,
  a helper whose journal folder cannot be
  resolved,
  or a look that could not be made.
  Remedies match:
  this worktree's units for the first and third;
  `cli-git.socket cli-git.service` for the single service,
  with
  the sentence that it serves every worktree;
  both sets for the unresolved one.
  The wait for the three-hour idle exit is offered only for a helper that runs with no socket listening,
  because a listening
  socket starts the helper again and does not stop by waiting.
  A claim left out on purpose:
  that the next commit with this release retires the single service.
  The code that retires it (`retire_earlier_service`) looks for its socket file under the new folder name
  (`LEGACY_SOCKET_RELATIVE`),
  where the first releases wrote `cli-git/service.sock`;
  that is a separate question to check
  before the message may promise it.

#### Earlier names after the rename audit

Recorded 2026-11-07,
after the coordinator confirmed a wrongly renamed constant:
`LEGACY_SOCKET_RELATIVE` named the earlier
program's file (`cli-git/service.sock`,
tag `monochromatic-git-policy-cli-v0.5.34`,
`service_instance.rs` line 26)
and held this
release's folder name.
The audit compares the tag with `main` for every name in the crate's non-test source (`package/git-policy/cli/src`,
leaving out
`*_tests.rs`,
`*test_support*.rs`,
`testing/`,
`binary/` and the word-merge corpus files).

Method and counts:

- `git grep -n -E '"[^"]*cli-git[^"]*"'` at the tag:
  313 lines naming `cli-git` in a string literal;
  on `main` the same search finds 74.
- `git diff -U0 monochromatic-git-policy-cli-v0.5.34 HEAD -- package/git-policy/cli/src` with the same exclusions,
  keeping the removed lines that are not comments and name `cli-git`:
  417 lines.
  By a pattern for constants,
  `join(...)`,
  ref names,
  unit lines,
  thread names and file names,
  122 of them are names of
  things;
  the rest are message text (the `cli-git:` prefix,
  `git cli-git ...` in help and hints).
  Each of the 122 was read next to its `main` counterpart.
- `git grep -h -o -E '\b(CLI_GIT|MONOCHROMATIC)_[A-Z_0-9]+'` at the tag and on `main`:
  42 names each,
  the same set (a `diff` of the sorted names is empty).
- The 12 pairs of `foundation/wrapper_names_old_and_new.rs` were followed to their uses of `.earlier`
  (`rg -n '\.earlier\b'`):
  `COMMAND_WORD` in `invocation/entry.rs` and `service/service_main.rs`,
  `SETTINGS_FILE` in
  `config/config_file.rs`,
  `SERVICE_UNIT` in `service/service_instance.rs`
  and `service/earlier_unix_sockets_that_listen.rs`,
  `DATA_FOLDER` in the two landing modules,
  and every folder and log pair through
  `foundation/kept_names_in_the_git_directories.rs`
  (moved in place by `repo_read/repository_facts.rs` on the first command in a repository).
  `PROGRAM` and `MESSAGE_PREFIX` hold their earlier half only for the table and for messages.
- `env!("CARGO_PKG_NAME")` in `invocation/real_git_candidate.rs` is the crate's name,
  which did not change
  (the `[[bin]]` did),
  so launcher records of earlier installs are still recognized.

Wrongly renamed (the name is the earlier program's artifact and held the new name),
each fixed with a red test first:

- `LEGACY_SOCKET_RELATIVE` in `service/service_instance.rs`:
  now `cli-git/service.sock`.
  `retire_earlier_service` and `in_place` look for the single service of the first releases there.
  With the old value they looked in a folder of this release that nothing writes,
  so that service was found only through its unit
  files,
  `ask_to_retire` was sent to a path nothing listens on,
  and a socket file at its real path did not keep the installation
  from counting as in place.
- `RECORD_MAGIC`,
  `SKIPPED_MAGIC` and `LOST_MAGIC` in `push/push_failure_notice.rs`:
  the first lines of the records of failed
  pushes.
  The folder of the records is moved in place,
  so earlier records are there with the earlier first line,
  and `parse_record` took
  them for the finished text of a still earlier release and showed them raw,
  to any worktree.
  Both spellings are read now;
  records are written in the current one.
- `CLAIM_HEADER` in `lock/single_owner_claim.rs`:
  the first line of the claim files of `migrate-to-subvolume` and
  `convert-worktree-to-subvolume`,
  whose file names keep the earlier prefix (`cli-git-convert.owner.N`).
  A claim of the earlier release,
  released or from a dead owner,
  was "not a claim file this command wrote"
  and stopped the command.
  Both spellings are read now.

Correctly renamed (the name is this release's own artifact;
the earlier counterpart is handled where stated):

- The units,
  the template names,
  the socket folder `%t/monochromatic-git-policy-cli/`,
  the state folder
  `monochromatic-git-policy-cli/service/<instance>` and `SyslogIdentifier`:
  this release's service.
  The earlier ones are named through `SERVICE_UNIT.earlier` (`earlier_*` helpers in `service/service_instance.rs`)
  and the
  adoption journals (`earlier_journals_in_state`),
  and the single service of the first releases through the kept `cli-git.socket`
  and `cli-git.service` names and the fixed `LEGACY_SOCKET_RELATIVE`.
- The command word,
  the settings file,
  the data folders and logs in the Git directories,
  the merge
  and worktree-copy record
  folders:
  pairs,
  moved in place or read under the earlier name.
- Folders and files under the data folder (`discarded`,
  `notices`,
  `orphaned-notices`,
  `work`,
  `removal-snapshots`,
  `standards-review-shadow.md`,
  `shadow/`):
  they move with the folder.
- Scratch and cache names (`REPOSITORY_SCRATCH_NAME`,
  now paired with its earlier name,
  see the `REPOSITORY_SCRATCH` pair under `Earlier names after the rename audit`;
  the state
  and cache folders of `foundation/scratch_location.rs` and
  `invocation/git_version_cache.rs`,
  the cache header,
  `SCRATCH_PREFIX`,
  `cli-git-service-<id>` scratch names,
  the journal
  identifiers and log prefixes,
  `LOG_COMMAND` and `RESTART_COMMAND`,
  thread and tag names):
  this release's own,
  rebuilt on use.
- The messages' `cli-git:` prefix and the `git cli-git ...` spellings in help and hints.

Correctly kept as the earlier name (74 literal lines on `main`;
the artifact is shared with the earlier release
or is still written
under that name,
so a reader of either release finds it):

- The ref names `refs/cli-git/kept/`,
  `refs/cli-git/trash`,
  `refs/cli-git/auto-push/fetch-`
  and `refs/worktree/cli-git-service/`.
- The index-lock marker `cli-git-service ` and the `cli-git-private-copy v1` marker.
- The sibling and stage names `.cli-git-converting`,
  `.cli-git-migrate-*`,
  `.cli-git-worktree-copy-`,
  `.cli-git-added-`,
  `.cli-git-direct-fix-`,
  `.cli-git-marker`,
  `.cli-git-new`,
  `.cli-git-claim.tmp.`,
  and the `cli-git-convert` journal,
  lock and
  claim names with their `cli-git-convert v1` header.
- `cli-git-transactions` and `cli-git-transaction`,
  `.claude/cli-git-trash`,
  `/dev/null/cli-git-no-hooks`,
  `--cli-git-keep-going`,
  `cli-git-lfs-pre-push`,
  the legacy `cli-git.config.mjs` and `cli-git.config.ts`,
  and the 42 environment variable
  and journal
  field names.

Gaps found,
not rename errors,
not changed here:

- `invocation/real_git_record.rs` reads the recorded Git from this release's service template only;
  an installation that has not
  yet written it does not use the earlier template as a fallback.
- The state and cache folders of the earlier release (`<state>/cli-git`,
  `<cache>/cli-git`) are not swept or moved;
  they hold
  nothing the wrapper reads,
  and the state folder also holds the earlier journals,
  which are read where they are.

Decided by the owner,
2026-11-07,
verbatim:
"Pair the names (Recommended)".
The option text it was offered:
add the old scratch name to the list of old and new names,
so the new program cleans it exactly as
it cleans its own scratch,
with one set of rules and nothing deleted that those rules would keep.
Built:
the pair `REPOSITORY_SCRATCH` (`cli-git-scratch` and `monochromatic-git-policy-cli-scratch`) in
`foundation/wrapper_names_old_and_new.rs`,
`REPOSITORY_SCRATCH_NAME` reads its current half,
and `prepare_repository_scratch` also
sweeps the earlier folder (`sweep_earlier_repository_scratch`) by the rules of its own:
the folder must be a real directory of
this user (a looser mode is tightened;
a link or another account's folder is left alone and logged),
entries older than
`SCRATCH_STALE_AFTER` go,
entries dated in the future and fresh ones stay,
the folder itself stays,
and nothing is made under the
earlier name.
A refusal of the earlier folder never fails the command that asked for scratch.
The per-user scratch of the earlier release (`<state>/cli-git/scratch`) is not part of this pick and is still not swept.
- The TypeScript packages the plan lists (`package/git-executable/core`
  and the others) were outside this audit of the crate.

### Where the plan was wrong or incomplete

- The plan's units section says a running old helper "keeps landing commits until replaced".
  A new wrapper never connects to it:
  the new socket is `$XDG_RUNTIME_DIR/monochromatic-git-policy-cli/<instance>.sock`.
  Both helpers can exist for one worktree until the old one idles out;
  nothing stops or removes the old units or the old
  template unit files (older installs in other worktrees need them),
  and the new journal folder
  `monochromatic-git-policy-cli/service/<instance>` under the state directory starts empty.
  The batches the old helper had accepted and left open stay in `cli-git/service/<instance>` (and,
  from the first releases,
  `cli-git/service`),
  so they are adopted,
  see "Open batches of the old helper".
- A socket path holds 107 bytes,
  and the program folder name is 21 bytes longer than `cli-git`:
  a runtime directory longer than 41 bytes no longer fits,
  so the service fixtures and the e2e scenarios use shorter runtime
  directory names.
- Renaming this repository's own settings file in step 1 would leave a worktree still on the old lock,
  whose installed wrapper
  is the old program that reads only `cli-git.config.jsonc`,
  with no configuration:
  no policy would run on any commit until the
  lock moves.
  Recorded 2026-11-04T23:00:06-05:00:
  the root rename moved to step 3.
  The branch renamed it in `f781971973` and put it back in a later commit;
  the owner's "rename them."
  stands,
  only the
  timing of this one file changes,
  and the new program reads the earlier name with a notice meanwhile.
  Controls that read the repository's own file find it with `settings_file_of` (current name first,
  then the earlier one),
  and
  the gate image copies whichever exists (`bin/native-test-settings-file.ts`).
  The `catalog-specifiers` workflow chooses the program,
  word and settings name of the pinned release.
- While worktrees on the old lock and worktrees on the new lock share one repository (the common Git directory),
  the new
  wrapper renames the shared data folder and logs and the old wrapper then makes them again on its next save:
  both names exist,
  every new-wrapper process says so once,
  and the audit log is split in two files until the last old install
  is gone.
  No data is lost;
  the plan's "Older installs" section did not say it.
- Tests:
  a crash between the look and the rename is tested as a split of the two steps (`look`,
  then a fresh run),
  not with
  an `strace` injected fault,
  because nothing is written before the rename,
  so no half state exists to inject into.
  The units under their new names are exercised with the fixture service and the fake service manager only;
  a real `systemd`
  start of them is not tested.
- The link folder `.cache/cli-git-bin` is hard-coded in `package/mpm-shadow/cli/src/toolchain/scope.rs` and in
  `.config/mise/conf.d/20-env.toml`,
  so its rename belongs to step 3 with the lock.

### Not renamed, on purpose

`--cli-git-keep-going`;
the Git configuration keys `hook.cli-git-lfs-pre-push.enabled` and `user.cli-git-test`;
the references `refs/cli-git/kept/`,
`refs/cli-git/trash`,
`refs/cli-git/auto-push/` and `refs/worktree/cli-git-service/`
(references are Git's retention state;
a rename would need packed-references handling);
the temporary file and folder prefixes `.cli-git-*` and `cli-git-*` (including `.cli-git-worktree-copy-`,
whose names the
copy records hold),
the index lock marker `cli-git-service` (a reader of an older marker must keep working),
the conversion and
migration journals,
`.claude/cli-git-trash`,
`cli-git-transactions` (an obsolete directory the startup step only reports),
thread names,
the journal field `CLI_GIT_AT_MS`,
the test variables `CLI_GIT_TEST_ONLY_*`,
and the mutation and e2e tool
prefixes `cli-git-mutpar` and `localhost/cli-git-concurrent-e2e`.

### Edits that wait for the release (step 3)

- `git mv cli-git.config.jsonc monochromatic-git-policy-cli.config.jsonc` at the repository root,
  in the same commit as the lock,
  and update the first comment line of the file;
  nothing else needs an edit (the controls and the gate image find the file).

- `toolchain.plan.jsonc` and `toolchain.lock.json`,
  entry `cargo:monochromatic-git-policy-cli`:
  `bins` becomes `["monochromatic-git-policy-cli"]`,
  `options.postinstall.link.sourceBin` the same,
  and the comment on the `cargo:monochromatic-git-policy-cli` entry names the new program.
  `package/git-policy/cli/bin/native-release-bins.unit.test.ts` stops accepting the earlier executable from then on
  (remove `earlierExecutables`).
- The root `mise.toml` task `changeset:version`:
  `monochromatic-git-policy-cli monochromatic-git-policy-cli fix --all --policy
  mono/dependent-version-bump`,
  and its comment;
  then `npm-release.yml` drops the `cli-git-native` link it makes.
- With the link folder (owner:
  rename it):
  `.cache/cli-git-bin` becomes `.cache/monochromatic-git-policy-cli-bin` in the plan,
  the lock,
  `.config/mise/conf.d/20-env.toml`,
  `package/mpm-shadow/cli/src/toolchain/scope.rs` and
  `old_copy_places.rs` with their tests,
  and the READMEs that name it.

## Step 3 as built

Recorded 2026-11-07T09:01:20-05:00,
on `main`,
after the merge `71498b658f` (2026-11-07T09:00:54-05:00) of the branch
`chore/git-policy-cli-rename-step3`.
The locked release is `monochromatic-git-policy-cli` 0.6.0 (crates.io 2026-11-07T13:10:36Z;
lock and install since `c048f3a566`).

### What changed

- The settings file:
  `git mv cli-git.config.jsonc monochromatic-git-policy-cli.config.jsonc`,
  content byte for byte
  (`c0b4812639`).
  The plan said to update the first comment line too,
  and that line said the file keeps its earlier name until the
  lock carries the release,
  which the rename made false;
  it changed in its own commit (`4935573a7b`).
  No generator manages the file (`file-enforcer.config.jsonc` has no rule for it),
  so no `file-enforcer write` was needed.
- The link folder:
  `.cache/cli-git-bin` is `.cache/monochromatic-git-policy-cli-bin` in `toolchain.plan.jsonc` (`linkDir` and
  its comment) and in `.config/mise/conf.d/20-env.toml` (`_.path`;
  hand-written,
  nothing generates it) (`eab04cdf95`),
  and in the default `link_dir` of the repository scope in `package/mpm-shadow/cli/src/toolchain/scope.rs`
  with its layout test,
  a doc comment and the README (`c4a1204ec9`).
- The lock:
  `mise run toolchain:update` wrote `toolchain.lock.json`,
  zero tools moved,
  and the only changes are the
  `linkDir` of `cargo:monochromatic-git-policy-cli` and the digest (`f44c638269`).
  `mise run toolchain:outdated` before it reported zero tools to move.
- The leftovers:
  the root task `changeset:version` runs `monochromatic-git-policy-cli monochromatic-git-policy-cli fix ...`,
  and `npm-release.yml` no longer links `cli-git-native` (`171f4046a1`);
  `native-release-bins.unit.test.ts` lost `earlierExecutables`,
  so a lock or release check that lists `cli-git-native`
  is a disagreement (`1abae36683`).
- Prose that named the file or the folder:
  the root README,
  the READMEs of `git-policy/cli`,
  `forbidden-strings/cli`,
  `config/lfs-r2-worker` and `mpm-shadow/cli`,
  the comment in `catalog-specifiers.yml`,
  two TypeScript comments and
  `doc/runbook/cli-git-global-install.md` (`2a59f6f09d`).

### Checked

- `mise run check:files` clean;
  `//package/git-policy/cli:lint:types` and `:lint:oxlint` clean;
  `node package/git-policy/cli/bin/native-release-bins.unit.test.ts` passes with the changed expectations.
- `mise run //package/mpm-shadow/cli:test`:
  every test passes,
  including the layout test of the new default folder.
- `mise run toolchain:install` in the main checkout:
  nothing to install,
  the postinstall link `git` is made under
  `.cache/monochromatic-git-policy-cli-bin` and points at `.cache/toolchain-launchers/monochromatic-git-policy-cli`.
  In a fresh `mise exec` environment `command -v git` is `.cache/monochromatic-git-policy-cli-bin/git`,
  and `git --version` ends with `monochromatic-git-policy-cli 0.6.0, shadowing git at /usr/bin/git`.
- The old folder `.cache/cli-git-bin` of the main checkout was empty before the install (its link and those of
  `mpm-shadow-bin` and `cargo-shadow-bin` were missing at 08:42;
  whoever cleared them is not known,
  and the install made
  them again);
  it is removed.
  A worktree that still has the old link is cleaned by its own `toolchain:install`,
  which retires a postinstall link
  the lock no longer declares (measured in the step 3 worktree).
- The commit that records this section is the first with the new settings file read from the main checkout:
  its output carries no settings-file notice.
  The merge `71498b658f` still printed it,
  because the wrapper reads the file before the merge lands.

### Kept under the earlier name on purpose

- `package/mpm-shadow/cli` tests `old_copies_tests.rs`,
  `across_roots_tests.rs`,
  `tests/old_copies.rs` and
  `launcher_tests.rs`,
  and `old_copy_places_tests.rs`:
  they model installs of the releases before the rename
  (`cli-git-native` copies and `.cache/cli-git-bin/git` links to them);
  a checkout made before this step still has such a
  folder,
  and `old_copy_places.rs` finds link folders by the `-bin` suffix,
  so those fixtures stay true.
- `EARLIER_SETTINGS_FILE` in `package/git-policy/cli/bin/native-test-settings-file.ts`,
  the `cp` of the settings file for a pinned release before the rename in `catalog-specifiers.yml`,
  and the word-merge corpus files under `package/git-policy/cli/src/native/merge/word_merge_corpus/`
  (recorded texts and their outputs):
  behaviour or data that older releases and older checkouts still need.
- `package/git-executable/core` keeps `cli-git-native` in its list of names never to pick as the real Git (older installs).
- Historical records in `doc/` (research,
  handover,
  planning of earlier dates) keep the old link folder name as written (WR6).
- The data names in `.git` and the wrapper's Rust code:
  another agent's fix
  (`fix/git-policy-cli-fold-earlier-names`) folds the earlier data names into the new ones.

## Folding the earlier names when both exist

Recorded 2026-11-07T09:05:57-05:00 on the branch `fix/git-policy-cli-fold-earlier-names` (unmerged,
unreleased).
This replaces "the old one is left untouched and reported" in "The settings file and the data the wrapper keeps" for the
data folders and logs:
the owner's rules "every name of a dirent must be understandable to a monkey" and "never discard user
data" both rule out telling the person to merge folders by hand,
and six lines on every command are noise.

### What was measured

2026-11-07T08:37:00-05:00,
main Git directory of this repository:
six names existed under both spellings (the data folder
`cli-git`,
`cli-git-push-trigger.jsonl`,
`cli-git-worktree-copy`,
`cli-git-audit.jsonl`,
`cli-git-commit-timing.jsonl`,
`cli-git-worktree-add-timing.jsonl`),
so every command printed six lines.
The current names were born on 2026-11-03 and 2026-11-04 (development builds of the renamed wrapper);
the installed 0.5.34 kept
writing the earlier names until 2026-11-07T08:24:40-05:00;
0.6.0 was installed at about 08:25.
Linked worktrees (339 `.git/worktrees/*`) hold only the earlier spelling of the data folder (144) and of the merge records (16),
never both,
so the existing in-place rename covers them;
the fold runs there too because it is part of the same settling.

### What the fold does

- Logs:
  `foundation/earlier_log_lines_added_to_current_log.rs`.
  The earlier log is moved aside by `rename_noreplace` to `<current>.earlier-lines-to-add`,
  its bytes are appended to the current
  log in one write,
  the append is synced,
  then the moved log and the journal are removed.
  The journal `<current>.earlier-lines-journal` is also the lock (`flock`,
  released by the kernel when the process dies):
  it
  records where the current log ended and which bytes of the moved log are being added,
  so a restart looks for exactly those
  bytes (in the current log from that place and in its numbered older copies) and appends them only when absent.
  The journal only grows:
  each record is a new last line and the last whole line that is a record wins;
  a journal left by an
  earlier fold is emptied before the move that starts a new one (the earlier code resumed such a line against the new moved
  log,
  which skipped the bytes before it).
  A line a program with the moved log still open adds later is folded in a further round.
  Recorded 2026-11-07T10:42:51-05:00 on the branch `fix/git-policy-cli-fold-keeps-late-lines`:
  the moved log is removed only
  after `foundation/file_open_for_write_by_any_process.rs` found no process holding it open for writing,
  and the size is
  checked once more after that probe.
  The probe takes a read lease on the moved log (`fcntl` `F_SETLEASE`,
  `F_RDLCK`) and gives it back at once:
  the kernel grants
  it only while no process has the file open for writing,
  in any PID namespace and for any user,
  and no lock file is added.
  Its signal is set to `SIGURG` first because a lease break's default signal ends the process.
  A `/proc/<pid>/fd` scan was rejected because it cannot see other users' processes or other namespaces.
  While a writer holds the moved log,
  the fold appends a last journal line that says every byte before a place is already in
  the current log,
  leaves the moved log and the journal,
  and ends silently;
  the next command resumes from that line and adds
  only the growth.
  Any other refusal of the lease (`EACCES`,
  `EINVAL` on a file system without leases,
  `/proc/sys/fs/leases-enable` set to 0)
  becomes the existing one-line failure,
  and everything stays in place.
  One command runs at most `ROUNDS_PER_COMMAND` (8) rounds;
  the number is derived in the constant's documentation from the
  measured gap between records (4 records in one second at most) and the measured cost of one round (43 ms).
- Not done,
  and why:
  lines are not interleaved by time stamp.
  Other programs of this release append to the current log with no lock,
  so replacing or rewriting it would drop every line
  appended between the read and the replace.
  The earlier log's lines therefore end the current log,
  in the earlier log's order.
  Every line keeps its bytes;
  the only added byte is a newline that ends a last line that had none.
  The audit log's older lines carry `timeMillis` and no `time`,
  so a merge by time would also have needed two readers.
- Folders:
  `foundation/earlier_folder_entries_moved_into_current_folder.rs`.
  Every entry moves by `rename_noreplace`;
  a folder in both is merged one level at a time with a work stack;
  a file or link in
  both keeps the earlier one as `<name>.from-earlier-release`;
  when that name is taken too the entry stays and one line says so.
  Emptied earlier folders are removed deepest first.
- When:
  `foundation/earlier_names_folded_into_current.rs`,
  from the settling of the Git directories,
  only for names that exist
  twice,
  and only when no process has the start arguments of the earlier commit service of the Git directory's worktree
  (`earlier_service_process_hold` in `landing/earlier_service_process_look.rs`:
  the process half of the existing look).
  The sockets of the earlier units are not looked at:
  they stay active for good (27 `cli-git@*` units were listed on this host
  at 2026-11-07) and write nothing.
- Said:
  one line per process when the fold waits (with the `systemctl --user stop` command that lets it proceed) and one when a
  name could not be folded;
  nothing after a fold.

### Limits

- The look sees the earlier service of the one worktree whose Git directory is being settled,
  in this user's PID namespace.
  Short-lived programs of the earlier release in other checkouts (a checkout whose lock file still names an older release runs
  that release's wrapper) are not seen by the look,
  but the lease probe sees them:
  a line such a program appends to the moved
  log is no longer lost,
  because the moved log stays until the program has closed it (the earlier text of this limit said such a
  line "is lost" in the instant between the last size check and the removal).
  A line it appends after the fold removed the moved log goes to a new earlier-named log that the next command folds again.
- While a writer keeps the moved log open,
  every command pays the look (about 10 ms) and the probe (an open and three `fcntl`
  calls) until that program closes the log;
  a program that never closes it leaves the moved log and journal for good,
  each
  holding bytes that are already in the current log.
- A file system without leases (some network file systems) or a Git directory owned by another user makes the probe fail,
  so the
  fold does not finish there:
  one line per process says so and every byte stays in place (the earlier fold needed only `flock`).
  `rustix` has no lease call,
  so the probe uses `libc::fcntl`.
- A write that a kill cut short can leave the first lines of a block in the current log;
  the restart adds the whole block again,
  so those lines appear twice and none is lost.
- Folding adds the earlier commit-timing log (250,668 bytes here) to a current one of 987,465 bytes,
  over the 1 MiB rotation
  limit,
  so the next append moves it aside and replaces the existing `.1` (the one-generation policy of `discard_audit.rs`).
- Numbered older copies of an earlier rotating log whose base name no longer exists are not revisited.

### Measured cost of the fold

Measured 2026-11-07 on copies of this repository's real data names in `~/temp/agent/fold-measure` (the release build of the
branch,
`status --short`,
seven alternating trials per arm,
each in a fresh repository;
the real `.git` was only read):
a repository with no names twice takes a median of 4.7 ms;
the first command that folds the six pairs (4,473 files and 2,736
log lines,
all kept) a median of 143.7 ms,
once per Git directory;
a command that finds a name twice while an earlier program
runs (the look is made,
nothing is folded) a median of 15.1 ms,
so the look adds about 10 ms to each command until that program
stops.
Measured 2026-11-07 for the lease probe with the control binary built at `ead9d7463c` (the fold before the probe,
copied to
`~/temp/agent/fold-control-ead9d7463c`) and the fixed one (`~/temp/agent/fold-fixed-dda3fe3332`),
same method,
nine alternating
trials per arm,
medians,
and a second series of the control binary as the same-binary spread.
One log pair only (the commit timing log,
1,112 plus 257 records):
nothing to fold 4.5 ms (control),
4.6 ms (fixed),
4.5 ms
(control again);
fold 47.8 ms (control),
49.8 ms (fixed),
48.1 ms (control again,
range 44.3 to 53.1 ms),
so the fixed fold
is 2.0 ms above the control with a same-binary spread of 8.8 ms.
All six pairs:
nothing to fold 5.1 ms,
4.9 ms and 5.1 ms;
fold 1,177.1 ms (control;
range 449.4 to 1,865.8 ms),
480.2 ms
(fixed) and 354.7 ms (control again;
range 323.9 to 682.3 ms):
the host's file-system syncs vary by more between runs of one
binary than the binaries differ,
so no cost of the probe shows at that size.
A moved log with a writer holding it open (a `sleep` with the earlier log open for appending):
the first command 41.6 ms for one
pair and 476.7 ms for six,
and each later command a median of 13.5 ms (one pair) and 14.0 ms (six),
which is the look's 10 ms plus
the probe,
against 4.9 ms with nothing to fold.
The commit path is not touched:
the fold runs on the settling of the Git directory,
once per Git directory plus the recurring
cost above,
far below "commits vs native git should not take 5s longer".
A first measurement series in a host-wide I/O stall (load average above 30) showed single commands of 10.5 s and 120.7 s in the
writer arms (also 19.0 s for the control binary and 5.6 s in a fold arm);
50 later runs of that arm at load 14 to 24 showed none above
1.3 s,
so the stall is the host's and was not reproduced.
No marker throttles the look;
a marker with a short life beside the names would,
and it is the next step if that 10 ms matters.
The fold runs on the command's own path because a forwarded command replaces the process,
so a thread would die with it.
