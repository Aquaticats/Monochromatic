# systemd-resolved 259 `resolvectl query --json` has no TTL, and the varlink `ResolveRecord` input flags reject `CLAMP_TTL`

A tool that needs the TTL of a DNS answer,
and wants it from one named link only,
cannot use `resolvectl query --json`.
It has to call the varlink method `io.systemd.Resolve.ResolveRecord` and read the TTL out of the raw record.
Passing the flag that looks like "clamp the TTL" as input makes the call fail.

## Symptom

- `resolvectl query --json=short github.com` prints addresses and no TTL,
  so the refresh interval of a rule that follows DNS cannot come from it.
- A `ResolveRecord` call with `flags` that include `SD_RESOLVED_CLAMP_TTL` (131072) fails with `org.varlink.service.InvalidParameter`,
  naming the `flags` parameter.
- A `ResolveRecord` call for a name with no `AAAA` record fails with `io.systemd.Resolve.NoSuchResourceRecord`,
  which is a normal answer and not a fault.
- A call naming a link with no DNS server fails with `io.systemd.Resolve.NoNameServers`.

## Root cause

`ResolveRecord` validates the caller's flags against a fixed list,
and `SD_RESOLVED_CLAMP_TTL` is not on it.
The method sets that flag itself after validation.

The method asks the validator to accept `SD_RESOLVED_NO_SEARCH` on top of the common list
(systemd commit `34d8764144ff93e658329dcb74d47f3c3e48bfff`,
`src/resolve/resolved-varlink.c:1156`):

```c
if (validate_and_mangle_query_flags(m, &p.flags, p.name, SD_RESOLVED_NO_SEARCH) < 0)
        return sd_varlink_error_invalid_parameter(link, JSON_VARIANT_STRING_CONST("flags"));
```

The validator rejects every bit outside the list (`src/resolve/resolved-dns-query.c:1628`):

```c
if (*flags & ~(SD_RESOLVED_PROTOCOLS_ALL|
               SD_RESOLVED_NO_CNAME|
               SD_RESOLVED_NO_VALIDATE|
               SD_RESOLVED_NO_SYNTHESIZE|
               SD_RESOLVED_NO_CACHE|
               SD_RESOLVED_NO_ZONE|
               SD_RESOLVED_NO_TRUST_ANCHOR|
               SD_RESOLVED_NO_NETWORK|
               ...
               ok))
        return -EINVAL;
```

The method then adds the clamp flag itself (`src/resolve/resolved-varlink.c:1172`),
so a cached answer already carries a TTL reduced by the age of the cache entry:

```c
r = dns_query_new_for_varlink(m, &q, question, question, /* question_bypass= */ NULL, p.ifindex, p.flags|SD_RESOLVED_NO_SEARCH|SD_RESOLVED_CLAMP_TTL, link);
```

The header comment (`src/shared/resolved-def.h:51`) calls the flag an input,
which is why it looks like something a caller may pass:

```c
/* Input: If reply is answered from cache, the TTLs will be adjusted by age of cache entry */
#define SD_RESOLVED_CLAMP_TTL       (UINT64_C(1) << 17)
```

An earlier reading in this investigation,
that the clamp flag had to be passed to get honest TTLs,
was wrong:
the method sets it unconditionally,
so the TTL in the reply is already the remaining TTL.

Not source-traced:
that `resolvectl query --json` prints no TTL was observed by running it,
not read in `src/resolve/resolvectl.c`.

The links above are commit-pinned paths of a clone,
not selection permalinks.
They were not loaded in `agent-browser`.

## Verification

Tested on systemd 259 (259.10-1.fc44) with the source at commit `34d8764144ff93e658329dcb74d47f3c3e48bfff`,
on 2026-10-31.
The harness is `${HOME}/temp/agent/wg-update-clearnet/measure-answers.ts`,
which opens `/run/systemd/resolve/io.systemd.Resolve`,
sends the request JSON followed by a NUL byte,
and reads up to the next NUL byte.

Works:

- `{"method":"io.systemd.Resolve.ResolveRecord","parameters":{"name":"github.com","type":1,"class":1,"ifindex":11,"flags":26881}}`
  with `26881 = DNS(1) + NO_SEARCH(256) + NO_SYNTHESIZE(2048) + NO_ZONE(8192) + NO_TRUST_ANCHOR(16384)`.
  The reply holds `rrs[].rr.address` as a byte array and `rrs[].raw` as a base64 wire record,
  whose TTL follows the owner name,
  the type and the class.
- All 21 update hosts of `package/wg/cli-quicker/src/update-hosts.data.ts` answered through the tunnel link:
  132 addresses,
  92 distinct,
  at most 24 for one host,
  TTLs from 1 s to 5018 s.

Fails:

- `flags` including `131072` (`CLAMP_TTL`):
  `org.varlink.service.InvalidParameter`.
- `type` 28 for a host without an `AAAA` record:
  `io.systemd.Resolve.NoSuchResourceRecord`.

The reply `flags` value of an answer was `8650753`,
which has bit 18 (`SD_RESOLVED_CONFIDENTIAL`) set.
The source comment says that flag means "Result was only sent via encrypted channels,
or never left this system",
so the flag alone does not prove a DNS-over-TLS hop.
The tunnel link's own setting does:
`resolvectl status fr-par-fr1` shows `+DNSOverTLS`,
and `resolvectl dnsovertls fr-par-fr1` prints `yes`,
the strict mode without cleartext fallback.

## Verified workarounds

Read the TTL out of the raw record.
Skip the owner name (labels ended by a zero byte,
or a two-byte pointer starting with `0xC0`),
skip the two-byte type and the two-byte class,
and read the four-byte big-endian TTL.
`package/wg/cli-quicker/src/tunnel-dns-lookup.ts` does this in `ttlOfRawRecord`.
Tradeoff:
the layout of `raw` is the DNS wire format,
but the field names of the varlink reply (`rrs`,
`rr`,
`address`,
`raw`) were observed on 259.10 only and not checked on other releases,
so a systemd update can move them;
the tests build replies in that shape,
which pins the parser to what was observed and cannot detect a change of systemd.

Do not pass `CLAMP_TTL`.
The method sets it.

## What does not work

- `resolvectl query --json` for TTLs:
  it prints none.
- Passing `SD_RESOLVED_CLAMP_TTL` as an input flag:
  rejected with `InvalidParameter`.
- Node's `dns.resolve4` with `{ ttl: true }` for the same job:
  it asks the stub on `127.0.0.53` and the stub chooses the upstream by its own rules,
  not by a named link.
  Its source (`src/cares_wrap.h` of Node v26.13.0) calls `ares_query_dnsrec`,
  which does not expand search domains,
  so the concern that a `lan` search domain could make `github.com.lan` an answer does not arise there.

## Upstream filing decision

Nothing to file.

1. Is it really upstream's fault?
   No.
   The flag list is deliberate,
   and the method sets the flag itself.
   The header comment says "Input" for a flag that is only set internally,
   which is wording and not behavior.
2. Can upstream fix it?
   Yes,
   by rewording one comment;
   there is nothing to fix in behavior.
3. Do they support this use case?
   Yes,
   `ResolveRecord` exists for per-link record queries.
4. Would the repository welcome a contribution?
   Not searched:
   contribution and AI-assistance policy files were not read.
5. Will they likely fix it?
   Not applicable;
   there is no behavior defect.
6. Prototype:
   none,
   because there is no defect.

Filing a one-comment wording issue would carry the poster's account in front of strangers for no benefit.
Whether to file anything is the poster's decision.
`.out-of-scope/` was not checked,
because nothing is drafted.
