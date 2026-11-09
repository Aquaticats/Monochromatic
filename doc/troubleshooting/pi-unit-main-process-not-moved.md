# `wg-quicker-exempt` 0.0.1 with systemd 259.10: a `pi-<name>.service` agent's traffic used the tunnel although the toolchain path rule matched it

## Symptom

Every Pi agent of this machine ran in a user unit,
and every one of its HTTPS connections left through the WireGuard tunnel.
Measured on 2026-11-09 with
`ss --tcp --numeric --processes state established '( dport = :443 )'`:
the connections of the processes named `pi` held local address `172.17.170.170`,
the address of the tunnel `fr-par-fr1`,
while `helium` and `claude` held direct ones.

The watcher log said the process was skipped,
not that it was uncovered:

```console
wg-quicker-exempt: POLICY NOT IN FORCE: process watch: update entry points, toolchain install roots and Pi stay outside the tunnel: pid 40 matched but was not moved: it is the main process of the service `pi-wg-quicker-model-providers-off-the-tunnel.service`, and moving it would empty that service
```

## Root cause

The toolchain path rule already reached Pi,
so the match was never the gap:

- `package/wg/cli-quicker-exempt/src/update_entry_points.rs:classify` tests
  `is_under_any(exe, install_roots)` first,
  and returns `MatchReason::ToolchainPath`.
- A running Pi's `/proc/<pid>/exe` is the toolchain's node:
  `/var/home/user/.local/share/monochromatic-dev/toolchain/node/26.13.0/node-v26.13.0-linux-x64/bin/node`
  on this host,
  which lies below the install root the watcher accepts in both home spellings.

The move is what fails,
and the reason is the shape of the cgroup Pi runs in.
Each agent is a transient user service:

```console
0::/user.slice/user-1000.slice/user@1000.service/app.slice/pi-commit-scratch-removal-off-the-path.service
```

`src/routing_support.rs:adoptable_cgroup` refuses to move a process whose cgroup leaf ends in
`.service`,
because the service's main process must stay in the service's own cgroup:

```rust
    if leaf.ends_with(".service") {
        return Err(format!("it is the main process of the service {}, and moving it would empty that service", quote_name(leaf)));
    }
```

That refusal is deliberate,
and this branch keeps it.
Emptying a running service's cgroup is what the user's manager reads as the service having exited,
and a unit created by `systemd-run` carries `KillMode=control-group` by default,
so the rest of the unit's processes would be ended with it.

The consequence was the gap the owner measured:
the toolchain path rule matched,
every move was refused,
the process was recorded as handled,
and nothing else covered it.

## Verification

The reproduction is a unit test over a fake procfs tree,
so it needs no root and no live unit:
`update_routing_tests::a_process_in_a_pi_unit_is_covered_by_the_mark_attached_to_its_own_cgroup`
under `mise run //package/wg/cli-quicker-exempt:test:unit`.
With the fix removed it prints the refusal quoted above as `Outcome::Skipped`;
with the fix in place it prints `Outcome::Covered`.

Two more tests pin the neighbours:
`a_process_in_an_unmarked_service_is_still_reported_as_not_moved` keeps a service the watcher does not name loud,
and `a_pi_process_outside_a_marked_unit_is_moved` keeps a Pi started in an ordinary scope moving.

On the real host,
`package/wg/cli-quicker-exempt/target/release/wg-quicker-exempt list-targets 1000`
listed 20 `pi-*.service` units beside the terminals and desktop applications,
which is what the socket-marking scan now attaches to.

## Verified workarounds

Cover the cgroup instead of moving its main process:

- A cgroup of the user's `app.slice` whose name the watcher's own rules select carries the socket mark itself
  (`src/application_targets.rs:is_socket_marked_cgroup`).
  The kernel applies a cgroup socket-address program to every process in the cgroup and to every
  cgroup the user's manager creates below it,
  so the agent,
  its driver and every child it starts are marked without a move.
  Tradeoff:
  the mark covers the whole unit,
  so everything in it leaves the tunnel,
  not only the provider requests,
  which is the same tradeoff the terminals already carry.
- The mark applies to sockets created after the attachment,
  so a connection an agent already holds keeps the tunnel until it is re-established.
  Tradeoff:
  applying the change does not move live connections,
  and the watcher restart that attaches the mark is required.

## What does not work

- Moving the main process of the unit anyway.
  The manager would accept the attach and the unit's cgroup would empty,
  which its own service tracking reads as an exit.
- Moving only the unit's other processes and leaving the main one.
  The agent's own provider requests come from the main process,
  so its traffic would stay on the tunnel.
- Marking the login session instead.
  A `session-N.scope` holds every process the user started there,
  which widens the exemption far past Pi and the terminals' rule.

## Upstream filing decision

Nothing to file.
The behavior is systemd's architecture,
not a defect:
a service's main process belongs to the service's cgroup,
and this repository's boundary is the decision of what to move and what to mark,
which is where the fix landed.
The systemd details this repository relies on are recorded in
`package/wg/cli-quicker-exempt/README.md`
(`bus_unit_method_attach_processes`,
 delegation,
 and the empty-cgroup handling);
this doc cites only the code and the measurements made here.
