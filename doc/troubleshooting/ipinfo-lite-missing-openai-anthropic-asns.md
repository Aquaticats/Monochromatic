# IPinfo Lite 2026-10-06 generation carries no networks
for OpenAI `AS401864` or Anthropic `AS400243` and `AS401551`,
so those `AS` entries in `wg-allowedips` inputs subtract nothing

## Symptom

`wg-allowedips` and `wg-quicker up` write one warning per affected entry and continue:

```text
[warn] [networks] ASN AS4167 contributed no networks; skipping ASN
[warn] [networks] ASN AS400243 contributed no networks; skipping ASN
[warn] [networks] ASN AS401551 contributed no networks; skipping ASN
[warn] [networks] ASN AS401864 contributed no networks; skipping ASN
```

The per-ASN cache files exist but hold zero bytes:

```console
$ ls -l ~/.cache/wg-allowedips/asn/cache_AS401864.txt
-rw-r--r--. 1 user user 0 Oct  6 06:45 cache_AS401864.txt
```

The visible consequence is not the warning.
It is that vendor traffic stays inside the generated `AllowedIPs` value.
Measured on 2026-10-06 against a disallowed input carrying `AS401864` and `AS401518`:
`api.openai.com`,
`auth.openai.com`,
`chat.openai.com`,
and `codex-portals.api.openai.com` all resolved to addresses that the generated value still routed
through the tunnel.

## Root cause

The consumer path is correct;
the dataset lacks the records.

`package/wg/core-allowedips/src/asn-fetch.ts:34` names the single upstream artifact:

```ts
const IPINFO_LITE_URL = 'https://ipinfo.io/data/ipinfo_lite.json.gz';
```

`package/wg/core-allowedips/src/asn-fetch.ts:228` downloads it with the caller's token:

```ts
  const response = await fetch(databaseUrl(token,),);
```

Each NDJSON line is pre-filtered by substring,
parsed,
then compared exactly
(`package/wg/core-allowedips/src/asn-fetch.ts:102` and `package/wg/core-allowedips/src/asn-fetch.ts:116`):

```ts
  if (!line.includes(targetAsn,))
    return NO_MATCHING_NETWORK;
```

```ts
  if (entry.asn !== targetAsn)
    return NO_MATCHING_NETWORK;
```

An empty result reaches `package/wg/core-allowedips/src/networks.ts:350`,
which warns and contributes nothing:

```ts
    if (networks.length === 0) {
      l.warn(`ASN ${asn} contributed no networks; skipping ASN`,);
      return [];
    }
```

The empty result is still cached atomically by `writeAsnCache`
(`package/wg/core-allowedips/src/asn-cache.ts:210`),
which is why a zero-byte cache file is the durable trace of an empty answer rather than of a failed
download.
A failed download throws instead,
and the command exits non-zero.

The deciding measurement is the artifact itself.
The 2026-10-06 generation contains `769376432` decompressed bytes of NDJSON,
and a whole-file search for each target string returns:

```text
AS401864  0 records   (OPENAI, registered 2024-11-21 per ARIN RDAP)
AS400243  0 records   (ANTHROPIC-2 per ARIN RDAP)
AS401551  0 records   (ANTHROPIC-3 per ARIN RDAP)
AS4167    0 records
AS401518  1 record    (OAI-01, `199.47.142.0/23`)
AS399358  3 records   (Anthropic, PBC)
AS32590   153 records (Valve, control case)
```

A bare substring search for `AS4167` returns `140` hits,
which is a trap:
every one belongs to a longer identifier
(`AS41678` 127,
`AS41676` 9,
`AS41670` 3,
`AS41679` 1).
Those lines pass the substring pre-filter and are then correctly rejected by the exact comparison,
so the pre-filter costs parses but never produces a wrong network.

IPinfo Lite holds exactly five records for these two vendors in this generation:

```text
160.79.104.0/23   AS399358  Anthropic, PBC
2607:6bc0::/48    AS399358  Anthropic, PBC
2607:6bc0:11::/48 AS399358  Anthropic, PBC
209.249.57.0/24   AS60808   Anthropic, PBC
199.47.142.0/23   AS401518  OpenAI OpCo, LLC
```

Upstream is a proprietary dataset with no public source tree,
so the trace covers the in-repo consumer path plus direct measurement of the downloaded artifact.

## Verification

Artifact identity,
from the authenticated redirect target and its response headers:

```text
url           https://dl.assets.ipinfo.io/v1/ipinfo_lite.json.gz
generation    1791273827000
overwrite-md5 f710e15bd939e7afee37c592893f50d4
last-modified Tue, 06 Oct 2026 08:03:47 GMT
compressed    27975368 bytes
decompressed  769376432 bytes
```

Runnable harness.
`IPINFO_TOKEN` stays in the environment so the token never reaches the transcript:

```sh
curl -sSL "https://ipinfo.io/data/ipinfo_lite.json.gz?token=${IPINFO_TOKEN}" \
  | gzip -dc > ipinfo_lite.json
for asn in AS401864 AS400243 AS401551 AS4167 AS401518 AS399358 AS32590; do
  printf '%s %s\n' "${asn}" "$(rg -c "\"asn\": \"${asn}\"" ipinfo_lite.json || echo 0)"
done
```

In-repo harness,
which reproduces the warning through the shipped command:

```sh
mise run //package/wg/cli-allowedips:run -- \
  --allowed ~/allowed.txt \
  --disallowed ~/disallowed.txt
```

Catalog of entries that resolve to networks in this generation:
`AS32590` (Valve,
`153` records),
`AS399358` (Anthropic,
`3` records),
`AS60808` (Anthropic,
`1` record),
`AS401518` (OpenAI,
`1` record),
`AS5091` and `AS394562` (Stripe).

Catalog of entries that resolve to nothing:
`AS401864`,
`AS400243`,
`AS401551`,
`AS4167`.

Registry identities came from ARIN RDAP,
for example `https://rdap.arin.net/registry/autnum/401864`,
whose `name` field is `OPENAI`.
`rdap.org` answers these with a `302` redirect,
so a client that does not follow redirects reports a parse failure instead of the name.

## Verified workarounds

- Name the hostnames the application actually contacts.
  Measured effect:
   adding `14` OpenAI and ChatGPT hostnames plus `oi-new-server.fly.dev` and
  `models.dev` to the disallowed input moved `16` of `17` candidate endpoints out of the generated
  value.
  Tradeoff:
   each hostname is a point-in-time DNS snapshot,
  and every removed host route splits the aggregate that contained it,
  so the value grows
  (`4022` networks before those additions,
  `6517` after).
- Use the vendor's registered prefixes as literal CIDR entries,
  for example `160.79.104.0/23` and `2607:6bc0::/48`.
  Tradeoff:
   it only covers what the vendor publishes,
  and it silently rots when they announce more.
- Use the ASN that IPinfo Lite does carry:
  `AS399358` and `AS60808` for Anthropic,
  `AS401518` for OpenAI.
  Tradeoff:
   `AS401518` yields one `/23`,
  which covers none of OpenAI's Cloudflare-fronted API hostnames.
- Exempt the application instead of its destinations,
  through `wg-quicker`'s `ExemptMark` cgroup marking.
  Tradeoff:
   it covers every destination of the exempted cgroup,
  and it needs the watcher running for the whole tunnel lifetime.

## What does not work

- Adding `AS401864`,
  `AS400243`,
  `AS401551`,
  or `AS4167` to a disallowed input.
  Zero records exist,
  so the entries only produce warnings.
- Waiting for cache expiry.
  The zero-byte caches in this investigation were rewritten by a fresh authenticated download at
  `2026-10-06T06:45:31`,
  so a refresh reproduces the empty answer.
- Expecting one vendor ASN to cover a vendor's whole footprint.
  Anthropic's traffic appears under two ASNs in this dataset,
  and OpenAI's public API hostnames sit on Cloudflare addresses that no OpenAI ASN contains.
- Treating a bare substring hit as evidence.
  `AS4167` matches `140` lines that all belong to `AS41670` through `AS41679`.

## Upstream filing decision

Nothing to file.

1. Really upstream's fault:
   yes.
   The in-repo filter is exact and demonstrably works for `AS32590`,
   `AS399358`,
   and `AS401518` in the same artifact.
2. Can upstream fix it:
   yes.
   The dataset could carry announcements for `AS401864`,
   `AS400243`,
   and `AS401551`.
3. Are they supporting this use case:
   yes.
   The product sells ASN-tagged network rows,
   and this repository consumes it exactly that way.
4. Would the repository welcome our contribution:
   not applicable.
   IPinfo Lite is a commercial dataset distributed as a download behind a token.
   There is no public source tree,
   no public issue tracker,
   and no contribution path for its contents,
   so no `CONTRIBUTING.md` or filing policy exists to check.
5. Will they likely fix it:
   unknown.
   No public signal either way,
   and absence of signal is not a failure of this constraint.
6. Have we prototyped a minimal fix:
   not applicable.
   The missing artifact is data,
   not code,
   so no patch to their side can be prototyped here.

Constraint 4 has no affirmative answer,
and constraint 6 cannot be met,
so the default policy holds and nothing is filed.
The actionable channel is IPinfo support or a different ASN data source,
both of which are purchasing decisions rather than repository work.
