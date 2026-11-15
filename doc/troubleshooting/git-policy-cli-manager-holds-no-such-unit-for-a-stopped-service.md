# The wrapper waits for a unit the manager reports `inactive`, and systemd reports a stopped unit as one it does not hold, so nothing starts the commit service and a made commit loses its backup push

Affected:
`monochromatic-git-policy-cli` `0.6.1-alpha.36` and earlier,
on Linux with a systemd user manager,
in a worktree whose commit service was retired before it served anything.
Measured on this host on 2026-11-14 in the worktree
`.claude/worktrees/pi-integration-pass-47-with-the-test-that-fails-when-two-failure-variants-share-a-number`
(instance `1f1e503a7d710859f32de730a8345bf8`),
at 20:51:03,
20:58:58,
21:07:21 and 21:15:01 local time,
and again in another worktree of the same evening
(instance `033c2f22c3af24a2905e0d4b4906eb83`)
from 21:22:54 until a person started that worktree's socket unit at 21:32:05.
The wrapper's side is
`package/git-policy/cli/src/native/service/service_client_reach_service.rs`,
`package/git-policy/cli/src/native/service/service_client_settings_and_watches.rs`,
`package/git-policy/cli/src/native/service/systemd_dbus.rs` and
`package/git-policy/cli/src/native/service/service_own_socket_of_a_retire_that_served_nothing.rs`.

## Symptom

A merge or a commit is made and its backup push is not handed over,
with these two lines verbatim:

```text
monochromatic-git-policy-cli: Waiting for the commit service of this worktree... nothing has answered at /run/user/1000/monochromatic-git-policy-cli/1f1e503a7d710859f32de730a8345bf8.sock yet (and the service manager does not hold monochromatic-git-policy-cli@1f1e503a7d710859f32de730a8345bf8.service); this wait ends as soon as it answers.
Your commit is saved. Its backup push and upkeep could not be handed to the commit service: the commit service did not start. Run `git push` when ready.
```

What makes it confusing:

- the service's own log holds no line about the commit,
  because nothing asked the manager for a start:
  `journalctl --user --unit monochromatic-git-policy-cli@<instance>.service`
  shows the retirement and then nothing.
- a plain commit in that worktree repairs it,
  because a commit provisions the service and a merge's hand-off does not.
- `git push` goes through for the same reason,
  so the remedy the message names is the one that works,
  and the cause is never named.
- every later merge of that worktree fails the same way,
  with the same words.
- a manager that cannot be asked at all prints the same words,
  which is why nothing in the message tells the two apart.

## Root cause

1. A client of the new release meets a service of the previous release and asks it to retire
   (`reach_current`,
   `service_client_reach_service.rs`).
1. A service that landed no commit in its own process stops its own socket as it goes:
   the manager writes `Closed <unit>.socket`,
   and the service logs
   `this commit service of <instance> was retired before it served anything, and it stopped its own socket so no connection starts it again`
   (`service_own_socket_of_a_retire_that_served_nothing.rs:70`,
   decided by `if served == 0` in `service_server.rs:569`).
   The repair that file counts on is the client's own:
   `the client's own wait starts the unit when it finds it inactive`.
1. The manager unloads a unit that stopped and that nothing references,
   and `GetUnit` for such a unit is answered with an error rather than a state.
   Measured against this machine's manager:

   ```text
   $ gdbus call --session --dest org.freedesktop.systemd1 --object-path /org/freedesktop/systemd1 \
       --method org.freedesktop.systemd1.Manager.GetUnit \
       'monochromatic-git-policy-cli@1f1e503a7d710859f32de730a8345bf8.service'
   Error: GDBus.Error:org.freedesktop.systemd1.NoSuchUnit: Unit monochromatic-git-policy-cli@1f1e503a7d710859f32de730a8345bf8.service not loaded.
   ```

   The same call for a unit whose service was running answered an object path instead
   (`@6b72042f0d046d851245c7a38f65a2a9.service`,
   this run's own worktree).
   `systemctl --user show` prints `LoadState=loaded` for the first one only because it loads the unit first.
1. The client turned that answer into nothing.
   `Bus::active_state` maps the manager's `NoSuchUnit` to `Ok(None)` (`systemd_dbus.rs:429`),
   `unit_state` collapsed `Ok(None)` and every error into one `None`
   (`service_client_settings_and_watches.rs`,
   before this branch),
   and `reach` read `None` as "the manager cannot be asked":
   it set `cannot_ask` and asked for no start,
   because only `Some("inactive")` and `Some("failed")` asked for anything
   (`service_client_reach_service.rs`,
   before this branch).
   A unit that stopped is never reported `inactive` to a client that only asks `GetUnit`,
   so the repair step 2 counts on could not run.
1. The wait ended at its ceiling as `ClientFailure::DidNotStart`,
   whose words said only that the service did not start,
   and `service_handoff.rs` turned that into the reason a made commit's backup push was not handed over.

`git push` then provisions the service on its way,
which is what the person was told to run.

An earlier reading of this log,
that the merges had met a manager nothing could reach,
was wrong for them:
their notice said `does not hold`,
which is the manager's answer about a unit,
not the silence of a manager.

## Verification

The manager's answer,
read only,
on this host:
a loaded unit and the sighting's unloaded unit answer as quoted above.

The client's behaviour without any manager,
in this package's own controls
(`mise run //package/git-policy/cli:native:test:host -- 'service::service_client_reach_service::'`):

- `a_unit_the_manager_holds_no_such_unit_for_is_asked_to_start`:
  the stand-in answers `NotHeld`,
  and the client must ask for the start and reach the service that start brings up.
  Before this branch it failed with
  `DidNotStart { unit: Some("…@0123….service"), answer: None, start_asked: false }`
  after twenty-one state answers and no start.
- `a_manager_that_cannot_be_asked_names_itself_and_asks_for_no_start`:
  a manager nothing can reach gives no start,
  the notice says `could not be asked about` rather than `does not hold`,
  and the failure and the hand-off's reason name the unit.
  Before this branch it failed on the notice's words.
- `landing::service_messages_of_a_start_that_was_asked_and_did_not_come::tests`:
  every state of the clause reads as itself and names no implementation word.

The user boundary,
in a disposable repository under a disposable `HOME` and `XDG_RUNTIME_DIR`,
with a private `dbus-daemon` on a socket of the fixture's own and no manager on it
(a Node script the run kept in its scratch folder and not in this repository,
which built the fixture with `/usr/bin/git`,
put the wrapper first on `PATH` as `bin/git`,
and ran `git merge --no-ff --no-edit main` there):

- `0.6.1-alpha.36` prints the two lines in "Symptom" for a merge,
  byte for byte.
- this branch prints
  `… (and the service manager could not be asked about …@4974….service); this wait ends as soon as it answers.`
  and
  `… the commit service did not start: the service manager could not be asked to start …@4974….service, so nothing started the commit service. Run git push when ready.`
- the push-trigger record (`<git dir>/monochromatic-git-policy-cli-push-trigger.jsonl`)
  carries the same words in `detail`.

Not run:
an end-to-end merge under a real manager that answers `NoSuchUnit` and then starts the unit.
That needs this machine's own user manager for a throwaway repository,
which a fixture with a disposable `XDG_RUNTIME_DIR` may not use,
and no manager was built in a container here.

## Verified workarounds

For a worktree already in this state,
before this branch:

- `systemctl --user start monochromatic-git-policy-cli@<instance>.service`.
  The instance is the 32-digit FNV-1a hash of the real path of the worktree's own Git directory
  (`service_instance.rs`,
  `instance_of`).
  Tradeoff:
  a later service that retires before serving anything stops it again.
- any command that provisions the service,
  such as a plain commit in that worktree.
  Tradeoff:
  a merge leaves no such command behind,
  and a person's next push works only because the push path provisions.

Neither is needed after this branch:
the client asks for the start when the manager says it holds no such unit.

## What does not work

- Waiting longer.
  Nothing in the wait starts the service,
  so a longer wait delays the same words.
- Reading `ActiveState` without `GetUnit`.
  The manager has no state for a unit it does not hold,
  and neither call loads an instance unit of a template.
- Asking for `Reload`.
  It makes the manager re-read every unit it holds,
  which no Git command may ask for
  (`doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md`).
  A start already loads the unit.

## Upstream filing decision

Nothing to file,
and this is the poster's decision to revisit,
never a recommendation.

1. Is it upstream's fault?
   No.
   `GetUnit` answering an error for a unit it does not hold is the manager's documented behaviour,
   and a start is the call that loads a unit.
   The defect is the client's reading of that answer and the words it chose.
1. Can upstream fix it?
   Not applicable.
1. Are they supporting this use case?
   Not applicable.
1. Would the repository welcome a contribution?
   Not applicable.
1. Will they likely fix it?
   Not applicable.
1. Has a minimal fix been prototyped?
   Yes,
   in this repository,
   which owns the client:
   `ManagerAnswer` keeps the three answers apart,
   `reach` asks for a start on `NotHeld`,
   and `DidNotStart` carries the unit and what the manager answered.
