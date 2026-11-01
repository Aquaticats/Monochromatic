# Owned Depot allocation normalization

The [normalized baseline](../audit/depot-owned-allocation-ledger-2026-10-18.evidence.json)
reconciles expanded research targeting application
`e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.
Input captures start at `2026-10-18T13:59:27.911853+00:00`;
the latest inventory refresh is `2026-10-18T14:47:19.720513+00:00`.
This is a captured owned-ID inventory,
not an account-wide usage or invoice census.
No allocation or settings mutation was performed by the reviewer.

## Identities and measured boundaries

The baseline contains 223 distinct records:
115 native job attempts from 77 runs,
79 GitHub job records from 45 runs,
seven SDK VMs and 22 BuildKit builds.
The GitHub records distinguish 52 Depot-runner requests,
25 standard public GitHub jobs and two unassigned corrupt-label jobs.
Two additional zero-job workflow requests and one SDK request without a returned VM identity retain explicit unknowns.
Fresh read-only metadata and 122 exact immutable workflow definitions support the CI joins.
Overlapping receipt pointers do not create additional allocations.

## Nominal compute and reservations

Priced observed intervals give native CI `USD 4.8546`,
Depot GitHub runners `USD 1.3372` and SDK VMs `USD 0.0081959`.
Standard public GitHub compute is `USD 0` under the current public-repository rule.
Their known subtotal is `USD 6.1999959` before allowances.
It excludes missing execution boundaries and unknown BuildKit architecture-builder intervals;
it is not an invoice or observed VM billing lifetime.

The BuildKit campaign records 403938 milliseconds of request envelopes.
Its separate maximum-admitted-two-builder proxy at `USD 0.04` per builder-minute is exactly `USD 0.538584`.
Actual builder compute remains unknown.
Per-request repeating values retain exact rational fractions.
The SDK-specific reference supplies its CI-rate model;
the pricing page's separate Agent-sandbox category is not substituted for that interface-specific source.

Per-record full-timeout bounds and 100 reported historical/admission budget observations are separate fields.
Their sums are counterfactual,
not spending or active campaign reservations.
The Rust cumulative model already includes reuse;
the SDK third admission is not another VM;
BuildKit summary mirrors are not another build.
Queued `started_at` alone does not prove execution:
the normalizer requires a runner assignment and an actually started step for GitHub pricing.
Unknowns remain null.

## Retained source and follow-ons

The [source archive](../audit/depot-owned-allocation-ledger-2026-10-18.sources.json.gz)
contains original UTF8 input bodies,
owned metadata,
immutable workflow bytes,
fresh public rate sources and the normalizers' exact mapping/arithmetic inputs.
Its record pointers apply to parsed `bodyUtf8` for JSON sources.
Every input and original body has byte-length and SHA256 metadata.
Currency strings avoid floating-point rounding in the normalized arithmetic.
Functional coverage remains qualified by each owning receipt.
Actions API `head_sha` stays distinct from actual checkout identity.
Six retained PR consumer observations supply actual merge SHAs and corresponding immutable workflow lookups;
other unobserved GitHub checkout SHAs remain null.

The coordinator will append its coherent graph `39135886118` after measurement.
Later coverage/platform follow-ons require separate measured appends.
The native BuildKit authentication plan retains its separate `USD 0.23` full-window admission and `USD 1` guard;
its admission is not added as observed compute.
Its later project `ng17dl3xrn` was created at `14:47:45.122 UTC`
and native run `dzsf27t0xx` was reported dispatched at `14:54:05.816 UTC`,
after the baseline inventory capture.
Those exact identities remain explicit follow-ons until terminal measurement is appended.
The coordinator's independent arithmetic review passes at `2026-10-18T15:41:16.033Z`.
It rehashes all 411 original bodies and both archive encodings,
rejects duplicate allocation identities,
and recomputes all 195 priced intervals from timestamps,
CPU identities and exact rational rates.
The known subtotal is exactly `61999959/10000000` dollars;
the separate BuildKit request proxy is exactly `67323/125000` dollars.
Post-baseline attempts still require measured appendices before a final combined total.

The new guest cohort's
[independent review](../audit/depot-independent-policy-process-mvm-review-2026-10-18.md)
passed original-byte,
CRC/member,
source,
command and exact transformed-payload joins.
The original queue attribution to `39132577785` at `3829831039cc43aa717c6c927f835b867e9313b6`
is retained as a corrected cancelled-generator control.
Actual guest consumers are native `25jpltfpkq` and GitHub `39134227406`
at `1373e624739fbc93502b0c8ca502820c6ff99b45`.

## Publication checks

Targeted `lint:monochromatic-lint` and rendered CommonMark inspection pass.
Original-byte checks cover all 411 retained source bodies;
all 1118 source-pointer joins and six actual PR-checkout joins resolve.
Exact Decimal/Fraction recomputation matches all 195 priced intervals and the separate BuildKit proxy.
Credential-pattern candidates classify as public Rust cfg names or normalized workflow filenames.
The independent coordinator review matches those baseline totals and preserves all 28 unknown-price records.
