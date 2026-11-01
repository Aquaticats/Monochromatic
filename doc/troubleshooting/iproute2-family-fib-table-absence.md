# iproute2 6.17.0 family-specific table query exits 2 when Linux 7.1.3 has not instantiated that FIB

## Symptom

On `iproute2` 6.17.0 with Linux `7.1.3-ogc5.1.fc44.x86_64`,
a routing table number can contain IPv4 routes while the same IPv6 query fails:

```console
$ sudo ip -n wgq-fib -4 route show table 52000
blackhole 192.0.2.0/24
$ sudo ip -n wgq-fib -6 route show table 52000
Error: ipv6: FIB table does not exist.
Dump terminated
$ echo $?
2
```

The inverse also applies.
IPv4 and IPv6 FIB tables have separate kernel existence,
even when userspace addresses both with the same numeric table ID.

This matters to allocation probes.
An application checking both families cannot treat every nonzero `ip route show` result as fatal,
but it also cannot suppress unrelated failures.

## Root cause

The source traces use:

- `iproute2` tag `v6.17.0`,
   commit `d2a8ffe85ac64f91ee2aa7679a5046f321b54127`;
- Linux tag `v7.1`,
   commit `8cd9520d35a6c38db6567e97dd93b1f11f185dc6`.

`ip route show table 52000` adds `RTA_TABLE` to a strict route dump request.
`ip/iproute.c:1717-1730` contains:

```c
static int iproute_dump_filter(struct nlmsghdr *nlh, int reqlen)
{
	struct rtmsg *rtm = NLMSG_DATA(nlh);
	int err;

	rtm->rtm_protocol = filter.protocol;
	if (filter.cloned)
		rtm->rtm_flags |= RTM_F_CLONED;

	if (filter.tb) {
		err = addattr32(nlh, reqlen, RTA_TABLE, filter.tb);
```

`lib/libnetlink.c:354-374` sends that request for the selected address family as `RTM_GETROUTE` with
`NLM_F_DUMP | NLM_F_REQUEST`:

```c
int rtnl_routedump_req(struct rtnl_handle *rth, int family,
		       req_filter_fn_t filter_fn)
{
	struct {
		struct nlmsghdr nlh;
		struct rtmsg rtm;
		char buf[128];
	} req = {
		.nlh.nlmsg_len = NLMSG_LENGTH(sizeof(struct rtmsg)),
		.nlh.nlmsg_type = RTM_GETROUTE,
		.nlh.nlmsg_flags = NLM_F_DUMP | NLM_F_REQUEST,
		.nlh.nlmsg_seq = rth->dump = ++rth->seq,
		.rtm.rtm_family = family,
	};
```

Linux keeps IPv6 tables in the IPv6 FIB hash.
`net/ipv6/ip6_fib.c:282-299` searches that hash and returns `NULL` when no IPv6 table has that ID:

```c
struct fib6_table *fib6_get_table(struct net *net, u32 id)
{
	struct hlist_head *head;
	struct fib6_table *tb;

	if (!id)
		id = RT6_TABLE_MAIN;

	head = &net->ipv6.fib_table_hash[id & (FIB6_TABLE_HASHSZ - 1)];

	hlist_for_each_entry_rcu(tb, head, tb6_hlist, true)
		if (tb->tb6_id == id)
			return tb;

	return NULL;
}
```

A strict IPv6 dump of that missing table deliberately returns `ENOENT` with the exact extended acknowledgement.
`net/ipv6/ip6_fib.c:685-694` contains:

```c
if (arg.filter.table_id) {
	tb = fib6_get_table(net, arg.filter.table_id);
	if (!tb) {
		if (rtnl_msg_family(cb->nlh) != PF_INET6)
			goto unlock;

		NL_SET_ERR_MSG_MOD(cb->extack, "FIB table does not exist");
		err = -ENOENT;
		goto unlock;
	}
```

`iproute2` prints the kernel extended acknowledgement,
then maps `ENOENT` to a failed dump.
`lib/libnetlink.c:731-758` contains:

```c
if (len < 0) {
	errno = -len;

	if (a->errhndlr && (a->errhndlr(h, a->arg2) & RTNL_SUPPRESS_NLMSG_DONE_NLERR))
		return 0;

	if (nl_dump_ext_ack_done(h, sizeof(int), len))
		return len;

	switch (errno) {
	case ENOENT:
	case EOPNOTSUPP:
		return -1;
```

The command path turns the failed dump into `Dump terminated` and return value `-2` at
`ip/iproute.c:2023-2036`.
The `ip` executable exposes that as shell exit status `2`:

```c
if (rtnl_dump_filter_errhndlr(&rth, filter_fn, stdout,
				      save_route_errhndlr, NULL) < 0) {
	fprintf(stderr, "Dump terminated\n");
	delete_json_obj();
	return -2;
}
```

An earlier reading that one numeric table is created for both families was wrong.
The separate IPv6 hash lookup and the reproduced IPv4-present/IPv6-absent result disprove it.
This is not evidence that route listing itself failed.
It is a precise report that the requested family has no table object.

## Verification

### Versions

```console
$ ip -Version
ip utility, iproute2-6.17.0, libbpf 1.6.3
$ uname --kernel-release
7.1.3-ogc5.1.fc44.x86_64
```

### Runnable harness

Use a disposable network namespace:

```console
$ sudo ip netns add wgq-fib
$ sudo ip -n wgq-fib link set lo up
$ sudo ip -n wgq-fib -4 route add blackhole 192.0.2.0/24 table 52000
$ sudo ip -n wgq-fib -4 route show table 52000
blackhole 192.0.2.0/24
$ sudo ip -n wgq-fib -6 route show table 52000
Error: ipv6: FIB table does not exist.
Dump terminated
$ sudo ip -n wgq-fib -6 route add unreachable default table 52000
$ sudo ip -n wgq-fib -6 route show table 52000
unreachable default dev lo metric 1024 pref medium
$ sudo ip netns delete wgq-fib
```

### Patterns that work cleanly

- Populated IPv4 table queried with `ip -4 route show table 52000`:
   exit `0`,
   route printed.
- Populated IPv6 table queried with `ip -6 route show table 52000`:
   exit `0`,
   route printed.
- Built-in family table queried for its own family:
   exit `0`,
   possibly empty output.

### Patterns that fail with family-specific absence

- Missing IPv4 table queried with `ip -4 route show table 52001`:
   exit `2`,
  `Error: ipv4: FIB table does not exist.` followed by `Dump terminated`.
- Missing IPv6 table queried with `ip -6 route show table 52001`:
   exit `2`,
  `Error: ipv6: FIB table does not exist.` followed by `Dump terminated`.
- Table populated only in the other family:
   same family-specific exit `2` and diagnostic.

## Verified workarounds

### Translate only the exact occupancy-probe response

`package/wg/cli-quicker/src/tunnel-table-diagnostic.ts:9-12` records both exact family diagnostics.
`package/wg/cli-quicker/src/tunnel-table-diagnostic.ts:34-46` treats a result as absence only when:

- exit status is exactly `2`;
- stderr after terminal whitespace removal equals the selected family's two-line diagnostic.

Every other nonzero result remains a `CommandError`.
The tradeoff is version and wording coupling.
A future `iproute2` diagnostic change fails closed and requires an intentional update.

### Instantiate a fail-closed route in the missing family

Adding `unreachable default` creates the family table and makes subsequent listing succeed.
`wg-quicker` uses this after claiming a table so exempt traffic cannot fall through to VPN routing
when one physical family is absent.
The tradeoff is semantic:
the family no longer has an absent table,
it has an explicit unreachable default owned by `wg-quicker`.

## What does not work

- Assuming one family's route creates the other family's table.
  The kernel uses separate family FIBs.
- Treating every nonzero result as table occupancy.
  A free candidate commonly produces exit `2` in each uninstantiated family.
- Matching only the substring `FIB table does not exist`.
  That can hide additional stderr or an unexpected exit status,
  converting a real command failure into a false-free allocation decision.
- Globally suppressing `ENOENT` in `iproute2`.
  Strict dump checking intentionally distinguishes an absent table from an existing empty table.

## Upstream filing artifact

### Upstream filing decision

1. **Is it really upstream's fault?**
    No.
   Linux accurately reports that the requested family-specific table is absent,
   and `iproute2` intentionally exposes strict dump errors.
   The 2019 [netdev thread][netdev-thread] records the maintainer rationale.
2. **Can upstream fix it?**
    Technically yes.
   `iproute2` already has a narrow suppression hook at `ip/iproute.c:1806-1816`,
   but changing custom-table behavior would erase the absent-versus-empty distinction.
3. **Are they supporting this use case?**
    Yes.
   Table-filtered route listing is implemented by the cited `RTA_TABLE` request path.
4. **Would the repository welcome our contribution?**
    Generally yes.
   `README.devel` directs patches to `netdev@vger.kernel.org` and names kernel-style submission rules.
   The repository has no GitHub issue templates and no AI-contribution policy was found in `README` or `README.devel`.
5. **Will they likely fix it?**
    No for custom tables.
   The [maintainer response][maintainer-response] describes strict absence as useful information.
   The later main-table save exception remained narrow,
   as shown by [the accepted patch discussion][main-table-patch] and current `save_route_errhndlr` source.
6. **Have we prototyped a minimal compatible fix?**
    No.
   Constraints 1 and 5 fail,
   so changing upstream behavior would oppose its stated semantics and the auto-prototype gate does not apply.

No `.out-of-scope/` entry matches `iproute2`,
 Linux FIBs,
 or this diagnostic.
Searches of open and closed GitHub issues and pull requests in `iproute2/iproute2` found no GitHub duplicate.
The behavior is already covered by the [2019 netdev thread][netdev-thread] and
[Debian bug 991016][debian-bug].
This investigation adds no upstream fact absent from those discussions,
so there is nothing additive to post and no new issue draft.

[debian-bug]: https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=991016
[main-table-patch]: https://lists.openwall.net/netdev/2021/07/07/68
[maintainer-response]: https://www.spinics.net/lists/netdev/msg559925.html
[netdev-thread]: https://www.spinics.net/lists/netdev/msg559739.html
