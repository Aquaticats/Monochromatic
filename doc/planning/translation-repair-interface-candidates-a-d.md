# Translation repair interfaces: Candidates A to D

Part of the [interface comparison index](translation-repair-interface-comparison.md).

## Current stop condition

Historical design evidence only.
Candidate M failed on 2026-09-01.
No candidate prompt may be retried,
and no successor implementation is authorized by this file.

## Candidate A: Serial accountable document producers

### Seam and interface

Seam is adoption of complete whole-document revisions.
Every model node can create full document when no prior candidate exists,
or revise prior complete document under one concrete responsibility.
No model merely diagnoses defects that another model must later rescue.

```ts
export type EditorialPlan = {
  readonly brief: DocumentBrief;
  readonly producers: readonly [
    WholeDocumentProducer,
    FidelityProducer,
    ExpressionProducer,
    ContinuityProducer,
  ];
};
```

### Fixed call graph

1.  Compile source-backed brief deterministically.
2.  Ask whole-document producer for complete draft.
3.  Ask fidelity producer for complete source-faithful revision.
4.  Ask expression producer for complete grammar,
    usage,
    and register revision.
5.  Ask continuity producer for complete reference,
    tense,
    chronology,
    and paragraph-relation revision.
6.  After each response,
    adopt complete candidate only when deterministic obligations pass;
    otherwise preserve prior candidate byte-for-byte.
7.  Publish final surviving candidate atomically.

Provider payload ceiling is 4.
No response can increase it.
If first producer does not return usable document,
remaining statically named producers receive source,
archive,
brief,
and empty-prior marker and can establish first candidate.
Any producer in fallback mode assumes full concrete quality contract;
its narrower role applies only when prior complete candidate exists.
Once candidate exists,
node failure cannot withhold it.

### Stage postconditions

Brief names every known identity,
link,
media claim,
front-matter field,
line structure,
and supported archive-only context.
Each producer returns one complete document,
not score,
verdict,
finding-only report,
alternative slate,
or patch requiring another model.

Responsibilities are disjoint.
Fidelity owns wrong meaning,
omission,
addition,
identity,
and attribution.
Expression owns grammar,
usage,
unclear expression,
and concrete register mismatch.
Continuity owns unclear reference,
tense,
chronology,
repetition,
and paragraph relation.
Each later producer receives exact prior document and returns complete candidate plus exact edit transaction.
Every edit names prior-text anchor,
source evidence,
and concrete responsibility.
Deterministic application must reproduce complete candidate exactly and rejects undeclared changes.

Deterministic adoption checks coverage,
identity,
syntax,
structure,
links,
media obligations,
and complete-document shape before changing current candidate.
No model Gate follows final producer.

### Pros

- A usable producer establishes output that later node failure cannot withhold.
- Every specialist improves text directly rather than manufacturing blocking work.
- Payload ceiling is 4 with no panel,
vote,
patch round,
or reconciliation call.
- Complete-document boundaries preserve document voice better than independent slice candidates.
- Reuses existing preparation and invariant modules behind one deep adoption seam.

### Cons

- Later whole-document producer can request more edits than responsibility requires.
- Source-anchored edit transactions constrain authority but cannot prove every semantic judgment.
- Serial calls trade some latency for stage accountability.
- If every configured producer becomes unavailable before first candidate,
  infrastructure cannot create translation output;
  fixed roster must exhaust into bounded production-unavailable diagnostic.

### Rejected Candidate A evidence

First throwaway A1 implemented draft,
three finding-only auditors,
and later patch commit.
Carena run reached complete draft and two audits,
then fidelity auditor exceeded 360,000-millisecond exchange deadline.
A1 returned suspended after 469 seconds and wrote no final page.
Its explicit resume was stopped after user clarified that this behavior leaves caller hanging.

A1 therefore falsified audit-dossier interface before output reading:
a non-producing auditor could withhold complete producer output.
A1 result remains operational evidence at
`~/temp/agent/prototype-Carena-A-accountable-editor-20260830/result.json`,
but is not quality evidence and must not be averaged with corrected Candidate A.

Corrected A2 ran four serial complete-document producer nodes on fresh Carena root.
Both providers were wet at preflight.
Four prompt-payload artifacts and four `SPEND` records exist;
provider-internal request attempts were not counted.
Process exhausted graph with exit code 1 after 432 seconds;
recorded invocation duration was 429,690 milliseconds.
No candidate was adopted and no accepted or published page existed:

- whole-document response document field introduced structural parse regression
- fidelity fallback response document field dropped one contributor form
- expression fallback response document field introduced footnote relation defect
- continuity fallback response document field had invalid front matter

A2 did not leave suspended work,
but bounded `ProductionUnavailableError` still left caller without output.
This falsifies Candidate A against user requirement.
Raw responses remain private under
`~/temp/agent/prototype-Carena-A2-serial-producers-20260830/`.
Do not score,
rank,
average,
quote,
or use them as translation-quality samples.
They are operational and contract evidence only.
Candidate A is rejected rather than repaired with another retry or Gate.

## Candidate B: Specification-first translation compiler

### Seam and interface

Seam is compilation from typed source specification to published document.
Translation specification,
not draft,
is source of truth.

```ts
export type TranslationSpecification = {
  readonly units: readonly SemanticUnit[];
  readonly relations: readonly SemanticRelation[];
  readonly identities: readonly IdentityObligation[];
  readonly structure: StructureContract;
  readonly media: readonly MediaObligation[];
  readonly archiveEvidence: readonly ArchiveEvidence[];
};
```

### Fixed call graph

1.  Ask specification author for typed source specification.
2.  Deterministically admit valid units and preserve raw source spans for any unsupported unit.
3.  Ask renderer for complete document plus unit-to-span realization map.
4.  In parallel,
    ask fidelity and omission producer,
    identity,
    structure,
    and media producer,
    and expression and document-relations producer
    for complete revised document and exact specification-linked transaction against same renderer base.
    If renderer was unusable,
    each receives explicit absence marker and can establish first candidate from source and specification.
5.  Validate all transactions independently.
    When base exists,
    merge non-conflicting valid transactions in fixed responsibility order.
    When base is absent,
    adopt first valid complete candidate in that same fixed order.
6.  Publish final surviving candidate atomically.

Provider payload ceiling is 5.
Specification-author failure leaves raw source envelope as renderer input and cannot block document production.
Renderer failure lets each later statically named producer establish first candidate from specification and source.
No response can add work.

### Stage postconditions

Every admitted source span belongs to one semantic unit.
Every unit declares propositions,
relations,
identity references,
and destination obligations.
Raw unsupported spans remain first-class obligations rather than disappearing from invalid specification.
Each document producer returns complete document,
unit-to-span realization map,
and exact edit transaction when prior candidate exists.
Every changed unit and edit anchors immutable raw source span and digest;
model-authored specification alone is never source authority.

Linker verifies realization map,
locked identities,
structure,
media,
coverage,
edit scope,
and complete document before adoption.
No model Gate follows final producer.

### Pros

- Strongest root-cause attack on omissions,
  identity drift,
  source relation loss,
  and visual evidence bypass.
- Quality obligations exist before prose.
- Revisions are linked to concrete specification obligations rather than global opinion.
- Specification is reusable audit artifact and test surface.
- Distinct responsibility producers can correct renderer blind spots directly.

### Cons

- Deterministic semantic-unit derivation is not actually solved for unrestricted Chinese prose.
  Calling it deterministic can encode missed meaning into specification.
- Model-authored specification can be confidently wrong despite raw-span fallback.
- More shallow role interfaces and artifact schema than Candidate A.
- Distinct producers can damage document voice established by renderer.
- Highest migration cost and largest new correctness surface.

### Rejected Candidate B evidence

Candidate B implemented its five-payload graph with one specification author,
one renderer,
and three concurrent complete-document fallback specialists.
Scripted controls proved valid-renderer preservation,
fixed-priority fallback selection,
raw-specification fallback,
restart without redispatch,
image-bearing prompts,
and transaction guards.
The complete unit suite recorded 871 passes and no failures.
Eight guard mutations covered specification identifiers,
realization locators,
transaction replay,
cross-role conflicts,
anchor drift,
fallback priority,
output envelope,
and image transmission.
Initial conflict mutation exposed a control that did not isolate conflict handling.
Commit `98931789a` corrected that control,
a repeated mutation failed it,
and all mutations were restored.
Both providers were wet immediately before live execution.

Fresh Carena execution exhausted all five nodes after 726,466 milliseconds and threw bounded
`ProductionUnavailableError`.
It wrote no candidate or publication receipt:

- specification author completed after 101,952 milliseconds but did not satisfy response schema
- renderer exceeded stream envelope after 214,597 milliseconds
- fidelity fallback stream was cut after 24,686 milliseconds
- authority fallback completed after 170,936 milliseconds but introduced footnote relation defect
- expression fallback completed after 409,737 milliseconds but supplied absent realization locator;
  exact runtime-validator replay also found footnote relation defect

The complete preserved tree contains no `fixed` page.
The two completed fallback documents each carried all 22 manifest source identifiers,
`fallback` mode,
null base digest,
and empty change list.
Those shape facts were insufficient for admission.
All five node records carry distinct structured contract digests.
The reply cache contains only three completed transport replies,
which usage records map to specification,
authority,
and expression nodes.
Cut and overrun nodes have no reply artifact by store design.
The specification schema-mismatch record has neither failure digest nor explicit reply-cache key;
next prototype must close that audit-binding gap.
Candidate B is rejected rather than retried,
repaired from quarantined responses,
or selected without output reading.
Prototype commits are
`de651603d`,
`3c6dbd3c2`,
and
`98931789a`.
Private artifacts remain at
`~/temp/agent/prototype-Carena-B-specification-compiler-20260830/`.
A preserved private copy is at
`~/Downloads/Carena0442-candidate-B-rejected-20260830/`.
Raw responses are operational and contract evidence only.

## Candidate C: Brief-before-prose editorial room

### Seam and interface

Seam is one final whole-document authorship operation informed by upstream specialist briefs.
No model reviews output after it exists.
This candidate moves expertise before prose rather than adding another postdraft catcher.

```ts
export type AuthorshipPlan = {
  readonly sourceBriefAuthor: ModelIdentity;
  readonly structureBriefAuthor: ModelIdentity;
  readonly expressionBriefAuthor: ModelIdentity;
  readonly documentEditors: readonly [ModelIdentity, ModelIdentity];
};
```

### Fixed call graph

1.  Build deterministic document envelope and authority facts.
2.  In parallel,
    ask source specialist for proposition and ambiguity brief,
    structure specialist for relations,
    media,
    links,
    and formatting brief,
    and expression specialist for voice,
    reference,
    tense,
    grammar,
    and register instructions grounded in archive.
3.  Reduce usable briefs,
    preserve disagreements,
    and fit one bounded editorial packet.
    Missing or unusable brief contributes nothing and cannot block authorship.
4.  In parallel,
    ask primary and statically named fallback document editors
    to produce final whole document once from exact same immutable packet.
5.  Validate both responses independently.
    Adopt primary when usable;
    otherwise adopt fallback.
    Fallback completion time never overrides fixed priority.
6.  Verify deterministic integrity and publish adopted whole document.

Provider payload ceiling is 5.
No candidate selection,
postdraft audit,
reconciliation,
or model Gate exists.

### Stage postconditions

Each specialist brief covers declared responsibility against source and archive before prose exists.
Each editor receives exact same complete packet and owns every wording choice.
Output carries source-unit realization map and brief disposition map.
Deterministic assembly checks maps,
identities,
syntax,
structure,
links,
media,
and publication bytes.

### Pros

- Attacks root cause most directly:
  quality information arrives before text is created.
- Payload ceiling is 5 including one fixed editor fallback.
- Two provider waves expose all independent work:
  three briefs,
  then two fixed-priority editors.
- No postdraft panel can manufacture more work.
- One adopted editor controls complete document voice.
- Simplest call graph,
audit,
and prompt-uniqueness proof.

### Cons

- Specialists cannot discover output-specific grammar,
reference,
or meaning defects before output exists.
- Editor can misunderstand or ignore briefs with no model correction path.
- Realization map can be confidently wrong.
- Mechanical integrity cannot prove finished prose quality.
- Risk is shifted to offline validation rather than reduced inside run.

### Rejected Candidate C evidence

Candidate C ran on fresh Carena inputs after scripted full-graph,
restart,
abort,
vision,
and deterministic-admission controls passed.
Its fixed manifest named three concurrent preparation briefs followed by two concurrent complete-document editors.
Both providers were wet at preflight.
The live invocation exhausted that graph after 370,068 milliseconds and threw bounded
`ProductionUnavailableError`.
It wrote no selected candidate,
final digest,
or publication receipt.

One preparation brief completed.
Two preparation responses were unusable:

- source brief exceeded specialist responsibility after 71,635 milliseconds
- expression brief supplied an unlocated anchor after 9,961 milliseconds

Those failures were nonblocking as designed.
Both complete-document editors then received same immutable packet and all image inputs.
Neither response passed deterministic first-candidate admission:

- primary editor completed after 221,049 milliseconds but returned invalid front matter
- fallback editor completed after 233,245 milliseconds but returned source-realization identifiers
  differing from manifest

Failure digests bind these categories without exposing response wording:

- `d28cd3694f5d448d868ec30fae6b14dd05970ce2e09d416495b9a2be25a0008c`:
  `brief item exceeds specialist responsibility`
- `0388f479397f9004f5163be1ecfd156dc7f1bab93bb9d2a90659bb0aa8f31915`:
  `brief item anchor is not located`
- `bc25fe043161a9e242a717ff29287ab4897b128b603c848bf0d42559b62c7cd4`:
  `candidate front matter invalid`
- `2a6cf4aca78445f8872b3c21838f96a1f26b47f5398aa8c4b24258abbc01ee5a`:
  `source realization ids differ from manifest`

Prototype implementation and guard commits are
`e7f69e076`,
`1c92c961a`,
and
`6d1a50dee`.
Result and node records remain private at
`~/temp/agent/prototype-Carena-C-brief-editor-20260830/`.
A private checksum-preserved copy is at
`~/Downloads/Carena0442-candidate-C-rejected-20260830/`.
The copy contains 19 files and occupies 110,332 bytes as measured after preservation.
Raw responses are operational and contract evidence only.
Do not score,
rank,
average,
quote,
repair,
or use them as translation-quality samples.
Candidate C is rejected rather than retried or salvaged with deterministic text mutation.
A materially changed brief-before-prose design would be a new candidate.

## Measured concurrency implications

Production uses 5 Synthetic slots per active model
and exposes 20 aggregate Synthetic slots across the current roster.
Hyper has no provider concurrency ceiling,
with width 64 corroborated on live request shape.
The Hyper account's 1,000 requests-per-hour limit is separate from simultaneous in-flight work.
See `doc/troubleshooting/translation-repair-provider-concurrency.md`.

Candidate C executed its five payloads in two dependency waves before rejection.
Candidate B executed its five payloads in three waves:
specification,
renderer,
then three independent specialist transactions.
Its concurrent specialist wave ended when slowest GLM response completed after 409,737 milliseconds.
Static fixed priority remained deterministic,
but no candidate was usable.
No cross-provider response was required,
and one model never received same canonical substantive prompt twice.

These are scheduling consequences,
not quality evidence.
Corpus-sized generation and complete-page reading remain prototype acceptance requirements.

## Finite work and guaranteed output

Finite calls to external models cannot prove every human reader will find no prose defect.
Claiming type shape proves quality would repeat current mistake of treating process completion as output evidence.
Actual complete output remains production-readiness evidence,
and any recurring defect means producing architecture is not ready.

Requirements coexist through producer adoption:

- invocation executes one finite manifest and never replenishes it
- each successful model node returns one complete candidate under named responsibility
- unusable later response has no effect and cannot withhold prior complete candidate
- before first candidate,
  only statically named fallback producers may run
- deterministic integrity decides adoption,
  never an aggregate quality opinion
- normal return always includes published complete output
- checkpoint and restart recover crashes or exact cancellation without becoming visible suspended translation
- completed canonical prompt is never resent

Transport loss or unusable output from every producer before first adoption
is not solvable by translation architecture.
Fixed producer roster exhausts into bounded `ProductionUnavailableError`
rather than suspension or ordinary translation result.
The pipeline must not disguise it as quality refusal,
automatically retry it,
or publish archive fallback as repaired output.

## Selection after A, B, and C rejection

No tested candidate is eligible for production selection.
All three exhausted their finite graph without a published page,
so no complete output exists to compare for fidelity,
grammar,
identity,
structure,
or media handling.
Operational evidence cannot be promoted into output-quality evidence.

Historical learning order is:
rejected C > rejected B > rejected A.

Rejected C ranks over rejected B because its two-wave pre-authorship graph had fewer stateful seams,
and one preparation brief was usable;
neither design adopted a candidate.
Rejected B ranks over rejected A because it proved raw-specification fallback,
concurrent bounded specialist execution,
and deterministic transaction controls before exhausting.
Rejected A ranks last because two implementations wrote no output,
and A1 allowed non-producing audit work to withhold complete draft.
This order records design learning only.
No rejected candidate can be retried or selected under same design.

## Candidate D proposal: Immutable-shell slot compiler

Candidate D makes models author translatable content,
not Markdown structure or semantic audit identifiers.
Deterministic parsing derives translatable slots and immutable shell from source plus archive authority.
Slots include headings,
paragraphs,
footnote prose,
and other body-language text.
Existing English front matter remains byte-authoritative,
including contributor identity fields and translated scalar values.
Shell also owns front-matter delimiters and keys,
links,
media references,
footnote relations,
comments,
and node order.

A producer receives full source,
archive,
all page-referenced images,
slot contract,
and complete shell context.
It returns one fixed-key slot record with manifest-derived structural keys such as `s0`.
Exact schema properties bind each translation to one slot without asking model to invent or repeat semantic identifiers.
Compiler reconstructs complete document and applies existing deterministic integrity validation before adoption.
Each producer owns full concrete quality contract across every slot and whole-page relation.

### Fixed call graph

1.  Deterministically derive immutable shell and fixed-key slot record schema.
2.  In parallel,
    ask primary and two statically named fallback authors for complete slot records from same evidence.
3.  Compile and validate all three records independently.
    Adopt first usable candidate by fixed manifest priority,
    never completion order.
4.  Ask one statically named holistic reviser for complete slot record against selected base,
    full source,
    archive,
    and all page-referenced images.
    Adopt usable complete response;
    otherwise preserve selected base byte-for-byte.
5.  Ask one statically named copy editor for complete slot record against latest usable candidate,
    full source,
    archive,
    and all page-referenced images.
    Adopt usable complete response;
    otherwise preserve latest usable candidate byte-for-byte.
6.  Atomically publish and read back final complete document.

Payload ceiling is 5 in three provider waves.
No preparation model,
Gate,
review-only node,
retry,
or response-created work exists.
All five provider nodes return complete candidate records or have no effect.
Each first-wave node can establish first complete candidate.
Role-differentiated canonical prompts preserve uniqueness when one provider and one model fill multiple seats.
Candidate D inherits manifest-bound restart without redispatch,
exact abort identity,
zero transport retries,
and durable transaction states from prototype runtime.
Schema-mismatch handling must additionally persist response digest,
base prompt digest,
and reply-cache key before response becomes spent-unusable.

Model-authored front-matter delimiters,
links,
media,
and footnote markers cannot cause rejection because those bytes are outside model response.
A model can still mistranslate slot content;
only complete-page reading can provide quality evidence.

### Design alternative

Alternative D2 asks each author for unrestricted complete Markdown and validates it after return.

#### D1 pros

- immutable shell removes structural failure classes observed in A,
  B,
  and C from model authority
- fixed-key structural properties remove model-authored semantic identifier values that failed B and C
- one whole-document authorship response still controls voice across every translatable slot
- three independent complete-candidate producers provide bounded fallback before two finite complete-record editors

#### D1 cons

- parser must classify every translatable and immutable region correctly
- sentence relations crossing slot boundaries remain model judgment
- fixed slot boundaries can constrain rhetorical reorganization
- fixed-key schema still needs corpus-sized provider validation

#### D1 measured output evidence and D1.1 correction

Fresh live D1 completed all three concurrent author payloads and published fixed-priority Kimi candidate.
Invocation duration was 347,578 milliseconds.
Published page occupied 20,014 bytes across 55 lines.
All three node records completed,
all reply-cache filenames exactly matched recorded cache keys,
and publication digest matched primary decision digest.
This was first replacement candidate to produce page.

Complete source,
archive,
selected page,
and both alternate candidates were read from beginning through footnote.
Selected Kimi page was not publication-ready:

- shell removed author-supplied leading spaces after inline footnote and link syntax
- one source role was genericized
- concrete overdose,
  repeated-school-year,
  posture,
  and blackmail wording was defective
- one protest relation gained unsupported alongside implication
- source recognition and support content was omitted
- one intentional happiness repetition was collapsed

Complete Qwen alternate preserved role,
overdose,
repeated-school-year,
posture,
recognition and support,
and rhetorical repetition more faithfully.
It also carried literal or awkward phrasing in future relation,
death counterfactual,
queer identity,
and closing imagery.
GLM alternate repeated more selected-candidate defects.
Fixed priority therefore changes from Kimi to Qwen on output evidence,
not completion time or model preference.

D1 artifacts remain private at
`~/temp/agent/prototype-Carena-D-immutable-shell-20260830/`.
Preserved copy is
`~/Downloads/Carena0442-candidate-D1-output-review-20260830/`.
Raw replies are quality evidence only inside this private comparison and are never publication fallback.

D1.1 changes compiler contract rather than retrying same prompts:

- preserve one author-supplied leading space when left shell byte is non-whitespace
- expose adjacent shell context and state that words cannot cross immutable syntax boundary
- use Qwen as fixed-priority base author from complete-page evidence
- add one finite Kimi holistic reviser returning complete slot record
- preserve Qwen base byte-for-byte when reviser is unusable

D1.1 has new manifest digest,
role prompts,
dependency wave,
and payload ceiling.
Scripted controls cover Qwen primary,
Kimi fallback,
GLM reserve,
reviser adoption,
reviser failure preservation,
all-author exhaustion,
and four-node restart without redispatch.
Candidate D1.1 remains unselected until fresh live output is read completely.

Fresh live D1.1 completed four payloads in two waves and published revised document after 732,142 milliseconds.
All node and publication digests bound correctly.
Published file occupied 21,241 bytes across 55 lines.
Complete page,
source,
archive,
all three author candidates,
and revised candidate were read.
The boundary spacing defect and role genericization were corrected,
and source repetition plus footnote details survived.
Page was still not publication-ready:

- opening compensation idiom lacked object and could be read as fabrication
- future relation,
  emotional coercion,
  temporary responsibility,
  and school-suspension phrasing remained source-language calques
- laboratory staining was rendered as ordinary dyeing
- one protest phrase left actor attribution ambiguous
- recognition and support relation was grammatically malformed
- one positive unrestrained expression became morally negative
- closing dialogue remained awkward English

More importantly,
fixed-priority Qwen response echoed source-language slot content instead of translating it.
Schema and deterministic integrity validation incorrectly marked that response complete.
Kimi reviser produced published English page,
but if reviser had been unusable D1.1 would have preserved Chinese base as `page.en.md`.
D1.1 is therefore rejected under tested admission contract.
Its preserved private artifact is
`~/Downloads/Carena0442-candidate-D1.1-output-review-20260830/`.

D1.2 changes canonical prompts and validator identity rather than redispatching D1.1 graph unchanged.
Every slot value containing Unicode Han ideographs is now spent-unusable before compilation.
Guard covers unified ideographs through Extension I plus compatibility supplements;
CJK punctuation is intentionally outside this source-echo check.
This is pinned zh-to-en prototype guard,
not general destination-language classifier.
Prompt adds complete-page idiom,
technical-verb,
pronoun,
and actor-attribution responsibilities.

Scripted source-echo control now records Qwen as `DestinationScriptError` with response,
cache,
and failure digests,
then deterministically selects Kimi fallback.
Restored controls and 871-suite build passed.
GFP mutations proved whole destination-script check and supplementary range boundary each load-bearing.
Four-node v3 restart did not redispatch.
Candidate D1.2 remains unselected until fresh live output is read completely.

Fresh live D1.2 completed four payloads in two waves and published after 689,075 milliseconds.
Published file occupied 21,887 bytes across 55 lines.
All provider nodes completed,
Qwen returned English rather than source echo,
and publication digest matched reviser decision.
Complete final page and all author candidates were read against source,
archive,
footnote,
contributor,
link,
and image.
D1.2 was still not publication-ready:

- 7 visible return-marker glyphs appeared inside two published paragraphs
- dedication punctuation after immutable footnote boundary was awkward
- emotional-coercion wording remained source-language calque
- roommate description and background pronouns remained unclear
- laboratory staining was still mistranslated as ordinary dyeing
- legal statement was rendered as confession
- parent identity became broader relative identity once
- final regret passage claimed narrator had asked Carena not to worry immediately after stating this was never said

D1.2 Qwen base and Kimi revision both carried visible return markers.
Under corrected admission both are unusable;
Kimi fallback did not carry markers but retained more calques.
D1.2 is rejected under tested presentation-admission contract.
Preserved private artifact is
`~/Downloads/Carena0442-candidate-D1.2-output-review-20260830/`.

D1.3 changes prompts and validator identity again.
Deterministic guard refuses visible return symbols,
control pictures,
replacement character,
and raw C0/C1 controls before compilation.
LF and CR remain accepted because compiler normalizes them to spaces.
One static third-wave Qwen copy editor receives latest usable complete candidate,
full source,
archive,
and page image.
It owns whole contract whether holistic reviser succeeded or not,
with general idiom,
technical and legal terminology,
actor,
pronoun,
chronology,
and register responsibilities.

D1.3 remains finite:
5 payloads,
3 waves,
zero retries,
and no response-created work.
Scripted evidence covers copy adoption,
copy failure preserving reviser,
reviser failure followed by copy adoption,
both editors failing while preserving author,
presentation-artifact author fallback,
all-author exhaustion,
and five-node restart without redispatch.
Wave-three SIGTERM records `CallerAbort` and restart publishes reviser without redispatch.
Wave-three SIGKILL leaves dispatched copy node;
restart records `IndeterminateTransmission` and publishes reviser without redispatch.
Stored-completed response revalidation currently fails run rather than degrading node;
manifest and prompt identity prevent cross-validator replay,
but corruption hardening remains production migration work.
Role-distinct fixture digests make preservation observable.
GFP mutations prove artifact refusal,
copy preservation,
author preservation,
and fixed priority load-bearing.
Restored controls and 871-suite build passed.
Candidate D1.3 remains unselected until fresh live output is read completely.
If D1.3 produces another unacceptable page on new prose defect classes,
serial editor layering stops and Candidate D returns to architecture selection.

Fresh live D1.3 completed all 5 payloads in 3 waves and published after 1,508,884 milliseconds.
Published file occupied 21,322 bytes across 55 lines.
All node,
decision,
and readback digests bound correctly;
page contained no Han source echo or visible return marker.
Complete final page and complete holistic-reviser candidate were read against source,
archive,
footnote,
contributor,
link,
and image.
D1.3 final was not publication-ready:

- emotional description became generic and awkward
- future relation remained literal
- responsibility and emotional-coercion language remained calqued
- school-suspension idiom remained opaque
- roommate background reference remained ambiguous
- laboratory staining was still mistranslated as ordinary dyeing
- police entrapment wording was imprecise
- queer identity wording became essentializing and awkward
- final sections drifted from retrospective past into present tense
- closing carefree register became indifference

Complete Kimi reviser already retained future,
emotional-coercion,
school-suspension,
laboratory-staining,
protest-attribution,
and closing-register defects.
Qwen copy editor failed to correct those classes and introduced further generic wording,
fragments,
imprecise police phrasing,
and tense drift.
Serial quality was therefore non-monotonic.
Deterministic validation can prove output usable but cannot prove later usable candidate better than prior candidate;
byte-preservation handles unusable output only.

D1.3 meets documented stop condition.
Candidate D is rejected under tested fixed-priority plus unconditional-serial-adoption design.
No further prompt iteration or serial editor layer is allowed.
Immutable shell,
source-echo guard,
presentation-artifact guard,
and audit runtime remain eligible components for a different architecture;
D graph itself is not eligible for production selection.
All tested Candidates A,
B,
C,
and D are rejected.
Next candidate must use comparative or conditional adoption grounded in concrete located defect classes,
not unconditional later-output adoption or terminal reviewer failure.
Preserved private D1.3 artifact is
`~/Downloads/Carena0442-candidate-D1.3-output-review-20260830/`.

Page evidence includes one `PhotoScroll` asset,
and every page-referenced image reaches every producer.
Entry profile image is presentation metadata from `info.yml`,
not document content or translation evidence,
and remains outside page prompt manifest.

#### D2 pros

- author has unrestricted freedom to reorganize Markdown and prose
- response contract is simpler than slot record plus shell compiler

#### D2 cons

- repeats invalid front matter,
  structure,
  footnote,
  and identifier failure surfaces already measured
- deterministic rejection again happens only after full-document generation spend
- unrestricted structure makes exact authority preservation harder

Within reusable D components,
immutable-shell D1 > unrestricted-Markdown D2,
because D1 removes measured structural admission failures while retaining whole-document authorship;
D2 preserves flexibility by retaining same failure authority already exhausted in three candidates.
This component ranking does not select Candidate D graph.
No tested architecture is selected after complete D1.3 reading.
