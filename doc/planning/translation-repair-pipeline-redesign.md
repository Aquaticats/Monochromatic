# Redesigning the translation-repair pipeline

## Status

This is the original requirement record,
not an accepted architecture decision.
Candidates A through M were compared,
implemented where warranted,
and rejected or stopped at design gate.
Candidate M failed its single pinned-Carena calibration on 2026-09-01,
so all further implementation has stopped.

The canonical disposition and failure hypotheses are in
[`translation-repair-redesign-failure-2026-09-01.md`](../audit/translation-repair-redesign-failure-2026-09-01.md).
The current repository and evidence state is in
[`translation-repair.md`](../handover/translation-repair.md).
Do not implement a replacement,
retry a spent prompt,
or launch another calibration until the owner explicitly authorizes a new phase.

## Corrected requirements

### Every invocation has finite work

No stage may contain an open-ended quality loop such as
`while (!accepted)` or equivalent recursion,
queue self-replenishment,
or retry scheduling with no statically defined frontier.

Each invocation must have a finite call graph.
Every model responsibility,
maximum payload count,
and dependency edge must be known before invocation starts.
A rejected result cannot cause same stage to call itself again.
Changing text,
findings,
nonce,
round,
or prompt wording cannot extend frontier.

This requirement rejects current loops in
`package/module/translation-repair/src/consolidation-naturalness-settle.ts` (removed before 2026-10-18)
and
`package/module/translation-repair/src/consolidate-slice-buy.ts` (moved on 2026-10-18 to
`package/translation-repair/consolidate/src/consolidate-slice-buy.ts`)
as replacement architecture.
Moving them behind new interface would not fix design.

### A normal run returns good output

A normal run with working model and publication infrastructure always returns one good complete document.
Pipeline must not return quality refusal,
`ERROR`,
`INCOMPLETE`,
or visible suspended translation because a reviewer,
auditor,
reviser,
or generic quality Gate failed.
That behavior leaves caller hanging and does not meet output contract.

Model nodes are preparation-evidence or candidate-producer nodes.
Preparation-evidence node may produce brief or specification before authorship;
its failure contributes nothing and cannot withhold producer work.
Every post-preparation model node is candidate producer.
First adopted producer owns full concrete quality contract:
fidelity,
completeness,
identity,
grammar,
clear references,
consistent tense,
paragraph relations,
and register.
Later stages improve named defect classes and are not required rescue for deficient baseline.
Each stage returns a complete candidate under one named responsibility,
or it has no effect and prior complete candidate remains available byte-for-byte.
A non-producing node cannot gain authority to withhold output.
Before first candidate exists,
finite manifest may try each statically named fallback producer once.
Operational run begins after provider preflight and finite manifest persistence.
First candidate is adopted only after one producer yields complete document passing deterministic obligations.
If every planned producer has transport failure or unusable response before first adoption,
command throws bounded `ProductionUnavailableError` with exhausted nodes.
It does not suspend,
auto-requeue,
or publish unchecked archive fallback.
`ProductionUnavailableError` concerns exhausted candidate producers only;
failed preparation-evidence node never causes it.
This physical infrastructure or model-output failure is only no-output exception.

Deterministic checkpoints retain completed nodes for exact cancellation and restart.
They are internal recovery mechanics,
not user-visible terminal or indefinite suspended state.
Restart requires same manifest digest and checkpoint,
executes pending nodes only,
and cannot add manifest nodes.
Completed,
failed,
unusable,
aborted,
or indeterminate nodes are spent.
Indeterminate transmission may reuse recorded payload but may never resend canonical prompt.
Caller abort bypasses fallback and throws exact `signal.reason`.

Assembly,
atomic write,
or readback failure after candidate exists throws bounded `PublicationUnavailableError`.
It does not suspend,
auto-requeue,
publish partial bytes,
publish archive fallback,
or become quality outcome.

### Naturalness is not a measurement

`naturalness`,
`absolute naturalness`,
`naturalness floor`,
and similar labels are too unstable to serve as metric,
threshold,
or aggregate quality fact.
A panel vote does not turn term into measurement.

Future architecture must use concrete,
anchored editorial evidence:
wrong meaning,
omission,
addition,
identity change,
syntax damage,
grammatical error,
unclear reference,
inconsistent tense,
broken paragraph relation,
register mismatch,
or another explicitly named defect.
Subjective editorial judgment remains evidence,
not numeric or absolute measurement.

Existing telemetry can retain historical field names for old artifacts.
New schema and implementation must not present naturalness verdict count as quality measure.

### No generic final quality gate

Generic final Gate exists because preceding stages together do not reliably produce good text.
Adding stronger Gate,
more reviewers,
or another correction loop treats symptom.
Replacement must make quality responsibility part of transformation that creates wording.

Every text-producing stage needs explicit postcondition and accountable owner.
Later stage returns complete candidate plus exact edit transaction anchored to prior bytes and source evidence.
Deterministic adoption rejects undeclared edits and preserves prior candidate when revision is unusable.
Later stage cannot be catch-all panel that rescues arbitrary upstream defects.
Final assembly should enforce deterministic integrity and publication mechanics,
not ask models whether combined pipeline happened to be good enough.

## Stopped Carena evidence

Run used pipeline digest
`sha256-tree-v1:0edfb04f6b9f7e0181de7b810ce483135dab59190c53cc6464433fb626bdff1e`,
overlap 4,
and both required providers wet.
It started at `2026-08-30T03:40:51.772Z`
and was stopped by user after 16,659 seconds of retained log time.
It produced no `TALLY`,
artifact,
or fixed page.

Consolidation started 8,735 seconds before retained log ended.
Eighteen slices were contested.
Four consolidation slices started;
one ended after 636,939 milliseconds with `exit=failed`,
and no other consolidation slice reached `SLICE-COST`.

Consolidation bought 186 roster rounds:

- 50 selection rounds
- 46 refiner rounds
- 46 rounds under historical `absolute-naturalness-review` name
- 41 consolidation polish gates
- 3 consolidation fidelity gates

Those rounds sum to 25,899,861 milliseconds because four slices overlap.
22,280,018 milliseconds,
86.0 percent of summed round time,
was post-quorum grace.
Forty of 46 historical naturalness reviews returned `unacceptable`;
five returned `acceptable`,
and one lacked quorum.

This is not provider outage.
Responses continued and both meters remained wet.
Overlap hid waits but could not make stage settle.

Durable sanitized evidence is
`doc/audit/translation-repair-carena-stopped-run.md`.
Complete partial evidence and analysis remain outside repository at:

- `~/temp/agent/analysis-Carena-current-overlap4-stopped-phases-20260830.json`
- `~/temp/agent/analysis-Carena-current-overlap4-stopped-latency-20260830.json`
- `~/Downloads/Carena0442-current-overlap4-run1-20260829/`

## Root-cause hypothesis

Current pipeline does not assign one module responsibility for producing finished English.
It composes candidate generators,
selectors,
fidelity gates,
refiners,
reviewers,
and polish gates,
then asks later panels to repair what previous panels accepted.

Per-slice candidate competition fragments document voice.
Selection identifies preferred candidate,
not necessarily finished wording.
A subsequent generic editor can alter source-faithful choice.
A subsequent generic review can reject any altered paragraph.
One dissent reopens full production,
selection,
and gate machinery.

`FINAL_POLISH_MINIMUM_CHARS = 0` in
`package/module/translation-repair/src/consolidation-polish-round.ts` (moved on 2026-10-18 to
`package/translation-repair/consolidate/src/consolidation-polish-round.ts`)
makes every structurally refinable paragraph eligible on every polish attempt.
`package/module/translation-repair/src/absolute-naturalness-review-stage.ts` (moved on 2026-10-18 to
`package/translation-repair/archive/src/absolute-naturalness-review-stage.ts`)
makes any usable rejection decisive.
`package/module/translation-repair/src/consolidation-naturalness-settle.ts` (removed before 2026-10-18)
adds each changed correction to prior evidence,
so exact cycle detector does not fire while wording and findings continue to vary.

These mechanisms make more review create more work without demonstrating convergence.
Concurrency cannot repair that relationship.

## Replacement design target

Explore interfaces built around finished-document authorship rather than stage rescue.
Candidate replacement should make these responsibilities explicit:

1.  Derive source-backed document specification,
    including identities,
    structure,
    claims,
    links,
    media evidence,
    and archive-supported context.
2.  Give one accountable document editor whole specification and whole archive context.
3.  Execute one finite dependency graph of role-partitioned whole-document producers.
    Dispatch ready independent nodes concurrently;
    preserve order only when a prompt consumes another node's output.
    Roles correct concrete defect classes rather than assign score or emit blocking opinion.
4.  Make each specialist a bounded producer of one complete responsibility-specific revision.
    Unusable specialist output has no effect and cannot withhold prior complete candidate.
5.  Assemble and verify deterministic structure,
    completeness,
    identity,
    source anchors,
    and publication integrity.
    No generic model Gate follows.

This shape is only initial design target.
Design-it-twice comparison must include alternatives with different seam placement.

## Measured scheduler capacity

Live short structured probes on 2026-08-30 established these planning bounds:

- Synthetic:
  5 in-flight calls per active model and 20 across four-model roster
- Hyper:
  no provider concurrency ceiling,
  with width 64 corroborated on live request shape
- Hyper request rate:
  owner-supplied account limit of 1,000 requests per hour,
  separate from in-flight concurrency

Synthetic aggregate width 20 completed twice without retry or non-200 status.
Raising only gpt-oss to width 10 produced 3 HTTP 429 responses,
so scheduler design must not infer slots from model size.
Corpus-sized generation still requires user-boundary validation.
See `doc/troubleshooting/translation-repair-provider-concurrency.md`.

Concurrency changes graph waves,
not semantic dependencies.
Candidate C's three preparation briefs belong in one wave.
Its primary and fallback editors can also start together from the same immutable packet,
with fixed primary priority and fallback adoption only when primary is unusable.
Candidate B should expose independent specialist transactions in one wave against the same adopted base,
then merge them deterministically.
No design may add panel work merely to occupy provider slots.

## Questions design must answer

- How does one editor prove complete source coverage without per-slice candidate tournament?
- Which editorial responsibilities belong to source fidelity,
expression,
and document continuity producers?
- How can producer roles partition responsibility so same general opinion is not multiplied?
- How does each complete revision preserve accepted wording while resolving cross-paragraph issues?
- How does every later producer preserve prior complete candidate when its own output is unusable?
- How does deterministic restart reuse completed nodes without creating a visible suspended translation?
- Which current caches and artifacts remain useful evidence,
and which encode architecture being removed?
- What user-facing interface makes one entry trivial to run,
inspect,
resume,
and publish?

## Immediate next evidence

Before selecting architecture:

- compare multiple radical interfaces against corrected requirements
- replay stopped Carena task ledger to calculate payload reduction each design would provide
- test each design against known omission,
identity,
archive,
visual,
front-matter,
and lane-choice fixtures
- define positive control proving concrete editorial defect detector can find seeded defect
- define negative control proving detector does not manufacture work from acceptable text
- require final output reading;
mechanical completion alone does not validate architecture
