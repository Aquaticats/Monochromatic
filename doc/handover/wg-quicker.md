# `wg-quicker` completion handover

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/wg-allowedips` is now `package/wg/core-allowedips`;
> `@monochromatic-dev/module-wg-allowedips` is now `@monochromatic-dev/wg-core-allowedips`;
> `package/cli/wg-allowedips` is now `package/wg/cli-allowedips`;
> `package/cli/wg-quicker-exempt` is now `package/wg/cli-quicker-exempt`;
> `package/cli/wg-quicker` is now `package/wg/cli-quicker`.

## Goal and safety boundary

The production replacement for `wg-quick up/down` is complete.
It:

- handles large `AllowedIPs` values without Bash pattern matching;
- generates peer-local `AllowedIPs` from allowed and disallowed source files;
- owns collision-safe dual-stack bypass routing;
- exempts Ghostty,
   Steam,
   Helium,
   Pale Moon,
   Firefox Nightly,
   ChatGPT,
   Interpreter,
   and Qure sockets through cgroup-BPF;
- keeps privileged BPF implementation in Rust;
- does not move Ghostty into another systemd slice.

The human brought real `mx-que-mx1` up during follow-up diagnosis and then brought it down after connectivity
stopped.
It is currently down.
No exemption watcher was running when ChatGPT and Interpreter coverage landed on 2026-10-06:
no WireGuard interface existed,
no `wg-quicker-exempt` process ran,
and no bypass `ip rule` was installed,
so the next `up` starts a watcher from the current companion build.
On 2026-10-08 that tunnel was up again with an active watcher,
`wg-quicker-exempt __watch mx-que-mx1 100 1000`,
and the priority `50` `fwmark 0x64` bypass rule installed,
so Qure coverage landed against live tunnel state rather than an idle host.
Do not bring tunnel up,
restart it,
or mutate live routing without explicit authorization.
IVPN Desktop split tunneling is currently disabled after its supported `ivpn splittun -off` recovery.
`/etc/wireguard/mx-que-mx1.conf` now contains `ExemptMark = 100` under `[Interface]`.
The atomic edit preserved root ownership,
mode `0600`,
single-link status,
and the down physical endpoint route.
`8888` was the earlier value;
it carries netavark's masquerade mask `0x2000`,
which source-NATed every exempted socket including IPv4 loopback,
and config parsing now rejects it.
`/etc/wireguard/gb-lon-gb2.conf` carries the same `ExemptMark = 100`,
and that tunnel was cycled so its watcher and `ip rule` selector use the new mark.
That file was mode `0644` owned `root:user`;
both configs are now mode `0600` owned `root:root`,
matching the `0700 root:root` directory that already blocked other users.
See `doc/troubleshooting/netavark-masquerade-mask-exempt-mark.md`.

On 2026-10-06 both configs stopped carrying a baked `AllowedIPs` value.
Each `[Peer]` section's single `AllowedIPs` line,
`90810` bytes holding `4082` networks generated on 2026-09-17,
was replaced atomically with `AllowedIPsFromFiles = ~/allowed.txt ~/disallowed.txt`,
so every `up` regenerates from the current input files.
Both edits preserved mode `0600`,
`root:root` ownership,
single-link status,
and the `unconfined_u:object_r:etc_t:s0` label.
Pre-edit copies are `/root/wg-quicker-conf-backup-gb-lon-gb2.conf-20261006`
and `/root/wg-quicker-conf-backup-mx-que-mx1.conf-20261006`,
each mode `0600`.
The directive expands through the runtime home directory,
which the sudo launcher sets to the caller's home;
a direct root invocation whose `HOME` is `/root` fails closed on the missing file
instead of silently narrowing coverage.

## Completed tasks

- Task `#1`:
   assess inherited handover.
- Task `#2`:
   extract shared AllowedIPs module.
- Task `#3`:
   correct peer-local `AllowedIPsFromFiles` semantics.
- Task `#4`:
   harden bypass routing and route watcher.
- Task `#5`:
   harden Rust BPF loader lifecycle.
- Task `#6`:
   cover Ghostty and Helium cgroups.
- Task `#7`:
   run final cross-package verification and update this handover.
- Blocker task `#8`:
   work around Linux bpffs SELinux pin regression.
- Tasks `#9` through `#14`:
   diagnose and harden self-elevation,
   caller-context preservation,
   exact companion resolution,
   and scrubbed-sudo verification.
- Tasks `#15` through `#17`:
   diagnose system-wide ISP egress,
   trace interacting IVPN and endpoint routes,
   and prevent endpoint recursion.
- Pale Moon follow-up:
   discover both installed Pale Moon executable names,
   verify live target enumeration,
   and update exemption diagnostics and documentation.
- ChatGPT and Interpreter follow-up:
   discover both applications' executables and desktop services,
   verify live target enumeration against real desktop launches,
   extend `~/disallowed.txt` with their measured endpoints,
   and switch both live configs to `AllowedIPsFromFiles`.
- Qure follow-up:
   discover its AppImage executables,
   bundled CLI wrappers,
   and desktop service,
   verify live target enumeration against a real launch,
   and extend `~/disallowed.txt` with its measured endpoints.

No tracked implementation task remains.

## Relevant commits

```text
b8958bde1 initial inherited implementation
7e37cbf76 shared AllowedIPs module and static consumers
68eb96063 peer-local AllowedIPsFromFiles semantics
cd72d867d bypass-routing hardening work
5016bffd0 bypass-routing hardening work
7eda14e17 bypass ownership lifecycle
6d785d0a2 watcher process-group termination
c3bb6735a exact bypass route ownership persistence
1bb08f1e4 restart checkpoint handover
4bef73b4e hardened Rust BPF link lifecycle and SELinux fallback
3b92eb5c8 Ghostty and Helium application watcher integration
e43e24afa self-elevation before config access
de20d4b26 privilege-process ownership boundary
c520bd996 caller context across sudo env reset
6cabed890 exact Rust companion resolution
94c907045 effective-UID privilege gate
622fd9844 caller-context schema regression tests
6805239fa and a8c2746c6 sudo troubleshooting documentation
2d94b86fd endpoint-recursion prevention and IVPN conflict preflight
ff9081828 normalized conflict detection and dual-stack route coverage
a41cb8efb missing-ExemptMark warning and live config note
743581a55 configured-exemption warning-path coverage
492b6914e actionable config-specific warning wording
7b3621d47 Pale Moon process discovery and exemption documentation
5c65b29ce exact Pale Moon matching and cgroup-boundary documentation
c07f6e56e ChatGPT and Interpreter executable discovery
9e5e33d24 Qure executable and desktop-service discovery
```

Other commits interleaved at `HEAD` belong to concurrent work and are unrelated.

## Shared AllowedIPs ownership

`package/module/wg-allowedips/` owns:

- allowed and disallowed input generation;
- CIDR parsing,
   normalization,
   minimization,
   and subtraction;
- DNS and ASN lookup seams and types;
- IPinfo ASN cache validation and atomic replacement;
- explicit cache-directory selection.

Both CLIs use static workspace imports from
`@monochromatic-dev/module-wg-allowedips/ts`.
There is no CLI-to-CLI dependency and no runtime `import.meta.resolve()` source lookup.
Built bundles work outside workspace package resolution.

## Peer-local file generation

A peer may contain:

```ini
[Peer]
PublicKey = peer-public-key
AllowedIPsFromFiles = ~/allowed.txt ~/disallowed.txt
Endpoint = vpn.example:51820
```

The directive belongs to containing `[Peer]`.
Expansion preserves directive insertion point.
Parser rejects:

- directive outside a peer;
- duplicate source directives in one peer;
- literal `AllowedIPs` and `AllowedIPsFromFiles` in same peer;
- malformed source path count.

Multiple peers can use independent source files.
`down` retains metadata without reading files,
resolving domains,
or refreshing ASN caches.

## Automatic tunnel routing

Automatic mode now puts every allowed prefix in one interface-specific policy table,
not only literal `/0` routes.
The WireGuard fwmark lets outer endpoint packets skip that table and retain physical main-table routing,
including when a non-default allowed prefix covers the endpoint.
Main non-default routes are evaluated first for connected physical and LAN paths.

Separate automatic tunnels use separate policy tables.
Overlapping allowed prefixes are therefore selected by policy-rule order before routes in different tables can
compete by prefix length.
Use explicit route ownership if simultaneous overlapping tunnels need cross-interface longest-prefix behavior.
Automatic partial tunnels now receive the source-mark validation and nft ingress protection previously reached
only by literal-default mode.

`up` rejects IVPN Desktop's exact active split-tunnel rule before interface creation.
The check recognizes mark `0xca6c` with table `17` or `ivpn-exclude-tbl` from both family JSON rule listings.
It is a one-shot preflight;
do not re-enable IVPN split tunneling while `wg-quicker` remains active.
See `doc/troubleshooting/ivpn-3-15-inverse-split-tunnel-overrides-wireguard.md`.

## Bypass routing

`package/cli/wg-quicker` now provides:

- dynamic dual-stack bypass-table allocation;
- preference selection before every existing positive-priority rule;
- per-interface and global `flock` operation locks;
- exact transition-state persistence;
- exact kernel-rendered route fingerprints;
- fail-closed unreachable defaults for a missing address family;
- route monitoring before initial synchronization;
- monitor-child restart;
- owner,
  PID,
  start-time,
  and full-command validation;
- validated process-group shutdown;
- state retention after cleanup failure;
- teardown of only persisted routes.

Route protocol `201` is a tag,
not ownership proof.
Teardown never flushes a whole table and preserves unrelated protocol-`201` routes.

Only exit `2` with selected family's exact `FIB table does not exist` diagnostic plus `Dump terminated` means an absent
iproute2 family table.
See `doc/troubleshooting/iproute2-family-fib-table-absence.md`.

## Rust BPF loader

Package:
 `package/cli/wg-quicker-exempt/`.

Public commands:

```text
wg-quicker-exempt attach <mark> <cgroup-dir>...
wg-quicker-exempt detach <cgroup-dir>...
wg-quicker-exempt list-targets <uid>
wg-quicker-exempt watch-start <key> <mark> <uid>
wg-quicker-exempt watch-stop <key>
```

The crate depends only on `libc` and uses stable raw `bpf(2)` UAPI.
No unvetted hashing or BPF library dependency was added.

### Program correctness

- ABI size and offset assertions cover every used `bpf_attr` arm.
- Big-endian targets fail at compile time.
- `BPF_ST_MEM32` is opcode `0x62` and writes at correct stack offset.
- Null map lookup jumps directly to allow return.
- `bpf_setsockopt` helper `49` receives four-byte `SO_MARK` value.
- Helper success allows socket operation.
- Helper failure denies socket operation.
- `OwnedFd` closes map,
  program,
  and link descriptors through all error paths.

### Pin lifecycle

Canonical cgroup path bytes are encoded injectively as chunked hexadecimal components.
This avoids slash replacement collisions,
bpffs-reserved dots,
and component-length overflow for supported paths.

The loader verifies:

- `/sys/fs/bpf` has bpffs magic `0xcafe4a11`;
- bpffs is a distinct mount boundary;
- staging and final directories are on same filesystem.

Four new links are created in staging.
First attach uses rename.
Replacement uses `renameat2(RENAME_EXCHANGE)`.
Failed attachment removes earlier new pins and preserves prior complete set.
Detach removes only four exact expected names,
then calls `rmdir` so unrelated entries block removal.

### Linux SELinux pin regression

Kernel `7.1.3-ogc5.1.fc44.x86_64` returns `EINVAL` from `BPF_OBJ_PIN`
because SELinux initializes inode security state before
checking `SBLABEL_MNT` on bpffs.
The package detects typed `BPF_OBJ_PIN EINVAL` without diagnostic-string matching.

On affected kernels it uses detached descriptor keeper:

- candidate creates all links before readiness;
- candidate exits on parent-pipe EOF before commit;
- parent persists transition state before sending `COMMIT`;
- child writes commit marker before `COMMITTED`;
- recovery rejects uncommitted candidate and preserves prior holder;
- recovery adopts committed candidate;
- detach validates PID,
  start time,
  full command,
  mark,
  and cgroup;
- shutdown escalates validated process from `SIGTERM` to `SIGKILL` when required;
- removed cgroups still map to lexical state key for cleanup.

See `doc/troubleshooting/linux-bpffs-selinux-object-pin-einval.md` for source trace,
reproduction,
upstream fix,
and recovered-kernel comparison.
Kernel `7.2.0-ogc6.1.fc44.x86_64` pins links successfully.
Fallback-specific debug functional tests inject typed object-pin failure so descriptor-keeper coverage remains
deterministic.
Separate public-CLI lifecycle test retains native pin-path coverage.

## Ghostty, Steam, Helium, Pale Moon, Firefox Nightly, ChatGPT, Interpreter, and Qure coverage

`wg-quicker` starts Rust watcher only after bypass route exists.
It stops watcher before removing bypass routing.
Changed config without `ExemptMark` still stops watcher when persisted bypass state proves prior ownership.

Every `up` whose parsed config omits `ExemptMark` emits a non-fatal warning before network mutation.
It states that Ghostty,
 Steam,
 Helium,
 Pale Moon,
 Firefox Nightly,
 ChatGPT,
 Interpreter,
 and Qure will use the tunnel.
It instructs the user to add `ExemptMark = 100` under `[Interface]`,
then bring the interface down and up again so application exemptions attach.
`down` does not emit this warning.

Target UID precedence:

1. `WG_QUICKER_EXEMPT_UID`;
2. `SUDO_UID`;
3. non-root effective UID.

Direct root execution without explicit or sudo identity fails closed.
A non-root `wg-quicker` process relaunches exact current Node runtime and CLI bundle through sudo before config access.
This avoids root-only config `EACCES` and sudo `secure_path` lookup failure while preserving `SUDO_UID`.
A private bounded caller-context file carries allowlisted home,
cache,
token,
runtime,
UID,
and companion settings through sudo `env_reset`.
Root child validates file type,
ownership,
mode,
link count,
size,
schema,
and caller identity before applying it.

Before network mutation,
application exemption command resolves to exact configured path,
paired repository Rust build,
privileged installed command,
or captured caller-path command.
Missing configured companion fails before interface creation.

Watcher behavior:

- watches existing user `app.slice`;
- installs inotify before first scan;
- attaches Ghostty main service;
- attaches every existing Ghostty surface scope;
- drains queued creation events and rescans before readiness;
- attaches future Ghostty surface scopes;
- identifies Steam's `app-steam@*.service` immediately;
- identifies Helium Chrome application-ID service immediately;
- identifies Firefox Nightly's `app-firefox\x2dnightly@*.service` immediately;
- identifies ChatGPT's `app-chatgpt@*.service` immediately;
- identifies Interpreter's `app-interpreter@*.service` immediately;
- identifies Qure's `app-qure@*.service` immediately;
- maps live Helium,
  renderer,
  zygote,
  and crashpad executables to current cgroups;
- maps exact `palemoon` and `palemoon-bin` executable names to current cgroups;
- maps exact `firefox` and `firefox-bin` names under a `firefox-nightly` install directory to current cgroups;
- maps every executable inside ChatGPT's root-owned `/usr/lib/chatgpt` package tree,
  including the `resources/codex` app-server that application spawns,
  to current cgroups;
- maps every `interpreter`-prefixed executable,
  including the AppImage file and the bundled `interpreter-*` agents,
  to current cgroups;
- maps every `qure`-prefixed executable,
  including the AppImage file,
  the mounted Electron image,
  and the agents its bundled CLI wrappers exec,
  to current cgroups;
- periodically rescans processes entering existing cgroups;
- retains known process-discovered cgroups through process restarts until cgroup removal;
- holds links directly for watcher lifetime;
- drops every link on validated watcher shutdown.

Original read-only host audit found all then-current targets:

- all `11` Ghostty service and surface cgroups were listed;
- all `16` live Helium executable processes mapped to listed cgroups;
- no then-current target was missing.

Follow-up read-only audit after the Steam extension found:

- live Steam client and helper processes shared one `app-steam@*.service` cgroup;
- rebuilt `wg-quicker-exempt list-targets` included that service alongside current Ghostty and Helium targets.

Pale Moon follow-up audit found:

- installed `palemoon` and `palemoon-bin` files share same ELF build ID;
- live `/proc/<pid>/exe` resolved to installed lowercase `palemoon` path;
- isolated disposable app-slice scope appeared in rebuilt `wg-quicker-exempt list-targets` output;
- refreshed live watcher marked TCP4,
  TCP6,
  UDP4,
  and UDP6 sockets created in disposable Pale Moon cgroup with `8888`;
- tunnel and real application cgroup marking remained untouched.

Firefox Nightly follow-up verification found:

- installed launcher resolves to `/var/home/user/.local/opt/firefox-nightly/firefox`;
- transient scope status showed launcher and child commands under same install directory as `firefox-bin`;
- rebuilt `wg-quicker-exempt list-targets` included a disposable app-slice scope running actual Firefox Nightly;
- normal KDE desktop launch created `app-firefox\x2dnightly@*.service` from `firefox-nightly.desktop`;
- `/proc/<pid>/exe` for desktop-launched Nightly resolved to installed `firefox-bin`;
- rebuilt target listing included exact desktop service and omitted running Firefox ESR service;
- both Nightly verification processes ended;
- disposable headless profile was removed without changing live tunnel or application cgroups.

ChatGPT and Interpreter follow-up verification found:

- a disposable `app.slice` service running ChatGPT's own bundled `resources/cua_node/bin/node` appeared in
  rebuilt `wg-quicker-exempt list-targets` output,
  while a sibling service running `/usr/bin/sleep` did not;
- normal KDE desktop launch created `app-chatgpt@<hex>.service` from `chatgpt.desktop`;
- ChatGPT's Electron main process then moved itself into a Chromium-created
  `app-org.chromium.Chromium-<pid>.scope`,
  which also held the spawned `resources/codex` app-server,
  so one running application occupied two cgroups;
- `19` live ChatGPT executables resolved to `/usr/lib/chatgpt/ChatGPT`,
  `resources/codex`,
  and `browser_crashpad_handler`;
- normal KDE desktop launch created `app-interpreter@<hex>.service` from `interpreter.desktop`,
  and the AppImage created a second `app-interpreter-<pid>.scope` holding its `interpreter-app-server`
  and `bwrap` children;
- `9` live Interpreter executables resolved to `~/AppImages/interpreter.appimage`,
  `/tmp/.mount_interp*/interpreter`,
  and `/tmp/.mount_interp*/resources/interpreter-app-server`;
- all `28` matched processes mapped to listed targets,
  and the rebuilt listing added exactly those four agent cgroups to the five-target baseline;
- privileged disposable-cgroup protocol coverage passed on the current kernel;
- both applications were stopped,
  failed unit state was reset,
  and the listing returned to the exact baseline with no matched process left.

Process discovery attaches entire current cgroup.
Sibling processes sharing Helium,
Pale Moon,
Firefox Nightly,
ChatGPT,
Interpreter,
or Qure cgroup also receive exemption until cgroup disappears or watcher stops.
A newly started process-discovered application can create sockets before next 250-millisecond rescan;
applications present at watcher startup are attached before readiness.

The historical `mx-que-mx1` watcher refresh used latest rebuilt companion at that time,
its persisted mark,
and its desktop UID.
Readiness handshake succeeded without restarting WireGuard interface or changing live routing.
No real application cgroup was marked during read-only audits.
State-mutating watcher tests used disposable cgroups.

After disposable verification passed,
active `gb-lon-gb2` watcher was replaced through rebuilt release companion with existing key,
mark `8888`,
and desktop UID `1000`.
New watcher start time replaced August process,
and `/proc/<pid>/exe` had same device and inode as rebuilt release binary.
Watcher readiness completed without restarting WireGuard interface or changing live routes.
Current target listing still excludes running Firefox ESR service.
Disposable exact Nightly-named cgroup created under watched `app.slice` received mark `8888` on TCP4,
TCP6,
UDP4,
and UDP6 socket probes from active watcher.
Fixture cgroup and probe were removed;
watcher remained active with empty error log.

## ChatGPT and Interpreter clearnet coverage

Both applications reach clearnet through two independent mechanisms as of 2026-10-06.

Socket marking matches two shapes per application.
ChatGPT matching accepts every executable inside the root-owned `/usr/lib/chatgpt` package tree,
because that application spawns `resources/codex`,
`resources/rg`,
and `resources/cua_node/bin/node` agents that can live in a cgroup of their own.
Interpreter matching accepts every executable whose name begins with `interpreter`,
covering `~/AppImages/interpreter.appimage`,
the mounted `/tmp/.mount_interp*/interpreter` Electron image,
the bundled `interpreter-*` agents,
and the same vendor's mise-installed terminal agent.
Both applications also match their observed desktop services.
Interpreter's own `app-interpreter-<pid>.scope` deliberately stays executable discovery's responsibility,
since its name derives from the executable rather than the desktop entry.

`AllowedIPs` generation gained the endpoints both applications use,
measured with the built `wg-allowedips` command against the value generated before and after the edit.
Sixteen of seventeen candidate hostnames left the generated value on the first regeneration:
`api.openai.com`,
`auth.openai.com`,
`chat.openai.com`,
`codex-portals.api.openai.com`,
`realtime.chatgpt.com`,
`learn.chatgpt.com`,
`web-sandbox.oaiusercontent.com`,
`images.openai.com`,
`developers.openai.com`,
`status.openai.com`,
`admin.openai.com`,
`statsigcdn.openai.com`,
`model-spec.openai.com`,
`privacy.openai.com`,
`oi-new-server.fly.dev`,
and `models.dev`.
Two measurements bound that IP-level path:

- `huggingface.co` sits behind CloudFront,
  whose answers rotated through three distinct address sets within ten minutes,
  so the first regenerated value covered none of the addresses a later client resolved
  and a second regeneration covered only its own snapshot.
  The cgroup exemption,
  not the `AllowedIPs` entry,
  keeps that traffic direct.
  The human chose to keep the entry anyway,
  at a measured cost of `650` networks and `21711` bytes of generated value,
  so a regeneration immediately before `up` still buys a working window.
- `AS4167`,
  `AS400243`,
  `AS401551`,
  and `AS401864` contributed no networks even after a fresh `IPINFO_TOKEN` refresh,
  so those four `AS` lines currently subtract nothing.
  `doc/troubleshooting/ipinfo-lite-missing-openai-anthropic-asns.md` traces that gap to the
  2026-10-06 IPinfo Lite generation,
  which holds five records total for both vendors.

Adding the hostnames grew the generated value from `4022` to `6517` networks,
because every removed host route splits the aggregate that contained it.
The route integration assertion still expected the pre-netavark `8888` recommendation
and now expects the reserved `100`.

Loading both live configs through `loadConfig` as root with the caller's home
expanded the directive to `/var/home/user/allowed.txt` and `/var/home/user/disallowed.txt`,
reported `exemptMark` `100`,
and generated `6602` networks for `mx-que-mx1`.
Membership probes against that generated value placed every added ChatGPT and Interpreter hostname
outside it,
kept `example.com` inside as a positive control,
and covered `4` of `12` then-current `huggingface.co` addresses,
which is the CloudFront rotation the human accepted when choosing to keep that entry.
The same run showed `sillylittleguy.orgi` in the disallowed input was a typo:
the corrected `sillylittleguy.org` resolves inside Anthropic's `2607:6bc0::/48`
and now generates as clearnet.
The four permanently dead Steam and Valve domain names remain,
as chosen,
and warn on every generation.

## Qure clearnet coverage

JetBrains Qure reaches clearnet through the same two independent mechanisms as of 2026-10-08.

Socket marking matches two shapes.
Qure ships as `~/AppImages/qure.appimage`,
which mounts read-only at `/tmp/.mount_qure.<random>` and runs the Electron image `qure-ai-assistant`.
Matching accepts every executable whose name begins with `qure`,
covering the AppImage file,
the mounted image,
the bundled `resources/qure_pytest/qure_pytest` runner,
and every agent the bundled CLI wrappers spawn,
because `~/.config/Qure/bin/preferred/browser-use`,
`~/.config/Qure/bin/preferred/playwright-cli`,
and `~/.config/Qure/bin/fallback/node` are shell scripts that `exec` that same mounted image.
Qure's `qure.desktop` entry creates `app-qure@<hex>.service`,
and the AppImage's own executable-named scope stays executable discovery's responsibility,
since that name derives from the executable rather than the desktop entry.
`chrome_crashpad_handler`,
`chrome-sandbox`,
and `AppRun` inside the mount carry Chromium-generic names,
so they bypass only while sharing a matched cgroup.
Qure's JVM agent ships as `resources/qure-agent-jvm/target/qure-agent.jar`,
which the AppImage bundles no Java executable to run,
so such an agent in a cgroup of its own relies on the `AllowedIPs` path instead.

`AllowedIPs` generation gained the endpoints Qure was measured using:
`api.quretests.com` and `quretests.com` from Chromium's `Network Persistent State` and the bundle,
`o447951.ingest.sentry.io` from the bundle's DSN and two live connections,
`eu.i.posthog.com` and `eu-assets.i.posthog.com` from live connections,
`download.jetbrains.com` and `download-cdn.jetbrains.com` from Sentry network breadcrumbs
and two live CloudFront connections identified by distribution,
and `us.i.posthog.com` and `us-assets.i.posthog.com` from the same telemetry SDK's other region.
Regenerating with the built command moved the value from `6537` to `9201` networks
and from `156582` to `245116` bytes,
with no new resolver or ASN warning.
Every then-current address of `api.quretests.com`,
`quretests.com`,
`o447951.ingest.sentry.io`,
`eu-assets.i.posthog.com`,
and `us-assets.i.posthog.com` left the generated value,
while `1` of `16` `eu.i.posthog.com` answers,
`9` of `16` `us.i.posthog.com` answers,
and `8` of `12` answers for each JetBrains download host rotated back in between generation and probe.
That is the AWS ELB and CloudFront decay `huggingface.co` documented,
and the two JetBrains hosts cost `1305` of the added networks for that partial coverage.
The human was shown both measured costs and chose to keep every entry:
the two JetBrains download hosts despite their rotation,
and the two United States PostHog hosts despite never being observed,
so this region carries each measured and bundle-referenced Qure endpoint.
`example.com` stayed inside as a positive control.
`o447951.ingest.sentry.io` was already clearnet before this change,
because the pre-existing `o33249.ingest.sentry.io` entry resolves to the same two
Google anycast addresses;
the explicit entry keeps Qure covered if the OpenAI region ever loses it.
An earlier draft of this region omitted `download.jetbrains.com`
on the strength of one bundle reference and no observed connection.
A later capture of Qure's own established sockets falsified that:
it held a connection to `2600:9000:287f:1e00:12:7c44:15c0:93a1`,
whose CloudFront low half matches every `download.jetbrains.com` answer,
and a second one to `download-cdn.jetbrains.com`'s distribution.
Both hosts are in the region now.

Read-only target enumeration against the running Qure launch proved the discovery delta.
The binary the active `mx-que-mx1` watcher still ran listed `16` targets;
the rebuilt companion listed exactly two more:
`app-org.chromium.Chromium-3862000.scope`,
holding Qure's Electron main process,
and `flatpak-session-helper.service`.
That launch came through Gearlever,
so `flatpak-spawn --host` placed `8` of Qure's `9` processes,
including the network service that owns every measured connection,
in a shared user-service cgroup that also holds `flatpak-session-helper`,
`p11-kit-server`,
and two `p11-kit-remote` processes.
Marking it therefore exempts those daemons,
and every later host-spawned flatpak child,
until that service cgroup disappears.
A launch from `qure.desktop` instead gives Qure its own `app-qure@<hex>.service` and scopes.

### Qure deployment on 2026-10-08

The human chose the relaunch path and authorized terminating Qure.
Teardown order mattered:
terminating the AppImage FUSE server and the Electron main together left the main process in
uninterruptible `D` state after the FUSE mount disappeared under it,
so two further `SIGTERM` deliveries did nothing and `SIGKILL` was required.
The second teardown terminated the Electron main first,
and every Qure process,
 including the runtime,
 exited cleanly within eight seconds.
See `doc/troubleshooting/appimage-fuse-teardown-order-d-state.md`.

With no `qure`-prefixed process left,
the rebuilt `list-targets 1000` returned the running watcher binary's set plus one Ghostty surface scope
created in the meantime by agent shell activity,
and listed neither `flatpak-session-helper.service` nor Qure's former Chromium scope,
so the refresh could not mark that shared service.
`watch-start mx-que-mx1 100 1000` then replaced the watcher:
old pid `50227` from 2026-10-06 exited,
new pid `3888199` started,
`/proc/3888199/exe` reported the rebuilt release binary's exact device and inode,
the recorded state became `1,3888199,21494257,100,1000`,
`watch.log` stayed at `0` bytes,
`ip rule show` was byte-identical before and after,
and no pin directory appeared because the watcher holds its links directly.

Relaunch needed a dedicated cgroup.
`gtk-launch qure` from an agent shell placed ten Qure processes in that shell's own
`app-ghostty-surface-transient-3862354.scope`,
which is already an exempt Ghostty target,
so Qure would have borrowed a terminal's exemption and died with it.
This launch shape produced the desktop-entry unit instead:

```sh
systemd-run --user --collect --slice=app.slice \
  --unit='app-qure@9f3c1d2e4b5a6c7d8e9f0a1b2c3d4e5f' \
  --setenv=DESKTOPINTEGRATION=1 \
  /home/user/AppImages/qure.appimage --no-sandbox
```

The relaunched Qure occupied exactly two cgroups:
that service,
 holding eight processes including the network service,
and `app-org.chromium.Chromium-3889366.scope`,
 holding the Electron main.
`list-targets 1000` listed both and no shared service.
This launch's mount was `/tmp/.mount_qure.aCNyBUv`,
which confirms the runtime builds that name from `.mount_`,
the first six characters of the AppImage filename,
 here `qure.a`,
and six random characters;
because a rename changes it,
 matching keys off the executable name rather than the mount path.

Marking was verified twice.
A disposable `app.slice` service running a copy of the host `python3` renamed to `qure-mark-probe`
read `SO_MARK` `100` on TCP4,
 TCP6,
 UDP4,
 and UDP6 sockets it created after attach,
while the same probe as plain `python3` in `background.slice` read `0` on all four.
Then Qure's own sockets were captured:
fifteen packets between its established source ports and Sentry's `2600:1901:0:5e8a::`
and the JetBrains CloudFront address crossed `wlp9s0` directly,
while a simultaneous 45-second `mx-que-mx1` capture on the same four Qure peers recorded
`0 packets captured`,
and a `curl https://example.com/` run from `background.slice` during the same window put
25 inner packets on `mx-que-mx1` from the tunnel address `fd00:4956:504e:ffff::ac11:aaaa`.
Both transient probe units used `--collect` and left no unit,
 cgroup,
 or mount behind;
the only remaining Qure unit is the application's own service.

## Verification evidence

### AllowedIPs packages

Passed:

```text
mise run //package/module/wg-allowedips:buildAndTest
mise run //package/module/wg-allowedips:lint:types
mise run //package/module/wg-allowedips:lint:oxlint
mise run //package/cli/wg-allowedips:buildAndTest
mise run //package/cli/wg-allowedips:lint:types
mise run //package/cli/wg-allowedips:lint:oxlint
```

Built CLI tests cover minimized output,
complete subtraction,
missing options,
unknown options,
positional input,
unreadable files,
parser failures,
DNS absence warnings,
and empty allowed input.

### OpenTofu consumer

Passed:

```text
mise run //package/config/tofu:test
mise run //package/config/tofu:lint
```

OpenTofu validation succeeded.
ASN cache tests covered filtering,
atomic replacement,
corrupt cache rejection,
and malformed records.

### `wg-quicker`

Passed:

```text
mise run //package/cli/wg-quicker:buildAndTest
mise run //package/cli/wg-quicker:lint:types
mise run //package/cli/wg-quicker:lint:oxlint
mise run //package/cli/wg-quicker:test:integration:bypass
mise run //package/cli/wg-quicker:test:integration:route
```

Disposable netns integration covers:

- table and preference collisions;
- literal `/0` and two-`/1` full-tunnel forms;
- marked and unmarked IPv4 and IPv6 routing;
- missing physical defaults;
- single-family fail-closed defaults;
- watcher resynchronization and monitor restart;
- operation-lock contention;
- wrong-owner state retention;
- unowned-default rejection;
- unrelated route preservation;
- changed-config teardown;
- built CLI `up` and `down`;
- exact IPv4 and IPv6 IVPN conflict rejection before interface creation;
- physical connected-route precedence over broad dual-stack policy prefixes;
- an IPv4 endpoint covered by non-default `AllowedIPs`;
- marked physical endpoint routing and bidirectional WireGuard transfer;
- actionable missing-`ExemptMark` warning on `up` and warning absence on `down`.

### `wg-quicker-exempt`

Passed:

```text
mise run //package/cli/wg-quicker-exempt:buildAndTest
mise run //package/cli/wg-quicker-exempt:test:functional
```

Privileged disposable-cgroup tests cover:

- TCP4 connect;
- TCP6 connect;
- UDP4 sendmsg;
- UDP6 sendmsg;
- process-exit persistence;
- repeated attach with changed mark;
- exact detach;
- partial-link rollback;
- failed candidate replacement preserving prior holder;
- pre-transition parent death;
- committed and uncommitted transition recovery;
- wrong-owner state retention;
- removed-cgroup cleanup;
- existing Firefox Nightly and Ghostty service marking;
- Firefox ESR service remaining unmarked;
- future Ghostty scope coverage;
- public watcher start and stop lifecycle.

Unit tests additionally cover target-name precision,
fake procfs Helium,
Pale Moon,
Firefox Nightly,
ChatGPT,
Interpreter,
and Qure mapping,
Firefox Nightly service,
executable,
channel,
and install-directory boundaries,
Pale Moon case and suffix near misses,
ChatGPT package-tree and sibling-directory boundaries,
Interpreter family,
stem,
and case near misses,
Qure family,
stem,
case,
and Chromium-generic-helper boundaries,
ChatGPT,
Interpreter,
and Qure desktop-service names,
path-key injectivity,
exact cleanup,
atomic exchange,
failed-exchange preservation,
instruction width,
and control-flow offsets.

### Documentation and workspace state

Markdown lint passed for affected documentation.
`git diff --check` passed before each commit.
No watcher,
disposable cgroup,
runtime state,
or unexpected pin remained after final verification.

## Operational notes

- Build `wg-quicker-exempt` release artifact or install it in privileged or caller `PATH`.
- Use `WG_QUICKER_EXEMPT_COMMAND` for an explicit executable;
  launcher resolves it before network mutation.
- Use root-owned installed artifacts when workspace integrity is not trusted.
- Use `ExemptMark = 100` or another positive mark sharing no bit with a reserved mask in `[Interface]`.
- Invoke `wg-quicker` normally;
  it uses sudo before config access and preserves original identity through `SUDO_UID`.
- Set `WG_QUICKER_EXEMPT_UID` explicitly for direct root or service execution.
- Use `wg-quicker-exempt list-targets <uid>` for read-only coverage audit.
- Keep IVPN Desktop split tunneling disabled while this tunnel is active.
- Ask before bringing up `mx-que-mx1` or mutating its live routes.
