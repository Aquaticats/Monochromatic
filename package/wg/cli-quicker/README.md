# wg-quicker

Bring WireGuard interfaces up and down with `wg-quick`-compatible lifecycle behavior,
without parsing large `AllowedIPs` values through Bash pattern matching.

```console
wg-quicker up wg0
wg-quicker down wg0
```

A bare interface name resolves to `/etc/wireguard/<name>.conf`.
An explicit `.conf` path is also accepted.

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
System-firewall paths,
daemon-config overrides,
and `WG_QUICKER_RUNTIME_DIRECTORY` must be absolute so separate lifecycle invocations cannot resolve different
files or state.
The overrides cross the sudo boundary through the validated private caller-context file.
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
 Firefox Nightly,
 ChatGPT,
 Interpreter,
 and Qure will use the tunnel and instructs the user to add
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
Firefox Nightly's `app-firefox\x2dnightly@*.service`,
ChatGPT's `app-chatgpt@*.service`,
Interpreter's `app-interpreter@*.service`,
and Qure's `app-qure@*.service`,
drains queued events,
and scans again to close creation race.
It reacts to future cgroup creation and periodically maps every live Helium executable,
including renderer,
zygote,
and crashpad processes,
both Pale Moon executable names,
and Firefox Nightly's `firefox` and `firefox-bin` executables under the exact `firefox-nightly` install directory
back to their current cgroups.
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
Known process-discovered cgroups remain attached until directory disappears,
covering process restarts inside same service or scope.
Process discovery attaches entire current cgroup,
so sibling processes in a shared cgroup also bypass tunnel until cgroup disappears or watcher stops.
A newly started Helium,
Pale Moon,
Firefox Nightly,
ChatGPT,
Interpreter,
or Qure process can create sockets before next 250-millisecond rescan;
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
  A missing `wg-quicker-exempt` program is reported by the stop itself and `down` goes on,
  as before.
- Remove the routing:
  the policy rules,
  the application-bypass routes,
  rules and state,
  the kill switch,
  and the interface.
  The interface delete is verified:
  an interface that is still there after `ip link delete` is a failure.
- Revert the DNS settings,
  remove the OpenSnitch allowance,
  and remove a leftover block.
  The tunnel is gone by then,
  so a failure here exposes nothing and is reported without a block.

If a step before the routing removal fails,
nothing was removed and the tunnel is still fully up:

```text
error: could not stop the route watcher (<reason>)
the tunnel fr-par-fr1 is still fully up; nothing was removed
fix: <what to do>, then run `wg-quicker down fr-par-fr1` again
```

If a step of the routing removal fails,
traffic would go outside the tunnel from a tunnel that looks up,
so `down` blocks traffic instead and says how to finish.
The block is an nft table of its own,
`inet wg-quicker-<interface>-blocked`,
that drops everything leaving through an interface other than loopback and everything forwarded.
The tunnel's kill switch (`inet wg-quicker-<interface>`) is a different table and only drops spoofed packets addressed to the tunnel's own addresses;
it blocks nothing outbound.
The network does not work while the block is in place:

```text
error: down fr-par-fr1 failed part way: could not delete the interface (<reason>)
traffic is BLOCKED (nft table inet wg-quicker-fr-par-fr1-blocked drops everything that leaves the machine), so nothing goes outside the tunnel unprotected, and the network will not work until down is finished
to finish: run `wg-quicker down fr-par-fr1` again, with this build or with the build that started the tunnel; a down that succeeds removes the block
to lift the block by hand: sudo nft delete table inet wg-quicker-fr-par-fr1-blocked
```

If the block cannot be installed either,
the error says that traffic leaves outside the tunnel now.
A successful `down`,
a `down` of an interface that is already gone,
and an `up` (which refuses an existing interface) remove a leftover block.
The rollback of a failed `up` does not use this order and does not block.

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

### Three methods, one union

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
   and the TypeScript toolchain `sync.ts` and `update.ts`.
   A process is recognized by the file name of its resolved executable (`mise`,
   `mpm-shadow`,
   `mpm`,
   `node`)
   together with its command-line words.
   The first command-line word alone never matches,
   because `exec -a mpm /usr/bin/curl` sets it freely.
2. The update hosts:
   Traffic to the hosts in `src/update-hosts.data.ts` bypasses the tunnel for every program,
   browsers included.
   The list holds every host named by `toolchain.lock.json`,
   the toolchain resolvers and `mpm-shadow`'s network client,
   and the redirect hosts the plan never names:
   `objects.githubusercontent.com`,
   `release-assets.githubusercontent.com`,
   `index.crates.io` and `static.crates.io`.
   A test scans those sources and fails when a host appears there that the list lacks.
3. The toolchain paths:
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

### How update hosts are routed

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
`up` prints `POLICY NOT IN FORCE` for the update-host policy,
and every refresh prints it again,
saying that the tunnel sets no DNS server,
that traffic to the update hosts uses the tunnel,
and that the process watch and the toolchain paths still run.
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
and every update host.
It also says why:
the repository keeps toolchain updates on the physical network.
It states what a reader could otherwise miss:

- a host rule carries a whole address,
  so a shared CDN address sends every other site on it outside the tunnel too;
  `docs.python.org` shares all eight of its addresses with `pypi.org`,
  and `raw.githubusercontent.com` and `avatars.githubusercontent.com` share four of eight with the release-asset hosts
  (measured 2026-10-31);
- the toolchain root rule covers every installed program and every script run by the toolchain's node,
  not only updates;
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
  so any program the user can run from such a path or under such a name leaves outside the tunnel too;
- what is not covered:
  processes in another pid namespace (containers and sandboxes),
  and processes the user's service manager cannot move.

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
