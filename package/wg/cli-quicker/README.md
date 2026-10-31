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
runtime,
UID,
and companion settings through a private mode-`0600` bounded file.
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
and selected paired or caller-path companion for root execution;
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
It prefers explicit `WG_QUICKER_EXEMPT_COMMAND`,
then paired repository release or debug build,
then installed command in privileged or captured caller `PATH`.
An unavailable configured companion fails before interface creation.
The watched desktop UID comes from `WG_QUICKER_EXEMPT_UID`,
 then `SUDO_UID`,
 then non-root effective UID.
Direct root execution without an explicit or sudo identity fails instead of watching root's app slice.

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

The watcher also attaches the toolchain updates slice,
`app-monochromatic\x2dupdates.slice`,
so the toolchain's sync and update runs and every child they start (cargo-binstall,
cargo,
rustup,
uv,
pnpm,
node fetch,
curl) take the physical route instead of the tunnel.
`mise run toolchain:sync`,
`toolchain:update`,
`toolchain:sync:global`,
`toolchain:update:global` and `mpm-shadow sync` and `update` re-execute themselves through
`systemd-run --user --scope --slice=app-monochromatic\x2dupdates.slice`
after a silent probe proves a user manager accepts the scope.
They stay in place,
with a logged reason,
on another platform,
in a container or any host without a user manager,
under `MONOCHROMATIC_TOOLCHAIN_CLEARNET=off`,
and when already inside the slice.
The `app-` prefix matters:
systemd nests the slice directly under `app.slice`,
the only directory the watcher scans and watches,
and `\x2d` keeps the name one hierarchy level.
Cgroup programs apply to descendant cgroups,
so each per-run scope inherits the mark with no attach of its own.
A slice stays active after its last scope exits,
so the attachment survives between runs.
Without a tunnel nothing marks the slice and the traffic takes the default route anyway.
When the slice does not yet exist,
the silent probe creates it and the real run starts only after the probe's process has ended.
That interval is the margin the watcher's inotify event has to attach the new slice;
nothing confirms the attachment,
so a watcher that is slower than the probe leaves the first sockets of that one run unmarked.
Later runs find the slice already attached.

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
