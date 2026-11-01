# ACM renewal for the CloudFront mirror is blocked by the apex CAA

## Purpose

AWS sent an `AWS_ACM_CAA_CHECK_FAILURE` health event on 2026-10-09 for the
certificate that serves `aws.aquati.cat`.
The certificate expires on 2026-11-22 at 23:59:59 UTC,
which is 44 days after the investigation recorded here.
ACM states in that event that it retries until renewal succeeds or the
certificate expires,
so nothing breaks immediately and nothing fixes itself either.

This file hands over a completed diagnosis,
seven remediation options with a ranking,
and the dead ends that were measured so they are not re-explored.
The remedy was executed and verified on 2026-10-09;
see "Executed remedy and verification".
The options and their ranking are retained because two of them remain live
choices for later work.

The diagnosis also retracts a mechanism claim in
[`doc/troubleshooting/aws-cloudfront-mirror.md`](../troubleshooting/aws-cloudfront-mirror.md)
issue 6,
which is what made this failure look impossible.

## Decision on 2026-10-09

The maintainer chose option 1 followed by option 2,
with `mirror.amazon.aquati.cat` as the option 2 hostname.

Option 1 first stops the expiry clock with an AWS-only change and no DNS
dependency.
Option 2 then restores a hostname inside `aquati.cat`.
Each needs one `UpdateDistribution` call.
If Njalla access is immediate,
option 2 alone also beats the deadline,
since ACM issues minutes after validation;
option 1 stays the safer first step because it needs no DNS.

The chosen hostname has two labels below the apex,
so the CAA record belongs at `amazon.aquati.cat`,
one label above the leaf:

```text
amazon.aquati.cat.         CAA   0 issue "amazon.com"
amazon.aquati.cat.         CAA   0 issue "amazontrust.com"
amazon.aquati.cat.         CAA   0 issue "awstrust.com"
amazon.aquati.cat.         CAA   0 issue "amazonaws.com"
mirror.amazon.aquati.cat.  CNAME <distribution-domain>.
```

The climb for `mirror.amazon.aquati.cat` terminates at `amazon.aquati.cat`,
so the apex keeps `letsencrypt.org` and `certainly.com` untouched.

One consequence to accept deliberately:
every name under `amazon.aquati.cat` inherits that set,
so nothing in the subtree can obtain a Let's Encrypt certificate unless
`0 issue "letsencrypt.org"` is added there too.
The subtree is a natural home for Amazon-issued names only.

Asked and left unanswered,
so parked rather than assumed:

- relocating the Fastly mirror under the same intermediate label,
  which would let the apex return to `letsencrypt.org` only.
   **Resolved
  2026-10-09**,
   and not by relocation:
   see the section headed
  "Fastly mirror moved to a leaf CAA,
   and the AWS mirror renamed".
- the stale `mise.toml` comment claiming dprint formats markdown,
   when the
  markdown plugin is commented out in `package/config/dprint/index.json`;
- recording why `certainly.com` sits at the apex.
   **Resolved 2026-10-09**:
  the record is deleted.
   The why stays historical;
   no commit or document
  ever recorded adding it.

Blockers measured on 2026-10-09,
both cleared during execution:

- `aws sts get-caller-identity` failed with the expired-session error quoted
  in "What to inspect and how to respond" until the maintainer ran the
  interactive `aws login`.
  The identity that resulted is the account root,
  recorded as an observation in "Executed remedy and verification".
- No Njalla credential existed in this environment until the maintainer wrote
  one to `~/.config/njalla/token` with mode 600.
  The API's wire shape is undocumented and its endpoint is sensitive to a
  trailing slash;
  both are recorded in
  [`doc/troubleshooting/njalla-dns-api-jsonrpc-shape.md`](../troubleshooting/njalla-dns-api-jsonrpc-shape.md).

Reusable prior art for the AWS mutations lives in the private task
directory recorded in
[`aws-cloudfront-mirror-correction.md`](aws-cloudfront-mirror-correction.md):
`common.ts` wraps the CLI with bounded waits,
saves immutable evidence per call,
and `apply-live.ts` shows the read-config,
change-one-field,
assert-only-that-field-changed pattern this work must repeat.

## Option 7, measured after the decision

Execution began with a read-only zone dump through the Njalla API,
which returned 85 records including 43 live `ANAME` records and 2
`Redirect` records.
`ANAME` is a flattening alias:
Njalla resolves the target itself and answers A and AAAA,
with no CNAME on the wire.
Two probes at throwaway labels,
each deleted and verified absent before the script exited,
measured what that means here.

- ANAME and CAA coexist at one name.
   At `anameprobe.aquati.cat` the authoritative server returned
  `A 135.181.104.96`,
  an empty CNAME answer,
  and `CAA 0 issue "amazon.com"` together.
- Flattening a CloudFront target serves the mirror.
   At `cfprobe.aquati.cat`,
  an ANAME to the distribution domain answered four A and eight AAAA
  records,
  and `curl --resolve` against two of those addresses while requesting the
  real mirror hostname returned HTTP 200 and 3445 bytes,
  matching the origin body length measured earlier.
- The flattened addresses were in `18.239.18.0/24` while the CNAME path
  resolved to `3.168.2.0/24` from this host,
  so Njalla publishes the edge set it resolves,
  not the querier's nearest edge.

That yields a seventh option no earlier section considered:
replace the CNAME at `aws.aquati.cat` with an ANAME to the same target,
and publish the four Amazon `issue` records at `aws.aquati.cat` itself.
The climb for that FQDN then terminates at the leaf,
because the leaf has CAA and shows no alias on the wire.
ACM's existing retry loop renews the certificate already in place,
so nothing on the AWS side changes at all.

- Pros:
   the hostname does not change;
  the apex does not change;
  the existing certificate is renewed rather than replaced;
  no `UpdateDistribution`,
  so no Free-plan mutation risk and no deploy wait;
  no window in which any hostname is broken;
  zero cost.
- Cons:
   every querier receives the edge addresses Njalla resolved rather than
  its own nearest edge,
  which costs latency for distant clients;
  Njalla publishes no documentation of how often it re-resolves an ANAME
  target,
  so the staleness window if CloudFront retires those addresses is
  unmeasured;
  the zone's 43 existing ANAME records all target names under
  `aquati.cat` with static addresses,
  so nothing in the zone already flattens a CDN.

Options 2 and 7 compose.
Option 7 keeps `aws.aquati.cat` alive on the existing certificate,
and option 2 adds `mirror.amazon.aquati.cat` on a CNAME with no flattening.
One certificate can cover both names,
because each name's climb terminates at its own CAA record.

These measurements postdate the choice recorded in "Decision on
2026-10-09",
so that choice is being re-confirmed before any mutation.
No DNS record,
AWS resource,
or certificate has been changed.

## What to inspect and how to respond

Follow "Remediation steps" for option 1,
then for option 2,
substituting `mirror.amazon.aquati.cat` for the placeholder hostname and
`amazon.aquati.cat` for the placeholder intermediate label.
Every step is a DNS edit at Njalla or an AWS CLI call;
none needs code changes in this repository.

Five of the six options need an authenticated AWS session;
only option 4 is a pure DNS edit.
At the time of writing,
`aws sts get-caller-identity` fails with:

```text
aws: [ERROR]: Your session has expired. Please reauthenticate using 'aws login'.
```

Re-authenticate before any step that reads or mutates ACM or CloudFront
state.

## Repository changes made while diagnosing

Redaction,
forward-only,
because this repository is public
(`Aquaticat/Monochromatic`,
visibility measured as `PUBLIC`):

- Account-scoped AWS identifiers became placeholders plus the read-only
  call that recovers each one,
  in
  [`doc/troubleshooting/aws-cloudfront-mirror.md`](../troubleshooting/aws-cloudfront-mirror.md),
  [`doc/handover/aws-cloudfront-mirror-correction.md`](aws-cloudfront-mirror-correction.md),
  and
  [`package/ssg/aquati.cat/cloudfront/README.md`](../../package/ssg/aquati.cat/cloudfront/README.md).
- Redacted:
   the AWS account ID,
  the ACM certificate ID and ARN,
  CloudFront distribution IDs,
  a fixture distribution domain,
  a task-created custom cache policy ID,
  a CloudFront function ETag and distribution ETags,
  a pricing-plan subscription ID,
  and the ACM DNS validation token pair.
- Kept:
   AWS-published managed policy IDs,
  which are the same constants in every account,
  and artifact SHA-256 values,
  which verify deployed bytes without identifying the account.
- Git history still contains the original values.
  No history rewrite was performed or proposed.

Corrections to falsified claims:

- Issue 6 of
  [`doc/troubleshooting/aws-cloudfront-mirror.md`](../troubleshooting/aws-cloudfront-mirror.md)
  is marked retracted,
  with its text kept as the record of what was believed.
- Its "Current configuration state" section no longer asserts that only
  Let's Encrypt may issue for `aquati.cat`,
  or that the renewal CAA chain runs through `cloudfront.net`.
- Its "What does not work" entries for a wider apex CAA and for an
  alternate label carry dated corrections.
- The CNAME-following claim in Path 4 of
  [`doc/troubleshooting/cloudflare-mirror-evaluation.md`](../troubleshooting/cloudflare-mirror-evaluation.md)
  carries the same correction.
  That document's plan-tier conclusion is unchanged.

## Measured state on 2026-10-09

Authoritative answers came from `1-you.njalla.no`;
cross-checks came from `9.9.9.9`.

CAA,
the whole failure in four queries:

```bash
dig +short CAA aws.aquati.cat
# (no answer; the node holds a CNAME, and CAA cannot coexist with it)

dig +short CAA aquati.cat
# 0 issue "certainly.com"
# 0 issue "letsencrypt.org"

dig +short CAA cloudfront.net
# 0 issuewild "amazonaws.com"
# 0 issuewild "digicert.com"

dig +short CAA fastly.net
# 0 issue "certainly.com;validationmethods=dns-01"
# 0 issue "globalsign.com"
# 0 issue "letsencrypt.org;validationmethods=dns-01"
# 0 iodef "mailto:security@fastly.com"
```

The mirror and its certificate:

```bash
dig +short CNAME aws.aquati.cat
# <distribution-domain>.

echo | openssl s_client -connect aws.aquati.cat:443 -servername aws.aquati.cat 2>/dev/null \
  | openssl x509 -noout -subject -issuer -dates
# subject=CN=aws.aquati.cat
# issuer=C=US, O=Amazon, CN=Amazon ECDSA 256 M04
# notBefore=May  9 00:00:00 2026 GMT
# notAfter=Nov 22 23:59:59 2026 GMT
```

ACM's DNS validation record is intact,
so validation is not a second blocker:

```bash
dig +short CNAME _<validation-token>.aws.aquati.cat
# _<validation-value>.<validation-zone>.acm-validations.aws.
```

The Fastly mirror,
which is the precedent for a CDN hostname under this apex:

```bash
dig +short CNAME fastly.aquati.cat
# x.sni.global.fastly.net.

echo | openssl s_client -connect fastly.aquati.cat:443 -servername fastly.aquati.cat 2>/dev/null \
  | openssl x509 -noout -subject -issuer -dates
# subject=CN=fastly.aquati.cat
# issuer=C=US, O=Certainly, CN=Certainly Intermediate R1
# notBefore=Sep 26 09:37:04 2026 GMT
# notAfter=Oct 26 09:37:03 2026 GMT
```

The origin and the apex:

```bash
echo | openssl s_client -connect aquati.cat:443 -servername aquati.cat 2>/dev/null \
  | openssl x509 -noout -subject -issuer -dates
# subject=CN=aquati.cat
# issuer=C=US, O=Let's Encrypt, CN=YE1
# notAfter=Nov  9 10:59:10 2026 GMT
```

Nameservers are `1-you.njalla.no`,
`2-can.njalla.in`,
and `3-get.njalla.fo`.
The zone SOA serial is `2609142324`,
so the last zone change predates the 2026-10-09 failure by weeks.

## Root cause

RFC 8659 section 3 defines the Relevant RRset as a climb over the parents
of the queried FQDN:

```text
Let CAA(X) be the RRset returned by performing a CAA record query for
the FQDN X, according to the lookup algorithm specified in Section
4.3.2 of [RFC1034] (in particular, chasing aliases).  Let Parent(X) be
the FQDN produced by removing the leftmost label of X.
```

Alias chasing happens inside the single-node query `CAA(X)`.
The climb `Parent(X)` stays in the original tree.
RFC 8659 section 7 states this replaced the RFC 6844 algorithm,
which did climb CNAME and DNAME chains,
and gives the reason:
hosting providers would otherwise impose their own CAA policies on
customer names.

So for `aws.aquati.cat`:

1.  `CAA(aws.aquati.cat)` chases the CNAME,
    queries `CAA(<distribution-domain>)`,
    gets an empty answer.
2.  `Parent(aws.aquati.cat)` is `aquati.cat`.
3.  `CAA(aquati.cat)` is non-empty:
    `letsencrypt.org` and `certainly.com`.
4.  The climb terminates.
    Amazon is absent,
    so ACM must refuse.

`cloudfront.net` is never consulted,
because the climb never enters that tree.
Had it been consulted,
its `issuewild`-only set would not restrict a non-wildcard request at all:
RFC 8659 section 4.3 requires each `issuewild` property to be ignored for
a request that is not a wildcard.

The refusal ACM actually issued is the empirical confirmation.
Under the retracted issue 6 model,
ACM would have climbed to `cloudfront.net`,
then `net`,
then the root,
found no restricting set,
and issued.

## Why now

The certificate was issued on 2026-05-09 while a temporary
`aws.aquati.cat. CAA 0 issue "amazon.com"` record existed.
Issue 5 of the AWS mirror document records that Njalla refused to add the
CloudFront CNAME while that CAA record was present,
so the CAA was deleted and the CNAME added.
Renewal is the first ACM operation since that deletion,
and it is the first one that re-runs the CAA check against the apex.

The trigger is a documented plan step,
not a configuration mistake by anyone.
Issue 6 supplied an incorrect reason to believe the step was safe.

## The constraint triangle

While DNS is at Njalla and the mirror leaf is a CNAME,
the CAA climb for `aws.aquati.cat` has exactly two candidate nodes:
the leaf,
which cannot hold CAA next to a CNAME,
and the apex.
At most two of these three properties can hold:

- the public hostname stays `aws.aquati.cat`;
- the apex CAA stays free of Amazon;
- the certificate stays ACM-managed.

Every option below gives up exactly one,
except option 3,
which buys back all three by moving that one name to a DNS host that
supports an alias-type record,
so the leaf can carry its own CAA.

**Correction,
 measured later the same day**:
 the premise is wrong.
Njalla does have an alias-type record,
`ANAME`,
and it coexists with CAA at the same name.
So the leaf can carry its own CAA without moving DNS anywhere.
That is option 7,
and it buys back all three properties at zero cost,
trading CloudFront's per-querier edge selection for them.
See "Option 7,
 measured after the decision".

## Options and ranking

### Option 1: publish the mirror on the CloudFront default hostname

Drop `aws.aquati.cat` as an alias,
delete the ACM certificate,
and advertise the distribution's own `<distribution-domain>` as the mirror
URL.
AWS maintains the `*.cloudfront.net` certificate,
which is authorized by `cloudfront.net`'s own `0 issuewild "amazonaws.com"`.

Verified working today,
not inferred:

```bash
curl -s -o /dev/null -D - https://<distribution-domain>/ | head -12
# HTTP/2 200
# etag: "digwj5nnibty2np"
# content-length: 3445
# server: Caddy
# x-cache: Miss from cloudfront

curl -s -o /dev/null -w '%{http_code} %{size_download}\n' https://<distribution-domain>/en/about
# 200 6081
curl -s -o /dev/null -w '%{http_code} %{size_download}\n' https://aws.aquati.cat/en/about
# 200 6081

echo | openssl s_client -connect <distribution-domain>:443 \
  -servername <distribution-domain> 2>/dev/null | openssl x509 -noout -subject -issuer -dates -ext subjectAltName
# subject=CN=*.cloudfront.net
# issuer=C=US, O=Amazon, CN=Amazon RSA 2048 M04
# notBefore=Aug 25 00:00:00 2026 GMT
# notAfter=Mar 10 23:59:59 2027 GMT
# X509v3 Subject Alternative Name: DNS:*.cloudfront.net, DNS:cloudfront.net
```

The root body over both hostnames shares one `etag` and one
`content-length`,
so the deployed origin-override function serves identical content on the
default hostname.
The root body contains one absolute `https://aquati.cat/` reference,
which the mirror hostname serves identically.

Cookie isolation on a shared second-level domain is settled:
`cloudfront.net` appears in the Public Suffix List,
so browsers do not share cookie scope across CloudFront tenants.

- Pros:
   no certificate to renew ever again;
  no CAA change anywhere;
  no dependency on Njalla resolving anything,
  which makes it the most independent of the three paths;
  zero cost;
  the AWS Health events stop.
- Cons:
   the URL is an AWS-assigned hostname rather than a name you
  control,
  so moving this mirror to another provider later means publishing a new
  URL;
  the string is stable for the life of the distribution but is not
  brandable;
  it requires an AWS-side mutation of a distribution whose Free pricing
  plan has already rejected one attempted change
  (see "Open questions").

### Option 2: move the mirror one label deeper and put the CAA there

Publish `CAA 0 issue "amazon.com"` at an intermediate label such as
`mirror.aquati.cat`,
and serve the mirror from `aws.mirror.aquati.cat` as a CNAME to the
distribution.
The climb terminates at the intermediate label and never reads the apex.

The intermediate node is an ordinary name with no CNAME,
so it can hold CAA.
The leaf may still be a CNAME,
because the CAA lives one level up.
ACM's DNS validation record for the new leaf is a child of a CNAME'd name,
which Njalla accepts:
the existing validation record for `aws.aquati.cat` proves it.

This is the option
[`doc/troubleshooting/aws-cloudfront-mirror.md`](../troubleshooting/aws-cloudfront-mirror.md)
dismissed under "Alternate label" with the claim that the label name is
irrelevant.
That claim holds only for a single label below the apex.

- Pros:
   the apex stays free of Amazon,
  which was the original design goal;
  ACM keeps managing renewal;
  zero cost;
  the hostname stays inside `aquati.cat`;
  the same intermediate label can later carry `certainly.com` for a
  relocated Fastly mirror,
  which would let the apex return to Let's Encrypt only.
- Cons:
   the public URL changes;
  a CloudFront distribution holds one viewer certificate,
  and a certificate covering both the old and new names would drag the
  apex back into the CAA check,
  so the old hostname stops working at cutover with no TLS-level redirect
  possible;
  it needs a new certificate request,
  a new validation record,
  and an alias change on the distribution.

### Option 3: delegate the mirror name to Route 53 with an alias record

Delegate `aws.aquati.cat` as its own zone to Route 53,
use an alias record to the distribution,
and publish `CAA 0 issue "amazon.com"` at that zone apex.
An alias record is not a CNAME on the wire,
so CAA can coexist with it.

- Pros:
   the only option that keeps all three properties in the constraint
  triangle;
  the hostname,
  the narrow apex,
  and ACM managed renewal all survive;
  Route 53 charges $0.50 per hosted zone per month and does not charge
  for queries to alias records that target CloudFront distributions.
- Cons:
   an NS delegation at Njalla;
  authoritative DNS for that name splits across two providers;
  a recurring cost and a second control plane for a mirror the existing
  documentation ranks below Fastly;
  the same AWS-side mutation risk as option 1.

### Option 4: widen the apex CAA

Add the four Amazon issuer domains at the apex,
keeping `letsencrypt.org` and `certainly.com`:

```text
aquati.cat. CAA 0 issue "amazon.com"
aquati.cat. CAA 0 issue "amazontrust.com"
aquati.cat. CAA 0 issue "awstrust.com"
aquati.cat. CAA 0 issue "amazonaws.com"
```

AWS documents all four values;
`amazon.com` alone suffices today and the other three survive an issuing
CA rotation.

- Pros:
   one DNS edit and nothing else;
  the only option that requires no AWS-side mutation at all,
  which matters for a distribution whose Free plan has rejected a change
  before;
  ACM resumes on its own retry loop;
  no cost.
- Cons:
   Amazon Trust Services becomes authorized for every name that
  inherits the apex,
  including the bare `aquati.cat`,
  which cannot be scoped narrower than its own record set;
  the AWS CAA documentation states that with no `issuewild` present and an
  `issue` record for ACM,
  Amazon may also issue wildcards,
  so `*.aquati.cat` is included unless `issuewild` records are added;
  it is the option the existing documentation rejected on principle.

Two partial recoveries,
both zero cost:

- `www.aquati.cat` is an A record,
  so `www.aquati.cat. CAA 0 issue "letsencrypt.org"` re-narrows it.
- `aquati.cat. CAA 0 issuewild "letsencrypt.org"` plus
  `aquati.cat. CAA 0 issuewild "certainly.com"` keeps wildcards away from
  Amazon,
  because RFC 8659 section 4.3 makes `issuewild` take precedence for
  wildcard requests.

### Option 5: retire the AWS mirror

Delete the ACM certificate,
the distribution,
and the CNAME.

- Pros:
   the problem disappears;
  no recurring work;
  the apex stays narrow.
- Cons:
   discards distribution `<distribution-id>`,
  which was repaired and end-to-end verified on 2026-09-07;
  Fastly remains the only mirror,
  and the Fastly certificate observed on 2026-10-09 has a 30-day life,
  so it carries its own renewal cadence.

### Option 6: import a Let's Encrypt certificate into CloudFront

- Pros:
   no Amazon authorization anywhere;
  one issuer family across origin and mirrors.
- Cons:
   ACM's managed renewal documentation states imported certificates
  are not eligible for managed renewal,
  so this creates a 60-to-90-day ACME DNS-01 pipeline against the Njalla
  API plus a distribution update and deploy wait,
  indefinitely,
  and moves the private key out of the managed boundary.

### Ranking

1 > 2 > 4 > 3 > 5 > 6.

- 1 over 2:
   option 1 removes the certificate lifecycle entirely and needs
  no DNS from you at all,
  which is verified working today,
  while option 2 keeps a certificate,
  a validation record,
  and a DNS dependency to buy a branded name for a mirror the existing
  documentation calls diversification rather than a primary path.
- 2 over 4:
   option 2 keeps the apex free of Amazon at zero cost,
  while option 4 buys URL continuity by authorizing Amazon Trust Services
  for every inheriting name in the zone including wildcards.
- 4 over 3:
   both keep the current hostname,
  but option 4 is one DNS edit with no AWS-side mutation and no new
  provider,
  while option 3 adds a monthly cost,
  an NS delegation,
  and a split control plane.
- 3 over 5:
   if the mirror is worth keeping,
  option 3 keeps it with the security property intact for $0.50 a month,
  which is trivial against the $250 per month Cloudflare Business tier
  already rejected for this mirror.
- 5 over 6:
   retiring removes recurring work,
  while option 6 creates a permanent renewal pipeline for the lowest
  priority mirror.

Options 1 and 2 compose in that order:
option 1 stops the expiry clock today with no certificate work,
and option 2 can follow later if a branded hostname turns out to matter.

## Verified dead ends

Measured,
so they should not be re-explored:

- **ACM's ACME issuance**,
  added in 2026.
  AWS documents that ACME-issued certificates cannot be used with Elastic
  Load Balancing,
  CloudFront,
  or API Gateway,
  because AWS does not hold the private key.
  They are also not auto-renewed.
  The issuing CA would still be Amazon,
  so the CAA requirement would not change either.
- **Scoping Amazon's authorization with RFC 8657 parameters**,
  such as `accounturi`.
  AWS's CAA documentation lists only flags `0`,
  tags `issue` and `issuewild`,
  and the four CA domain names,
  and states that `iodef` is currently ignored.
  No parameter support is documented,
  so option 4 cannot be narrowed to one AWS account.
- **Deleting the apex CAA records.**
  An empty set all the way to the root restricts nothing,
  so every CA becomes authorized for the whole zone.
  Strictly worse than option 4.
- **A single-label alternate name** such as `cdn.aquati.cat`.
  Its only ancestor with CAA is the apex,
  so the structural problem is unchanged.
  This is the case the existing documentation generalized too far.
- **An alias-type record at Njalla.**
  Njalla's published records documentation lists A,
  AAAA,
  and SRV,
  and the live zone shows CNAME,
  MX,
  TXT,
  CAA,
  HTTPS,
  and NS.
  No ALIAS or ANAME type is documented,
  and
  [`doc/troubleshooting/aws-cloudfront-mirror.md`](../troubleshooting/aws-cloudfront-mirror.md)
  independently records the same finding.
  Flattening a CloudFront endpoint into static A records would also pin
  edge IPs that AWS rotates.
- **Waiting for ACM to succeed on retry.**
  The CAA check is deterministic against the same record set.
  Retries change nothing until DNS changes.

## Remediation steps

### Option 1

1.  Re-authenticate the AWS CLI.
2.  Read the live distribution configuration and ETag,
    and save both.
3.  Confirm the Free pricing plan permits the change on a copy or during
    a maintenance window,
    since the same plan rejected a custom cache policy on 2026-09-07.
4.  Remove `aws.aquati.cat` from `Aliases` and set the viewer certificate
    back to the CloudFront default in one `UpdateDistribution` call.
5.  After the deployment reaches `Deployed`,
    delete the CNAME at Njalla and delete the ACM certificate.
6.  Publish the default hostname wherever the mirror URL is referenced.

### Option 2

1.  At Njalla,
    add `mirror.aquati.cat. CAA 0 issue "amazon.com"`,
    plus `0 issue "letsencrypt.org"` if any name under that label will
    ever use Let's Encrypt.
2.  Add `aws.mirror.aquati.cat. CNAME <distribution-domain>.`
3.  Request a new ACM certificate for `aws.mirror.aquati.cat` with DNS
    validation in `us-east-1`.
4.  Add the validation CNAME ACM reports;
    it is a child of the new CNAME'd leaf,
    which Njalla accepts.
5.  After the certificate reaches `ISSUED`,
    update the distribution's `Aliases` and `ViewerCertificate` in one
    call.
6.  Remove the old `aws.aquati.cat` CNAME and delete the old certificate
    only after the new hostname verifies.

### Option 3

1.  Create a Route 53 hosted zone for `aws.aquati.cat`.
2.  In it,
    create an alias record to the distribution and
    `CAA 0 issue "amazon.com"` at the zone apex.
3.  At Njalla,
    replace the `aws.aquati.cat` CNAME with NS records pointing at the
    four Route 53 name servers.
4.  Wait for the delegation to propagate,
    then confirm ACM's retry succeeds without any other change.

### Option 4

1.  At Njalla,
    add the four `issue` records listed in option 4.
2.  Optionally add the two `issuewild` records and the `www` record from
    the partial recoveries in option 4.
3.  Confirm the apex set with `dig +short CAA aquati.cat`.
4.  Wait for ACM's retry;
    no AWS-side action is needed.

### Option 5

1.  Remove the alias and certificate as in option 1,
    then delete the distribution.
2.  Delete the CNAME and the ACM validation record at Njalla.
3.  Detach the CloudFront function first if the Free plan refuses to
    delete a distribution that still has one associated.

## Verification

At the boundary a viewer uses,
not at the API:

```bash
echo | openssl s_client -connect <mirror-hostname>:443 -servername <mirror-hostname> 2>/dev/null \
  | openssl x509 -noout -dates -issuer
# notAfter must be later than Nov 22 23:59:59 2026 GMT

curl -s -o /dev/null -w '%{http_code} %{size_download}\n' https://<mirror-hostname>/
# 200 and a body length matching https://aquati.cat/
```

ACM state,
after re-authentication:

```bash
aws acm describe-certificate --certificate-arn <certificate-arn> --region us-east-1 \
  --query 'Certificate.{Status:Status,Renewal:RenewalSummary,Domain:DomainValidationOptions}'
```

For option 1 and option 2,
the boundary check must run against the hostname that will actually be
published,
and the old hostname must be checked for the intended failure
(a TLS name mismatch) rather than left silently broken.

## Executed remedy and verification

The maintainer chose option 7 for the expiring certificate,
then asked for the option 2 hostname as well,
so both were implemented together with one certificate covering both names.

DNS at Njalla,
in this order:

1.  Two probes at throwaway labels,
    `anameprobe` and `cfprobe`,
    established that ANAME coexists with CAA and that a flattened
    CloudFront target serves the mirror.
    Both were deleted inside the same run,
    and the zone record count returned to its original 85.
2.  `aws.aquati.cat`:
    the CNAME was replaced with an ANAME to the same distribution domain,
    and the four Amazon `issue` records were added at the leaf.
    TTL 86400 was carried over from the CNAME it replaced,
    on the instruction to keep the TTL as-is and shorten it only if
    flattening staleness ever hurts.
3.  `amazon.aquati.cat`:
    the four Amazon `issue` records,
    so the climb for names below it terminates there.
4.  `mirror.amazon.aquati.cat`:
    a real CNAME to the distribution domain,
    so CloudFront keeps per-querier edge selection for that name.

Njalla states the CNAME restriction verbatim when it applies,
which is why step 2 replaced the alias rather than adding beside it:

```text
add-record rejected: code 400 You can not have both CNAME and CAA records.
```

On the AWS side:

5.  One certificate was requested for both names,
    `EC_prime256v1`,
    DNS validation,
    `us-east-1`.
    It reached `ISSUED` in 126 seconds,
    `notBefore` 2026-10-09,
    `notAfter` 2027-04-24.
6.  The existing name reused its deterministic validation record,
    which independently re-confirms issue 4 of the mirror troubleshooting
    doc.
    The new name needed one new validation CNAME,
    added automatically.
7.  `UpdateDistribution` changed `Aliases` and `ViewerCertificate` and
    nothing else,
    asserted field by field against the live configuration before
    submission.
    It was accepted and reached `Deployed` in 432 seconds.
8.  The replaced certificate was deleted after the maintainer authorized it.
    Three assertions ran first:
    the distribution's attached ARN differed from the deletion target,
    the target reported no attached resources,
    and the target's subject alternative name list held exactly one name,
    `aws.aquati.cat`,
    so it could not be the two-name replacement.
    `describe-certificate` on the deleted ARN now returns
    `ResourceNotFoundException`,
    and the filtered inventory holds one certificate whose subject
    alternative names cover both hostnames.

The deletion surfaced the strongest evidence that option 7 works as designed.
The replaced certificate's `notAfter` had moved from 2026-11-22 to 2027-04-24
between the morning's diagnosis and the evening's cleanup,
while its subject alternative name list was still the single original name.
Nothing renewed it except ACM's own retry loop,
which is exactly what the leaf CAA records unblocked.
A certificate's `notAfter` cannot change without reissuance,
so the field alone proves the renewal.

That answers the Free-plan question:
this plan does permit changing `Aliases` and `ViewerCertificate`.
It also accepted an alias whose DNS is a flattened ANAME,
which confirms empirically that CloudFront authorizes an alternate domain
name by the attached certificate's SAN rather than by the DNS record type.

Boundary verification,
every check passing:

- `/` and `/en/about` through both hostnames matched the origin on status,
  byte count,
  and body digest:
  200 and 3418 bytes with digest `15789f6f382e8750`,
  then 200 and 6070 bytes with digest `4f333bd94794b4af`.
- Both hostnames serve the new certificate,
  whose SAN list holds both names.
- IPv4 and IPv6 each returned 200 on both hostnames.
- The distribution lists both aliases.
- The replaced certificate reports no attached resources.

A second boundary check ran after the deletion:
`aws.aquati.cat`,
`mirror.amazon.aquati.cat`,
and `aquati.cat` each returned 200 with an identical 3445-byte body.
The byte count differs from the digest comparison earlier in this section
because the origin's content changed between the two checks;
parity held at both.

Cost of the whole remedy was zero.
ACM's pricing page states that it issues certificates at no cost for use
with services integrated with ACM,
and charges per domain only for ACME-enrolled and exportable certificates.
Alternate domain names carry no per-alias charge,
and Njalla records are included with the domain.

Two observations recorded rather than acted on:

- `aws acm list-certificates` returned an empty list throughout,
  because its documented default key-type filter admits only RSA and both
  certificates are ECDSA.
  Diagnosed in
  [`doc/troubleshooting/acm-list-certificates-key-type-default.md`](../troubleshooting/acm-list-certificates-key-type-default.md).
  The distribution was therefore identified by the alias it serves rather
  than by a saved identifier.
- The CLI session is the account root identity.
  Routine certificate and distribution work through root is a wider blast
  radius than this task needed.
  Nothing was changed about it.

## Fastly mirror moved to a leaf CAA, and the AWS mirror renamed

Executed later on 2026-10-09 for
[issue 674](https://github.com/Aquaticat/Monochromatic/issues/674).

The issue proposed relocating the Fastly mirror one label deeper,
to a
hostname like `fastly.mirror.aquati.cat` under an intermediate
`mirror.aquati.cat` carrying `0 issue "certainly.com"`.
The maintainer chose
instead to keep the existing hostname by applying option 7's mechanism to
Fastly:
make the leaf a flattened alias so it can carry its own CAA.
Two further
instructions arrived during execution:
add `mirror.fastly.aquati.cat` as a
working second hostname,
and move the AWS mirror from `aws.aquati.cat` to
`amazon.aquati.cat`.

The end state is symmetric across both mirrors,
and in both cases the
flattened name doubles as the climb-terminating label for the CNAME below it:

```text
aquati.cat.               CAA    0 issue "letsencrypt.org"
fastly.aquati.cat.        ANAME  x.sni.global.fastly.net.
fastly.aquati.cat.        CAA    0 issue "certainly.com"
mirror.fastly.aquati.cat. CNAME  x.sni.global.fastly.net.
amazon.aquati.cat.        ANAME  <distribution-domain>.
amazon.aquati.cat.        CAA    0 issue "amazon.com"   (plus three more)
mirror.amazon.aquati.cat. CNAME  <distribution-domain>.
```

### What Fastly publishes, measured before touching anything

`GET /tls/configurations/<tls-configuration-id>?include=dns_records` returns
the records Fastly expects for a hostname on that configuration:
four `A`
records marked `region: global`,
and the `CNAME` target.
Those four addresses
are the same values the `managed-http-a` challenge lists.

`GET /tls/subscriptions/<id>?include=tls_authorizations` returns three
challenge types for a shared-SNI hostname:

- `managed-dns`,
   a `CNAME` at `_acme-challenge.<hostname>` into
  `fastly-validations.com`;
- `managed-http-cname`,
   a `CNAME` at the hostname to the shared-SNI target;
- `managed-http-a`,
   `A` records at the hostname equal to the four anycast
  addresses above.

Fastly's Certainly product page states the renewal contract:
certificates are
valid 30 days,
and Fastly re-verifies and renews after 20 days "as long as your
DNS records point to Fastly and your Certification Authority Authorization
(CAA),
if in use,
is set to Certainly".

Fastly's apex-domain guide does **not** recommend "proprietary CNAME flattening
features offered by some DNS providers (e.g.,
ALIAS or ANAME)",
and instead
offers anycast `A` and `AAAA` records.
The maintainer chose the ANAME route
anyway,
after that trade-off was put to them with the measurements.

A throwaway probe at `fastlyprobe.aquati.cat`,
deleted inside the same run,
measured what Njalla's flattening actually publishes for this target:
`A 199.232.173.242` and `AAAA 2a04:4e42:6b::498`,
one of each.
That address
serves the mirror:
`curl --resolve` against it while requesting the real
hostname returned HTTP 200 with 3445 and 6081 bytes on two routes,
byte-for-byte identical to the live CNAME path.
It is a valid Fastly anycast
address,
but it is not one of the four values Fastly lists for
`managed-http-a`.
The zone record count returned to its starting value before
the script exited.

### Changes made

DNS at Njalla:

1.  `fastly.aquati.cat`:
    the CNAME was replaced with an ANAME to the same
    target,
    TTL 86400 carried over,
    then `0 issue "certainly.com"` was
    added at the leaf,
    TTL 86400 matching the apex record it replaces.
    Njalla refuses CAA beside a CNAME with
    `code 400 You can not have both CNAME and CAA records.`,
    which is why the
    alias had to be replaced rather than joined.
2.  `mirror.fastly.aquati.cat`:
    a real CNAME to the shared-SNI target,
    TTL 86400,
    matching `mirror.amazon.aquati.cat`.
3.  `_acme-challenge.mirror.fastly.aquati.cat`:
    the CNAME Fastly's
    `managed-dns` challenge asked for,
    TTL 300.
4.  `amazon.aquati.cat`:
    an ANAME to the distribution domain,
    TTL 86400.
    Its four Amazon CAA records already existed.
5.  The new ACM validation CNAME under `amazon.aquati.cat`,
    TTL 300.
    The
    value ACM gave for `mirror.amazon.aquati.cat` matched the record already
    in the zone byte for byte,
    which independently re-confirms issue 4 of
    [`doc/troubleshooting/aws-cloudfront-mirror.md`](../troubleshooting/aws-cloudfront-mirror.md).
6.  `aws.aquati.cat`:
    all six records deleted,
    the ANAME,
    the four CAA
    records,
    and the validation CNAME.
7.  `aquati.cat`:
    `0 issue "certainly.com"` deleted.

On the Fastly side:

8.  `mirror.fastly.aquati.cat` was added as a service domain.
    In this
    account's service model domains attach at the service level,
    so no
    version was cloned or activated.
9.  A **separate** Certainly subscription was created for the new name rather
    than adding it to the existing one,
    so the working `fastly.aquati.cat`
    certificate was never re-issued.
    Its authorization went `blocked`,
    then
    `passing`,
    then the subscription reached `issued`,
    and Fastly created
    the TLS activation itself.

On the AWS side:

10. One certificate was requested for `amazon.aquati.cat` and
    `mirror.amazon.aquati.cat`,
    `EC_prime256v1`,
    DNS validation,
    `us-east-1`.
    It reached `ISSUED` with both validations `SUCCESS`.
    A
    third certificate was unavoidable:
    ACM cannot add a name to an issued
    certificate,
    and CloudFront rejects an alias its viewer certificate does
    not cover.
11. One `UpdateDistribution` set `Aliases` to the two current names and
    `ViewerCertificate.ACMCertificateArn` to the replacement.
    A field-level
    diff against the saved live configuration asserted that only
    `Aliases.Items` and `ViewerCertificate.ACMCertificateArn` moved,
    plus the
    two response-only members `Certificate` and `CertificateSource`,
    which
    `update-distribution` rejects if echoed back.
    The deploy reached
    `Deployed`.
12. The replaced certificate was deleted under the same three assertions the
    earlier remedy used,
    with the third adapted because both certificates
    now hold two names:
    the target's subject alternative names contain
    `aws.aquati.cat`,
    where the replacement's contain `amazon.aquati.cat`.

### Verification

Content parity against the origin held for all four serving hostnames on two
routes,
by status,
byte count,
and body digest:
200 and 3445 bytes with digest
`15789f6f382e8750`,
then 200 and 6081 bytes with digest `4f333bd94794b4af`.
Strict TLS verification reported `ssl_verify_result` 0 everywhere,
and each
certificate's subject alternative names cover the hostname served.

The issuance for `mirror.fastly.aquati.cat` is the most interesting result,
but it is worth being precise about what it proves,
because it is **not** a
differential test.
That name has no CAA of its own,
and its CNAME target
`x.sni.global.fastly.net` answers `NOERROR` with `ANSWER: 0` for CAA,
which is
a genuine empty RRset rather than a failed lookup,
so the climb moved to
`fastly.aquati.cat` and found `0 issue "certainly.com"` there.
RFC 8659
section 3 makes that the Relevant RRset,
and the apex is not consulted.
The
issuance confirms Certainly accepted that set in practice.
It does not by
itself exclude the apex as the authorizing node,
because Certainly issued at
2026-10-09 22:51 UTC and the apex record was not deleted until roughly 23:45
UTC,
so both nodes authorized Certainly at that moment.
What carries the
conclusion is the measured climb,
not the issuance.

Deleting the apex record then turned the next renewal into the differential
test the issuance could not be.
From that point the only node that can
authorize Certainly for `fastly.aquati.cat` or `mirror.fastly.aquati.cat` is
`fastly.aquati.cat` itself.

The retired name was verified with controls rather than by assertion alone.
The zone holds a wildcard `*` HTTPS record advertising `alpn="h3"`,
so every
name under `aquati.cat` exists and `NXDOMAIN` is unreachable.
`aws.aquati.cat`
answers `NOERROR` with zero A and zero AAAA,
identical to a label that never
existed,
while all four serving hostnames answer with at least one record.

The zone went from 95 records to 93:
minus one CNAME,
minus six `aws.` records,
minus one apex CAA,
plus one ANAME and one CAA at `fastly.`,
plus two
`mirror.fastly.` records,
plus one `amazon.` ANAME,
plus one `amazon.`
validation CNAME.

### Residual risk and its fallback

One thing is still unverified.
Fastly's `tls_authorizations` for
`fastly.aquati.cat` reports `state: passing` with `updated_at` 2026-05-29,
months before this change,
so the field is cached and gives no live signal
about the new DNS shape.
Whether Fastly's day-20 re-verification still
considers the hostname pointed at Fastly once the leaf is an ANAME publishing
`199.232.173.242`,
rather than a CNAME or the four addresses in its
`managed-http-a` list,
can only be settled by that renewal,
due around
2026-10-16 for a certificate expiring 2026-10-26.

That renewal now settles two things at once rather than one.
Because the apex
record is already gone,
a successful re-issuance is also the differential
proof that the CAA record at `fastly.aquati.cat` authorizes Certainly on its
own,
which the earlier `mirror.fastly.aquati.cat` issuance could not provide.
A failure would leave the two causes entangled,
and separating them would mean
applying the fallback below and watching the next attempt.

The maintainer accepted deleting the apex record before that renewal,
on the
grounds that nothing depends on the Fastly mirror.
The CAA half of the risk
was already closed by the `mirror.fastly.aquati.cat` issuance.

If the renewal does fail,
the fallback is DNS-only and needs no Fastly-side
mutation:
replace the ANAME with the four `A` records Fastly publishes in
`managed-http-a`,
plus the four `AAAA` records measured from the shared-SNI
target (`2a04:4e42::498`,
`2a04:4e42:200::498`,
`2a04:4e42:400::498`,
`2a04:4e42:600::498`).
The leaf CAA record is unaffected by that swap.
Fastly's dualstack guide says the official anycast IPv6 list comes from
support;
the measured values are the ones the CNAME target already answers
with.
The cost of the fallback is manual maintenance if Fastly ever rotates
those addresses,
which a CNAME or an ANAME would follow by itself.

## Open questions

- Does Fastly's day-20 re-verification still accept `fastly.aquati.cat` as an
  ANAME?
   Due around 2026-10-16.
   See "Residual risk and its fallback".
- Does the origin's Let's Encrypt certificate use DNS-01 against Njalla?
  Only relevant if DNS for the apex ever moves.
- A Certificate Transparency watch for `aquati.cat` is tracked as
  [issue 675](https://github.com/Aquaticat/Monochromatic/issues/675).
  Amazon is authorized at `amazon.aquati.cat`,
  Certainly at
  `fastly.aquati.cat`,
  and each covers the CNAME leaf below it,
  rather than
  either sitting at the apex.
   That narrows what each could issue for but
  does not eliminate it,
  and `iodef` is documented as ignored by ACM,
  so it
  would not report these refusals.
- Should `fastly.aquati.cat` also authorize `letsencrypt.org`?
   It currently
  authorizes Certainly only,
  which is the tightest set that works.
   Fastly
  can migrate a subscription between certification authorities,
  and a
  Certainly-only record would block a move to Let's Encrypt until the record
  is widened.
   Left narrow deliberately.

## Redaction convention

Angle-bracket placeholders in this file and in the files it references are
deliberate,
not editorial gaps.
Each one names an account-scoped AWS value that a reader of this public
repository does not need.
The recovering call is written next to the placeholder in
"Repository changes made while diagnosing" and in the identifier
inventory of
[`doc/troubleshooting/aws-cloudfront-mirror.md`](../troubleshooting/aws-cloudfront-mirror.md).

Values that are public through DNS by design are written literally where
they carry evidence:
hostnames,
CNAME targets,
CAA record sets,
and certificate subject and issuer fields.

## References

- RFC 8659 section 3,
  Relevant Resource Record Set:
  <https://www.rfc-editor.org/rfc/rfc8659.txt>
- RFC 8659 section 7,
  Differences from RFC 6844,
  including why CNAME chain climbing was removed:
  <https://www.rfc-editor.org/rfc/rfc8659.txt>
- AWS Certificate Manager,
  Certification Authority Authorization problems,
  with the four Amazon CA domain names:
  <https://docs.aws.amazon.com/acm/latest/userguide/troubleshooting-caa.html>
- AWS Certificate Manager,
  configure a CAA record,
  including the `issuewild` interaction and the ignored `iodef`:
  <https://docs.aws.amazon.com/acm/latest/userguide/setup.html#setup-caa>
- AWS Certificate Manager,
  managed certificate renewal eligibility,
  including imported and ACME certificates:
  <https://docs.aws.amazon.com/acm/latest/userguide/managed-renewal.html>
- AWS Certificate Manager,
  issuing certificates through ACME,
  including the CloudFront binding restriction:
  <https://docs.aws.amazon.com/acm/latest/userguide/acm-acme-issuance.html>
- Amazon Route 53 pricing,
  hosted zones and free alias queries to CloudFront:
  <https://aws.amazon.com/route53/pricing/>
- Public Suffix List,
  which contains `cloudfront.net`:
  <https://publicsuffix.org/list/public_suffix_list.dat>
- Njalla record documentation:
  <https://njal.la/docs/records/>
- Related:
  [`doc/troubleshooting/aws-cloudfront-mirror.md`](../troubleshooting/aws-cloudfront-mirror.md),
  [`doc/troubleshooting/cloudflare-mirror-evaluation.md`](../troubleshooting/cloudflare-mirror-evaluation.md),
  [`doc/handover/aws-cloudfront-mirror-correction.md`](aws-cloudfront-mirror-correction.md),
  [`package/ssg/aquati.cat/cloudfront/README.md`](../../package/ssg/aquati.cat/cloudfront/README.md)
