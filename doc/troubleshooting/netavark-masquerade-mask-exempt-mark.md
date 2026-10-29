# netavark 1.17.2 masquerades any packet marked `0x2000`, so `ExemptMark = 8888` breaks IPv4 loopback

`netavark`, the podman network backend, installs one unconditional postrouting
masquerade rule keyed on the mark bit `0x2000`.
`wg-quicker`'s documented application-exemption mark `8888` is `0x22b8` and carries that bit,
so every socket the exemption watcher marks is source-NATed,
IPv4 loopback included.
A Gradle daemon bound to `127.0.0.1` then sees the Wi-Fi address as its client's peer and resets the connection.

Recorded from [issue 553](https://github.com/Aquaticat/Monochromatic/issues/553).

## Symptom

Every Gradle build launched from an exempted application cgroup under the watched `app.slice`
(Ghostty,
 Steam,
 Helium,
 Pale Moon,
 Firefox Nightly,
 ChatGPT,
 Interpreter,
 and Qure)
fails with:

```text
Could not dispatch a message to the daemon.
```

`~/.gradle/daemon/9.5.1/daemon-<pid>.out.log` records the daemon side:

```text
2026-09-17T20:31:39.540-0400 [DEBUG] [org.gradle.internal.remote.internal.inet.TcpIncomingConnector] Listening on [f8297e94-02e8-4fd2-a668-f1b17ae168cd port:37051, addresses:[localhost/127.0.0.1]].
2026-09-17T20:31:39.565-0400 [ERROR] [org.gradle.internal.remote.internal.inet.TcpIncomingConnector] Cannot accept connection from remote address /192.168.253.108:38024.
```

The daemon listened on `127.0.0.1`,
the client connected to `127.0.0.1`,
and the accepted socket reported the Wi-Fi address as its peer.

Three properties distinguish this from an ordinary Gradle or JVM fault:

- The client's own `getsockname()` still reports `127.0.0.1`;
  only the peer the server reads is rewritten.
- IPv6 loopback is unaffected:
  the same probe over `::1` reports `::1`.
- An unmarked socket is untouched:
  the same probe run without `SO_MARK` reports `127.0.0.1`,
  which is why a shell outside a watched cgroup cannot reproduce the failure and
  the fault looks intermittent per terminal rather than system-wide.

## Root cause

### The mark

`wg-quicker` enables application exemptions with `ExemptMark` under `[Interface]`.
`package/wg/cli-quicker/README.md` and the `up` warning both recommended `8888`.
`wg-quicker-exempt` loads a `BPF_PROG_TYPE_CGROUP_SOCK_ADDR` program on the
`connect4`, `connect6`, `udp4_sendmsg`, and `udp6_sendmsg` hooks of each watched
application cgroup;
the program calls `bpf_setsockopt(SOL_SOCKET, SO_MARK, mark)` on every socket,
with no destination test.

```rust
// package/wg/cli-quicker-exempt/src/bpf.rs:110-116 (instruction builder)
BpfInsn::new(BPF_JEQ_IMM, 0, 0, 9, 0),
BpfInsn::new(BPF_MOV64_REG, 1, 6, 0, 0),
BpfInsn::new(BPF_MOV32_IMM, 2, 0, 0, SOL_SOCKET),
BpfInsn::new(BPF_MOV32_IMM, 3, 0, 0, SO_MARK),
BpfInsn::new(BPF_MOV64_REG, 4, 0, 0, 0),
BpfInsn::new(BPF_MOV32_IMM, 5, 0, 0, SO_MARK_VALUE_SIZE),
BpfInsn::new(BPF_CALL, 0, 0, 0, FN_SETSOCKOPT),
```

Marking loopback sockets is harmless on its own.
Policy routing never sends them anywhere unusual,
because the `local` table rule at priority `0` resolves `127.0.0.1` before any
fwmark rule is consulted:

```console
$ ip rule show
0:	from all lookup local
50:	from all fwmark 0x22b8 lookup 52000 proto 201
32764:	from all lookup main suppress_prefixlength 0
32765:	not from all fwmark 0xca6c lookup 51820
32766:	from all lookup main
32767:	from all lookup default

$ ip route get 127.0.0.1 mark 0x22b8
local 127.0.0.1 dev lo table local src 127.0.0.1 mark 0x22b8 uid 1000
```

An earlier reading in issue 553 concluded the rewrite was
"a masquerade or SNAT rule without an output-interface restriction,
likely added by the VPN client that owns the `gb-lon-gb2` interface",
and proposed inspecting the VPN's rules.
That reading was wrong about the owner.
`wg-quicker` installs no NAT at all;
its only nftables table is a kill switch plus two mangle rules:

```ts
// package/wg/cli-quicker/src/tunnel-firewall.ts:70-75
statements.push(
  `add chain inet ${nftable} postmangle { type filter hook postrouting priority -150; }`,
  `add rule inet ${nftable} postmangle meta l4proto udp meta mark ${String(table,)} ct mark set mark`,
  `add chain inet ${nftable} premangle { type filter hook prerouting priority -150; }`,
  `add rule inet ${nftable} premangle meta l4proto udp meta mark set ct mark`,
);
```

Grepping the whole live ruleset for source rewriting returns exactly one rule,
and it belongs to podman:

```console
$ sudo nft --numeric list ruleset | grep -E '\b(masquerade|snat|dnat|redirect)\b'
		meta mark & 0x00002000 == 0x00002000 masquerade
```

### The rule

netavark defines the bit as a bare constant:

```rust
// netavark v1.17.2 src/firewall/nft.rs:32
const MASK: u32 = 0x2000;
```

and installs a postrouting rule matching it with no interface, address, or
namespace restriction:

```rust
// netavark v1.17.2 src/firewall/nft.rs:136-154
// Postrouting: meta mark & 0x2000 == 0x2000 masquerade
batch.add(make_rule(
    Cow::Borrowed(POSTROUTINGCHAIN),
    Cow::Owned(vec![
        stmt::Statement::Match(stmt::Match {
            left: expr::Expression::BinaryOperation(Box::new(
                expr::BinaryOperation::AND(
                    expr::Expression::Named(expr::NamedExpression::Meta(expr::Meta {
                        key: expr::MetaKey::Mark,
                    })),
                    expr::Expression::Number(MASK),
                ),
            )),
            right: expr::Expression::Number(MASK),
            op: stmt::Operator::EQ,
        }),
        stmt::Statement::Masquerade(None),
    ]),
));
```

The rule's only guard is the mark.
netavark's own packets reach it through `NETAVARK-HOSTPORT-SETMARK`,
which sets the bit with `meta mark set meta mark | 0x00002000`.
Any other subsystem that sets a mark sharing that bit is indistinguishable from
netavark's own traffic at this rule.

`8888` is `0x22b8`.
`0x22b8 & 0x2000 == 0x2000`, so the rule matches,
`Masquerade(None)` rewrites the source to the address the outbound route selects,
and a `127.0.0.1` listener reads `192.168.253.108`.

The rule persists for as long as podman's tables exist,
whether or not any container is running,
so the failure is continuous rather than intermittent.
What varies is which processes carry the mark,
which is why a build started from Ghostty fails while the same command from an
unwatched cgroup succeeds.

The iptables backend uses the same value:

```rust
// netavark v1.17.2 src/firewall/varktables/types.rs:36
const HEXMARK: &str = "0x2000";
```

## Verification

Versions under test:

- netavark `1.17.2-1.fc44.x86_64`, podman `5.8.4-1.fc44.x86_64`
  (source read at upstream tag `v1.17.2`, commit `bd7a464`).
- Gradle `9.5.1`, Temurin `21.0.12+8.0.LTS`.
- Kernel `7.2.0-ogc6.1.fc44.x86_64`.

Harness, which needs `CAP_NET_ADMIN` to set `SO_MARK`:

```python
# ~/temp/agent/marked-loopback-probe.py
import socket, threading, sys

MARK = int(sys.argv[1]) if len(sys.argv) > 1 else 8888
SO_MARK = 36

srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
srv.bind(('127.0.0.1', 0))
srv.listen(1)
port = srv.getsockname()[1]

result = {}
def accept():
    conn, peer = srv.accept()
    result['peer'] = peer
    conn.close()

t = threading.Thread(target=accept)
t.start()

cli = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
cli.setsockopt(socket.SOL_SOCKET, SO_MARK, MARK)
cli.connect(('127.0.0.1', port))
print(f'mark={MARK} local={cli.getsockname()}')
t.join(5)
print(f'mark={MARK} peer={result.get("peer")}')
cli.close()
srv.close()
```

Marks that keep loopback intact,
every one of them clear of bit `0x2000`:

```console
$ sudo python3 ~/temp/agent/marked-loopback-probe.py 0
mark=0 peer=('127.0.0.1', 43922)
$ sudo python3 ~/temp/agent/marked-loopback-probe.py 696
mark=696 peer=('127.0.0.1', 40646)
$ sudo python3 ~/temp/agent/marked-loopback-probe.py 51820
mark=51820 peer=('127.0.0.1', 35016)
$ sudo python3 ~/temp/agent/marked-loopback-probe.py 100
mark=100 peer=('127.0.0.1', 33960)
```

Marks that are rewritten,
every one of them carrying bit `0x2000`:

```console
$ sudo python3 ~/temp/agent/marked-loopback-probe.py 8888
mark=8888 local=('127.0.0.1', 40198)
mark=8888 peer=('192.168.253.108', 40198)
$ sudo python3 ~/temp/agent/marked-loopback-probe.py 8192
mark=8192 peer=('192.168.253.108', 52384)
$ sudo python3 ~/temp/agent/marked-loopback-probe.py 8193
mark=8193 peer=('192.168.253.108', 53312)
```

`696` is `8888` with only bit `0x2000` cleared,
which isolates that bit as the cause rather than the value.
The `mark=8888` case also shows the split the symptom section names:
`local` stays `127.0.0.1` while the peer the server reads is rewritten.

Marking the negative result as trustworthy needs the positive control above:
a probe that reports `127.0.0.1` for mark `0` only proves the harness works once
some mark in the same harness reports a rewritten peer.

## Verified workarounds

### Choose an `ExemptMark` clear of `0x2000`

```ini
[Interface]
ExemptMark = 100
```

Then `wg-quicker down <interface>` and `wg-quicker up <interface>`,
which reloads the BPF mark map and replaces the `ip rule` selector.

Tradeoffs:
the tunnel drops for the duration of the cycle,
every exempted application's existing connections are marked with the old value
until their sockets close,
and the value is only safe against the masks
`package/wg/cli-quicker/src/exempt-mark-reservation.ts` lists.
A future subsystem claiming low bits would need another change.

### Remove podman's nftables tables

`podman network reload --all`, or stopping podman and deleting `table inet netavark`,
removes the masquerade rule and unblocks loopback without touching the tunnel.

Tradeoffs:
published container ports stop working,
and netavark recreates the table on the next container start,
so this is a diagnostic step, not a fix.

### Build inside a private network namespace

`unshare --user --map-root-user --net` gives the build a clean loopback,
after which the artifact is installed with the host's `adb`.

Tradeoffs:
the build loses host network access,
`/root/.android` becomes unwritable and Gradle logs
`Unable to initialize metrics, ensure /root/.android is writable`,
and every invocation needs the wrapper.
This was the workaround issue 553 asked to retire;
the `ExemptMark` change retires it.

Pitfalls paid while the bridge was live,
recorded so a rebuild does not pay them again:

- Without `ip link set lo up` the namespace has no interface at all and Gradle
  fails with `Could not determine a usable wildcard IP for this machine.`
- The mapped-root UID makes JVMs resolve `user.home` to `/root`;
  export `HOME` and `GRADLE_USER_HOME`,
  and `ANDROID_SDK_HOME` plus `ANDROID_USER_HOME` for the debug keystore lookup.
  Gradle strips `-Duser.home` from `org.gradle.jvmargs`,
  so the env vars are the only lever.
- A bare `gradlew` inherits whatever `ANDROID_HOME` the shell exports;
  pin the install that carries `platforms/<compileSdk>` or the build fails with
  `Failed to find target with hash string`.
- mise's rust backend syncs the nightly channel over the network;
  bypass mise inside the namespace and call `gradlew` directly with `--offline`.
- Device-facing steps (`adb install`, captures) must run outside the namespace
  because the adb server socket lives in the host namespace.

## What does not work

- **Inspecting the VPN's rules for a masquerade.**
  Issue 553's suspected cause pointed at the VPN client owning `gb-lon-gb2`.
  `wg-quicker` installs no NAT rule at all,
  so `sudo nft list ruleset | grep -E 'masq|snat'` returns only netavark's rule.
- **Reading `ip route get 127.0.0.1 mark 0x22b8`.**
  It reports `dev lo src 127.0.0.1`,
  which looks like proof that the mark is harmless.
  The rewrite happens at the postrouting NAT hook,
  after the route lookup the command models,
  so route output cannot see it.
- **Running the loopback probe from an agent shell.**
  A shell under `app.slice/claude-code-bash` is not a watched cgroup,
  so its sockets carry no mark and the probe reports `127.0.0.1`.
  That null result says nothing about the failing terminal;
  the mark has to be set explicitly, as the harness above does.
- **Listing marked sockets with `ss --extended`.**
  iproute2 on this host prints `uid`, `ino`, `sk`, and `cgroup` but never a
  `fwmark` field,
  so `ss --tcp --all --extended | grep fwmark` returns zero matches whether or
  not sockets carry a mark.
  The zero is an artifact of the output format, not evidence.
- **Waiting for the rule to disappear.**
  It is created with podman's tables and survives with no containers running.

## Upstream filing decision

1. **Is it really upstream's fault?**
   No.
   netavark picks one bit, documents it in a named constant, and matches it.
   `wg-quicker` chose `8888` without checking whose bits it borrowed.
   The defect is in this repository's mark selection.
2. **Can upstream fix it?**
   In principle netavark could narrow the rule,
   but a mark-based masquerade is the standard nftables idiom for host-port
   publishing and any bit it picks is claimable by someone else.
3. **Are they supporting this use case?**
   Not applicable:
   there is no netavark use case here to support.
4. **Would the repo welcome our contribution?**
   Not reached.
5. **Will they likely fix it?**
   Not reached.
6. **Have we prototyped a minimal fix compatible with their architecture?**
   Not reached, and not applicable:
   the fix belongs on this side of the boundary.

Constraint 1 fails, so nothing is filed against `containers/netavark`,
and no issue draft is kept.
`.out-of-scope/` was read before this decision;
it holds no entry for netavark, podman, or mark collisions,
so the exemption path did not apply either.
No duplicate search was run against the netavark tracker,
because there is no upstream behavior to report:
the rule works as its source says it does.

## Repository fix

`package/wg/cli-quicker/src/exempt-mark-reservation.ts` lists the masks other
subsystems match on and rejects a colliding `ExemptMark` at config-parse time,
naming the value, the mask, its owner, and the replacement.
`RECOMMENDED_EXEMPT_MARK` is `100`,
and the `up` warning, `package/wg/cli-quicker/README.md`,
and `doc/handover/wg-quicker.md` recommend it in place of `8888`.

Loading a config that still carries `8888` now fails with:

```text
ConfigError: Invalid `ExemptMark' value `8888' (0x22b8): it shares bits with a packet mask another subsystem matches on.

Bit range 0x2000 belongs to netavark, the podman network backend (`MASK`, `src/firewall/nft.rs:32`): its `table inet netavark` postrouting rule `meta mark & 0x2000 == 0x2000 masquerade` carries no interface or address restriction, so it source-NATs every matching packet, IPv4 loopback included, and a server bound to `127.0.0.1` then reads the outbound interface address as its peer.

Pick a mark sharing no bit with those masks, such as `ExemptMark = 100' (0x64), then bring this interface down and up again so the watcher and policy rule carry the new mark. Keeping the current value requires removing the other subsystem instead: the collision is in its rules, not in this config file.
```

On this host `/etc/wireguard/gb-lon-gb2.conf` and `/etc/wireguard/mx-que-mx1.conf`
were changed to `ExemptMark = 100` in place, preserving inode, owner, mode, and
link count,
and `gb-lon-gb2` was cycled.
`ip rule` then shows `50: from all fwmark 0x64 lookup 52000 proto 201`,
the watcher runs as `wg-quicker-exempt __watch gb-lon-gb2 100 1000`,
the marked probe at `100` reports `127.0.0.1`,
the same probe at `8888` still reports `192.168.253.108`,
and `./gradlew help --no-daemon` succeeds in
`package/music-player/android-app` on the host.
