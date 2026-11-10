# Removing an `ip rule` leaves an established connection on its old path

An earlier wg-quicker report said that replacing the route watcher,
or taking the tunnel down and up,
resets an open connection to an address whose clearnet rule the watcher owns.
The first half of that is not what this kernel does,
and the difference decides why a host-list reload needs no restart at all.

## What was measured

Measured on 2026-11-10 on kernel `7.2.0-ogc6.1.fc44.x86_64` in disposable user and network namespaces,
with two veth pairs between a client namespace and a server namespace:

1. The server owned `10.9.9.9/32` on its loopback and answered TCP on port 8080.
2. The client reached that one destination through the first pair or the second,
   chosen by policy rules:
   `to 10.9.9.9 lookup 100 pref 100` (source `10.7.0.1`)
   and `to 10.9.9.9 lookup 200 pref 200` (source `10.8.0.1`).
3. The client connected from `10.7.0.1:45871`;
   the server reported exactly one connection,
   from `10.7.0.1:45871`.
4. With the connection established,
   the rule at preference 100 was deleted,
   so `ip route get 10.9.9.9` answered `dev path2 table 200 src 10.8.0.1`.
5. The client wrote again on the same socket.

Result:
the connection stayed open,
the server reported no error,
no reset and no second connection,
and the client's second write was accepted.

The routing decision for a *new* connection had changed to source `10.8.0.1`,
and the established socket did not follow it.

## Why

Linux caches the route of a connected socket,
including the source address picked when it connected,
so a policy-rule change does not re-route an established flow.
The socket keeps its cached destination entry,
and the rule change only affects lookups made afterwards.

What does end those connections is the interface they are bound to:
`ip link delete` of the WireGuard interface drops every connection through the tunnel,
which is what `wg-quicker down` does,
and what "take the tunnel down and bring it up" therefore costs.
A rule that is missing also sends *new* connections into the tunnel,
whose source address is the tunnel's,
which is a policy failure and not a reset of the connection that was open.

## What this means for wg-quicker

- Reloading the clearnet host lists inside the running route watcher cannot reset a connection,
  on either count:
  no interface is recreated,
  and a rule that both the old and the new list name is never removed.
- The reset that a tunnel restart causes is the interface deletion,
  so avoiding a restart avoids it,
  which is why the watcher reads the installed host list instead of being replaced.
- A rule that a *new* list no longer names is removed after two consecutive refreshes without it,
  and new connections to that address then use the tunnel by design,
  because the owner's policy of keeping those addresses outside the tunnel no longer names it.

## Reproducing it

No root,
no real tunnel and no change to any live interface is needed:
`unshare --user --map-root-user --net` gives a fresh network namespace with `CAP_NET_ADMIN`,
the two inner namespaces are made with `unshare --net` holders,
and their veth ends are moved in with `ip link set <name> netns <pid>`.
`ip netns add` cannot be used there,
because it writes under `/run`,
which a user namespace cannot own.

The measured run used `node` in both inner namespaces:
a server that reports each connection and close,
and a client that connects,
is made to lose its rule,
and writes again.
