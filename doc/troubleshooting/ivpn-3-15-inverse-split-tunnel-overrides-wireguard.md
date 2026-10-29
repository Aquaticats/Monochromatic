# IVPN Desktop 3.15.13 inverse split tunneling overrides another WireGuard tunnel and exposes ISP egress

## Symptom

A custom WireGuard interface can be up with a recent handshake while fresh application connections still expose
physical ISP addresses.

The observed interface was `mx-que-mx1`.
 Unmarked route probes selected that interface for both tested public destinations:

```text
198.50.177.220 dev mx-que-mx1 src 172.17.170.170
121.127.43.196 dev mx-que-mx1 src 172.17.170.170
```

Firefox,
 Chromium,
 a fresh neutral systemd scope,
 and IVPN's geo API all observed the physical IPv4 address
`104.247.226.233`.
 A fresh API request changed neither WireGuard transfer counter while IVPN split tunneling was active.

IVPN Desktop itself was disconnected,
 but its inverse split-tunnel setting remained enabled:

```text
Split Tunnel : Enabled
Inverse mode : Enabled
Allow connectivity when VPN is disconnected : Enabled
```

The active policy path was different from the unmarked route probe:

```text
fwmark 0xca6c lookup ivpn-exclude-tbl
```

Table `ivpn-exclude-tbl`,
 numeric table `17`,
 carried physical defaults through `wlan0`.

Disabling IVPN split tunneling exposed an independent `wg-quicker` defect in the version before commit
`2d94b86fd`.
 Peer endpoint `121.127.43.196:2049` was covered by a non-default `AllowedIPs` prefix and routed into
`mx-que-mx1` itself.
 A fresh request then added transmitted WireGuard bytes but received none and timed out.
 `wg-quicker down mx-que-mx1` removed the recursive routes and restored physical connectivity.

## Root cause

This is the interaction of intended IVPN inverse-mode semantics and the old `wg-quicker` route design.
 It is not a Firefox-specific failure.

### IVPN intentionally bypasses VPNs for ordinary applications

IVPN Desktop 3.15.13 describes inverse mode as selected-app-only VPN use.
 `cli/commands/splittun.go:216-219` says every other application uses the default connection:

```go
c.BoolVar(&c.onInverse, cmd_name_on_inverse, false,
    `Enable inverse mode. Only specified applications utilize the VPN connection,
    while all other traffic circumvents the VPN, using the default connection.`)
c.BoolVar(&c.offInverse, cmd_name_off_inverse, false, `Disable inverse mode`)
```

The Linux implementation assigns fixed identities in
`daemon/References/Linux/etc/splittun.sh:26-33` and `:55-60`:

```sh
_cgroup_classid=0x4956504e
_routing_table_name=ivpn-exclude-tbl
_routing_table_weight=17
_packets_fwmark_value=0xca6c
```

Inverse mode negates the cgroup match at `daemon/References/Linux/etc/splittun.sh:188-193`:

```sh
# 'splitted' apps use only VPN connection, all the rest apps use default connection settings (bypassing VPN)
local inverseOption=""
if [ ${_is_inversed} -eq 1 ]; then
    inverseOption=" ! "
fi
```

The output-mangle rule at `daemon/References/Linux/etc/splittun.sh:209-215` consequently marks non-DNS output
from applications outside IVPN's cgroup:

```sh
${bin_iptables} -w ${_iptables_locktime} -I ${POSTROUTING_nat} \
    -m cgroup ${inverseOption} --cgroup ${_cgroup_classid} -o ${def_inf_name} -j MASQUERADE
${bin_iptables} -w ${_iptables_locktime} -I ${OUTPUT_mangle} \
    -m cgroup ${inverseOption} --cgroup ${_cgroup_classid} -j MARK --set-mark ${_packets_fwmark_value}
```

Lines 216 through 218 insert TCP and UDP destination-port `53` returns after the mark command in source order.
Because every rule uses iptables `-I`,
 those returns execute before the mark rule:

```sh
${bin_iptables} -w ${_iptables_locktime} -I ${OUTPUT_mangle} \
    -m cgroup ${inverseOption} --cgroup ${_cgroup_classid} -p tcp --dport 53 -j RETURN
${bin_iptables} -w ${_iptables_locktime} -I ${OUTPUT_mangle} \
    -m cgroup ${inverseOption} --cgroup ${_cgroup_classid} -p udp --dport 53 -j RETURN
```

A destination-port `53` probe can therefore follow the custom tunnel while web traffic uses physical egress.

The script sends the mark to its own table at
`daemon/References/Linux/etc/splittun.sh:393-404`:

```sh
echo "${_routing_table_weight}      ${_routing_table_name}" >> /etc/iproute2/rt_tables
${_bin_ip} rule add fwmark ${_packets_fwmark_value} table ${_routing_table_name}
${_bin_ip} -6 rule add fwmark ${_packets_fwmark_value} table ${_routing_table_name}
```

`updateRoutes` then puts physical defaults in that table at
`daemon/References/Linux/etc/splittun.sh:451-460`:

```sh
${_bin_ip} route replace default via ${_def_gateway} dev ${_def_interface_name} \
    table ${_routing_table_name}
${_bin_ip} -6 route replace default via ${_def_gatewayIPv6} dev ${_def_interface_nameIPv6} \
    table ${_routing_table_name}
```

A plain `ip route get <destination>` has no application packet mark.
 It therefore reported the custom WireGuard route while real application packets followed IVPN table `17`.

The daemon keeps this behavior active independently of current IVPN connection state.
 `daemon/service/service.go:1275-1279` passes both preference state and current connection state into split-tunnel
configuration:

```go
if len(s.splitTunnelling_getDisabledReason()) > 0 {
    enabled = false
}
return splittun.ApplyConfig(enabled, prefs.IsInverseSplitTunneling(),
    prefs.SplitTunnelAllowWhenNoVpn, isVpnConnected, addressesCfg, prefs.SplitTunnelApps)
```

### Old `wg-quicker` routing trapped its own endpoint

Before commit `2d94b86fd`,
 automatic routing put non-`/0` allowed prefixes directly in the main table and enabled
WireGuard's fwmark policy table only for literal `/0` prefixes.
 The generated route set included `121.127.43.196`,
 so the most specific main-table match sent the peer's outer UDP
packets back into `mx-que-mx1`.

The fixed design is recorded in `package/wg/cli-quicker/src/tunnel-route.ts:149-156`:

```ts
 * Automatic routing places every allowed prefix in one policy table carried by
 * the interface fwmark. A `not fwmark` rule per represented family routes inner
 * traffic through that table, while WireGuard's marked outer packets skip it.
 * Keeping every allowed prefix out of the main table prevents endpoint recursion
 * even when a non-default prefix contains the peer's public endpoint.
```

The implementation allocates the mark and table for every nonempty automatic route set at
`package/wg/cli-quicker/src/tunnel-route.ts:201-230`.
 Ordinary inner packets use the policy table.
 WireGuard outer packets carry the interface fwmark,
 skip that table,
 and retain the physical main-table path.

The earlier endpoint-only hypothesis was incomplete.
 Endpoint recursion explained the timeout only after IVPN's override was removed.
 It did not explain why active-IVPN requests completed through the ISP without changing WireGuard counters.
 The observed `0xca6c` rule and IVPN source establish that first path.

## Verification

### Version and source boundary

The installed package and CLI reported:

```text
ivpn-3.15.13-1.x86_64
version:3.15.13 (date:2026-07-06 commit:ae77f76bb8cf1ee0d155ab9e0dd4cecbc32e8fcb) amd64
```

IVPN source tag `v3.15.13`,
 commit `ae77f76bb8cf1ee0d155ab9e0dd4cecbc32e8fcb`,
 was cloned read-only from
`https://github.com/ivpn/desktop-app.git` into private scratch path
`~/temp/agent/ivpn-desktop-3.15.13-20260801`.
 Its origin,
 tag,
 commit,
 and clean status were checked.
 No third-party source file was modified.

### Read-only diagnosis harness

Run these probes while the symptom is present:

```sh
ivpn splittun -status
ip -4 rule show
ip -6 rule show
sudo nft --numeric list ruleset
ip -4 route get 198.50.177.220
ip -4 route get 198.50.177.220 mark 0xca6c
sudo wg show mx-que-mx1 transfer
curl --ipv4 --silent --show-error --max-time 20 https://api.ivpn.net/v4/geo-lookup
sudo wg show mx-que-mx1 transfer
```

The decisive difference is the marked lookup.
 When IVPN inverse split tunneling is active,
 mark `0xca6c` selects table `17` and its physical default.
 The two transfer readings determine whether the request entered WireGuard.

### Disposable regression harness

Commit `2d94b86fd` adds `package/wg/cli-quicker/src/tunnel-route.integration.test.ts` and this package task:

```sh
mise run //package/wg/cli-quicker:test:integration:route
```

The harness creates client and server network namespaces,
 a physical veth path,
 and two WireGuard peers.
 Client endpoint `203.0.113.1` is deliberately covered by non-default `AllowedIPs = 192.0.0.0/2`.
 It verifies:

- exact IVPN mark/table conflict rejects `up` before interface creation;
- marked outer endpoint traffic retains the physical veth path;
- inner traffic uses the allocated WireGuard policy table;
- a bidirectional ping increases received and sent transfer counters;
- `down` removes the interface.

Observed result:

```text
wg-quicker endpoint routing integration passed
```

### Patterns that work cleanly

- IVPN split tunneling disabled,
   fixed `wg-quicker`,
   and a peer endpoint covered by non-default `AllowedIPs`.
- Fixed `wg-quicker` with an exact synthetic IVPN rule absent.
- `wg-quicker down` after the old recursive route state,
   which restored the physical endpoint path.

### Patterns that fail

- IVPN inverse split tunneling enabled while ordinary applications run outside classid `0x4956504e`.
  Their non-DNS packets receive mark `0xca6c` and use physical table `17`.
- Old `wg-quicker` after IVPN split tunneling is disabled when a non-default allowed prefix covers the peer endpoint.
  The outer UDP path recurses into WireGuard and receives no response.
- An unmarked `ip route get` used as proof of real application egress while a mangle rule adds a packet mark later.
  The probe and application evaluate different policy-routing inputs.

## Verified workarounds

### Disable IVPN split tunneling

The supported command is:

```sh
ivpn splittun -off
ivpn splittun -status
```

`cli/commands/splittun.go:299-312` converts `-off` into `isEnabled = false` and sends the updated configuration:

```go
if c.off {
    isEnabled = false
}
if err = _proto.SetSplitTunnelConfig(isEnabled, isInverse, isAnyDns,
    isAllowWhenNoVpn, false); err != nil {
    return err
}
```

The Linux daemon invokes the script's `stop` operation at `daemon/splittun/splittun_linux.go:383-393`:

```go
if !isEnable {
    enabled, err := isEnabled()
    if err == nil && !enabled {
        return nil
    }
    err = shell.Exec(log, stScriptPath, "stop")
    // ...
    log.Info("Split Tunnel disabled")
}
```

The script removes mark rules and flushes table `17` at
`daemon/References/Linux/etc/splittun.sh:495-505`.
 On the affected host,
 the command returned `Split Tunnel : Disabled`;
 both family rule lists reverted to standard
local/main/default rules and the IVPN nft matches disappeared.

Tradeoff:
 this disables IVPN's complete split-tunnel feature,
 including its saved selected-application behavior.
 It does not erase the saved app list because `-off` does not use the separate `-clean` reset path.

### Use fixed `wg-quicker`

Commit `2d94b86fd` routes every automatic `AllowedIPs` prefix through the marked policy table.
 It also checks both family rule sets before creating the interface.
 `package/wg/cli-quicker/src/policy-routing-conflict.ts:124-129` rejects IVPN's exact rule with:

```text
IVPN Desktop split tunneling is active (fwmark 0xca6c, table 17) and would bypass this WireGuard tunnel.
Disable IVPN split tunneling with `ivpn splittun -off` before running `wg-quicker up`.
```

Tradeoff:
 automatic non-default prefixes now use a WireGuard policy table instead of direct main-table routes and receive
source-mark validation plus nft ingress protection.
 Separate automatic tunnels have separate tables;
policy-rule priority decides overlapping prefixes before longest-prefix matching can compare routes across those
tables.
 Explicit numeric `Table`,
 `Table = off`,
 physical connected routes,
 and marked application exemptions keep their
separate semantics.

## What does not work

- Testing only Firefox does not isolate the issue.
  Chromium and a fresh neutral scope followed the same physical marked path.
- Creating a new socket does not avoid the rule.
  IVPN marks non-DNS packets by cgroup match at output time.
- A destination-port `53` leak probe does not model web traffic in this mode.
  IVPN inserts DNS returns before its mark rule.
- Checking proxy environment or GNOME proxy mode does not explain policy routing.
  No relevant proxy was active.
- Checking only `ip route get` without `mark 0xca6c` gives the route for a different packet state.
- Leaving IVPN inverse mode enabled and adding `wg-quicker` exemptions does not compose safely.
  IVPN's output-mangle mark applies independently and can overwrite application socket marks.
- Re-enabling IVPN split tunneling after `wg-quicker up` is not caught by the one-shot preflight.
  The daemon can recreate mark and route rules on a later configuration or network-state change,
  restoring a silent bypass.
- Editing IVPN's script constants can evade exact detection.
  The preflight recognizes installed values `0xca6c`,
   table `17`,
   and `ivpn-exclude-tbl`,
   including numeric and
  zero-padded iproute2 JSON mark forms.
- Disabling IVPN split tunneling alone did not restore VPN connectivity with old `wg-quicker`.
  It exposed endpoint recursion and caused the timeout described in this document.
- Adding only an endpoint host route would pin a current gateway and require separate ownership,
   change tracking,
  teardown,
   multi-peer,
   and roaming logic.
  Moving all automatic allowed routes to the existing marked policy design avoids persistent physical host routes.

## Upstream filing decision

No `.out-of-scope/` entry covers IVPN or this behavior.
 Searches covered open and closed issues and pull requests for `inverse split tunnel`,
 `split tunnel wireguard`,
`fwmark 0xca6c`,
 and endpoint-routing terms.
 Issue [#272][issue-272] is the related feature request and confirms inverse mode is an opt-in VPN design.
 Its full body and comments were read.
 It does not report a defect in the observed behavior.

1. **Is it really upstream's fault?**
    No.
   IVPN's CLI says ordinary traffic circumvents VPNs in inverse mode,
    and the implementation does exactly that.
2. **Can upstream fix it?**
    An interoperability warning could be added,
    but no upstream correctness defect was found.
3. **Are they supporting this use case?**
    Yes.
   Issue [#272][issue-272] requested selected-app-only VPN use,
    and maintainers delivered it in v3.13.1.
4. **Would the repo welcome our contribution?**
    Generally yes.
   `.github/CONTRIBUTING.md`,
    the bug template,
    and the pull-request template invite focused reports and patches.
   No AI-assistance ban was found.
5. **Will they likely fix it?**
    Not as a bug.
   The behavior is the feature's explicit contract,
    so changing it would defeat inverse mode.
6. **Have we prototyped a minimal fix compatible with their architecture?**
    Not applicable because constraint one
   fails.
   The required fix belongs in the consumer:
    disable the conflicting feature,
    detect its exact rule,
    and prevent
   WireGuard endpoint recursion.

Nothing should be filed upstream and there is no additive comment for issue [#272][issue-272].
 Posting the observation there would restate the requested behavior rather than advance that thread.

[issue-272]: https://github.com/ivpn/desktop-app/issues/272
