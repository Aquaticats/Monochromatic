# The route question that is asked again when one ask fails

What a whole run of this package's suite met on 2026-11-27,
what it measured,
and what the package now does about it (issue 6619),
and then how long an ask fails while a user manager reexecutes,
the span the asking again is now sized from,
and what that costs where no manager answers (issue 6623),
and then what the discovery reads to tell a host with no manager from a manager that is not answering for a moment,
so that a call there asks once again (issue 6629),
and then which transports that read counts,
from systemd's own source at the two versions it runs beside,
and what a call that begins inside a reexec meets there,
and the asks of a manager that answered (the alpha.16 read's Low 1 to Low 4,
2026-11-27).

Status:
**unapproved and non-final**.
Written by five Pi runs:
one on branch
`fix/cargo-shadow-cli-issue-6619-ten-scope-tests-failed-together-once-and-passed-in-the-next-two-runs`,
one on branch
`fix/cargo-shadow-cli-issue-6623-the-route-questions-asking-again-is-sized-from-a-measured-manager-reexec`,
which added the measurement,
the sized constants and the two controls of the span,
and one on branch
`fix/cargo-shadow-cli-issue-6629-a-call-with-no-user-manager-to-reach-asks-once-and-a-reexec-is-still-ridden-through`,
which added the read of the transports an ask could be carried over,
its two controls,
the sampling of what that read sees across a real reexec,
and the cost measured again,
and one on branch
`fix/cargo-shadow-cli-cargo-shadow-alpha-16-release-pass-the-asking-again-is-spent-where-the-records-say-and-its-controls-can-fail`,
which changed the transports the read counts to the ones systemd's source shows,
added the controls of each of them,
made the discovery ask where the slice is before it starts the slice so an answer of the manager's ends the asking,
added the controls of that refusal and of an answer that is not a path,
and made the warning a kept unit carries say what the read saw,
and one on branch
`fix/cargo-shadow-cli-cargo-shadow-alpha-17-release-pass-five-record-sentences-say-what-is-so-and-two-controls-can-fail`,
which corrected five of this record's sentences against systemd's own file at each tag and a same-vendor second read,
added the control of the pause the call site sleeps and the control of an address beside a runtime directory,
and said what the span does not cover.
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
because an ask can be more than one program start and a slow success followed by a fast failure must still be put again.
The words of the last failure are still the refusal,
so a run that goes on in the weaker path,
and a control that reports `not run`,
name which ask failed and what it said.
Since 2026-11-27 the first ask of an attempt is the ask of where the slice is (`ask_where_the_slice_is_once`),
because that ask answers whatever the slice's state and is carried over a transport the same way:
a manager that is there answers it,
so a start that fails after it is the manager's own answer about the slice.
The slice is started only where that answer is no control group at all,
and the path is asked for again after the start.
A start the manager refused is followed by one more ask:
if the manager answers it,
the asking ends with the start's own words,
because a refusal does not change with a wait (measured below);
if it fails too,
the failure was a transport and the question is put again.
An answer that is not a control group path ends the asking the same way,
after the ask that followed the start,
because the manager answered twice.
The five attempts span 0.800 s,
which is the figure the measurement below gives them;
they were three attempts fifty milliseconds apart (0.100 s) until 2026-11-27.
Since issue 6629 the asking again is kept for the case it is for:
a manager that can be reached and did not answer for a moment.
A host whose socket file or whose manager's own directory is still there is read the same way,
because at the instant an ask fails that is what a manager in a reexec looks like.
Where no user manager can be reached at all,
the discovery reads that in its own process at the first failed ask
(`src/control_group_route_manager_reach.rs`,
"What the discovery reads where no user manager can be reached" below),
and that ask is the refusal,
so a call there makes one ask and spends no wait.

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
  Six reexecs of 0.430 s to 0.588 s and two reloads of 0.654 s and 0.734 s,
  in three runs of 284,
  305 and 574 rounds of the asks,
  each reaching 10 s past its last disturbance:
  every ask answered except during the reexecs themselves,
  where 59 asks refused over those six windows,
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
  Nine reexecs of 0.292 s to 0.393 s and three reloads of 0.375 s to 0.448 s,
  in three runs of 6,886,
  7,145 and 7,307 rounds of the asks,
  each reaching at least 5 s past every window and 12 s past its last:
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
  the namespace and the `statx` answered every time of 7,302 in the last of those runs,
  which is the one whose every ask was the caller's own group's (the two before it read a leaf service
  group of the guest that carries no `io.weight`,
  a guest's own arrangement and not a moment).
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

What the span does not cover:
the rest of a call.
A reexec that begins after the discovery has answered fails everything after it,
in both releases.
A same-vendor second read of `0.1.5-alpha.17` on 2026-11-27 fired a container manager's reexec
0 to 110 ms after each of its 24 calls began,
so every discovery was done before the window opened,
and 11 of the 24 calls exited 1
(7 of `0.1.5-alpha.16`'s 12 and 4 of `0.1.5-alpha.17`'s 12,
every one on the delegated route),
with the scope launcher's
`Failed to connect to user scope bus via local transport: Connection refused`
in the output of the calls whose standard error that read kept,
and `cargo update --workspace --offline failed` from the install.
The two releases differ only in where the reexec fell inside each call,
and the code that makes the scopes is the same,
so neither release tried a scope's creation again.
It is in the Open list.

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

### The control of the pause the call site sleeps

`the_gap_between_two_asks_is_the_pause_the_call_site_sleeps`
(`src/control_group_route_discovery_reexec_tests.rs`)
plants a `systemctl` that refuses every ask and writes down when each one began and ended,
and requires the time from one ask's end to the next one's beginning to be at least `ASKING_AGAIN_PAUSE`.
The two controls above hold the constants and the moment they outlast,
and neither of them reads the sleep in `ask_where_the_slice_is`:
with `std::thread::sleep(ASKING_AGAIN_PAUSE * 3 / 4)` there,
the span becomes 0.6 s where these records name 0.800 s,
its margin over the measured 0.588 s falls to about 2 percent,
and the two controls above stay green
(a same-vendor second read of this release named that edit on 2026-11-27).
It runs wherever the suite runs and needs no container.
The gap it measures holds the next stand-in's own start and the discovery's work between two programs as well as the sleep,
so a host that spends a quarter of the pause there can hide the edit.

Measured 2026-11-27 on this host:
with the sleep in place it passed in 0.908 s with pauses of 209.0 ms to 209.9 ms before the second to fifth ask,
and with `std::thread::sleep(ASKING_AGAIN_PAUSE * 3 / 4)` there it failed in 2.06 s with
`the shortest pause between two asks was 174.647195ms, shorter than the 200ms the call site sleeps, so the span is not the 800ms the records name`,
while the two controls above stayed green in the same run
(`the discovery rode through a moment of 600ms in 869.207269ms, with 5 asks`,
and the constants' control in 0.032 s).
Its four gaps with the edit were 174.6 ms,
229.7 ms,
358.7 ms and 460.3 ms,
so the edit showed in one gap of the four:
this host's work around the sleep can be as long as the difference the control is about,
and only the shortest gap speaks.

### What a call costs where no manager answers, before and after issue 6629's change

Measured 2026-11-27 in a container of the same image with no user manager at all,
as root,
so `systemctl --user` refuses at once and the run takes the weaker path:
a small catalog repository whose lockfile is stale,
`cargo-shadow install --dry-run`,
the released `0.1.5-alpha.15` binary and the branch's,
each beside a byte-identical copy of itself as a control arm,
nine rounds of all four arms interleaved.
The cargo behind the call is the package's own test stand-in (`tests/shim.rs`,
`FAKE_CARGO`),
so each figure is the shadow's own work and not cargo's.

Before the change,
which is what put the five asks into every call where no manager answers
(the round for issue 6623 measured this,
and this round did not measure it again):

- the released binary (one ask):
  43.2 ms median (29.8 ms to 68.5 ms);
- the branch as it was (five asks):
  883.1 ms median (880.3 ms to 889.3 ms);
- ratio per round 12.90 to 29.68,
  median 20.55;
- every one of the 36 calls printed the weaker path's notice and exited 0.

After the change,
measured again with the same harness,
against the final binary (`132472627c89846bfb70dfbcdf6359b32e4f78f0d7a4b247aec8c2b5996c7508`)
and the released one (`f84d5708e4397c2c17e76ce8724ae02a130e4aa78d6c8aec8c15ffa4aa84a82d`),
nine rounds of all four arms for each of the two ways a host has no manager:

- neither variable set:
  the released binary 38.43 ms median (37.49 ms to 43.11 ms),
  this branch's 43.78 ms median (38.25 ms to 48.12 ms),
  **one ask per call for all 36 calls**;
  ratio per round 1.01 to 1.28,
  median 1.11;
- `XDG_RUNTIME_DIR` naming a directory that holds nothing,
  and no bus address:
  the released binary 43.98 ms median (38.20 ms to 44.91 ms),
  this branch's 40.12 ms median (38.66 ms to 45.24 ms),
  **one ask per call for all 36 calls**;
  ratio per round 0.87 to 1.05,
  median 1.00;
- the copies against their own originals moved by as much as the two binaries did
  (0.83 to 1.43 for the branch's pair and 0.90 to 1.21 for the released pair where neither variable is set,
  0.87 to 1.15 and 0.88 to 1.15 in the second),
  so the spread of the ratio is the rounds' own noise and not the two binaries;
- every one of the 72 calls printed the weaker path's notice and exited 0.

So a call where no manager can be reached makes one ask and spends no wait,
which is what the released binary does,
and no difference between the two binaries' calls was seen there that is larger than the rounds' own spread:
the branch's median ratio is 1.11 in the first case and 1.00 in the second,
and a byte-identical copy of each binary moved as far as that on its own.
The absolute figures are about 40 ms here,
which is what the earlier round measured for the released binary (43.2 ms),
so the two rounds agree on the scale and only the per-round ratio is the finding.
Where a manager answers,
the asking again costs nothing unless an ask fails.

## What the discovery reads where no user manager can be reached

This is the change issue 6629 asked for,
and it is `src/control_group_route_manager_reach.rs`:
a read in the discovery's own process,
at a failed ask and before the wait that would come next,
of the transports an ask could be carried over.
A host where nothing can be reached ends the asking at its first failure;
where a manager can be reached the read runs again at each of the five,
which is a handful of `stat` calls each time.
It reads:

- `$XDG_RUNTIME_DIR` and `$DBUS_SESSION_BUS_ADDRESS` as the program an ask starts would see them,
  which is this process's own values changed by the settings a control plants;
- `$XDG_RUNTIME_DIR/systemd/private`,
  the manager's own socket,
  which is the transport `systemctl --user` takes first;
- every `unix:path=` entry of `$DBUS_SESSION_BUS_ADDRESS`,
  which is the session bus the address names;
- `$XDG_RUNTIME_DIR/bus`,
  the socket of the user bus,
  counted only where no bus address is set;
- `$XDG_RUNTIME_DIR/systemd`,
  the directory the manager keeps its own files in.

Which of them an ask can take is not recalled from what a bus usually does:
it is in the systemd the package runs beside.
That is the change of 2026-11-27,
and it is why the read no longer counts the bus socket beside a bus address.

### The transports the ask can take, from systemd's own source

`systemctl --user` reaches its manager through `bus_connect_transport_systemd`
(`src/shared/bus-util.c:504-545` at v262),
whose user-scope branch is `bus_connect_user_systemd` (`:519-524`).
Read at both versions the package was met beside on 2026-11-27:
262.2 on this host and in the read's container,
and 255 in the guest of issue 6623.

systemd 262,
`bus_connect_user_systemd` (`src/shared/bus-util.c:266-300` at v262):

```c
        e = secure_getenv("XDG_RUNTIME_DIR");
        if (!e)
                return -ENOMEDIUM;

        ee = bus_address_escape(e);
        ...
        bus->address = strjoin("unix:path=", ee, "/systemd/private");
        ...
        r = sd_bus_start(bus);
        if (r < 0)
                return r;
```

and its caller falls back to the session bus only where that answer is `-ENOMEDIUM` and an address is set
(`:525-532`,
`if (r == -ENOMEDIUM && secure_getenv("DBUS_SESSION_BUS_ADDRESS")) ... sd_bus_default_user(ret)`).
So with `$XDG_RUNTIME_DIR` set,
systemd 262 takes `$XDG_RUNTIME_DIR/systemd/private` alone,
and the bus socket and every address entry are transports the ask never takes.

systemd 255's copy of the same function (`src/shared/bus-util.c:234-268` at v255) differs in two lines
(`:244` against `:276` and `:260` against `:292`, in the order the block below shows):

```c
        e = secure_getenv("XDG_RUNTIME_DIR");
        if (!e)
                return sd_bus_default_user(ret_bus);
        ...
        r = sd_bus_start(bus);
        if (r < 0)
                return sd_bus_default_user(ret_bus);
```

So systemd 255 takes the private socket,
and falls back to the session bus where it cannot be connected.

Both versions' session bus is `bus_set_address_user`
(`src/libsystemd/sd-bus/sd-bus.c:1373-1398` at v262,
`:1336-1361` at v255):

```c
        a = secure_getenv("DBUS_SESSION_BUS_ADDRESS");
        if (!a) {
                ...
                e = secure_getenv("XDG_RUNTIME_DIR");
                if (!e)
                        return log_debug_errno(SYNTHETIC_ERRNO(ENOMEDIUM), ...);

                ee = bus_address_escape(e);
                ...
                if (asprintf(&_a, DEFAULT_USER_BUS_ADDRESS_FMT, ee) < 0)
                        return -ENOMEM;
                a = _a;
        }
```

So the address is preferred to `$XDG_RUNTIME_DIR/bus`,
and `$XDG_RUNTIME_DIR/bus` is taken only where the address is not set.

Measured 2026-11-27 on this host with systemd 262.2,
read-only `systemctl --user show app.slice --property=ControlGroup --value` with `SYSTEMD_LOG_LEVEL=debug`:

- `$XDG_RUNTIME_DIR` and an address that names a socket that is not there:
  `sd-bus: starting bus by connecting to /run/user/1000/systemd/private...`,
  exit status 0 and the slice's path printed,
  so the address was never used;
- the runtime directory unset (`env -u XDG_RUNTIME_DIR`) and this account's address set:
  `sd-bus: starting bus by connecting to /run/user/1000/bus...`,
  exit status 0 and the path printed,
  so the address's socket carried it;
- the runtime directory set and empty (`XDG_RUNTIME_DIR=`),
  with the address set:
  `sd-bus: starting bus by connecting to /systemd/private...`,
  exit status 1,
  because an empty variable is a set one for systemd and there is no fallback from that failure.

So the read counts the transports of both versions the package runs beside:
the private socket where `$XDG_RUNTIME_DIR` is set,
the address where one is set,
and the bus socket only where no address is set,
where 255 could reach it and 262 cannot.
That is the union,
which is the safe direction:
refusing a call across a reexec is the worse mistake,
and a state the read keeps asking in costs the span and not a wrong refusal.

No words are matched,
so no message catalogue and no locale can change the answer.
This machine and the container below have only `C`,
`C.utf8` and `POSIX` locales,
so a match under another locale could not have been shown here;
what can be shown is that the words never enter the decision:
the control `a_question_that_keeps_failing_is_still_a_missing_capability_and_names_its_last_words` refuses every
ask with `the stand-in never answers a slice`,
which is no program's own words at all,
and the discovery still puts the question five times,
because a manager can be reached there.

What the read answers:

- Both variables unset or empty:
  nothing names a transport,
  so no manager can be reached.
  `systemctl --user` says the same case in its own words
  (`Failed to connect to user scope bus via local transport: $DBUS_SESSION_BUS_ADDRESS and $XDG_RUNTIME_DIR not defined ...`).
- A name is set and none of the paths is there,
  nor the manager's directory:
  no manager can be reached.
  `systemctl --user` answers `Failed to connect to user scope bus via local transport: No such file or directory`
  for a runtime directory that holds nothing and for one that does not exist,
  and for a bus address whose socket is not there.
- The manager's own directory is there and its socket is not:
  a manager may be there and the asking again is kept (below).
- One of the sockets is there,
  or the address names something this process cannot look for
  (an abstract socket,
  an escaped byte,
  another transport):
  a manager may be there and the asking again is kept.
  That is the safe direction for the cost:
  a socket file left behind with nothing listening answers `Connection refused`,
  and a call there spends the span rather than being refused early (the states are in the README's
  `Development`).
  An empty `$XDG_RUNTIME_DIR` with a live address is that case too:
  the read names the address and keeps the asking,
  where systemd 262 refuses at once because it takes `/systemd/private`,
  which spends the span on a host that could answer sooner.

### The states the span is spent in, measured

Two runs of five rounds of every state,
in a rootless container whose own systemd was PID 1 and which had no user manager:
`0.1.5-alpha.16` beside a byte-identical copy of it before the change of 2026-11-27,
and the same released binary beside this branch's after it.
Every call is `cargo-shadow install --dry-run` in a catalog repository whose lockfile is stale,
with the package's own stand-in cargo behind it,
so the figure is the shadow's own work.
The four figures are the median of five rounds in this order:
`0.1.5-alpha.16`,
its copy,
`0.1.5-alpha.16` again,
this branch's binary.

Five asks a call,
and the median a call took:

- `$XDG_RUNTIME_DIR/systemd/private` a socket with nothing listening on it:
  853.7 ms,
  852.8 ms,
  855.2 ms,
  856.0 ms.
- `$XDG_RUNTIME_DIR/bus` a socket with nothing listening on it:
  853.0 ms,
  853.8 ms,
  855.6 ms,
  855.4 ms.
- a `unix:path=` entry of the address naming a socket with nothing listening on it:
  855.3 ms,
  854.3 ms,
  856.1 ms,
  856.1 ms.
- `$XDG_RUNTIME_DIR/systemd` a directory and no socket at this instant:
  856.0 ms,
  856.1 ms,
  856.0 ms,
  855.6 ms.
- the same through a symlink to the runtime directory:
  855.0 ms,
  854.8 ms,
  856.5 ms,
  856.6 ms.
- plain files at `systemd/private` and at `bus`:
  854.9 ms,
  854.3 ms,
  856.8 ms,
  857.0 ms.
- an address naming an abstract socket:
  856.7 ms,
  859.8 ms,
  856.9 ms,
  858.0 ms.
- an address with an escaped byte (`%74`):
  855.4 ms,
  855.4 ms,
  859.1 ms,
  856.7 ms.
- an address of another transport (`unix:tmpdir=/tmp`):
  863.3 ms,
  863.1 ms,
  857.3 ms,
  857.4 ms.
- a live session bus with no systemd on it,
  reached by the runtime directory:
  865.5 ms,
  860.7 ms,
  856.9 ms,
  858.0 ms.
- the same bus reached by the address:
  876.4 ms,
  876.0 ms,
  861.6 ms,
  862.7 ms.

One ask a call:

- neither variable set:
  29.5 ms,
  29.6 ms,
  32.6 ms,
  31.9 ms.
- `$XDG_RUNTIME_DIR` a directory holding nothing:
  29.8 ms,
  29.8 ms,
  32.5 ms,
  32.7 ms.
- `$XDG_RUNTIME_DIR` naming a directory that does not exist:
  30.6 ms,
  31.5 ms,
  32.4 ms,
  32.4 ms.

So 11 states spend the span and 3 spend one ask a call,
which is what the README's `Development` now names,
where it said before 2026-11-27 that the span was spent "there and nowhere else".
The two runs are five rounds each on one host under other agents' work,
so an absolute figure is context and the counts are the finding;
the arm beside the other binary in its own run is what shows the binaries are not the difference.

### A manager that reexecutes must not read as no manager

A reexec of the manager unlinks and rebinds its own sockets,
so a socket can be absent for a moment.
Sampled 2026-11-27 in the container below,
whose own user manager held 600 units and was reexecuted three times,
in two runs.

The first run took one sample in a loop with no fork in it and wrote a line only where a sample differed from
the one before it,
so its interval between two samples is 12.65 us and its file holds the moments of change and not a count of
each state:
553,220 samples over 7 s,
and three reexecs of 0.390 s,
0.369 s and 0.286 s beginning 1.50 s,
3.39 s and 5.26 s after the first sample.

- The manager's own socket was absent in **two intervals**,
  0.283 ms and 0.172 ms long,
  at the tail of the reexec of 0.369 s and the one of 0.286 s;
- `$XDG_RUNTIME_DIR/bus` was there in every sample,
  and so was `$XDG_RUNTIME_DIR/systemd`;
- 1,010 manager asks were made in that time,
  148 refused,
  **every one of the 148 inside one of the three windows**
  (145 `Failed to connect to user scope bus via local transport: Connection refused`,
  2 `Failed to get properties: Transport endpoint is not connected`,
  1 `Failed to get properties: Connection reset by peer`);
- no ask answered with the words of the no-manager case.

The second run declared its count before it started:
500,000 samples,
each written as a line of its own so the count in each state is exact,
and three reexecs of 0.374 s,
0.412 s and 0.412 s beginning 1.00 s,
2.38 s and 3.79 s after the first sample.

- **every one of the 500,000 samples found the manager's own socket present**,
  as the bus socket and the manager's directory were;
- 6,133 manager asks were made in that time,
  251 refused,
  **every one of the 251 inside one of the three windows**
  (248 `Connection refused`,
  3 `Transport endpoint is not connected`);
- no ask answered with the words of the no-manager case.

So the socket's absence in a reexec is intermittent:
two intervals of it in one run,
and none in the other.
The two runs are not the same instrument:
the first samples every 12.65 us and keeps only the changes,
which is why an absence of 0.17 ms could be seen at all,
while the second samples every 104 us (52 s over 500,000 samples) and keeps every sample,
so an absence shorter than about 0.1 ms can pass between two of its samples.
Each sampler asks a `-S` and a `-d` question of the path,
which is what the read itself now asks,
so neither of them reads a plain file at a socket's name as a transport.

The instant in which the two cases cannot be told apart is one of those intervals,
0.17 ms to 0.28 ms while the manager's own socket is unlinked and rebound,
and the read's answer there is that a manager may be there:
the manager's own directory is read as well as its socket,
and a host that never had a manager has neither.
Going the other way at that instant would refuse a call across a reexec,
which is the failure issue 6619 was about.
That is also why a host whose manager died leaving a socket file behind,
with the manager's directory still there,
is still asked five times:
at the instant an ask fails the two look the same,
and refusing a reexec is the worse of the two mistakes.

### The container the reexec control is run in, with this change

The recipe is in `## The two controls of the span` above;
run there 2026-11-27 with this change,
the test binary built from this branch and copied in,
as root with a user manager of its own:

```text
261 discoveries in 4s, 3 of them inside 3 real reexecs of 834.752787ms together, every one answered
test control_group_route_discovery::reexec_tests::the_question_answers_across_a_real_reexec_of_a_containers_manager ... ok
```

Run once more in the same container after `origin/main` was merged into the branch,
with a test binary rebuilt from the merged tree:

```text
267 discoveries in 4s, 5 of them inside 3 real reexecs of 810.655757ms together, every one answered
test control_group_route_discovery::reexec_tests::the_question_answers_across_a_real_reexec_of_a_containers_manager ... ok
```

And once more with the read in its final form,
which asks each socket path whether it is a socket and the manager's path whether it is a directory,
and leaves a bus address whose path holds a `%` unjudged:

```text
266 discoveries in 4s, 5 of them inside 3 real reexecs of 963.98263ms together, every one answered
test control_group_route_discovery::reexec_tests::the_question_answers_across_a_real_reexec_of_a_containers_manager ... ok
```

And with the asking again taken out,
which is `ASKING_AGAIN_ATTEMPTS` set to one and nothing else changed,
the same binary's control in the same container:

```text
the discovery refused 36.734257ms after it began, inside a container whose manager was reexecuting: the user manager cannot be asked where app.slice is (systemctl failed (Failed to get properties: Transport endpoint is not connected))
```

So the control is able to fail there,
and it fails in the shape of the ten controls of 2026-11-27.
Its host run is green either way:
outside a container it prints what it did not do and ends.

### The controls of the read

`control_group_route_manager_reach::manager_reach_tests`:

- `a_host_with_no_runtime_directory_and_no_bus_address_is_asked_once`;
- `a_host_whose_runtime_directory_holds_no_manager_socket_is_asked_once`;
- `a_manager_socket_at_the_runtime_directory_keeps_the_asking_again`;
- `a_bus_socket_at_the_runtime_directory_keeps_the_asking_again_where_no_bus_address_is_set`;
- `a_bus_address_naming_a_socket_keeps_the_asking_again_where_no_runtime_directory_is_set`;
- `a_bus_address_naming_a_socket_keeps_the_asking_again_where_a_runtime_directory_is_set`;
- `a_bus_socket_beside_a_bus_address_is_not_a_transport_the_ask_takes`;
- `an_address_this_process_cannot_judge_keeps_the_asking_again`;
- `the_manager_s_own_directory_keeps_the_asking_again`.

The first two cover the two ways a host has no manager that a container,
a CI job or a session without a manager gives,
and each requires one ask and less than two waits.
The next five are one per transport,
written on 2026-11-27 with the source above:
four the read counts (the manager's own socket,
the user bus socket where no address is set,
a socket an address names where no runtime directory is set,
and a socket an address names where one is)
and one it does not (the user bus socket beside a bus address),
and each requires the count the read answers with:
one ask where nothing a transport can be carried over is there,
and the full five where one is.
The address beside a runtime directory was added on 2026-11-27,
because a read that counted the address only where no runtime directory is set stayed green under every other control
(a same-vendor second read of this release named that edit):
the only control that set both before it named an address that is not there,
so one ask was right either way.
Measured 2026-11-27 on this host:
the new control passed in 0.877 s with five asks,
and under that edit it failed in 0.039 s with one ask where it requires five.
`an_address_this_process_cannot_judge_keeps_the_asking_again` covers the three unjudged addresses
(an abstract socket,
an escaped byte,
another transport),
and `the_manager_s_own_directory_keeps_the_asking_again` the reexec tolerance the directory is for.

The other probes of earlier rounds keep their answers:
a runtime directory that does not exist and an address naming a socket that is not there end the asking at one ask,
as the two "asked once" controls do.
A socket file with nothing listening is the case the Open list names,
where the read keeps the asking again.

Each plants a host through the settings an ask is made with,
so the host under the fixture keeps its own manager,
and asks through a stand-in that writes down the ask and hands it to the real `systemctl`,
so the refusal in the message is the system's own words about the host the control built.
The socket a control plants is a socket file made with `mknodat` and not a bound listener:
the read judges the file's kind,
and a fixture path under this package is longer than a socket address holds (`SUN_LEN` is 108 bytes).

Measured 2026-11-27 on this host:

- with the change,
  the two "asked once" controls passed in 0.026 s each with one ask,
  and the six that keep the asking passed in 0.84 s to 2.5 s with the count they require
  (`an_address_this_process_cannot_judge_keeps_the_asking_again` walks three addresses,
  15 asks);
- with the bus socket counted beside a bus address again (the read before 2026-11-27's change),
  `a_bus_socket_beside_a_bus_address_is_not_a_transport_the_ask_takes` failed in 0.838 s with five asks where it requires one;
- with the manager's directory check deleted,
  `the_manager_s_own_directory_keeps_the_asking_again` failed with one ask where it requires five;
- with `paths_in_a_bus_address` answering no path for an address it cannot judge,
  `an_address_this_process_cannot_judge_keeps_the_asking_again` failed with one ask where it requires five.

## What a call that begins inside a reexec meets, and which asks end the asking

Measured 2026-11-27,
in a container whose user manager and bus were the container's own
(the recipe is in "The two controls of the span" above,
with 600 units loaded so a reexec takes a while),
never against this account's manager:

- 24 reexecs of that manager,
  one call per reexec,
  the arms alternating:
  12 calls of `0.1.5-alpha.16` and 12 of `a16ctl`,
  a byte-identical copy of it.
  Each call was begun 60 ms after the reexec was fired,
  so it began while the manager was down,
  and a poller asked the manager every 5 ms through the same window.
  The windows the poller saw were 265.4 ms to 365.2 ms long.
- Every one of the 24 calls took the delegated route:
  no weaker-path notice,
  and its scopes were made.
- 41 asks of those calls failed,
  and every one of them inside a window:
  38 answered `Failed to connect to user scope bus via local transport: Connection refused`,
  and 3 answered a start failure that names the unit and the verb:

  ```text
  Failed to start app.slice: Transport endpoint is not connected See user logs and 'systemctl --user status app.slice' for details.
  ```
- The third shape names the unit and the verb,
  as a refusal of the manager's does,
  and it is not one:
  the connection was accepted and the manager went away under the request.
  So a failure's shape cannot tell a refusal from a transport that died,
  and the one thing that can is the manager answering a later ask.
  That is what the discovery now does,
  and it is why the start's failure is followed by one more ask.

So no ask of a call that began inside a reexec was an answer of the manager's,
and the change the alpha.16 read called Low 3's pick stands:
an ask the manager answered with a refusal of its own ends the asking,
and a failure to reach the manager is put again.

The same 24 reexecs were run again with this branch's binary,
after the change that asks where the slice is first,
12 calls of it and 12 of `0.1.5-alpha.16` alternated:

- every one of the 24 calls took the delegated route,
  and 21 of them began inside a window;
- 33 asks failed,
  every one of them inside a window:
  30 `Failed to connect to user scope bus via local transport: Connection refused`,
  2 of the start-failure shape above,
  and 1 `Failed to get properties: Transport endpoint is not connected`,
  which is the same connection going away under the ask that now comes first;
- no failure was an answer of the manager's,
  so the asking again rode through every window,
  and the shape the asks meet in a reexec is unchanged by the new order.

With the two runs together:
48 calls begun inside 48 reexecs,
42 of them inside a window the poller saw,
74 failed asks and not one of them an answer of the manager's,
and the delegated route in all 48.

Measured against a manager that answers with a refusal of its own,
`app.slice` masked in that container's manager and stopped,
so `show` answers no control group at once and `start` answers
`Failed to start app.slice: Unit app.slice is masked.`:

- `0.1.5-alpha.15`:
  1 ask,
  35.8 ms median,
  the weaker path 5 of 5;
- `0.1.5-alpha.16`:
  5 asks,
  895.1 ms median,
  and 883.1 ms in the second run where its copy was not beside it,
  the weaker path 5 of 5;
  its byte-identical copy,
  in the first run:
  5 asks,
  885.1 ms median;
- this branch's binary:
  **3 asks,
  56.5 ms median,
  the weaker path 5 of 5**,
  with the manager's own words unchanged.

Measured through a stand-in that answers every ask of where the slice is with an empty value
and hands the start to that container's manager,
which is what an answer that is not a path looks like:

- `0.1.5-alpha.16`:
  10 asks,
  895.2 ms median,
  and 890.4 ms in the second run;
  its byte-identical copy,
  in the first run:
  10 asks,
  895.9 ms median;
- this branch's binary:
  **3 asks,
  41.8 ms median**,
  and the refusal is `the user manager reports no control group for app.slice`.

Both are held by two controls in `control_group_route_discovery::asking_again_tests`:

- `a_refusal_the_manager_answers_ends_the_asking`,
  whose stand-in answers the ask of where the slice is with nothing and refuses the start with the masked slice's
  words,
  and which requires 3 asks and the manager's words in the refusal.
  Its words are that capture,
  copied here and into the control,
  and the state itself is measured against a container's own manager above.
- `an_answer_that_is_not_a_path_ends_the_asking`,
  whose stand-in answers the ask of where the slice is with nothing and hands the start to the real manager,
  and which requires 3 asks and the "reports no control group" refusal.

Measured 2026-11-27 on this host:
both passed in 0.037 s and 0.036 s with 3 asks each.
Each was shown red once,
by the edit that puts the answer again and never committed:
with the ask that follows a refused start not ending the asking,
`a_refusal_the_manager_answers_ends_the_asking` failed in 0.892 s with 15 asks where it requires 3,
and with an answer that is not a path put again,
`an_answer_that_is_not_a_path_ends_the_asking` failed in 0.940 s with 15 asks.

What this does not cover:
a reexec window that opens between the start's failure and the ask that follows it.
The ask after it would answer,
and a transport failure would then be read as the manager's answer,
which would refuse the call.
The gap is one program start,
and the window in which a connect is refused was 65 ms to 372 ms long in the containers
a same-vendor second read of this release measured on 2026-11-27
(65 ms to 99 ms with no services loaded,
88 ms to 96 ms with 20,
294 ms to 372 ms with 600),
so the two can meet,
and that read reached the state in 2 of its 24 calls whose reexec was aligned to the call's start.
What it would print is a refusal of the shape of the three above,
which names the unit and the verb and is a transport that died.
It is in the Open list.

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

That test is now `git_sha1::git_sha1_tests::the_digest_of_nothing_is_the_published_one`:
`refactor/cargo-shadow-cli-lib-rs-is-under-the-line-budget-with-room-for-the-modules-to-come` (2026-11-27)
moved its declaration from the crate root beside `src/git_sha1.rs`,
and the figure above is the run as it was measured then.

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
- A manager that died and left its files behind.
  The read keeps the asking again where a socket is there or `$XDG_RUNTIME_DIR/systemd` is,
  so a host whose manager is gone and whose socket file or runtime directory was not removed is still asked
  five times,
  which is 5 asks and about 0.86 s a call (measured 2026-11-27 for every state the README's `Development` lists).
  That is the safe direction:
  at the instant an ask fails,
  a socket file with nothing behind it and a socket file inside a reexec look the same,
  and refusing a reexec is the worse of the two mistakes.
  No measurement here covers how often a runtime directory outlives its manager,
  and this is the one way of having no manager that issue 6629's first request does not cover.
- A reexec window that opens between a refused start and the ask that follows it.
  The ask after the start is what tells the manager's answer from a transport that died,
  and a window inside that one program start would make a transport failure read as the manager's answer,
  which would refuse the call.
  The three asks of the start-failure shape measured above are what it would look like.
  A same-vendor second read of 2026-11-27 reached the state in 2 of its 24 calls
  whose reexec was aligned to the call's start,
  and the window a connect is refused in was 65 ms to 372 ms long in its containers,
  against a gap of one program start.
  A host whose `app.slice` is inactive and whose manager reexecutes at that instant is the state,
  and nothing here stages it.
- A call whose scope creation meets a reexec.
  The span is the discovery's,
  and no wait covers the scopes a call makes after it (above),
  so a reexec that begins after the discovery has answered fails the call in both releases.
  Either a scope's creation that meets a reexec is tried again,
  which needs a retry around `systemd-run` that can tell a transport that died from a scope that was made,
  or the failure stays as it is,
  which costs a failed call inside that window.
  No measurement here says how often a call meets one.
- Whether an empty `$XDG_RUNTIME_DIR` beside a live address should end the asking.
  systemd 262 treats an empty variable as a set one and refuses at once (`/systemd/private`),
  while the read names the address and keeps the asking,
  which spends the span on a host that could answer sooner.
  The direction is the safe one and the state is not measured here.
- The long help layout of `install --help` and `update --help`.
  clap prints it because `--dry-run`'s help has a second paragraph,
  and the README says so beside the `--dry-run` option;
  no record names whether the layout is wanted.
- The words of the ten refusals of 2026-11-27 stay unknown (issue 6628),
  so nothing here shows whether any of them was the no-manager case the read now stops at.
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
- The repository linter's `rust/max-lines` finding on `src/lib.rs` that this document named
  (305 code lines,
  limit 300) is gone:
  `src/lib.rs` holds 201 code lines on this branch's base,
  because a refactor before this round moved the round suites' declarations beside their modules.
  This round put one declaration in it,
  `control_group_route_one_ask`,
  which is the module the one ask moved to when this round's prose took `control_group_route_discovery.rs` past the limit.
- `control_group_route_discovery.rs` grew past the code-line limit with this round's comments,
  so the one ask,
  its failure and its bound rule moved to `control_group_route_one_ask.rs`,
  re-exported by the discovery;
  the controls' module paths did not change.
- A kept run is written when the run ends,
  not as it streams,
  so a task that is killed before its suite ends keeps nothing (above).
  Writing it as it streams would cost a file per run,
  including the runs that pass,
  and this round did not measure whether that is worth it.
