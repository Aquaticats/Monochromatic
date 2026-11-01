# Translation repair interfaces: Candidates E to G

Part of the [interface comparison index](translation-repair-interface-comparison.md).

## Current stop condition

Historical design evidence only.
Candidate M failed on 2026-09-01.
No candidate prompt may be retried,
and no successor implementation is authorized by this file.

## Candidate E proposal: quote-bound conditional shell adoption

Candidate E reuses immutable shell,
source-echo refusal,
presentation-artifact refusal,
and transaction runtime,
but discards fixed-priority quality selection and unconditional serial adoption.
Quality evidence is concrete defect class located to fixed candidate and slot,
with exact source and candidate anchors.
No auditor may return naturalness score or unlocated preference.

### Candidate E1 prime fixed graph

1.  Derive immutable shell and fixed-key slot schema.
2.  Ask 3 independent complete authors concurrently.
3.  Ask 3 independent full-contract auditors concurrently to inspect all complete candidates against source,
    archive,
    and page-referenced images.
4.  Validate every finding anchor deterministically against source slot and candidate slot.
5.  Confirm finding only when at least 2 usable auditors agree on candidate,
    slot,
    and concrete defect class.
6.  Select baseline by lexicographic defect tuple:
    fewer severe confirmed findings,
    then fewer total confirmed findings,
    then fixed manifest priority.
7.  Ask one complete resolver to correct selected confirmed-defect slots only.
    Resolver receives no alternate candidate.
8.  Refuse resolver response if any unlocated slot changes or deterministic candidate guard fails.
9.  Ask 3 post-auditors concurrently to inspect baseline and resolution.
10. Adopt resolution only when
    at least 2 post-auditors are usable and resolution finding set is strict subset of baseline finding set.
    Otherwise publish baseline byte-for-byte.
11. Atomically publish and read back selected complete document.

Payload ceiling is 10 in 4 dependency waves.
Every author and resolver returns complete candidate or has no effect.
Auditors cannot withhold existing complete candidate.
No response creates new node,
retry,
Gate,
or work queue.

Finding schema has fixed candidate properties and bounded arrays.
Each entry contains fixed-key slot,
concrete defect class,
exact nonempty source anchor,
and exact nonempty candidate anchor.
Classes are:

- wrong meaning
- omission
- unsupported addition
- identity or attribution
- actor or reference
- chronology
- technical or legal term
- grammar or usage
- tense
- register
- source-language calque

Severe tuple component counts wrong meaning,
omission,
unsupported addition,
identity or attribution,
actor or reference,
chronology,
and technical or legal term.
Second tuple component counts every confirmed finding.
No free-form severity,
score,
or naturalness verdict enters selection.
Quorum agrees on candidate,
slot,
and defect class rather than exact anchor text;
anchors prove each claim is located while allowing auditors to quote different spans of same defect.
One invalid or unbound finding makes whole auditor response unusable.
Strict response loss is preferred over silently pruning unauditable claims;
reduced-quorum table then preserves finite baseline behavior.
Total finding comparison can still reward shared auditor blind spots,
so retained-output calibration must reproduce known human comparisons before graph spend.

Reduced-quorum behavior is manifest-owned:

- 3 usable auditors require any 2 agreeing
- 2 usable auditors require both agreeing
- 1 or 0 usable auditors produce no confirmed findings and fixed-priority baseline
- fewer than 2 usable post-auditors always preserve baseline

Post-audit adoption compares whole documents only.
Per-slot mixing is forbidden because it would recreate transaction conflicts and lose whole-page voice.
Resolution must change at least one confirmed-defect slot and no other slot.
Resolution finding keys must be strict subset of baseline finding keys;
new confirmed defect or unresolved key preserves baseline.

### Alternatives

#### E1 prime pros

- replaces blind author priority with quote-bound comparative evidence
- prevents D1.3 regression through conditional whole-document adoption
- resolver changes are limited to auditor-located slots
- auditor failure preserves complete baseline and never suspends publication

#### E1 prime cons

- 10 payloads and 4 waves increase latency
- auditor agreement can miss shared model blind spots
- terse bounded matrices need provider conformance validation
- resolver may fail to correct located defects and then has no effect

#### E3 pros

- author comparison fixes D1 blind selection with fewer waves
- no resolver means no later-output regression

#### E3 cons

- every measured first-wave author candidate remained below publication bar
- selection alone cannot exceed measured author ceiling

#### E2 pros

- authors receive shared source-risk context before writing
- one brief can focus all candidates on difficult source relations

#### E2 cons

- repeats Candidate C brief-before-prose dependency
- one brief defect correlates across every downstream author
- brief producer does not establish complete candidate

Ranking:
E1 prime > E3 > E2,
because E1 prime addresses both blind selection and later regression;
E3 addresses blind selection but cannot repair measured author defects;
E2 repeats rejected shared-preparation dependency.
Candidate E1 prime entered retained-output calibration before any new candidate publication spend.

### Rejected Candidate E1 prime calibration

Candidate E1 prime dispatched 9 zero-retry auditor payloads in one dependency wave against retained D outputs.
The bounded run ended after 1,170,562 milliseconds.
Strict whole-response admission made only 3 responses usable:

- D1 comparison had 0 usable auditors and fell back to known-inferior primary author
- D1.3 post comparison had 1 usable auditor and no quorum
- seeded comparison had 2 usable auditors and confirmed all 3 planted defects
- 5 responses parsed as JSON but failed caller guard
- 1 GLM D1.3 response ended with `StreamCutShortError`

Every parsed provider reply was already stored privately by prompt digest.
Digest-bound replay proved all 5 caller-guard failures were structurally valid ballots.
Across those ballots,
72 of 80 findings had exact source and candidate anchors and unique candidate-slot-class keys.
Strict response admission discarded those 72 located findings together with 3 duplicates,
3 source-anchor misses,
and 2 candidate-anchor misses.
Candidate E1 prime is rejected under whole-response admission;
it cannot proceed to resolver or publication spend.
Private evidence is preserved at
`~/Downloads/Carena0442-candidate-E1-prime-calibration-rejected-20260830/`.

### Candidate E1 double-prime admission

Candidate E1 double-prime changes audit admission and decision semantics rather than changing prose prompts:

1.  Structurally valid fixed-candidate ballot becomes usable.
2.  Each finding is independently admitted only when exact source and candidate anchors bind within named slot.
3.  Duplicate candidate-slot-class finding and unbound finding are excluded and recorded by candidate,
    slot,
    class,
    and deterministic rejection reason.
4.  Each usable auditor independently applies severe-count,
    total-count,
    then manifest-priority tuple to complete author candidates.
5.  At least 2 auditors must select same baseline.
    Otherwise fixed-priority baseline is published with explicit `evidenceFloorMet: false`;
    such run cannot qualify as architecture quality evidence.
6.  Resolver receives union of admitted located findings for selected baseline and may change only those slots.
7.  Each usable post-auditor independently establishes nonempty-baseline strict-subset relation.
8.  At least 2 post-auditors must establish strict subset before resolution can be adopted.
    Otherwise baseline survives byte-for-byte.

Auditor prompts omit manifest candidate priority.
Auditors may evaluate output from same model family,
but exact anchors,
independent role prompts,
and 2-vote decision prevent one self-review from controlling selection or adoption.
Finding-empty ballot abstains rather than voting fixed priority.
Every post-auditor ballot persists baseline keys,
resolution keys,
new resolution keys,
approval,
and whether auditor shares resolver model identity.
A third dissent is recorded but does not veto 2 independent approvals;
this implements stated 2-auditor adoption rule rather than silently changing it to unanimity.

Zero-spend digest-bound replay over exact E1 prime provider replies admitted every completed ballot:

- D1 had 3 usable auditors;
  votes were 2 for Qwen fallback,
  with one empty admitted ballot abstaining,
  reproducing complete-page comparison
- D1.3 had 2 usable auditors;
  each located at least 1 concrete defect and neither established resolution strict subset,
  reproducing copy-editor rejection
- seeded arm had 3 usable auditors and retained quorum confirmation for every planted slot

Replay completed in 48 milliseconds without provider calls.
It calibrates audit admission and decisions only.

Full scripted graph fixes ceiling at 10 payloads across 4 waves and proved:

- base run selected fallback author on 3 defect-based votes,
  constrained resolver to located slot,
  received 3 post approvals,
  and adopted complete resolution
- all-author failure terminated with `ProductionUnavailableError` after 3 author nodes and no later calls
- no usable author-audit ballot published fixed-priority baseline with `evidenceFloorMet: false`
- invalid or unlocated resolver,
  invalid post wave,
  and regression post wave each preserved baseline byte-for-byte
- 2 usable approving post auditors plus 1 unusable auditor adopted;
  2 approvals plus 1 located dissent also adopted and retained dissent keys
- completed restart left 31 non-result files byte-identical,
  all 10 node mtimes unchanged,
  and normalized result byte-identical
- resolver SIGTERM restart retained exact `CallerAbort`,
  did not redispatch,
  and published baseline
- post SIGTERM recorded 3 `CallerAbort` nodes;
  restart preserved their digests and baseline
- post SIGKILL left 3 dispatched nodes;
  restart converted all to `IndeterminateTransmission` and preserved baseline
- types,
  local controls,
  and complete suite passed with 871 suite verdicts and no failures

Restored GFP mutations separately proved located-only resolver change,
nonempty resolver change,
2-vote post floor,
strict-subset new-key exclusion,
empty-ballot abstention,
and hidden-priority refusal.
Earlier admission mutations proved structural slot membership,
source and candidate anchor binding,
duplicate pruning,
and author-selection vote floor.
Every mutation failed named control and restored suite again passed 871 verdicts with no failures.

### Rejected E1 double-prime Hyper-only output

Fresh Hyper-only run completed in 1,142,906 milliseconds and published complete Qwen baseline.
Run proved one-provider bounded operation but did not produce publication-ready page:

- Qwen author completed
- Kimi author consumed 16,000 output tokens and returned empty content,
  then structural guard recorded schema mismatch
- GLM author and relation auditor failed locally with `NoProviderForModelError`
- 2 usable auditors inspected only Qwen candidate and both selected it
- runtime incorrectly marked `evidenceFloorMet: true` although no comparative candidate existed
- auditors located findings in 7 slots
- resolver completed but changed 13 slots,
  including 6 outside located set,
  so located-only gate rejected whole response before post audit
- publication correctly preserved Qwen baseline byte-for-byte

Complete-page reading rejected baseline for concrete defects in existing classes:
unsupported addition,
source-detail omission,
role generalization,
technical-term misuse,
actor and device-reference ambiguity,
wrong causal basis,
chronology wording,
and lost rhetorical repetition.
Source shell,
front matter,
contributor link,
footnote relation,
page media reference,
Han-script refusal,
presentation-artifact refusal,
and publication readback remained correct.
Page image showed memorial,
trans identity,
and chemistry context consistent with translated content and no additional text obligation.
Private evidence is preserved at
`~/Downloads/Carena0442-candidate-E1-double-prime-hyper-only-output-review-20260830/`.

This rejects E1 double-prime output and its current evidence-floor predicate,
not reusable architecture.
Candidate E1 triple-prime correction requires:

- at least 2 usable author candidates from distinct model identities before comparative floor can pass
- at least 2 agreeing auditor model identities;
  this necessarily includes external vote because selected author has 1 model identity
- `evidenceFloorMet: false` publication when any floor condition fails
- Hyper-only reserve author and auditor selected only after structured vision and complete-document validation
- persisted schema-mismatch detail distinguishing unparseable output from caller-guard rejection
- evaluation of 2 concurrent fixed resolver seats as alternative to one over-eager resolver nullifying every located fix

No defect wording from this run may enter new prompts.
Retained output remains negative calibration evidence only.
### Rejected E1 triple-prime under active Hyper roster

E1 triple-prime comparative-floor correction is implemented and tested:

- candidates and ballots persist model identity
- evidence requires at least 2 votes,
  2 candidate model identities,
  and 2 agreeing auditor model identities
- one candidate or repeated auditor identity cannot satisfy floor
- schema mismatch preserves stable reason distinguishing guard rejection,
  invalid JSON,
  truncated thinking,
  and other mismatches
- controls,
  retained replay,
  scripted one-author behavior,
  restored build,
  and 871 suites pass
- GFP independently proves candidate diversity,
  auditor diversity,
  parser-reason,
  and persistence guards load-bearing

Hyper reserve evaluation then exhausted current active roster.
MiniMax M3 was only distinct image-capable integrated model.
Its quote-bound auditor completed strict response with 4 admitted findings and 1 duplicate pruned.
Its complete author consumed requested 32,000 output tokens,
ended `max_tokens`,
and returned unparseable JSON missing final outer brace.
Node correctly persisted `spent-unusable` with `schema-mismatch` and `unparseable-json` detail.
Caller repair is forbidden,
and prompt uniqueness forbids redispatching same model plus canonical author prompt with changed ceiling.
Schema-invalid response remains operational evidence and was not read as page-quality evidence.

No current active Hyper model distinct from Qwen and Kimi satisfies image,
complete-author,
and quote-bound-auditor contracts.
Full triple-prime graph run would therefore begin from known failed comparative-author precondition:
with Hyper alone it must set `evidenceFloorMet: false`,
skip conditional repair,
and republish same Qwen baseline already rejected by complete-page reading.
Further resolver controls cannot repair absent comparative evidence.
Running that graph would spend payloads without changing admissible outcome,
so it is not performed.

Private MiniMax evidence is preserved at
`~/Downloads/Carena0442-minimax-reserve-validation-rejected-20260830/`.
Technology vet report is
`doc/audit/tech-candidate-e-hyper-reserve-model-vet-2026-08-30.md`
on main branch.

### Rejected E1 triple-prime after Hyper roster expansion

A frozen roster-expansion evaluation then screened every out-of-roster Hyper vision row.
Kimi K2.6 and K2.7 Code exited before spend
because direct-service documentation did not support required named forced-tool schema
under default or mandatory thinking.
Hyper gateway translation remained unprobed,
so this is conservative integration screening rather than observed gateway incompatibility.
Owner-excluded Qwen3.8 Max remained excluded.

Five Qwen alternatives each received one canonical complete-author payload and one quote-bound D1-auditor payload.
All 10 zero-retry Hyper arms completed strict schema in one dependency wave.
Every compiled author page was read completely against pinned source,
archive,
and page-referenced image.
One analyst's complete-page review found all 5 failed publication-ready author hard gate:
1 hid severe semantic truncation behind populated slot keys,
3 retained concrete grammar,
reference,
provenance,
or source-carryover defects,
and strongest complete candidate introduced unsupported actor gender plus fidelity and English defects.
Strongest auditor cannot satisfy reserve requirement because same model's author failed.
All runtime survivors also remain Qwen-family siblings,
so distinct ids would not prove cross-family independence.

No roster candidate is recommended.
Prompt uniqueness forbids repeating any model plus canonical prompt pair from this run.
Private artifacts and located review are retained under
`~/Downloads/Carena0442-hyper-roster-expansion-validation-20260830/`.
Full technology evidence is in
`doc/audit/tech-candidate-e-hyper-roster-expansion-vet-2026-08-30.md`.

Candidate E1 double-prime and triple-prime are rejected under tested active-roster and expansion designs.
Reusable components remain immutable shell,
per-finding admission,
model-diverse comparative floor,
located-only resolution,
strict-subset adoption,
finite runtime,
restart,
abort identity,
and provider-isolation controls.
No Candidate E output is eligible for publication or production integration.

Final Candidate E learning order:
rejected expanded E1 triple-prime > rejected active-roster E1 triple-prime >
rejected E1 double-prime > E3 > E2 > rejected E1 prime.
Expanded triple-prime proves finite roster exhaustion;
active-roster triple-prime corrects false comparative evidence but lacks candidate diversity;
double-prime recovers located evidence but falsely passed one-candidate floor;
E3 cannot exceed measured author ceiling;
E2 repeats Candidate C dependency;
E1 prime discarded most valid ballot evidence.

## Measured selection outcome

No replacement architecture is selected.
Tested deployment candidates across A through E have zero survivor under fixed completion and quality contract.
A1 ended in `StreamCutShortError` and A2 in bounded `ProductionUnavailableError` before adoption.
B and C failed complete-page quality.
D failed complete-page quality after repeated bounded refinements.
Active-roster E lacked qualifying reserve and did not run full triple-prime graph;
expansion E produced distinct ids but every tested author failed complete-page quality.

Production replacement planning remains blocked.
Continuation requires fresh finite architecture definition based on measured failures,
not another serial editor,
retry loop,
Candidate E payload,
or relaxation of publication-ready output.
Semantic slot truncation,
identity fabrication,
and same-family correlation are now explicit design inputs.

## Next definition round

### Rejected Candidate F: witness-switched donor assembly

Candidate F would remove Candidate E resolver
and let deterministic compiler replace only whole immutable-shell slots copied from already complete candidates.
Auditors would compare baseline and donor defect sets;
a donor slot would be eligible after 2 independent per-auditor strict-subset findings,
then 2 post auditors would compare baseline and composite.

#### Pros

- Resolver cannot author unsupported prose.
- Every unchanged slot remains byte-identical.
- Donor substitutions preserve shell structure and bounded authority.
- Static graph can fit 5 retained authors,
  3 comparative auditors,
  and 2 post auditors in 3 dependency waves.

#### Cons

- Reported defect sets are open-world under-approximations.
  Missing donor finding is not evidence donor is clean.
- Two auditors can miss same new donor defect,
  especially among correlated model families.
- Whole-slot mixing can damage document voice and cross-slot reference.
- Empty finding set can mean abstention or missed evidence rather than clean coverage.
- Candidate identity must be removed from comparison key,
  while slot plus defect class is too coarse to distinguish source obligations.

Candidate F is rejected at design gate.
It improves mutation safety over E but cannot turn missing findings into quality proof.
A closed-world source-obligation checklist and explicit checked-clean states
would change interface enough to be different candidate.

### Candidate G proposal: verified realization ledger

Candidate G moves seam to whole-candidate admission against closed-world source obligations.
No prose resolver,
donor splice,
postdraft editor,
or generated work exists.
Model-produced realization is claim;
independent verification is evidence,
not mathematical proof.

```ts
export type SourceSpan = {
  readonly startOffset: number;
  readonly endOffset: number;
  readonly digest: string;
};

export type SourceObligation = {
  readonly id: ObligationId;
  readonly kind: 'clause' | 'relation' | 'identity' | 'link' | 'media' | 'format' | 'archive-authority';
  readonly sourceSpans: readonly SourceSpan[];
  readonly relationEndpoints: readonly ObligationId[];
  readonly targetCardinality: 'one-or-more' | 'shell-owned';
  readonly authority: 'source' | 'archive-allowed' | 'shell-locked';
  readonly evidenceDigest: string;
};

export type TargetAnchor = {
  readonly slotKey: SlotKey;
  readonly startOffset: number;
  readonly endOffset: number;
  readonly digest: string;
};

export type RealizedCandidate = {
  readonly candidateId: CandidateId;
  readonly modelId: ModelIdentity;
  readonly candidateDigest: string;
  readonly slots: Readonly<Record<SlotKey, string>>;
  readonly realization: Readonly<Record<ObligationId, readonly TargetAnchor[]>>;
};

export type CandidateVerification = {
  readonly candidateId: CandidateId;
  readonly candidateDigest: string;
  readonly obligations: Readonly<Record<ObligationId, {
    readonly status: 'preserved' | 'defect';
    readonly verifiedTargetAnchors: readonly TargetAnchor[];
  }>>;
  readonly globalChecks: Readonly<Record<GlobalCriterion, 'clean' | 'defect'>>;
  readonly findings: readonly LocatedFinding[];
};

export type VerifierBallot = {
  readonly verifierModelId: ModelIdentity;
  readonly manifestDigest: string;
  readonly candidates: Readonly<Record<CandidateId, CandidateVerification>>;
};
```

Clause obligations carry contiguous parsed source spans.
Relation obligations name endpoint obligation ids and source connective spans.
Identity obligations bind canonical person and contributor forms.
Link and media obligations bind exact destination or asset digest.
Format and archive-authority obligations identify shell-owned or explicitly archive-allowed regions.
Target offsets disambiguate repeated phrases;
string equality alone never locates anchor.
Located finding names exactly 1 obligation id or global criterion,
defect class,
manifest-owned source spans,
and zero or more exact target anchors.
Runtime assigns candidate id,
model id,
and candidate digest after validating raw author slots and map;
author cannot self-assert those bindings.

#### Fixed call graph

1.  Deterministically split source shell text into bounded clause obligations from parsed source offsets.
    Add identity,
    relation,
    link,
    contributor,
    media,
    formatting,
    and archive-only obligations without model work.
2.  In wave 1,
    ask fixed author roster concurrently for complete immutable-shell slot map plus total realization ledger.
    Every author owns full fidelity,
    completeness,
    identity,
    grammar,
    reference,
    tense,
    relation,
    and register contract.
3.  Admit candidate only when every non-shell obligation has required anchor cardinality,
    every target offset and digest binds exact declared slot,
    every shell key appears once,
    and existing deterministic guards pass.
    Mapping remains untrusted semantic claim.
4.  In wave 2,
    ask fixed verifier roster concurrently to inspect every admitted whole candidate against complete source,
    archive,
    realization ledger,
    and all page-referenced images.
    Candidate aliases hide author model and priority.
    Every verifier independently marks every candidate-obligation pair and every global criterion.
5.  Admit ballot only when verifier identity,
    manifest digest,
    every candidate digest,
    full candidate-obligation matrix,
    and global-check matrix match manifest.
    Preserved status requires verifier-confirmed exact target offsets.
    Defect status requires finding linked to obligation or global criterion.
    Omission may have no target anchor;
    unsupported addition uses target-only global finding.
    Empty,
    partial,
    or duplicate ballot abstains.
6.  Candidates with 2 complete clean ballots from distinct verifier model identities rank ahead.
    Dissent persists.
    Hidden fixed priority selects within same evidence class.
    Same-family identity remains correlated evidence,
    not independence proof.
7.  Calibration writes selected whole candidate only to private review root,
    with `evidenceFloorMet: false` when no candidate has 2 clean ballots.
    Verifier failure cannot withhold structurally usable private candidate.
    Calibration never calls production `PublicationPort` or returns `CompletedEntry`.
    No candidate text is mixed or revised after authorship.

Prototype ceiling is 7 payloads:
4 complete authors and 3 complete-candidate verifiers in 2 dependency waves.
Before live spend,
serialized worst-case author and verifier schemas must fit 32,000-token project ceiling with measured headroom.
Qwen3.8 Flash previously used 27,922 completion tokens to audit only 3 candidates,
so unmeasured 4-candidate matrix is blocker rather than assumed fit.
Manifest binds exact obligation ledger,
author and verifier identities,
source,
archive,
shell,
media,
schemas,
provider mode,
and fixed priority.
Each model plus canonical substantive prompt receives at most 1 provider payload.
One wet provider supplies whole manifest;
cross-provider responses are never correctness dependency.

#### Stage postconditions

Deterministic obligation ledger is total over parsed source text but makes no semantic claim.
Author realization anchors cannot certify their own meaning.
Verifier record is structurally complete only when it covers every candidate-obligation pair and every global criterion,
binds candidate digests,
and carries verifier model identity.
Verifier independently checks semantic source-to-target realization;
author map is only navigation claim.
Candidate selection never treats missing finding as checked-clean.
Verifier timeout,
refusal,
or malformed response has no effect and never suspends calibration.
Every model node receives `photo1.webp` and never presentation-only profile image.
Restart,
indeterminate transmission,
caller abort,
cache eligibility,
and prompt uniqueness reuse Candidate E proven controls.

#### Pros

- Makes unmapped source clauses visible even when every slot key is populated.
- Whole candidate preserves voice and cross-paragraph coherence.
- Closed-world coverage replaces Candidate E open-world defect-count comparison.
- Verifiers inspect output-specific identity,
fidelity,
and English defects before selection.
- No model can create repair work or mutate selected prose.
- Finite 2-wave graph remains highly parallel.

#### Cons

- Exact anchor can support wrong semantic mapping;
  verification remains fallible judgment.
- Author ledger expands output and may increase truncation risk.
- Valid mapping can falsely attach omitted meaning to unrelated exact anchor.
- Large all-candidate verifier schema may exceed output ceiling.
- Fixed-priority private review below evidence floor does not solve production normal-return contract.
- Same-family verifier agreement remains correlated even with distinct model ids.
- Production eligibility still requires complete-page acceptance across repeated isolated runs.

### Next-round ranking

Ranking:
Candidate G > rejected Candidate F.
G ranks above F because explicit total coverage can distinguish checked-clean from unobserved,
whereas F treats absent donor findings as improvement evidence.
F retains safer mutation than Candidate E but cannot close donor-defect blind spot.

Candidate G enters scripted controls and retained-output calibration only.
Controls must cover omitted clause,
fabricated identity,
wrong relation,
unsupported addition,
repeated anchor,
archive-regurgitated text,
partial and empty ballot,
dissent,
all-verifier failure,
and known Qwen3.6 Plus truncation with concise valid negative control.
Restart,
indeterminate transmission,
cache binding,
prompt uniqueness,
provider isolation,
and exact caller abort must pass for both waves.
It is not selected for production.
Live calls are authorized;
Candidate G did not proceed before schema-size measurement,
implementation review,
GFP controls,
and advisor review.

### Candidate G envelope disposition

Candidate G failed the compact schema-stress gate used on 2026-08-31.
The output-envelope boundary resolution supersedes that gate as an architecture rejection criterion.
No live Candidate G payload was sent.

The pinned Carena shell has 23 slots and 134 obligations.
The strict author schema serialized to 2,994 bytes;
the four-candidate verifier schema serialized to 7,397 bytes.
Compact response witnesses excluded optional JSON whitespace,
which is not bounded by JSON Schema.

Four compact schema-valid stress witnesses were serialized.
They are not claimed as mathematical tokenizer extrema.
All four pass committed structural guards,
but none is candidate-admissible or evidence-admissible:
the author witnesses violate target cardinality or presentation requirements,
the lower verifier witness has defect statuses without findings,
and the upper verifier witness repeats findings against clean statuses.
Their measurements were:

- author lower witness:
  7,294 bytes,
  2,432 tokens under the project three-bytes-per-token estimate,
  and 2,528 tokens under the official Qwen3.6-27B tokenizer;
- author upper stress witness:
  2,820,201 bytes,
  940,067 estimated tokens,
  and 2,317,000 Qwen tokens;
- four-candidate verifier lower witness:
  93,488 bytes,
  31,163 estimated tokens,
  and 46,671 Qwen tokens;
- four-candidate verifier upper stress witness:
  437,076 bytes,
  145,692 estimated tokens,
  and 139,331 Qwen tokens.

The witness SHA-256 digests in the same order are:

- `6f931d1f874d583816823f8b2e68036fb3d8ca22cff04b93d1dc5e9f190ed38e`;
- `cc8ec3c3902a4090966a892036a54fceb67471d44fbf24e8c646aeb636179c22`;
- `3ca1631bdfd98dd894a567386ad09d70bfc2278c5b3f44621cb8f19780d73f59`;
- `88039b143e279111d27b7f7b3292314ba2694a39fd775c11e2d49b18bed011aa`.

The exact tokenizer was `Qwen/Qwen3.6-27B` at commit
`6a9e13bd6fc8f0983b9b99948120bc37f49c13e9`,
loaded directly from `tokenizer.json` through Python `tokenizers` 0.22.2,
with no chat-template wrapper.
The completion ceiling is 32,000 tokens.
The author upper stress witness tokenized with Qwen3.6-27B exceeded it by 2,285,000 tokens.
That proves the schema permits responses outside the provider envelope.
It does not prove that a working model can return those responses under the manifested transport cap,
and deterministic admission still gives truncated output no effect.
The verifier token counts remain Qwen-specific evidence and are not attributed to MiniMax or DeepSeek tokenizers.
Measurement tooling and structural controls are committed as `a15bd65df`.

A private compact-wire experiment kept three disjoint author and finding anchors,
made runtime own target digests,
and replaced repeated ids with manifest indexes.
It reduced usable witnesses,
but a source-derived per-slot prose cap remained substantively unproved,
and schema-valid control,
escape,
and emoji arms still exceeded the ceiling.
The experiment was reverted rather than weakening author authority or declaring semantic invalidity a transport bound.

Candidate G is not production-selected because it has no qualified author.
Its four-candidate verifier completion headroom is unproven under planned model tokenizers and blocks pre-spend.
Its reusable findings are closed-world obligation coverage,
manifest-ordered status arrays,
runtime-owned digests,
and the distinction between schema maximum and semantically acceptable output.
Any Candidate G continuation must measure complete provider-returnable verifier responses without capping publication-quality prose or dropping split-anchor evidence.
