# wg-quicker design

Replacement for `wg-quick` that parses the config in TypeScript instead of bash.
Motivation:
 bash `${var%%*([[:space:]])}` trimming is quadratic,
 so a ~91KB
`AllowedIPs` line (4102 prefixes from `wg-allowedips`) hangs `wg-quick up`
indefinitely at 100% CPU.
 The `wg` binary parses the same line in ~1ms.

## Decision

Keep the user's existing config as-is (full expanded `AllowedIPs`).
 Parse only the
`[Interface]` keys wg-quick consumes,
 and pass the raw `[Peer]`/`AllowedIPs` block
straight to `wg addconf` (which is exactly what wg-quick does with `WG_CONFIG`).

## What wg-quick `up` does (reverse-engineered from `/usr/bin/wg-quick`)

1. `parse_options`:
    read conf,
    collect `[Interface]` `Address`/`MTU`/`DNS`/`Table`/
   hooks;
    append every non-Interface line to `WG_CONFIG`.
2. Refuse if interface exists;
    set trap to `del_if` on failure.
3. `add_if`:
   `ip link add dev IF type wireguard`.
4. `PreUp` hooks.
5. `set_config`:
   `wg addconf IF < WG_CONFIG` (peers,
    keys,
    endpoints,
    allowed-ips).
6. `add_addr` per `Address` (v4/v6 by presence of `:`).
7. `set_mtu_up`:
    MTU from conf,
    else endpoint route MTU,
    else default 1500,
    minus 80.
8. `set_dns`:
    wg-quick uses `resolvconf`;
    we use `resolvectl` (systemd-resolved).
9. For each allowed-ip prefix (sorted longest-first):
   `add_route`.
    A `/0` prefix
   triggers `add_default`:
    pick free table (51820),
   `wg set fwmark`,
    add
   `not fwmark TABLE table TABLE` + `table main suppress_prefixlength 0` ip rules,
   add default route in TABLE,
    and install kill-switch firewall (nft or iptables).
   Non-/0 prefixes get a plain route only if not already covered.
10. `PostUp` hooks;
     clear trap.

## What wg-quick `down` does

`PreDown`,
 optional `save_config`,
`del_if` (removes DNS,
 firewall,
 the /0 policy
rules when set,
 then `ip link delete dev IF`),
`unset_dns`,
`remove_firewall`,
`PostDown`.

## wg-quicker scope (v1)

- Subcommands:
  `up`,
  `down`.
   (`save`/`strip` omitted;
   not needed for the bug.)
- Full-tunnel path only matters:
   when allowed-ips contains a `/0`,
   replicate
  `add_default` policy routing + nft kill-switch.
   Non-/0 prefixes replicate
  `add_route` covered-check.
- DNS via `resolvectl dns`/`domain`/`default-route`,
   reverted on down.
- Hooks `PreUp`/`PostUp`/`PreDown`/`PostDown` executed with `%i` expansion.
- `Table = off` supported (skip all route/rule).
   Fixed numeric `Table` supported.
- Idempotent teardown;
   errors roll back via `down` on failed `up`.

## Per-app exclusion (cgroup-BPF marker)

Goal:
 exempt **all of Ghostty** (shells live in transient `app-ghostty-surface-transient-*.scope`
cgroups,
 not the service cgroup) and **Helium** (AppImage,
 needs its own delegated cgroup)
from the tunnel.
 Validated empirically in a root netns:

- A cgroup-BPF `BPF_CGROUP_INET4_CONNECT` (+ INET6,
   UDP sendmsg variants) program sets
  `SO_MARK` on sockets via `bpf_setsockopt(ctx, SOL_SOCKET, SO_MARK, &mark, 4)` using a value
  read from a `BPF_MAP_TYPE_ARRAY` (map_lookup → value ptr;
   setsockopt rejects a bare map_ptr).
- `ip rule add fwmark <EXEMPT> table main pref 100` (priority **below** the tunnel's
  `not fwmark <TUNNEL> table <TUNNEL>` rule and the `suppress_prefixlength 0` rule) routes
  marked traffic via the main table,
   bypassing the tunnel.
- Proven:
   exempt-cgroup socket got SO_MARK=8888;
  `ip route get 8.8.8.8 mark 8888` → physical
  link,
   unmarked → tunnel.

### Cgroup targeting (final: no Ghostty slice change)

User constraint:
 do NOT change Ghostty's slice/launch.
 So we attach to Ghostty's **existing**
cgroups and cover future per-window scopes dynamically.
 Helium uses its own delegated scope.

- Ghostty cgroups to cover:
   the service `app-com.mitchellh.ghostty@.service` AND every
  `app-ghostty-surface-transient-*.scope` (shells/commands run in those,
   a fresh scope per
  window).
   Attach the marker to each existing one,
   then watch the parent `app.slice` with
  **inotify** (`IN_CREATE|IN_MOVED_TO`) to attach to new `app-ghostty-surface-*` scopes as they
  appear,
   plus a reconciliation rescan after installing the watch to close the create race.
  Race window (a brand-new window's first socket before attach) is bounded and documented.
- Helium:
  `systemd-run --user --scope --collect --unit=helium -- /home/user/AppImages/helium.appimage`
  gives one stable cgroup (`helium.scope`) all browser/renderer/zygote processes inherit;
   attach
  once.
   (Slice change accepted for Helium;
   only Ghostty's is off-limits.)
- Marker:
   cgroup-BPF program attached via BPF_LINK_CREATE (pinned for clean teardown) to
  connect4/6 + udp4/6_sendmsg;
   sets SO_MARK from a one-entry array map.
- Routing:
  `ip rule add fwmark <EXEMPT> table main pref 100` (priority below the tunnel rules)
  routes marked traffic direct.
   SO_MARK at sendmsg is persistent per-socket,
   covering UDP/QUIC.
  Pre-existing connections are not retroactively rerouted.
- Config:
   optional `ExemptMark = <n>` in `[Interface]`.
   When set,
  `up` installs the fwmark rule
  before the tunnel rules and invokes the sibling `wg-quicker-exempt` binary to attach the
  marker to the configured cgroups;
  `down` removes the rule.
   Marker binary is built from
  `package/wg/cli-quicker-exempt` (Rust,
   raw bpf syscalls,
   pins links under
  `/sys/fs/bpf/wg-quicker-exempt/`).

### Loader

Self-contained C program using raw `bpf()` syscalls (no libbpf/bpftool;
 Bazzite is rpm-ostree
Atomic,
 clang+libbpf runtime present but not headers/bpftool).
 Loads connect4/6 + sendmsg hooks,
sets the mark map,
 attaches to each target cgroup fd.

## Out of scope (documented)

- Userspace `wireguard-go` fallback (kernel module present here).
- `resolvconf` (openresolv) path;
   target system uses systemd-resolved.
- `SaveConfig` (writes conf back) — not needed,
   conf is source of truth.

## Package

- `@monochromatic-dev/wg-cli-quicker`,
   dir `package/wg/cli-quicker` (SGD).
- Mirrors `wg-allowedips` scaffolding:
   rolldown node build,
   module-logger,
  module-test,
   mise tasks extending shared build/lint/test.
- Entry `src/index.ts` (bin `wg-quicker`),
   logic in `src/` modules,
   tests
  `*.unit.test.ts` importing built `dist`,
   plus a root-netns integration test.
