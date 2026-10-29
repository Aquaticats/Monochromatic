# OpenSnitch 1.8.0 strict deny blocks WireGuard transport unless wg-quicker installs endpoint rule

## Symptom

Current `wg-quicker` automatically adds visible OpenSnitch 1.8.0 system rules for WireGuard peer endpoint UDP
ports.
The verified IPv4 and nftables setup now works with OpenSnitch's default `allow` action and with strict
`DefaultAction = deny`.
The command warns that each managed rule accepts any process's outbound UDP to that destination port before
application filtering,
and removes the rule on `down` or failed-`up` rollback.
A private lifecycle manifest preserves the original file and ports across interrupted processes,
path changes,
and missing-link retries until live removal is proved.
The default `/run` manifest is intentionally process-lifecycle state,
not host-reboot recovery.

The live boundary test covered IPv4,
the nftables backend,
and the `proc` monitor.
The integration rejects OpenSnitch's iptables backend instead of claiming protection from an ignored nftables
rule.
IPv6,
eBPF attribution,
and application exemptions received source review only.

Without the managed allowance or an equivalent user rule,
`wg-quicker up` still exits successfully because creating a WireGuard interface does not wait for a handshake.
The visible failure occurs when traffic tries to cross the tunnel:

```console
PING 10.44.0.2 (10.44.0.2) 56(84) bytes of data.

--- 10.44.0.2 ping statistics ---
2 packets transmitted, 0 received, 100% packet loss, time 1043ms
```

The WireGuard peer then reports no handshake and only attempted transmission:

```console
latest-handshakes: 0
transfer: 0 B received, 148 B sent
```

OpenSnitch records the outer packet as UDP with no userspace owner and with `wg-quicker`'s WireGuard mark intact:

```text
new connection udp => 59716:192.0.2.1 -> 192.0.2.2 ():2049 uid: 4294967295, mark: ca6c
[-1] FindProcess() error: Unable to get process information
```

There is no `wg-quicker` diagnostic because OpenSnitch applies its policy after interface setup.
This repository's host did not have OpenSnitch installed when investigated.
The compatibility result therefore comes from the released OpenSnitch daemon in disposable network namespaces,
not from changing the host firewall.

## Root cause

### The tools own separate nftables objects

`wg-quicker` creates a per-interface table in
`package/wg/cli-quicker/src/tunnel-firewall.ts:35-50`:

```ts
const iface = config.interfaceName;
/**
 * Name of the dedicated nft table for this interface.
 */
const nftable = `wg-quicker-${iface}`;
/**
 * nft statements built up and applied atomically via `nft -f`.
 */
const statements: string[] = [`add table inet ${nftable}`,];
```

OpenSnitch 1.8.0 names its table `opensnitch` in
`evilsocket/opensnitch@v1.8.0:daemon/firewall/nftables/exprs/enums.go:5-8`:

```go
const (
	TABLE_OPENSNITCH     = "opensnitch"
	CHAIN_FILTER_INPUT   = "filter_input"
	CHAIN_MANGLE_OUTPUT  = "mangle_output"
```

The released daemon created both `inet opensnitch` and `inet wg-quicker-wgqos` in the same disposable namespace.
Bringing the tunnel down removed only `inet wg-quicker-wgqos` and retained OpenSnitch's table.
This rules out table-name collision and broad table cleanup as the compatibility problem.

### OpenSnitch sees WireGuard's outer UDP packet

OpenSnitch installs an output chain at nftables mangle priority in
`evilsocket/opensnitch@v1.8.0:daemon/firewall/nftables/chains.go:131-138`:

```go
n.AddChain(exprs.CHAIN_FILTER_INPUT, exprs.TABLE_OPENSNITCH, exprs.NFT_FAMILY_INET,
	nftables.ChainPriorityFilter, nftables.ChainTypeFilter, nftables.ChainHookInput, filterPolicy)
if !n.Commit() {
	return fmt.Errorf("Error adding DNS interception chain filter_input-opensnitch-inet")
}
n.AddChain(exprs.CHAIN_MANGLE_OUTPUT, exprs.TABLE_OPENSNITCH, exprs.NFT_FAMILY_INET,
	nftables.ChainPriorityMangle, nftables.ChainTypeRoute, nftables.ChainHookOutput, manglePolicy)
```

Its non-TCP interception rule queues new or related packets in
`evilsocket/opensnitch@v1.8.0:daemon/firewall/nftables/rules.go:93-119`:

```go
n.Conn.AddRule(&nftables.Rule{
	Position: 0,
	Table:    table,
	Chain:    chain,
	Exprs: []expr.Any{
		&expr.Meta{Key: expr.MetaKeyL4PROTO, Register: 1},
		&expr.Cmp{
			Op:       expr.CmpOpNeq,
			Register: 1,
			Data:     []byte{unix.IPPROTO_TCP},
		},
		&expr.Ct{Register: 1, SourceRegister: false, Key: expr.CtKeySTATE},
		// ... NEW or RELATED state comparison ...
		&expr.Queue{
			Num:  n.QueueNum,
			Flag: n.getBypassFlag(),
		},
	},
	UserData: []byte(InterceptionRuleKey),
})
```

A kernel WireGuard interface emits its encrypted peer traffic as a second,
outer UDP connection.
The disposable run proved this packet reached OpenSnitch with UID `4294967295`,
no process,
and mark `0xca6c`.

### Default action determines the no-process result

When process lookup returns no connection,
OpenSnitch applies the configured default action in
`evilsocket/opensnitch@v1.8.0:daemon/main.go:408-440`:

```go
con := conman.Parse(packet, uiClient.InterceptUnknown())
if con == nil {
	applyDefaultAction(&packet, nil)
	return
}

func applyDefaultAction(packet *netfilter.Packet, con *conman.Connection) {
	log.Trace("Applying DefaultAction (%s) on %s", uiClient.DefaultAction(), con)
	if uiClient.DefaultAction() == rule.Allow {
		packet.SetVerdictAndMark(netfilter.NF_ACCEPT, packet.Mark)
		return
	}
	packet.SetVerdict(netfilter.NF_DROP)
}
```

Default allow therefore passes the packet.
Default deny drops it when OpenSnitch cannot attribute it or find an earlier system rule.
This is OpenSnitch policy behavior,
not a routing-rule collision.

### Accepted packet marks remain compatible with wg-quicker routing

`wg-quicker` depends on WireGuard's outer packet retaining the interface mark.
Its policy rule in `package/wg/cli-quicker/src/tunnel-route.ts:121-143` is:

```ts
args: [
  proto,
  'rule',
  'add',
  'not',
  'fwmark',
  String(table,),
  'table',
  String(table,),
],
```

Ordinary inner traffic enters the WireGuard table,
while an outer packet marked with that table number skips it and follows physical routing.

OpenSnitch reads the incoming netfilter mark in
`evilsocket/opensnitch@v1.8.0:daemon/netfilter/queue.go:207-214`:

```go
p := Packet{
	verdictChannel:  make(chan VerdictContainer),
	Mark:            uint32(mark),
	UID:             uid,
	NetworkProtocol: xdata[0] >> 4,
	IfaceInIdx:      int(devIn),
	IfaceOutIdx:     int(devOut),
}
```

Both default and rule-based allow paths return that same mark.
The rule-based path is recorded in
`evilsocket/opensnitch@v1.8.0:daemon/main.go:541-547`:

```go
} else if r.Action == rule.Allow {
	packet.SetVerdictAndMark(netfilter.NF_ACCEPT, packet.Mark)
	ruleName := log.Green(r.Name)
```

The successful handshake after OpenSnitch logged `mark: ca6c` is end-to-end evidence that the mark still prevented
endpoint recursion.

### Application exemptions remain subject to OpenSnitch

`wg-quicker-exempt` sets `SO_MARK` from cgroup socket-address hooks in
`package/wg/cli-quicker-exempt/src/bpf.rs:90-123`:

```rust
const SO_MARK: i32 = 36;
// ...
BpfInsn::new(BPF_MOV32_IMM, 3, 0, 0, SO_MARK),
BpfInsn::new(BPF_CALL, 0, 0, 0, FN_SETSOCKOPT),
```

OpenSnitch 1.8.0 uses kprobes and tracepoints for process discovery,
not the cgroup connect and UDP send-message attachment points owned by `wg-quicker-exempt`.
Its allow paths preserve the existing mark as shown in the previous source trace.
The source-level contracts therefore compose:
the exemption chooses physical routing,
then OpenSnitch can still allow or deny that physical connection.

The disposable compatibility run did not attach `wg-quicker-exempt` to real desktop cgroups because that would have
crossed the fixture boundary.
The cgroup-exemption conclusion is consequently source-backed rather than a combined live-cgroup test.

### OpenSnitch has WireGuard-specific process discovery and a fallback rule

OpenSnitch 1.8.0's eBPF program includes IPv4 and IPv6 tunnel probes in
`evilsocket/opensnitch@v1.8.0:ebpf_prog/opensnitch.c:345-346` and
`evilsocket/opensnitch@v1.8.0:ebpf_prog/opensnitch.c:546-547`:

```c
SEC("kprobe/udp_tunnel6_xmit_skb")
int kprobe__udp_tunnel6_xmit_skb(struct pt_regs *ctx)
```

```c
SEC("kprobe/iptunnel_xmit")
int kprobe__iptunnel_xmit(struct pt_regs *ctx)
```

Kernel and eBPF prerequisites can still make attribution unavailable.
OpenSnitch's [FAQ][opensnitch-faq] and maintainer diagnoses in issues
[#454][opensnitch-454] and [#1250][opensnitch-1250] recommend checking the eBPF module and enabling
`Debug invalid connections` when WireGuard fails.

The packaged firewall configuration also contains a disabled fallback at
`evilsocket/opensnitch@v1.8.0:daemon/data/system-fw.json:217-237`:

```json
{
  "Enabled": false,
  "Description": "Exclude WireGuard VPN from being intercepted",
  "Expressions": [
    {
      "Statement": {
        "Name": "udp",
        "Values": [
          {
            "Key": "dport",
            "Value": "51820"
          }
        ]
      }
    }
  ],
  "Target": "accept"
}
```

That fallback matches a peer endpoint using UDP destination port `51820`.
It does not match a WireGuard peer on another port.
The active repository host tunnel used endpoint port `2049` when inspected,
so the stock value would need adjustment for that tunnel.

### wg-quicker now owns endpoint-rule lifecycle

Current `wg-quicker` extracts distinct peer endpoint ports and installs OpenSnitch rules after WireGuard accepts the
peer config but before bringing the link up.
`package/wg/cli-quicker/src/tunnel.ts:294-309` contains the ordering:

```ts
try {
  await addLink({ interfaceName: iface, },);
  await executeHooks({
    hooks: config.preUp,
    interfaceName: iface,
  },);
  await applyPeerConfig({ config, },);
  await installOpenSnitchEndpointAllowance({
    interfaceName: iface,
    endpointPorts: config.endpointPorts,
  },);
  await addAddressesAndUp({ config, },);
```

The config-tree implementation preserves unrelated and unknown JSON fields,
removes stale rules owned by the same interface,
and appends one enabled rule per sorted port.
The ownership and replacement path appears in
`package/wg/cli-quicker/src/opensnitch-config-tree.ts:275-424`:

```ts
const prefix = managedPrefix({ interfaceName, },);
const retainedRules = existingRules.filter(function retainRule(rule,): boolean {
  return !isManagedRule({
    value: rule,
    prefix,
  },);
},);
const managedRules = managedPorts.map(function toRule(port,): JsonRecord {
  return createManagedRule({
    interfaceName,
    port,
  },);
},);

const replacementChain: JsonRecord = {
  ...targetChain,
  Rules: [
    ...retainedRules,
    ...managedRules,
  ],
};
```

Each generated rule uses description
`wg-quicker managed endpoint [<interface>] [netns:<namespace-key>] UDP destination port <port>`,
`Enabled = true`,
string `Position = "0"`,
and target `accept`.
The namespace key is a truncated SHA-256 digest of `/proc/self/ns/net` identity.
The same interface name in another namespace therefore has a different ownership prefix,
manifest,
and interface lock;
shared config reconciliation retains both rule sets.
The integration requires OpenSnitch's nftables backend,
version 1 system-firewall schema,
an enabled top-level firewall,
and exactly one `inet opensnitch mangle_output` chain.
It fails and rolls tunnel startup back rather than writing a rule that the daemon would ignore.
The released 1.8.0 defaults in
`evilsocket/opensnitch@v1.8.0:daemon/data/default-config.json:25-28` satisfy these requirements:

```json
"Firewall": "nftables",
"FwOptions": {
  "ConfigPath": "/etc/opensnitchd/system-fw.json"
}
```

An explicit `WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG` takes precedence.
Otherwise,
`package/wg/cli-quicker/src/opensnitch-daemon-config.ts:212-278` reads absolute `FwOptions.ConfigPath` from the
selected daemon config.
The final strict-deny fixture set only `WG_QUICKER_OPENSNITCH_DAEMON_CONFIG`,
resolved a non-default watched system-firewall path,
and passed traffic.

Before config mutation,
`package/wg/cli-quicker/src/opensnitch-state.ts` atomically persists namespace-scoped interface ownership,
config path,
and potential ports in private runtime state.
Cleanup uses that recorded path even if daemon `FwOptions.ConfigPath` changes while the interface is up.
It clears state only after config reconciliation and live-chain proof succeed.

Cleanup removes owned rules before checking whether the WireGuard link still exists.
`package/wg/cli-quicker/src/tunnel.ts:398-412` retries that removal even when `down` finds the link already absent.
The ordering in `package/wg/cli-quicker/src/tunnel-cleanup.ts:24-34` also recovers an allowance left by an
interrupted startup:

```ts
const iface = config.interfaceName;
await removeOpenSnitchEndpointAllowance({ interfaceName: iface, },);
await stopApplicationExemptions({
  interfaceName: iface,
  configured: config.exemptMark !== undefined,
},);
if (!(await linkExists({ interfaceName: iface, },)))
  return;
```

Reconciliation records formerly owned exact ports that no retained rule still accepts.
`down` requires those ports to disappear from the live nftables chain before it returns.
The final custom-path fixture checked the chain immediately after `down` and found no managed destination-port
rule while ICMP and NFQUEUE rules remained.

### OpenSnitch live reload needs bounded writes and convergence proof

OpenSnitch 1.8.0 reloads its complete system firewall for every filesystem `Write` or `Remove` event.
`evilsocket/opensnitch@v1.8.0:daemon/firewall/config/config.go:250-258` contains the watcher:

```go
func (c *Config) monitorConfigWorker() {
	for {
		select {
		case <-c.ctx.Done():
			goto Exit
		case event := <-c.watcher.Events:
			if (event.Op&fsnotify.Write == fsnotify.Write) || (event.Op&fsnotify.Remove == fsnotify.Remove) {
				c.LoadDiskConfiguration(common.ReloadConf)
			}
		}
	}
```

Each event deletes and recreates system rules in
`evilsocket/opensnitch@v1.8.0:daemon/firewall/nftables/monitor.go:56-61`:

```go
func (n *Nft) ReloadConfCallback() {
	log.Important("reloadConfCallback changed, reloading")
	n.DeleteSystemRules(!common.ForcedDelRules, !common.RestoreChains, log.GetLogLevel() == log.DEBUG)
	n.AddSystemRules(common.ReloadRules, !common.BackupChains)
}
```

An initial `writeFile` implementation could emit two `Write` events from truncate plus content write.
The released daemon then performed two back-to-back reloads,
emitted the following diagnostic,
and lost system rules in the disposable namespace:

```text
WAR nftables: error applying changes: conn.Receive: netlink receive: no such file or directory
```

Atomic path replacement was also rejected.
It changes the inode watched by OpenSnitch and produced unstable second-reload behavior during removal.

The current writer performs one positional write to the existing inode for each bounded attempt.
When new JSON is shorter,
it pads the document with valid trailing JSON whitespace instead of truncating.
`package/wg/cli-quicker/src/opensnitch-config-file.ts:102-180` contains the workaround:

```ts
const padding = Buffer.alloc(
  Math.max(
    0,
    minimumSize - replacement.length,
  ),
  ' ',
);
const payload = Buffer.concat([
  replacement,
  padding,
],);
await using handle = await open(
  path,
  'r+',
);
const { bytesWritten, } = await handle.write(
  payload,
  0,
  payload.length,
  0,
);
```

The recovery fixture later observed the same daemon diagnostic after some single positional writes.
One event reduces the trigger surface but does not eliminate OpenSnitch's internal reload race.
`package/wg/cli-quicker/src/opensnitch-live.ts:179-228` therefore requires consecutive live nftables listings in
the daemon's network namespace.
Every managed port rule must precede the first queue rule,
and formerly owned exact ports must be absent.
When no daemon exists,
the verifier lists surviving nftables tables and refuses to clear ownership while a stale managed allowance remains.
`package/wg/cli-quicker/src/opensnitch-operation.ts:288-338` performs one bounded same-inode rewrite and repeats
the proof after a missed convergence.
Startup fails and rolls back when that retry also fails.
If the daemon is stopped,
the persisted rule loads at its next start.

## Verification

### Versions and artifact identity

The combined test used:

- `wg-quicker` automatic-rule implementation through commit
  `ac93f26e22ab8af99660ac59cd40d8d4e609587f`;
- missing-link retry wiring commit `0166622637f1ca21eba12a473d5882d98ef5e361`;
- path-resolution and removal-verification fix commit
  `df738d6622712865caa2d34646bf3eb3f4d517bd`;
- initial single-write live-reload fix commit `ba90d31612ac5a09ca498444af1daed21718a81d`;
- OpenSnitch release `v1.8.0`,
  commit `b404c4c6316760fa7bc415509d3f8d747f7dc9cc`;
- released RPM SHA-256 `e06e9119daf764e56455b61c319e496274c0274bb53bb94a0ff1ab72967fea7d`;
- released daemon SHA-256 `8669e280de9135a921a7ffe48ec4b47ec15d2d1a947046cd4de9b23f8efaf6e3`;
- Bazzite 44 host kernel `7.1.5-ogc5.1.fc44.x86_64`;
- nftables backend,
  OpenSnitch process monitor `proc`,
  and disposable client and server network namespaces.

The compatibility fixture used a WireGuard peer on `192.0.2.2:2049`,
tunnel addresses `10.44.0.1/24` and `10.44.0.2/24`,
and `AllowedIPs = 0.0.0.0/0`.
The automatic-rule fixture used the same endpoint and tunnel addresses with
`AllowedIPs = 10.44.0.2/32`.
Both ran inside disposable namespaces.
No host route,
firewall,
or WireGuard interface was changed.

### Runnable package checks

```console
mise run //package/wg/cli-quicker:buildAndTest
mise run //package/wg/cli-quicker:test:integration:route
mise run //package/wg/cli-quicker:test:integration:bypass
mise run //package/wg/cli-quicker:lint:types
mise run //package/wg/cli-quicker:lint:oxlint
```

The combined boundary invocation used the built CLI and released daemon:

```console
sudo ip netns exec wgqosc813 env \
  WG_QUICKER_RUNTIME_DIRECTORY="$scratch/runtime" \
  WG_QUICKER_OPENSNITCH_DAEMON_CONFIG="$scratch/default-config.json" \
  WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG="$scratch/system-fw.json" \
  "$node_bin" package/wg/cli-quicker/dist/final/node/index.mjs \
  up "$scratch/wgqos.conf"

sudo ip netns exec wgqosc813 ping -c 3 -w 5 10.44.0.2
sudo ip netns exec wgqosc813 wg show wgqos
sudo ip netns exec wgqosc813 nft list table inet opensnitch
sudo ip netns exec wgqosc813 nft list table inet wg-quicker-wgqos
```

The fixture prepared OpenSnitch without installing it on the host:

```console
mkdir --parents "${HOME}/temp/agent"
chmod 700 "${HOME}/temp/agent"
gh release download v1.8.0 \
  --repo evilsocket/opensnitch \
  --pattern 'opensnitch-1.8.0-1.x86_64.rpm' \
  --dir "${HOME}/temp/agent/opensnitch-v1.8.0-rpm"

cd -- "${HOME}/temp/agent/opensnitch-v1.8.0-rpm/root"
rpm2cpio ../opensnitch-1.8.0-1.x86_64.rpm \
  | cpio --extract --make-directories --preserve-modification-time
```

The daemon ran inside the client namespace with fixture-local configuration and rules:

```console
sudo ip netns exec wgqosc813 env \
  LD_LIBRARY_PATH="${HOME}/temp/agent/opensnitch-v1.8.0-rpm/lib" \
  "${HOME}/temp/agent/opensnitch-v1.8.0-rpm/root/usr/bin/opensnitchd" \
  -config-file "$scratch/default-config.json" \
  -fw-config-file "$scratch/system-fw.json" \
  -rules-path "$scratch/rules" \
  -process-monitor-method proc \
  -ui-socket "unix://$scratch/osui.sock" \
  -debug
```

The exact network topology is the same two-namespace,
physical-veth,
and two-peer topology implemented by
`package/wg/cli-quicker/src/tunnel-route.integration.test.ts`.
The combined test added OpenSnitch to its client namespace before invoking the real `wg-quicker` bundle.

### Working catalog

- Default action `allow`,
  no endpoint exclusion:
  the OpenSnitch log captured outer UDP mark `0xca6c`,
  WireGuard completed a recent handshake,
  transferred `476 B` received and `532 B` sent,
  and ping received `3` of `3` replies.
- Default action `deny`,
  endpoint-port system rule `udp dport 2049 accept` before the NFQUEUE rules:
  WireGuard completed a recent handshake,
  transferred `476 B` received and `532 B` sent,
  and ping received `3` of `3` replies.
- Initial automatic configuration under default action `deny`:
  `wg-quicker up` emitted the policy-widening warning,
  inserted its visible config rule,
  produced one OpenSnitch reload callback,
  retained ICMP and NFQUEUE rules,
  completed a handshake,
  transferred `476 B` received and `564 B` sent,
  and ping received `3` of `3` replies.
  After a measured `20`-second monitor interval,
  the managed and queue rules remained and ping received `2` of `2` replies.
- Current automatic cleanup:
  `wg-quicker down` removed the managed config and nftables rule,
  retained OpenSnitch ICMP and NFQUEUE rules,
  kept the valid JSON file at measured size `8174` bytes through whitespace padding,
  and removed the WireGuard link.
- Final custom-path fixture:
  only `WG_QUICKER_OPENSNITCH_DAEMON_CONFIG` was set;
  `wg-quicker` followed non-default `FwOptions.ConfigPath`,
  installed UDP port `2050`,
  completed a handshake,
  and ping received `3` of `3` replies under default deny.
  Immediately after `down` returned,
  port `2050` was absent while ICMP and NFQUEUE rules remained.
- Final namespace-scoped strict-deny fixture against released OpenSnitch 1.8.0:
  the config carried `wgqscope` plus a namespace key,
  endpoint port `2054` appeared before NFQUEUE,
  WireGuard completed a handshake,
  and ping received `3` of `3` replies.
  Immediately after `down`,
  the config rule,
  live port rule,
  manifest,
  and WireGuard link were absent while NFQUEUE remained.
- Recovery fixture under default action `deny`:
  endpoint port `2051` was accepted before NFQUEUE,
  a private manifest recorded the original absolute config path and port,
  and ping received `3` of `3` replies.
  After daemon `ConfigPath` changed from fixture path A to path B,
  `down` removed the rule from path A,
  left path B untouched,
  removed the link,
  and cleared the manifest.
- Stale-live-rule recovery fixture:
  with JSON already clean but port `2051` injected into the live chain,
  `down` retained the manifest after negative proof failed while still removing the WireGuard link.
  A later cleanup after live state recovered verified absence and cleared the manifest.
- Missing-link recovery fixture:
  after the WireGuard link was deleted externally,
  `down` removed the config rule and manifest before reporting that the link was absent.
- Network-namespace ownership fixture:
  two namespaces installed interface name `wgshared` with ports `2052` and `2053` into one shared config file.
  Distinct namespace-scoped descriptions and manifests coexisted;
  namespace A cleanup removed only port `2052` and retained namespace B's rule and manifest.
- Processless stale-kernel fixture:
  with no `opensnitchd` process,
  JSON already clean,
  and a surviving table accepting port `2053`,
  cleanup retained its manifest after both bounded proofs failed.
  Removing the stale table and retrying cleanup cleared the manifest.
- Network-namespace daemon isolation control:
  package unit tests passed while released `opensnitchd` ran in a different network namespace;
  daemon detection excluded that unrelated process by `/proc/<pid>/ns/net` identity.
- OpenSnitch started before `wg-quicker`:
  both nftables tables coexisted and the default-allow tunnel passed traffic.
- OpenSnitch restarted while the WireGuard fixture existed,
  followed by a fresh `wg-quicker down` and `up`:
  the endpoint-port rule still passed traffic.
- `wg-quicker down` while OpenSnitch ran:
  the WireGuard link and `wg-quicker` table disappeared,
  while OpenSnitch's populated table remained.

### Failing catalog

- Default action `deny`,
  `InterceptUnknown = false`,
  and no endpoint system rule:
  `wg-quicker up` exited successfully,
  but latest handshake remained `0`,
  transfer stayed at `0 B` received and `148 B` sent,
  and ping received `0` of `2` replies.
- OpenSnitch's disabled stock exclusion was not a workaround for the fixture's port `2049` because its only match is
  UDP destination port `51820`.
- A truncate-plus-write implementation emitted two OpenSnitch reload callbacks for one logical edit.
  The daemon logged `conn.Receive: netlink receive: no such file or directory` and dropped system rules.
  One positional write plus whitespace padding replaced that implementation.
- Released OpenSnitch 1.8.0 also emitted the same diagnostic after some later single-write configuration and cleanup
  attempts.
  Consecutive kernel-state proof caught one missed convergence;
  current `wg-quicker` performs one bounded same-inode rewrite before failing safely.
- OpenSnitch `Firewall = iptables` does not consume rules nested in the nftables `mangle_output` chain.
  Current `wg-quicker` rejects that backend before writing.

## Verified workarounds

### Keep default allow and apply application rules

OpenSnitch's released default is `DefaultAction = allow`.
The combined fixture worked without a WireGuard-specific exception,
and OpenSnitch preserved `wg-quicker`'s mark.

Tradeoff:
a no-process connection that has no matching rule receives the permissive default.
This is unsuitable when the intended policy requires unknown kernel traffic to fail closed.

### Use current wg-quicker automatic endpoint rules

For strict default deny,
run current `wg-quicker up` normally.
Standard OpenSnitch 1.8 paths need no additional setting.
Custom daemon deployments provide daemon config path;
`wg-quicker` follows its `FwOptions.ConfigPath`:

```console
WG_QUICKER_OPENSNITCH_DAEMON_CONFIG=/custom/default-config.json \
wg-quicker up wg0
```

Set `WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG` only to override that `FwOptions` path.
System-firewall paths,
daemon-config overrides,
and runtime-directory overrides must be absolute.
`wg-quicker` adds the actual peer endpoint ports,
records cleanup ownership before mutation,
requires accept rules before NFQUEUE,
and warns with exact scope.
The disposable strict-deny test verified endpoint port `2049` without restarting the daemon.
`down` removes the managed rules through the same live-reload path.
A failed live proof retains private cleanup state;
a later `down` retries the originally recorded file and ports even if the link or current daemon path changed.

Tradeoff:
each port-only rule bypasses OpenSnitch application attribution for every process's outbound UDP connection to that
destination port while the interface is up.
It does not bypass `wg-quicker` routing or encryption.
The integration intentionally rejects OpenSnitch's iptables backend and unsupported system-firewall schemas.
Its runtime lock serializes wg-quicker processes,
not concurrent edits from the OpenSnitch UI;
avoid changing system rules while a tunnel lifecycle command runs.

For an older `wg-quicker` build,
enable or add OpenSnitch's system rule for the actual endpoint port and restart the packaged daemon after saving:

```console
sudo systemctl restart opensnitch.service
```

The released `Exclude WireGuard VPN from being intercepted` rule works unchanged only when the destination port is
`51820`.

### Use OpenSnitch eBPF attribution or debug unknown connections

OpenSnitch documents eBPF tunnel probes and the `Debug invalid connections` fallback.
A user can approve the resulting kernel or unknown connection rather than bypassing the queue by port.

Tradeoff:
this depends on kernel tracing features,
OpenSnitch's eBPF objects,
and successful tunnel probe attachment.
The combined fixture deliberately used `proc`,
so this path is upstream-documented rather than locally verified here.

## What does not work

- Allowing only the `wg-quicker` executable does not identify the outer peer packet.
  The packet observed in the fixture had no userspace UID or PID because the WireGuard kernel interface emitted it.
- Assuming every WireGuard peer uses destination port `51820` does not work.
  WireGuard endpoints may use another configured port,
  including the fixture and active host port `2049`.
- Setting `ExemptMark` is not an OpenSnitch bypass.
  It intentionally sends selected application sockets through physical routing,
  but those packets still cross OpenSnitch's output interception chain.
- Writing `system-fw.json` through truncate plus content write is not safe for this integration.
  OpenSnitch reloads each filesystem write event;
  duplicate callbacks reproduced netlink errors and missing system rules.
- Replacing `system-fw.json` atomically by rename is not safe for OpenSnitch's file-inode watcher.
  The verified writer retains the inode and pads shorter JSON instead.
- Adding only a nested nftables rule while OpenSnitch uses its iptables backend has no effect.
  Current `wg-quicker` rejects that backend.
- Treating OpenSnitch's shutdown warnings as tunnel-failure evidence does not work.
  The fixture emitted `queue ... stuck, closing by timeout` only during daemon teardown after successful handshakes;
  the live connection catalog showed no such warning.
- OpenSnitch issue `#1629` reports ProtonVPN and OpenSnitch 1.7.x timeout behavior.
  It is not a reproduction of the released 1.8.0 nftables result recorded here.

## Upstream filing decision

`.out-of-scope/` contains no OpenSnitch,
WireGuard,
or firewall exemption.

The upstream issue and pull-request search covered open and closed results for `WireGuard`,
`VPN`,
`NFQUEUE`,
and kernel connections.
The relevant threads were read in full:

- `#454`,
  the original WireGuard and kernel-process investigation;
- `#688`,
  an incomplete system-firewall edit and netlink-error report;
- `#781`,
  the fixed 1.6.0 periodic nftables netlink-error report after firewall reload;
- `#976`,
  a closed kernel-requirement nftables error report;
- `#853`,
  the fixed 1.6.0-rc.4 regression;
- `#1250`,
  the fixed IPv6 tunnel attribution issue;
- `#1406`,
  the incomplete hardened-kernel report;
- open `#1629`,
  the ProtonVPN 1.7.x timeout report;
- merged pull request `#1423`,
  which moved OpenSnitch 1.8.0 into its application-specific nftables table.

The required filing constraints resolve as follows:

1.  Upstream fault is **no** for the compatibility problem.
    Released 1.8.0 worked with `wg-quicker` under default allow and a valid system rule.
    Strict deny dropping an unattributed connection follows the configured policy.
    Duplicate file-write events exposed a separate reload quirk,
    but one-event consumer writing resolved it without an upstream change.
2.  An upstream fix is **not applicable** to the verified compatibility result.
    A separate attribution regression would be fixable,
    but this harness did not reproduce one.
3.  The use case is **supported**.
    The FAQ discusses WireGuard,
    source includes tunnel probes,
    and the default system-firewall data includes a WireGuard exclusion.
4.  External contributions are **welcome**.
    The repository has bug and feature templates,
    no `CONTRIBUTING.md` or pull-request template,
    and no prohibition on assisted reports was found.
    Pull request `#1423` demonstrates a recent external firewall contribution and maintainer review.
5.  Likely upstream action is **none** from this result.
    The strict-deny behavior is policy enforcement.
    Closed issue `#781` already tracks a related nftables reload failure,
    while this reproduction depends on a disposable namespace and a rejected multi-event writer.
6.  A minimal compatible fix is **not applicable** because constraints 1 and 5 fail.
    The consumer-side endpoint-port configuration was tested instead.

### Upstream filing artifact

Nothing should be filed or added to an existing thread from this investigation.
The result confirms compatibility and an existing documented policy requirement.
It does not reproduce open issue `#1629`'s ProtonVPN timeout.
The rejected multi-event writer resembles closed issue `#781`,
but the disposable namespace evidence does not establish a current host regression and the consumer workaround is
verified.
Commenting on either thread would conflate different versions or triggers rather than advance diagnosis.

## Sources

- [OpenSnitch FAQ][opensnitch-faq]
- [OpenSnitch system rules documentation][opensnitch-system-rules]
- [OpenSnitch 1.8.0 release][opensnitch-release]
- [OpenSnitch WireGuard issue 454][opensnitch-454]
- [OpenSnitch nftables netlink issue 688][opensnitch-688]
- [OpenSnitch periodic reload issue 781][opensnitch-781]
- [OpenSnitch nftables requirements issue 976][opensnitch-976]
- [OpenSnitch ProtonVPN issue 1629][opensnitch-1629]
- [OpenSnitch application-specific nftables pull request 1423][opensnitch-1423]

[opensnitch-faq]: https://github.com/evilsocket/opensnitch/wiki/FAQs
[opensnitch-system-rules]: https://github.com/evilsocket/opensnitch/wiki/System-rules
[opensnitch-release]: https://github.com/evilsocket/opensnitch/releases/tag/v1.8.0
[opensnitch-454]: https://github.com/evilsocket/opensnitch/issues/454
[opensnitch-688]: https://github.com/evilsocket/opensnitch/issues/688
[opensnitch-781]: https://github.com/evilsocket/opensnitch/issues/781
[opensnitch-976]: https://github.com/evilsocket/opensnitch/issues/976
[opensnitch-1250]: https://github.com/evilsocket/opensnitch/issues/1250
[opensnitch-1629]: https://github.com/evilsocket/opensnitch/issues/1629
[opensnitch-1423]: https://github.com/evilsocket/opensnitch/pull/1423
