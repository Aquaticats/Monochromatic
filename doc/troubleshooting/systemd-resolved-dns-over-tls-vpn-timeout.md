# systemd-resolved 259.7 strict DNS-over-TLS delays nslookup 9.18.50 past its UDP timeout

## Symptom

On Bazzite 44 with `systemd-resolved` 259.7 and BIND `nslookup` 9.18.50,
a lookup can print this warning before eventually returning an answer:

```text
;; communications error to 127.0.0.53#53: timed out
```

`127.0.0.53` is the local `systemd-resolved` full stub.
It is not GitHub's DNS server.
`/etc/resolv.conf` is a symlink to
`/run/systemd/resolve/stub-resolv.conf`,
 which contains:

```text
nameserver 127.0.0.53
options edns0 trust-ad
search lan
```

The observed command did not fail permanently.
Its first attempt timed out,
 `nslookup` retried,
 and the same process returned:

```text
Server:         127.0.0.53
Address:        127.0.0.53#53

Non-authoritative answer:
www.github.com  canonical name = github.com.
Name:   github.com
Address: 140.82.114.3
```

Recurrence is expected if resolver state returns to the first VPN DNS server.
This run observed the switch away from that server,
 but did not demonstrate a complete cycle back to it.
The VPN link had these resolver properties before the failing lookup:

```text
Protocols: +DefaultRoute ... +DNSOverTLS
Current DNS Server: 198.245.51.147
DNS Servers: 198.245.51.147 1.1.1.1
DNS Domain: ~.
```

After the delayed lookup,
 `Current DNS Server` was `1.1.1.1`.

## Root cause

Strict DNS-over-TLS is effective on a VPN link whose first DNS server did not complete a TCP port 853
connection from this host within the probe window.
`systemd-resolved` waited on that path,
 switched to the second server,
 and eventually answered.
BIND's shorter UDP wait for the local stub expired first,
 so `nslookup` reported one timed-out attempt and retried.

The evidence identifies a failed TCP port 853 path,
 not why that path drops traffic.
The DNS server might lack DNS-over-TLS support,
 or a firewall or network segment might filter that port.

### Step 1: nslookup sends the query to the local stub

The generated `/etc/resolv.conf` names `127.0.0.53`,
 so `nslookup` sends an ordinary UDP DNS packet to
`systemd-resolved` on loopback.
Both UDP and TCP listeners were present on `127.0.0.53:53`.
The loopback interface was up.
This rules out an absent stub listener and a down loopback interface.

BIND 9.18.50 defines its default UDP timeout as five seconds in `bin/dig/dighost.h:68-73`:

```c
/*% Default TCP Timeout */
#define TCP_TIMEOUT 10
/*% Default UDP Timeout */
#define UDP_TIMEOUT 5

#define SERVER_TIMEOUT 1
```

When the receive operation times out,
 `bin/dig/dighost.c:4043-4068` emits the exact diagnostic and queues another
UDP request while retries remain:

```c
if (eresult != ISC_R_SUCCESS) {
        char sockstr[ISC_SOCKADDR_FORMATSIZE];

        isc_sockaddr_format(&query->sockaddr, sockstr, sizeof(sockstr));
        dighost_warning("communications error to %s: %s", sockstr,
                        isc_result_totext(eresult));

        if (l->retries > 1 && !l->tcp_mode) {
                dig_query_t *newq = NULL;

                /*
                 * For UDP, insert a copy of the current query just
                 * after itself in the list, and start it to retry the
                 * request.
                 */
                newq = new_query(l, query->servname, query->userarg);
                ISC_LIST_INSERTAFTER(l->q, query, newq, link);
                if (l->current_query == query) {
                        query_detach(&l->current_query);
                }
                if (l->current_query == NULL) {
                        l->retries--;
                        debug("making new UDP request, %d tries left",
                              l->retries);
                        start_udp(newq);
                }
```

This explains both parts of the surface behavior:
 the warning is printed for the expired attempt,
 but the process
continues because retries remain.

### Step 2: strict DNS-over-TLS is effective on the VPN link

`/etc/systemd/resolved.conf.d/cloudflare-gateway.conf` contains a global
`DNSOverTLS=yes` setting and a global `~.` route-only domain.
The configured Cloudflare Gateway hostname is omitted from this document because it identifies the Gateway location.

The live effective mode was verified directly:

```text
$ resolvectl dnsovertls mx-que-mx1
Link 43 (mx-que-mx1): yes
```

No explicit per-link mode was found in the link's current-boot resolver configuration.
The current-boot journal recorded bus updates for the link's DNS servers,
 route domain,
 and default-route flag,
 but no DNS-over-TLS setter call.
This is consistent with global inheritance,
 though the effective `yes` value alone cannot distinguish inheritance from an explicit identical setting.

In systemd 259.7,
 `src/resolve/resolved-link.c:819-825` returns the manager's global mode when the link mode is unset:

```c
DnsOverTlsMode link_get_dns_over_tls_mode(Link *l) {
        assert(l);

        if (l->dns_over_tls_mode != _DNS_OVER_TLS_MODE_INVALID)
                return l->dns_over_tls_mode;

        return manager_get_dns_over_tls_mode(l->manager);
}
```

`src/resolve/resolved-dns-scope.c:83-96` copies that effective mode into the link's DNS scope:

```c
if (protocol == DNS_PROTOCOL_DNS) {
        /* Copy DNSSEC mode from the link if it is set there,
         * otherwise take the manager's DNSSEC mode. Note that
         * we copy this only at scope creation time, and do
         * not update it from the on, even if the setting
         * changes. */

        if (link) {
                s->dnssec_mode = link_get_dnssec_mode(link);
                s->dns_over_tls_mode = link_get_dns_over_tls_mode(link);
        } else {
                s->dnssec_mode = manager_get_dnssec_mode(m);
                s->dns_over_tls_mode = manager_get_dns_over_tls_mode(m);
        }
}
```

The matching manual source states the same rule at `man/resolved.conf.xml:268-272`:

```xml
<para>In addition to this global <varname>DNSOverTLS=</varname> setting
<citerefentry><refentrytitle>systemd-networkd.service</refentrytitle><manvolnum>8</manvolnum></citerefentry>
also maintains per-link <varname>DNSOverTLS=</varname> settings. For system DNS servers (see above), only the global
<varname>DNSOverTLS=</varname> setting is in effect. For per-link DNS servers the per-link setting is in effect, unless
it is unset in which case the global setting is used instead.</para>
```

The manual warns at `man/resolved.conf.xml:241-249` that strict mode requires every selected server to support
DNS-over-TLS:

```xml
<para>Takes a boolean argument or <literal>opportunistic</literal>. If
true all connections to the server will be encrypted. Note that this
mode requires a DNS server that supports DNS-over-TLS and has a valid
certificate.
...
If the DNS server does not support DNS-over-TLS all DNS requests will fail.</para>
```

### Step 3: systemd-resolved tries the VPN DNS server on TCP port 853

The global settings and the VPN link both had the `~.` route-only domain.
For equally specific routing-domain matches,
 `man/systemd-resolved.service.xml:207-212` says the resolver sends the
query to all associated scopes in parallel:

```xml
<listitem><para>If a name to look up matches (that is: is equal to or has as suffix) any of the
configured routing domains (search or route-only) of any link, or the globally configured DNS settings,
"best matching" routing domain is determined: the matching one with the most labels. The query is then
sent to all DNS servers of any links or the globally configured DNS servers associated with this "best
matching" routing domain. (Note that more than one link might have this same "best matching" routing
domain configured, in which case the query is sent to all of them in parallel).</para>
```

The parallel global path is inferred from that documented rule;
packet-level tracing of its timing was not captured in this run.
`resolvectl monitor` did identify interface index 43,
 the VPN link,
 on the eventual answer.
The VPN scope initially selected `198.245.51.147`.
For a TLS feature level,
 `src/resolve/resolved-dns-transaction.c:686-692` selects port 853:

```c
if (t->server->port > 0)
        return t->server->port;

return DNS_SERVER_FEATURE_LEVEL_IS_TLS(t->current_feature_level) ? 853 : 53;
```

A direct TCP connection probe produced:

```text
198.245.51.147: TimeoutError: timed out; elapsed=3.041s
1.1.1.1: connected; elapsed=0.009s
172.64.36.1: connected; elapsed=0.008s
```

Plain DNS to the same VPN primary worked:

```text
;; Query time: 24 msec
;; SERVER: 198.245.51.147#53(198.245.51.147) (UDP)
```

The measured failure is therefore specific to the TCP port 853 path used by strict DNS-over-TLS.
It is not general reachability loss to that server because UDP port 53 remained reachable.
The probe does not distinguish absent DNS-over-TLS support from path filtering.

### Step 4: systemd-resolved switches servers after the failed attempt

The resolver intentionally retains one current server until an error,
 then rotates.
`man/systemd-resolved.service.xml:320-331` documents that behavior:

```xml
<listitem><para>The <filename>nss-dns</filename> resolver maintains little state between subsequent DNS
queries, and for each query always talks to the first listed DNS server from
<filename>/etc/resolv.conf</filename> first, and on failure continues with the next until reaching the
end of the list which is when the query fails. The resolver in <command>systemd-resolved</command>
however maintains state, and will continuously talk to the same server for all queries in a particular
lookup scope until some form of error is seen at which point it will switch to the next server, and
then stay with it for all queries on the scope until the next failure, and so on, eventually returning
to the first configured server.</para></listitem>
```

The source performs that retry and rotation in `src/resolve/resolved-dns-transaction.c:525-535`:

```c
if (next_server && t->scope->protocol == DNS_PROTOCOL_DNS)
        log_debug("Retrying transaction %" PRIu16 ", after switching servers.", t->id);
else
        log_debug("Retrying transaction %" PRIu16 ".", t->id);

/* Before we try again, switch to a new server. */
if (next_server)
        dns_scope_next_dns_server(t->scope, t->server);

r = dns_transaction_go(t);
```

The observed transition matched this call chain:

- Before the delayed query,
   the VPN scope's current server was `198.245.51.147`.
- The first uncached positive query through that scope took 5.34 seconds.
- After the query,
   the current server was `1.1.1.1`.
- The next two uncached positive queries through the scope took 0.06 seconds each.

The `nslookup` client gives its loopback UDP request five seconds.
The first resolver transaction exceeded that boundary,
 so the client warned.
Its retry then received the answer produced after `systemd-resolved` had switched servers.

## Verification

### Source trace and versions

The live system reported:

```text
Bazzite 44.20260721.0 (Kinoite)
bind-utils-9.18.50-1.fc44.x86_64
systemd-259.7-1.fc44.x86_64
systemd-resolved-259.7-1.fc44.x86_64
```

Source inspection used read-only clones in the private agent scratch directory.
No third-party clone was modified.
The clones were checked out at these exact release tags:

```text
isc-projects/bind9 v9.18.50
commit ad1a84a99f853f5194ec0afc52a152f2b9579074

systemd/systemd v259.7
commit a8b15d5d21271c63deea160c15b958c23a267022
```

### Runnable harness

The warning is state-dependent.
It appears when `Current DNS Server` on the VPN link is the first entry and strict DNS-over-TLS is active.
The following commands capture the precondition,
 trigger,
 and server transition:

```bash
VPN_LINK=mx-que-mx1
resolvectl status "${VPN_LINK}"
/usr/bin/time --format='elapsed=%e exit=%x' nslookup www.github.com
resolvectl status "${VPN_LINK}"
resolvectl statistics
```

The deterministic lower-level probe checks the mismatched transport directly:

```bash
python3 - <<'PY'
import socket
import time

for server in ('198.245.51.147', '1.1.1.1', '172.64.36.1'):
    started = time.monotonic()
    try:
        with socket.create_connection((server, 853), timeout=3):
            outcome = 'connected'
    except OSError as error:
        outcome = f'{type(error).__name__}: {error}'
    print(f'{server}: {outcome}; elapsed={time.monotonic() - started:.3f}s')
PY
```

### Patterns that work cleanly

- `nslookup www.github.com 1.1.1.1` bypassed the local stub and completed in 0.03 seconds.
- Plain UDP DNS to `198.245.51.147:53` completed in 0.03 seconds or less in repeated probes.
- TCP port 853 connected to `1.1.1.1` and to the configured Cloudflare Gateway address.
- Subsequent uncached positive lookups through the VPN scope completed in 0.06 seconds after
  `systemd-resolved` selected `1.1.1.1`.
- The loopback DNS listeners on `127.0.0.53:53` and `127.0.0.54:53` accepted queries.

### Patterns that expose the delay or failure

- The first `nslookup www.github.com` with the VPN primary selected printed the exact communications warning,
  then succeeded on retry.
- The first uncached positive `resolvectl` query through the VPN scope took 5.34 seconds.
- A TCP connection to `198.245.51.147:853` did not complete within the three-second probe window.
- An uncached negative query can wait for every parallel scope,
   and two such probes took 8.81 and 8.84 seconds.
  This is expected from the documented rule that a positive response may win immediately,
   while all negative paths must
  finish before the final negative response is known.

## Verified workarounds

### Give nslookup enough time to receive the resolver's eventual answer

```bash
nslookup -timeout=10 www.github.com 127.0.0.53
```

This completed without the communications warning in 9.11 seconds.

Tradeoff:
it masks the DNS-over-TLS mismatch and preserves the wait whenever the bad server is selected.
It does not repair resolver policy.

### Bypass the local stub for a one-off public lookup

```bash
nslookup www.github.com 1.1.1.1
```

This completed in 0.03 seconds.

Tradeoff:
it bypasses `systemd-resolved`,
 split DNS,
 the configured Cloudflare Gateway policy,
 and any VPN DNS-leak controls.
It is suitable as a diagnostic or emergency one-off command,
 not a system-wide fix.

## Durable remediation choices not applied

These choices change resolver or VPN policy,
 so the diagnosis did not apply them to the live system.
They require selecting the intended security boundary first.

### Set the VPN link's DNS-over-TLS mode explicitly

A per-link `DNSOverTLS=no` setting would stop that link from inheriting strict global mode.
DNS packets would remain inside the encrypted VPN tunnel,
 but would use ordinary DNS within that tunnel.

- Pro:
   narrow change that leaves strict DNS-over-TLS enabled for the global Cloudflare servers.
- Con:
   the VPN link creator must persist the setting;
  a one-time runtime setting can disappear when the link is recreated.
- Con:
   DNS on the VPN link is no longer separately encrypted with TLS.

### Replace or remove the VPN DNS server with the failing port 853 path

Configure the VPN integration so every listed per-link server has a working strict DNS-over-TLS path,
 or omit the
`198.245.51.147` entry.

- Pro:
   preserves strict DNS-over-TLS without fallback.
- Con:
   the VPN client may own and republish the list.
- Con:
   replacing the provider DNS server can change leak protection,
   filtering,
   and private-zone behavior.

### Make global DNS-over-TLS opportunistic

Change the global policy from `DNSOverTLS=yes` to `DNSOverTLS=opportunistic`.
The resolver can then downgrade when a per-link server's TLS path fails.

- Pro:
   automatically accommodates mixed server capabilities.
- Con:
   weakens the declared strict-encryption policy and permits unauthenticated downgrade.
- Con:
   it does not resolve the competing global and VPN `~.` routing policies.

### Remove the competing VPN DNS route

If the intended policy is truly to send every query to the configured Cloudflare Gateway,
 configure the VPN not to
publish its own `~.` DNS route and server list.
The kernel currently routes the Cloudflare DNS addresses through the VPN interface,
 so this can retain VPN transport,
but the VPN product's own policy must be checked before changing it.

- Pro:
   restores one authoritative all-domain DNS policy.
- Con:
   can break VPN private zones or provider DNS-leak controls.
- Con:
   the VPN client can restore the route during reconnection.

Ranking for the narrow goal of removing the delay while preserving strict global Cloudflare DNS-over-TLS:
explicit per-link mode > compatible VPN DNS list > removal of the VPN DNS route > opportunistic global mode.
The first choice changes only the affected link.
A compatible list preserves stronger transport but depends more heavily on VPN support.
Route removal changes DNS ownership.
Opportunistic mode weakens the broadest policy.

If the real goal is instead to force every query through Cloudflare Gateway filtering,
 route removal ranks first.
That different ranking depends on a security preference not established by this diagnosis.

## What does not work

- Blaming GitHub does not fit the evidence.
  Direct queries returned GitHub addresses,
   and the timeout names the local loopback stub.
- Treating the warning as a final command failure is incorrect when the same process later prints an answer.
  BIND's source prints the warning per failed attempt and then retries.
- Restarting `systemd-resolved` was not tested.
  It would not change the configured strict mode or server list,
  so it is not a demonstrated repair for this configuration.
- Editing `/etc/resolv.conf` is not a durable fix on this host.
  It is a managed symlink and does not express per-link routing or DNS-over-TLS policy.
- BIND issue 4044,
   "nslookup reports timeout if input lookup is delayed",
   is not this incident.
  That issue concerned delayed interactive standard input.
  BIND 9.18.50 contains its regression test at `bin/tests/system/nslookup/tests.sh:49-65`,
   and this incident used a
  noninteractive one-command invocation with a genuinely delayed DNS response.
- A successful cached `getent` or second `nslookup` does not disprove the first-attempt delay.
  Those calls run after the resolver has cached data or switched to `1.1.1.1`.

## Upstream filing artifact

### Upstream filing decision

No matching exemption exists in `.out-of-scope/`.
Searches of open and closed systemd issues and pull requests for combinations of `DNSOverTLS`,
 `VPN`,
 `per-link`,
`global`,
 and `timeout` found no duplicate.
The BIND delayed-input issue 4044 is related only by wording and does not match the signal or input path.

The six filing constraints are:

1.  **Is it really upstream's fault?**
     No.
    systemd's documented inheritance rule is working as implemented.
    Strict mode explicitly requires selected DNS servers to support DNS-over-TLS.
    The local configuration combines that strict global default with a per-link server whose port 853 path timed out.
2.  **Can upstream fix it?**
     The necessary control already exists as a per-link DNS-over-TLS setting.
    Upstream cannot infer whether the user wants strict global policy,
     VPN-provider DNS,
     or downgrade.
3.  **Are they supporting this use case?**
     Yes,
     when link owners set the per-link mode or all selected servers support
    the inherited global mode.
    The manual documents both requirements.
4.  **Would the repo welcome our contribution?**
     Contributions are welcome,
     but this report would not qualify.
    `docs/CONTRIBUTING.md:14-24` limits issues to bugs and feature requests and requires a reproduction.
    `docs/CONTRIBUTING.md:53-67` requires AI assistance disclosure and thorough human review.
    The issue templates and repository policy were checked;
     no policy bans a reviewed,
     disclosed contribution.
5.  **Will they likely fix it?**
     No upstream behavior change is warranted by the evidence.
    Automatically downgrading would violate `DNSOverTLS=yes`,
     while ignoring per-link DNS would violate routing policy.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
     No.
    Constraints 1 and 5 fail,
     so the auto-prototype gate does not trigger.
    The available fixes are local policy changes,
     not a systemd patch.

### Filing artifact

Nothing should be filed upstream.
There is no additive bug report or comment draft because the source,
 documentation,
 and live behavior agree.
The actionable artifact is this local configuration diagnosis.
