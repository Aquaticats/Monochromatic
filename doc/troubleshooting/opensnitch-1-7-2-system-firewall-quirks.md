# OpenSnitch 1.7.2 system firewall: IPv6 addresses build IPv4 matches, nothing filters while stopped

Status note at 2026-11-18T13:09:51-05:00:
the host moved to the AUR `opensnitch-git` on 2026-11-18,
at 2026-11-18T12:38:18-05:00 by `pacman.log`.
The package is `1.8.0.r224.a1353848-1`,
and its daemon reports `1.9.0`.
The text of this doc describes 1.7.2 and was not rechecked on 1.9.0.
The table and chain names changed:
1.7.2 uses the tables `filter`,
`mangle` and `nat`,
master uses one `inet opensnitch` table with chains `filter_input`,
`filter_forward`,
`mangle_output` and so on,
and the daemon hardcodes `opensnitch`/`filter_input` for its DNS rule and `mangle_output` for interception.
Where this doc says `inet filter input`,
the matching chain on 1.9.0 is `inet opensnitch filter_input`.
See [the AUR packages doc](opensnitch-ebpf-module-clang-23-needs-llvm-22.md) for the switch,
the eBPF module build and the source lines.

Status at 2026-11-18T12:16:50-05:00:
agent-written from a live ufw-to-OpenSnitch cutover on a CachyOS desktop;
not reviewed or approved by the owner.
The host runs CachyOS `opensnitch 1.7.2-4.1` (`cachyos-extra-znver4`),
a rebuild of Arch's `extra/opensnitch 1.7.2-4`.
OpenSnitch's system firewall (`/etc/opensnitchd/system-fw.json`) replaced ufw as the only inbound firewall.
The owner's choices behind that setup,
and the generated rule set,
live in the machine's migration notes;
this file records the tool's behaviour.

Source citations are pinned to commit `8c3fa40cf476d1a68df04d49bb98509437a47b4f`,
which the annotated tag `v1.7.2` points to
(`gh api repos/evilsocket/opensnitch/git/tags/ce824b3649e0b6ccdbe071452e7dae4b18d207ed`).

## Symptom

Each behaviour turned up while writing an inbound policy equivalent to ufw's.

1.  An `ip` or `ip6` system-rule statement with an IPv6 address,
    such as `{"Name": "ip6", "Values": [{"Key": "daddr", "Value": "ff02::fb"}]}`,
    loads without any warning,
    but builds an IPv4 match whose comparison data is empty.
    It can never match the intended IPv6 traffic.
2.  Editing `system-fw.json` while the daemon runs triggers a live reload.
    Afterwards OpenSnitch's own DNS-capture rule,
    `udp sport 53 queue flags bypass to 0`,
    sat at the end of `inet filter input` instead of first:

    ```text
    ...
    limit rate 3/minute burst 10 packets log prefix "[opensnitch inbound drop] " drop
    udp sport 53 queue flags bypass to 0
    ```

    At the end it never sees DNS replies,
    because `ct state established,related accept` sits before it and accepts them.
    The daemon log then showed:

    ```text
    [2026-11-18 16:59:31]  WAR  nftables DNS rule not in 1st position (28)
    [2026-11-18 16:59:31]  IMP  nftables firewall rules changed, reloading
    ```

3.  Stopping `opensnitchd` removes its chains.
    After `systemctl stop opensnitchd`,
    `nft list chain inet filter input` failed:

    ```text
    Error: No such file or directory
    list chain inet filter input
                           ^^^^^
    ```

    With ufw disabled,
    nothing filters inbound traffic while the daemon is not running.
    The packaged unit starts the daemon `After=network.target`,
    with `Restart=always` and `RestartSec=30`,
    while ufw's unit runs `Before=sysinit.target`.
4.  An `icmp` or `icmpv6` statement with no `type` value matches nothing.
    A comma-separated port list cannot contain a range.
    ICMPv6 MLD types (130 to 132 and 143) have no names.
    An `iifname` statement reads the interface from `Key`,
    not `Value`.
5.  Every stock and generated system rule says `"Position": "0"`,
    yet the chain ends up in the same order as the JSON array.
6.  An application rule with `"sensitive": false` lowercases its regular expression,
    so `[A-Z]` becomes `[a-z]` and `\S` becomes `\s`.
7.  A Python script cannot be allowed by its own path:
    `/usr/bin/torbrowser-launcher` connects as `/usr/bin/python3.14`,
    so a `process.path` rule for the script never matches,
    and one for the interpreter would allow every Python script.
8.  A rule file put in place by `install` or `mv` is not loaded until the daemon restarts.

## Root cause

### Address statements are IPv4-only

The parser sends both statement names to one constructor,
passing the table family,
not the statement name
([daemon/firewall/nftables/parser.go:98-104][parser-ip]):

```go
// daemon/firewall/nftables/parser.go
	case exprs.NFT_FAMILY_IP, exprs.NFT_FAMILY_IP6:
		exprIP, err := exprs.NewExprIP(family, expression.Statement.Values, cmpOp)
		if err != nil {
			log.Warning("%s addr statement error: %s", logTag, err)
			return nil
		}
		exprList = append(exprList, *exprIP...)
```

`NewExprIP` pins an `inet` table to IPv4 and marks IPv6 as a TODO
([daemon/firewall/nftables/exprs/ip.go:34-44][ip-new]):

```go
// daemon/firewall/nftables/exprs/ip.go
func NewExprIP(family string, ipOptions []*config.ExprValues, cmpOp expr.CmpOp) (*[]expr.Any, error) {
	var exprIP []expr.Any

	// if the table family is inet, we need to specify the protocol of the IP being added.
	if family == NFT_FAMILY_INET {
		exprIP = append(exprIP, &expr.Meta{Key: expr.MetaKeyNFPROTO, Register: 1})
		exprIP = append(exprIP, &expr.Cmp{Op: expr.CmpOpEq, Register: 1, Data: []byte{unix.NFPROTO_IPV4}})
	}
	for _, ipOpt := range ipOptions {
		// TODO: ipv6
		switch ipOpt.Key {
```

The payload offsets are the IPv4 header's
([exprs/ip.go:91-99][ip-payload]):

```go
// daemon/firewall/nftables/exprs/ip.go
	case NFT_DADDR:
		// NOTE 1: if "what" is daddr and SourceRegister is part of the Payload{} expression,
		// the rule is not added.
		return &expr.Payload{
			DestRegister: 1,
			Offset:       16, // daddr
			Base:         expr.PayloadBaseNetworkHeader,
			Len:          4, // 16 ipv6
		}
```

The address bytes come from `To4()`,
which returns `nil` for an IPv6 address,
so the comparison carries no data and no error is raised
([exprs/ip.go:114-127][ip-get]):

```go
// daemon/firewall/nftables/exprs/ip.go
func getExprIP(value string, cmpOp expr.CmpOp) (*[]expr.Any, error) {
	ip := net.ParseIP(value)
	if ip == nil {
		return nil, fmt.Errorf("Invalid IP: %s", value)
	}

	return &[]expr.Any{
		&expr.Cmp{
			Op:       cmpOp,
			Register: 1,
			Data:     ip.To4(),
		},
	}, nil
}
```

`getExprRangeIP` uses `To4()` the same way for `a-b` ranges.
`daemon/firewall/nftables/exprs/ip.go` is byte-identical at `v1.8.0`
(commit `b404c4c6316760fa7bc415509d3f8d747f7dc9cc`)
and on `master` at `a1353848ba1b660320e90cefea782c3fba272c00`
([master exprs/ip.go:114-127][ip-get-master]).
Its last change was `a18404fba9` on 2023-07-08,
"tests: added sys-fw ip and iface tests".

The session first described this as a `To4()` bug alone.
That reading was too narrow:
the `nfproto` comparison and the payload offsets are IPv4-only as well,
so returning 16 bytes from `getExprIP` alone would still not match IPv6 packets.
The prototype in "Verification" changes all three.

### A configuration reload puts the system rules ahead of the DNS-capture rule

A change to `system-fw.json` runs `ReloadConfCallback`,
which deletes the system rules and adds them again
([daemon/firewall/nftables/monitor.go:53-58][monitor-conf]):

```go
// daemon/firewall/nftables/monitor.go
// ReloadConfCallback gets called after the configuration changes.
func (n *Nft) ReloadConfCallback() {
	log.Important("reloadConfCallback changed, reloading")
	n.DeleteSystemRules(!common.ForcedDelRules, !common.RestoreChains, log.GetLogLevel() == log.DEBUG)
	n.AddSystemRules(common.ReloadRules, !common.BackupChains)
}
```

System rules are inserted at the top of the chain
(see "Rules are added backwards, each at the top"),
so after the reload they all sit ahead of the interception rule that used to be first.
The rules monitor checks the DNS rule's position
([monitor.go:29-34][monitor-check]):

```go
// daemon/firewall/nftables/monitor.go
		for rdx, r := range rules {
			if string(r.UserData) == InterceptionRuleKey {
				if c.Table.Name == exprs.NFT_CHAIN_FILTER && c.Name == exprs.NFT_HOOK_INPUT && rdx != 0 {
					log.Warning("nftables DNS rule not in 1st position (%d)", rdx)
					return false
				}
```

and,
when the check fails,
removes and re-adds the interception rules
([monitor.go:60-66][monitor-rules]):

```go
// daemon/firewall/nftables/monitor.go
// ReloadRulesCallback gets called when the interception rules are not present.
func (n *Nft) ReloadRulesCallback() {
	log.Important("nftables firewall rules changed, reloading")
	n.DisableInterception(log.GetLogLevel() == log.DEBUG)
	time.Sleep(time.Millisecond * 500)
	n.EnableInterception()
}
```

`EnableInterception` calls `QueueDNSResponses`,
which inserts the DNS rule at position 0
([daemon/firewall/nftables/rules.go:35-37][rules-dns]):

```go
// daemon/firewall/nftables/rules.go
		// nft list ruleset -a
		n.Conn.InsertRule(&nftables.Rule{
			Position: 0,
```

So the misplacement lasts until the next monitor check:
at most `FwOptions.MonitorInterval`,
`15s` in the stock config.
During that window DNS replies are not queued,
and prompts lose the host names of the connections they show.

The session at first treated a daemon restart as the required workaround.
The monitor code says the order repairs itself.
The self-repair was not observed directly:
the daemon was restarted at 16:59:41 UTC,
ten seconds after the monitor logged its reload,
and the chain was not listed in between.
After the restart,
the DNS rule was first again.

### Stop deletes the system rules and tables

`Stop` cleans up everything the daemon added
([daemon/firewall/nftables/nftables.go:102-111][nft-stop]):

```go
// daemon/firewall/nftables/nftables.go
// Stop deletes the firewall rules, allowing network traffic.
func (n *Nft) Stop() {
	n.ErrChan = make(chan string, 100)
	if n.IsRunning() == false {
		return
	}
	n.StopConfigWatcher()
	n.StopCheckingRules()
	n.CleanRules(log.GetLogLevel() == log.DEBUG)
```

`CleanRules` calls `DeleteSystemRules` with `force` and `restoreExistingChains` set
([nftables.go:144-148][nft-clean]),
which deletes the rules,
restores the policy of the chains it backed up,
and deletes the tables
([daemon/firewall/nftables/system.go:139-153][system-delete]):

```go
// daemon/firewall/nftables/system.go
func (n *Nft) DeleteSystemRules(force, restoreExistingChains, logErrors bool) {
	n.Lock()
	defer n.Unlock()

	if err := n.delRulesByKey(SystemRuleKey); err != nil {
		log.Warning("error deleting interception rules: %s", err)
	}

	if restoreExistingChains {
		n.restoreBackupChains()
	}
	if force {
		n.DelSystemTables()
	}
}
```

The backup and restore exist so that a drop policy cannot outlive the daemon and block all inbound traffic
([daemon/firewall/nftables/chains.go:167-186][chains-backup]):

```go
// daemon/firewall/nftables/chains.go
// backupExistingChains saves chains with Accept policy.
// If the user configures the chain policy to Drop, we need to set it back to Accept,
// in order not to block incoming connections.
func (n *Nft) backupExistingChains() {
	if chains, err := n.Conn.ListChains(); err == nil {
		for _, c := range chains {
			if c.Policy != nil && *c.Policy == nftables.ChainPolicyAccept {
				log.Debug("%s backing up existing chain with policy ACCEPT: %s, %s", logTag, c.Name, c.Table.Name)
				origSysChains[getChainKey(c.Name, c.Table)] = c
			}
		}
	}
}

func (n *Nft) restoreBackupChains() {
	for _, c := range origSysChains {
		log.Debug("%s Restoring chain policy to accept: %s, %s", logTag, c.Name, c.Table.Name)
		*c.Policy = nftables.ChainPolicyAccept
		n.Conn.AddChain(c)
	}
```

This is upstream's design.
In issue [#1183][issue-1183],
"Traffic not blocked if service not running",
the maintainer answered that
[the service must be running to block outbound connections][issue-1183-running]
and suggested a separate nftables table with a drop policy
([comment 2415318670][issue-1183-table]).
The maintainer closed the issue on 2025-12-12 with
[We added two options for this feature request][issue-1183-closed].
The thread does not name the two options,
and this session did not identify them;
v1.8.0 was published on 2025-12-15.
Issue [#1225][issue-1225] is the opposite failure:
a drop policy left behind after disabling the firewall.

### Statement builders: empty ICMP sets, single-port sets, interface keys

`buildICMPRule` splits the `type` value on commas
([daemon/firewall/nftables/rule_helpers.go:40-44][icmp-split]),
then uses a single comparison only when exactly one type was given.
Zero types take the set branch with an empty anonymous set,
which matches nothing
([rule_helpers.go:80-99][icmp-set]):

```go
// daemon/firewall/nftables/rule_helpers.go
	if len(setElements) == 1 {
		ICMPrule = append(ICMPrule, ICMPtemp...)
	} else {
		set := &nftables.Set{
			Anonymous: true,
			Constant:  true,
			Table:     tbl,
			KeyType:   setType,
		}
		if err := n.Conn.AddSet(set, setElements); err != nil {
			log.Warning("%s AddSet() error: %s", logTag, err)
			return nil
		}
		sysSets = append(sysSets, []*nftables.Set{set}...)

		ICMPrule = append(ICMPrule, []expr.Any{
			&expr.Lookup{
				SourceRegister: 1,
				SetName:        set.Name,
				SetID:          set.ID,
```

`GetICMPv6Type` in `daemon/firewall/nftables/exprs/utils.go` names destination-unreachable,
packet-too-big,
time-exceeded,
parameter-problem,
echo request and reply,
router and neighbour solicitation and advertisement,
and redirect;
an unknown name maps to type 0.
MLD queries and reports have no name.

A comma in a port value makes a set,
and each element must parse as one integer,
so `"47984-47990,48010"` silently drops the range
([daemon/firewall/nftables/exprs/port.go:50-73][port-set]):

```go
// daemon/firewall/nftables/exprs/port.go
func NewExprPortSet(portv string) *[]nftables.SetElement {
	setElements := []nftables.SetElement{}
	ports := strings.Split(portv, ",")
	for _, portv := range ports {
		portExpr := exprPortSubSet(portv)
		if portExpr != nil {
			setElements = append(setElements, *portExpr...)
		}
	}

	return &setElements
}

func exprPortSubSet(portv string) *[]nftables.SetElement {
	port, err := strconv.Atoi(portv)
	if err != nil {
		return nil
	}

	return &[]nftables.SetElement{
		{Key: binaryutil.BigEndian.PutUint16(uint16(port))},
	}

}
```

A value with `-` and no comma becomes a range (`NewExprPortRange`),
which works.

The interface statement takes the name from the first value's `Key`
([parser.go:89-96][parser-iface]):

```go
// daemon/firewall/nftables/parser.go
	// TODO: support iif, oif
	case exprs.NFT_IIFNAME, exprs.NFT_OIFNAME:
		isOut := expression.Statement.Name == exprs.NFT_OIFNAME
		iface := expression.Statement.Values[0].Key
		if iface == "" {
			log.Warning("%s network interface statement error: %s", logTag, expression.Statement.Name)
			return nil
		}
		exprList = append(exprList, *exprs.NewExprIface(iface, isOut, cmpOp)...)
```

and a trailing `*` becomes a prefix match
([daemon/firewall/nftables/exprs/iface.go:24-33][iface-wild]):

```go
// daemon/firewall/nftables/exprs/iface.go
func ifname(n string) []byte {
	buf := make([]byte, 16)
	length := len(n)
	// allow wildcards
	if n[length-1:] == "*" {
		return []byte(n[:length-1])
	}
	copy(buf, []byte(n+"\x00"))
	return buf
}
```

### Rules are added backwards, each at the top

`AddSystemRules` walks each chain's rules from last to first
([system.go:121-128][system-loop]):

```go
// daemon/firewall/nftables/system.go
			for i := len(chain.Rules) - 1; i >= 0; i-- {
				if chain.Rules[i].UUID == "" {
					uuid := uuid.New()
					chain.Rules[i].UUID = uuid.String()
				}
				if chain.Rules[i].Enabled {
					if err4, _ := n.AddSystemRule(chain.Rules[i], chain); err4 != nil {
						n.SendError(fmt.Sprintf("%s (%s)", err4, chain.Rules[i].UUID))
```

and `AddSystemRule` calls `InsertRule` with the rule's `Position`
([system.go:174][system-insert]),
which inserts at the top for position 0
([rules.go:182-201][rules-insert]).
Inserting the last rule first,
each at the top,
leaves the chain in array order.

### A script's process path is its interpreter

The kernel runs a script through the interpreter its shebang names,
so the process executable,
and `process.path`,
is the interpreter.
The `process.command` operand matches the whole command line,
the arguments joined with spaces
([daemon/rule/operator.go:352-353][operator-cmd]):

```go
// daemon/rule/operator.go
	} else if o.Operand == OpProcessCmd {
		return o.cb(strings.Join(con.Process.Args, " "))
```

For `/usr/bin/torbrowser-launcher`,
whose shebang is `#!/usr/bin/python`,
the first argument is `/usr/bin/python` and the second is the script's path.
A `list` operator requires every child to match
([operator.go:327-333][operator-list]):

```go
// daemon/rule/operator.go
func (o *Operator) listMatch(con interface{}, hasChecksums bool) bool {
	res := true
	for i := 0; i < len(o.List); i++ {
		res = res && o.List[i].Match(con.(*conman.Connection), hasChecksums)
	}
	return res
}
```

1.7.2 reads the children from the operator's `list` array.
The older form,
the list serialized as JSON inside `data`,
is still read but marked deprecated
([daemon/rule/loader.go:356-366][loader-oplist]):

```go
// daemon/rule/loader.go
// Deprecated: rule.Operator.Data no longer holds the operator list in json format as string.
func (l *Loader) unmarshalOperatorList(op *Operator) error {
	if op.Type == List && len(op.List) == 0 && op.Data != "" {
		if err := json.Unmarshal([]byte(op.Data), &op.List); err != nil {
			return fmt.Errorf("error loading rule of type list: %s", err)
		}
		op.Data = ""
	}

	return nil
}
```

### Rule files reload on Write and Remove only

The rules directory watcher loads a rule on a `Write` event and unloads it on a `Remove` event
([loader.go:472-486][loader-live]):

```go
// daemon/rule/loader.go
		case event := <-l.watcher.Events:
			// a new rule json file has been created or updated
			if event.Op&fsnotify.Write == fsnotify.Write {
				if strings.HasSuffix(event.Name, ".json") {
					log.Important("Ruleset changed due to %s, reloading ...", path.Base(event.Name))
					if err := l.loadRule(event.Name); err != nil {
						log.Warning("%s", err)
					}
				}
			} else if event.Op&fsnotify.Remove == fsnotify.Remove {
				if strings.HasSuffix(event.Name, ".json") {
					log.Important("Rule deleted %s", path.Base(event.Name))
					// we only need to delete from memory rules of type Always,
					// because the Remove event is of a file, i.e.: Duration == Always
					l.deleteRule(event.Name)
```

`install` and `mv` put a file in place by rename,
which raises `Create` (or `Rename`),
not `Write`,
so the new rule is ignored until a restart.
The 1.8.0 loader behaves the same;
see [opensnitch-1-8-auto-reject-blocks-toolchain-updates.md][doc-auto-reject].

### Case-insensitive application rules lowercase the pattern

`Operator.Compile` lowercases the expression before compiling it
([daemon/rule/operator.go:144-152][operator-compile]),
and `reCmp` lowercases the value it matches
([operator.go:231-240][operator-recmp]):

```go
// daemon/rule/operator.go
	} else if o.Type == Regexp {
		o.cb = o.reCmp
		if o.Sensitive == false {
			o.Data = strings.ToLower(o.Data)
		}
		re, err := regexp.Compile(o.Data)
```

The same behaviour in 1.8.0,
and the rule-selection walk,
are in [opensnitch-1-8-auto-reject-blocks-toolchain-updates.md][doc-auto-reject].
`FindFirstMatch` is the same in 1.7.2
([daemon/rule/loader.go:496-515][loader-match]):
the first matching `reject`,
`deny` or `precedence` rule in bytewise name order wins,
otherwise the last matching rule does.

## Verification

Version under test:
CachyOS `opensnitch 1.7.2-4.1`,
nftables backend,
`ProcMonitorMethod: ebpf`,
kernel `7.3.4-1-cachyos`,
on 2026-11-18.

### Live ruleset matches the config

After starting the daemon with the generated `system-fw.json`,
`sudo nft list chain inet filter input` listed every configured rule in array order,
with OpenSnitch's DNS rule first:

```text
type filter hook input priority filter; policy drop;
udp sport 53 queue flags bypass to 0
iifname "lo" accept
ct state established,related accept
ct state invalid drop
iifname "virbr*" accept
iifname "podman*" accept
icmp type { destination-unreachable, echo-request, time-exceeded, parameter-problem } accept
meta l4proto ipv6-icmp accept
meta l4proto igmp accept
udp sport 67 udp dport 68 accept
udp sport 547 udp dport 546 accept
udp dport 5353 accept
ip daddr 239.255.255.250 udp dport 1900 accept
tcp dport 53317 accept
...
limit rate 3/minute burst 10 packets log prefix "[opensnitch inbound drop] " drop
```

`meta l4proto ipv6-icmp accept` came from
`{"Name": "meta", "Values": [{"Key": "l4proto", "Value": "icmpv6"}]}`,
and port ranges such as `tcp dport 47984-47990` came from one range per rule.

### Inbound drop works: netns test

A throwaway network namespace plays a LAN host.
The host listens on an allowed port (53317) and a port with no rule (8123):

```bash
#!/usr/bin/env bash
# inbound-test.sh
# Inbound test for the OpenSnitch system firewall: a throwaway netns plays a LAN host.
# Host listens on 10.199.0.1:53317 (allowed: LocalSend) and :8123 (not allowed); the netns probes both.
set -u
cleanup() { sudo ip netns del fwtest 2>/dev/null; sudo ip link del fwt0 2>/dev/null; kill "$L1" "$L2" 2>/dev/null; }
trap cleanup EXIT
sudo ip netns add fwtest
sudo ip link add fwt0 type veth peer name fwt1
sudo ip link set fwt1 netns fwtest
sudo ip addr add 10.199.0.1/30 dev fwt0
sudo ip link set fwt0 up
sudo ip netns exec fwtest ip addr add 10.199.0.2/30 dev fwt1
sudo ip netns exec fwtest ip link set fwt1 up
sudo ip netns exec fwtest ip link set lo up
listen() { python3 -I -c "import socket,sys; s=socket.socket(); s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1); s.bind(('10.199.0.1', int(sys.argv[1]))); s.listen(); s.settimeout(20); c,_=s.accept(); c.close()" "$1" & }
listen 53317; L1=$!
listen 8123; L2=$!
timeout 1 tail -f /dev/null
probe() { sudo ip netns exec fwtest python3 -I -c "import socket,sys; s=socket.socket(); s.settimeout(3)
try:
    s.connect(('10.199.0.1', int(sys.argv[1]))); print('port', sys.argv[1], 'CONNECTED')
except Exception as e:
    print('port', sys.argv[1], 'BLOCKED', type(e).__name__)" "$1"; }
probe 53317
probe 8123
sudo dmesg | grep 'opensnitch inbound drop' | tail -2
```

On this kernel the `veth` module had to be loaded first;
see [kernel-upgrade-without-reboot-removes-running-kernel-modules.md][doc-kernel-modules].
Output with ufw disabled:

```text
port 53317 CONNECTED
port 8123 BLOCKED TimeoutError
[ 6313.793227] [opensnitch inbound drop] IN=fwt0 OUT= MAC=... SRC=10.199.0.2 DST=10.199.0.1 LEN=60 ... PROTO=TCP SPT=48490 DPT=8123 ... SYN URGP=0
```

`fwt0` matches neither `virbr*` nor `podman*`,
so the probe exercised the drop policy.

### Application rules: Go RE2 checker

The migrated application rules were checked with Go's `regexp` package,
using a model of `FindFirstMatch` and `Operator` written from the sources in "Root cause",
including `list` operators and the `process.command` operand.
At 2026-11-18T12:16:50-05:00 it reported
`SUMMARY rules=112 bad=0 cases=391 fails=0`.
The cases were every old rule's path translated to this host,
current install paths,
lookalike paths that must not match,
and the executables holding sockets at the time.
For the Tor Browser launcher list rule,
the launcher's command line is allowed,
while other Python scripts,
a lookalike `/tmp/torbrowser-launcher`,
and the same command line under a non-Python executable are not.
A model written from the same reading proves the reading consistent,
not the daemon correct.

### Live checks after the cutover

- The coordinating session ran `git ls-remote` against a `git://git.kernel.org/` URL,
  which failed with "Software caused connection abort":
  the daemon applied the migrated `reject-git` rule.
- `sudo bpftool perf list` showed the daemon's eBPF programs attached as kprobes on `tcp_v4_connect`
  and `inet_dgram_connect`,
  so `ProcMonitorMethod: ebpf` works on kernel `7.3.4-1-cachyos`.
- With ufw off,
  `ip -6 route show default` listed a router-advertised default route
  (`proto ra`),
  and the coordinating session reported `ping -6` working,
  so the ICMPv6 accept rule passes neighbour discovery and router advertisements.

### IPv6 address statements: harness before and after the prototype

The harness prints what `NewExprIP` builds,
without root and without netlink.
It lives in the daemon module of a disposable upstream clone at `master`
(`a1353848ba1b660320e90cefea782c3fba272c00`,
origin `https://github.com/evilsocket/opensnitch.git`),
run with `cd daemon && go run ./cmd/ipharness`:

```go
// daemon/cmd/ipharness/main.go
// Harness: print the nftables expressions OpenSnitch builds for system-firewall ip/ip6 address statements.
package main

import (
	"fmt"

	"github.com/evilsocket/opensnitch/daemon/firewall/config"
	"github.com/evilsocket/opensnitch/daemon/firewall/nftables/exprs"
	"github.com/google/nftables/expr"
)

func main() {
	for _, c := range []struct{ family, key, value string }{
		{"inet", "daddr", "239.255.255.250"},
		{"inet", "daddr", "ff02::fb"},
		{"inet", "saddr", "fe80::1-fe80::ffff"},
		{"ip6", "daddr", "ff02::fb"},
	} {
		list, err := exprs.NewExprIP(c.family, []*config.ExprValues{{Key: c.key, Value: c.value}}, expr.CmpOpEq)
		fmt.Printf("%s %s %s err=%v\n", c.family, c.key, c.value, err)
		if list == nil {
			continue
		}
		for _, e := range *list {
			switch v := e.(type) {
			case *expr.Meta:
				fmt.Printf("  meta key=%d\n", v.Key)
			case *expr.Cmp:
				fmt.Printf("  cmp data=%v (len %d)\n", v.Data, len(v.Data))
			case *expr.Payload:
				fmt.Printf("  payload offset=%d len=%d\n", v.Offset, v.Len)
			case *expr.Range:
				fmt.Printf("  range from=%v to=%v\n", v.FromData, v.ToData)
			}
		}
	}
}
```

Unpatched output
(meta key 15 is `nfproto`;
data `[2]` is `NFPROTO_IPV4`):

```text
inet daddr 239.255.255.250 err=<nil>
  meta key=15
  cmp data=[2] (len 1)
  payload offset=16 len=4
  cmp data=[239 255 255 250] (len 4)
inet daddr ff02::fb err=<nil>
  meta key=15
  cmp data=[2] (len 1)
  payload offset=16 len=4
  cmp data=[] (len 0)
inet saddr fe80::1-fe80::ffff err=<nil>
  meta key=15
  cmp data=[2] (len 1)
  payload offset=12 len=4
  range from=[] to=[]
ip6 daddr ff02::fb err=<nil>
  payload offset=16 len=4
  cmp data=[] (len 0)
```

Patched with [opensnitch-1-7-2-system-firewall-quirks.patch](opensnitch-1-7-2-system-firewall-quirks.patch),
the IPv4 case prints the same lines,
and the IPv6 cases become:

```text
inet daddr ff02::fb err=<nil>
  meta key=15
  cmp data=[10] (len 1)
  payload offset=24 len=16
  cmp data=[255 2 0 0 0 0 0 0 0 0 0 0 0 0 0 251] (len 16)
inet saddr fe80::1-fe80::ffff err=<nil>
  meta key=15
  cmp data=[10] (len 1)
  payload offset=8 len=16
  range from=[254 128 0 0 0 0 0 0 0 0 0 0 0 0 0 1] to=[254 128 0 0 0 0 0 0 0 0 0 0 0 0 255 255]
ip6 daddr ff02::fb err=<nil>
  payload offset=24 len=16
  cmp data=[255 2 0 0 0 0 0 0 0 0 0 0 0 0 0 251] (len 16)
```

The reference is nft's own compiler,
run in check mode inside a throwaway namespace
(`sudo ip netns exec nftref nft -c --debug=netlink -f ref.nft`)
for `ip6 daddr ff02::fb accept` and `ip6 saddr fe80::1-fe80::ffff accept` in an `inet` table:

```text
  [ meta load nfproto => reg 1 ]
  [ cmp eq reg 1 0x0a ]
  [ payload load 16b @ network header + 24 => reg 1 ]
  [ cmp eq reg 1 0xff020000 0x00000000 0x00000000 0x000000fb ]
  [ meta load nfproto => reg 1 ]
  [ cmp eq reg 1 0x0a ]
  [ payload load 16b @ network header + 8 => reg 1 ]
  [ range eq reg 1 0xfe800000 0x00000000 0x00000000 0x00000001 0xfe800000 0x00000000 0x00000000 0x0000ffff ]
```

The patched expressions match nft's.
`go build ./firewall/nftables/exprs/` passes.
`go vet` on the package fails before reaching the patch,
because the test imports need the generated `daemon/ui/protocol` package,
which is not in the repository.
Upstream's `TestExprIP` needs root and a network namespace;
it was not run.

### Not verified

- A reboot:
  the host has not booted with this configuration or with the owner's unit override.
- Inbound traffic from a real second host;
  only the netns probe.
- Whether a live reload's DNS-rule misplacement repairs itself
  ("A configuration reload puts the system rules ahead of the DNS-capture rule" says it should).
- A daemon-level reproduction of the IPv6 address defect
  (loading an `ip6 daddr` system rule and listing the chain);
  the harness exercises the same constructor without the daemon.
- The patched daemon running,
  or a packet matched by a patched rule.

## Verified workarounds

### Match IPv6 by protocol, port or interface

Never by address.
The inbound policy accepts DHCPv6 replies with `udp sport 547 dport 546`,
mDNS with `udp dport 5353`,
and ICMPv6 with `meta l4proto icmpv6`.
Tradeoff:
the mDNS rule also accepts unicast packets to port 5353,
which ufw's `-d ff02::fb` rule would not,
and SSDP is accepted for IPv4 only (`ip daddr 239.255.255.250`).

### Restart after editing `system-fw.json`

Restart the daemon after editing `system-fw.json`,
and after any other live reload of the system firewall,
then confirm with `sudo nft list chain inet filter input` that `udp sport 53 queue` is first.
The monitor code says waiting one interval should be enough,
but that was not observed,
so this setup restarts.
Set `"FlushConnsOnStart": false` in `/etc/opensnitchd/default-config.json` first,
so a restart does not kill established connections;
SSH sessions stayed connected across the restarts with it.
Tradeoff:
connections open before the daemon starts are never queued for a verdict,
so after a restart,
already-allowed connections stay allowed until they reconnect.

### Update rule files in place

Write a changed rule over its file in place,
for example `sudo tee /etc/opensnitchd/rules/<name>.json < new.json > /dev/null`,
so the watcher sees `Write`;
name the file `<name>.json` after the rule's `name`.
Deleting a file unloads its rule.
Tradeoff:
an in-place write can be read half-written;
the daemon then logs a load error and loads the rule again on the next `Write`.

### Start the daemon before networking

The owner chose a full unit override,
`/etc/systemd/system/opensnitchd.service`,
with `Wants=network-pre.target` and `Before=network-pre.target` instead of `After=network.target`,
`RestartSec=2`,
and `StartLimitIntervalSec=0`,
because restarting every 2 s would otherwise hit systemd's default limit of 5 starts in 10 s.
A drop-in cannot do this:
it cannot remove the packaged `After=network.target`,
and adding `Before=network-pre.target` next to it would be an ordering cycle.
`systemd-analyze verify graphical.target` reported no cycle,
and NetworkManager is now ordered after opensnitchd.
Tradeoffs:
package changes to the unit are not picked up while the override exists
(compare with `diff /usr/lib/systemd/system/opensnitchd.service /etc/systemd/system/opensnitchd.service`);
the unit is `Type=simple`,
so systemd considers it started before the daemon has installed its rules,
which narrows the boot window but does not provably close it;
`systemctl stop opensnitchd` and the restart delay still leave inbound unfiltered;
the override has not been tested at boot.
The maintainer's suggestion in #1183,
a separate nftables table with a drop policy,
would close the stop window,
but adds a second firewall to maintain;
it was not adopted.

### Accept all ICMPv6 for MLD

`meta l4proto icmpv6` accepts every ICMPv6 type.
Tradeoff:
a superset of ufw's list,
redirects included,
with no hop-limit or link-local source checks.

### One rule per port range

Write each range as its own rule,
and keep comma lists to single ports.

### Interface name in `Key`

`{"Name": "iifname", "Values": [{"Key": "virbr*", "Value": ""}]}`.

### At least one ICMP type

Always list at least one type,
using the names `GetICMPType` or `GetICMPv6Type` accept.

### Allow one Python script with a `list` rule

Match the interpreter and the script's command line together:

```jsonc
// /etc/opensnitchd/rules/allow-torbrowser-launcher.json (operator only)
{
  "operand": "list",
  "data": "",
  "type": "list",
  "list": [
    {
      "operand": "process.path",
      "type": "regexp",
      "data": "^/usr/bin/python3(?:\\.[0-9]+)?$",
      "sensitive": true,
      "list": []
    },
    {
      "operand": "process.command",
      "type": "regexp",
      "data": "^/usr/bin/python[0-9.]* /usr/bin/torbrowser-launcher(?: .*)?$",
      "sensitive": true,
      "list": []
    }
  ],
  "sensitive": true
}
```

Tradeoff:
the command line is chosen by whoever starts the process,
so any program that can run the system Python with that script path as its first argument gets the allowance;
the rule narrows which script,
it does not authenticate it.

### Case-sensitive application rules

Use `"sensitive": true`,
with Go's inline `(?i:...)` group where a part must ignore case,
such as `^/tmp/\.mount_(?i:helium)[0-9A-Za-z]+/.*(?i:helium)$` for an AppImage.
Tradeoff:
every other character must be written in the exact case the path uses.

## What does not work

- `ip6 daddr ff02::fb`,
  or `ip daddr` with any IPv6 literal:
  loads silently and never matches,
  as "Address statements are IPv4-only" shows.
- `"Value": "47984-47990,48010"` for a port:
  the range element fails `strconv.Atoi` and is dropped from the set.
- An `icmpv6` statement with no `type`,
  meant to accept all ICMPv6:
  it becomes an empty set lookup.
- `sudo install` or `mv` of a rule file into the rules directory:
  the rename raises no `Write`,
  so the rule stays unloaded until a restart.
- A `process.path` rule for a script's own path:
  the process path is the interpreter.
- `systemctl edit opensnitchd` adding `Before=network-pre.target`:
  the packaged `After=network.target` stays,
  which would form an ordering cycle;
  hence the full override.

## Upstream filing decision

The candidate is the IPv6 address defect:
system-firewall address statements accept IPv6 literals but build IPv4 matches with empty data.
The other behaviours are design or limitations a careful config works around:
the stop behaviour is upstream's design per #1183,
the DNS-rule position repairs itself by design,
and the statement-builder,
ordering and case behaviours are documented here.
Nothing is filed for them.

`.out-of-scope/` (checked 2026-11-18) has no OpenSnitch or nftables entry.

Duplicate and work-in-progress searches on 2026-11-18:
`gh search issues --repo evilsocket/opensnitch` for "ipv6 system firewall",
"ip6 daddr",
"ipv6 daddr" and "system rules ipv6";
`gh issue list --repo evilsocket/opensnitch --state all --search` for "ipv6 system firewall",
"ip6 rule",
"To4" and "ICMPv6".
None describes IPv6 addresses in system rules.
`gh pr list --state all --search "ip.go"` returned #1692,
#1708 and #1423,
none touching `exprs/ip.go`,
and none of the 30 newest open pull requests touches that file.
The file's last commit is from 2023-07-08.

### Is it upstream's fault?

Partly.
The IPv6 path is an explicit `// TODO: ipv6`,
a missing feature;
but silently building a rule that can never match,
instead of returning an error,
is a defect in the existing code.

### Can upstream fix it?

Yes:
the prototype adds 43 lines and removes 5 in one file.

### Is the use case supported?

Partly.
The parser accepts the `ip6` statement name,
and the `inet` table family carries both protocols.
The GUI's firewall dialogs were not checked for an IPv6 address field.

### Would the repository welcome the contribution?

No `CONTRIBUTING.md` exists.
`.github/ISSUE_TEMPLATE/` holds `bug_report.md`,
`feature-request.md` and `config.yml`,
which sends questions to Discussions.
`gh search code --repo evilsocket/opensnitch` for "vouch" and for AI-assistance wording returned nothing.
No ban was found.

### Will they likely fix it?

No signal against:
the TODO says the gap is known,
and in #1225 the maintainer reproduced a system-firewall bug and proposed a fix.

### Is there a prototype?

Yes:
[opensnitch-1-7-2-system-firewall-quirks.patch](opensnitch-1-7-2-system-firewall-quirks.patch),
against `master` `a1353848`,
verified by the harness and the nft comparison in "Verification".
It picks `NFPROTO_IPV6`,
the IPv6 header offsets and 16-byte data when an address is IPv6 or the table family is `ip6`,
and leaves IPv4 output unchanged.

### Readiness check

Run on 2026-11-18 against `master` `a1353848`:

- Latest release and other maintained lines:
  source compared only;
  `exprs/ip.go` is identical in v1.7.2,
  v1.8.0 and `master`.
  No 1.8.0 binary was run.
- Prior art:
  the commits on `exprs/ip.go` were listed;
  their pull request discussions were not read.
- Intake:
  the bug template asks for version,
  OS,
  window manager,
  kernel and the last log lines;
  the draft fills them from this host.
- Automation on new issues:
  `.github/workflows/` holds `build_ebpf_modules.yml`,
  `generic_validations.yml` and `go.yml`;
  their triggers were not read.
- Day-of searches:
  run once,
  as recorded in "Upstream filing decision";
  repeat them on the day of any filing.
- The draft's reproduction as written:
  not run against the daemon.
- A public reproduction repository with CI:
  not created;
  creating one needs the owner's permission.

Verdict:
not fileable as-is.
Two blockers stop a recommendation:
the daemon-level reproduction was not run,
because this session was limited to documentation,
and a reproduction repository needs the owner's go-ahead.
Filing puts the poster's GitHub account in front of the maintainers,
and some projects run trust lists that close or flag low-effort,
agent-written reports.
None was found here,
but that risk stays with the poster,
and whether to file is the poster's decision.

Draft (do not file as-is):

~~~md
Title: [Bug Report] System firewall: ip/ip6 saddr/daddr with an IPv6 address builds an IPv4 match with empty data

Labels: bug

### Describe the bug:

A system-firewall rule whose `ip` or `ip6` statement holds an IPv6 address loads without a warning,
but the nftables rule it builds compares `nfproto` with IPv4, loads 4 bytes at the IPv4 header offset,
and compares them with zero bytes of data, so it can never match IPv6 traffic.

`daemon/firewall/nftables/exprs/ip.go` (identical in v1.7.2, v1.8.0 and master a1353848):
- `NewExprIP` adds `meta nfproto == NFPROTO_IPV4` for inet tables and has `// TODO: ipv6` (L34-44);
- `getExprIPPayload` uses offsets 16 (daddr) and 12 (saddr) with length 4 (L81-111);
- `getExprIP` and `getExprRangeIP` use `ip.To4()`, which is nil for IPv6 (L114-127, L131-149).

Include the following information:
 - OpenSnitch version: 1.7.2 (CachyOS package 1.7.2-4.1); source unchanged on master
 - OS: CachyOS (Arch Linux based)
 - OS version: rolling, 2026-11-18
 - Window Manager: KDE Plasma (Wayland)
 - Kernel version: 7.3.4-1-cachyos

### To Reproduce:

1. Add a rule to the `input` chain in `/etc/opensnitchd/system-fw.json`:
   `{"Name": "ip6", "Values": [{"Key": "daddr", "Value": "ff02::fb"}]}` with target `accept`.
2. Restart opensnitchd and run `sudo nft --debug=netlink list chain inet filter input`.
3. See the rule's expressions: `nfproto` compared with 0x02, payload load 4b @ network header + 16,
   and an empty comparison, instead of 0x0a, 16b @ + 24 and the 16-byte address.

Without the daemon, this program shows the same expressions from `NewExprIP` (run from `daemon/`):
(harness from this doc)

### Post error logs:

No error or warning is logged; that is part of the problem.

### Suggested fix

Pick NFPROTO_IPV6, the IPv6 header offsets (saddr 8, daddr 24, length 16) and 16-byte data when the
address is IPv6 or the table family is ip6. A prototype patch (one file, +43/-5) is attached; its output
matches `nft -c --debug=netlink` for `ip6 daddr ff02::fb` and `ip6 saddr fe80::1-fe80::ffff`.
Upstream's `TestExprIP` would need IPv6 cases; it was not run (needs root).
~~~

Fields only the poster can confirm:
the OS and window manager lines describe this host,
and the poster should check them before posting.

[parser-ip]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/parser.go#L98-L104
[parser-iface]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/parser.go#L89-L96
[ip-new]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/exprs/ip.go#L34-L44
[ip-payload]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/exprs/ip.go#L91-L99
[ip-get]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/exprs/ip.go#L114-L127
[ip-get-master]: https://github.com/evilsocket/opensnitch/blob/a1353848ba1b660320e90cefea782c3fba272c00/daemon/firewall/nftables/exprs/ip.go#L114-L127
[monitor-conf]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/monitor.go#L53-L58
[monitor-check]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/monitor.go#L29-L34
[monitor-rules]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/monitor.go#L60-L66
[rules-dns]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/rules.go#L35-L37
[rules-insert]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/rules.go#L182-L201
[nft-stop]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/nftables.go#L102-L111
[nft-clean]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/nftables.go#L144-L148
[system-delete]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/system.go#L139-L153
[system-loop]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/system.go#L121-L128
[system-insert]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/system.go#L174
[chains-backup]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/chains.go#L167-L186
[icmp-split]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/rule_helpers.go#L40-L44
[icmp-set]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/rule_helpers.go#L80-L99
[port-set]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/exprs/port.go#L50-L73
[iface-wild]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/firewall/nftables/exprs/iface.go#L24-L33
[operator-compile]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/rule/operator.go#L144-L152
[operator-recmp]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/rule/operator.go#L231-L240
[loader-match]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/rule/loader.go#L496-L515
[loader-oplist]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/rule/loader.go#L356-L366
[loader-live]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/rule/loader.go#L472-L486
[operator-list]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/rule/operator.go#L327-L333
[operator-cmd]: https://github.com/evilsocket/opensnitch/blob/8c3fa40cf476d1a68df04d49bb98509437a47b4f/daemon/rule/operator.go#L352-L353
[issue-1183]: https://github.com/evilsocket/opensnitch/issues/1183
[issue-1183-running]: https://github.com/evilsocket/opensnitch/issues/1183#:~:text=For%20now%20the%20service%20must%20be%20running%20to%20block%20outbound%20connections
[issue-1183-table]: https://github.com/evilsocket/opensnitch/issues/1183#issuecomment-2415318670
[issue-1183-closed]: https://github.com/evilsocket/opensnitch/issues/1183#issuecomment-3644448269
[issue-1225]: https://github.com/evilsocket/opensnitch/issues/1225
[doc-auto-reject]: opensnitch-1-8-auto-reject-blocks-toolchain-updates.md
[doc-kernel-modules]: kernel-upgrade-without-reboot-removes-running-kernel-modules.md
