# A wrapper control stopped a commit service no command of it reached

## Symptom

`push_programs_held_in_memory::a_push_a_restarted_service_has_to_try_again_stops_and_names_the_command`
(`package/git-policy/cli`) failed in four of eight runs on one host on 2026-11-24,
alone as well as beside other tests (issue 6447).
When it failed,
the push record kept `"transport":true`,
the push completed after the control's "restart",
and the wait for the failure notice ended at its bound of 120 seconds.

The same trace appears in a passing run,
so the test result did not say which behaviour had been measured:
two services of one worktree started within milliseconds of each other,
one of them ended at the control's `stop_all`,
and nothing said which one the wrapped commands had spoken to.

## Root cause

The control's fixture built a world with a service of its own:

```rust
let service: FixtureService = start_fixture_service(&fixture);
service.serve(&fixture, repo.join(".git").as_path());
```

and every wrapped command it ran was handed a different runtime directory.
`bounded_for` sets `XDG_RUNTIME_DIR` from `service_runtime`:

```rust
if let Some(runtime) = service_runtime(fixture) {
    command.env("XDG_RUNTIME_DIR", runtime);
}
```

and `service_runtime` starts `Fixture.service` on first use and serves every worktree it finds:

```rust
let mut service = fixture.service.lock().expect("the fixture's service lock");
if service.is_none() {
    *service = Some(start_fixture_service(fixture));
}
let started = service.as_ref()?;
started.serve_worktrees(fixture);
return Some(started.runtime.clone());
```

`start_fixture_service` names a fresh runtime directory on every call,
so the fixture ran two services of one instance,
in two runtime directories,
both writing the fixture's journal.
The wrapped commit spoke to the second one,
which is where `service_push_programs_held_in_memory` held `GIT_SSH_COMMAND`,
and the control's `stop_all` ended the first one,
which held nothing.

What remained was a push record in `pending`
and a service that still held the settings and had not looked at it yet.
`LOOK_AGAIN_AFTER` is five seconds,
so whichever look came first decided the run:

- a look of the service that held the settings,
  once the control's restart had landed after it,
  found the record,
  ran the push with those settings and completed it.
  That is what the release intends for a service that still holds them.
- a look of the new service at its own start found the record first,
  and that service held no settings,
  so it wrote the notice.
  The control then passed on words a service had produced that no command of the control had spoken to.

The journal of one failing fixture root,
read line by line:
the `rt-1` service wrote its `started release` line tens of milliseconds after the `rt-0` one,
both of them named one instance,
the `rt-0` one wrote `exiting reason=signal-15` at the control's `stop_all`,
and a `pushed branch="refs/heads/main" ... outcome=done` line followed it.

## What the release promises, and what the control has to form

A push whose command chose a transport stops only when the settings are gone:
`push_without_its_transport::gone` answers `theServiceWasStartedAgain` for a record the service holds nothing for,
and `SPEC.md` says the push then stops rather than speaking another transport.
A service that still holds the settings is meant to push,
so a run in which the push completes is not a defect in the push.
The scene the control names,
a restart before the push runs,
never formed.

The control now forms that scene itself and races nothing:

- the wrapper is killed at `AFTER_THE_VERDICT`,
  after its gate approved the push and wrote that approval into the record,
  and before it promoted the record.
  A held record is drained by no round,
  so no service can run it while the scene is made.
- the control stops the fixture's own service,
  which is the one every wrapped command reaches,
  and with the process its memory of the settings goes.
- the control promotes the record itself with `work_item_holding::promote`,
  the step the killed command would have taken,
  while no service runs.
- it serves the next instance,
  whose start look finds a runnable record it holds nothing for,
  and the words appear well inside the five seconds of `LOOK_AGAIN_AFTER`.

A control that starts a service of its own,
and every helper that stops one,
now acts on `Fixture.service`:
`the_service_lock` in `binary_push_programs_held_in_memory_tests.rs` holds that lock,
and `stop_the_service` and `serve_the_service` are the only places the control ends or starts one.

## Verification

- Red on demand on 2026-11-24:
  `push_without_its_transport::gone` answered `None` for a record whose service holds nothing,
  which is the guard the control stands on,
  saved as `keep/plant-the-gone-guard-removed.patch` in
  `~/temp/agent/push-control-restarted-service-scene-2026-11-24`.
  The new service logged `pushed branch="refs/heads/main" ... outcome=done push_ms=142`,
  and the control failed at `never happened: the push records what stopped it` after 123 seconds.
- Green with the guard restored:
  twelve consecutive runs of that one control,
  each passing in about four seconds,
  and the same control passed in a run of the four controls of its file and in one run of its own after its helpers were
  renamed.
- One `started release` line per worktree at the top of a control's journal is the cheap check that the control has one
  service.
  A second line of the same instance means two services answer for one worktree,
  and whatever the control stops may hold none of the settings it means to take away.

## Still open

- A push a restarted service has to try again is recovered from a claim only after `CLAIM_LEASE`,
  which is 120 seconds,
  so a control that lets the old service claim the record and then restarts it waits out that lease.
  This control avoids that wait by keeping the record held until the restart has happened.

## The same defect in the end-to-end carry controls

`package/git-policy/cli/src/native/binary/binary_push_programs_end_to_end_tests.rs`
built its own world the same way:
it started a `FixtureService`,
served the repository in it,
and its restart control stopped and served that one,
while every wrapped command reached the fixture's own.
The control could not show it here,
because this host has a real `git-lfs` in `/usr/bin`
and every control of that file declined then;
the decline was itself a defect,
recorded in `doc/troubleshooting/git-policy-cli-gate-skips-the-real-git-lfs-controls.md`.
The round of 2026-11-24 that made those controls run everywhere fixed both:
its restart control and the second arm of its `a7` control
stop `Fixture.service`,
form the scene with the record held at `AFTER_THE_VERDICT`,
promote the record while no service runs,
and then serve the next instance,
as this control does.
Its evidence is in `~/temp/agent/end-to-end-push-program-controls-run-in-the-gate-2026-11-24/keep/`:
twelve green runs of that control,
and a red one with the source of the searched list classified as the command's own `PATH`,
which failed at `the refusal names the service's own list` in 1.57 s.
