# Entry, forwarding and service-lifecycle route findings on `main`

Status:
audit,
2026-11-09.
Audited revision:
`main` at `20acf6f904`,
in the worktree
`/var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/pi-entry-and-service-lifecycle-route-findings-on-main`.
No production code was changed and no test was committed.
The scratch scripts and fixtures are in `${HOME}/temp/agent/pi-entry-route-audit`
and are named where each result is quoted.
Scope:
how the installed program decides it stands in for Git
(`src/native/invocation/`),
finds the real Git and forwards to it
(`invocation/real_git.rs`,
`invocation/real_git_candidate.rs`,
`invocation/forwarding.rs`),
the guards against starting itself
(`invocation/started_as_git.rs`,
`invocation/real_git_candidate.rs`,
`service/service_main.rs`),
how a child is started,
watched and stopped
(`process/`,
`push/start_retry.rs`,
`push/bounded_push.rs`),
and the resident service's start,
readiness,
upgrade,
shutdown and restart
(`service/service_main.rs`,
`service/service_provision*.rs`,
`service/service_client*.rs`,
`service/service_server*.rs`).
Linux only,
and the copy route only where forwarding and the child's end are concerned;
the ignored-state copy itself is
[`cli-git-copy-route-findings-on-main.md`](cli-git-copy-route-findings-on-main.md),
and committing through the service is
[`cli-git-commit-and-landing-route-findings-on-main.md`](cli-git-commit-and-landing-route-findings-on-main.md).
This page does not repeat either.

Purpose:
the owner's rules for this program are
"Commits should never fail."
(`package/git-policy/cli/SPEC.md:2742`),
"never discard user data"
(`SPEC.md:2744`),
"nothing should ever wait forever or indefinitely"
(`SPEC.md:2734`),
"Use as few locks as possible."
and
"We should never invoke ourselves recursively."
(`SPEC.md:4620`).
The handover's "Open review findings"
(`doc/handover/cli-git-rust-implementation.md:7488`)
names the copy,
commit and push routes as audited or in progress
and says nothing about this one.
This page measures the entry,
forwarding and lifecycle route against those rules,
finding by finding,
with the sequence that produces each wrong result.

## What the route is

One invocation of the program,
in order:

1.  `run_process` asks the runtime probe,
    switches hooks off for every later Git start,
    reads its own executable and the current directory,
    and stops at once when
    `refusal_when_started_as_git` finds the marker
    (`src/native/invocation/entry.rs:336-345`,
    `started_as_git.rs:391-400`).
2.  `plan_invocation` stops when the forwarding target names this executable
    (`entry.rs:153-158`),
    strips the wrapper's own controls,
    answers the management namespace itself,
    else resolves the real Git
    (`real_git.rs:489-517`,
    with the fallback to the unit's `--git`
    from `real_git_resolution.rs`).
3.  The action is planned:
    a plain forward
    that replaces the process with Git
    (`entry_forward_action_planning.rs`,
    `Action::Forward`),
    the announced query that runs Git as a child and prints Git's version line after it
    (`entry.rs:388-402`),
    the coordinated forward for an index writer
    (`entry_forward_action_planning.rs:150-167`),
    the settled forward for the copy route
    (`entry_forward_action_planning.rs:141-149`),
    or a stop.
4.  `run_process` performs it
    (`entry.rs:358-441`).
    A plain forward never returns;
    the others run Git as a child and return Git's status
    (`worktree_copy_forward.rs:202-242`).
5.  A commit provisions the service,
    then reaches it
    (`landing/service_commit.rs:314-322`):
    unit texts written when they differ,
    the manager reloaded,
    a failed state reset,
    the worktree's socket started
    (`service_provision.rs:187-232`).
6.  The service starts from the manager's socket,
    settles the batches an earlier instance left,
    accepts connections as tasks,
    lands batches,
    and ends on a signal with its own bounded shutdown
    (`service_main.rs:254-356`,
    `service_server.rs:249-478`,
    `service_server_shutdown_after_a_stop.rs:62-82`).

## Findings ranked by what they risk losing

The first finding risks the host:
one wrapper per level,
each waiting on the next,
with no end.
The second makes every commit in one worktree fail
with advice that cannot work.
The third reports a different exit status than Git for a command whose
output reader went away,
which changes a script's decision.
The last two lose a leftover unit
and ten seconds,
never work.

### 1. A `git` on `PATH` that starts the wrapper as a child makes the wrapper start itself without end

`src/native/invocation/real_git_candidate.rs:506-552`:
a candidate that is an executable file,
is not this executable,
is not a copy of it,
has no launch record naming this wrapper,
and is not a script carrying the TypeScript wrapper's marker text,
is `CandidateKind::RealGit`.
`src/native/invocation/real_git.rs:489-517`
takes the first `RealGit` it finds.
The guard that would stop the loop is
`src/native/invocation/started_as_git.rs:275-292`:

```rust monochromatic-lint-skip-rustdoc
//! Quoted from src/native/invocation/started_as_git.rs:275-279.
pub fn started_as_git(marker: &StartMarker, facts: &ProcessFacts<'_>) -> bool {
    let candidate: u32 = if marker.mode == StartMode::Replace { facts.own_pid } else { facts.parent_pid };
    if candidate != marker.starter_pid {
        return false;
    }
```

In `Child` mode the marker names the wrapper that started Git,
and the check compares it with **this process's parent only**.
A script between the two adds one process,
so the arriving wrapper's parent is the script
and the marker matches nothing.

`SPEC.md:4703-4711` records exactly this as not covered,
and promises a mitigation:

```text
- Not covered:
  a user script named `git` that starts the wrapper as a child,
  with no launch record and no marker
  string.
  The parent of the arriving wrapper is the script,
  not the starter;
  the message of a wrapper that cannot find
  Git lists every candidate,
  which shows the script as the entry to remove.
```

The mitigation does not apply:
the wrapper **does** find Git (the script),
so no candidate list and no message is ever printed,
which is what the reproduction shows.

The sequence,
with `PATH` holding the wrapper's directory and a directory whose `git` is a script:

```sh
#!/bin/sh
/usr/local/bin/mono-git "$@"     # no exec
```

1.  The first wrapper searches `PATH`,
    skips itself,
    and selects the script as real Git (`real_git.rs:489-517`).
2.  It replaces itself with the script (`forwarding.rs:230-262`),
    so the script's shell keeps the wrapper's process number and its
    `replace` marker.
3.  The script starts the wrapper as a **child**.
    That wrapper's parent is the shell,
    its marker names the older process number in `replace` mode,
    so `started_as_git` compares the marker with its own number and says no.
4.  It searches `PATH` again,
    finds the same script,
    and step 2 repeats.

The wrong result:
no process ever exits,
each level waits on the next,
the chain grows by one wrapper per level until the host or the scope refuses to
start processes,
and nothing is printed.
At that point every other program on the machine that needs a new process also
fails.
No user data is touched;
what is lost is the machine's ability to start a process,
and both owner rules that this route exists for are broken:
"We should never invoke ourselves recursively."
and
"nothing should ever wait forever or indefinitely".

Reproduced,
in "Reproductions" A,
with the `exec` spelling of the same script as the positive control:
the `exec` form stops at once with the refusal text,
the child form grows to the scope's process cap and is still running.

Suggested fix,
a suggestion only:
make the marker carry a start count,
one field more,
incremented by `mark_real_git_start`,
and stop when it passes a small bound with the refusal text that already
exists,
which catches any number of process starts between two wrappers;
or,
cheaper but weaker,
also stop when the process named by a `child` marker is an ancestor of this one,
compared through `/proc/<pid>/stat` and the birth identities the module
already reads.
Either way `SPEC.md:4703-4711` needs correcting:
its promised message never appears.

### 2. A service unit in `failed` is never repaired, so every commit in that worktree fails and the advice cannot work

`src/native/service/service_provision.rs:344-350`:

```rust monochromatic-lint-skip-rustdoc
//! Quoted from src/native/service/service_provision.rs:344-350.
pub fn provision(host: &ProvisionHost) -> Result<(), ClientFailure> {
    if in_place(host) {
        // Units set up before this release have no note of their worktree yet.
        note_worktree(host.runtime_dir.as_path(), host.instance.as_str(), host.git_dir.as_path());
        return Ok(());
    }
```

`in_place` tests three things (`service_provision.rs:140-170`):
each unit file holds exactly the text this wrapper would write,
no file of the earlier release's names is left,
and the worktree's socket file exists.
A unit in `failed` state passes all three:
its text is current and systemd keeps the socket file
(`RemoveOnStop=` is unset in the socket template).

The repair that would fix it is only reached when `in_place` is false,
in `ask_manager` (`service_provision.rs:236-247`):

```rust
//! Quoted from src/native/service/service_provision.rs:240-246.
    if known {
        // A start that failed under an old unit leaves the service failed and its start counter full, and the socket
        //  failed with it; the new unit would be refused as "repeated too quickly" until the interval passed.
        for unit in [service_unit_instance(instance), socket_unit_instance(instance)] {
            let _reset = manager.reset_failed(unit.as_str());
        }
    }
    return manager.start(socket_unit_instance(instance).as_str());
```

The sequence:

1.  The worktree's service unit ends in `failed`:
    its start is refused often enough for the manager's start-rate limit
    (`Restart=on-failure` in the service template,
    `service_units/monochromatic-git-policy-cli@.service`),
    or its program refuses to start
    (`EXIT_REFUSED` at `service_main.rs:121`,
    `EXIT_NOT_LISTENING` at `:124`).
2.  The person runs `git commit`.
    `provision` finds the texts current and the socket file present,
    so it returns without a reload,
    without `reset_failed`,
    and without a start (`service_provision.rs:344-350`).
3.  The client connects and nothing answers.
    It asks the manager every second
    (`MANAGER_ASK_EVERY`,
    `service_client_settings_and_watches.rs:136`)
    and gets `failed` twice,
    which ends the wait
    (`service_client_reach_service.rs:121-126`) with `ClientFailure::DidNotStart`.
4.  The message is
    "The commit service did not start,
    so this commit was not made.
    Its log shows why:
    ...
    Run the commit again after fixing that."
    (`landing/service_messages.rs:172-175`).
    Running it again reproduces step 1 to 3:
    nothing on this path can clear the failed state.

The wrong result:
every commit in that worktree fails in about two seconds
with an instruction that cannot succeed,
until a person runs `systemctl --user reset-failed` or logs in again.
That is a broken
"Commits should never fail."
The route already contains the repair;
it is unreachable whenever the unit text is unchanged,
which is every case except an upgrade.

Read from the code only,
in "Reproductions" B,
because reproducing it needs a real user manager and the installed units,
which this audit must not touch.

Suggested fix,
a suggestion only:
make `in_place` also false when either unit is not at rest
(one `ActiveState` question to the manager,
or the two reset calls unconditionally on the provisioning path the first
commit of a command takes),
or name `reset-failed` in the failure text so the advice can work.

### 3. On the copy route a forwarded command's Git is a child, and its death by a signal becomes exit status 1

`src/native/invocation/entry_forward_action_planning.rs:141-149`
sends every forwarded command run from a linked worktree or a bare repository
to `Action::ForwardSettled`,
because `copy_applies_with` returns true for those shapes
(`worktree_copying/worktree_copy_forward.rs:96`).
That action runs Git as a child (`worktree_copy_forward.rs:202-242`,
`forwarding.rs:187-206`)
and maps its end to a number:

```rust
//! Quoted from src/native/worktree_copying/worktree_copy_forward.rs:466-472.
            match run_real_git(self.real_git, child_arguments.as_slice(), child_overlay.as_slice()) {
                Ok(ChildOutcome::Exited(code)) => return Ok(GitStatus { code: Some(code) }),
                Ok(ChildOutcome::Signaled(_)) => return Ok(GitStatus { code: None }),
```

and then (`worktree_copying/worktree_copy.rs:237-249`):

```rust monochromatic-lint-skip-rustdoc
//! Quoted from src/native/worktree_copying/worktree_copy.rs:237-245.
pub fn exit_code(settlement: &Settlement) -> i32 {
    let git_code: i32 = settlement.git.code.unwrap_or(1);
    match (
        settlement.git.succeeded(),
        settlement.copy_failure.is_some(),
    ) {
        (true, false) => return 0,
        (false, false) => return git_code,
```

The sequence,
which is everyday use in this repository,
where most commands run in a linked worktree:

1.  In a linked worktree,
    the person runs `git log -p | head -1`.
2.  The wrapper takes the copy route and starts Git as a child
    with the caller's streams.
3.  `head` exits after one line,
    the pipe closes,
    and Git dies of `SIGPIPE`,
    which is what Git does with a closed reader.
4.  `ChildOutcome::Signaled(13)` becomes `GitStatus { code: None }`,
    then `1`.

The wrong result:
the caller sees exit status 1 where Git's caller sees a death by `SIGPIPE`
(141 under `pipefail`,
and `WIFSIGNALED` true under `wait`).
A script that tells "the reader went away" from "Git failed"
(`set -o pipefail`,
`${PIPESTATUS[0]}`,
a supervisor mapping 128+N)
takes the wrong branch.
The same mapping applies to any signal that kills Git alone:
the OOM killer,
a person's `kill <git pid>`,
a `timeout` that signals only the child.
The status is never a false success,
so nothing is lost;
what is lost is the promise that a forwarded read-only command behaves like Git.
stdin,
stdout and stderr do behave like Git's:
the child inherits them (`forwarding.rs:187-206`)
and the reproduction read a `cat-file --batch` answer through the wrapper.

Reproduced,
in "Reproductions" C,
with two positive controls:
native Git and the wrapper's own plain forward both report
`{"code":null,"signal":"SIGPIPE"}`,
and only the copy route reports `{"code":1,"signal":null}`.

Suggested fix,
a suggestion only:
keep the signal in the settlement
(`GitStatus` gains `signaled: Option<i32>`)
and,
after the copy has settled,
end the wrapper by that same signal
(`libc::signal(signal, SIG_DFL); libc::raise(signal)`)
so the caller's `wait` sees what Git's would,
or leave the wrapper killed by the signal as the plain forward does.
The announced query path
(`entry.rs:448-500`) has the same shape but maps a signal to `128 + signal`
(`forwarding.rs:118-140`),
which is closer and still not a signal death.

### 4. A kill between starting the units and writing the note leaves a service that never learns whose units they are

`src/native/service/service_provision.rs:214-228`
starts the socket unit,
then writes the note:

```rust
//! Quoted from src/native/service/service_provision.rs:217-226.
    let asked: Result<(), CallError> = ask_manager(manager, reload, known, host.instance.as_str());
    ...
    note_worktree(host.runtime_dir.as_path(), host.instance.as_str(), host.git_dir.as_path());
    stop_earlier_instance_socket(host, manager);
```

The service reads that note at its start
(`service_main.rs:288-300`)
and only a `Served` answer sets `own_git_dir`:

```rust
//! Quoted from src/native/service/service_main.rs:291-298.
        match own_worktree_state(name, journal.as_path()) {
            OwnWorktree::Gone(_) => {
                let _served_nothing = stop_own_socket_when_the_worktree_is_gone(name, journal.as_path());
                return EXIT_NOTHING_TO_SERVE;
            }
            OwnWorktree::Served(git_dir) => own_git_dir = Some(git_dir),
            OwnWorktree::Unmapped => {}
        }
```

With no `own_git_dir` the serving loop never wakes for the worktree question
(`service_server.rs:331`)
and never calls `own_units_stop`
(`service_server.rs:341-348`).

The sequence:

1.  A commit provisions the units and asks the manager to start the socket.
2.  The process is killed before `note_worktree` writes the note.
3.  The manager starts the service for the next connection.
    `worktree_of` finds no note,
    and the journal has no `gitdir=` record yet because nothing has landed there,
    so the answer is `Unmapped`
    (`service_units_of_removed_worktrees.rs:250-268`).
4.  The worktree is removed by a plain `git worktree remove`,
    which the copy route serves without asking the service,
    or deleted without Git.
5.  Nothing asks:
    the service has no `own_git_dir`,
    the sweep counts the instance as unmapped and stops nothing
    (`service_units_of_removed_worktrees.rs`,
    `stop_units_of_gone_worktrees`).

The wrong result:
a socket unit and a service unit for a directory that is gone keep running
until the three-hour idle exit,
and the sweep that exists to stop them can never name them.
No user data is touched;
what is lost is the guarantee that a removed worktree takes its units with it.
The window is two calls wide and needs a kill to enter it,
so it ranks below the three above.

Read from the code only.

Suggested fix,
a suggestion only:
write the note before `ask_manager`
(it needs no manager and names a directory that already exists),
so a kill leaves a note without units,
which the survey reports and the next provisioning overwrites.

### 5. A socket unit a person stopped makes the next commit ten seconds slower

`service_provision.rs:159` treats a present socket file as proof the units are in
place,
and `in_place` is what `provision` returns on,
so nothing is started.
The client only asks the manager to start a unit after `START_WAIT`
(`service_client_reach_service.rs:104-118`):

```rust
//! Quoted from src/native/service/service_client_reach_service.rs:105-117.
                    if answer.as_deref() == Some("inactive")
                        && started.elapsed() >= config.start_wait
                        && match last_start_request {
                            Some(at) => at.elapsed() >= config.start_wait,
                            None => true,
                        }
                    {
                        let _taken = (watch.start)(watch.bus.as_path(), watch.unit.as_str());
                        last_start_request = Some(Instant::now());
                    }
```

`config.start_wait` is `START_WAIT`,
ten seconds
(`service_client_listen_and_ask_to_retire.rs:34`,
chosen when there is no manager watch;
with a watch it is also the wait before a start request).

The sequence:

1.  A person stops the socket unit,
    which the documented downgrade remedy tells them to do
    (`SPEC.md`,
    the paragraph "A downgrade to a release with one service"
    names the stop before an older release commits),
    or `systemctl --user stop` after a fault.
    systemd keeps the socket file because the template sets no `RemoveOnStop=`.
2.  The next `git commit` finds the texts current and the socket file present,
    so `provision` starts nothing (`service_provision.rs:344-350`).
3.  The client connects,
    is refused,
    and hears `inactive` from the manager,
    but the start request waits for `start_wait` to pass.
4.  At ten seconds the manager is asked,
    the socket starts,
    the service starts,
    and the commit lands.

The wrong result:
that commit takes ten seconds longer than native Git,
against the owner's "commits vs native git should not take 5s longer".
Nothing is lost and the next commit is normal.

Read from the code only.

Suggested fix,
a suggestion only:
ask the manager to start the unit on the first `inactive` answer,
which is a no-op when the manager has already started it,
or let `provision` treat a stopped socket as not in place.

## Every state change, and what a kill leaves

For each step,
what is on disk,
what a kill just before and just after leaves,
and what notices it later.

1.  Entry itself:
    `hooks_off::activate` and `remember_own_executable` are memory only
    (`entry.rs:338-342`,
    `real_git.rs:584`).
    A kill leaves nothing and nothing notices.
2.  Real-Git resolution:
    reads only,
    the candidate headers,
    a whole-file comparison when lengths match
    (`real_git_candidate.rs`,
    `identical_content`),
    and the unit's `--git` when `PATH` offers nothing
    (`real_git_record.rs`).
    A kill leaves nothing.
3.  Copy-route recovery inside a forwarded command:
    writes and removes copy state.
    It is the copy route's subject,
    audited in `cli-git-copy-route-findings-on-main.md`,
    and is skipped here.
4.  A unit file written:
    `write_atomically` writes `"<name>.tmp-<pid>-<n>"` and renames it
    (`service_provision_unit_file_ownership_and_writes.rs:29-38`),
    so a reader sees the old file or the new one whole.
    A kill between the write and the rename leaves a temporary file that nothing
    removes and nothing reads:
    `in_place` compares only the two real names
    (`service_provision.rs:144-152`),
    the survey reads only names ending `.sock` and `.git-dir`
    (`service_units_of_removed_worktrees.rs:292-300`),
    and the manager loads only unit names.
    The cost is one small file per killed provisioning in the runtime directory,
    which goes at logout.
5.  The earlier release's units removed:
    `clear_earlier_units` and `retire_earlier_service` remove only files carrying
    the wrapper's marker,
    and `remove_settled` treats a file someone else removed as done
    (`service_provision_unit_file_ownership_and_writes.rs`,
    `remove_settled`).
    A kill after the removals and before the current texts are written leaves no
    units at all,
    which `in_place` reports false for,
    so the next commit writes them.
    The earlier service's own retirement is retried because its files are only
    removed after it stopped:
    `for file in &owned_files { remove_settled(..) }` is the last step
    (`service_provision_earlier_service_retirement.rs`,
    `owned_files`).
    Sound.
6.  The socket unit started and the note not yet written:
    finding 4.
7.  The note written:
    temporary file then rename,
    so a kill leaves either the old text or the new one,
    and the same temporary-file litter as step 4.
8.  The service creates its journal directory
    (`service_main.rs:311-313`):
    a kill leaves an empty directory,
    which the next start uses.
9.  The service finishes the earlier single service's open batches
    (`service_main.rs:342-349`)
    and then prunes that shared journal's finished records.
    A kill between the two leaves them unpruned,
    which the next start repeats.
10. The service settles this worktree's own open batches,
    which `serve` runs **after** the `ready` line
    (`service_main.rs:350`,
    `service_server.rs:269`).
    The `ready` line is a log entry,
    not a readiness signal:
    the listening socket belongs to the manager and queues connections from
    before the service starts,
    so a client that connects early waits rather than fails
    (`service_server_accept_connections_as_tasks.rs:40-66`).
11. The service's finished-batch records pruned:
    `prune_done` keeps `DONE_RETAINED` and deletes older records
    (`service_main.rs:348`,
    `service_server.rs:271-273`).
    A client that retries a landed request whose record was pruned in between
    would be landed again.
    The window is 1024 newer batches on one worktree
    (`landing/service_journal.rs`,
    `DONE_RETAINED`),
    against a client that retries within milliseconds,
    so it is a limit rather than a sequence I can state as reachable.
    The record itself is the commit route's subject.
12. The service's shutdown:
    the handler writes one line and records the stop
    (`service_signal.rs:196-260`),
    the shutdown task tells every waiter,
    waits `SETTLE_CEILING` for the work in flight,
    and ends the process by the signal that stopped it
    (`service_server_shutdown_after_a_stop.rs:62-82`,
    `stop_of_the_service_noticed_by_its_children.rs:419-430`).
    A kill before the handler is installed ends the process as a plain death;
    nothing durable is owed at that point.
    A kill after it leaves the Git children of a landing and Git's own
    `index.lock`,
    which the next instance reclaims because the marker names a dead process
    (`SPEC.md:6237-6260`),
    and the open batch,
    which the next start settles.
    That recovery is the commit route's subject and is cited,
    not re-derived.
13. `service_durable::drain` before every exit of the serving loop
    (`service_server.rs:345`,
    `:353`,
    `:359`,
    `:477`):
    a kill skips it,
    and what it would have synced is the journal writes of
    batches already answered.
    The client was answered only after those writes,
    so the batch is found again by the next `settle_open_batches`,
    which is the same recovery as step 12.
14. A scratch directory held by an open connection
    (`service_client.rs:387`,
    `watch_scratch`):
    the connection is the claim,
    so a kill of either end closes it and the service removes the directory
    (`service_server_connection_handling.rs:305`,
    `hold_a_scratch_directory`).
    Nothing is left.

## Every wait and its ceiling

Ceilings are the constants in `service/wait_ceilings.rs` unless another file is
named.

- A refused process start:
  `START_RETRY_BUDGET`,
  5 seconds,
  then the refusal is the answer (`push/start_retry.rs:33-34`).
- A forwarded command's Git on the copy route:
  **no ceiling**,
  which is right:
  it is the person's command with the person's terminal
  (`forwarding.rs:187-206`).
- The copy route's own Git queries ("`ls-files`",
  "`worktree list`"):
  `PLUMBING_STEP_CEILING`,
  two minutes (`wait_ceilings.rs:378`).
- One manager call while provisioning:
  `EXCHANGE_WAIT` (`systemd_dbus.rs`,
  `service_provision.rs:212`).
- The earlier single service asked to retire:
  `RETIREMENT_WAIT`,
  10 seconds,
  then the commit fails with the sentence that names the stop
  (`service_provision_host_and_manager.rs:64`,
  `service_provision_earlier_service_retirement.rs`,
  `wait_until_stopped`).
- The wait for a service to greet,
  with a manager to ask:
  `START_CEILING`,
  one minute;
  without one:
  `START_WAIT`,
  10 seconds
  (`service_client_reach_service.rs:98-131`,
  `wait_ceilings.rs:60`,
  `service_client_listen_and_ask_to_retire.rs:34`).
- The wait for an other-release service to retire:
  `start_wait` from the first meeting,
  10 seconds,
  then `ClientFailure::OtherRelease`
  (`service_client_reach_service.rs:175-205`).
- The wait for a request's answer:
  `ANSWER_CEILING`,
  twenty minutes,
  then the request is withdrawn
  (`service_client.rs:170-230`,
  `wait_ceilings.rs:34`).
- The wait for a withdrawal to be answered:
  `WITHDRAW_CEILING`,
  ten minutes (`wait_ceilings.rs:345`).
- The extra wait when the request is inside a write that never ends:
  `SHORT_EXTRA_WAIT`,
  seven seconds (`wait_ceilings.rs:553`).
- The silence bound between messages,
  heartbeats included:
  `heartbeat_silence`
  (`service_client_settings_and_watches.rs`,
  `heartbeat_silence`).
- A reconnection after a broken connection:
  `RECONNECTS`,
  five,
  with `CONNECT_PAUSE` 25 ms times the break number
  (`service_client.rs`,
  `RECONNECTS` and `CONNECT_PAUSE`).
- A connection's request line:
  `REQUEST_READ_TIMEOUT`,
  10 seconds (`service_server_connection_handling.rs:35`).
- The serving loop's own waits:
  the idle bound `IDLE_EXIT`,
  three hours,
  shortened to `OWN_WORKTREE_CHECK`,
  60 seconds,
  when the worktree is known
  (`service_server.rs:326-360`).
- The service's shutdown:
  `SETTLE_CEILING`,
  about 24 seconds
  (`STOP_CEILING` 18.1 from `worst_delay_seen_on_this_host.rs`,
  plus `STOP_GRACE` 5,
  plus 1),
  the last-resort `alarm` at `LAST_RESORT`,
  about 29 seconds,
  and the unit's `TimeoutStopSec=`,
  35 seconds
  (`service_provision_host_and_manager.rs:80`).
- Ending a child:
  `STOP_GRACE`,
  5 seconds,
  extended to `STOP_CEILING` while a member of its tree is in uninterruptible
  sleep (`push/bounded_push.rs:91`,
  `process/stop_waits_while_a_member_cannot_be_interrupted.rs:120`,
  `escalation_waits`).

Waits with no ceiling in this route:

- the person's forwarded Git,
  deliberately;
- the serving loop's wait for work,
  which a daemon has and which its idle bound and its stop both end;
- `wait_for_the_wake`,
  which only the stop handler can end and which the handler always ends with a
  byte or with the alarm
  (`stop_of_the_service_noticed_by_its_children.rs:431`,
  `wait_for_the_wake`);
- the recursion chain of finding 1,
  which is the one wait here that nothing ends.

One boundary worth naming:
the client gives up at `START_CEILING` (60 seconds),
and a start's own settling of an earlier batch is bounded at
`SETTLE_LOCK_CEILING`,
one minute
(`wait_ceilings.rs:279`).
A service whose recovery takes the whole minute therefore answers after the
client's ceiling,
so the commit fails with "did not start" while the service is starting well.
It needs a foreign `index.lock` held for about a minute during recovery,
which the ceiling itself allows;
I did not reproduce it.

## Every lock, and why it exists

This route has **no lock of its own**.
`rg 'Mutex|RwLock|Condvar|flock|lockf|fcntl'` over `service/service_provision*.rs`,
`service/service_client*.rs`,
`service/service_main.rs`,
`service/service_server*.rs`,
`invocation/` and `process/`
finds one `fcntl`,
which sets `O_NONBLOCK` on the wake pipe the service's stop reads,
and no lock.

- Provisioning is lock-free by idempotence:
  identical texts through renames,
  `remove_settled` for a removal another command finished,
  and a start that is a no-op when the manager already started the unit
  (`service_provision.rs:187-232`,
  `service_provision_unit_file_ownership_and_writes.rs`,
  `remove_settled`).
- The one lock the route meets is Git's `index.lock`,
  taken by the recovery a start performs.
  Its exclusion is the claim chain beside the lock and the marker in it,
  not a lock primitive
  (`SPEC.md:5837`,
  `doc/planning/cli-git-one-reclaimer-per-index.md`).
- The service's own stop has no table of children and no lock:
  each wait owns its child and ends it
  (`stop_of_the_service_noticed_by_its_children.rs:1-40`).
- The one mutex this route's processes touch is the journald stream's pending
  bytes (`journald_native.rs`),
  which belongs to logging and not to this route.

Locks that could be removed:
none,
because there are none to remove.
The owner's "Use as few locks as possible."
is met here.

## Races between two commands on one repository

1.  Two commits provisioning at once:
    both write identical texts,
    both read them back as current,
    and the second start is a no-op.
    Sound,
    except for the failure cleanup:
    a commit whose `ask_manager` fails removes the files **it** wrote
    (`service_provision.rs:219-224`),
    so a commit that had just read them as current can start a socket whose unit
    file is gone and fail with the manager's refusal.
    The window needs a refused bus call;
    not reproduced.
    A fix worth considering is to remove the file only when it still holds this
    call's text.
2.  Two releases at once,
    an older one still on `PATH` and a newer one installed:
    each writes unit texts naming its own binary (`service_provision.rs:200-216`),
    so the last writer wins and the socket starts whichever text is on disk.
    A commit of the older release can be served by the newer binary,
    hear `OtherRelease` after ten seconds,
    and fail,
    which is the documented upgrade limit
    (`service_client_reach_service.rs:175-205`,
    `wait_ceilings.rs:60`).
    Nothing is lost;
    the commit says so and can be run again.
3.  Two worktrees of one repository:
    two instances,
    two journals,
    two sockets,
    one common object store.
    Nothing in this route is keyed by the common directory except the
    provisioning of the shared templates,
    which is idempotent.
    Sound.
4.  A worktree removed while its service serves it:
    the service asks every `OWN_WORKTREE_CHECK` (`service_server.rs:341-348`),
    the copy route asks before a removal
    (`worktree_copy_forward.rs:266`,
    `units_of_worktree_about_to_go`),
    and both use the same `worktree_is_gone`
    (`service_units_of_removed_worktrees.rs:191`,
    `worktree_is_gone`),
    so they cannot disagree.
    Sound,
    unless the note is missing,
    which is finding 4.
5.  An upgrade while a commit waits:
    the client asks the old service to retire,
    the old service finishes the batch it is in and exits with status 0,
    so `Restart=on-failure` does not start it again,
    and the next connection starts the new binary from the rewritten text
    (`SPEC.md:5794-5800`).
    A suspended client whose request was landed retries the same identity and is
    answered "already landed" from the journal
    (`service_client.rs`,
    `Submitted` and the journal's landed records).
    Sound.
6.  A stop while a commit is in flight:
    the connection is closed without an answer,
    the client reconnects with the same identity,
    and the new instance settles the open batch and answers it
    (the commit route's subject).
    The wait has the ceilings named above.
    Sound within those bounds.

## Reproductions

All three ran against the debug binary built from the audited revision
(`mise run //package/git-policy/cli:native:build`,
`target/debug/monochromatic-git-policy-cli`,
`0.6.1-alpha.5`),
by path,
on fixtures under `${HOME}/temp/agent/pi-entry-route-audit`.
The fixtures were made with plain `git` under `PATH=/usr/bin:/bin`,
as the copy-route audit did,
so no fixture commit went through the installed wrapper or its service.

### A. The wrapper starting itself through a script that does not `exec`

Script:
`recursion.ts`.
`bin-a/git` is a symlink to the built binary;
`bin-b/git` is a two-line script;
the command is `bin-a/git status` with `PATH=bin-a:bin-b`,
so the only `git` files are the wrapper and the script.
The run is bounded by the scope's `TasksMax=40` and by `timeout`.
The count is the wrapper processes below the first.

The control,
the same script with `exec`:

```text
{"label":"control-exec","counts":[{"atMilliseconds":150,"wrapperProcessesBelowTheFirst":0,...}],
 "ending":{"code":2,"signal":null},
 "stderrFirstLine":"monochromatic-git-policy-cli: this monochromatic-git-policy-cli executable (...)
 was started as Git by a monochromatic-git-policy-cli wrapper: this is process 4089149, ...
 The file that process chose as Git is .../bin-b/git. ..."}
```

The subject,
the same script without `exec`:

```text
{"label":"child-no-exec",
 "counts":[{"atMilliseconds":150,"wrapperProcessesBelowTheFirst":0,"allDescendants":0},
           {"atMilliseconds":300,"wrapperProcessesBelowTheFirst":10,"allDescendants":21},
           {"atMilliseconds":450,"wrapperProcessesBelowTheFirst":16,"allDescendants":32},
           {"atMilliseconds":600,"wrapperProcessesBelowTheFirst":16,"allDescendants":32}],
 "ending":"STILL RUNNING AFTER 2.5 s",
 "stderrLines":1,
 "stderrFirstLine":"",
 "stderrLastLine":""}
```

Ten wrapper levels at 300 ms and sixteen at 450 ms,
stopped there only because the scope refused more processes,
nothing on standard error,
and still running when the probe ended it.
The control proves the guard works when the script `exec`s;
the subject proves it does not when the script does not.

### B. A service unit in `failed`, from the code

Not reproduced.
It needs a real user manager whose unit is failed and the installed units,
which this audit must not touch.
The code path is quoted in finding 2,
and the unit's `Restart=on-failure`
and `service_main.rs:121-124`'s refusals are what make the state reachable.

### C. The exit status of a read-only command whose reader went away

Script:
`pipe-closes-early.ts`.
Each arm runs `log -p --all` with its output on a pipe,
reads one line,
closes the pipe,
and reports how the process ended.
The fixture's history prints about 229 kB,
well past one pipe buffer.

```text
{"last":"native Git, linked worktree","exit":{"code":null,"signal":"SIGPIPE"}}
{"last":"wrapper, linked worktree (copy route)","exit":{"code":1,"signal":null}}
{"last":"wrapper, main checkout (replaces itself with Git)","exit":{"code":null,"signal":"SIGPIPE"}}
```

Through `bash` with `set -o pipefail`,
the same three arms end the pipeline with 141,
1,
and 141.
The copy route is the only one that differs from Git.

The stdin and stdout half of the same promise,
from `signal-fidelity.ts`:

```text
{"tag":"stdin-stdout","child":4059255,
 "headLine":"39c487afa0168fe725ed0947ab79296741cd828d commit 468","bytes":521,"stderr":""}
```

An object name written to the wrapper's standard input was answered on its
standard output by the child Git,
so both streams are the caller's,
unchanged.

## What I checked and found sound

- The plain forward is exact:
  the wrapper replaces its process with Git
  (`forwarding.rs:230-262`),
  so the caller's process number,
  streams,
  signals and exit status are Git's.
  Measured in "Reproductions" C,
  where that arm reports the same `SIGPIPE` death as native Git.
- A forwarded command's stdin,
  stdout and stderr are inherited by the child on the copy route,
  and the wrapper's own queries before it capture their output instead of
  consuming the caller's input
  (`repo_read/git_metadata.rs:94`,
  `process/captured_child_run_to_its_end.rs:40-50`).
  Measured in "Reproductions" C.
- The hooks-off options follow every global option of the caller and precede the
  subcommand,
  so a caller's own `-c` cannot switch a hook back on,
  and a `git config` query that only reports or changes configuration gets none
  (`hooks_off.rs`,
  `hooks_off_arguments` and `add_arguments`).
- The recursion guard works for every shape it claims:
  this executable through any link or spelling,
  a byte-identical copy,
  a toolchain launcher whose record names this wrapper,
  a script carrying the TypeScript wrapper's marker,
  another release arriving as `replace` or as `child`,
  and a service started with `--git` naming any of those
  (`real_git_candidate.rs:506-552`,
  `started_as_git.rs:275-292`,
  `service_main.rs:231-234`).
  The `exec` arm of "Reproductions" A is its positive control.
- Every real-Git start carries the marker:
  the shared builder (`forwarding.rs:86`),
  the process-replacing start,
  the push builder
  (`push/auto_push_single_flight.rs:153-160`),
  and the two cleared-environment starts
  (`repo_read/git_system_config.rs:75-86`,
  `repo_read/git_metadata.rs:94`).
  `rg 'Command::new'` over the route's sources finds no other Git start.
- Readiness is the listening socket,
  which the manager owns,
  so a connection made before the service is running waits in the backlog and is
  accepted when it starts;
  the service's `ready` line is a log entry and nothing waits on it
  (`service_main.rs:350`,
  `service_server_accept_connections_as_tasks.rs:40-66`).
- The unit texts are compared,
  not merely present,
  so an upgrade repair happens on the first commit that follows it
  (`service_provision.rs:144-152`).
- `write_atomically` publishes a unit file or the note whole,
  and a reader sees the old text or the new one.
- The stop is a death by the signal that began it
  (`stop_of_the_service_noticed_by_its_children.rs:250`),
  so `Restart=on-failure` does not fight a stop,
  and the unit's `KillMode=mixed` keeps the manager from signalling the Git
  children the service is still unwinding.
  The unit's `TimeoutStopSec=` is above the service's own last-resort bound
  (`service_provision_host_and_manager.rs:80`).
- A service whose worktree is gone stops its own socket before serving,
  and the sweep and `doctor` use the same rule
  (`service_units_of_removed_worktrees.rs:191`,
  `worktree_is_gone`,
  `:328-331`),
  so a removal takes its units with it
  unless the note is missing (finding 4).
- Two worktrees of one repository share nothing in this route except the
  idempotent template writes,
  and their instances,
  journals and sockets are separate
  (`service_instance.rs`,
  `instance_of` and `socket_path`).
- A killed service is restarted by its socket,
  and a run that ended by a stop is not restarted by the manager,
  because it ends by the signal rather than by a status.

## What I could not determine

- Whether a real user manager reaches `failed` for this worktree's socket or
  service unit in practice (finding 2).
  I read no journal for the units and could not create the state without
  touching the installed units.
- How the recursion of finding 1 ends on a host with no process cap.
  The measured arm stopped at the scope's `TasksMax=40`;
  whether it ends with `EAGAIN` and a message or grows until the host refuses
  every start was not measured,
  and could not be,
  safely.
- The ten-second stall of finding 5.
  It needs a stopped unit in a real user manager.
- Whether a client can be suspended long enough for `prune_done`'s 1024-record
  window to delete a landed answer it still needs.
  That record belongs to the commit route and I did not measure its reach.
- Whether the `START_CEILING` and `SETTLE_LOCK_CEILING` boundary named in the
  waits section is reachable with a real foreign lock.
- Windows and macOS,
  which the owner's pause covers.
  The non-Unix branches of `forwarding.rs`,
  `signal_relay.rs` and `hooks_off.rs` are not compiled or run here.

## What to inspect and how to respond

Inspect,
in this order:

1.  "Reproductions" A,
    whose script is `${HOME}/temp/agent/pi-entry-route-audit/recursion.ts`,
    against `started_as_git.rs:275-292` and `real_git_candidate.rs:506-552`;
2.  "Reproductions" C,
    whose script is `pipe-closes-early.ts`,
    against `worktree_copy_forward.rs:466-472` and
    `worktree_copy.rs:237-249`;
3.  the two findings read from the code,
    against `service_provision.rs:140-170`,
    `:236-246`,
    `:344-350`,
    and `service_client_reach_service.rs:98-131`.

Respond with which of the two reproduced findings to fix first,
and whether `SPEC.md:4703-4711` should be corrected before either is built.
The audit changed no production code,
so nothing else is pending.
