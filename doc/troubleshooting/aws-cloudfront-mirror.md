# AWS CloudFront mirror: hostname correction and Free-plan restrictions

**Original investigation**:
 2026-05-09.
**Reassessment**:
 2026-09-07.
**CAA renewal remedy**:
 2026-10-09.
 The mirror leaf became an ANAME carrying its own CAA,
 a second hostname
`mirror.amazon.aquati.cat` was added under a label that terminates the climb,
and issue 6 is retracted.
 See
[`doc/handover/acm-caa-renewal-blocked.md`](../handover/acm-caa-renewal-blocked.md).
**Hostname move**:
 2026-10-09,
 later the same day.
 The public mirror hostname moved from
`aws.aquati.cat` to `amazon.aquati.cat`,
 the label that already carried the four
Amazon CAA records and already terminated the climb for
`mirror.amazon.aquati.cat`.
`aws.aquati.cat` is retired:
 all six of its DNS records are deleted and the
name now answers `NOERROR` with no address,
 exactly as a label that never
existed does.
 Every `aws.aquati.cat` mention after this paragraph is a
historical record and keeps its original text;
 the current serving names are
`amazon.aquati.cat` and `mirror.amazon.aquati.cat`.
The same change removed `0 issue "certainly.com"` from the apex,
 so
`dig +short CAA aquati.cat` returns only `0 issue "letsencrypt.org"` again,
which is the property this document was written to protect.
 The Fastly mirror
moved to the same leaf-CAA shape at `fastly.aquati.cat`.
 Tracked as
[issue 674](https://github.com/Aquaticat/Monochromatic/issues/674).
**Subject**:
 Setting up `aws.aquati.cat` as a public CloudFront mirror of
self-hosted `aquati.cat` while keeping the apex CAA limited to
`letsencrypt.org`.
 Records issues 1 to 6 along the AWS-side configuration
path (certificate issuance,
 DNS,
 distribution creation),
 plus issue 7
(CloudFront HTTP 502).
The reassessment retracts the inference that this proves an unfinished TLS 1.3 rollout.

**Mirror status on 2026-09-07**:
Restored and verified at `2026-09-07T21:42Z`.
The live endpoint serves matching origin content through a native origin Host/SNI override.
Direct probes of `aquati.cat:443` still negotiate TLS 1.3 and reject TLS 1.2 over IPv4 and IPv6 with `aquati.cat` SNI.
Authenticated inspection subsequently confirmed that both active CloudFront policies forward the viewer's `Host`.
The configured origin serves the page with `aquati.cat` SNI/Host,
but rejects mirror SNI and returns an empty body with mirror Host.
Removing viewer Host from both policies restored matching content on a disposable distribution.
The live Free plan rejected its custom cache policy.
The native origin Host/SNI override passed independent fixture validation and was then deployed to production.
Only the viewer-request function association changed;
both original policies and the active Free subscription remain unchanged.
See issue 7 for evidence,
omitted alternatives,
and verification limits.

## Background

`aquati.cat` is self-hosted on Hetzner using Caddy.
 The apex CAA record is:

```text
aquati.cat. IN CAA 0 issue "letsencrypt.org"
```

The goal:
 serve `aws.aquati.cat` as a public CloudFront mirror without
broadening the apex CAA to permit Amazon.
 Keeping the apex CAA narrow
preserves the property that only Let's Encrypt may issue certificates for
`aquati.cat`.

Context for priority:
 `fastly.aquati.cat` already exists as a working
mirror,
 so `aws.aquati.cat` is diversification rather than a primary
alternative path.
The original disposition was to wait.
That priority judgment does not establish an AWS rollout defect;
issue 7 records the reassessment.

The configuration path:

1. Drop cross-origin `Alt-Svc` from the apex.
2. Request a single-name ACM cert for `aws.aquati.cat`.
3. Place CAA on the subdomain to permit Amazon during issuance.
4. Remove the subdomain CAA when `aws.aquati.cat` becomes a CNAME to
   CloudFront.
5. Encounter HTTP 502 from CloudFront (issue 7).
   The authorized hostname correction subsequently restored `/` and the tested routes/resources.

This document records the issues encountered along that path.

## Issue 1: cross-origin Alt-Svc forces a multi-SAN cert

### Symptom

An initial design served `aws.aquati.cat` as the CloudFront mirror **and**
advertised it from the apex with `Alt-Svc`:

```http
HTTP/2 200 OK
alt-svc: h2="aws.aquati.cat:443"; ma=86400
```

This forces the cert at `aws.aquati.cat` to cover both `aquati.cat` and
`aws.aquati.cat`,
 which in turn requires the apex CAA to permit Amazon.
That breaks the goal of keeping the apex CAA letsencrypt-only.

### Minimal repro

```bash
aws acm request-certificate \
  --domain-name aws.aquati.cat \
  --subject-alternative-names aquati.cat \
  --validation-method DNS \
  --region us-east-1
# Validation eventually fails with a CAA-related rejection on the
# aquati.cat name (apex CAA permits letsencrypt.org only).
```

### Root cause

RFC 7838 section 2.1 requires the alternative service to present a TLS
certificate that is valid for the **original** origin,
 not just for the
alternative service host name.
 So a cross-origin `Alt-Svc` from
`aquati.cat` to `aws.aquati.cat` forces the alternative service's cert to
include `aquati.cat` as a SAN.
 ACM's CAA check then reads the apex CAA,
which permits only Let's Encrypt,
 and refuses to issue.

Citation:
<https://datatracker.ietf.org/doc/html/rfc7838#section-2.1>

### Verified solution

Drop `Alt-Svc` from the apex response.
 The CloudFront mirror is served as
its own origin (`https://aws.aquati.cat`),
 not as an alternative origin
for `aquati.cat`.
 The cert can then be a single-name cert for
`aws.aquati.cat` only,
 and the apex is no longer a CAA-relevant name
during ACM issuance.

### Verification commands

```bash
curl -sI https://aquati.cat/ | grep -i alt-svc
# Expected: empty
```

## Issue 2: CAA tree-walk reaches the apex without a subdomain CAA

### Symptom

With no CAA record at `aws.aquati.cat`,
 ACM's CAA check walks up to the
apex,
 finds `letsencrypt.org` only,
 and refuses to issue.
 The validation
status moves through `PENDING_VALIDATION` and then fails with a CAA error
referencing `aquati.cat`.

### Minimal repro

```bash
# With no CAA at aws.aquati.cat, only at the apex:
aws acm request-certificate \
  --domain-name aws.aquati.cat \
  --validation-method DNS \
  --region us-east-1
# Issuance fails: the CAA record on aquati.cat does not authorize
# amazon.com (or any AWS issuer domain) to issue a certificate.
```

### Root cause

RFC 8659 section 3 specifies the CAA tree-walk.
 Starting from the queried
name,
 the resolver requests CAA records;
 if the set is empty,
 it walks up
to the parent label and repeats;
 the walk terminates at the first label
that has a non-empty CAA record set,
 or at the DNS root.

For `aws.aquati.cat`,
 the walk starts at `aws.aquati.cat` (no CAA),
 moves
to `aquati.cat` (apex CAA `letsencrypt.org` only),
 and stops there.
Amazon is not in the apex set,
 so ACM refuses.

Citation:
<https://datatracker.ietf.org/doc/html/rfc8659#section-3>

### Verified solution

Place a CAA record at `aws.aquati.cat` that permits Amazon.
 The walk now
terminates one level below the apex and never reads the apex CAA.
 The
apex CAA stays untouched.

```text
aws.aquati.cat. 300 IN CAA 0 issue "amazon.com"
```

For full AWS CA coverage,
 AWS documents four issuer domains:
`amazon.com`,
 `amazontrust.com`,
 `awstrust.com`,
 `amazonaws.com`.
`amazon.com` alone is sufficient for ACM-issued certs in practice,
 but
adding all four is safer if AWS rotates the issuing CA.

### Verification commands

```bash
dig +short CAA aws.aquati.cat
# Expected: 0 issue "amazon.com"

dig +short CAA aquati.cat
# Expected: 0 issue "letsencrypt.org" (unchanged)
```

## Issue 3: CloudFront rejects EC_secp384r1 certs with a misleading error

### Symptom

Requesting an ACM cert with `--key-algorithm EC_secp384r1` succeeds.
 The
cert is issued and validated normally.
 Attaching it to a new CloudFront
distribution then fails with:

```text
An error occurred (InvalidViewerCertificate) when calling the
CreateDistribution operation: The specified SSL certificate doesn't
exist, isn't in us-east-1 region, isn't valid, or doesn't include a
valid certificate chain.
```

The error string lists four conditions,
 none of which apply:
 the cert
exists,
 it is in `us-east-1`,
 it is valid (`Status: ISSUED`),
 and its
chain is complete.
 The actual cause (an unsupported key algorithm) is
not mentioned anywhere in the error.

### Minimal repro

```bash
aws acm request-certificate \
  --domain-name aws.aquati.cat \
  --validation-method DNS \
  --key-algorithm EC_secp384r1 \
  --region us-east-1

# After validation completes and Status is ISSUED:

aws cloudfront create-distribution --distribution-config '{
  "ViewerCertificate": {
    "ACMCertificateArn": "<arn>",
    "SSLSupportMethod": "sni-only",
    "MinimumProtocolVersion": "TLSv1.2_2021"
  }
}'
# Fails with InvalidViewerCertificate as quoted above.
```

### Root cause

CloudFront supports a restricted set of public key algorithms.
 Per the
AWS documentation,
 the supported algorithms are RSA at 1024,
 2048,
 3072,
or 4096 bits,
 plus ECDSA `prime256v1` (P-256).
 `EC_secp384r1` (P-384) is
not on the supported list.

The `InvalidViewerCertificate` error message does not mention algorithm,
which makes the diagnosis non-obvious;
 the message reads as if the ARN
were wrong,
 the region were wrong,
 or the chain were broken.

Citation:
<https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/cnames-and-https-requirements.html#https-requirements-size-of-public-key>

### Verified solution

Re-request the cert with a supported algorithm.
 `EC_prime256v1` is the
closest equivalent:

```bash
aws acm request-certificate \
  --domain-name aws.aquati.cat \
  --validation-method DNS \
  --key-algorithm EC_prime256v1 \
  --region us-east-1
```

`RSA_2048` is the alternative if EC is not desired.

### Verification commands

```bash
aws acm describe-certificate \
  --certificate-arn <new-arn> \
  --region us-east-1 \
  --query 'Certificate.KeyAlgorithm'
# Expected: "EC-prime256v1"

aws cloudfront create-distribution --distribution-config '{...}'
# Expected: success, no InvalidViewerCertificate.
```

## Issue 4: ACM validation CNAME tokens are deterministic per (account, domain)

### Symptom

After replacing the rejected `EC_secp384r1` cert (issue 3),
 the next
expectation is that a fresh validation CNAME is needed for the new
request.
 The existing validation record from the previous (failed)
request is still in DNS.

### Observation

`describe-certificate` for the new request returns the **same**
validation CNAME name and value as the old request.
 The pre-existing
record validates the new request immediately,
 with no DNS edits.

### Minimal repro

```bash
# After the first (failed) request:
aws acm describe-certificate --certificate-arn <old-arn> \
  --region us-east-1 \
  --query 'Certificate.DomainValidationOptions[0].ResourceRecord'

# After the replacement request:
aws acm describe-certificate --certificate-arn <new-arn> \
  --region us-east-1 \
  --query 'Certificate.DomainValidationOptions[0].ResourceRecord'

# Output: identical Name and Value across both requests.
```

### Root cause

ACM derives the validation CNAME deterministically from the AWS account
ID and the domain name.
 The same `(account, domain)` tuple always yields
the same `_<token>.<domain>` CNAME pointing to
`_<value>.<token>.acm-validations.aws.`,
 regardless of the cert's key
algorithm,
 validity period,
 or whether a previous request was abandoned.

This is not called out in a single canonical AWS doc,
 but it is
consistent across cert reissuance,
 deletion-and-recreation,
 and
algorithm changes for the same domain in the same account.
 Confirmed
empirically here.

### Verified solution

When re-requesting an ACM cert for the same domain in the same account,
leave the validation CNAME in place.
 ACM picks it up on the next
describe poll and validates the replacement request without any DNS
change.

### Verification commands

```bash
# Confirm the CNAME stayed put across the re-request:
dig +short CNAME _<token>.aws.aquati.cat
# Expected: _<value>.<token>.acm-validations.aws.

# Confirm the new cert moved to ISSUED without a DNS edit:
aws acm describe-certificate --certificate-arn <new-arn> \
  --region us-east-1 \
  --query 'Certificate.Status'
# Expected: "ISSUED"
```

## Issue 5: CAA-vs-CNAME conflict at the same DNS name

### Symptom

After the cert is issued,
 the next step is to point `aws.aquati.cat` at
the CloudFront distribution via a CNAME.
 Adding the CNAME at the
registrar (Njalla) fails:

```text
Cannot add CNAME: aws.aquati.cat already has a CAA record. CNAME
records cannot coexist with other record types at the same name.
```

### Minimal repro

```text
# Existing record (from issue 2):
aws.aquati.cat. IN CAA 0 issue "amazon.com"

# Attempted addition:
aws.aquati.cat. IN CNAME <distribution>.cloudfront.net.

# Registrar refuses; resolvers that ignored the registrar's enforcement
# would produce undefined behavior at query time.
```

### Root cause

RFC 1912 section 2.4 states plainly:
 a CNAME record is not allowed to
coexist with any other data.
 RFC 2181 section 10.5.1 narrows the
exception to DNSSEC sibling records (the `SIG`/`NXT` family in the
original text,
 with later equivalents).
 CAA is not on that exception
list.

The CAA record placed at `aws.aquati.cat` in issue 2 therefore blocks
the CNAME add.
 Njalla enforces this at the registrar layer.

Citations:

- <https://datatracker.ietf.org/doc/html/rfc1912#section-2.4>
- <https://datatracker.ietf.org/doc/html/rfc2181#section-10.5.1>

### Verified solution

Remove the CAA record at `aws.aquati.cat` before adding the CNAME:

1. Delete `aws.aquati.cat. IN CAA 0 issue "amazon.com"`.
2. Add `aws.aquati.cat. IN CNAME <distribution>.cloudfront.net.`.

**Better solution,
 measured 2026-10-09**:
 the conflict only exists for a
real CNAME.
Njalla's `ANAME` type is a flattening alias that answers A and AAAA with
no CNAME on the wire,
and it does coexist with CAA at the same name.
Keeping the alias and the authorization at the leaf means the RFC 8659
climb terminates there and the apex is never read,
so nothing has to be removed and later restored.
Njalla states the CNAME restriction verbatim when the conflict is real:

```text
add-record rejected: code 400 You can not have both CNAME and CAA records.
```

The trade is that a flattened alias publishes the addresses the provider
resolved rather than the querier's nearest edge,
and Njalla documents no re-resolution interval.
See option 7 in
[`doc/handover/acm-caa-renewal-blocked.md`](../handover/acm-caa-renewal-blocked.md).

### Verification commands

```bash
dig +short CAA aws.aquati.cat
# Expected: empty

dig +short CNAME aws.aquati.cat
# Expected: <distribution>.cloudfront.net.
```

## Issue 6: removing the subdomain CAA stays safe because ACM follows CNAMEs

**Retracted on 2026-10-09.**
The mechanism below is wrong,
and relying on it is what let a renewal run into expiry.
RFC 8659 section 3 climbs the parents of the queried FQDN;
only the single-node CAA query chases the alias.
Section 7 states this replaced the RFC 6844 algorithm that climbed
CNAME and DNAME chains.
ACM's renewal refusal for `aws.aquati.cat` on 2026-10-09 is the measured
confirmation.
See [`doc/handover/acm-caa-renewal-blocked.md`](../handover/acm-caa-renewal-blocked.md).
The text below is kept as the record of what was believed.

### Concern

Removing the CAA at `aws.aquati.cat` (issue 5) appears to re-expose the
original problem from issue 2:
 with no CAA at the subdomain,
 the CAA
tree-walk would reach the apex,
 hit `letsencrypt.org` only,
 and block
ACM's renewal of the cert.

### Why it stays safe

RFC 8659 section 3 requires CAs to follow CNAME aliases during CAA
processing.
 When the queried name has no CAA records but does have a
CNAME,
 the resolver follows the alias and walks the CAA chain from the
**target** of the CNAME,
 not from the parent of the original name.

After `aws.aquati.cat` becomes a CNAME,
 the CAA processing chain at
ACM renewal time is:

1. `aws.aquati.cat`:
    no CAA,
    but a CNAME is present,
    so follow the
   alias.
2. `<distribution>.cloudfront.net`:
    no CAA,
    walk up.
3. `cloudfront.net`:
    no CAA,
    walk up.
4. `net`:
    no CAA,
    walk up.
5. (root):
    no CAA.

No restrictive CAA is encountered along the chain.
 ACM is permitted,
and renewal works.

The apex CAA at `aquati.cat` is never read during this process.
 Once
the CNAME is followed,
 the walk continues up the **target's** parent
chain (`cloudfront.net`,
 `net`,
 root),
 not back into the original tree.

Citation:
<https://datatracker.ietf.org/doc/html/rfc8659#section-3>

### Verification commands

```bash
# After cert renewal completes, confirm ACM kept the cert valid:
aws acm describe-certificate --certificate-arn <arn> \
  --region us-east-1 \
  --query 'Certificate.Status'
# Expected: "ISSUED"

# Confirm no CAA blocks anywhere on the resolution chain:
dig +short CAA aws.aquati.cat
# Expected: empty (CNAME present, queried name has no CAA)

target=$(dig +short CNAME aws.aquati.cat | sed 's/\.$//')
dig +short CAA "$target"
# Expected: empty

dig +short CAA cloudfront.net
# Expected: empty
```

## Issue 7: CloudFront HTTP 502 does not establish absent TLS 1.3 origin support

### Symptom

After completing issues 1 to 6,
 the CloudFront distribution `<distribution-id>`
deploys cleanly.
 The first end-to-end test against `https://aws.aquati.cat/`
returns HTTP 502 from CloudFront:

```bash
curl -sI https://aws.aquati.cat/
# HTTP/2 502
# x-cache: Error from cloudfront
# via: 1.1 fa01d52e12da475687f2d2f66a3af028.cloudfront.net (CloudFront)
# x-amz-cf-pop: YTO53-P2
```

The original investigation attributed this to a TLS version mismatch.
Direct OpenSSL probes establish how `aquati.cat:443` responds to those inputs,
not CloudFront's actual handshake or current origin configuration:

```bash
openssl s_client -connect aquati.cat:443 -servername aquati.cat \
  -tls1_2 -brief </dev/null
# error:0A00042E:SSL routines:ssl3_read_bytes:tlsv1 alert protocol version
# Alert 70 ("protocol_version"): server refused TLS 1.2.

openssl s_client -connect aquati.cat:443 -servername aquati.cat \
  -tls1_3 -brief </dev/null
# CONNECTION ESTABLISHED
# Protocol version: TLSv1.3
```

### Root cause: viewer-host forwarding is incompatible with this origin

The 2026-05-09 inference was too strong.
The [AWS announcement][origin-tls-announcement],
dated 2025-11-20,
says:

> TLS 1.3 support is automatically enabled for all origin types,
> including custom origins,
> Amazon S3,
> and Application Load Balancers,
> with no configuration changes required on your part.

AWS describes CloudFront automatically negotiating TLS 1.3 with supporting origins.
The current [API reference][origin-protocol-api] still lists
`SSLv3 | TLSv1 | TLSv1.1 | TLSv1.2`.
That proves the documented configuration vocabulary,
not the maximum version in CloudFront's ClientHello.
Neither document resolves what happened on this distribution's connection.
The linked [origin cipher guide][origin-ciphers] still lacks TLS 1.3 cipher names;
the documentation is not a substitute for a correlated handshake capture.

The original evidence also mixed incompatible timelines and boundaries:

- The cited 2025-11-04 comment predates the 2025-11-20 announcement.
- A viewer-certificate policy failure does not establish an origin-protocol failure.
- A generic HTTP 502 is compatible with multiple causes in
  [AWS's troubleshooting guide][cloudfront-502],
  including certificate-name mismatch,
  incomplete chains,
  DNS,
  and origin connectivity.

[Caddy issue #7445][caddy-cloudfront-issue] contains a separate report of CloudFront sending
an unexpected SNI name and Caddy logging `no certificate available`.
It is a hypothesis source,
not proof that this distribution has the same cause.
No CloudFront implementation source or origin-side trace was available in this reassessment.
No claim about the deployed Caddy version's internal call chain is made.
The public-precedent source check separately identifies the diagnostic in current CertMagic source.

### Authenticated follow-up on 2026-09-07

The user restored AWS CLI authentication.
The following read-only calls succeeded:

```bash
# Distribution details were queried with secret-bearing header values omitted.
aws cloudfront get-distribution --id <distribution-id> \
  --query '{ETag:ETag,Status:Distribution.Status,LastModifiedTime:Distribution.LastModifiedTime}' \
  --output json --no-cli-pager
aws cloudfront get-origin-request-policy --id 216adef6-5c7f-47e4-b989-5492eafa07d3 \
  --output json --no-cli-pager
aws cloudfront get-cache-policy --id 4cc15a8a-d715-48a4-82b8-cc0b614638fe \
  --output json --no-cli-pager
aws cloudfront get-origin-request-policy --id b689b0a8-53d0-40ab-baf2-68738e2966ac \
  --output json --no-cli-pager
```

The distribution read returned:

- `Status: Deployed`,
  `Enabled: true`,
  ETag `<original-etag>`,
  last modified `2026-05-09T05:36:01.600000+00:00`.
- Origin `aquati.cat`,
  HTTPS port 443,
  `https-only`,
  `OriginSslProtocols: ["TLSv1.2"]`,
  `IpAddressType: dualstack`.
- No alternate cache behaviors,
  edge-function associations,
  custom origin headers,
  custom error responses,
  or enabled Origin Shield.
- Legacy distribution logging disabled.
  This field alone does not establish whether separately configured standard logging v2 exists.
- Viewer certificate policy `TLSv1.3_2025`,
  independently disproving the historical claim that this distribution could not accept that viewer setting.

Both active policies independently forward the viewer hostname:

- Origin-request policy `Managed-AllViewer`
  (`216adef6-5c7f-47e4-b989-5492eafa07d3`)
  has `HeadersConfig.HeaderBehavior: allViewer`.
- Cache policy `UseOriginCacheControlHeaders-QueryStrings`
  (`4cc15a8a-d715-48a4-82b8-cc0b614638fe`)
  includes `host` in its header allowlist,
  alongside `x-method-override`,
  `origin`,
  `x-http-method`,
  and `x-http-method-override`.

The cache-policy API returns the enclosing field name
`ParametersInCacheKeyAndForwardedToOrigin`.
AWS's [policy interaction guide][policy-interaction] explicitly says:

> The cache policy allow list overrides the origin request policy block list.

AWS names the cache policy as the rule that wins:
a header selected there still reaches the origin even if the origin-request policy excludes it.
Consequently,
changing only to `Managed-AllViewerExceptHostHeader` would not remove viewer `Host`
from this distribution's effective forwarding configuration.
The retrieved replacement policy has `HeaderBehavior: allExcept` and `Headers.Items: ["host"]`.

The [custom-origin request guide][custom-origin-requests] documents replacing `Host` with the origin domain
when viewer Host is not selected for forwarding.
Its encryption section explicitly includes TLS 1.3;
the TLS 1.3 evidence is therefore not limited to the announcement.
AWS's [load-balancer troubleshooting note][origin-sni-note] links Host-based caching with origin SNI
and certificate selection.
That is corroborating documentation,
not a captured ClientHello from this distribution.

#### Controlled origin results

Both IPv4 and IPv6 yielded these results with TLS pinned to 1.3 and HTTP pinned to 1.1:

- SNI `aquati.cat`,
  HTTP Host `aquati.cat`:
  GET `/` returned HTTP 200 and 3,445 body bytes.
- SNI `aquati.cat`,
  HTTP Host `aws.aquati.cat`:
  GET `/` returned HTTP 200 and zero body bytes.
- SNI and HTTP Host `aws.aquati.cat`,
  connection forced directly to the origin address:
  TLS handshake failed with curl exit 35 and `tlsv1 alert internal error`.
  The corresponding OpenSSL probe recorded alert 80.

Representative IPv4 invocations:

```bash
# Working origin-name control; output: HTTP 200, body 3445 bytes.
curl --silent --show-error --ipv4 --http1.1 --tlsv1.3 --tls-max 1.3 \
  --output /dev/null --write-out 'HTTP %{http_code}, body %{size_download} bytes\n' \
  --connect-timeout 15 --max-time 30 https://aquati.cat/

# Only HTTP Host changes; output: HTTP 200, body 0 bytes.
curl --silent --show-error --ipv4 --http1.1 --tlsv1.3 --tls-max 1.3 \
  --header 'Host: aws.aquati.cat' \
  --output /dev/null --write-out 'HTTP %{http_code}, body %{size_download} bytes\n' \
  --connect-timeout 15 --max-time 30 https://aquati.cat/

# Mirror SNI at the origin address fails during TLS.
curl --silent --show-error --head --http1.1 --tlsv1.3 --tls-max 1.3 \
  --resolve aws.aquati.cat:443:135.181.104.96 \
  --connect-timeout 15 --max-time 30 https://aws.aquati.cat/
```

The IPv6 controls used `--ipv6` instead of `--ipv4`,
and `--resolve 'aws.aquati.cat:443:[2a01:4f9:c012:34ed::1]'` for mirror SNI.
A fresh CloudFront HEAD still returned HTTP 502 at `LHR61-P6`.
The evidence establishes incompatible viewer-name delivery at the configured origin;
it does not establish which handshake CloudFront attempted or that no second defect exists.

#### Proposed correction and verification boundary

Exclude viewer `Host` from both policy contributions,
so CloudFront supplies origin Host `aquati.cat` instead.
The proposed change is:

- Use a cache policy equivalent to the current managed policy except for removing `host`.
  Preserve its TTLs,
  remaining header keys,
  cookies,
  query strings,
  and compression settings.
  Reuse an equivalent custom policy if one exists;
  otherwise a custom clone will no longer inherit future AWS-managed policy changes.
- Associate `Managed-AllViewerExceptHostHeader`
  (`b689b0a8-53d0-40ab-baf2-68738e2966ac`)
  in the same update.
- Leave origin domain,
  TLS settings,
  Caddy,
  DNS,
  and viewer certificate unchanged.

This is a proposed configuration correction,
not a verified workaround.
After action authorization,
validation should first reproduce failure on a disposable distribution with the original policies,
then change only the policy pair.
Its generated hostname differs from the live alias,
so it tests the mechanism rather than fully reproducing the production request identity.
A passing original-policy control would invalidate that reproduction.
A successful correction must deliver actual pages,
clean routes,
assets,
headers,
compression,
and browser behavior,
not merely HTTP 200.
The empty-body probe demonstrates why status alone is insufficient.
Use uncached request identities and retain original policy IDs for rollback.
No AWS resource was created or changed during this review.

### Authorized correction and Free-plan restriction

After the user authorized the targeted correction,
a disposable distribution reproduced HTTP 502 with both original policies.
Replacing them with a custom cache-policy clone minus `host` and
`Managed-AllViewerExceptHostHeader` restored matching origin content.
The comparison covered 12 routes/resources,
including language pages,
clean URLs,
CSS,
JavaScript,
font,
favicon,
manifest,
and a nonexistent file.
Status,
body SHA-256,
selected origin headers,
and decoded compressed responses matched.
The fixture used its generated hostname and default certificate,
and omitted the live WAF because AWS rejected sharing that pricing-plan resource.
Production's alias,
certificate,
and WAF remained untouched.

The subsequent live `UpdateDistribution` changed only those policy IDs using a fresh ETag,
but AWS rejected it:

```text
Distributions with the Free pricing plan can't have the following features: Custom cache policy
```

A fresh read confirmed the original live configuration was unchanged.
No pricing-plan or billing change was made.
The [plan documentation][flat-rate-plans] lists custom cache policies only for Business and Premium,
but CloudFront Functions are included in Free.

The replacement implementation uses runtime `cloudfront-js-2.0` with:

```js
cf.updateRequestOrigin({ hostHeader: 'aquati.cat', sni: 'aquati.cat' });
```

The [native helper documentation][origin-helper] explicitly defines these origin-only overrides
and inheritance of omitted origin settings.
The function returns the original viewer request,
retaining the original cache key and both policies.
Its exact uploaded artifact and rollback are recorded in
[`package/ssg/aquati.cat/cloudfront/README.md`](../../package/ssg/aquati.cat/cloudfront/README.md).
The API's virtual tests preserved requests across every configured HTTP method,
but AWS explicitly says those tests do not verify origin modification.

For independent validation,
the disposable distribution was restored to its original policies.
A fresh query returned HTTP 502 at `2026-09-07T21:25:20Z`.
Only the viewer-request function association was then added.
Real-traffic validation then passed at `2026-09-07T21:29:28Z`.
The same published artifact was detached from the fixture before live association
because flat-rate resources cannot share CloudFront Functions.
The fixture and unused custom policy were deleted;
list calls confirmed their absence.

#### Verified production resolution

The live function association was accepted at `2026-09-07T21:38:48Z`.
The deployed distribution ETag is `<etag>`.
A full configuration comparison proved that only
`DefaultCacheBehavior.FunctionAssociations` changed from the pre-task snapshot.
The comparison includes origin settings,
TLS policies,
viewer certificate,
WAF,
cache/origin-request policies,
and other behavior fields.
The pricing API still returned the original subscription as `FREE`,
`ACTIVE`,
with unchanged subscription ETag `2`.

The live content harness passed at `2026-09-07T21:42:12Z`.
Paths and resource classes:

- `/`,
  `/en`,
  `/en/about`,
  `/en/tag/accessibility`,
  `/ca`,
  `/zh`.
- Fingerprinted CSS,
  client JavaScript,
  WOFF2 font,
  SVG favicon,
  and `/manifest.webmanifest`.
- A nonexistent resource,
  matching the origin's empty HTTP 404 response.

Fresh query keys bypassed previous error objects.
Statuses,
body hashes,
and selected response headers matched the origin.
The root body contained 3,445 bytes with SHA-256
`15789f6f382e87508716b0840b9962fde1fe0324ed5ad868ec953629b52e903e`.
Additional checks covered HEAD,
compressed content parity,
and actual gzip/Brotli wire bodies decoded to the original CSS hash.
Both encodings advertised `Vary: Accept-Encoding`.
Repeated CSS requests returned Miss/Miss without origin `Cache-Control`;
no cache-hit guarantee is inferred from that observation.

Live browser verification followed the English language link,
loaded fonts/CSS/JavaScript/images/manifest,
toggled the theme through its visible label,
and followed `About Aquaticat` to its rendered heading and title.
No page errors were reported.
The browser was closed after verification.
Search remains a separately disclosed verification limit,
not a claim that the whole application was exhaustively tested.

Primary-origin TLS controls still accepted TLS 1.3 and rejected TLS 1.2 with curl exit 35 and
`alert protocol version` over IPv4 and IPv6.
The mirror delivered matching root bodies over TLS 1.3 in both address families.
No Caddy,
DNS,
certificate,
WAF,
TLS-protocol,
or pricing-plan modification was needed.
Successful correction establishes hostname handling as a sufficient remedy for the observed 502;
it does not substitute for an origin-side packet capture.

Rollback removes only the exact viewer-request function association using a fresh ETag.
The original full snapshot is a comparison reference,
not a payload to overwrite future unrelated changes.
The function source and rollback boundary live in the package's CloudFront README.

Browser search did not produce results in either the primary-site control or the fixture;
a Pagefind worker request remained pending.
This does not establish a CloudFront regression or a working search interaction.
Navigation,
loaded resources,
and theme toggling worked during the policy-fixture checks.

#### Deployment wait observation

The native `aws cloudfront wait distribution-deployed` command returned success after 1,813 seconds.
While it was still reported running,
a separate `get-distribution` read returned `Deployed`.
Its eventual success notification followed around the stop request.
That timing does not prove the waiter stalled or establish when global deployment completed.
The assertion that the wait was unnecessary was retracted.

Subsequent checks use a managed process with timestamped `GetDistribution` observations,
bounded call timeouts,
and a checkpoint after 10 observations separated by 30 seconds.
A pending checkpoint does not authorize the next mutation.
This keeps control with the agent without requiring another user prompt.
The handover records the communication gap and proposed agent-guideline addition.

### Public precedents checked on 2026-09-07

The user asked whether other people had this specific issue.
Public reports corroborate both the Caddy symptom and the policy interaction.
They do not establish an exact duplicate of our complete Caddy,
Free-plan,
policy,
and native-function combination,
or measure how prevalent it is.

#### Caddy and CloudFront with a distribution-name certificate lookup

[Caddy #7445][caddy-cloudfront-issue],
opened on 2026-01-21,
reports a Caddy origin that serves `HELLO` directly but returns CloudFront HTTP 502 through a distribution.
The reporter supplied this debug-log message on 2026-01-22:

```text
http: TLS handshake error from 1.2.3.4:123456: no certificate available for 'xxxxxxxx.cloudfront.net'
```

The reporter interpreted this as CloudFront supplying the distribution hostname in ClientHello.
Their follow-up said enabling Host passthrough changed the name to the Host-header domain,
but they had not found how to use the origin domain.
The maintainer closed the report because it did not appear to be a Caddy defect,
while leaving discussion open.

This is a close Caddy/CloudFront symptom match,
with origin-side diagnostic evidence that our investigation did not obtain.
The thread does not provide the exact cache/origin-request policy pair,
pricing plan,
or a successful tested correction.
Its reference to default behavior is the reporter's observation,
not proof of a universal CloudFront default.

#### Exact managed-cache-policy trap on AWS re:Post

In ["configure the Cloud Front to set the origin request header"][host-policy-report],
the origin accepts its own hostname,
but CloudFront forwards the viewer hostname.
The author supplied CLI output confirming
`Managed-AllViewerExceptHostHeader`,
ID `b689b0a8-53d0-40ab-baf2-68738e2966ac`,
was associated with the behavior.

Commenter `Kal` explicitly names:

> Using a Cache Policy of UseOriginCacheControlHeaders-QueryStrings
> with an Origin Request policy of AllViewerExceptHostHeader,
> I am still seeing the Cloudfront Distribution Domain name being sent to my backend server as the Host name.

This matches the one-policy-only correction we identified as insufficient:
the same managed cache policy still includes `host`,
so the origin-request exclusion cannot cancel it.
It does not match our original origin-request policy,
which was `Managed-AllViewer`.
The thread records no successful correction,
Caddy version,
or plan constraint.
The generated AI answer recommending only the origin-request policy is not used as evidence of a fix.
AWS's [policy interaction guide][policy-interaction],
not the unanswered comments alone,
establishes why cache-policy inclusion wins.

#### Self-reported successful correction with a different backend

The [Fly.io discussion][fly-host-report] starts with direct-origin success and CloudFront HTTP 502.
On 2024-05-27,
the author reports successful delivery using
`Managed-AllViewerExceptHostHeader` with `Managed-CachingDisabled`,
while retaining an HTTPS origin.
They describe the viewer hostname causing a certificate-name mismatch at the backend.

This corroborates the failure family and a reported working policy combination.
It is not a controlled reproduction of our incident:
the backend is not Caddy,
the serving configuration also changed during troubleshooting,
and disabling caching would not preserve our cache semantics.
Its TLS 1.2 configuration is not a recommendation for this TLS-1.3-only origin.

#### Host-derived SNI can also be intentional

The accepted answer in ["Cloud Front To ALB SNI"][alb-sni-report] explains how forwarding viewer Host
makes origin SNI use that viewer hostname when the load balancer has its certificate.
The asker replies that it worked.
This is community corroboration of the mechanism in the opposite direction,
not another matching failure or formal service documentation.
The answer's linked AWS whitepaper currently redirects to a general security page,
so that redirected page is not treated as verification of the specific claim.

#### Source check for the certificate diagnostic

Read-only source inspection used [CertMagic][certmagic-source] commit
`31be911f5425b80393154a94a21403c9c0029910`.
The clone's origin was verified and no source was changed.
This is a current-source explanation of the diagnostic,
not identification of the reporter's or our deployed version.

`handshake.go:292` obtains the name used by the handshake certificate path:

```go
// certmagic/handshake.go:292
name, err := cfg.getNameFromClientHello(hello)
```

For a nonempty TLS server name,
`handshake.go:904` normalizes and returns `hello.ServerName`:

```go
// certmagic/handshake.go:904
name, err := idna.Lookup.ToASCII(strings.TrimSpace(hello.ServerName))
if err != nil {
	return "", err
}
if name != "" {
	return name, nil
}
```

The unsuccessful certificate path in `handshake.go:419` emits:

```go
// certmagic/handshake.go:419
return Certificate{}, fmt.Errorf("no certificate available for '%s'", name)
```

The empty-SNI branch at `handshake.go:911` can instead use a configured default name or local address:

```go
// certmagic/handshake.go:911
if cfg.DefaultServerName != "" {
	return normalizedName(cfg.DefaultServerName), nil
}
return localIPFromConn(hello.Conn), nil
```

Consequently,
this diagnostic is relevant to certificate selection during TLS,
but its text is not universally a verbatim SNI capture.
The public reporter's SNI interpretation stays attributed to that reporter.

#### Search scope and conclusion

Queries covered Caddy/CloudFront SNI and certificate failures,
the exact managed cache policy,
`AllViewerExceptHostHeader`,
Free-plan/custom-policy restrictions,
and `updateRequestOrigin` with `hostHeader` and `sni`.
Comparable searches included Fly.io and ALB rather than treating a narrow Caddy query as exhaustive.
The Caddy issue was read with all comments;
the re:Post and Fly.io pages were read including the follow-up discussion.

Conclusion:
this is a documented policy interaction with public reports of matching failure components,
not a unique networking anomaly or an established CloudFront implementation bug.
No public report found in these searches reproduces every constraint and the final native-function remedy together.
The issue 7 upstream-filing decision remains unchanged:
no implementation defect is established,
and no upstream issue or comment was posted.

### Omitted avenues and their tradeoffs

This is an architecture review,
not a completed vendor selection or authorization to deploy.

#### Diagnose the existing origin path without lowering TLS

- Benefit:
  a hostname,
  certificate,
  or routing correction could preserve the current architecture and TLS policy.
- Cost:
  authenticated inspection is complete,
  but a controlled CloudFront correction and correlated origin evidence are outstanding;
  an additional CloudFront interoperability defect remains possible.
- Verification:
  test the policy-pair correction described in "Authenticated follow-up on 2026-09-07".
  Correlate a failing request with CloudFront detailed results and the origin's received
  SNI,
  supported TLS versions,
  certificate selection,
  and network outcome.

AWS [documents][origin-request-policies] an existing mechanism that replaces the viewer's `Host`
with the origin domain when the viewer `Host` is excluded.
This establishes a configuration avenue,
not that attaching `AllViewerExceptHostHeader` alone fixes this site.
The authenticated cache-policy read proves why that change alone is insufficient.
Broad forwarding of other viewer data is a separate static-site policy question,
not part of the proposed hostname correction.

#### Publish a separate static-artifact replica

The issue assumes a mirror must fetch from the live Caddy origin.
The repository's site is generated into static files:
see [`package/ssg/aquati.cat/README.md`](../../package/ssg/aquati.cat/README.md).
Its serving boundary in
[`package/ssg/aquati.cat/Caddyfile:2`](../../package/ssg/aquati.cat/Caddyfile)
is:

```caddyfile
# package/ssg/aquati.cat/Caddyfile:2
root * dist
try_files {path} {path}.html {path}/index.html
file_server
```

Publish the same release to a separate origin for CloudFront rather than retrieving it from Caddy.
A concrete AWS-native candidate is a private S3 bucket origin with origin access control (OAC).
[AWS documents][s3-oac] a bucket-policy distribution-ARN condition that scopes access,
and HTTPS origin requests with OAC's `always` signing setting.
The REST origin does not supply Caddy's `try_files` behavior;
key layout or edge rewriting would need validation.
The S3 website endpoint is a different mechanism:
[it does not support origin HTTPS][origin-https] and does not support OAC.

- Benefit:
  removes the CloudFront-to-Caddy handshake from the serving path;
  could keep the AWS copy available when the primary host fails.
  This design need not change the primary site's Caddy configuration or TLS policy.
- Cost:
  adds artifact publication,
  freshness monitoring,
  rollback,
  and serving-policy parity work.
  Shared source,
  build,
  DNS,
  and publication-credential dependencies need separate analysis;
  their isolation depends on the design.
- Verification before selection:
  prove self-contained assets and runtime requests,
  clean URLs,
  directory indexes,
  missing-file status,
  redirects,
  MIME types,
  compression,
  cache/security headers,
  complete-release publication,
  and continued serving without the primary origin.
  OAC's HTTPS guarantee alone is not evidence that every connection negotiated TLS 1.3.

This is the strongest omitted architectural direction for origin independence,
not a validated recommendation to adopt S3.
No bucket or distribution was created or modified.

#### Dedicated compatibility endpoint

A separate origin hostname or listener could isolate a TLS compatibility policy from the public apex.
The original document named this option;
issue #146's compressed choice set omitted it.

- Benefit:
  can leave the apex's TLS-1.3-only handshake unchanged and retain live content delivery.
- Cost:
  a TLS 1.2 leg still exists if that is the required compatibility concession;
  this fails a requirement forbidding TLS 1.2 everywhere.
  Another endpoint needs certificate,
  access-control,
  and routing verification.
  An AWS address allowlist alone does not authenticate this particular distribution.
- Status:
  not deployed or verified;
  no reason to build it before establishing the current failure's cause.

#### Chain CloudFront through Fastly

A candidate topology is `CloudFront -> Fastly -> Caddy`.
Fastly's viewer endpoint returning 200 does not prove CloudFront-origin compatibility.

- Benefit:
  reuses an existing delivery path without changing the primary origin's TLS policy.
- Cost:
  makes the AWS endpoint depend on Fastly,
  defeating independence from that CDN;
  adds cache,
  invalidation,
  Host/SNI,
  and incident-attribution interactions.
- Status:
  untested candidate,
  not a diversification recommendation.

#### Priority ranking

Investigate the existing path first,
then evaluate independent artifact replication.
Diagnosis precedes redesign because no protocol defect has been established;
replication follows because it addresses the shared-origin dependency diagnosis alone cannot remove.
These are complementary steps,
not exclusive choices.

For further evaluation toward restoring the AWS endpoint:
artifact replica > dedicated compatibility endpoint > CDN chaining.
Replication could also remove the live primary dependency;
a compatibility endpoint preserves it but avoids dependence on Fastly;
chaining adds that CDN dependency.
If independence from the primary origin is mandatory,
only replication among these candidates qualifies.
Under a strict prohibition on any TLS 1.2 leg,
a compatibility endpoint requiring that concession is excluded.
This is an investigation ranking,
not a validated deployment selection.

### Workaround verification status

The policy-pair correction is verified on the disposable distribution,
but production rejects its custom cache policy under the Free plan.
The native function substitution is verified on the fixture and live endpoint without replacing either policy.
The original heading incorrectly called unshipped configuration sketches verified workarounds.
The alternatives in "Omitted avenues and their tradeoffs" remain proposals.
Global TLS relaxation is outside issue #146's scope;
waiting remains an operational choice,
not a demonstrated technical remedy.

### Verification on 2026-09-07

Read-only probes ran from `/var/home/user/Monochromatic`.
CloudFront is an unversioned managed service;
the deployed Caddy version was not obtained.
DNS returned `135.181.104.96` and `2a01:4f9:c012:34ed::1`.
The commands without `-4` selected that IPv6 address.

Working catalog:

```bash
# Origin TLS 1.3 succeeds with aquati.cat SNI over both address families.
openssl s_client -connect aquati.cat:443 -servername aquati.cat -tls1_3 -brief </dev/null
openssl s_client -4 -connect aquati.cat:443 -servername aquati.cat -tls1_3 -brief </dev/null
# Protocol version: TLSv1.3
# Ciphersuite: TLS_AES_128_GCM_SHA256
# Verification: OK

curl --silent --show-error --head --connect-timeout 15 --max-time 30 https://fastly.aquati.cat/
# HTTP/2 200
# server: Caddy
```

Failing catalog,
separated by signal:

```bash
# Origin rejects TLS 1.2 over both address families.
openssl s_client -connect aquati.cat:443 -servername aquati.cat -tls1_2 -brief </dev/null
openssl s_client -4 -connect aquati.cat:443 -servername aquati.cat -tls1_2 -brief </dev/null
# tlsv1 alert protocol version; SSL alert number 70; exit 1

# Keeping IPv4 and TLS 1.3 unchanged, changing only SNI fails.
openssl s_client -4 -connect aquati.cat:443 -servername aws.aquati.cat -tls1_3 -brief </dev/null
openssl s_client -4 -connect aquati.cat:443 -servername <distribution-domain> -tls1_3 -brief </dev/null
# tlsv1 alert internal error; SSL alert number 80; exit 1

curl --silent --show-error --head --connect-timeout 15 --max-time 30 https://aws.aquati.cat/
# HTTP/2 502
# x-cache: Error from cloudfront
# x-amz-cf-pop: LHR61-P6
```

The SNI positive control succeeds with `aquati.cat` and fails with the other names.
This proves name-sensitive behavior at this origin,
not that CloudFront sent either failing name.
A separate GET also returned the generic CloudFront 502 page.

Initial evidence acquisition limits,
with authentication subsequently restored:

- `aws cloudfront get-distribution-config --id <distribution-id>` with a field-filtered query failed:
  `aws: [ERROR]: Your session has expired. Please reauthenticate using 'aws login'.`
  Exit status 255;
  `aws configure list-profiles` listed only `default`.
  No active distribution settings were inferred from this failure.
  The authenticated follow-up resolved this access blocker and records actual settings.
- A read-only SSH attempt to obtain Caddy journal entries used batch authentication and strict host-key checking.
  It failed with `Host key verification failed.` because no trusted ED25519 key was known for `aquati.cat`.
  Host verification was not bypassed;
  origin logs were not read.
- No production mutations,
  packet captures,
  candidate deployments,
  or builds were performed.

The original `update-distribution` verification example was removed:
it is a mutation API,
not a read-only capability probe.
Its historical rejection does not make future execution safe.

## Current configuration state

**Status**:
mirror restoration verified on 2026-09-07.
The deployed function overrides origin HTTP Host and TLS SNI to `aquati.cat`.
Both original policies remain associated,
and all other distribution fields match the original snapshot.
The live Free plan remains active.
The disposable distribution and unused custom cache policy are deleted.
The remaining certificate and DNS inventory records the original investigation
unless explicitly corroborated in the authenticated follow-up.

Concrete identifiers recorded on 2026-05-09,
redacted on 2026-10-09 because this repository is public.
Each placeholder is recoverable from the account with one read-only call:

- AWS account:
   `<account-id>`
  (`aws sts get-caller-identity --query Account`).
- ACM certificate ARN:
   `<certificate-arn>`
  (`aws acm list-certificates --region us-east-1`).
- CloudFront distribution ID:
   `<distribution-id>`
  (`aws cloudfront list-distributions`).
- CloudFront distribution domain:
   `<distribution-domain>`
  (`aws cloudfront get-distribution --id <distribution-id> --query 'Distribution.DomainName'`,
   also public as the `amazon.aquati.cat` ANAME target;
   before the
  2026-10-09 hostname move it was the `aws.aquati.cat` CNAME,
   then ANAME,
   target).

DNS records,
 measured on 2026-10-09 after the hostname move:

- `aquati.cat. IN CAA 0 issue "letsencrypt.org"` (apex).
   This is the only
  CAA record at the apex.
   The `0 issue "certainly.com"` record that sat
  beside it was drift from the starting state this document describes as
  letsencrypt-only,
   and was deleted on 2026-10-09 once the Fastly mirror
  carried its own CAA at `fastly.aquati.cat`.
- `amazon.aquati.cat. IN ANAME <distribution-domain>.`
   Added on 2026-10-09
  when this name became the serving hostname.
   It is a flattened alias
  rather than a CNAME so the node can carry CAA beside it.
- `amazon.aquati.cat. IN CAA 0 issue "amazon.com"`,
   plus the same record for
  `amazontrust.com`,
   `awstrust.com`,
   and `amazonaws.com`.
   These four
  already existed as the climb-terminating label for
  `mirror.amazon.aquati.cat`,
   and they are what now let ACM renew this name
  without touching the apex.
- `_<validation-token>.amazon.aquati.cat. IN CNAME _<validation-value>.<validation-zone>.acm-validations.aws.`
  (kept permanently for renewal;
   deleting this record breaks the next
  ACM renewal cycle.
   Recover both halves with
  `aws acm describe-certificate --certificate-arn <certificate-arn> --region us-east-1 --query 'Certificate.DomainValidationOptions[?DomainName==`amazon.aquati.cat`].ResourceRecord'`).
- `mirror.amazon.aquati.cat. IN CNAME <distribution-domain>.`
   A second mirror hostname added on 2026-10-09.
   It is a real CNAME,
   so CloudFront keeps per-querier edge selection for it.
   Its climb
  terminates at `amazon.aquati.cat`.
- `_<validation-token>.mirror.amazon.aquati.cat. IN CNAME _<validation-value>.<validation-zone>.acm-validations.aws.`
  (also kept permanently for renewal;
   deleting either validation record
  breaks renewal of the two-name certificate).
- `aws.aquati.cat`:
   no records.
   The ANAME,
   the four CAA records,
   and the
  validation CNAME were deleted on 2026-10-09 after the distribution stopped
  listing the name as an alias.

ACM cert,
 current as of 2026-10-09 after the hostname move:

- Region:
   `us-east-1` (mandatory for CloudFront viewer certs).
- Names:
   `amazon.aquati.cat` and `mirror.amazon.aquati.cat`.
   One certificate covers both because a distribution holds exactly one
  viewer certificate.
- Key algorithm:
   `EC_prime256v1`.
- Validation method:
   DNS,
   one record per name,
   both of which must stay published.
- Validity:
   `notBefore` 2026-10-09,
   `notAfter` 2027-04-24.
- The single-name predecessor was renewed by ACM on 2026-10-09 once the
  leaf CAA records unblocked it,
   then deleted after the two-name
  certificate was attached and verified.
- That two-name certificate covered `aws.aquati.cat` and
  `mirror.amazon.aquati.cat`.
   The hostname move needed a third
  certificate,
   because ACM cannot add a name to an issued certificate and
  CloudFront rejects an alias its viewer certificate does not cover.
   The
  replacement was requested for both current names,
   reached `ISSUED` with
  both DNS validations `SUCCESS`,
   and was attached by one
  `UpdateDistribution` that also dropped the retired alias.
   The predecessor
  was then deleted under three assertions:
   the distribution's attached ARN
  was the replacement,
   the target reported `InUseBy: []`,
   and the target's
  subject alternative names contained `aws.aquati.cat` where the
  replacement's contained `amazon.aquati.cat`.
   `describe-certificate` on the
  deleted ARN now returns `ResourceNotFoundException`.
  Deleting the predecessor is coupled to deleting the four CAA records at
`aws.`:
 an unattached certificate whose authorization has been removed would
fail its next renewal and reproduce the incident this document records.

Properties this configuration was believed to preserve,
with a 2026-10-09 correction:

- The apex CAA is unchanged.
   Only Let's Encrypt may issue certs for
  `aquati.cat`.
   **Correction,
   2026-10-09**:
   the apex now also publishes
  `0 issue "certainly.com"`,
   so two CAs are authorized for apex-inheriting names.
   **Restored,
   2026-10-09**:
   that record is deleted.
   The Fastly mirror
  publishes `0 issue "certainly.com"` at `fastly.aquati.cat` instead,
   so the
  climb for it and for `mirror.fastly.aquati.cat` terminates there and never
  reads the apex.
   `dig +short CAA aquati.cat` again returns only
  `0 issue "letsencrypt.org"`,
   which is the property this document exists
  to protect.
- Amazon may issue certs only for `aws.aquati.cat`,
   and only because the
  CAA chain at renewal time runs through `cloudfront.net`'s empty CAA,
  not through the apex.
   **Correction,
   2026-10-09**:
   this is false and is the root cause of the
  renewal failure recorded in
  [`doc/handover/acm-caa-renewal-blocked.md`](../handover/acm-caa-renewal-blocked.md).
   `cloudfront.net` is not empty;
   it publishes `0 issuewild "amazonaws.com"` and
  `0 issuewild "digicert.com"`.
   More decisively,
   RFC 8659 section 3 climbs the parents of the queried FQDN,
   not the parents of its CNAME target,
   so ACM read the apex and refused.
   Issue 6 is retracted.

## What does not work

The following alternatives were considered and rejected:

- **Wider apex CAA**,
   e.g. adding `0 issue "amazon.com"` at
  `aquati.cat`.
   Rejected on principle:
   the goal is to keep the apex's
  attack surface narrow.
   Permitting Amazon to issue for the apex
  defeats the purpose of running a restrictive CAA at all.
   Reopened by the 2026-10-09 renewal failure:
   the apex now also publishes
  `0 issue "certainly.com"`,
   so the property this rejection protects no longer holds as measured.
   **Resolved,
   2026-10-09**:
   that record is deleted,
   the rejection stands
  again,
   and the need it was reopened for is met by leaf and intermediate
  label CAA records instead.
   Ranking and alternatives are in
  [`doc/handover/acm-caa-renewal-blocked.md`](../handover/acm-caa-renewal-blocked.md).
- **ALIAS records at the apex** pointing to CloudFront.
   Njalla supports
  only standard DNS record types;
   there is no Route 53 ALIAS or
  CNAME-flattening equivalent.
   Even if it were available,
   the apex
  would still need CAA permitting Amazon at issuance time (same dead
  end as widening the apex CAA).
   **Correction,
   2026-10-09**:
   the first half is false.
   Njalla does have a flattening alias,
   its `ANAME` type,
   already live on 43 names in this zone.
   A probe at a throwaway label confirmed an ANAME coexists with a CAA
   record at the same name and answers A and AAAA with no CNAME on the
   wire,
   and a second probe confirmed an ANAME to the CloudFront distribution
   domain serves the mirror.
   The second half still holds:
   at the apex the Relevant RRset is the apex's own,
   so an ANAME there would not avoid authorizing Amazon at the apex.
   At the mirror leaf it does avoid that,
   which is option 7 in
   [`doc/handover/acm-caa-renewal-blocked.md`](../handover/acm-caa-renewal-blocked.md).
- **Alternate label** like `cdn.aquati.cat`.
   The CAA tree-walk and
  CAA-vs-CNAME constraints apply equally to any subdomain of
  `aquati.cat`.
   The label name is irrelevant;
   the structural issues
  are identical.
   **Correction,
   2026-10-09**:
   this holds only for a single label below
  the apex.
   A two-label path breaks the walk:
   `mirror.aquati.cat` is an ordinary node,
   so it can publish `0 issue "amazon.com"`,
   and the climb for `aws.mirror.aquati.cat` terminates there without
   reading the apex.
   The leaf may still be a CNAME,
   because the CAA lives one level up.
   The cost is a hostname change,
   not a security widening.
   See
  [`doc/handover/acm-caa-renewal-blocked.md`](../handover/acm-caa-renewal-blocked.md).
- **`EC_secp384r1` cert** (P-384).
   CloudFront rejects it with the opaque
  `InvalidViewerCertificate` error documented in issue 3.
   Use
  `EC_prime256v1` (P-256) or `RSA_2048` instead.
- **`OriginSslProtocols.Items: ["TLSv1.3"]`** in the distribution config.
  The original investigation recorded `MalformedXML`;
  the current API reference still omits this value.
  This does not prove TLS 1.3 is absent from automatic negotiation (issue 7).
- **Omitting `OriginSslProtocols`** in the distribution config.
   The API
  returns `InvalidArgument: The parameter OriginSslProtocols is required.`
  The field cannot be left blank or absent.
- **`OriginProtocolPolicy: http-only`** as a way to bypass TLS to origin.
  Caddy enforces a redirect from HTTP to HTTPS,
   so CloudFront would
  receive a 308 from the origin and not retry over HTTPS.
   The redirect
  is also visible to viewers if `ViewerProtocolPolicy` allows HTTP.
- **A viewer-certificate TLS policy as an origin fix**.
  Viewer and origin connections are separate boundaries.
  A historical provider issue about the viewer policy does not diagnose this origin failure.
- **Treating HTTP 502 or the API enum as rollout evidence**.
  Neither reveals the actual CloudFront ClientHello.
  The reassessment retracts the original conclusion that these observations established an unfinished rollout.

## Why we do not file these upstream

Walked per issue against the 5-constraint upstream-filing check.
 Default
policy:
 do not file.
 The audit trail is the point.

### Issues 1, 2, 5, 6: RFC-mandated behavior

Issues 1 (RFC 7838 forces multi-SAN cert for cross-origin Alt-Svc),
 2
(RFC 8659 tree-walk reaches apex),
 5 (RFC 1912 plus RFC 2181 ban
CAA-vs-CNAME coexistence),
 6 (RFC 8659 CNAME-following keeps chain safe)
are RFC-mandated behaviors,
 not bugs.

1. **Upstream's fault?
   ** No. RFCs 7838,
    8659,
    1912,
    2181 define the
   behavior.
    ACM,
    Njalla,
    and CloudFront each implement the specs
   correctly.
2. **Can upstream fix it?
   ** Not applicable;
    the RFC is the spec.
3. **Supporting the use case?
   ** Not applicable;
    the use case is fully
   supported,
    just constrained by the specs.
4. **Will they fix it?
   ** Not applicable.
5. **Minimal-fix prototype?
   ** Not applicable.

**Decision:
 no upstream report.
** The "fix" is to understand the spec
and route around it (issue 1:
 drop Alt-Svc;
 issue 2:
 place subdomain
CAA;
 issue 5:
 remove subdomain CAA before adding CNAME;
 issue 6:
trust the CNAME-following walk).

### Issue 3: CloudFront InvalidViewerCertificate error string omits key algorithm cause

The error string lists four conditions (ARN,
 region,
 validity,
 chain),
none of which apply when the cause is an unsupported key algorithm
(`EC_secp384r1`).
 UX defect,
 not a behavior defect.

1. **Upstream's fault?
   ** Yes.
    Error message is genuinely misleading.
2. **Can upstream fix it?
   ** Yes;
    one-line change to the error string in
   the `CreateDistribution` validation path to name the unsupported
   algorithm or list the supported set.
3. **Supporting the use case?
   ** Yes.
    Single-name ACM cert + CloudFront
   distribution is a documented,
    common combination.
4. **Will they fix it?
   ** Unknown.
    AWS service teams do iterate on error
   message wording,
    but this specific message has been in the field for
   years across multiple algorithm changes (RSA → ECDSA,
    P-256
   introduction).
    Low signal that a fix is queued.
5. **Minimal-fix prototype?
   ** Not feasible;
    the validation code is
   closed-source.
    A user-facing prototype would be a Re:
   Post feedback
   item with the misleading message and the actual cause.

**Decision:
 do not file as a bug.
** Already-existing internal
diagnosis is enough;
 AWS does not provide a tracker that accepts UX
feedback at a granularity finer than Re:
Post.
 If a Re:
Post post would
help future searchers find the diagnosis,
 the draft below is the
content.
 Do not file as-is.

````md
**Title:** CloudFront `InvalidViewerCertificate` error string does not
mention unsupported key algorithm when ACM cert uses EC_secp384r1

**Symptom:**

```
An error occurred (InvalidViewerCertificate) when calling the
CreateDistribution operation: The specified SSL certificate doesn't
exist, isn't in us-east-1 region, isn't valid, or doesn't include a
valid certificate chain.
```

None of the four listed conditions applies when the actual cause is an
ECDSA P-384 key. The supported algorithms are RSA at 1024/2048/3072/4096
plus ECDSA prime256v1
(<https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/cnames-and-https-requirements.html#https-requirements-size-of-public-key>).

**Suggested fix:** Add the supported-algorithm set to the error string
when the ARN, region, validity, and chain are all valid but the key
algorithm is not in the supported set.
````

### Issue 4: ACM validation CNAME deterministic per (account, domain), docs gap

ACM derives the validation CNAME deterministically from the AWS account
ID and the domain name,
 so re-requests for the same `(account, domain)`
reuse the existing CNAME.
 This is consistent and useful,
 but not
documented in a single canonical AWS doc.

1. **Upstream's fault?
   ** Yes,
    narrowly:
    docs gap.
    Behavior is
   reasonable;
    documentation does not call it out.
2. **Can upstream fix it?
   ** Yes;
    one paragraph in the ACM DNS
   validation docs.
3. **Supporting the use case?
   ** Yes;
    re-requesting after a failed cert
   is a common path.
4. **Will they fix it?
   ** Unknown.
    Docs feedback through the page's
   feedback widget is the standard route.
5. **Minimal-fix prototype?
   ** Docs-only change;
    the prototype is the
   one-paragraph addition.

**Decision:
 low-priority docs feedback only.
** Future investigators
can find the diagnosis here;
 AWS docs feedback widget is the route if
filed.
 Do not file as a bug.

### Issue 7: upstream filing decision

The 2026-09-07 reassessment supersedes the original upstream attribution.

1. **Upstream's fault?**
   No upstream defect established.
   The active policies forward a hostname that the configured origin does not serve correctly;
   the policy-pair fixture correction restored matching content without changing TLS.
   No CloudFront handshake capture was obtained.
2. **Can upstream fix it?**
   Not assessed without an established defect;
   extending the API enum is not a proven requirement for automatic TLS 1.3.
3. **Supporting the use case?**
   Yes according to AWS's origin TLS 1.3 announcement.
4. **Would upstream welcome this contribution?**
   No contribution is proposed;
   contribution-policy auditing is deferred until there is additive defect evidence.
5. **Will upstream fix it?**
   No forecast:
   AWS describes support as already available.
   Search found the existing [AWS feature request][origin-feature-request]
   and [Caddy #7445][caddy-cloudfront-issue];
   the latter was read with all comments and describes a separate SNI incident.
6. **Minimal-fix prototype?**
   A consumer-side policy correction passed fixture testing.
   This establishes a configuration remedy,
   not an upstream defect requiring a source patch.

The `.out-of-scope/` file inventory contained no AWS or Caddy exemption.
No upstream issue or comment was filed.
There is no established upstream defect.
The public-precedent review found that the existing Caddy thread lacks our tested consumer-side correction,
so an additive-only draft is retained instead of asserting there is nothing new to share.
The `.out-of-scope/` inventory and AWS/CloudFront/Caddy/CertMagic content search were repeated before drafting;
no matching exemption was found.
Waiting for an inferred TLS rollout is not the remedy.
The user authorized the targeted hostname correction after the review;
the native-function implementation is deployed and verified on the live endpoint.

#### Additive comment draft for Caddy #7445

Do not file as-is.
This is configuration corroboration,
not a validated Caddy defect report;
contribution-policy review and authorization to post to this external repository remain outstanding.
It intentionally does not claim to reproduce the other reporter's exact distribution.

~~~md
We restored delivery for a separate Caddy/CloudFront hostname mismatch without changing Caddy or origin TLS.
Our distribution forwarded viewer Host through both `Managed-AllViewer` and the cache policy
`UseOriginCacheControlHeaders-QueryStrings`.

Changing only the origin-request policy to `AllViewerExceptHostHeader` would have left Host included by the cache policy.
AWS documents that the cache-policy allowlist overrides the origin-request blocklist:
https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/understanding-how-origin-request-policies-and-cache-policies-work-together.html

An independent test distribution recovered after removing Host from both forwarding contributions.
Our live Free plan rejected the custom cache policy needed to preserve the other cache settings.
We therefore tested a viewer-request CloudFront Function using the native origin helper:
`cf.updateRequestOrigin({ hostHeader: 'aquati.cat', sni: 'aquati.cat' })`,
returning the unchanged viewer request.
The hostname here is our configured origin,
not a value another deployment should copy.
https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/helper-functions-origin-modification.html

With both original policies restored,
the function alone changed fresh 502 responses into matching origin content on the fixture and subsequently production.
We verified routes,
assets,
statuses,
bodies,
and compression rather than relying only on a root HTTP 200.

We did not capture our original CloudFront ClientHello or obtain origin logs,
so this does not prove what SNI CloudFront sent before our correction,
nor establish that this issue's distribution has the same policy combination.

AI assistance disclosure:
an AI coding assistant performed the investigation and automated fixture/production checks and drafted this comment.
These checks are agent-executed evidence,
not a claim of independent human reproduction or source review.
~~~

## References

Sources for the reassessment were accessed on 2026-09-07.

[origin-tls-announcement]: https://aws.amazon.com/about-aws/whats-new/2025/11/amazon-cloudfront-tls13-origin/
[origin-protocol-api]: https://docs.aws.amazon.com/cloudfront/latest/APIReference/API_OriginSslProtocols.html
[origin-ciphers]:
https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/secure-connections-supported-ciphers-cloudfront-to-origin.html
[cloudfront-502]: https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/http-502-bad-gateway.html
[caddy-cloudfront-issue]: https://github.com/caddyserver/caddy/issues/7445
[origin-request-policies]:
https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/using-managed-origin-request-policies.md
[s3-oac]: https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/private-content-restricting-access-to-s3.md
[origin-https]:
https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/using-https-cloudfront-to-custom-origin.html
[origin-feature-request]: https://repost.aws/questions/QUzNusy9axTz2iWIyfK1q-nw/feature-cloudfront-origin-tls-v1-3
[policy-interaction]:
https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/understanding-how-origin-request-policies-and-cache-policies-work-together.html
[custom-origin-requests]:
https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/RequestAndResponseBehaviorCustomOrigin.md
[origin-sni-note]: https://repost.aws/knowledge-center/cloudfront-https-connection-fails
[origin-helper]:
https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/helper-functions-origin-modification.md
[flat-rate-plans]: https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/flat-rate-pricing-plan.md
[host-policy-report]:
https://repost.aws/questions/QUyKQbjgzkRYG-DI0bsKE6Mg/configure-the-cloud-front-to-set-the-origin-request-header
[fly-host-report]: https://community.fly.io/t/fly-io-apps-behind-cloudfront/20028
[alb-sni-report]: https://repost.aws/questions/QU6ztFUrqgSlOTgVFu2CM9KA/cloud-front-to-alb-sni
[certmagic-source]: https://github.com/caddyserver/certmagic

- RFC 7838,
   HTTP Alternative Services,
   section 2.1:
  <https://datatracker.ietf.org/doc/html/rfc7838#section-2.1>
- RFC 8659,
   DNS Certification Authority Authorization,
   section 3:
  <https://datatracker.ietf.org/doc/html/rfc8659#section-3>
- RFC 1912,
   Common DNS Operational and Configuration Errors,
  section 2.4:
  <https://datatracker.ietf.org/doc/html/rfc1912#section-2.4>
- RFC 2181,
   Clarifications to the DNS Specification,
   section 10.5.1:
  <https://datatracker.ietf.org/doc/html/rfc2181#section-10.5.1>
- CloudFront supported public key sizes and algorithms:
  <https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/cnames-and-https-requirements.html#https-requirements-size-of-public-key>
- AWS announcement,
   "Amazon CloudFront now supports TLS 1.3 for origin
  connections" (November 2025):
  <https://aws.amazon.com/about-aws/whats-new/2025/11/amazon-cloudfront-tls13-origin/>
- AWS API reference,
   `OriginSslProtocols`:
  <https://docs.aws.amazon.com/cloudfront/latest/APIReference/API_OriginSslProtocols.html>
- AWS re:
  Post,
   "[Feature] CloudFront Origin TLS v1.3":
  <https://repost.aws/questions/QUzNusy9axTz2iWIyfK1q-nw/feature-cloudfront-origin-tls-v1-3>
- Stack Overflow,
   "Does AWS CloudFront support TLSv1.3 for custom origin?
  ":
  <https://stackoverflow.com/questions/58209174/does-aws-cloudfront-support-tlsv1-3-for-custom-origin>
- Reddit r/aws,
   "What TLS version does AWS use for communication b/w
  cloudfront distribution and S3 origins":
  <https://www.reddit.com/r/aws/comments/133zunj/>
- Terraform AWS provider issue 43840,
   "TLS v1.3 missing from Viewer
  Certificate in Cloudfront Distribution":
  <https://github.com/hashicorp/terraform-provider-aws/issues/43840>
- AWS published IP ranges (filter by `service == "CLOUDFRONT_ORIGIN_FACING"`):
  <https://ip-ranges.amazonaws.com/ip-ranges.json>
- Caddy v2 placeholders and matchers:
  <https://caddyserver.com/docs/conventions>,
  <https://caddyserver.com/docs/caddyfile/matchers>
