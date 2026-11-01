# Njalla's DNS JSON-RPC API answers a valid `list-records` call
with `-32000 Invalid arguments` when the endpoint path lacks its trailing slash

**Date**:
 2026-10-09.
**Subject**:
 `POST https://njal.la/api/1` with a well-formed JSON-RPC body returns an
argument-validation error,
while the identical body posted to
`https://njal.la/api/1/` succeeds.
The same investigation found that Njalla's published records documentation
describes three record types while a live zone uses ten,
which is how
[`aws-cloudfront-mirror.md`](aws-cloudfront-mirror.md)
came to record that Njalla has no CNAME-flattening equivalent.
Discovered while automating the DNS side of the remedy in
[`doc/handover/acm-caa-renewal-blocked.md`](../handover/acm-caa-renewal-blocked.md).

## Symptom

Every method fails identically at the unslashed endpoint,
including one that takes no parameters at all:

```text
{"error": {"code": -32000, "message": "Invalid arguments"}, "jsonrpc": "2.0"}
```

The HTTP status is 200,
so nothing about the transport suggests a routing problem.
The error names arguments,
which sends the investigator to the parameter list rather than to the URL.

Two further surprises appear once the endpoint is correct.
Adding a CAA record beside an existing CNAME is refused with the registrar's
own wording:

```text
add-record rejected: code 400 You can not have both CNAME and CAA records.
```

And the response envelope differs between methods.
`add-record` returns the created record at the top level:

```json
{"id":"2849539","name":"anameprobe","type":"ANAME","content":"aquati.cat","ttl":300}
```

while `remove-record` returns a list and a status:

```json
{"records":[{"id":"2849535","name":"anameprobe","type":"ANAME","content":"aquati.cat","ttl":300}],"status":"removed"}
```

Record identifiers are strings in responses,
which matters when a caller types them as numbers and then compares.

## Root cause

Njalla's API is a closed service with no published source,
so the cause of the trailing-slash sensitivity cannot be traced to a code path.
What follows is black-box measurement,
not a reading of deciding source,
and it does not establish that the behaviour is intended.

The endpoint path is the only variable that matters.
Holding the body constant at a `list-records` call for a real domain,
five shapes were posted and the outcomes measured:

- `https://njal.la/api/1/` with a string `id`:
   200,
  95 records.
- `https://njal.la/api/1/` with a numeric `id`:
   200,
  95 records.
- `https://njal.la/api/1/` with no `id` member:
   200,
  95 records.
- `https://njal.la/api/1/` with no `jsonrpc` member:
   200,
  95 records.
- `https://njal.la/api/1` with a string `id`:
   200,
  `-32000 Invalid arguments`.

So the server does not require JSON-RPC 2.0 framing,
does not require an `id`,
and does not care whether `id` is a string or a number.
It rejects the same body one character earlier in the path.

### A reading that was wrong

The first diagnosis was that the request needed JSON-RPC 2.0 framing with a
string `id`,
because the published Go client sends
`{"jsonrpc":"2.0","method":...,"params":...,"id":"1"}`
and an early attempt using a numeric `id` at the unslashed path had failed.
Both variables were changed at once,
so the successful call proved nothing about either.
The matrix above separates them:
`id` shape and the `jsonrpc` member are irrelevant,
and the trailing slash is the whole difference.

The second finding has a documented cause on Njalla's side.
`https://njal.la/docs/records/` describes exactly three record types,
`A`,
`AAAA`,
and `SRV`.
A live zone of 85 records used ten:

```json
{"ANAME":43,"CNAME":12,"TXT":13,"MX":4,"A":3,"AAAA":3,"Redirect":2,"HTTPS":2,"CAA":2,"Dynamic":1}
```

Eight of those ten are absent from the documentation,
including `ANAME`,
which is a flattening alias that answers `A` and `AAAA` with no CNAME on the
wire and that does coexist with `CAA` at the same name.
Reading the documentation as a capability list is what produced the incorrect
claim in
[`aws-cloudfront-mirror.md`](aws-cloudfront-mirror.md)
that Njalla offers no CNAME-flattening equivalent.

## Verification

Tested on 2026-10-09 against a live zone,
with a token from the account's API settings,
using Node 26's `fetch`.
No client library was involved,
so the results describe the service rather than a wrapper.

Runnable harness.
It prints one line per body shape and never prints the token:

```bash
node --input-type=module -e '
import {readFileSync} from "node:fs";
import {homedir} from "node:os";
const token = readFileSync(homedir() + "/.config/njalla/token", "utf8").trim();
const bodies = {
  "slashed, string id":   ["https://njal.la/api/1/", {jsonrpc:"2.0", id:"1", method:"list-records", params:{domain:"<domain>"}}],
  "slashed, numeric id":  ["https://njal.la/api/1/", {jsonrpc:"2.0", id:1,   method:"list-records", params:{domain:"<domain>"}}],
  "slashed, no id":       ["https://njal.la/api/1/", {jsonrpc:"2.0",         method:"list-records", params:{domain:"<domain>"}}],
  "slashed, no jsonrpc":  ["https://njal.la/api/1/", {              id:"1",  method:"list-records", params:{domain:"<domain>"}}],
  "unslashed, string id": ["https://njal.la/api/1",  {jsonrpc:"2.0", id:"1", method:"list-records", params:{domain:"<domain>"}}],
};
for (const [label, [url, body]] of Object.entries(bodies)) {
  const r = await fetch(url, {method:"POST", headers:{Authorization:`Njalla ${token}`, "Content-Type":"application/json"}, body:JSON.stringify(body)});
  const j = await r.json();
  console.log(label.padEnd(22), r.status, j.error ? `${j.error.code} ${j.error.message}` : `records=${j.result?.records?.length}`);
}'
```

### Catalog: calls that succeed

All against `https://njal.la/api/1/` with an
`Authorization: Njalla <token>` header:

- `list-records` with `{"domain":"<domain>"}`
- `list-records` with a numeric `id`,
  a missing `id`,
  or a missing `jsonrpc` member
- `add-record` with `{"domain","type","name","content","ttl"}` for types
  `ANAME`,
  `CNAME`,
  and `CAA`
- `remove-record` with `{"domain","id"}`,
  where `id` is taken from `list-records`

### Catalog: calls that fail

- Any method at `https://njal.la/api/1`,
  `https://njal.la/api/`,
  or `https://njal.la/api`.
  The first answers with `-32000 Invalid arguments`;
  the other two answer 403 and an HTML page respectively,
  so they are routing failures rather than argument failures.
- `add-record` for a `CAA` at a name that already holds a `CNAME`,
  refused with `code 400 You can not have both CNAME and CAA records.`
- Trusting the `add-record` envelope to carry the same shape as
  `remove-record`,
  or the reverse.

### Zone accounting that proves the probes were cleaned up

The zone held 85 records before the work.
The remedy removed one `CNAME` and added one `ANAME`,
four `CAA` records at that leaf,
four `CAA` records at an intermediate label,
one `CNAME` leaf,
and one validation `CNAME`,
which is 85 minus 1 plus 10,
or 95.
`list-records` returns 95.
Two throwaway probe labels created during investigation are therefore absent,
since either one would make the count 96 or higher.

## Verified workarounds

### 1. Post to the slashed endpoint

`https://njal.la/api/1/`,
with `Authorization: Njalla <token>`.

Tradeoff:
none functionally,
but the difference from the published Go client's constant is one character,
so a reader comparing the two will not see it.
Write the endpoint down where the call is made.

### 2. Read records back instead of trusting the write envelope

After a mutation,
call `list-records` and select by name and type.

Tradeoff:
one extra round trip per mutation,
and a race if anything else edits the zone concurrently.
It removes all dependence on envelope shape,
which differs per method.

### 3. Take identifiers from `list-records`

`remove-record` and `edit-record` need an `id`,
and identifiers are strings in responses.

Tradeoff:
an extra read before every removal.
Comparing a string `id` against a number silently matches nothing,
which is the failure this avoids.

### 4. Use `ANAME` where an alias must coexist with `CAA`

Measured:
an `ANAME` and a `CAA` record at one name both publish,
the `A` answer carries the flattened addresses with no `CNAME`,
the `CNAME` query returns empty,
and the `CAA` query returns the record.
An `ANAME` pointed at a CloudFront distribution domain answered four `A` and
eight `AAAA` records,
and requests to two of those addresses served the mirrored content with HTTP
200 and a matching body length.

Tradeoff:
flattening publishes the addresses Njalla resolved rather than the querier's
nearest edge,
and Njalla documents no re-resolution interval,
so the staleness window if the target's addresses rotate is unmeasured.
For a CDN target this costs edge locality;
for a static-address target it costs nothing.

Measured a second time on 2026-10-09 against a Fastly shared-SNI target,
`x.sni.global.fastly.net`,
and the result differs from the CloudFront case in
a way that matters.
The CloudFront probe answered four `A` and eight `AAAA`
records.
The Fastly probe answered **one** `A` and **one** `AAAA`,
`199.232.173.242` and `2a04:4e42:6b::498`,
and the IPv4 address was not among
the four that Fastly's own API publishes as the expected `A` records for that
hostname.
Both are Fastly anycast ranges and the address served the content
correctly,
but flattening handed every querier a single address instead of a
set,
and not the set the CDN documents.
So the flattening tradeoff is not
uniform across CDN targets:
it depends on how many addresses the target's
authoritative server returns to Njalla's resolver,
which is not under the zone
owner's control.
A CDN that publishes a fixed anycast set through its API is a
candidate for explicit `A` and `AAAA` records instead of an `ANAME`,
which is
what Fastly's own apex-domain guidance recommends and what its discouragement
of "proprietary CNAME flattening features offered by some DNS providers (e.g.,
ALIAS or ANAME)" amounts to in practice.

## What does not work

- **Fixing the body.**
  Adding `jsonrpc`,
  adding or removing `id`,
  and changing `id` between string and number were all measured irrelevant.
- **The other endpoint paths.**
  `https://njal.la/api/` answers 403 with an HTML page,
  and `https://njal.la/api` answers 200 with an HTML document.
  Neither is a JSON-RPC endpoint.
- **Inferring capabilities from `https://njal.la/docs/records/`.**
  It documents `A`,
  `AAAA`,
  and `SRV`.
  The live zone used ten types,
  and the type that mattered most here,
  `ANAME`,
  is not among the three.
- **Adding a `CAA` beside a `CNAME`.**
  Refused at the API with the verbatim message in "Symptom".
  Replace the `CNAME` with an `ANAME` instead,
  per workaround 4.
- **Proving a name is retired by expecting `NXDOMAIN`.**
  This zone holds a
  wildcard record,
  `*.aquati.cat. IN HTTPS` advertising `alpn="h3"`,
  so
  every name under the apex exists by synthesis and `NXDOMAIN` is unreachable
  for any of them.
   A deleted name answers `NOERROR` with `ANSWER: 0` and the
  SOA in the authority section,
   which is indistinguishable from a label that
  never existed.
   Verify retirement with a control pair instead:
   the retired
  name and a known-never-existed name must both answer zero records,
   while a
  name that must still resolve answers at least one.
   Measured on 2026-10-09,
  all three behaved as described and a serving name answered four `A` records.

## Upstream filing decision

`.out-of-scope/` was checked and holds no exemption matching Njalla,
a DNS provider,
or this bug class.
The six-constraint check still ends at do not file.

1.  **Is it really upstream's fault?**
    Partly,
    and the split matters.
    The documentation gap is a defect:
    the records page omits eight of the ten types a live zone uses,
    and no public page describes the API at all,
    which is how an incorrect capability claim entered this repository's own
    troubleshooting documentation.
    The trailing-slash behaviour is unexplained rather than known-bad;
    a service may legitimately route `/api/1` and `/api/1/` differently,
    though answering a routing mismatch with an argument-validation error is
    misleading.
2.  **Can upstream fix it?**
    Yes,
    but not through any channel this repository can drive.
    Njalla is a closed service with no public source repository and no public
    issue tracker.
    The documented contact paths are an authenticated in-account support
    system and a documentation feedback request,
    both of which are private correspondence rather than a trackable report.
3.  **Are they supporting this use case?**
    Yes for the API:
    the account settings issue tokens,
    third-party clients exist for it,
    and the token used here had record-management scope.
    No for documenting it:
    the published guidance covers records,
    dynamic DNS,
    and mail,
    with no API page linked from `https://njal.la/docs/`,
    and the natural URL `https://njal.la/docs/api/` returns HTTP 404 with a
    page-not-found title,
    measured on 2026-10-09.
4.  **Would the repo welcome our contribution?**
    Not applicable.
    There is no repository.
    Checked:
    `njal.la/docs/` links six guidance pages and no source forge;
    the two client implementations relied on here,
    `romualdr/node-njalla-dns` and `dweee/libdns-njalla`,
    are third-party and are not Njalla's own,
    so editing them would not reach the service.
5.  **Will they likely fix it?**
    No signal either way.
    There is no public tracker to search,
    so no duplicate check is possible and no maintainer position is on record.
6.  **Have we prototyped a minimal fix?**
    Not applicable.
    Constraint 1 is only partly met and constraints 2 and 4 fail,
    so the auto-prototype trigger does not fire.
    There is also nothing consumer-side to patch beyond what workaround 1
    already does:
    this repository holds no Njalla client,
    and the task scripts that called the API live outside it.

Decision:
do not file.
The durable output is this document plus the corrected capability claim in
[`aws-cloudfront-mirror.md`](aws-cloudfront-mirror.md).
If the maintainer ever wants the documentation gap raised,
the only channel is Njalla's authenticated support system,
and the content worth sending is the records-page omission list in
"Root cause".

## References

- Njalla records guidance,
  which documents only `A`,
  `AAAA`,
  and `SRV`:
  <https://njal.la/docs/records/>
- Njalla guidance index,
  which links no API page,
  and `https://njal.la/docs/api/`,
  which returns HTTP 404:
  <https://njal.la/docs/>
- Third-party client showing the slashed endpoint constant and the
  `Authorization: Njalla <token>` header:
  <https://github.com/dweee/libdns-njalla/blob/master/client.go>
- Earlier third-party client whose default endpoint string lacks the trailing
  slash:
  <https://github.com/romualdr/node-njalla-dns>
- RFC 8659 section 3,
  the climb that makes an intermediate label sufficient:
  <https://www.rfc-editor.org/rfc/rfc8659.txt>
- Related:
  [`aws-cloudfront-mirror.md`](aws-cloudfront-mirror.md)
  issues 2,
  5,
  and 6,
  and
  [`doc/handover/acm-caa-renewal-blocked.md`](../handover/acm-caa-renewal-blocked.md),
  the task this surfaced during.
