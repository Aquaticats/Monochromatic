# Translation repair interfaces K to M: Candidate M, risk-attested authors and role-split challengers

Candidate M's plan,
its author and challenger contracts,
alternatives and acceptance controls,
its implementation evidence,
calibration preflight,
and the calibration result that ended implementation.

Part of [Translation repair interfaces: Candidates K to M](translation-repair-interface-candidates-k-m.md),
which lists every file of this record.

## Candidate M plan: risk-attested authors and role-split challengers

Candidate M directly removes Candidate L's measured verifier status-alphabet surface without editing or redispatching a
spent candidate.
It does not claim to solve Qwen's author deadline or GLM's thinking-only output-ceiling failure.
Authors receive a substantively new risk-attestation contract.
Verifiers replace exhaustive status strings with one bounded whole-page challenge in each fixed responsibility role.
No response chooses,
creates,
or removes a node.

If Qwen's author fails again,
its six challengers skip and only MiniMax's candidate can qualify.
If GLM abstains in either challenger role,
no candidate can satisfy the strict two-family nonself floor.
Both conditions are terminal non-publication by design,
not triggers for fallback or retry.

### Risk-attested author contract

Qwen3.8-27B and MiniMax M3 each receive one complete source,
archive,
image,
immutable shell,
and 27-value response schema.
The packet also carries one static generic risk register derived from prior complete-page failures:

- actor attribution;
- event ownership and sequence;
- temporal and pronominal reference;
- unsupported emphasis;
- source-image relation;
- memorial register and contributor voice.

The response is exactly:

```ts
export type CandidateMAuthorResponse = {
  readonly slots: Readonly<Record<CandidateMMutableKey, string>>;
  readonly riskAttestations: {
    readonly actorAttribution: 'checked';
    readonly eventOwnershipSequence: 'checked';
    readonly temporalPronominalReference: 'checked';
    readonly unsupportedEmphasis: 'checked';
    readonly sourceImageRelation: 'checked';
    readonly memorialRegisterContributorVoice: 'checked';
  };
};
```

The manifest owns that key order and the sole `checked` code.
Admission compares `Object.keys(response.riskAttestations)` exactly with manifested order before hashing.
It rejects order deviation rather than canonicalizing it.
Missing,
extra,
unknown,
duplicated,
or differently coded members reject the response atomically.
The raw duplicate-member guard runs before parsing.
The ordered risk register,
code alphabet,
attestation object digest,
author protocol digest,
author schema digest,
manifest identity,
and deterministic candidate proof bind one another.

Every code means only that the author performed the named check.
It supports admission integrity only.
It is not clean evidence,
never votes,
and cannot override a verifier defect.
Runtime admits a candidate only when all 27 values and all six attestations are present exactly once.

This is substantively different from Candidate L.
Candidate L asked for values only under one general complete-page instruction.
Candidate M changes the packet,
response schema,
author responsibility,
protocol digest,
and deterministic admission contract.
It does not send Candidate L's canonical author prompt again.

### Role-split challenger contract

Each usable candidate has six statically manifested challengers:
Qwen,
GLM 5.3 Flash,
and MiniMax each receive one fidelity role and one publication-language role.
Every challenger receives the complete source,
archive,
candidate,
review plan,
deterministic proof,
and every page-referenced image.

The fidelity role owns:

- source facts,
actors,
relationships,
chronology,
causality,
reference,
omission,
and unsupported addition;
- identity,
front-matter meaning,
links,
media relation,
and image-grounded claims;
- exact contributor authority where wording changes meaning or agency.

The publication-language role owns:

- grammar,
idiom,
sentence attachment,
reference clarity,
tense,
register,
and paragraph coherence;
- memorial tone,
contributor voice,
and publication readiness;
- language defects in all 27 candidate-authored values.

Each challenger returns one compact atomic response:

```ts
export type CandidateMChallenge = {
  readonly candidateId: string;
  readonly candidateDigest: string;
  readonly deterministicProofDigest: string;
  readonly sourceReviewPlanDigest: string;
  readonly role: 'fidelity' | 'publication-language';
  readonly verdict: 'clean' | 'defect';
  readonly findings: readonly [] | readonly [CandidateMFinding];
};
```

A clean verdict requires an empty finding array.
A defect verdict requires exactly one publication-blocking finding with exact manifested source and target evidence.
The verifier must review the whole page before returning either verdict.
It need not narrate every defect after the first decisive counterexample.

Clean responsibility remains role-specific,
but valid defect classes use this closed role table:

- fidelity only:
  `wrong-meaning`,
  `omission`,
  `unsupported-addition`,
  `identity-attribution`,
  `chronology`,
  `technical-legal-term`,
  and `image-relation`;
- publication language only:
  `grammar-usage`,
  `tense`,
  `register`,
  `source-language-calque`,
  `paragraph-coherence`,
  and `contributor-voice`;
- shared cross-role veto:
  `actor-reference`,
  `event-ownership`,
  and `reference-attachment`.

Every finding has required arrays
`sourceEvidence`,
`targetAnchors`,
and `imageEvidenceIndexes`.
Each source evidence member is exactly
`{ scope: 'front-matter' | 'clause' | 'relation', subjectIndex: number }`.
All scopes,
indexes,
and anchors bind manifested evidence and candidate digests.
The closed class-to-source-scope table is:

- front matter permits
  `wrong-meaning`,
  `omission`,
  `unsupported-addition`,
  `identity-attribution`,
  `actor-reference`,
  `reference-attachment`,
  `grammar-usage`,
  `tense`,
  `register`,
  `source-language-calque`,
  and `contributor-voice`;
- clause permits every Candidate M defect class;
- relation permits
  `wrong-meaning`,
  `omission`,
  `unsupported-addition`,
  `actor-reference`,
  `event-ownership`,
  `reference-attachment`,
  `chronology`,
  `paragraph-coherence`,
  and `image-relation`.

A source index valid in another namespace or a class not allowed in that namespace abstains atomically.
Evidence cardinality is class-specific:

- fidelity classes other than omission and image relation require at least one source subject and one exact target anchor;
- omission requires at least one source subject and permits zero or one nearby target anchor;
- image relation requires at least one exact target anchor and one image index;
- publication-language classes require at least one exact target anchor and permit zero or one source subject;
- shared cross-role classes require at least one source subject and one exact target anchor;
- classes without image ownership require an empty image-index array.

Target anchors identify candidate front-matter or body slots plus exact start,
end,
and runtime-recomputed substring digest.
Model-facing evidence does not retain source or candidate wording outside the private candidate and prompt artifacts.

A challenger may report one fully evidenced shared blocker rather than suppress it or return clean.
An allowed shared defect vetoes the candidate,
but it does not provide clean evidence for either role.
A role-exclusive class emitted by the other role,
an unknown class,
or invalid evidence makes the whole challenge abstain atomically.
That abstention neither vetoes nor counts as clean and never creates a handoff node.

This interface removes Candidate L's long status alphabet,
per-clause status vector,
27-character language vector,
and overflow algebra from the wire.
Runtime still binds complete review-plan digest,
candidate proof,
role,
model family,
and evidence anchors.
An invalid or incomplete challenge abstains atomically.

### Static graph and selection

Candidate M uses manifest version 3 and exact architecture discriminator
`candidate-m-risk-challenger`.
Version 1,
version 2,
missing,
or different architecture identity cannot bind Candidate M nodes or restart artifacts.

The manifest fixes at most 14 nodes before contact:

1.  two concurrent risk-attested authors;
2.  six concurrent role-split challenger templates for each author ordinal;
3.  deterministic private selection after every dispatched sibling settles.

An unusable author skips its six candidate-bound challengers.
Every independent sibling settles before exact caller cancellation propagates.
Indeterminate transmission is spent.
Restart dispatches no terminal or potentially transmitted node.
No fallback route,
continuation,
correction,
or generated work exists.

A candidate is publication-eligible only when:

- no admitted self or nonself challenge reports a defect;
- each role has clean nonself evidence from both remaining model families;
- those clean families are distinct under the existing conservative family map;
- every deterministic candidate,
front-matter,
syntax,
media,
and contributor guard passes.

Self clean evidence never qualifies.
A valid self defect vetoes.
An abstention is neither clean nor dissent.
Self abstention neither qualifies nor vetoes.
A nonself abstention blocks only by leaving its role below the clean-family floor.
Partial role coverage cannot combine into clean evidence.
Private fallback remains minimum numeric priority,
then minimum candidate ordinal,
and records `evidenceFloorMet: false` when either role lacks its floor.

### Challenger options considered

#### Option A: role-split first-defect challengers

Pros:

- removes the status alphabet that invalidated Qwen and MiniMax Candidate L ballots;
- removes the large status-vector output that Qwen and MiniMax failed to follow;
- isolates semantic fidelity from publication language while both still read the whole page;
- one exact counterexample is sufficient to veto a candidate;
- keeps all work statically named and atomic.

Cons:

- raises the maximum graph from eight to 14 payloads;
- clean evidence is a whole-role assertion rather than a visible per-subject vector;
- GLM remains load-bearing for the strict two-family nonself floor;
- GLM's measured thinking-only `max_tokens` failure occurs before tool output,
  so the smaller response does not mitigate it;
- role overlap can produce duplicate defects,
  though duplicates have the same veto effect.

#### Option B: combined first-defect challengers

Pros:

- keeps Candidate L's eight-node maximum;
- removes status vectors and overflow while preserving one whole-page verdict;
- has the smallest verifier payload count.

Cons:

- retains the combined semantic and language burden that made every Candidate L verifier unusable;
- gives no role-specific evidence when a verifier abstains;
- one clean assertion spans too many distinct responsibilities.

#### Option C: independent status and finding halves

Pros:

- narrows each output further;
- can retain exhaustive coverage in one half and exact evidence in the other;
- separates compact classification from prose evidence.

Cons:

- independently generated halves can disagree and force abstention;
- doubles verifier calls without making one atomic clean decision;
- finding work depends conceptually on another response even when nodes are statically listed;
- recreates the status vectors that failed Candidate L.

Ranking:
Option A > Option B > Option C.
Option A ranks over Option B because Candidate L measured complete failure under combined exhaustive review,
while role splitting removes that measured burden and keeps atomic decisions.
Option B ranks over Option C because one complete challenge is auditable without reconciling independently generated halves.

### Author options considered

#### Option A: closed risk attestations plus 27 values

Pros:

- changes author responsibility and schema enough to be a new substantive protocol;
- directly foregrounds the actor and event-attachment defects found in Candidate L;
- adds a bounded attention mechanism without serial editing or dynamic work;
- preserves immutable-shell compilation.

Cons:

- attestations are not independently trustworthy and cannot count as quality evidence;
- extra fields can increase schema-following burden;
- the same model can attest despite still making the named defect.

#### Option B: values-only author with rewritten prose instructions

Pros:

- keeps the smallest response;
- MiniMax already completed Candidate L's 27-value schema;
- introduces no new admission surface.

Cons:

- differs from a spent Candidate L prompt mainly by wording;
- provides weak prompt-uniqueness evidence;
- adds no structural attention to the measured actor and attachment failures.

#### Option C: readable review-plan author

Pros:

- exposes every source clause and relation before writing;
- gives the author the richest explicit coverage plan;
- reuses implemented plan evidence.

Cons:

- Candidate K measured zero admitted authors under this responsibility;
- Qwen returned unparseable tool JSON;
- GLM and MiniMax exhausted the output ceiling;
- it repeats a rejected producer interface.

Ranking:
Option A > Option B > Option C.
Option A ranks over Option B because its changed response and admission contract provide structural prompt uniqueness and
focus the measured defect classes.
Option B ranks over Option C because a lean values-only producer has one measured admitted result,
while readable-plan authors admitted none.

### Candidate M acceptance controls

Candidate M cannot enter live calibration until tests and GFP controls prove:

- manifest version 3,
  exact `candidate-m-risk-challenger` discriminator,
  stale-version refusal,
  two authors,
  two roles,
  three verifier families,
  14-node ceiling,
  and exact static skips;
- author packets contain the generic risk register but omit Candidate L and Candidate K reviewer bookkeeping;
- exact 27 values plus the six-key `riskAttestations` object and sole `checked` alphabet;
- manifested attestation order through exact `Object.keys` comparison,
  order-shuffle,
  duplicate,
  missing,
  unknown,
  extra,
  and wrong-code rejection;
- attestation digest binding through protocol,
  schema,
  manifest,
  candidate proof,
  and restart,
  with no attestation contributing to selection;
- each role's schema,
  allowed finding classes,
  source evidence,
  target anchors,
  candidate binding,
  and first-defect cardinality;
- clean means zero findings and defect means exactly one finding;
- two-family nonself clean evidence independently for both roles;
- self clean exclusion and self defect veto;
- deterministic fallback order and explicit ineligibility below either role floor;
- the exact role-to-defect-class table,
  exact source namespace,
  class-to-source-scope table,
  and class-specific source,
  target,
  and image evidence cardinality;
- GFP wrong-scope mutations for front matter,
  clause,
  and relation evidence;
- shared actor-reference,
  event-ownership,
  and reference-attachment vetoes that never become clean evidence;
- atomic abstention for unknown,
  out-of-role,
  or invalidly evidenced classes,
  and no handoff behavior;
- abstention as neither clean nor dissent,
  self abstention as no effect,
  and nonself abstention as role-floor absence only;
- all images,
  provider isolation,
  prompt uniqueness,
  raw duplicate rejection,
  exact abort identity,
  all-sibling settlement,
  indeterminate transmission,
  and zero-call restart;
- realistic maximum request serialization below the manifested client limit;
- complete-Carena request-envelope tokenization under each exact model tokenizer;
- when an exact tokenizer is unavailable,
  an authenticated provider token-count endpoint or a previously accepted larger envelope with identical static content
  and a proved conservative token relation;
- a route marked unmeasured and no live spend when neither evidence path exists;
- synthetic forced-tool probes treated only as route compatibility,
  never as evidence that complete-page GLM work will finish;
- author deadline 1,800,000 milliseconds,
  challenger deadline 900,000 milliseconds,
  and 32,000 output tokens bound into protocol and manifest identity;
- targeted lint,
  types,
  rebuilt tests,
  full `buildAndTest`,
  GFP mutations,
  and advisor review;
- a self-hashed no-key harness independently proves all 14 static request bindings before one pinned-Carena spend;
- complete source,
  archive,
  image,
  and every admitted candidate are read before any production disposition,
  because clean challenges no longer expose per-subject status coverage.

The longer author deadline is a finite calibration bound,
not a claim that Qwen will complete.
The challenger deadline does not mitigate GLM's output-ceiling behavior.
Any failure at either boundary remains terminal for that one new prompt.

### Candidate M implementation evidence

Candidate M's contract foundation is commit `88e8bb97b`.
Its generic architecture-specific node failure records,
two-author lifecycle,
12-challenger role wave,
selection,
restart,
cancellation,
and test surface are commit `562e9948b`.
The detached-worktree GFP harness and its command-scoped mise trust boundary are commit `717bd8a08`.
All three commits are present on the GitHub branch
`prototype/translation-repair-finite-pipelines`.

Targeted type-aware lint completed with zero warnings and zero errors.
Type checking,
Candidate M tests,
Candidate L regression tests,
and Candidate K regression tests passed after rebuilding `dist/`.
Full package `buildAndTest` completed with 880 runner-prefixed `PASS` verdicts and zero `FAIL` verdicts.

The reproducible harness is
`package/module/translation-repair/src/corpus-run/prototype-risk-challenger-gfp.ts` (removed before 2026-10-18) and its
three named helper modules.
Its source-set SHA-256 is
`903b1bd54ffdb86d99b5f0f4f35890f4122ca1d54c9e5cf041d59ff7991bd002`
and is recorded by the adjacent sanitized JSON summary.
The adjacent sanitized JSON summary is commit `d9305094f` and retains only source identity,
mutation names,
gate phases,
exit codes,
restored-baseline status,
publication `dist` before and after identity,
and leftover disposable-worktree count.
The harness proved targeted-test detection for seven restored mutations:

- clean or defect finding cardinality;
- source-plan and schema binding helper;
- runtime schema reconstruction against persisted plan identity;
- Candidate M restart failure-category whitelist;
- author admission failure-category persistence;
- strict per-role nonself family floor;
- between-wave cancellation before challenger-plan creation.

Every mutation rebuilt successfully and then failed the targeted Candidate M test gate.
The restored rebuild and targeted test passed.
After the architecture-specific failure-category refactor,
an independent advisor's sole blocker was that the newest helper and restart-test edits had not yet passed the required gates.
Those targeted gates,
GFP controls,
and the full package gate subsequently passed.
### Candidate M calibration preflight

The public calibration artifact is pinned by byte digest,
not by mutable gist identity.
Its revision-specific GitHub location is
[Candidate M calibration revision `b6c9fb5e`][candidate-m-calibration-gist].
The exact harness SHA-256 is
`6f0004e010d1477c3e2eea84287d03d21724495d6c2595d8fceef9131b3bb952`.
The same revision retains sanitized envelope generators,
exact tokenizer counts,
and no-key preflight metadata without corpus wording,
image bytes,
credentials,
or provider payloads.

Pinned official tokenizer artifacts measured these complete textual requests:

- Qwen author:
  15,120 tokens;
- MiniMax author:
  14,355 tokens;
- Qwen challengers:
  at most 60,320 tokens;
- GLM challengers:
  at most 55,556 tokens;
- MiniMax challengers:
  at most 51,974 tokens.

Each request carries the same pinned 2048 by 2048 page image that reached its corresponding Hyper route during Candidate
L.
The live harness exact-tokenizes every actual request before transport and rejects any challenger exceeding its same-model,
same-image retained-candidate ceiling.
The live catalog pins context windows of 1,000,000 tokens for Qwen,
1,048,576 for GLM,
and 512,000 for MiniMax,
plus complete canonical catalog-row digests.

A fresh download from that revision matched the harness digest.
Its final no-key run reached absent-key failure digest
`5e84ae43d4940471c17dded79af1e4c64f031f1099d76d818cf776a793f07ca2`,
emitted no stack or local path,
and left the calibration output root absent.
It pinned the prototype commit,
complete built MJS tree,
entry artifact,
source,
archive,
image,
full catalog rows,
manifest,
review plan,
provider routes,
14 exact node bindings,
and deterministic fixture candidates.
An independent final review found no remaining launch blocker when the live invocation omits the already-used preflight
output path.

The live run must set `umask(0o077)`,
require directory mode `0700` and file mode `0600`,
and verify those modes recursively on success or failure.
It is authorized for one spend only.
If Candidate M fails,
all further implementation stops and the redesign moves directly to the timestamped retrospective and complete takeover
documentation.

### Candidate M calibration result

Candidate M is rejected after its single pinned-Carena live calibration.
The exact GitHub harness,
prototype,
manifest,
corpus,
built tree,
source,
archive,
image,
catalog,
route,
schema,
and node bindings matched the preflight pins.
No non-default reasoning,
thinking,
effort,
temperature,
or retry parameter was sent.
Every request carried the page image.

The run reached all 14 terminal static nodes:

- eight provider exchanges;
- four completed nodes;
- four spent-unusable nodes;
- six deterministic skips.

Qwen produced the sole admitted candidate.
MiniMax's author stopped at 32,000 output tokens.
Qwen's fidelity challenge completed clean but was self-clean and could not qualify.
Qwen's publication-language challenge failed exact anchor admission.
Both GLM challenges stopped at 32,000 output tokens.
Both MiniMax challenges completed clean.

Only MiniMax supplied clean nonself evidence for each role.
The selected private fallback therefore recorded
`evidenceFloorMet: false` and `productionEligible: false`.
It had no admitted dissent,
with Qwen and GLM recorded as abstaining verifier identities.

The run's own restart and one independent offline restart each made zero transport calls and reproduced the persisted
result.
The independent replay left the 43-file pre-post-audit runtime tree byte-identical.
All private output-root directories and files are mode `0700` and `0600`,
respectively.

Complete reading covered all 59 source lines,
61 archive lines,
57 candidate lines,
and the 2,048 by 2,048 page image.
Front matter,
contributor identity,
link,
media reference,
and footnote destination survived.
The candidate was independently rejected for publication-language,
source-structure,
fidelity-omission,
actor-attribution,
and unsupported-emphasis defect classes.
The evidence-floor failure alone already made Candidate M a failed calibration under its settled contract.

The full timestamped disposition,
A through M retrace,
private evidence paths,
and future hypotheses are in
[`translation-repair-redesign-failure-2026-09-01.md`](../audit/translation-repair-redesign-failure-2026-09-01.md).
No further implementation is authorized.
Candidate M and every prior tested candidate remain ineligible for production.
No spent or skipped Candidate M prompt may be dispatched later.

[candidate-m-calibration-gist]: https://gist.github.com/Aquaticat/6ff4fd2f600fc257064aa32eee4c9753/b6c9fb5ebf943a5cd8cfeebbb4c6a2fbbc1e6654
