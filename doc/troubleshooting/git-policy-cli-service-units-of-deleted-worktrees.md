# Commit service units of deleted worktrees kept listening and retrying for ever

Date:
 2026-11-11T09:30-05:00.
Subject:
 the units of a throwaway repository deleted by a proof kept a socket listening and a service retrying its push every 600 s,
 and nothing noticed the repository was gone.
Owner's rules this serves:
 "nothing should ever wait forever or indefinitely"
 and "The wrapper should have cleaned up after itself automatically deterministically."

## Symptom

On 2026-11-11T08:55-05:00 this machine's user manager held 984 unit instances named
`monochromatic-git-policy-cli@<hash>`,
522 of them active:
every one of them a unit instance of the git wrapper's commit service,
each described as "Commit service of one worktree for the git wrapper".
`systemctl --user show --property=ExecStart` of one of them printed:

```text
monochromatic-git-policy-cli service run --journal /home/user/.local/state/monochromatic-git-policy-cli/service/06690c9baafc3b8422946954a4a4c956 --git /usr/bin/git --instance 06690c9baafc3b8422946954a4a4c956
```

Its journal showed the same line every 600 s,
climbing for a day:

```text
push will be tried again branch="refs/heads/main" oid=... next_try=149 in_seconds=600 cause: the repository at /tmp/lfs-login-proof-wGfMF6/repository-a/.git is not reachable now (is its drive mounted?)
```

The repository that line names was deleted hours earlier,
and so was the directory that held it.
The worktree of that unit is in its note,
`/run/user/1000/systemd/user/monochromatic-git-policy-cli-worktree-notes/06690c9b….git-dir`:

```text
gitdirHex=2f746d702f6c66732d6c6f67696e2d70726f6f662d7747664d46362f7265706f7369746f72792d612f2e676974
```

which decodes to `/tmp/lfs-login-proof-wGfMF6/repository-a/.git`,
a path whose holder `/tmp/lfs-login-proof-wGfMF6/repository-a` does not exist.

## Root cause

Three facts together keep such a unit alive,
and the fourth is how the units came to exist in the first place.

### 1. A deleted repository read as one a drive may still fill

`worktree_is_gone` decides whether a worktree is gone,
and before this branch it asked one question
(`src/native/service/service_units_of_removed_worktrees.rs`,
the function's body at `6cfc399d09`):

```rust
let anchor: Option<&Path> = if parent.file_name() == Some(std::ffi::OsStr::new("worktrees")) { parent.parent() } else { Some(parent) };
return match anchor {
    Some(place) => place.is_dir(),
    None => false,
};
```

The rule reads: the Git directory is missing **while the place that holds it is there**.
A linked worktree's Git directory is `<common>/worktrees/<id>`,
so the place is `<common>`;
a main checkout's is `<checkout>/.git`,
so the place is the checkout.
A repository deleted with `rm -rf` takes the Git directory **and** the place that held it,
and the rule answered "not gone",
the same answer it gives for a repository on a drive that is not mounted.
That answer is what makes the note's instance look live for ever.

### 2. The worktree was never asked about, because a pending push counted the service busy

The serving loop answers a wake with the first arm that matches
(`src/native/service/service_server.rs`,
`serve_in_a_landing`).
At `6cfc399d09` the arms were ordered like this:

```rust
Received::Timeout if busy.load(Ordering::SeqCst) > 0 => {
    last_work = Instant::now();
    continue;
}
Received::Timeout if own_worktree_gone_now(settings, until_idle) => { /* stop and end */ }
```

and the after-commit work counts the service busy for as long as a push waits for a new try
(`src/native/service/service_server_work_loop_awaiting.rs:313`):

```rust
// A repository waiting to be looked at again keeps the service from exiting idle, as running work does.
if !retries.is_empty() && !counted_waiting {
    busy.fetch_add(1, Ordering::SeqCst);
    counted_waiting = true;
}
```

A service with one pending push therefore never reached `own_worktree_gone_now`:
the busy arm refreshed `last_work` and took `continue` on every wake,
so the idle exit three hours away never came either.
It did not matter that the worktree question was cheap and decisive;
it was behind an arm that was always taken.

### 3. The plan of the deleted repository never ended

The plan is a file per branch under the instance's journal
(`src/native/push/push_retry_store.rs`),
read again at every start (`src/native/service/service_journal_restored_at_start.rs`).
The drain ends the plans of a repository the filesystem proves gone
(`src/native/landing/unreachable_git_dir_proven_gone.rs`),
and that proof reads the nearest directory above the Git directory that exists:
a mounted filesystem's root,
or one holding other entries,
proves the repository deleted;
an empty one may be a mount point waiting for its drive.
The services measured here run `0.6.1-alpha.10`, whose drain predates that proof,
so each of their plans was kept,
and a kept plan is also what keeps the service busy (fact 2).

### 4. How units for throwaway repositories came to exist in the owner's manager

The measured units start the toolchain binary
`/var/home/user/.local/share/monochromatic-dev/toolchain/cargo-monochromatic-git-policy-cli/0.6.1-alpha.10/…`,
not a build of this crate,
and their journals are under the account's real state directory.
They are the LFS login proof of 2026-11-10,
whose script (`~/temp/agent/lfs-login/proof/run.ts`) handed Git the whole of its own environment:

```ts
const GIT_ENV = {
  ...process.env,
  GIT_ASKPASS: '',
  GIT_TERMINAL_PROMPT: '0',
};
```

so `HOME`, `XDG_RUNTIME_DIR` and `DBUS_SESSION_BUS_ADDRESS` were the account's own,
the installed wrapper provisioned socket activation in the account's real manager
(`src/native/service/service_provision.rs`),
and the proof's throwaway repositories under `/tmp` and `${HOME}/temp/agent` became
worktrees with a unit instance each.
The brief of that proof did not tell the run to keep its own session,
which the `pi-as-subagent` skill now does
(`.agents/skills/pi-as-subagent/SKILL.md`,
"give that run its own `HOME` and `XDG_RUNTIME_DIR` under its run folder").

## How to list the units whose worktree is gone

Read-only,
over the user manager's own list and the notes the wrapper wrote beside the sockets:

```sh
systemctl --user list-units --plain --no-legend 'monochromatic-git-policy-cli@*' |
  awk '$3 == "active" { print $1 }' |
  while read -r unit; do
    instance=${unit#monochromatic-git-policy-cli@}
    instance=${instance%.*}
    note="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/systemd/user/monochromatic-git-policy-cli-worktree-notes/$instance.git-dir"
    if [ ! -f "$note" ]; then printf '%s\tunmapped\n' "$unit"; continue; fi
    git_dir=$(tail -c +11 "$note" | xxd -r -p)
    [ -d "$git_dir" ] || printf '%s\t%s\n' "$unit" "$git_dir"
  done
```

It lists every active unit instance whose note names a Git directory that is not there,
and `unmapped` for an instance whose note is missing.
`git monochromatic-git-policy-cli doctor` counts removed worktrees and says nothing was changed,
and `git monochromatic-git-policy-cli service stop-units-of-removed-worktrees` stops them,
which is the coordinator's decision and not this command's.

Measured on this machine at 2026-11-11T09:05-05:00,
over the 520 active unit instances the manager then held:

- 211 socket unit instances and 14 running service units,
  225 in all,
  belong to a Git directory whose whole repository the filesystem proves gone;
- the rule this branch started from (`the place that held it is there`) found 34 of them;
- 95 active units have no note at all, so nothing can name their worktree,
  and neither rule touches them;
- 186 active units belong to a worktree that is still there.

The count drifts while agents work:
the same command listed 491 instances twenty minutes earlier,
and the brief's 984/522 was taken at 08:55.

## What this branch changes

- `worktree_is_gone` asks a second question when the place that held the Git directory is missing too:
  the proof the retry plans already use
  (`src/native/landing/unreachable_git_dir_proven_gone.rs`,
  `prove`).
  A repository deleted under a directory that holds other entries,
  or from the root of a mounted filesystem,
  is gone;
  a path behind an empty directory still proves nothing.
  The service,
  the survey,
  `doctor` and the sweep now share one answer for both cases.
- The serving loop asks whether its worktree is gone **before** the busy arm,
  so a service with a pending push is asked like any other
  (`src/native/service/service_server.rs:401`).
- A service that finds its worktree gone ends the retry plans of that repository
  (`src/native/service/service_plans_of_a_gone_repository.rs`),
  writes one journal line naming the branches whose pushes will not be tried again,
  and then removes the note,
  the socket file and the journal as before
  (`src/native/service/service_units_of_removed_worktrees.rs:446`).
  A plan of a repository that is still there,
  and every other record,
  stays and keeps the journal.
- Provisioning asks where a socket unit the manager already holds listens **before** it starts anything,
  so a command whose runtime directory the manager does not use no longer starts a unit it can never reach
  (`src/native/service/service_provision.rs:263`).
- The controls' service command removes the runtime directory and the session bus it would otherwise inherit,
  so no control can reach a real user manager
  (`src/native/binary/binary_support_fixture_service.rs`,
  `service_command`).

## Verification

The controls that fail before these changes and pass after them:

- `a_worktree_whose_whole_repository_was_deleted_is_gone`
  (`src/native/service/service_units_of_removed_worktrees_tests.rs`):
  fails with "the directory that held the repository holds other entries" on the old rule.
- `a_service_with_a_push_plan_pending_ends_when_its_repository_is_deleted`
  (`src/native/service/service_server_own_worktree_tests.rs`):
  fails on the old arm order, where the busy answer kept the service alive.
- `a_service_of_a_gone_worktree_ends_the_plan_of_that_repository_and_the_journal_goes`
  and `a_journal_that_keeps_a_plan_of_a_repository_that_is_there_stays`
  (`src/native/service/service_units_of_removed_worktrees_tests.rs`).
- `a_plan_of_a_deleted_repository_ends_and_the_plans_of_repositories_that_are_there_stay`
  (`src/native/service/service_plans_of_a_gone_repository_tests.rs`).
- `a_service_the_manager_made_elsewhere_is_refused_with_both_addresses`
  (`src/native/service/service_provision_refusals_before_the_wait_tests.rs`):
  now also asserts that nothing was started.
- `a_service_a_control_starts_is_never_handed_the_manager_of_its_parent`
  (`src/native/binary/binary_service_tests.rs`).

The measured before and after for the rule itself is the two counts above:
34 units found by the old rule,
225 by the new one.

## What does not work

- Killing the service by name,
  or a `kill` of the unit's main process:
  a stopped service is started again by the socket unit on the next connection,
  the units are the manager's,
  and the wrapper never kills by name
  (`service_units_of_removed_worktrees.rs`,
  module doc).
- Removing the socket file,
  or the note,
  without stopping the unit:
  the manager keeps listening on the socket it made,
  and the file is what the survey reads,
  so the instance stops being countable while its unit runs on.
- Waiting for the idle exit:
  a pending push plan counts the service busy
  (`service_server_work_loop_awaiting.rs:313`),
  and the busy arm refreshed the idle clock on every wake,
  so the three-hour exit was postponed for ever.
- `git worktree prune`,
  `git worktree remove --force` run by a Git the wrapper does not serve,
  and `rm -rf` of the repository:
  none of them asks the manager,
  so the units stay until a service or the sweep ends them.
- Giving a proof run its own `XDG_RUNTIME_DIR` without a bus:
  the wrapper then has no route to a service and refuses every commit
  (`src/native/landing/service_route.rs`,
  `choose_route`),
  which is why the proof needed the account's own session in the first place.
  A proof that must commit needs either the real session (and then its repositories leave units
  this release now ends by itself) or a manager of its own (banned on this host).

## Upstream filing

Nothing to file:
the defect is this repository's own code,
and the only outside fact used is systemd's `%t`,
which is documented behavior.
