# wg-quicker

Bring WireGuard interfaces up and down with `wg-quick`-compatible lifecycle behavior,
without parsing large `AllowedIPs` values through Bash pattern matching.

```console
wg-quicker up wg0
wg-quicker down wg0
```

A bare interface name resolves to `/etc/wireguard/<name>.conf`.
An explicit `.conf` path is also accepted.
The interface name (the bare name,
or the file name without `.conf`) becomes part of commands,
file names and nft table names,
so it is checked at the command line,
before `sudo` is asked for and before anything is read:
1 to 15 characters (Linux's limit) from letters,
digits and `_ . -`,
not starting with `-` and not `.` or `..`.
Linux also allows `=` and `+`,
but `nft` takes neither in a table name,
so no block could be made for such an interface.
Any other name,
such as one with a space,
a quote,
a `;` or a `$(...)`,
is refused with an error that shows the name.
The route watcher reads an interface name back from the state file in the runtime directory
and puts it into commands that run as root,
such as `resolvectl dns <name>` and `wg show <name>`.
`WG_QUICKER_RUNTIME_DIRECTORY` does not cross `sudo` into the root run,
so the caller cannot choose that directory for it;
a disposable fixture sets the variable inside the root command.
The state file is still not trusted,
because whoever can write the directory can write the file:
the name in it must pass the same rule,
and a state file with another name is refused.
The routes in it are deleted as root,
and `ip` reads a keyword by any prefix and lets a later one overwrite an earlier one,
so each route must match the grammar the route writers use and nothing else:
`default` or `unreachable default`,
then `via <address>`,
`dev <name>`,
`onlink` and `metric <number>`,
and `table <claimed table>` and `proto 201`,
each at most once and in that one spelling.
The claimed table must be at least the first table the allocator hands out (52000).
A route copied by an earlier version with a preferred source or a router preference is read with those dropped;
any other route is refused.
The bypass table copies only those attributes of a physical default route.
It drops `src`,
`pref`,
`scope`,
`mtu`,
`hoplimit`,
`advmss`,
`linkdown` and `dead`.
That is not free:
an IPv6 default with `pref high` or `pref low` is copied as `medium`,
and the kernel ranks IPv6 defaults of the same metric by it;
a marked socket that is not bound to an address,
on an interface with several addresses,
gets the interface's primary source address,
not the preferred source the physical route named;
and the copy does not carry the path's MTU or hop limit.
A physical default route that carries any other attribute (for example `features ecn`,
`initcwnd`,
`realm`,
`tos`,
`nhid` or `nexthop`)
is outside the grammar and the dropped set.
It stops the application bypass with a `BypassRouteError` that names the route,
before any state changes:
`up` installs nothing,
and the route watcher leaves the bypass table as it is.
It fails closed.
A state file written by an older build that holds such a form fails to parse,
and the error names the state file.
The other places that read a name back do not trust it either:
the OpenSnitch state compares the name it holds with the one asked for and refuses a difference,
and every other name comes from the config,
which was checked.

When invoked as non-root,
`wg-quicker` relaunches exact current Node executable and CLI bundle through `sudo` before reading config.
Sudo inherits terminal streams for authentication and records original desktop identity in `SUDO_UID`.
This avoids both root-only config `EACCES` and sudo `secure_path` hiding workspace-local bin.

Default sudo `env_reset` drops custom settings and replaces `PATH`.
The launcher carries only allowlisted caller home,
cache,
token,
and runtime settings through a private mode-`0600` bounded file.
No setting that names the companion program,
the watched user or the caller's `PATH` crosses `sudo`,
because any of them would let the caller choose what runs as root in place of the process watch.
The root child validates file type,
ownership,
mode,
link count,
size,
schema,
and `SUDO_UID` before applying settings.
Consequently,
`AllowedIPsFromFiles = ~/...` still expands against caller home instead of `/root`.

Self-elevation intentionally authorizes current user-owned runtime,
bundle,
and the paired repository build of the companion for root execution;
use root-owned installed artifacts when workspace integrity is not trusted.

See
[sudo privilege troubleshooting](../../../doc/troubleshooting/sudo-1-9-secure-path-workspace-cli.md)
for source trace and verification.

## Config behavior

`wg-quicker` consumes wg-quick interface keys such as `Address`,
`DNS`,
`MTU`,
`Table`,
and lifecycle hooks.
It forwards WireGuard-native interface and peer lines to `wg addconf` through a private temporary file.
This keeps existing expanded `AllowedIPs` lines usable without Bash's superlinear trimming path.

## Automatic routing

For automatic `Table` behavior,
`wg-quicker` allocates one policy table and sets the WireGuard interface fwmark whenever at least one allowed
prefix exists.
Every allowed prefix goes in that table,
including non-default prefixes.
Main-table connected routes run first;
ordinary matching traffic then enters the policy table,
while WireGuard's marked outer UDP packets skip it and retain the physical main-table path.
This prevents a peer endpoint covered by its own `AllowedIPs` from routing recursively into the tunnel.
Automatic partial routes now use the same source-mark validation and nft ingress protection as automatic default
routes.

Separate automatic tunnels use separate policy tables.
When their allowed prefixes overlap,
policy-rule priority chooses the first table before longest-prefix matching can compare routes across tunnels.
Use explicit routing ownership when simultaneous overlapping automatic tunnels must preserve cross-interface
longest-prefix selection.

An explicit numeric `Table` continues to receive routes without automatic policy rules.
`Table = off` installs no routes.

Before creating an interface,
`wg-quicker` uses iproute2 JSON rule output to reject IVPN Desktop's exact split-tunnel rule
(`fwmark 0xca6c`,
 table `17`).
This fail-closed preflight applies in every table mode because externally marked application traffic can override
routing supplied outside automatic mode too.
IVPN inverse split tunneling marks ordinary non-DNS application traffic for a physical table and otherwise
silently bypasses custom WireGuard route selection.
Disable that conflicting feature before `up` and do not re-enable it while the tunnel is active;
the preflight is not a persistent rule monitor.
Customized IVPN mark or table constants are outside exact detection.
See
[IVPN routing troubleshooting](../../../doc/troubleshooting/ivpn-3-15-inverse-split-tunnel-overrides-wireguard.md)
for source trace,
diagnosis,
and verified recovery.

## OpenSnitch endpoint allowance

When OpenSnitch 1.8's system-firewall file exists,
`wg-quicker up` adds one enabled system rule for each distinct peer `Endpoint` UDP destination port.
The rule sits in OpenSnitch's existing `inet opensnitch mangle_output` chain before its NFQUEUE rules,
so WireGuard's kernel-generated outer connection works even when OpenSnitch uses `DefaultAction = deny`.
If OpenSnitch is not installed,
the integration is a no-op.

Each port rule accepts outbound UDP from any process to that destination port before application filtering.
`wg-quicker` emits a warning naming the ports,
interface,
and changed file whenever this policy widening is active.
The rules carry interface-and-network-namespace-specific `wg-quicker managed endpoint [...] [netns:...]`
descriptions and remain visible in OpenSnitch's system-firewall configuration.
Equal interface names in separate network namespaces retain independent rules,
locks,
and lifecycle manifests.
`down` and failed-`up` rollback remove only that interface's rules,
even when the WireGuard link is already absent.
Before config mutation,
`wg-quicker` stores exact config path and potential ports in private per-interface-and-namespace runtime state.
Cleanup uses that recorded path if `FwOptions.ConfigPath` later changes,
and clears runtime state only after config reconciliation and live-chain proof succeed.
Teardown waits until formerly owned exact port rules disappear from the live chain,
unless another retained OpenSnitch rule still accepts the same port.
A failed proof retains runtime state so a later `down` can retry cleanup.
This `/run` state survives an interrupted process but not a host reboot;
host-crash recovery is outside this lifecycle guarantee.

The integration supports OpenSnitch 1.8's nftables backend and version 1 system-firewall schema.
It rejects disabled system firewall,
unsupported schema,
missing `mangle_output` chain,
non-regular config paths,
and non-nftables backends instead of claiming strict-deny compatibility without an active rule.
OpenSnitch live reload receives one positional file write;
shorter JSON is padded with valid trailing whitespace so truncate does not emit a second reload event.
`wg-quicker` then requires consecutive healthy nftables observations in the daemon's network namespace.
When no daemon process exists,
it still inspects any surviving OpenSnitch table and refuses to forget a stale managed allowance.
Each required accept rule must precede the first NFQUEUE rule,
and each no-longer-owned exact port must be absent.
A missed watcher convergence triggers one bounded same-inode rewrite and another proof attempt;
startup fails and rolls back if that retry also fails.

Standard paths are `/etc/opensnitchd/default-config.json` and `/etc/opensnitchd/system-fw.json`.
Without a system-firewall override,
`wg-quicker` follows `FwOptions.ConfigPath` from the selected daemon config.
Custom deployments can set `WG_QUICKER_OPENSNITCH_DAEMON_CONFIG`;
`WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG` takes precedence over `FwOptions.ConfigPath` when both are set.
System-firewall paths
and daemon-config overrides must be absolute so separate lifecycle invocations cannot resolve different
files,
and so must `WG_QUICKER_RUNTIME_DIRECTORY` where it is set.
The OpenSnitch overrides cross the sudo boundary through the validated private caller-context file;
`WG_QUICKER_RUNTIME_DIRECTORY` does not.
The runtime `flock` serializes wg-quicker writers only;
avoid editing the same OpenSnitch system-firewall file concurrently through its UI.
See
[OpenSnitch troubleshooting](../../../doc/troubleshooting/opensnitch-1-8-wg-quicker-wireguard.md)
for source trace,
warning scope,
and disposable strict-deny verification.

## OpenSnitch toolchain allowance

OpenSnitch with an unattended prompt saves an exact-path `reject-always-simple-*` rule for any program that connects
while nobody answers.
A toolchain update then fails at the network level (`os error 103`) the first time a new version of
`node`,
`uv`,
`pnpm`,
`cargo-binstall`,
or a rustup toolchain connects.
`wg-quicker opensnitch-rules` installs rules that match the toolchain's directories by regular expression,
so a new version needs no prompt and no new rule:

```console
wg-quicker opensnitch-rules apply
wg-quicker opensnitch-rules status
wg-quicker opensnitch-rules remove
wg-quicker opensnitch-rules remove-superseded --dry-run
```

The command is independent of the tunnel and of `up` and `down`,
because allow rules must outlive the tunnel.
It relaunches itself through `sudo` like `up`,
since the daemon's rule files are mode `0600` and root-owned.

Three rules are written to the daemon's `Rules.Path` directory (`/etc/opensnitchd/rules` by default),
each matching `process.path`:

- `000-allow-monochromatic-toolchain-install-roots` covers everything below `monochromatic-dev/toolchain/` and
  `monochromatic-dev/toolchain-global/` in the XDG data home,
  where `mpm-shadow` and the TypeScript sync install every tool version.
- `000-allow-monochromatic-toolchain-exposure-links` covers files directly in `.cache/toolchain-bin/` and
  `.cache/toolchain-launchers/` of the checkout and of each `.claude/worktrees/<id>`.
- `000-allow-monochromatic-toolchain-rust-tools` covers `cargo`,
  `cargo-binstall` and `rustup` in the Cargo home `bin` directory,
  and every binary in `<rustup home>/toolchains/<toolchain>/bin/`.

Every expression is anchored,
so a sibling such as `toolchain-evil/` or a root appearing in the middle of a path never matches.
The home directory appears in each spelling a process path can take:
as given,
as `realpath` resolves it,
and with `/home` and `/var/home` swapped when `/home` links to `/var/home`.
No user name is written into the code;
the home comes from `HOME`,
which the privilege context carries across `sudo`.
Pass `--data-home`,
`--cargo-home`,
`--rustup-home` or `--repository-root` when a directory is not at its default,
because `sudo` resets `XDG_DATA_HOME`,
`CARGO_HOME` and `RUSTUP_HOME`.
Run the command as the user:
a direct `sudo wg-quicker opensnitch-rules ...` carries no caller context and leaves root's `HOME`,
so the command refuses unless all three homes are passed.
Read the patterns that `apply` prints and confirm they name your home.
Without `--repository-root` the checkout is the nearest ancestor of the running bundle holding `toolchain.lock.json`,
with a trailing `/.claude/worktrees/<id>` dropped.

### Why these rules win over an auto-saved reject

OpenSnitch 1.8.0 selects a rule in `daemon/rule/loader.go`:
`sortRules` sorts enabled rule names with a bytewise string sort,
and `FindFirstMatch` walks that order,
returns at once on a matching reject,
deny or `precedence` rule,
and otherwise returns the last matching rule.
An allow rule without `precedence` therefore never beats a matching reject,
whatever its name,
and a precedence allow named after the reject loses to it.
The generated rules set `"precedence": true` and start with `000-`,
which sorts before the daemon's `reject-always-simple-*`,
`deny-always-simple-*` and `allow-always-simple-*` names,
so they are reached first and end the walk.
A rule sorting before them that is a reject,
deny or precedence rule would still win;
`status` lists any such rule.
The cost is that a destination-based deny rule no longer applies to these paths.

The patterns set `"sensitive": true`,
because the daemon lowercases the pattern of a case-insensitive rule before compiling it,
which would turn `[A-Z]` or `\S` into a different expression.
Patterns are RE2,
with no lookaround or backreference.
A pattern that fails to compile is logged as a warning and the rule is ignored,
so grep `/var/log/opensnitchd.log` for `compil` after applying.

### Live reload

The daemon reloads a rule only on a filesystem `Write` event and unloads one only on `Remove`.
A file renamed into place raises neither,
so `apply` creates the final `<name>.json` and writes it in place.
An existing file is rewritten at offset zero through the same inode and padded with spaces to its old length,
so a shorter replacement never exposes a truncated document.
The file name equals the rule name,
which the daemon relies on to unload a deleted rule.
Byte-identical files are left alone,
and a same-named file that `wg-quicker` did not write stops the run.

### Superseded reject rules

`remove-superseded` finds enabled `reject` and `deny` rules that compare one exact `process.path` the allowance covers.
Those rules are harmless once the allowance exists,
so nothing removes them unless asked.
With `--dry-run` it only lists them.
Otherwise it copies each file to `--backup-directory` (default `rules-superseded-backup` beside the rules directory,
outside the watched path),
refuses to overwrite an earlier backup,
and then unlinks the original so the daemon sees the `Remove`.
Rules of type `list`,
and rules comparing a regular expression,
are never touched.

See
[OpenSnitch auto-reject troubleshooting](../../../doc/troubleshooting/opensnitch-1-8-auto-reject-blocks-toolchain-updates.md)
for the source trace and the verification.

## Generate AllowedIPs from files

A peer may replace literal `AllowedIPs` with `AllowedIPsFromFiles`:

```ini
[Peer]
PublicKey = peer-public-key
AllowedIPsFromFiles = ~/allowed.txt ~/disallowed.txt
Endpoint = vpn.example:51820
```

The directive must occur inside its owning `[Peer]` section.
Each peer may contain either one literal `AllowedIPs` value or one `AllowedIPsFromFiles` directive,
not both.
Multiple peers can use independent source files.
The generated line is inserted where the directive occurred.

Each source file uses the format documented by `package/wg/cli-allowedips/README.md`.
Generation comes from `@monochromatic-dev/wg-core-allowedips` through a static source import.
ASN refreshes use `IPINFO_TOKEN` and the cache directory described by that module.

`down` parses source directives but does not read their files,
resolve domains,
or refresh ASN data.
Only `up` generates peer prefixes.

## Application exemptions

`ExemptMark = <positive integer>` enables socket-mark policy routing used by the Rust
`wg-quicker-exempt` companion.
When `up` does not find this key,
it warns that Ghostty,
 Steam,
 Helium,
 Pale Moon,
 Firefox release,
 Firefox Nightly,
 ChatGPT,
 Interpreter,
 Qure,
 OdyTTY and Pi will use the tunnel and instructs the user to add
`ExemptMark = 100` under `[Interface]`,
then bring the interface down and up again so application exemptions attach.
The warning is non-fatal;
tunnel activation continues.

Config parsing rejects an `ExemptMark` sharing any bit with a mask another
subsystem matches on,
listed in `src/exempt-mark-reservation.ts`.
`0x2000` is netavark's `MASK`:
its `table inet netavark` postrouting rule
`meta mark & 0x2000 == 0x2000 masquerade` source-NATs every matching packet,
including IPv4 loopback,
so a server bound to `127.0.0.1` reads the outbound interface address as its peer.
The earlier recommended value `8888` is `0x22b8` and carries that bit;
`doc/troubleshooting/netavark-masquerade-mask-exempt-mark.md` records the symptom it produced.
The companion attaches cgroup-BPF programs to selected application cgroups.
The tunnel lifecycle does not move Ghostty into another systemd slice.

Before network mutation,
`wg-quicker` resolves `wg-quicker-exempt` to exact executable path.
It looks only in root-controlled locations:
the paired repository release or debug build,
then the absolute directories of the privileged `PATH`,
which under `sudo` is its `secure_path`.
The caller's own `PATH` is never searched,
and a relative or empty `PATH` entry is skipped because it means the current directory.
When nothing is found and the config sets `ExemptMark`,
`up` does not search further and does not go on:
it throws a `WatcherMissingError` that names every location it searched and says how to install the program,
and the failure runs the same rollback as any other failed step of the up,
so the tunnel is not left up and traffic goes direct.
The owner chose that,
see "A missing watcher stops `up`" below.
A config without `ExemptMark` never looks for the program.
`down` reports instead of failing when the companion is missing,
so tearing the tunnel down is never blocked;
it says that a watcher an earlier run started may still run.
No environment variable names the executable or the watched user:
`WG_QUICKER_EXEMPT_COMMAND` and `WG_QUICKER_EXEMPT_UID` were removed,
because setting either before `sudo` pointed the process watch at a stand-in program or at another user,
which is a switch for a policy that has none.
The watched desktop UID comes from `SUDO_UID`,
 then the non-root effective UID.
Direct root execution without a sudo identity fails instead of watching root's app slice.

The Rust watcher installs inotify on user's existing `app.slice` before its first scan,
attaches the Ghostty service,
every `app-ghostty-surface-transient-*.scope`,
Steam's `app-steam@*.service`,
Firefox release's `app-firefox@*.service`,
Firefox Nightly's `app-firefox\x2dnightly@*.service`,
ChatGPT's `app-chatgpt@*.service`,
Interpreter's `app-interpreter@*.service`,
Qure's `app-qure@*.service`,
OdyTTY's `app-odytty@*.service`,
and every `pi-<name>.service` Pi agent unit,
drains queued events,
and scans again to close creation race.
It reacts to future cgroup creation and periodically maps every live Helium executable,
including renderer,
zygote,
and crashpad processes,
both Pale Moon executable names,
and `firefox` and `firefox-bin` executables under the exact `firefox` or `firefox-nightly` install directories
back to their current cgroups.
Firefox ESR's `firefox-esr` install directory and desktop service remain tunnel-routed.
The same periodic scan maps every ChatGPT executable inside the root-owned `/usr/lib/chatgpt` package tree,
so the `codex`,
`rg`,
and `node` agents that application spawns keep their bypass even in a cgroup of their own.
It also maps every Interpreter executable whose name begins with `interpreter`,
covering the AppImage file,
the mounted Electron image,
the bundled `interpreter-*` agents,
and the same vendor's terminal agent.
The same periodic scan maps every Qure executable whose name begins with `qure`,
covering the AppImage file,
the mounted Electron image,
the bundled pytest runner,
and the agents its bundled CLI wrappers exec.
It also maps every OdyTTY executable whose name begins with `odytty`,
covering the AppImage file and the native image it runs.
Known process-discovered cgroups remain attached until directory disappears,
covering process restarts inside same service or scope.
Process discovery attaches entire current cgroup,
so sibling processes in a shared cgroup also bypass tunnel until cgroup disappears or watcher stops.
A newly started Helium,
Pale Moon,
Firefox release,
Firefox Nightly,
ChatGPT,
Interpreter,
Qure,
or OdyTTY process can create sockets before next 250-millisecond rescan;
applications already running during watcher startup are attached before readiness.
Watcher state validates PID,
process start time,
and complete command before shutdown.

The watcher also moves update entry points into the toolchain updates slice,
`app-monochromatic\x2dupdates.slice`,
and marks that slice.
See [Update traffic policy](#update-traffic-policy).

On `up`,
 application bypass routing:

- claims a free route table after checking IPv4 and IPv6 routes and rules;
- chooses a free positive rule preference that runs before every existing positive-priority rule;
- holds per-interface and global allocation `flock` locks under `/run/wg-quicker`
  until routes and rules make ownership visible;
- copies each physical main-table default into the claimed table;
- installs an unreachable default for a family with no physical default,
  preventing marked traffic from falling through to tunnel policy;
- tags owned rules and routes with route protocol `201`;
- persists interface name,
  mark,
  table,
  preference,
  ownership token,
  and exact kernel-rendered route fingerprints in a root-owned state file;
- starts a detached route watcher in the caller's privilege and network namespace.

The watcher resynchronizes owned defaults after DHCP,
IPv6 router-advertisement,
and roaming changes.
It starts route monitoring before synchronization so changes are queued across that boundary,
and restarts an unexpectedly exited monitor child.
A state-owner token,
PID,
process start time,
nonempty executable argument,
and exact watcher-script and state arguments prevent signaling a reused or unrelated PID during teardown.
The executable installation path may differ after a Node runtime update.
Shutdown signals the validated detached process group
so monitor and in-flight route-command children terminate before ownership release.
See
[`doc/troubleshooting/node-runtime-watcher-upgrade.md`](doc/troubleshooting/node-runtime-watcher-upgrade.md)
for the diagnosed upgrade failure and verified recovery.

On `down`,
 persisted state identifies the exact watcher,
rules,
and routes to remove.
Teardown deletes only persisted route fingerprints,
not every route carrying protocol `201`.
It does not flush whole tables and does not depend on current config values,
so unrelated routes and configuration edits made after `up` are preserved.
Synchronization refuses a table containing an unrecorded default before mutation.

## An interrupted run, and how long a command may take

`up` installs handlers for `SIGINT` and `SIGTERM` before its first step and removes them when it ends.
A signal therefore stops the up between two steps,
runs the same rollback a failing step runs,
and reports the signal and the step that did not run:

```text
The up was interrupted by SIGINT before it could install the routes and rules, so it is rolled back: what this up already created is being removed.
```

Without those handlers the signal's default action ends the process at once,
before any rollback,
and whatever the up had already created stayed behind.
The decision is the owner's:
"nothing should ever wait forever or indefinitely",
and an interrupt that leaves a half-configured tunnel is the same failure in the other direction.
A second signal stops at once and abandons the rollback,
because a person who asks twice wants the process gone.
`down` keeps the signal's default action:
its order already blocks traffic before it removes routing,
so a `down` that is killed part way leaves traffic blocked and not outside the tunnel.

Every command `wg-quicker` runs itself ends at a deadline:
`ip`, `wg`, `nft`,
`resolvectl` and `sysctl` get sixty seconds,
and a command still running then is killed and reported as
`CommandTimeoutError`,
naming the command and the limit.
Sixty seconds is a stall detector and not a target:
measured on the host of the review inside a disposable network namespace,
the slowest call was `ip link delete` at 52 ms,
and `wg addconf` with a peer of 5000 prefixes took 5 ms.
A hook is the user's own command and `wg-quick` waits for it,
so a hook gets ten minutes,
which is where it has certainly stopped making progress;
the error names the hook and the limit,
so a hook that legitimately needs longer can be changed.

## Taking the tunnel down

`wg-quicker down` runs the steps that can fail before it removes any routing.
This is the owner's decision,
the option "Keep the tunnel up":
"Run every step that can fail (stopping watchers,
reading state) before touching routing;
if one fails,
stop with the tunnel fully up and a loud error saying what to do."
The accepted cost is that "a later step failing (for example deleting the interface) is still possible;
that case falls back to blocking".
The owner also answered no to a separate `wg-quicker restart` command,
so there is none.

The order is:

- Stop the route watcher.
  Reading the persisted state and checking the watcher's identity happen here.
- Stop the application watcher of `wg-quicker-exempt`.
  If this fails,
  the route watcher is started again.
  A missing `wg-quicker-exempt` program is a failure too:
  a watcher started earlier may still run,
  so `down` stops with the tunnel fully up and the error names the places the program was looked for.
- Install the traffic block (an nft table,
  described next).
  If this fails,
  nothing was removed,
  the route watcher is started again,
  and `down` stops with the tunnel fully up;
  the application watcher stays stopped,
  and the error says so.
- Remove the routing:
  the policy rules,
  the application-bypass routes,
  rules and state,
  the kill switch,
  and the interface.
  The interface delete is verified:
  an interface that is still there after `ip link delete` is a failure.
- Lift the block as soon as the interface is verified gone.
- Revert the DNS settings and remove the OpenSnitch allowance.
  The tunnel is gone by then,
  so a failure here exposes nothing and is reported without a block.

If a step before the routing removal fails,
nothing was removed and the tunnel is still fully up:

```text
error: could not stop the route watcher (<reason>)
the tunnel fr-par-fr1 is still fully up; nothing was removed
fix: <what to do>, then run `wg-quicker down fr-par-fr1` again
```

A `down` of an interface that is already gone does the removals that do not need the link,
because another tool can delete a link while the application bypass is active.
The bypass routes,
rules and state,
the route watcher,
the kill switch and the OpenSnitch allowance all outlive the link,
and leaving them kept host rules selecting the physical bypass table and a watcher refreshing them.
Only two steps read the link and are skipped without it:
removing the policy rules,
whose table comes from the interface's own fwmark,
and deleting the interface.
The traffic block is removed last,
and the command still reports the missing interface,
and any cleanup step that failed,
so an ordinary typo is still an error and a real leftover is named.

The block is installed before any routing is removed and lifted after the interface is verified gone,
so traffic cannot go outside the tunnel from a tunnel that looks up at any point of the removal:
not when a step fails,
and not when `down` is killed or crashes part way,
because the block is an nft table in the kernel and stays until it is removed.
It is an nft table of its own,
`inet wg-quicker-<interface>-blocked`,
that drops everything forwarded and everything leaving through an interface other than loopback,
except the clearnet traffic described under the heading "Clearnet traffic still flows".
The tunnel's kill switch (`inet wg-quicker-<interface>`) is a different table and only drops spoofed packets addressed to the tunnel's own addresses;
it blocks nothing outbound.
The rest of the network does not work while the block is in place,
which is a few moments in a `down` that succeeds.
Installing the block is one nft transaction that creates the table when it is absent,
empties it when it is present and fills it,
so a `down` that is run again after a failed one,
with its routing already gone,
never has a moment without the block.
`mise run //package/wg/cli-quicker:test:integration:block` shows it with the real `nft` in a disposable user and network namespace,
which needs no privilege:
replacing a block in place is one transaction,
where deleting the table and installing it again is two,
with the table gone between them.

### Clearnet traffic still flows

The owner decided on 2026-11-01:

> `down` doesn't have to block all traffic during `down`.
> It can avoid blocking already-clearnet ones.

So the block does not drop what the routing already sends around the tunnel by design,
while `down` runs and while a block left by a failed `down` stands,
within the limit on new sockets that this section states.
It reads that from what the routing itself matches on,
and adds no rule of its own:

- A packet carrying the exempt mark (`ExemptMark`) passes.
  This is exactly what the policy rule `fwmark <mark> table <bypass table>` matches,
  with no mask,
  so the block matches the whole mark too.
  The cgroup programs of the updates slice and of the exempted applications set the mark on their sockets
  (`connect4`,
  `connect6`,
  `udp4_sendmsg` and `udp6_sendmsg`).
- A packet to an update host passes.
  These are the destinations of the `to <address> table <bypass table>` rules the route watcher keeps,
  read from `ip rule show` at the time the block is installed.
  It passes in the output chain,
  for this machine's own traffic,
  and in the forward chain,
  for guests behind this machine such as containers and virtual machines.
  The owner chose the forward chain on 2026-11-01 (the option "Pass forwarded update hosts"):
  "Add the same update-host destination rules to the forward chain",
  so that "guests can reach those hosts directly during the block".
  A guest's connection needs both directions through the forward chain,
  so the forward chain also passes a packet from an update host that is a reply,
  matched on the source address and on the direction of the connection:
  `ct direction reply ip saddr` and `ct direction reply ip6 saddr` rules with the same addresses.
  A rule on the destination alone passes the request and drops every reply,
  which was found by a review that connected a guest to a host through a router:
  with the rules for the destination only,
  the request was accepted and the answer dropped,
  and the connect failed.
  A rule on the source address alone passed any forwarded packet that carried an update host's address as its source,
  whoever sent it,
  which a second review showed in the same trio:
  a guest that owns such an address could send a packet from it to an address that is not an update host,
  and the block accepted it.
  `ct direction reply` closes that.
  Only a packet that the connection tracker sees as the reply direction of a flow passes,
  and only from an update host:
  a packet a guest sends is the original direction of its flow,
  and so is a packet from an update host that starts a flow,
  so neither passes any more.
  Two forged packets from a guest can still make a "reply" in the tracker:
  one that opens a flow towards the update host's address,
  and one that carries that address as its source.
  The reply rules therefore also match `iifname` of the physical default interfaces of their address family,
  where a real reply arrives and a guest's packet does not.
  The interfaces are the `dev` of the routes that the bypass table copies from the main table,
  read from the ownership state when the block is installed.
  A family with no physical default interface gets no reply rule.
  The rules do not match the state of the connection (`ct state established,related`),
  because that would also pass the guests' flows that were bound for the tunnel once the routing is gone.
  The rule needs the connection tracker,
  and nft turns it on for the namespace when the rule is loaded.
  A flow that was already running when the block was installed is oriented by the first packet the tracker sees of it,
  so if that is a reply,
  the flow's direction is taken the other way and the update host's later packets of that flow are dropped;
  this is not verified on the real host,
  where the tracker is already running when other firewall rules use it.
  This machine's own output chain needs no such rule:
  the replies to its own packets arrive through the input hook,
  which the block does not filter.
- Loopback passes,
  as before.
- Everything else is dropped:
  an unmarked packet,
  a packet with any other mark,
  and every forwarded packet that is not to an update host.
  The mark has no rule in the forward chain,
  because it is set on the sockets of this machine and a forwarded packet comes from no socket of this machine.

The application watcher is stopped before the block is installed,
and stopping it closes the marker links it holds for the cgroups it watches
(`MarkerLinks` in `package/wg/cli-quicker-exempt/src/bpf.rs`:
dropping it detaches the marker).
From then on a socket that the updates slice or an exempted application opens is no longer given the mark.
So the mark exception covers the sockets that were marked before the watcher stopped,
such as connections that are already open,
and the update-host destinations cover the rest of what the slice reaches by design.
A new connection of the slice to any other destination is dropped for as long as the block stands,
which is a few moments in a `down` that succeeds and until the next `down` finishes after a failed one.
This follows from the code and was not run against the real watcher.
The owner accepted this gap on 2026-11-01 (the option "Accept the gap"):
the order stays as it is,
with the application watcher stopped before the block is installed.

These rules stand ahead of the drop of their chain,
in the same nft transaction that installs the block.
The block is built from the ownership state in `/run/wg-quicker`,
which is still there when the block is installed.
An interface without an application bypass has nothing to let through,
and its block drops everything outbound as before.
If the state or the rules cannot be read,
`down` stops before it changes anything,
with the tunnel fully up and an error that says what could not be read,
because a block built without that knowledge would cut traffic the owner wants to keep.
The owner chose this on 2026-11-01 (the option "Keep refusing + manual steps"):
"Keep the refusal,
and have the agent add exact manual steps to its message
(which state file to inspect or remove,
and the `ip link delete` and `nft` commands)."
So the refusal ends with lines of exact commands,
worked out on the machine that runs the `down`:

```text
error: could not install the traffic block that protects the removal of the routing (ClearnetUnreadableError: could not read what the routing sends around the tunnel (<reason>), which the traffic block lets through; a block built without it would cut that traffic)
the tunnel fr-par-fr1 is still fully up; no routing was removed; <what happened to the watchers>
fix: fix what the error names (...), then run `wg-quicker down fr-par-fr1` again
to look at what could not be read:
  sudo cat /run/wg-quicker/interface-<key>.json
  sudo ip -4 rule show
  sudo ip -6 rule show
first stop the route watcher, whichever way you go on: without the state file wg-quicker no longer looks for it, and a watcher left running adds rules again:
  sudo cat /run/wg-quicker/interface-<key>.json.watcher.json
  # the pid is 2285916; the next command must print a line that ends with /run/wg-quicker/interface-<key>.json; any other output, or none, means 2285916 is not the watcher any more, so do not run the kill after it
  ps -o args= -p 2285916
  sudo kill 2285916
then, if the state file is damaged, move it away and run `wg-quicker down fr-par-fr1` again; ...:
  sudo mv /run/wg-quicker/interface-<key>.json /run/wg-quicker/interface-<key>.json.damaged
or take the tunnel down by hand instead, with the route watcher stopped as above, and run these in order (each is what wg-quicker itself runs, except the first and the last; none needs the state file). The first puts the traffic block in place in one nft transaction, as wg-quicker does before it removes any routing, so that a step that fails leaves traffic blocked and not outside the tunnel; the last lifts it:
  # no exceptions could be read, so this block lets nothing through except loopback, the updates slice included, until the last command of this list lifts it
  sudo nft 'add table inet wg-quicker-fr-par-fr1-blocked; flush table inet wg-quicker-fr-par-fr1-blocked; add chain inet wg-quicker-fr-par-fr1-blocked output { type filter hook output priority -400; policy accept; }; add rule inet wg-quicker-fr-par-fr1-blocked output oifname != "lo" counter drop; add chain inet wg-quicker-fr-par-fr1-blocked forward { type filter hook forward priority -400; policy accept; }; add rule inet wg-quicker-fr-par-fr1-blocked forward counter drop'
  sudo <path of wg-quicker-exempt> watch-stop fr-par-fr1
  sudo ip -4 rule delete table <policy table>
  sudo ip -6 rule delete table <policy table>
  sudo ip -4 rule delete table main suppress_prefixlength 0
  sudo ip -6 rule delete table main suppress_prefixlength 0
  sudo nft delete table inet wg-quicker-fr-par-fr1
  sudo ip link delete dev fr-par-fr1
  # only when the config sets DNS
  sudo resolvectl revert fr-par-fr1
...the bypass rules and routes that carry protocol 201 (the message says how to find and remove them)
  sudo rm --force /run/wg-quicker/interface-<key>.json /run/wg-quicker/interface-<key>.json.watcher.json
  # lift the traffic block; the network works again
  sudo nft delete table inet wg-quicker-fr-par-fr1-blocked
```

The route watcher is stopped before anything else,
for both ways on,
because `stopRouteWatcher` reads the state file and returns at once when it is absent:
a `down` that is run after the damaged state file was moved away does not find the watcher,
which keeps running and adds its rules again.
The watcher is told apart from other processes by the record next to the state file,
so the record is shown first and the process is stopped before the state file is touched.
The process ID is read from that record when it can be read,
and put into the commands.
A bare number can be reused by another program after the watcher has died,
so the kill is preceded by `ps -o args= -p <pid>`,
which shows what runs under the number now,
and a remark says what it must show (a command that ends with the state file path,
which is the last word of the watcher's command line)
and not to kill otherwise.
When the record cannot be read,
the check and the kill stand in a remark with `<pid>` instead of as command lines,
because a command with a placeholder cannot be pasted whole.

The by-hand list removes routing,
so it opens with the traffic block and ends by lifting it.
Without the block a step that failed part way would leave the traffic going outside the tunnel,
which is what the order of `down` exists to prevent.
The first command is the same statements as the script that `wg-quicker` applies,
joined by semicolons into the one argument that `nft` applies as a single transaction.
It carries the exceptions for the clearnet traffic when they could be read just now
(the refusal is usually the failure to read them,
so often they cannot),
and the remark above it says which.
`mise run //package/wg/cli-quicker:test:integration:block` installs the block both ways in a disposable namespace
and requires the same table and a single nft transaction from the printed form.


The message names the real state file path (the key is a hash of the interface name,
and `WG_QUICKER_RUNTIME_DIRECTORY` moves the directory),
the real path of `wg-quicker-exempt` that `wg-quicker` resolved,
and the real policy table,
read from `wg show <interface> fwmark`;
where a value cannot be read,
the line says where to read it instead of inventing one.
Every value that goes into a command is shell-quoted,
so a path with a space,
a quote,
a `;`,
a `$(...)` or a newline is still one word when the line is pasted into bash,
zsh or ksh,
and the printed lines hold no control character or line break (a control character is written as an escape in the `$'...'` form).
Remarks stand on lines of their own that begin with `#`,
so that every command line can be pasted whole.
Each command is the one `wg-quicker` itself runs for that step,
and none needs the state file.

A `down` that is run again after a failed one may find that the failed one already removed the ownership state.
The block of the earlier attempt is then kept as it is,
with the exceptions it was installed with,
and not replaced by one that has none.
Sockets that carry the mark keep passing during that window,
because the kept block still lets them through.
The kill switch (`inet wg-quicker-<interface>`) has chains only in the prerouting and postrouting hooks (`src/tunnel-firewall.ts`),
so it has nothing that drops marked output packets and does not change this.

An unprivileged process cannot give its packets the mark.
The kernel refuses `SO_MARK` without `CAP_NET_RAW` or `CAP_NET_ADMIN`
in the user namespace of the socket's network namespace
(`sk_setsockopt` and `sock_cmsg_send` in `net/core/sock.c` of Linux v7.2.11,
lines 1527 to 1532 and 3041 to 3044).
A process in a user and network namespace of its own can mark its own sockets there,
but its packets reach this host through the forward hook,
which has no mark rule,
so the mark gains it nothing:
it reaches the update hosts only,
like any guest.
`mise run //package/wg/cli-quicker:test:integration:block` shows all of this with the real `nft`
in a disposable user and network namespace:
a packet with the exempt mark and a packet to an update host are counted by their accept rules,
a packet with the mark one higher,
an unmarked packet and an unmarked packet from a process
without `CAP_NET_RAW` and `CAP_NET_ADMIN` (whose attempt to set the mark fails with `EPERM`) are counted by the drop,
loopback is counted by none,
and the accept rules stand ahead of the drop.
A second scenario builds a router,
a guest and an outside from three network namespaces and forwards the guest's packets:
an echo request to an update host and its reply are counted by the forward chain's accept rules,
and any other forwarded packet by its drop,
for IPv4 and for IPv6.
A packet that the guest sends with an update host's address as its source,
after giving itself that address,
is dropped,
as well for IPv6;
with the rule on the source address alone,
as an earlier build had it,
that packet is accepted,
and the test fails.
A TCP connect from the guest to an update host is made with the block as it is installed.
With the rules for the replies taken out of the table by handle,
which is the block as it was before they existed,
the same connect fails,
the request counted by its accept rule and the answer by the drop.
Installing the block again puts the rules back,
and the connect is made.
A connect to another address of the outside,
where the same listener would answer,
still fails.
A packet of the router itself to an IPv6 update host is counted by the output chain's accept rule too.

### Failure messages

If a step of the routing removal fails,
the error says the block is in place:

```text
error: down fr-par-fr1 failed part way: could not delete the interface (<reason>)
traffic is BLOCKED (nft table inet wg-quicker-fr-par-fr1-blocked drops everything that leaves the machine except the clearnet traffic it was installed to let through (traffic with the exempt mark, which the updates slice and exempted applications use, and traffic to the update hosts, including forwarded traffic to them, when the interface has an application bypass), and was installed before any routing was removed), so nothing goes outside the tunnel unprotected except what the routing sends around it by design, and the rest of the network will not work until down is finished
to finish: run `wg-quicker down fr-par-fr1` again, with this build or with the build that started the tunnel; a down that succeeds removes the block
to lift the block by hand: sudo nft delete table inet wg-quicker-fr-par-fr1-blocked
```

A block that cannot be lifted after the interface is gone is the first problem the error names,
with the command that lifts it.
A `down` that is run again after a failed one finds the block of the earlier attempt,
and a refusal in its first steps says that the earlier `down` already removed part of the routing,
that the tunnel is not fully up and that traffic is blocked,
where it would otherwise say that nothing was removed.
A successful `down`,
a `down` of an interface that is already gone,
and an `up` (which refuses an existing interface) remove a leftover block,
and each of them shows that it is gone:
when the table is still there after the delete,
or cannot be shown gone,
they stop with an error that names the table and the command that removes it,
`sudo nft delete table inet wg-quicker-<interface>-blocked`.
The table counts as gone only when `nft --json list tables` exits with status zero and its listing has no `inet` table of that name;
a failed listing,
such as one run without privilege,
and output that is not a listing show nothing.
No error text is matched,
because it is the C library's description of an errno and not an interface of `nft`.
`up` stops before it changes anything,
because a tunnel brought up behind the block would carry nothing,
and a `down` of an interface that is already gone fails even though it tolerates a missing interface,
because the network stays blocked.
The rollback of a failed `up` does not use this order and does not block.

A program that cannot be started fails its step with a `CommandSpawnError` that names the program and the system's reason,
for example a host without `wg`:

```text
Command could not be started: wg addconf /tmp/wg-quicker-XXXX/addconf.conf
spawn wg ENOENT
```

The step fails like any other,
so the rollback of the `up` runs and the tunnel is not left half configured.
A program that never ran is never reported as an exit code,
which is why `ip` and its companions cannot use the "allowed to fail" path for a missing program.

### Upgrading a running tunnel

With a build that has both fixes (commits `cf48274391` and `ab046a3a97` or later),
take the tunnel down with that build and bring it up with that build,
whichever build started the tunnel.
The persisted state,
the watcher record and the state path have the same format in every build so far,
so the newer `down` reads what the older `up` wrote.
It stops the watcher without taking an exiting watcher for another command,
and if a step that can fail fails first,
the tunnel stays up and the error says what to do.

The watcher check compares the exact path of the watcher script.
A rebuild in place keeps that path;
a `down` from a copy of the CLI at another path is refused,
and the refusal names the command the watcher runs and the command it expected.

A build before those fixes behaves differently,
and that is the case of the incident of 2026-10-31
(see [the incident](doc/troubleshooting/node-runtime-watcher-upgrade.md)).
Its `down` can fail with `Refusing to signal PID ... because command is not bypass watcher` while the signaled watcher is still exiting:
a signaled process reads an empty command line before it is a zombie,
and the check took that for another command.
The older order then left the interface up with the routing already removed,
which sent all traffic outside the tunnel for 92 seconds in the incident.
The earlier advice,
to take the tunnel down with the build that started it and then bring it up with the new one,
was the workaround for this before the cause was known.
It does not avoid the race,
because the build that started the tunnel has it too.
So for a tunnel that is still run by a build without the fixes:
if `down` reports the refusal,
run `wg-quicker down` again at once,
since the watcher is gone by then,
and then bring the tunnel up with the build that has the fixes.

## Update traffic policy

The wg packages alone decide that update traffic leaves outside the tunnel.
The toolchain (`package/file-enforcer` and `package/mpm-shadow`) knows nothing about the tunnel and never places itself.
The same packages keep the API hosts of Pi's model providers outside the tunnel,
and every Pi process with them,
for the owner's reason in this section:
the agents' work is this machine's own,
and the tunnel's exit is not its path.
The user's words,
verbatim:

> mpm-shadow should never determine itself goes through clearnet.
> Our wg set of pkgs should determine that!

> Anyone using our wg set of tools should be warned on start and the behavior shouldn't have a toggle.
> It should always enforce our policies,
> even clearnet.

The first words of the user's second note explain why the start notice is loud:

> this case was exceptionally bad,
> because it would have had privacy implications and outright break people living in exceptionally ridiculous networking environments.
> Imagine somewhere where if some people detect you're accessing crates.io you're paid a visit.

### Two configs that run without the policies, loudly

Two configs bring the tunnel up with neither policy in force.
Both are deliberate decisions of the owner,
recorded here so a later review does not mistake them for a gap.

- A config without `ExemptMark`.
  The owner's words,
  verbatim:

  > Warn if ExemptMark is unset but continue to up,
  > because sometimes for some configs we want no exemptions.

  So `up` prints a start report that names both policies as not in force and continues.
  This is not a toggle of the policies:
  no setting turns a method off for a config that has a mark,
  and a config without a mark gets the report on every start.
- A config with `Table = off`.
  The owner chose to keep it loud and not in force:
  wg-quicker installs no routing rule and no bypass table,
  so there is nothing for the policies to act on,
  and every start prints `POLICY NOT IN FORCE` naming `Table = off` as the reason.
  `up` continues here too.

`run-subcommand.unit.test.ts` checks that `up` runs after the report in both cases,
that the report comes first,
and that `down` prints no start report.

### A missing watcher stops `up`

A config that sets `ExemptMark` needs `wg-quicker-exempt`,
which runs the process watch and marks the sockets.
When the program cannot be found in a root-controlled location,
the owner chose "Abort up as before".
The owner's words,
verbatim:

> `up` fails and rolls back,
as before this branch.

So `up` throws a `WatcherMissingError` from the exemption step,
the rollback of `up` removes what the earlier steps did,
and the message names the locations searched,
the build command `mise run //package/wg/cli-quicker-exempt:build`,
and the option of installing the program where sudo's `secure_path` finds it.
The caller's own `PATH` is still never searched.
A config without `ExemptMark` is not affected,
which is the owner's earlier decision above.
`tunnel-up.unit.test.ts` runs the real `up` sequence over recording steps and the real exemption step:
a missing program with `ExemptMark` stops the up,
runs the rollback once and skips the post hooks;
no `ExemptMark` goes on without the exemption step;
a found program is run once;
and any other failing step is rolled back and rethrown.
The real `up` needs root,
so only this sequence is covered here.

### Every method, one union

Any match means clearnet.
Every method is always on:
there is no setting,
flag,
environment variable,
or config key that disables any of them,
and a test fails if one appears in either wg package.

1. The process watch:
   `wg-quicker-exempt`'s root watcher moves a process that is an update entry point into the exempt slice,
   so it and its later children inherit the socket mark.
   The entry points are `mise run` of a toolchain task that fetches,
   `mpm-shadow install`,
   `update` and `outdated` with or without `-g`,
   the `mpm` shadow,
   and `node` running the TypeScript `mpm-shadow` launcher (`package/monorepo-task/core-exec/src/mpm-shadow.ts` or its built entry),
   which downloads the locked `mpm-shadow` archive when no installed copy is new enough.
   A process is recognized by the file name of its resolved executable (`mise`,
   `mpm-shadow`,
   `mpm`,
   `node`)
   together with its command-line words.
   The first command-line word alone never matches,
   because `exec -a mpm /usr/bin/curl` sets it freely.
   Pi is an entry point of its own:
   `node` whose first command-line word is `pi`,
   which is the single word Pi leaves after it rewrites its process title,
   or `node` running Pi's CLI entry,
   `/pi-coding-agent/dist/bundle/cli.js`.
   The word counts only for `node`,
   so `exec -a pi /usr/bin/curl` does not match,
   and a checkout of the same package runs the same entry and matches too.
2. The socket-marked cgroups:
   A cgroup of the user's `app.slice` whose name the watcher selects carries the socket mark itself,
   so every process in it,
   and every process in a cgroup the manager creates below it,
   leaves the tunnel without being moved.
   The terminals and desktop applications above are selected this way,
   and so is every `pi-<name>.service` unit,
   because the user's service manager cannot move a service's main process:
   the mark covers it instead,
   which is the rule the terminals have.
   This is how a Pi agent run stays outside the tunnel,
   its driver and every child process with it.
3. The update hosts:
   Traffic to the hosts in `src/update-hosts.data.ts` bypasses the tunnel for every program,
   browsers included.
   The list holds every host named by `toolchain.plan.jsonc`,
   `toolchain.lock.json`,
   the Rust resolvers and installers under `package/mpm-shadow/cli/src/toolchain` and `package/file-enforcer/core-rust/src`,
   and the redirect hosts the plan never names:
   `objects.githubusercontent.com`,
   `release-assets.githubusercontent.com`,
   `index.crates.io` and `static.crates.io`.
   A test scans those sources and fails when a host appears there that the list lacks.
4. The model provider hosts of Pi:
   Traffic to the hosts in `src/provider-hosts.data.ts` bypasses the tunnel for every program,
   browsers included,
   and the list is kept apart from the update hosts because they leave the tunnel for another reason:
   they carry the requests of the agents.
   It holds `api.deepseek.com`,
   `api.neuralwatt.com`,
   `api.synthetic.new`,
   `opencode.ai`,
   `token-plan.ap-southeast-1.maas.aliyuncs.com` and
   `token-plan.cn-beijing.maas.aliyuncs.com`,
   each recorded beside the provider id whose base URL names it:
   `deepseek`,
   `neuralwatt`,
   `synthetic`,
   `opencode-go`,
   `qwen-token-plan`,
   `qwen-token-plan-individual` and `qwen-token-plan-cn`.
   A request to any of them answers without a redirect,
   and the suite reads the list from those base URLs,
   so a host cannot be added to a provider without the list following it.
5. The toolchain paths:
   A process whose executable is under `<data home>/monochromatic-dev/toolchain/` or `toolchain-global/`
   is moved like the first method.
   Every program the toolchain installed there qualifies,
   for example `node`,
   `bun`,
   `pnpm`,
   `uv`,
   `zig`,
   `cmake`,
   `rust`,
   `caddy`,
   `sops` and `age` (the examples are the tools installed when this was written;
   the start notice of `wg-quicker-exempt` lists what is installed on the machine now),
   so while a tunnel with `ExemptMark` is up,
   every script and dev server run by the toolchain's node,
   every build by its compilers,
   and everything they fetch runs outside the tunnel,
   not only updates.
   This is the user's choice,
   not a side effect,
   and the start notice says it.
   The toolchain's node is the executable of a running Pi,
   so this rule reaches every Pi process as well,
   and the socket-marked unit above is what keeps it outside the tunnel,
   because the process watch never moves a service's main process.

### How the host rules are routed

One reconciliation installs the rules of both lists,
because a refresh that installed only its own list would remove the other list's rules as unwanted.
The traffic block reads its exceptions from these rules,
so a provider host is let through the block the same way an update host is.

For each address of each listed host,
the route watcher adds `ip rule add to <address> table <bypass table> pref <bypass preference> protocol 201`.
The bypass table already carries copies of the physical defaults,
so the rule sends every packet to that address along the physical route,
whatever process sent it.
The rules are owned by the lifecycle:
`down` lists the rules with protocol `201`,
the claimed table,
the claimed preference and a destination selector,
and deletes exactly those.
No rule is installed for a loopback,
private,
link-local,
shared,
documentation,
benchmarking,
protocol-assignment,
6to4,
Teredo,
segment-routing,
deprecated site-local,
NAT64 (both prefixes),
IPv4-mapped,
multicast or reserved address,
so a hostile or broken DNS answer cannot route local traffic through the bypass table;
a refused answer is reported.

#### Which DNS servers answer

Host rules use only answers from the tunnel's own `DNS =` servers,
asked through the tunnel.
That was the owner's choice,
labelled "Tunnel DNS only":
a resolver the tunnel's owner did not configure is not trusted to decide which addresses leave the tunnel.
The lookup goes to `systemd-resolved` over its varlink socket with `ResolveRecord`,
naming the tunnel link's interface index and the flags `DNS`,
`NO_SEARCH`,
`NO_SYNTHESIZE`,
`NO_ZONE` and `NO_TRUST_ANCHOR` (26881),
so only unicast DNS on that link can answer and the name is asked exactly as written.
No search domain is appended:
the `lan` search domain of the network cannot turn `github.com` into `github.com.lan` and make the router's answer a rule.
Node's own resolver is not used.
Its source (`src/cares_wrap.h` of v26.13.0) calls `ares_query_dnsrec`,
which does not expand search domains either,
but it would send plain DNS to the stub on `127.0.0.53`,
and the stub would choose the upstream by its own routing domains and not by the tunnel link.
The varlink call names the link,
so the choice is not left to routing domains.
A call ends at a total deadline of ten seconds from its start,
not after ten seconds of silence,
because a peer that sends one byte now and then is never silent,
and a reply longer than 1 MiB is refused,
because the reply for one record is a few hundred bytes.

The link keeps its own DNS-over-TLS setting,
so the query path keeps DNS-over-TLS.
On 2026-10-31 `resolvectl status fr-par-fr1` showed `+DNSOverTLS` and `resolvectl dnsovertls fr-par-fr1` printed `yes`,
the strict mode without a cleartext fallback.
The reply of every measured lookup for all 21 hosts also carried the confidential flag of `systemd-resolved`,
which its source documents as "only sent via encrypted channels,
or never left this system";
the flag alone therefore does not prove a TLS hop,
and the strict link setting is what does.
The servers of `fr-par-fr1` are `2606:4700:4700::1111`,
`1.1.1.1`,
`9.9.9.9`,
`149.112.112.112`,
`2620:fe::fe` and `2620:fe::9`.

A config with no `DNS =` gets no host rules.
`up` prints `POLICY NOT IN FORCE` for the clearnet-host policy,
and every refresh prints it again,
saying that the tunnel sets no DNS server,
that traffic to the update hosts and the model provider hosts uses the tunnel,
and that the process watch,
the toolchain paths and the Pi coverage still run.
Where the DNS servers or the peer endpoints of the tunnel cannot be read,
the refresh changes nothing and says so.

#### What never gets a rule

- The tunnel's own DNS servers and the endpoints of its peers,
  in any spelling,
  because a rule for them would move the tunnel's control traffic or its DNS onto the physical network.
  The refusal is reported by name.
- More than 64 usable addresses from one host,
  or more than 384 distinct addresses in total.
  Only an address that passed every check counts toward the 64,
  so a resolver cannot hide the real addresses behind unusable ones.
  The numbers come from the answers measured on 2026-10-31:
  132 addresses in total,
  92 distinct,
  and at most 24 for one host (`registry.npmjs.org`),
  so 64 is about 2.7 times the largest answer and 384 about four times the distinct total.
  The cut is reported with how many addresses were cut.
  A report names its first ten problems and counts the rest.

`ip rule` commands run at most 8 at a time,
and each rule added or removed is logged at info level with the host or hosts behind it.

#### Refresh

The watcher resolves every host through the tunnel link,
asking for both address families with their TTLs,
adds new addresses at once,
and sleeps for the smallest TTL clamped between 30 seconds and 15 minutes.
When every lookup of a refresh timed out,
the resolver is not answering yet (the live restart of 2026-10-31 saw that for the first minute after the tunnel was created),
so the next refresh is in 5 seconds instead;
the refresh is still reported loudly,
and any other failure keeps the usual delay.
The wait ends at once when the watcher is asked to stop.
An address leaves the rules only after it was absent from two consecutive refreshes,
because round-robin DNS and CDNs drop an address from one answer and return it in the next.
The first refresh runs during `up`,
so the rules exist before traffic flows.

### Warned on every start, loud when not in force

Every `up` with `ExemptMark` prints a notice on standard error and in the log that names exactly which traffic leaves
outside the tunnel:
the update entry points,
everything the toolchain installed,
Pi with every process of its `pi-<name>.service` unit,
every update host,
and every model provider host of Pi.
It also says why:
the repository keeps toolchain updates on the physical network,
and the agents' provider requests are its own work,
not the tunnel's.
It states what a reader could otherwise miss:

- a host rule carries a whole address,
  so a shared CDN address sends every other site on it outside the tunnel too;
  `docs.python.org` shares all eight of its addresses with `pypi.org`,
  and `raw.githubusercontent.com` and `avatars.githubusercontent.com` share four of eight with the release-asset hosts
  (measured 2026-10-31);
- the toolchain root rule covers every installed program and every script run by the toolchain's node,
  not only updates,
  and a running Pi is the toolchain's node;
- a Pi unit is not moved:
  the user's service manager cannot move a service's main process,
  so the socket mark attached to the unit's own cgroup covers it and every process below it,
  the rule the terminals have;
- what can imitate a match:
  the executable's file name together with its command-line words,
  the tail of a toolchain script path,
  any executable named `mpm`,
  and any executable under the install roots,
  which are inside a home directory the user can write
  (a link is never followed,
  by the owner's choice "Refuse links":
  `wg-quicker-exempt` ignores a toolchain folder that is a link,
  wherever it points,
  and reports it loudly at every start),
  so any program the user can run from such a path or under such a name leaves outside the tunnel too,
  and so does `exec -a pi node`,
  because Pi is matched by the one command-line word it sets for itself,
  and any unit named `pi-<name>.service`,
  because a unit name is a name the user can choose;
- what is not covered:
  processes in another pid namespace (containers and sandboxes),
  processes of a login session,
  and the main process of a service whose cgroup the watcher does not mark.

The notice is built by `startNotices`,
a pure function of the config facts,
so a test checks every combination of mark,
`Table = off` and DNS servers and fails if any start would print nothing.
With `Table = off` wg-quicker installs no routing rule and no bypass table,
so the start prints the `POLICY NOT IN FORCE` report instead of the notice,
saying that where traffic goes is decided by the user's own routing.

`wg-quicker-exempt` prints its part of the notice when its watcher starts,
into its log,
and the loud lines it printed while starting reach the output of `up`.

Nothing degrades silently.
A report that starts with `POLICY NOT IN FORCE` names the policy that cannot be enforced and what that leaves:

- a start without `ExemptMark` names both policies,
  because the bypass table that carries them does not exist and update traffic uses the tunnel;
- a config with no `DNS =` server,
  a DNS failure,
  a refused answer,
  a cut by the address caps,
  or a failed `ip rule` command names the update-host policy and the affected hosts or addresses;
- a failed slice creation,
  process-event subscription,
  dropped events,
  unmovable process,
  failed adoption,
  or failed BPF attach names the process-watch policy or the socket-marking policy.

No failure installs a different route on its own:
the traffic that the failed policy covers stays in the tunnel.

### Failures after startup

The route watcher runs detached with its own output discarded,
so a failure after `up` returned would reach nobody.
Each failure is therefore delivered the way the owner chose,
"Notification + journal":
to the system journal at error level,
and as a desktop notification to the user who started the tunnel through `sudo`,
which is `SUDO_UID` and `SUDO_GID` as the watcher inherits them.
The journal line carries the identifier `wg-quicker`
(`journalctl --identifier=wg-quicker`),
once per failure kind per minute;
the notification comes once per failure kind every ten minutes,
sent by `busctl` as that user through `setpriv` with a cleared environment.

The limit keys on the kind of failure and on a normalized cause within the kind,
never on the raw text,
because the text embeds names a user chose,
such as a host answer,
and a key on the raw text would let distinct names defeat the limit.
The kinds are `update-host problems`,
`update-host refresh failed`,
`route monitor failed`,
`route watcher stopped` and `route watcher died`.
The normalized cause is the line,
at most its first 1024 characters,
with every run of digits made alike and every name a user chose replaced by a placeholder
(an absolute path and a unit name ending in `.service`,
`.scope`,
`.slice`,
`.socket` or `.mount`).
The bound is large because every loud line starts with a fixed phrase of over a hundred characters,
and a bound of 200 ended inside it,
so two different errors of one kind collapsed into one cause.
Each channel delivers the first four distinct causes of a kind in each interval,
so two different problems in the update-host refreshes are two reports and not one,
and at most 32 further causes per interval over all kinds together;
the first cause of every kind is always delivered.
The map of remembered kinds holds at most 64,
dropping the expired ones first and then the oldest.
A repeat or a further cause held back by the limit is counted,
and the next delivery says "repeated N more time(s) since the last report";
the identity of a held-back cause beyond the cap is not kept.
A later notification of the same kind and cause replaces the earlier one on the desktop
(the id `Notify` printed is passed as `replaces_id`),
distinct causes stay side by side,
and each notification expires after ten minutes,
so one persistent failure shows one notification and not several.
What `up` itself finds while it starts is not rate limited:
the refresh that runs during `up` logs every report to the terminal before anything is delivered,
and only the detached watcher's later deliveries to the journal and the desktop go through the limit.
A channel that fails is named in the watcher's log,
and a start without a `sudo` caller still journals and says that no user can be notified.

Text that a user chose reaches a line escaped.
The loud report builder escapes control characters,
escape sequences,
line breaks,
line and paragraph separators,
direction marks and the invisible tag characters as `\u{XXXX}`,
the journal input is escaped again,
the notification body has its markup characters turned into entities as well,
and the lines relayed from the start of `wg-quicker-exempt` are escaped before they are logged.

The failures that reach this channel are a failed update-host refresh,
a refresh with problems (including a tunnel with no `DNS =` server),
a route monitor session that failed and will be replaced,
a watcher that stopped for good,
and a watcher that died.
The same texts stay on the terminal during `up`.
`wg-quicker-exempt` delivers its own failures the same way,
under the identifier `wg-quicker-exempt`.

### When the route watcher itself dies

The watcher cannot report its own death when `SIGKILL`,
the out-of-memory killer or `systemd-oomd` ends it,
and from then on the bypass table no longer follows the physical default route
and the update-host rules are no longer refreshed.
A second small process,
the sentinel (`bypass-watch-sentinel`),
is started once the watcher has registered its identity record.
It is started outside the cgroup of the watcher:
as its own transient service of the system manager,
`wg-quicker-sentinel-<watcher pid>-<start time>`,
through `systemd-run --collect` with `ManagedOOMPreference=omit`,
with the program (`node`) behind `/usr/bin/env`,
and with the sudo identity passed as `--setenv` because a service does not inherit it.
The program is started behind `/usr/bin/env` because an SELinux-enforcing host does not let the system manager
(`init_t`) execute files under a home directory,
and the toolchain `node` sits in one:
the unit was accepted and then failed with `status=203/EXEC`.
The cause and the measurements are in `doc/troubleshooting/selinux-denies-system-service-exec-of-home-files.md`.
The watcher runs in the cgroup of the terminal or session that ran `wg-quicker up`,
and a kill of that whole cgroup,
for example by `systemd-oomd`,
would take a sentinel in the same cgroup with it and leave nothing to report.
A service in its own root-owned cgroup survives that,
and `systemd-oomd` is asked not to choose the service itself.
If the manager refuses the unit,
`systemd-run` is missing or hangs (it is ended after ten seconds),
or the manager accepts the unit and the sentinel never confirms (its program can still fail to execute after the start was accepted),
the unit is stopped and the sentinel starts detached beside the watcher instead,
and `up` says loudly that a kill of the watcher's whole cgroup then ends both without a report.
After `systemctl stop` the unit's state is read whatever `stop` said
(a unit that already ended and was collected is no longer loaded,
which `stop` reports as a failure although nothing runs),
and only `inactive` or `failed` counts as gone.
If the unit cannot be shown to be gone,
a second line says so:
two sentinels may run,
and one death of the watcher may be reported twice.
`/usr/bin/env` reads a first argument that contains `=` as a variable assignment and has no option that prevents it,
so a `node` path with `=` in it is not started as a service:
`up` says why and the sentinel starts detached beside the watcher,
which does not go through `env`.
A `node` path that is not valid UTF-8 cannot be told apart from its replacement in the string Node.js gives the program,
which is a limit of this side.
`systemd-run` and `systemctl` run with a cleared environment and a fixed search path (`/usr/bin:/bin`),
never the caller's.
It checks every half second that the process with the watcher's pid and start time still exists,
so a reused pid is not mistaken for the watcher.
When the watcher is gone it waits one and a half seconds and reads the identity record:
a record removed by the watcher's own clean exit or by `wg-quicker down`,
or a record that names another watcher,
means the end was asked for and the sentinel exits quietly;
a record that still names the dead watcher means it was killed or crashed,
and the sentinel delivers a report of kind `route watcher died` naming both policies;
an unreadable record is reported as unknown.
A watcher that stops with an error already delivers its own report,
and its record is removed on the way out,
so that case is not reported twice.
A shutdown or a reboot is not a policy failure:
it ends the watcher with everything else,
and the state in `/run` and the tunnel go with it.
When the verdict would be a report (the record still names the dead watcher,
or cannot be read)
the sentinel first asks whether the whole system is stopping,
and if it is,
it reports nothing.
The system counts as stopping only when `systemctl show --property=ActiveState --value shutdown.target` prints `active` or `activating`.
`/run/nologin` is deliberately not a signal:
`man 8 shutdown` documents that with a time argument the file is created five minutes before the system goes down,
when a real death should still be reported,
and a stale or administrator-made file would hide every death.
A logout while the system keeps running is still reported:
the tunnel stays up with no watcher,
so the update traffic of later sessions is no longer exempted,
which is a real failure.
A probe that cannot tell (`systemctl` fails or runs past five seconds) counts as a system that keeps running,
so a failing probe never hides a death.
The case this can hide is a watcher that dies after `shutdown.target` became active,
which is already part of the shutdown.
A sentinel that was started is not taken for running because its process was created.
When it has loaded it writes a marker beside the state (`<state>.sentinel.json`)
with the watcher's pid,
its own pid and its own start time,
and `up` waits up to two seconds for a marker that names this watcher
while the sentinel's process still exists with that start time.
If none appears,
or if the sentinel cannot be started at all,
`up` says so loudly,
with `journalctl --unit=wg-quicker-sentinel-<pid>-<start time>` as the place to look
(the unit is collected when it ends,
so `systemctl status` shows nothing for it),
and goes on with the tunnel.
The marker is written to a new file that is renamed over `<state>.sentinel.json`,
so a link planted at that path is replaced and never followed.
The service start was exercised on the real host with a throwaway unit and a stand-in watcher process:
the sentinel ran in `unconfined_service_t` in `system.slice` with `ManagedOOMPreference=omit`,
wrote its marker,
and exited when the stand-in watcher died;
the unit that names `node` directly failed with `status=203/EXEC` in the same test.
The limit is stated plainly:
if the sentinel is killed first,
or both die together,
nothing reports it,
and a clean system shutdown ends both;
with the service,
a kill of the watcher's whole cgroup no longer takes the sentinel with it.
`wg-quicker-exempt` has its own sentinel for its watcher.

### Boundaries

- A CDN may hand a program an address that differs from the one the watcher resolved.
  Both use the same resolver path and cache,
  but nothing guarantees equal answers.
- The destination rule also carries every other site on a shared address,
  not only the update host.
  The owner kept the list as built,
  `github.com` and `api.github.com` included,
  and accepted the shared addresses.
- A process in a Flatpak or `bwrap` sandbox cannot get itself moved by imitating a name:
  the watcher sees it from the host with its real executable and a different pid namespace,
  so `exec -a mpm sleep 8` inside `bwrap --unshare-pid` showed the executable `/usr/bin/sleep`,
  was visible to the host watcher,
  and was adopted into a throwaway slice by a matcher that trusted the first command-line word;
  the matcher now trusts the executable's file name and skips another pid namespace with a report.
- Sockets a process opened before it was moved keep the tunnel route.
- Only processes inside the user's systemd manager subtree can be moved.
  See the [wg-quicker-exempt README](../cli-quicker-exempt/README.md#update-traffic-policy).

## Development

Run package checks through mise:

```console
mise run //package/wg/cli-quicker:buildAndTest
mise run //package/wg/cli-quicker:test:integration:bypass
mise run //package/wg/cli-quicker:test:integration:route
mise run //package/wg/cli-quicker:lint:oxlint
mise run //package/wg/cli-quicker:lint:types
```

State-mutating integration tests use disposable network namespaces.
Do not use the real `mx-que-mx1` interface for verification.
