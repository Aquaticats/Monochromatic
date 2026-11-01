# Cloudflare has no free plan that lets `cloudflare.aquati.cat` work as a Njalla-DNS-hosted diversification mirror

This file documents a procurement decision against using Cloudflare
for a third diversification mirror behind `fastly.aquati.cat`
(working) and `aws.aquati.cat` (HTTP 502;
the 2026-09-07 [AWS reassessment](aws-cloudfront-mirror.md#issue-7-cloudfront-http-502-does-not-establish-absent-tls-13-origin-support)
retracts the assumption that an unfinished TLS 1.3 rollout is the cause).
 The shape matches
the canonical troubleshooting structure even though the "bug" is a
plan-tier constraint rather than a code defect.

Date:
 2026-05-09.
 Decision:
 **not pursued.
**

## Symptom

Goal:
 add `cloudflare.aquati.cat` as a third CDN endpoint pointing
at the same origin as the Fastly and AWS mirrors.
 Keep
`aquati.cat`'s authoritative DNS at Njalla;
 keep the apex CAA
record `0 issue "letsencrypt.org"`.
 Use Cloudflare's free plan.

Outcome:
 every viable Cloudflare configuration either violates a
constraint (move DNS away from Njalla) or requires a paid plan
(Business or Enterprise).
 No combination of free Cloudflare plan

- DNS-at-Njalla + diversification-only goal works without
  significant off-label use of the Cloudflare-for-SaaS product.

## Root cause

Cloudflare's free tier monetises by encouraging customers to move
authoritative DNS onto Cloudflare's nameservers (the "full setup").
That model is incompatible with the constraint of keeping
`aquati.cat`'s DNS at Njalla.
 The two off-the-shelf
"DNS-stays-elsewhere" configurations (partial CNAME setup,
subdomain NS delegation) are gated to Business and Enterprise
respectively.

The remaining off-label path (Cloudflare for SaaS Custom Hostnames)
fits the constraint mechanically but is targeted at multi-tenant
SaaS providers proxying customer hostnames,
 not at a single
operator running their own mirror.
 The overhead does not match the
"diversification-only" goal.

## Verification (Cloudflare plan availability matrix)

Each candidate path was checked against the current Cloudflare
plan documentation as of 2026-05-09.
 Sources cited per path.

### Path 1 (excluded by user): move `aquati.cat` DNS to Cloudflare

Cloudflare's intended free model.
 Apex zone moves to Cloudflare,
orange-cloud the relevant subdomain,
 Universal SSL handles the
cert.
 Excluded because it breaks the constraint of keeping DNS at
Njalla and the apex CAA setup the AWS path was carefully designed
around.

Tradeoff if attempted:
 free;
 loses Njalla DNS hosting;
 CAA must
move to Cloudflare's nameservers.
 Rejected by constraint.

### Path 2 (paid): Partial (CNAME) setup

Lets Cloudflare proxy a single subdomain via a CNAME at the
existing DNS provider,
 with no nameserver change.
 Per Cloudflare's
docs:

- Free:
   not available
- Pro:
   not available
- Business:
   available
- Enterprise:
   available

Source:
<https://developers.cloudflare.com/dns/zone-setups/partial-setup/>.
~$250/mo for Business is too steep for a diversification mirror.

### Path 3 (paid): Subdomain setup (NS delegation as a separate zone)

Add `cloudflare.aquati.cat` as its own Cloudflare zone;
 delegate
via NS records at Njalla.
 Per Cloudflare's docs:

- Free:
   not available
- Pro:
   not available
- Business:
   not available
- Enterprise:
   available

Source:
<https://developers.cloudflare.com/dns/zone-setups/subdomain-setup/>.
Enterprise tier well outside scope for a diversification mirror.

### Path 4 (free, off-label): Cloudflare for SaaS / Custom Hostnames

Free plan includes 100 custom hostnames;
 $0.10 per hostname/month
beyond.

Source:
<https://developers.cloudflare.com/cloudflare-for-platforms/cloudflare-for-saas/plans/>.

Mechanics:
 a placeholder zone (any domain owned and added to
Cloudflare Free) acts as the SaaS provider zone.
`cloudflare.aquati.cat` is added as a Custom Hostname on that
zone.
 CNAME at Njalla points `cloudflare.aquati.cat` at a proxied
CNAME target on the placeholder zone.
 Fallback origin proxies
back to `aquati.cat`.
 Cert is issued per-hostname via DCV;
 CAA
chain follows the CNAME away from the apex
(<https://datatracker.ietf.org/doc/html/rfc8659#section-3>) so the
apex `letsencrypt.org`-only CAA is not read.
**Correction,
 2026-10-09**:
 that CAA reasoning is wrong.
RFC 8659 section 3 climbs the parents of the queried FQDN,
and only the single-node CAA query chases the alias;
section 7 records that this replaced the RFC 6844 chain-climbing.
ACM's renewal refusal for `aws.aquati.cat` on 2026-10-09 is the measured
confirmation.
This path would therefore have needed the apex CAA to authorize
Cloudflare's issuing CAs as well,
which does not change the plan-tier gating that decided it.
See [`aws-cloudfront-mirror.md`](aws-cloudfront-mirror.md) issue 6 and
[`../handover/acm-caa-renewal-blocked.md`](../handover/acm-caa-renewal-blocked.md).

Off-label because the SaaS product targets multi-tenant SaaS
providers proxying customer hostnames.
 Using it for one's own
hostname is allowed but adds operational overhead disproportionate
to the goal:
 a placeholder domain registration (~$10/yr),
nameserver migration of the placeholder zone,
 SaaS configuration,
DCV setup,
 and per-hostname renewal monitoring,
 all to add a
third mirror behind a working Fastly setup.

## Verified workarounds

There are no satisfactory workarounds within the stated constraints
on the free tier.
 The candidate workarounds and their tradeoffs:

### Accept Path 1: move DNS to Cloudflare

Free;
 meets the diversification goal;
 loses Njalla DNS hosting and
the apex CAA structure.
 Tradeoff:
 large operational change for a
mirror that is third in line behind Fastly and AWS.

### Pay for Path 2 or Path 3

Meets all technical constraints;
 costs $250+/mo.
 Tradeoff:
unjustified spend for a diversification-only outcome.

### Implement Path 4 (off-label SaaS)

Free;
 meets all stated constraints;
 significant ongoing
operational overhead (placeholder zone,
 DCV,
 per-hostname renewal,
SaaS configuration).
 Tradeoff:
 disproportionate effort for a
third mirror.
 Documented as a fallback if a future requirement
makes the effort worthwhile.

## What does not work

- **Workers on a `workers.dev` URL + Njalla HTTP redirect**:
   the
  public URL changes after redirect;
   not a transparent alternate
  endpoint,
   which defeats the diversification goal.
- **Cloudflare Tunnel**:
   the public hostname must live on a
  Cloudflare zone;
   collapses into Path 1 (move apex) or Path 4
  (SaaS Custom Hostnames).
- **Cloudflare Pages with custom domain**:
   same constraint as
  Tunnel (custom-domain binding requires the hostname's zone on
  Cloudflare).

## Decision

Skip Cloudflare.
 The mirror set remains `fastly.aquati.cat`
(working) plus the unresolved `aws.aquati.cat` endpoint.
The [AWS mirror investigation](aws-cloudfront-mirror.md)
no longer treats waiting for TLS 1.3 as an established resolution path.
 Re-evaluate
if Cloudflare loosens partial-setup or subdomain-setup plan
gating,
 or if caching or WAF requirements change the goal beyond
pure diversification.

## Why we do not file this upstream

The constraint is a Cloudflare procurement policy,
 not a defect.
Walking the 5 constraints anyway:

1. **Is it really upstream's fault?
   ** No. Plan gating is a
   business decision.
2. **Can upstream fix it?
   ** Yes,
    by opening partial setup to free
   tier.
    They will not,
    because partial setup is the documented
   conversion path to paid plans.
3. **Are they supporting this use case?
   ** Yes for paid tiers,
    no
   for free.
4. **Will they likely fix it?
   ** No.
5. **Have we prototyped a minimal fix?
   ** N/A.

Decision:
 no upstream report.
 Procurement decision is on our side.

## References

- Cloudflare for SaaS plans:
  <https://developers.cloudflare.com/cloudflare-for-platforms/cloudflare-for-saas/plans/>
- Partial (CNAME) setup availability:
  <https://developers.cloudflare.com/dns/zone-setups/partial-setup/>
- Subdomain setup availability:
  <https://developers.cloudflare.com/dns/zone-setups/subdomain-setup/>
- RFC 8659 §3,
   CAA tree-walk and CNAME-following:
  <https://datatracker.ietf.org/doc/html/rfc8659#section-3>
- Related:
   [AWS mirror investigation](aws-cloudfront-mirror.md)
  (the AWS path's CAA pattern this evaluation reused).
