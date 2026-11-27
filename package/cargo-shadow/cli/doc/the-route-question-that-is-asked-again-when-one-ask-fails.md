# The route question that is asked again when one ask fails

What a whole run of this package's suite met on 2026-11-27,
what it measured,
and what the package now does about it (issue 6619),
and then how long an ask fails while a user manager reexecutes,
the span the asking again is now sized from,
and what that costs where no manager answers (issue 6623).

Status:
**unapproved and non-final**.
Written by two Pi runs:
one on branch
`fix/cargo-shadow-cli-issue-6619-ten-scope-tests-failed-together-once-and-passed-in-the-next-two-runs`,
and one on branch
`fix/cargo-shadow-cli-issue-6623-the-route-questions-asking-again-is-sized-from-a-measured-manager-reexec`,
which added the measurement,
the sized constants and the two controls of the span.
Every claim about behaviour here names what it rests on:
a command a round ran,
a line of this package,
or another program's own words.
What is still open is at the end.

## What happened

One whole run of `mise run //package/cargo-shadow/cli:test` on 2026-11-27,
in a worktree at `aa5200d1dd`,
failed ten tests,
and the next two whole runs of the same tree passed 941 of 941 each.
Every one of the ten needs a delegated control group,
and every one failed in ten to thirteen milliseconds.
The run's own command piped the task through `tail`,
so the messages of those ten are not in any record.

## What the ten share

Each of the ten is a control that needs a delegated scope,
so each begins with `this_test_needs_a_delegated_scope`
(`src/cgroup_test_support.rs`),
which asks `detect_route` whether this host can give one.
Between that question and the work the control measures,
a control makes a scope,
and making one takes a tenth of a second or more.

Measured 2026-11-27 on this host:

- `cgroup_probe_tests::probe_0_the_host_can_give_the_tests_a_delegated_scope`,
  which is that question and nothing else,
  took 16 ms
  (`cargo nextest run --release -E 'test(=cgroup_probe_tests::probe_0_the_host_can_give_the_tests_a_delegated_scope)'`).
- the ten tests alone,
  once:
  ten passed,
  in 0.099 s to 5.100 s each.
  The slowest is
  `bounded_run_end::retire_tests::the_question_at_a_run_end_keeps_the_bound_the_promise_names`,
  which reads a promise of five seconds.
- the ten tests with a `systemctl` on `PATH` that refuses every request:
  ten failed,
  in 0.008 s and 0.009 s each,
  every one of them printing

  ```text
  cargo-shadow tests: not run [<the test>]: this host cannot give it a delegated control group (the user manager cannot start app.slice (systemctl failed (Failed to start app.slice: stand-in refusal)))
  ```

A run in which all ten failed in about ten milliseconds is therefore a run in which each of them
answered no to the question and died there.
The one red run's ten failed in ten to thirteen milliseconds,
which is that shape and not the shape of any assertion or of a scope that was started and failed.

## What did not reproduce, and what the timings rule out

- Another round's work starting and stopping scopes did not make them fail here.
  With a run of the scope-heavy modules (91 tests,
  starting and stopping scopes throughout) beside the ten,
  twice,
  the ten passed in both rounds (0.353 s to 5.104 s each).
  That is not the same as "churn is not the cause":
  it says that this shape,
  at this rate,
  does not produce the failure on this host.
- The manager's own answer under that churn answered every time.
  2,000 asks of the question `detect_route` asks (`systemctl --user start app.slice` and
  `systemctl --user show app.slice --property=ControlGroup --value`) over 100 seconds,
  in one scope with that suite running in the same one,
  every ask answered with the path of the slice.
  A slice that empties and is collected is not the case here either:
  `app.slice` holds this account's application scopes and its agent units,
  so it does not empty.
- The tests' own assertions and the stopping of a scope are ruled out by the timings.
  Nothing behind the question finishes in ten milliseconds;
  a control that failed at an assertion would have taken at least the tenth of a second a scope takes,
  and a `systemd-run` that fails leaves the control waiting out `STATE_WAIT` (twenty seconds)
  for a member that never appears.
- A fixture folder,
  or one test executable,
  shared by two whole runs is ruled out.
  `scratch()` makes a randomly named directory for each control,
  and the two runs of that minute read different build folders
  (`~/.cargo/build/f3/…` and `~/.cargo/build/toolchain/nightly-2026-11-16/…`,
  from the journal's own test-scratch paths).

## What the one red run met, and what is not measured

The question was answered no.
Which of its answers was no is not measured,
because the messages were lost.
The answers the discovery can give were read from its own refusals
(`detect_route` in `src/control_group_route_discovery.rs`),
and the ones that do not depend on the moment cannot have been it:
a host that is not a cgroup v2 mount,
no `systemd-run` on `PATH`,
a slice without `cgroup.kill`,
the inode of the cgroup namespace,
and the view of the slice directory are all the same on every ask this host gives.
What is left is the two answers that are about a moment:
a program this package starts (`systemctl`) that did not run or did not answer,
and the read of the caller's control group's limits,
which is four files of
`/sys/fs/cgroup` under the caller's own group.
The first of those is what a refused ask looks like,
and it is the one the controls here plant;
the second is not planted by any control of this package.

The host's own record of that minute,
read with `journalctl --user` and `journalctl --since`,
holds two things that could have disturbed a program started then:
the user manager reexecuted at 14:07:27,
and systemd logged that the kernel's OOM killer had killed processes in `user.slice` and `-.slice` at 14:07:28.
`app.slice`'s own `memory.events` reports `oom_kill 6` and `max 4579289` over the slice's life,
and this kernel's journal holds no OOM line for the whole day,
so those counters cannot be dated to that minute from here.
None of this is shown to be the cause,
and this round did not try to make either one happen:
the rules of this machine ban generating load,
and a reexec of the account's manager is not a round's to ask for.

## What the package does now about it

`detect_route` asks `ask_where_the_slice_is` (`src/control_group_route_discovery.rs`),
which asks the manager up to `ASKING_AGAIN_ATTEMPTS` (five) times,
`ASKING_AGAIN_PAUSE` (two hundred milliseconds) apart,
and answers the path the first good ask reported.
One failure of that program is no longer the host's answer about itself.
An ask whose program spent its own deadline is not put again:
a manager that answered nothing inside its bound is what a host without one looks like,
and a second ask would only spend a second bound.
Each program an ask runs is timed on its own (`spent_its_bound`),
because the discovery starts two of them and a slow success followed by a fast failure must still be put again.
The words of the last failure are still the refusal,
so a run that goes on in the weaker path,
and a control that reports `not run`,
name which ask failed and what it said.
The five attempts span 0.800 s,
which is the figure the measurement below gives them;
they were three attempts fifty milliseconds apart (0.100 s) until 2026-11-27.

Measured 2026-11-27 on this host,
with a `systemctl` on `PATH` that refuses the first ask of every process that calls it,
so each of the ten meets a refused ask at its own start:

- with the change:
  the ten passed (0.109 s to 5.181 s each),
  and the controls of the change passed,
  four of them:
  the question put again after a failed ask,
  a manager that answered nothing asked once,
  a program that ended inside its bound read as no spent deadline,
  and a question that keeps failing still refused with the manager's own words.
- with `ASKING_AGAIN_ATTEMPTS` set back to one and nothing else changed:
  the ten failed in 0.010 s to 0.012 s each,
  every one of them printing
  `not run [...] this host cannot give it a delegated control group (the user manager cannot start app.slice (systemctl failed (the stand-in refused this ask)))`,
  and two of the four controls failed with them.

The controls are `control_group_route_discovery::asking_again_tests`:
the question is put again after a failed ask,
a manager that answered nothing is asked once,
a program that ended inside its bound is no spent deadline,
and a question that keeps failing still refuses with the manager's own words.
The last two are not made red by the change being taken out:
the bound rule is a rule about one program,
and the manager that answers nothing is asked once either way.
The question that fails at once and the question that keeps failing are:
with `ASKING_AGAIN_ATTEMPTS` at one they both fail,
the second on its own claim that the question was put more than once.

## How long an ask fails while the manager reexecutes, measured

Asked 2026-11-27,
in a container whose user manager and bus were the container's own
and in a guest whose were the guest's,
never against this account's manager.
The harness made every ask `detect_route` makes,
in the order that function makes them,
read off it (`src/control_group_route_discovery.rs`):
the cgroup mount's statfs,
the `systemd-run` on `PATH`,
`systemctl --user start app.slice`,
`systemctl --user show app.slice --property=ControlGroup --value`,
the slice directory's stat,
its `cgroup.kill`,
the namespace's inode,
the slice directory's `statx`,
and the read of the caller's own control group's limits (all four files).
Each ask logged when it began,
what it answered and how long it took.

- The container:
  `docker.io/cachyos/cachyos-v4:latest`,
  `podman run --cpus=4 --memory=6g --pids-limit=8192 --blkio-weight=10`,
  Linux 7.3.5,
  systemd 262,
  a user manager of its own holding 657 units (this account's held 606 on that day),
  and `systemctl --user` answered over the manager's private socket.
  Seven reexecs of 0.430 s to 0.588 s and two reloads of 0.654 s and 0.734 s,
  in three runs of 284,
  305 and 574 rounds of the asks,
  each reaching 10 s past its last disturbance:
  every ask answered except during the reexecs themselves,
  where 59 asks refused over those seven windows,
  in 3 ms to 6 ms
  (`Failed to connect to user scope bus via local transport: Connection reset by peer`)
  or after waiting 0.08 s to 0.35 s
  (`Transport endpoint is not connected`,
  `Connection reset by peer`),
  and neither reload refused an ask at all.
  In a control run of the same length with no disturbance every ask answered,
  and the long asks (2.5 s to 3.0 s) that appear in both runs are this machine's pressure,
  not the manager's.
- The guest:
  Ubuntu through `mvm` (Linux 6.8,
  systemd 255,
  4 CPUs,
  7.9 GiB),
  a user manager of its own holding 760 units,
  and `systemctl --user` answered over the session bus (dbus-broker),
  which is the transport this account's manager has.
  Seven reexecs of 0.292 s to 0.393 s and three reloads of 0.375 s to 0.448 s,
  in three runs of 6,886,
  7,145 and 7,307 rounds of the asks,
  each reaching 5 s past its last disturbance:
  fifteen asks refused,
  every one of them inside a reexec window
  (`Transport endpoint is not connected`,
  `Message recipient disconnected from message bus without replying`,
  `Warning! D-Bus connection terminated.`),
  after 0.08 s to 0.35 s each,
  and not one ask refused in the five seconds after any of them
  (1,129 to 1,340 manager asks per window),
  nor in any reload's window or the seconds after it.
  The read of the caller's limits,
  the slice's stat,
  its `cgroup.kill`,
  the namespace and the `statx` answered every time.
- The case the measurement must be able to see:
  the same asks with no manager to reach refused every one of them
  (`Failed to connect to user scope bus via local transport: No such file or directory`),
  so a run in which nothing failed is a run in which the asks were answered,
  not one in which they were not made.

So an ask fails while the manager reexecutes and not after it,
for as long as the reexec lasts:
the longest window measured was 0.588 s (container) and 0.393 s (guest),
against the 0.365 s this account's own reexec took on 2026-11-27.
`ASKING_AGAIN_ATTEMPTS` (five) and `ASKING_AGAIN_PAUSE` (two hundred milliseconds)
span 0.800 s,
which covers the longest of them even when the first ask falls at its start.

What this does not cover:
the event itself.
The ten controls that failed on 2026-11-27 asked about half a second **after** the reexec had ended,
and no ask failed in the seconds after any reexec measured here,
in either environment,
so no figure here is sized to that event,
and what makes an ask fail after a reexec has ended is still unknown.
The constants are sized to a reexec,
which is what a measurement showed.

### The two controls of the span

The figure is held by two controls (`src/control_group_route_discovery_reexec_tests.rs`):

- `a_moment_as_long_as_the_longest_measured_reexec_is_asked_through`
  plants a `systemctl` that refuses every ask for 0.600 s and then runs the real manager,
  and requires the discovery to find the route anyway.
  It runs wherever the suite runs,
  and it fails where the two constants are set back to their old values:
  measured 2026-11-27 with three attempts fifty milliseconds apart,
  the same control failed in 0.157 s with
  `the user manager cannot start app.slice (... the stand-in refused an ask inside the measured moment)`.
  With the sized constants it rode through the moment in 0.644 s with five asks.
- `the_question_answers_across_a_real_reexec_of_a_containers_manager`
  reexecutes the user manager it finds,
  three times,
  and requires every discovery made across them to answer.
  It runs only inside a container
  (`/.dockerenv` or `/run/.containerenv`),
  because the manager it reexecutes must not be this account's;
  outside one it prints what it did not do and ends rather than failing the host's run.
  Measured 2026-11-27 in the container above,
  as root with a user manager of its own of 657 units:
  two runs,
  260 and 210 discoveries in four seconds,
  5 and 4 of them overlapping three real reexecs (0.94 s and 1.00 s together),
  and every discovery answered.
  With the two constants set back to three attempts fifty milliseconds apart,
  the same control in the same container failed in 1.26 s:
  `the discovery refused 142.363298ms after it began, inside a container whose manager was reexecuting: the user manager cannot start app.slice (systemctl failed (Failed to connect to user scope bus via local transport: Connection refused))`.

The container that control is run in,
and why there:

```sh
# package/cargo-shadow/cli/doc/the-route-question-that-is-asked-again-when-one-ask-fails.md
# On the host: the test binary, which is copied in because the container has no toolchain.
mise exec -- cargo nextest list --locked --offline --release --lib --message-format json \
  | jq --raw-output '."rust-suites"[]."binary-path"'
# A container with a manager of its own; systemd is its PID 1 and this account's manager is untouched.
podman run --rm --detach --name cargo-shadow-reexec --systemd=always \
  --cpus=4 --memory=6g --pids-limit=8192 --blkio-weight=10 \
  docker.io/cachyos/cachyos-v4:latest /usr/lib/systemd/systemd
podman cp "$BINARY" cargo-shadow-reexec:/root/cargo_shadow_tests
```

```sh
# package/cargo-shadow/cli/doc/the-route-question-that-is-asked-again-when-one-ask-fails.md
# Inside the container: a user manager of its own, and units for it to serialize, so the reexec takes a while.
podman exec cargo-shadow-reexec bash -c '
  install -d -m 700 /run/user/0
  systemd-run --unit=reexec-manager --property=Slice=user-0.slice --property=Delegate=yes \
    --property=Environment=XDG_RUNTIME_DIR=/run/user/0 --property=Type=notify /usr/lib/systemd/systemd --user
  mkdir -p /root/.config/systemd/user
  for i in $(seq 1 600); do
    printf "[Unit]\n[Service]\nExecStart=/bin/sleep 100000\n" > /root/.config/systemd/user/load-$i.service
  done
  export XDG_RUNTIME_DIR=/run/user/0
  systemctl --user daemon-reload
  seq 1 600 | sed "s|^|load-|; s|$|.service|" | xargs systemctl --user start
  chmod 755 /root/cargo_shadow_tests
  /root/cargo_shadow_tests --exact \
    control_group_route_discovery::reexec_tests::the_question_answers_across_a_real_reexec_of_a_containers_manager \
    --nocapture
'
podman stop cargo-shadow-reexec
```

The container is where it runs because the manager it reexecutes must be a container's own:
this machine's rules (and this package's) forbid a test from reexecuting the account's manager,
and the package's `test` task runs on a host whose manager no test may disturb.
The reexec is the container's root user manager,
which its own `systemd-run` started,
so the host's manager,
bus and journal are not touched at all.

### What the asking again costs where no manager answers

Measured 2026-11-27 in a container of the same image with no user manager at all,
as root with no `XDG_RUNTIME_DIR`,
so `systemctl --user` refuses at once and the run takes the weaker path:
a small catalog repository whose lockfile is stale,
`cargo-shadow install --dry-run`,
the released `0.1.5-alpha.15` binary (one ask,
the code before the asking again)
and this branch's (five asks),
each beside a byte-identical copy of itself as a control arm,
nine rounds of all four arms interleaved:

- the released binary:
  43.2 ms median (29.8 ms to 68.5 ms),
  one ask;
- this branch's:
  883.1 ms median (880.3 ms to 889.3 ms),
  five asks;
- ratio per round 12.90 to 29.68,
  median 20.55;
- the copies:
  0.49 to 1.79 of their original for the released pair,
  and 0.97 to 1.00 for this branch's pair,
  so the ratio's spread is the baseline's own noise and not the arms' names or folders;
- every one of the 36 calls printed the weaker path's notice and exited 0.

So a call where no manager answers spends about 0.88 s before it settles on process-group containment,
where the released binary spent about 0.04 s.
The cargo behind the call is the package's own test stand-in (`tests/shim.rs`,
`FAKE_CARGO`),
so the figure is the shadow's own work and not cargo's.
That cost is the price of asking five times instead of once,
and it is paid on every call in a container,
a CI job or a session with no user manager;
where a manager answers,
the asking again costs nothing unless an ask fails.

## A failing run leaves its messages

The messages of the one red run were piped through `tail` and lost.
The package's `test` task now keeps the whole output of a failing run in
`target/test-runs/cargo-shadow-tests-<moment>-<pid>.log` under the package,
which the repository's `.gitignore` excludes,
and its last lines name that file.
The suite is still streamed to the terminal as it is written,
so a run that takes minutes is not hidden while it happens.
A degraded run keeps both of its runs when it fails:
the one that looked for the tests this host cannot run,
and the report run.
What this does not cover,
and why:
the output is held in the task's own memory and written when the run ends,
so a task killed outright,
by the kernel's OOM killer among others,
keeps nothing;
and the kept file is standard output followed by standard error,
not the order the two streams interleaved in.

Measured 2026-11-27 on this host,
with one planted panic in `git_sha1_tests::the_digest_of_nothing_is_the_published_one`:
the task exited 100 with `946 tests run: 945 passed, 1 failed, 26 skipped`,
its last line named the file it kept,
and that file held the whole run:
the build's own warning,
the `FAIL` line,
the panic's message and the summary.

## Open

- Which answer the one red run got,
  and what made that program fail at 14:07 that day.
  A run of the ten beside other work does not produce it here (above),
  and the two host events in that minute are not shown to be it.
- What makes an ask fail **after** a reexec has ended.
  Every refusal measured here fell inside a reexec window and none in the five seconds after one,
  and the ten of 2026-11-27 asked about half a second after the reexec had ended.
  The span sized here rides through the longest reexec measured;
  it is not shown to cover the event,
  and the next red run's kept messages are what would show what does.
- Which of the discovery's answers refused in the ten:
  the asks across a reexec refused at the manager (`systemctl`),
  and never at the read of the caller's limits,
  the slice's stat,
  its `cgroup.kill`,
  the namespace or the `statx`,
  but the ten's own words were lost before this measurement could ask the same of them.
- A control for the other answer that can change:
  a caller's limits that cannot be read.
  No control of this package plants a read of `/sys/fs/cgroup` that fails for a moment.
- The repository linter's `rust/max-lines` finding on `src/lib.rs` (304 code lines,
  limit 300) is on this branch's base already and is not this round's.
- A kept run is written when the run ends,
  not as it streams,
  so a task that is killed before its suite ends keeps nothing (above).
  Writing it as it streams would cost a file per run,
  including the runs that pass,
  and this round did not measure whether that is worth it.
