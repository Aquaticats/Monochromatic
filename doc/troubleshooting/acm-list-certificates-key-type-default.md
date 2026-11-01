# AWS CLI 2.37.0 `acm list-certificates` returns an empty list for every ECDSA certificate because the documented default key-type filter is RSA only

**Date**:
 2026-10-09.
**Subject**:
 `aws acm list-certificates` in `us-east-1` returns
`{"CertificateSummaryList": []}` with HTTP 200 and no error,
in an account that holds two issued certificates,
because both were requested with `--key-algorithm EC_prime256v1`.
Discovered while identifying the mirror certificate during the work in
[`doc/handover/acm-caa-renewal-blocked.md`](../handover/acm-caa-renewal-blocked.md).

## Symptom

The inventory call reports nothing:

```bash
aws acm list-certificates --region us-east-1
# {
#     "CertificateSummaryList": []
# }
```

Naming every status changes nothing:

```bash
aws acm list-certificates --region us-east-1 \
  --certificate-statuses ISSUED PENDING_VALIDATION FAILED INACTIVE REVOKED EXPIRED VALIDATION_TIMED_OUT
# {
#     "CertificateSummaryList": []
# }
```

A certificate demonstrably exists,
because addressing it directly succeeds:

```bash
aws acm describe-certificate --region us-east-1 --certificate-arn <certificate-arn> \
  --query 'Certificate.{Status:Status,Key:KeyAlgorithm,Domain:DomainName}'
# {
#     "Status": "ISSUED",
#     "Key": "EC-prime256v1",
#     "Domain": "aws.aquati.cat"
# }
```

The same call also serves a live distribution,
so the certificate is neither expired nor hypothetical.

Two related surprises appear in the same surface:

```bash
aws acm list-certificates --region us-east-1 --certificate-key-pair-origins AMAZON_ISSUED
# aws: [ERROR]: An error occurred (ValidationException) when calling the ListCertificates operation:
# 1 validation error detected: Value of the input at 'certificateKeyPairOrigins' failed to satisfy
# constraint: Member must satisfy constraint: [Member must satisfy enum value set:
# [CUSTOMER_PROVIDED, ACME, AWS_MANAGED], Member must not be null]
```

```bash
aws acm search-certificates --region us-east-1 --query 'CertificateSummaryList[].DomainName'
# null
```

That `null` is a wrong key,
not an empty account:
`SearchCertificates` returns its hits under `Results`.

## Root cause

`ListCertificates` applies a default key-type filter that excludes every
ECDSA algorithm.
The installed CLI carries the service model that documents it,
at
`~/.local/share/mise/installs/aws/2.37.0/aws/dist/awscli/botocore/data/acm/2015-12-08/service-2.json:505`:

```json
"documentation":"<p>Retrieves a list of certificate ARNs and domain names. You can request that only certificates that match a specific status be listed. You can also filter by specific attributes of the certificate. Default filtering returns only <code>RSA_2048</code> certificates. For more information, see <a>Filters</a>.</p> <note> <p>By default, this action does not return certificates with a <code>CertificateKeyPairOrigin</code> of <code>ACME</code>. To include ACME certificates, specify <code>ACME</code> in the <code>CertificateKeyPairOrigins</code> filter.</p> </note>",
```

The `Filters.keyTypes` member states the same default with one more
algorithm and one more condition,
in the same file at line 2597:

```json
"keyTypes":{
  "shape":"KeyAlgorithmList",
  "documentation":"<p>Specify one or more algorithms that can be used to generate key pairs.</p> <p>Default filtering returns only <code>RSA_1024</code> and <code>RSA_2048</code> certificates that have at least one domain. To return other certificate types, provide the desired type signatures in a comma-separated list. For example, <code>\"keyTypes\": [\"RSA_2048\",\"RSA_4096\"]</code> returns both <code>RSA_2048</code> and <code>RSA_4096</code> certificates.</p>"
},
```

The public botocore model carries both strings at the same line numbers,
so the citation is reproducible outside this machine:
`botocore/data/acm/2015-12-08/service-2.json:505` and `:2597`
(<https://github.com/boto/botocore/blob/develop/botocore/data/acm/2015-12-08/service-2.json>).

The chain that produces the empty list:

1.  The CLI serializes `ListCertificates` with no `Includes` member,
    because the caller passed no `--includes`.
2.  The service substitutes the documented default for `Filters.keyTypes`,
    which admits only RSA algorithms.
3.  Both certificates in this account were requested with
    `--key-algorithm EC_prime256v1`.
    That choice is forced here rather than preferred:
    CloudFront rejects `EC_secp384r1` with an `InvalidViewerCertificate`
    error that never names the algorithm,
    recorded as issue 3 in
    [`aws-cloudfront-mirror.md`](aws-cloudfront-mirror.md).
4.  No certificate matches the default key-type filter,
    so the response is an empty list with HTTP 200.
    Nothing signals that a filter was applied.
5.  `DescribeCertificate` takes an ARN and applies no key-type filter,
    which is why the direct read succeeds while the inventory read does not.

The origin filter documented in the same model at line 3076 is not the
cause:

```json
"documentation":"<p>Filter the certificate list by certificate key pair origin. Specify one or more <code>CertificateKeyPairOrigin</code> values. Default filtering returns only certificates with key pair origin of <code>AWS_MANAGED</code> and <code>CUSTOMER_PROVIDED</code>.</p>"
```

An Amazon-issued certificate is `AWS_MANAGED`,
so it is already inside the default origin set.

### Two earlier readings that were wrong

The first hypothesis was an identity or permission scope problem,
because an empty list reads like a denied or misdirected call.
`aws sts get-caller-identity` disproved it:
the session is the account root identity,
`arn:aws:iam::<account-id>:root`,
so no scope narrowing was possible.

The second hypothesis was the 2026 inventory split described in the ACM
ACME documentation,
which states that ACME-issued certificates do not appear in
`ListCertificates` results by default.
That is a real default,
but it cannot explain this account,
which holds no ACME certificates.
Passing `--certificate-key-pair-origins ACME` also returned an empty list,
and passing `AWS_MANAGED` explicitly returned both certificates only once
`--includes keyTypes=EC_prime256v1` accompanied it.
Origin filtering was never the blocker;
key type was.

## Verification

Version under test:
`aws-cli/2.37.0 Python/3.14.6 Linux/7.2.8-ogc4.1.fc44.x86_64 exe/x86_64.bazzite.44`,
with the bundled model at the path cited in "Root cause".
Account state at the time:
two issued certificates,
both `EC-prime256v1`,
one attached to a CloudFront distribution and one not yet attached.

Runnable harness.
Each line prints how many certificates the call returned:

```bash
for f in "" "keyTypes=RSA_2048" "keyTypes=EC_prime256v1" "keyTypes=RSA_2048,EC_prime256v1"; do
  if [ -z "$f" ]; then
    printf '%-42s %s\n' "no filter" "$(aws acm list-certificates --region us-east-1 --query 'length(CertificateSummaryList)')"
  else
    printf '%-42s %s\n' "$f" "$(aws acm list-certificates --region us-east-1 --includes "$f" --query 'length(CertificateSummaryList)')"
  fi
done
```

Observed on 2026-10-09:

```text
no filter                                0
keyTypes=RSA_2048                        0
keyTypes=EC_prime256v1                   2
keyTypes=RSA_2048,EC_prime256v1          2
```

The third line is the positive control.
A probe that can only return zero proves nothing,
so the catalog below is only meaningful because the same call shape
returns two certificates when the key type matches.

### Catalog: calls that silently return nothing

Each returned HTTP 200 with `CertificateSummaryList: []` in this account.

- `aws acm list-certificates --region us-east-1`
- `aws acm list-certificates --region us-east-1 --max-items 10`
- `aws acm list-certificates --region us-east-1 --certificate-statuses ISSUED`
- `aws acm list-certificates --region us-east-1 --certificate-statuses` with all seven statuses
- `aws acm list-certificates --region us-east-1 --includes keyTypes=RSA_2048`
- `aws acm list-certificates --region us-west-2`,
  which shows the default is not region-specific
- `aws acm list-certificates --region us-east-1 --certificate-key-pair-origins ACME`,
  correct enum value,
  empty because the account holds no ACME certificates

### Catalog: calls that return the certificates

- `aws acm list-certificates --region us-east-1 --includes keyTypes=EC_prime256v1`
- `aws acm list-certificates --region us-east-1 --includes keyTypes=RSA_2048,EC_prime256v1`
- `aws acm list-certificates --region us-east-1 --includes keyTypes=RSA_2048,EC_prime256v1,EC_secp384r1`
- `aws acm list-certificates --region us-east-1 --certificate-key-pair-origins AWS_MANAGED --includes keyTypes=RSA_2048,EC_prime256v1`
- `aws acm list-certificates --region us-east-1 --certificate-statuses ISSUED --includes keyTypes=RSA_2048,EC_prime256v1`
- `aws acm search-certificates --region us-east-1`,
  hits under `Results`,
  with no key-type default
- `aws acm describe-certificate --region us-east-1 --certificate-arn <certificate-arn>`

### Catalog: calls that fail loudly

- `--certificate-key-pair-origins AMAZON_ISSUED`,
  `IMPORTED`,
  or all three of `AMAZON_ISSUED IMPORTED ACME` together,
  each raising the `ValidationException` quoted in "Symptom".
  The valid enum is `AWS_MANAGED | ACME | CUSTOMER_PROVIDED`.

### Surfaces this verification did not exercise

The account holds no RSA certificates,
so the run cannot distinguish the model's two claims about the default:
line 505 says `RSA_2048` only,
while line 2597 says `RSA_1024` and `RSA_2048` and adds a condition that
the certificate have at least one domain.
Both agree that ECDSA is excluded,
which is the behavior this document depends on.
Distinguishing them needs an account holding an `RSA_1024` certificate.

## Verified workarounds

### 1. Name every key algorithm explicitly

```bash
aws acm list-certificates --region us-east-1 \
  --includes keyTypes=RSA_1024,RSA_2048,RSA_4096,EC_prime256v1,EC_secp384r1,EC_secp521r1
```

Tradeoff:
the list is a snapshot of the algorithms ACM supports today,
so a future algorithm is silently excluded until someone extends the
command.
That is the same failure mode as the default,
just less likely.

### 2. Use `search-certificates` instead

```bash
aws acm search-certificates --region us-east-1 \
  --query 'Results[].{Arn:CertificateArn,Subject:X509Attributes.Subject.CommonName}'
```

Tradeoff:
no key-type default,
but a different response shape.
Hits arrive under `Results` rather than `CertificateSummaryList`,
and attributes are nested under `X509Attributes`,
so any script written against `list-certificates` needs reworking rather
than a flag change.
It is also a newer operation,
so older CLI versions may not offer it.

### 3. Discover certificates through their consumers

```bash
aws cloudfront list-distributions \
  --query 'DistributionList.Items[].{Id:Id,Aliases:Aliases.Items,Cert:ViewerCertificate.ACMCertificateArn}'
```

This is what the mirror work actually used,
including to identify the distribution itself.

Tradeoff:
it finds only certificates attached to a resource,
so an unattached or freshly requested certificate stays invisible.
It also enumerates one consumer at a time;
Elastic Load Balancing and API Gateway need their own calls.

### 4. Keep the ARN from the call that created it

`RequestCertificate` returns the ARN,
and `DescribeCertificate` accepts it with no filtering.

Tradeoff:
it requires durable local state.
Losing the ARN turns this back into workaround 3.

## What does not work

- **Naming every status.**
  The status filter intersects with the key-type default rather than
  replacing it,
  so all seven statuses still returned an empty list.
- **Naming the origin.**
  `AWS_MANAGED` is already in the default origin set,
  so specifying it changes nothing on its own.
  It only helped when combined with `--includes keyTypes=`.
- **Guessing enum values from the older vocabulary.**
  `AMAZON_ISSUED` and `IMPORTED` appear in ACM prose and in the ACME
  documentation's phrasing,
  but the request enum is `AWS_MANAGED | ACME | CUSTOMER_PROVIDED`.
  Both guesses raise `ValidationException`.
- **Pagination flags.**
  `--max-items` and `--starting-token` cannot surface items the filter
  already removed.
- **Another region.**
  `us-west-2` returned an empty list too,
  and certificates for CloudFront must be in `us-east-1` regardless.
- **Re-authenticating.**
  The session was already the account root identity.

## Upstream filing decision

`.out-of-scope/` was checked:
the directory holds eleven exemption files
(`bun-install.md`,
`cargo-workspace.md`,
`claude-code-upstream-bugs.md`,
`codex-harness.md`,
`jsr.md`,
`lightningcss.md`,
`low-impact-typescript-formatting.md`,
`module-es-monolith.md`,
`pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`,
`typescript-project-references.md`),
and a search of that directory for `aws`,
`acm`,
and `cloudfront` returned nothing.
No exemption applies,
so the six-constraint check runs.

1.  **Is it really upstream's fault?**
    No.
    The behavior is documented twice in the service model and once in the
    published API reference
    (<https://docs.aws.amazon.com/acm/latest/APIReference/API_ListCertificates.html>).
    An empty result for an unfiltered call is the documented default,
    not a defect.
    The only genuine defect candidate is wording:
    model line 505 says the default returns only `RSA_2048`,
    while line 2597 says `RSA_1024` and `RSA_2048` plus an at-least-one-domain
    condition.
    That is a documentation inconsistency in generated model text.
2.  **Can upstream fix it?**
    Not in the repositories this could be filed against.
    `aws/aws-cli` and `boto/botocore` both carry `service-2.json` as
    generated data synced from AWS-internal model definitions,
    which is why the two copies agree line for line.
    A hand edit to either would be overwritten by the next sync.
    The wording fix belongs to the ACM service team's model source,
    which is not public and has no tracker.
3.  **Are they supporting this use case?**
    Yes.
    The filter is documented,
    the enum is validated with an explicit message,
    and `SearchCertificates` exists as an inventory call without the
    key-type default.
4.  **Would the repo welcome our contribution?**
    Probably,
    but it is moot under constraints 1 and 2.
    Checked:
    `aws/aws-cli` has issues enabled with 764 open,
    no root `CONTRIBUTING.md` (the contents API returns 404),
    `blank_issues_enabled: false` in
    `.github/ISSUE_TEMPLATE/config.yml`,
    which routes general questions to Discussions,
    and templates `bug-report.yml`,
    `documentation.yml`,
    `feature-request.yml`,
    `source-distribution.yml`.
    No ban on AI-assisted reports was found in the files checked.
5.  **Will they likely fix it?**
    No signal either way.
    A duplicate search found nothing:
    `gh search issues --repo aws/aws-cli "list-certificates RSA_2048"`,
    `gh search issues --repo boto/botocore "acm list-certificates keyTypes"`,
    and `gh search issues "acm list-certificates empty ECDSA"` all returned
    empty result sets.
    The search mechanism was proven live by a control query,
    `gh search issues --repo aws/aws-cli "certificate"`,
    which returned three hits.
    So this is an unreported wording inconsistency,
    not a known and declined one.
6.  **Have we prototyped a minimal fix?**
    Not applicable,
    and the auto-prototype trigger did not fire.
    The skill's prototype step applies when constraints 1 to 5 hold or
    sorta-hold;
    constraint 1 fails outright because the behavior is documented,
    and constraint 2 fails because the only fixable artifact is generated
    data in a repository that cannot carry a hand edit.
    There is no consumer-side code to patch either:
    this repository holds no ACM inventory automation,
    and the mirror task scripts already read the certificate through
    workaround 3.

Decision:
do not file.
The user-facing gap is a missing hint in a documented default,
the fixable artifact is generated,
and the workaround is one flag.

## Upstream filing artifact

Nothing to add.
No new-issue draft is kept,
because constraint 1 fails on documented behavior and constraint 2 fails
on generated model data.
No additive comment draft is kept either,
because the duplicate search found no thread to comment on.
If a future session finds the wording inconsistency already reported,
the additive content would be the measured positive control in
"Verification":
the same call returning 0 and 2 depending only on `keyTypes`.

## References

- `ListCertificates` API reference,
  including the `RSA_2048` default sentence:
  <https://docs.aws.amazon.com/acm/latest/APIReference/API_ListCertificates.html>
- `Filters` structure reference:
  <https://docs.aws.amazon.com/acm/latest/APIReference/API_Filters.html>
- Bundled service model,
  cited by line in "Root cause":
  <https://github.com/boto/botocore/blob/develop/botocore/data/acm/2015-12-08/service-2.json>
- ACM ACME issuance,
  the source of the second,
  unrelated default that was ruled out:
  <https://docs.aws.amazon.com/acm/latest/userguide/acm-acme-issuance.html>
- Related:
  [`aws-cloudfront-mirror.md`](aws-cloudfront-mirror.md)
  issue 3,
  which is why this account's certificates are ECDSA in the first place,
  and
  [`doc/handover/acm-caa-renewal-blocked.md`](../handover/acm-caa-renewal-blocked.md),
  the task during which this surfaced.
