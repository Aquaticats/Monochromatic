# WG Tunnel 5.2.1 on Android rejects a loopback `AllowedIPs` route with `Bad address`

Status note:
 the log archive does not embed a config identity or config hash.
 At first inspection after the incident,
 `/var/home/user/mx-que-mx1.conf` was 90,952 bytes,
 contained 4,074 `AllowedIPs` entries,
 and included `::1/128` at entry 1,321.
 The file later changed externally to 91,118 bytes and 4,085 entries,
 with no modeled loopback or non-canonical route rejection found by the static harness.
 The archive-to-config association rests on the timeline and exact stack match,
 so rerun the harness before editing the current file.

## Symptom

WG Tunnel 5.2.1 standalone on GrapheneOS refuses to establish the tunnel.
 The log archive
`wg_tunnel_logs_2026-07-28_11-03-33_5.2.1_standalone.zip` has SHA-256
`bb078848c90d0a9f1c0e43ea55015fd0ccf5fad42aa2d16d9e23fb7bd9978bf8`.
 Its member `logcat_2026-07-28_11-03-13_v5.2.1_standalone.txt` has SHA-256
`46141924311408c73c3c4e08a8354770f2fe218d644e308f14a80ad96408e508`.
 Lines 14 through 18 and lines 41 through 45 contain this stack:

```text
java.lang.IllegalArgumentException: Bad address
    at android.net.VpnService.check(VpnService.java:461)
    at android.net.VpnService$Builder.addRoute(VpnService.java:603)
    at android.net.VpnService$Builder.addRoute(VpnService.java:638)
    at com.zaneschepke.tunnel.service.VpnService.b(...)
```

The failing surface syntax is an IPv6 loopback route in peer `AllowedIPs`:

```text
::1/128
```

A different `Bad address` variant exists for non-canonical CIDRs such as `192.168.10.0/20`.
 That variant travels through `VpnService.checkNonPrefixBytes`,
 not `VpnService.check`,
 so the exact stack line distinguishes it from the loopback failure in this incident.

## Root cause

WG Tunnel 5.2.1 converts every peer `AllowedIPs` entry into an Android VPN route.
 In tag `5.2.1`,
 commit `f585050bf23e444007c276fb8d6c93bb0e75fa03`,
 `tunnel/src/main/java/com/zaneschepke/tunnel/service/VpnService.kt:258-266` does not filter
loopback routes before calling Android:

```kotlin
// tunnel/src/main/java/com/zaneschepke/tunnel/service/VpnService.kt
// Parse peer routes
config.peers.forEach { peer ->
    peer.allowedIPs
        ?.split(",")
        ?.map { it.trim() }
        ?.filter { it.isNotEmpty() }
        ?.forEach { entry ->
            val (address, prefix) = entry.parseInetNetwork()
            addRoute(address, prefix)
```

The parser accepts the numeric address and prefix.
 `tunnel/src/main/java/com/zaneschepke/tunnel/util/Extensions.kt:28-42` parses the address with
Android's numeric parser and checks only the mask range:

```kotlin
// tunnel/src/main/java/com/zaneschepke/tunnel/util/Extensions.kt
val address =
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
        android.net.InetAddresses.parseNumericAddress(rawAddress)
    } else {
        InetAddress.getByName(rawAddress)
    }

val maxMask = if (address is Inet4Address) 32 else 128
val mask = rawMask?.toIntOrNull() ?: maxMask

if (mask !in 0..maxMask) {
    throw IllegalArgumentException("Invalid network mask: $rawMask (must be 0-$maxMask)")
}

return address to mask
```

GrapheneOS then rejects loopback routes.
 Branch `16`,
 commit `d697c573a824058f1067fc4b317a560d71ce937c`,
 and branch `17`,
 commit `b9fb6acfe18ae7149e3d3f05a4d8ba23f5dad6a7`,
 have the same check at `core/java/android/net/VpnService.java:459-461`:

```java
// core/java/android/net/VpnService.java
private static void check(InetAddress address, int prefixLength) {
    if (address.isLoopbackAddress()) {
        throw new IllegalArgumentException("Bad address");
```

The public string overload parses the address,
 then `addRoute(InetAddress, int)` calls `checkNonPrefixBytes` and the private route method.
 `core/java/android/net/VpnService.java:635-638` shows that path:

```java
// core/java/android/net/VpnService.java
public Builder addRoute(@NonNull InetAddress address, int prefixLength) {
    checkNonPrefixBytes(address, prefixLength);

    return addRoute(new IpPrefix(address, prefixLength), RouteInfo.RTN_UNICAST);
}
```

The private method invokes the loopback check at `core/java/android/net/VpnService.java:602-603`:

```java
// core/java/android/net/VpnService.java
private Builder addRoute(@NonNull IpPrefix prefix, int type) {
    check(prefix.getAddress(), prefix.getPrefixLength());
```

The earlier desktop validation was therefore correct but incomplete for Android.
 Linux `wg setconf` accepted `::1/128` as a WireGuard allowed IP,
 while Android's `VpnService` cannot install a loopback route.
 The config was WireGuard-valid and Android-VPN-route-invalid at the same time.

## Verification

Versions and source identities:

- WG Tunnel log version is `5.2.1 (standalone)`.
- WG Tunnel release version is `5.2.1`.
  This was the latest stable release on 2026-07-28.
- WG Tunnel source tag is `5.2.1`.
  The commit is `f585050bf23e444007c276fb8d6c93bb0e75fa03`.
- GrapheneOS source branch is `platform_frameworks_base` branch `16`.
  The commit is `d697c573a824058f1067fc4b317a560d71ce937c`.
- GrapheneOS source branch is `platform_frameworks_base` branch `17`.
  The commit is `b9fb6acfe18ae7149e3d3f05a4d8ba23f5dad6a7`.

### Source investigation boundary

WG Tunnel was cloned read-only from `https://github.com/wgtunnel/android.git` into the private scratch path
`~/temp/agent/wgtunnel-android-2026-07-28`.
 Tag `5.2.1` was fetched and inspected with `git show`.
 The clone ended with a clean `git status`,
 no upstream file was modified,
 and no patch was retained.

GrapheneOS `platform_frameworks_base` was cloned read-only from
`https://github.com/GrapheneOS/platform_frameworks_base.git` into the private scratch path
`~/temp/agent/grapheneos-frameworks-base-2026-07-28`.
 Branch `16` was checked out sparsely for `core/java/android/net/VpnService.java`,
 and branch `17` was fetched for the same file.
 The clone ended with a clean `git status`,
 no upstream file was modified,
 and no patch was retained.

### Static route harness

This harness streams the config,
 checks only `AllowedIPs`,
 and exits nonzero on a modeled loopback or non-canonical route rejection.
 It does not execute Android,
 WG Tunnel,
 Binder serialization,
 or `VpnService`.
 It does not emit or persist private-key material,
 although the private-key line passes through process memory while the file streams:

```bash
python3 - <<'PY'
import ipaddress
import sys

path = "/var/home/user/mx-que-mx1.conf"
entries = []

with open(path, encoding="utf-8") as config:
    for line in config:
        key, separator, value = line.partition("=")
        if separator and key.strip().lower() == "allowedips":
            entries.extend(item.strip() for item in value.split(",") if item.strip())

invalid = []
for index, entry in enumerate(entries, 1):
    interface = ipaddress.ip_interface(entry)
    reasons = []
    if interface.ip.is_loopback:
        reasons.append("Android VpnService rejects loopback routes")
    if interface.ip != interface.network.network_address:
        reasons.append("Android VpnService requires canonical network address")
    if reasons:
        invalid.append((index, entry, "; ".join(reasons)))

print(f"checked={len(entries)} android_route_rejections={len(invalid)}")
for index, entry, reason in invalid:
    print(f"entry {index}: {entry}: {reason}")
sys.exit(1 if invalid else 0)
PY
```

Against the config as first inspected after the incident,
 the harness failed with exactly one route:

```text
checked=4074 android_route_rejections=1
entry 1321: ::1/128: Android VpnService rejects loopback routes
```

A temporary candidate with only `::1/128` removed produced these static and Linux-parser results:

```text
android route entries checked=4073 rejections=0
wireguard parser after removing ::1/128: valid
host network-state unchanged: yes
```

After the file changed externally,
 the same modeled checks produced:

```text
current file: android route entries checked=4085 rejections=0
```

A zero result means no modeled loopback or non-canonical route rejection.
 It does not prove successful Android activation,
 Binder serialization,
 DNS setup,
 handshake,
 or end-to-end traffic.

The pattern catalog was checked with the same modeled Android constraints:

```text
0.0.0.0/0: accept
192.168.176.0/20: accept
2000::/3: accept
::/0: accept
127.0.0.0/8: reject: loopback
::1/128: reject: loopback
192.168.10.0/20: reject: noncanonical
```

Patterns that work cleanly:

- Canonical IPv4 CIDRs,
  including `0.0.0.0/0` and `192.168.176.0/20`.
- Canonical IPv6 CIDRs,
  including `2000::/3` and `::/0`.

Patterns that fail:

- IPv4 loopback routes such as `127.0.0.0/8`.
- IPv6 loopback routes such as `::1/128`.
- Non-canonical CIDRs such as `192.168.10.0/20`,
  which Android reports through the separate `checkNonPrefixBytes` path.

## Verified workarounds

### Remove the loopback route

Delete the `::1/128` token from the peer's `AllowedIPs` list.
 Do not replace it with `::/0` unless full IPv6 tunneling is intended.

This local program writes a separate Android candidate and preserves the original.
 It assumes every `AllowedIPs` assignment is a simple single-line value without an inline comment.
 The output contains the original `PrivateKey`,
 so it is created with mode `0600` from the outset and existing output is never truncated:

```python
# /tmp/remove-android-loopback-route.py
import os
from pathlib import Path

source = Path("/var/home/user/mx-que-mx1.conf")
target = source.with_name("mx-que-mx1-android.conf")

if target.exists():
    raise FileExistsError(f"refusing to overwrite {target}")

flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
if hasattr(os, "O_NOFOLLOW"):
    flags |= os.O_NOFOLLOW

with source.open(encoding="utf-8") as input_file:
    fd = os.open(target, flags, 0o600)
    with os.fdopen(fd, "w", encoding="utf-8") as output_file:
        for line in input_file:
            key, separator, value = line.partition("=")
            if separator and key.strip().lower() == "allowedips":
                entries = [entry.strip() for entry in value.split(",") if entry.strip()]
                entries = [entry for entry in entries if entry != "::1/128"]
                line = key + "=" + ", ".join(entries) + "\n"
            output_file.write(line)

print(target)
```

The documented program was extracted from this file and run against a disposable fixture.
 It removed `::1/128`,
 created the candidate as mode `0600`,
 and refused to overwrite that candidate on a second run.

The source file was mode `0644` when measured,
 which exposes its `PrivateKey` to other local users.
 Both the source and generated candidate should be mode `0600`.

Static verification result:
 removing only that token reduced the first-inspected list from 4,074 entries to 4,073,
 eliminated all modeled loopback and non-canonical route rejections,
 and still passed Linux WireGuard parsing in a temporary network namespace.
 The host's interfaces,
 routes,
 routing rules,
 and `/etc/resolv.conf` were unchanged.
 This was not an end-to-end GrapheneOS activation test,
 so Android success remains to be verified through WG Tunnel.

Tradeoff:
 traffic to `::1` stays on the Android device.
 That is normally the only useful behavior because loopback names the local host,
 not the WireGuard peer.
 If a remote service was intended,
 it needs a non-loopback address on the peer.

Tradeoff for large route lists:
 removing the token does not reduce a large list enough to avoid Android Binder limits.
 The first-inspected candidate still had 4,073 routes,
 and the later file had 4,085 routes.
 WG Tunnel issue [#827][issue-827] documents a separate Android IPC limit for very large
`AllowedIPs` lists.

### Rerun the app with fresh logs

If the current file no longer contains `::1/128`,
 import or save it again in WG Tunnel and capture a new log archive.
 A new `Bad address` stack means another route variant remains.
 A Binder or IPC error may indicate the route-count limit documented in issue [#827][issue-827].

Tradeoff:
 this verifies the real Android boundary,
 but it temporarily changes VPN state on the phone.
 Run it only when activating that tunnel is acceptable.

## What does not work

- Validating only with Linux `wg setconf` does not prove Android compatibility.
  Linux accepted the first-inspected `::1/128`,
  while GrapheneOS rejected it in `VpnService.Builder.addRoute`.
- Replacing `::1/128` with `::/0` changes the intended routing from one local address to all IPv6 destinations.
- Canonicalizing CIDRs does not fix this incident.
  The first-inspected list had zero non-canonical CIDRs;
  the failing entry was loopback.
- Collapsing the later 4,085-entry list does not reduce its size.
  Python's `ipaddress.collapse_addresses` returned 4,085 networks,
  a reduction of zero.
- Searching for a key or endpoint problem follows the wrong stack frame.
  The failing frame is `VpnService.Builder.addRoute`,
  so the failing input is an `AllowedIPs` route,
  not `PrivateKey`,
  `PublicKey`,
  or `Endpoint`.
- Moving the file to `/etc/wireguard` does not apply to WG Tunnel on Android.
  That path is for desktop `wg-quick`;
  Android apps import the config through Android storage and the `VpnService` API.

## Upstream filing artifact

### Out-of-scope check

`.out-of-scope/` was checked for WireGuard,
 WG Tunnel,
 Android,
 and VPN exemptions.
 No matching exemption exists.

### Duplicate search

The upstream tracker was searched across issues and pull requests for `addRoute "Bad address"`,
`loopback AllowedIPs`,
 `loopback`,
 and `1/128`.
 A direct quoted search for `::1/128` was rejected by GitHub search syntax,
 so broader terms were used.

Relevant existing reports:

- [wgtunnel/android#229][issue-229]:
  closed report for non-canonical CIDR `192.168.10.0/20`,
  with `Bad address` from `checkNonPrefixBytes`.
  It is the same symptom class but a different Android rejection branch.
- [wgtunnel/android#165][issue-165]:
  closed report for unclear `AllowedIPs` failures and improved error reporting.
- [wgtunnel/android#827][issue-827]:
  closed report for very large `AllowedIPs` lists reaching Android IPC limits.
  It is a separate possible blocker after the loopback route is removed.

No existing report specifically documented `::1/128` reaching `VpnService.check` at line 461 in WG Tunnel 5.2.1.

### Upstream filing decision

1.  Is it really upstream's fault?
    No for the platform rejection.
    GrapheneOS intentionally rejects loopback VPN routes.
    WG Tunnel could improve validation or skip a known-unsupported route with a warning,
    but it cannot make Android route `::1`.
2.  Can upstream fix it?
    Partially.
    Upstream can identify the offending entry,
    reject the config earlier,
    or filter loopback routes.
    Upstream cannot preserve the literal `::1/128` route through Android `VpnService`.
3.  Are they supporting this use case?
    Not explicitly.
    The README supports WireGuard and split tunneling,
    but no docs,
    examples,
    or tests found during this investigation promise loopback `AllowedIPs` support.
4.  Would the repo welcome our contribution?
    Yes in general.
    The README says feedback,
    issues,
    code,
    and translations are welcome.
    The repository has a bug issue template.
    No `CONTRIBUTING.md` or AI-report restriction was found in the checked tree.
5.  Will they likely fix it?
    No direct signal for this exact loopback variant.
    Issue [#229][issue-229] was closed after error handling improved,
    and issue [#827][issue-827] was closed because large-list support would require major changes.
6.  Have we prototyped a minimal fix compatible with their architecture?
    No upstream patch was prototyped.
    The auto-prototype gate did not fire because constraints 1 and 3 fail.
    The config-side fix is to remove the Android-unsupported loopback route.
    End-to-end Android activation remains user-side verification.

Decision:
 do not file as-is.
 The platform restriction and config-side workaround are clear,
 and the existing tracker already covers both generic `AllowedIPs` error reporting and large-list IPC limits.

Draft,
 not fileable as-is:

~~~md
Title: [BUG] - Android rejects ::1/128 in AllowedIPs with Bad address
Labels: bug

WG Tunnel 5.2.1 standalone on GrapheneOS rejects a tunnel whose peer `AllowedIPs` contains
`::1/128`.

WG Tunnel parses each `AllowedIPs` entry and calls `VpnService.Builder.addRoute` in
`tunnel/src/main/java/com/zaneschepke/tunnel/service/VpnService.kt:258-266`.
GrapheneOS `core/java/android/net/VpnService.java:459-461` rejects loopback addresses with
`IllegalArgumentException("Bad address")`.

Reproduction:

1. Import a syntactically valid WireGuard config containing `AllowedIPs = ::1/128`.
2. Enable the tunnel.
3. Observe `Bad address` from `VpnService.check(VpnService.java:461)`.

Expected behavior:

WG Tunnel should either report `::1/128` as an Android-unsupported route before activation,
or skip loopback routes with a visible warning.

Suggested fix:

Add route validation near the `addRoute(address, prefix)` call in
`tunnel/src/main/java/com/zaneschepke/tunnel/service/VpnService.kt`.
At minimum,
include the offending route in the user-facing error.
If silently filtering is acceptable,
skip `address.isLoopbackAddress` routes and warn that Android keeps loopback local.
~~~

[issue-165]: https://github.com/wgtunnel/android/issues/165
[issue-229]: https://github.com/wgtunnel/android/issues/229
[issue-827]: https://github.com/wgtunnel/android/issues/827
