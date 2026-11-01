# OpenSnitch `reject` rules on nightly `cargo` paths fail crates.io downloads with `Could not connect to server`

## Symptom

On this host, `cargo` from some rustup nightly toolchains cannot download anything from crates.io.
The first sightings were the `1.101.0-nightly` toolchains `nightly-2026-10-17` and `nightly-2026-10-18`,
so the failure was first filed as a 1.101 regression.
That attribution is wrong: the version is a coincidence (see "What does not work").

Any registry operation fails after about 15 ms, even with `CARGO_NET_RETRY=0`:

```console
$ cargo +nightly-2026-10-18 fetch
    Updating crates.io index
warning: spurious network error (3 tries remaining): [7] Could not connect to server (Failed to connect to index.crates.io:443 after 123 ms: Could not connect to server)
error: failed to get `itoa` as a dependency of package `probe v0.0.0 (/var/mnt/encrypted/temp/agent/cargo-net/crate)`
Caused by:
  download of config.json failed
Caused by:
  [7] Could not connect to server (Failed to connect to index.crates.io:443 after 15 ms: Could not connect to server)
```

`curl` reaches the same host, and so do `cargo +stable`, `cargo +nightly-2026-09-22`, `cargo +nightly-2026-10-12`
and the floating `cargo +nightly` (which is the same binary as `nightly-2026-10-18`).
The failure reproduced in the agent shell here, and per the handover also in a systemd user unit outside the sandbox
(not re-run for this note).
It does not depend on IPv4 versus IPv6:
every address that libcurl tries (four IPv6 and four IPv4 addresses of `index.crates.io`) fails the same way.

## Root cause

An OpenSnitch application firewall on the host holds a permanent `reject` rule for the exact executable path of
the failing `cargo` binaries.
OpenSnitch applies rules per `process.path`, so the same bytes at an allowed path connect and at a rejected path do not.
Nothing in cargo, libcurl, DNS, TLS, or the network path is involved.

### How libcurl sees it

`CARGO_HTTP_DEBUG=true CARGO_LOG=trace` shows libcurl trying every address and each `connect` ending in
`Software caused connection abort` within about 2 ms:

```text
http-debug: *   Trying [2a04:4e42:200::649]:443...
http-debug: * connect to 2a04:4e42:200::649 port 443 from fd00:4956:504e:ffff::ac11:aaaa port 37276 failed: Software caused connection abort
http-debug: *   Trying 151.101.2.137:443...
http-debug: * connect to 151.101.2.137 port 443 from 172.17.170.170 port 37896 failed: Software caused connection abort
http-debug: * Failed to connect to index.crates.io:443 after 17 ms: Could not connect to server
```

`strace --follow-forks` shows the socket calls of the failing cargo and of the working stable cargo are the same
(`socket`, `setsockopt(TCP_NODELAY)`, non-blocking `connect` returning `EINPROGRESS`);
only the asynchronous result differs:

```text
failing: connect(6, {AF_INET6, 2a04:4e42:600::649, 443}) = -1 EINPROGRESS
         getsockopt(6, SOL_SOCKET, SO_ERROR, [ECONNABORTED], [4]) = 0
working: connect(6, {AF_INET6, 2a04:4e42::649, 443}) = -1 EINPROGRESS
         getsockopt(6, SOL_SOCKET, SO_ERROR, [0], [4]) = 0
```

A socket that is created normally and then aborted after `connect` was issued
is a socket destroyed from outside the process.

### Which component destroys the socket

Linux sets `ECONNABORTED` when a socket is destroyed through the `SOCK_DESTROY` command of the `inet_diag` netlink API
(`net/ipv4/tcp_diag.c:653` in `torvalds/linux` master, read on 2026-10-18; the host runs kernel 7.2.9, not compared line by line):

```c
err = sock_diag_destroy(sk, ECONNABORTED);
```

OpenSnitch 1.8.0 (`rpm`: `opensnitch-1.8.0-1`, daemon `opensnitchd 1.8.0`) issues that command for every connection
whose effective rule has the `reject` action (`daemon/main.go:549-550` at tag `v1.8.0` of `evilsocket/opensnitch`):

```go
		if r.Action == rule.Reject {
			netlink.KillSocket(con.Protocol, con.SrcIP, con.SrcPort, con.DstIP, con.DstPort)
		}
		packet.SetVerdict(netfilter.NF_DROP)
```

`KillSocket` ends in `SocketKill`, which sends `SOCK_DESTROY` (`daemon/netlink/socket_linux.go:18` and `:189`).
A plain `deny` rule only drops packets and would show as a timeout, not an immediate abort.

### Which rules reject cargo

`opensnitch.service` is active, the host runs SELinux in `Enforcing` mode (not involved: it would give `EACCES`),
and the rule directory `/etc/opensnitchd/rules/` lists file names (the contents are `root`-only, mode `0600`).
Rule file names encode the matched executable path.
The rules for the rustup toolchain `cargo` binaries, with their last-modified time (local time, EDT), are:

```text
reject 2026-09-17 20:26  nightly-2026-09-16   (connects anyway, see "Open question")
reject 2026-09-17 20:26  stable               (connects anyway, see "Open question")
reject 2026-10-17 04:00  nightly-2026-10-15   (toolchain no longer installed)
reject 2026-10-18 04:04  nightly-2026-10-17   fails
reject 2026-10-18 11:41  nightly-2026-10-18   fails
reject 2026-10-18 11:43  nightly-2026-10-13   fails
allow  2026-08-28 18:22  nightly (floating)   connects
allow  2026-09-04 .. 2026-10-13 for 1.98.0, 1.98.1, nightly-2026-09-11, -09-12, -09-22, -10-11, -10-12   connect
```

The full names have the shape
`reject-always-simple-var-home-user-rustup-toolchains-nightly-2026-10-18-x86-64-unknown-linux-gnu-bin-cargo.json`.
The daemon log `/var/log/opensnitchd.log` records the creation of the three recent ones:

```text
[2026-10-18 08:04:25] IMP Saved new rule: reject if process.path is '/var/home/user/.rustup/toolchains/nightly-2026-10-17-x86_64-unknown-linux-gnu/bin/cargo'
[2026-10-18 15:41:28] IMP Saved new rule: reject if process.path is '/var/home/user/.rustup/toolchains/nightly-2026-10-18-x86_64-unknown-linux-gnu/bin/cargo'
[2026-10-18 15:43:39] IMP Saved new rule: reject if process.path is '/var/home/user/.rustup/toolchains/nightly-2026-10-13-x86_64-unknown-linux-gnu/bin/cargo'
```

The log is in UTC, hence four hours ahead of the file times above.
The rule matches `/var/home/user/...`, the resolved path,
not the `/home/user/...` spelling (`/home` is a symlink to `var/home`, and `strace` shows cargo launched as `/home/user/.rustup/...`).

### Why a reject rule exists for each new toolchain

Every nightly bump installs `cargo` at a new path, so OpenSnitch has no rule for it.
The daemon then asks the OpenSnitch user interface for a pop-up.
The user interface settings on this host (`~/.config/opensnitch/settings.conf`) are:

```text
default_action=2
default_duration=8
default_timeout=30
disable_popups=false
```

In the installed user interface (`/usr/lib/python3/dist-packages/opensnitch`, package `opensnitch-ui-1.8.0-1`):

```python
# config.py:44-46
    ACTION_DENY_IDX = 0
    ACTION_ALLOW_IDX = 1
    ACTION_REJECT_IDX = 2
# dialogs/prompt/__init__.py:344-346
    def on_timeout_triggered(self):
        self._timeout_triggered = True
        self._send_rule()
# dialogs/prompt/__init__.py:626-627, inside _send_rule
            elif self._default_action == Config.ACTION_REJECT_IDX:
                self._rule.action = Config.ACTION_REJECT
```

`default_action=2` is reject.
`default_duration=8` is the ninth entry of the duration combo in `res/prompt.ui` (once, 30s, 5m, 15m, 30m, 1h, 12h,
until reboot, forever), and `get_duration` returns `Config.DURATION_ALWAYS` for any index above 7
(`dialogs/prompt/_utils.py:125-143`).
So a pop-up that nobody answers within 30 seconds saves a permanent `reject` rule for the executable path.
A cargo started by an agent while the user is away from the screen is exactly that case.
The first cargo run of a new toolchain creates the rule; a user who answers in time gets an `allow` rule instead
(the `allow` files for `nightly-2026-09-22`, `-10-11` and `-10-12` exist).
The saved rule times (04:04, 11:41, 11:43) fit unattended runs, but the rules do not record whether a rule came from a
timeout or from a click, so that part is an inference.

## Verification

Environment: Linux 7.2.9, OpenSnitch 1.8.0 (nftables, `ebpf` process monitor), rustup toolchains in `~/.rustup`.
Harness: a scratch crate with one dependency (`itoa = "=1.0.11"`) and a scratch `CARGO_HOME`,
run as `CARGO_HOME=<fresh dir> CARGO_NET_RETRY=0 cargo <toolchain> fetch` from the crate directory.

Same-bytes control, run in the same shell and the same environment:

```text
sha256 08a9f0b8ce3f...c4dd  ~/.rustup/toolchains/nightly-2026-10-17-.../bin/cargo               (reject rule)
sha256 08a9f0b8ce3f...c4dd  /var/mnt/encrypted/temp/agent/dylint-closure/rustup-home/toolchains/nightly-2026-10-17-.../bin/cargo  (allow rule)
$ <allow-ruled copy> fetch   -> Downloaded itoa v1.0.11
$ <~/.rustup copy> fetch     -> [7] Could not connect to server (Failed to connect to index.crates.io:443 after 15 ms)

sha256 8a57929438a0...07d2  ~/.rustup/toolchains/nightly-x86_64-.../bin/cargo       (floating nightly, allow rule)
sha256 8a57929438a0...07d2  ~/.rustup/toolchains/nightly-2026-10-18-.../bin/cargo   (reject rule)
$ cargo +nightly fetch              -> Downloaded itoa v1.0.11   (rustc commit 3960fa69bb61...)
$ cargo +nightly-2026-10-18 fetch   -> Could not connect to server
```

Both pairs are one binary at two paths, so the executable path is the only variable.
Version control: `nightly-2026-10-11` and `nightly-2026-10-12` are `1.101.0-nightly` and fetch successfully
(they have `allow` rules), while `nightly-2026-10-13`, also `1.101.0-nightly`, fails.
`cargo -vV` of `stable`, `nightly-2026-09-11`, `nightly-2026-09-22`, `nightly-2026-10-13` and `nightly-2026-10-18`
reports the same bundled `libcurl: 8.21.0-DEV (sys:0.4.90+curl-8.21.0 vendored ssl:OpenSSL/3.6.3)`,
so the HTTP stack is identical across working and failing toolchains.

Working catalog (fetch run and succeeded):
`cargo +stable` (1.98.0), `+nightly-2026-09-16`, `-09-22`, `-10-11`, `-10-12`,
`+nightly`, and the allow-ruled copy of the `2026-10-17` cargo.
Failing catalog (`Could not connect to server` in 15 ms to 17 ms): `+nightly-2026-10-13`, `-10-17`, `-10-18`,
and the `~/.rustup` copy of the `2026-10-17` cargo.

## Verified workarounds

### No host change: fill the cache with a cargo that has an allow rule

`cargo +nightly fetch --locked` in the package (floating nightly, same binary as the pinned `nightly-2026-10-18`
today) or `cargo +stable fetch --locked`, then run the pinned nightly with `CARGO_NET_OFFLINE=true`.
This is the workaround the IDE task `lint:closures` already uses.
Note of 2026-10-19:
the IDE task `lint:closures` was removed.
`monochromatic-lint` checks the cargo cache offline for every Rust package and,
on a cold cache,
skips the package with a note that names `cargo fetch --locked`.
It never fetches,
so this workaround is now a step for the person or the host workflow that runs before the linter.
Tradeoffs: the lockfile check and registry cache come from a different cargo version (use `--locked` so it cannot
rewrite `Cargo.lock`); every crate a build needs has to be fetched first, including build-script and
`dylint_driver` dependencies; and the floating `nightly` binary only matches the pinned one until the next
`rustup update`.
Running `cargo` from any toolchain path that has no rule yet (a fresh `rustup update` or a new scratch `RUSTUP_HOME`)
shows a pop-up, and an unanswered one mints another permanent `reject` rule, so such first runs belong to a person
at the screen.

### Host change (applied on 2026-10-18)

The owner asked for it and the coordinating session applied it with `sudo`:

- `/etc/opensnitchd/rules/000-allow-cargo.json` was added (mode 0600, owner root).
  It is a `regexp` rule on `process.path` with `precedence` and action `allow`, matching
  `^/var/(home/user/\.rustup|mnt/encrypted/temp/agent/.+)/toolchains/[^/]+/bin/cargo$`:
  the `cargo` of any rustup toolchain in the home directory or in a scratch rustup home of an agent session.
- Test with the old reject rule still in place: `cargo +nightly-2026-10-18 fetch` in a scratch crate with a fresh
  scratch `CARGO_HOME` downloaded `itoa v1.0.11`, so `precedence` does win over a `reject-*` file.
- The six `reject-always-simple-...-bin-cargo.json` rules for the toolchains `nightly-2026-09-16`, `-10-13`, `-10-15`,
  `-10-17`, `-10-18` and `stable` were moved to `/etc/opensnitchd/rules.removed-2026-10-18/` (a backup, not a deletion).
  The same fetch worked again afterwards and `opensnitch.service` stayed active.
- Left alone: the `reject` rules for `rustup` (`/var/home/user/.cargo/bin/rustup`) and `cargo-semver-checks`.
- Not changed: the pop-up preference below, so a program path outside every rule still gets a permanent reject
  after 30 unanswered seconds.

The original recommendation follows for the record.

Delete or disable the three reject rules of installed toolchains in the OpenSnitch user interface (Rules tab), or remove
the files as `root`:

```text
/etc/opensnitchd/rules/reject-always-simple-var-home-user-rustup-toolchains-nightly-2026-10-13-x86-64-unknown-linux-gnu-bin-cargo.json
/etc/opensnitchd/rules/reject-always-simple-var-home-user-rustup-toolchains-nightly-2026-10-17-x86-64-unknown-linux-gnu-bin-cargo.json
/etc/opensnitchd/rules/reject-always-simple-var-home-user-rustup-toolchains-nightly-2026-10-18-x86-64-unknown-linux-gnu-bin-cargo.json
```

Removal alone is enough to restore access.
It is only hygiene once the allow rule below exists.
The OpenSnitch wiki (`Rules.md`, "Rule precedence") says the
daemon walks rules in name order and selects the first matching `reject` or `deny` rule, or a matching rule with
`precedence` set, and otherwise the last matching `allow`;
so a `000-` rule with `precedence` is selected before the later `reject-*` files.

Then add one rule so that future toolchains stop repeating this.
The first draft of the rule, `000-allow-rustup-toolchain-cargo.json` (rule syntax from the wiki's `Rules.md` and
`Rules-examples.md`); the rule that was installed is the one named above, with the wider path pattern:

```json
{
  "name": "000-allow-rustup-toolchain-cargo",
  "enabled": true,
  "precedence": true,
  "action": "allow",
  "duration": "always",
  "operator": {
    "type": "regexp",
    "operand": "process.path",
    "sensitive": false,
    "data": "^/var/home/user/\\.rustup/toolchains/[^/]+/bin/cargo$"
  }
}
```

Tradeoffs: this allows the toolchain `cargo` of every rustup toolchain to use the network, which is what a package
manager needs; `rustc` does not need it and stays under the pop-up rule.
The `precedence` flag is what lets it win over any older `reject-*` file for the same path, which also means it
cannot be overridden later by a stricter rule without removing it.

### Pop-up preference (user choice)

The root of the recurring rejects is `default_action=2` with `default_duration=8` (forever) and a 30 second timeout.
Alternatives: a timeout action of `allow` with a limited duration, or `reject` with duration `until reboot` or `once`,
so an unanswered pop-up does not become a permanent rule.
This is a policy trade between "unattended unknown programs stay blocked" and "unattended tools keep working",
so it is the user's decision.
The upstream user interface default is `DEFAULT_DURATION_IDX = 6 # until restart` (`config.py:86`), but index 6 in
`get_duration` is `12h` and `until reboot` is 7; the comment and the code disagree by one.
That mismatch is a cosmetic aside and not the cause here.

## What does not work

- A cargo 1.101 regression, a libcurl 8.21 problem, OpenSSL, or HTTP/2: ruled out by the same-bytes control and by
  `nightly-2026-10-11` and `-10-12` (1.101) working.
  The `CARGO_HTTP_*` and proxy environment overrides were not tried one by one, because the socket is destroyed
  before any TLS or HTTP byte exists, and the path swap already isolates the cause.
- DNS, the CA bundle, SELinux, IPv6 happy eyeballs, a proxy: libcurl resolves the host and every address fails at
  the TCP layer with `ECONNABORTED`; an SELinux denial would be `EACCES`.
- The VPN: the source address `172.17.170.170` belongs to the `fr-par-fr1` interface of the host's VPN
  (`ivpn-service` is running), but the working cargo and `curl` leave through the same interface and address.
- Running `cargo` from a path with no rule to test the default: the pop-up could save a new permanent rule, which is a
  host change.
  The default for an unknown path was therefore not exercised.
- The earlier note in `dylint-library-tests-and-shared-build-dir.md` ("not tied to the newest nightly",
  `SO_ERROR = ECONNABORTED`) was right on both points; it only lacked the cause.

## Open question: two reject files that do not block

`reject-always-simple-...-toolchains-stable-...-bin-cargo.json` and the one for `nightly-2026-09-16`, both last changed
2026-09-17 20:26, exist, yet the cargo of both toolchains connects.
Their contents are `root`-only, so whether they are disabled, were replaced in the running daemon, or match a different
path could not be read, and `journalctl --unit=opensnitch.service` exited with status 1 and no output for this user.
This does not affect the conclusion, since the path-swap controls above hold regardless, but it means the rule
files should be inspected as `root` before deleting anything.

## Upstream filing decision

No file in `.out-of-scope/` mentions OpenSnitch, crates.io, networking, or rustup (checked with `rg --ignore-case`).
Constraint 1 (really upstream's fault) fails for cargo, rustup, and the Linux kernel: they behave correctly.
For OpenSnitch the daemon and user interface act as configured (a reject rule destroys the socket; an unanswered
pop-up applies the configured default).
No upstream issue is drafted.
The consumer-side fix is the host rule above.

Sources read on 2026-10-18: the installed `opensnitch-ui` 1.8.0 files named above;
`evilsocket/opensnitch` at tag `v1.8.0` (`daemon/main.go`, `daemon/netlink/socket_linux.go`) and the wiki clone
(`Rules.md`);
`torvalds/linux` `net/ipv4/tcp_diag.c`.
