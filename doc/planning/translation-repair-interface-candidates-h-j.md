# Translation repair interfaces: Candidates H to J

Part of the [interface comparison index](translation-repair-interface-comparison.md).

## Current stop condition

Historical design evidence only.
Candidate M failed on 2026-09-01.
No candidate prompt may be retried,
and no successor implementation is authorized by this file.

## Candidate H proposal: closed-world verdict with bounded defect evidence

Candidate H removes author self-realization and separates complete checking from exhaustive defect narration.
A verifier still returns one status for every source obligation and global criterion,
but status arrays are manifest-ordered compact codes rather than repeated objects.
Concrete findings remain exact and split-anchor-capable.

```ts
export type CandidateHAuthorResponse = {
  readonly slots: Readonly<Record<SlotKey, string>>;
};

export type CandidateHFinding = {
  readonly scope: 'obligation' | 'global';
  readonly manifestIndex: number;
  readonly defectClassIndex: number;
  readonly targetAnchors: readonly TargetAnchor[];
};

export type CandidateHVerification = {
  readonly candidateId: CandidateId;
  readonly candidateDigest: string;
  readonly obligationStatuses: readonly ('preserved' | 'defect')[];
  readonly globalStatuses: readonly ('clean' | 'defect')[];
  readonly overflow: boolean;
  readonly findings: readonly CandidateHFinding[];
};
```

Every author returns one complete immutable-shell slot map and owns the full quality contract.
Runtime attaches candidate identity,
model identity,
document digest,
slot digest,
and manifest binding.
No author ledger or author-generated audit claim is required.

Every verifier receives complete source,
archive,
shell,
closed-world obligation ledger,
all admitted anonymous candidates,
and every page-referenced image.
Each candidate response has exactly one status per obligation and global criterion.
A clean candidate is one whose statuses are all preserved or clean,
whose `overflow` is false,
and whose findings are empty.

A defective candidate does not need an exhaustive prose dossier to lose:

For manifested per-candidate finding cap `C` and defect-status count `D`:

- `overflow` must equal `D > C`;
- with `overflow: false`,
  every defect status has one unique concrete finding,
  every finding links to a defect status,
  and clean statuses have no findings;
- with `overflow: true`,
  findings have length `C`,
  each finding links to a distinct defect status,
  and the candidate is unconditionally unclean;
- an absent finding never makes a candidate cleaner,
  because selection uses explicit complete status arrays rather than finding counts.

These findings are bounded certificates that a candidate is unclean,
not an exhaustive defect inventory.
This keeps concrete evidence while bounding narration.
`overflow` is not truncation recovery or response-created work.
It is a manifested terminal classification saying that bounded evidence already proves the whole candidate cannot win.
No resolver,
donor splice,
editor,
retry,
or dynamic node follows it.

### Candidate H fixed graph

1.  Build immutable shell and closed-world obligation ledger deterministically.
2.  Run three whole-document authors concurrently.
3.  Admit every complete candidate independently;
    unusable author output has no effect.
4.  Run three all-candidate verifiers concurrently.
5.  Admit only whole all-candidate ballots covering exactly the candidate set derived from one complete author-wave settlement.
    Bind settlement digest,
    candidate digests,
    exact finding indexes,
    split target anchors,
    and correct overflow accounting.
    One malformed candidate row makes the whole verifier abstain;
    retained row evidence is audit-only and cannot vote.
6.  Set calibration evidence floor only with at least two admitted candidate model identities and two complete clean ballots from distinct verifier families other than candidate's own author model.
    Hidden manifest priority supplies private calibration fallback below the evidence floor.
    Candidate's own verifier ballot cannot certify it clean,
    while explicit self-model defect still counts as dissent.
    Production eligibility additionally requires no admitted dissenting defect or overflow ballot.
7.  Write only to private review root;
    no production publication is authorized by calibration status.

The proposed graph has at most six payloads in two dependency waves.
One wet provider remains sufficient.
Every node receives all page-referenced images.
A verifier failure abstains and cannot suspend or mutate a complete candidate.
Candidate aliases and ballot order are derived independently of author identity and fallback priority;
priority exists only in private manifest metadata.
Wire anchors may omit repeated digests only when runtime recomputes and persists exact candidate-substring SHA-256 values,
and restart reproduces byte-identical enriched evidence.
Omission findings remain source-located by obligation index;
other findings require target anchors.

### Candidate H options considered

Candidate H3,
the all-candidate compact matrix with bounded defect evidence,
is preferred.
It preserves one verifier view across candidates,
keeps the six-payload graph,
and makes overflow an explicit losing verdict.
Its cost is reduced per-defect exhaustiveness after the cap.

Candidate H1 would run one verbose verifier payload per candidate.
It preserves familiar verbose rows,
but expands the graph to twelve payloads and leaves one-candidate worst-case findings near the output ceiling.

Candidate H2 would split each verifier-candidate pair into status and finding payloads.
Each payload is smaller,
but the graph reaches twenty-one payloads and independently generated halves can disagree,
turning a working verifier into an abstention.

Ranking:
H3 > H1 > H2.
H3 ranks above H1 because it keeps closed-world comparison in one bounded ballot without multiplying calls.
H1 ranks above H2 because one atomic verifier response is more auditable than independently generated status and evidence halves.

Candidate H is an implemented private calibration prototype,
not yet a viable successor.
The output-envelope boundary resolution supersedes abstract schema stress as architecture rejection.
The verifier finding cap is frozen at `C = 8`.
The active evidence still contains no qualified author roster:
all five Qwen expansion authors failed complete-page review,
and MiniMax returned an unusable author response.

Before spend,
controls must prove complete arrays,
no absent-as-clean path,
overflow accounting,
three disjoint anchors,
hidden priority,
author and verifier diversity,
provider isolation,
restart,
indeterminate transmission,
and exact concurrent abort cleanup.
Maximum compact schema witnesses and realistic complete Carena author outputs must be tokenized under every planned model tokenizer available;
unavailable tokenizers remain explicit roster blockers rather than guessed equivalence.

### Candidate H envelope disposition

Candidate H is not production-selected.
Its compact schema-stress witness proves that the schema is overpermissive,
not that the provider-returnable author envelope is unbounded.
Its slot-only author uses the immutable-shell response envelope from Candidates D and E:
23 required slot strings,
each permitting 20,000 characters.
A Candidate H-specific compact stress witness was constructed against the schema returned by `slotResponseFormat` as
`{ slots: Record<SlotKey, string> }`.
`slotDocumentGuard` accepted the witness.
It measured 2,760,208 bytes and 2,300,131 tokens under `Qwen/Qwen3.6-27B` tokenizer commit
`6a9e13bd6fc8f0983b9b99948120bc37f49c13e9`.
Its SHA-256 is
`bbd5a9efc8c16d465d784525c8396f2ba1ca467e6a57ff2df110302ef22f46a8`.
The author response schema itself measured 1,664 bytes.
These token counts are Qwen3.6-specific because Candidate H has no frozen qualified roster.
Post-transport presentation rejection cannot enforce the output envelope.

No source-derived prose cap is authorized.
The five retained complete Qwen pages prove only that those rejected outputs fit their measured lengths;
they do not prove that a publication-ready candidate cannot require more.
The compact schema-stress failure no longer rejects Candidate H by itself.
Candidate H code is implemented on the finite-prototype branch,
but it is not production-selected.
Its completed live calibration rejects the current roster and ballot interface as recorded in
"Rejected Candidate H live calibration."

Candidate H contributes one reusable interface idea:
complete status arrays plus exact overflow algebra can bound defect narration without treating missing findings as clean evidence.
That idea may be reused only by a successor whose author boundary has a defensible envelope and whose roster contains a completely read acceptable author.

### Output-envelope boundary resolution

Candidate G and Candidate H prove that the current schemas are overpermissive.
They do not prove that every free-prose author interface exceeds the provider envelope.
Four different response sets must not be conflated:

1.  abstract JSON instances satisfying response schema;
2.  compact canonical serializations of those instances;
3.  provider-returnable output under the manifested completion-token cap;
4.  provider-returnable responses admitted by deterministic publication guards.

Optional JSON whitespace makes the set of textual schema serializations unbounded unless a canonical grammar is part of the contract.
Provider-returnable output is bounded by the provider completion cap by definition.
Deterministic admission is a subset of returned output.
The current measurements establish only that schema-valid compact stress instances can exceed the cap and that some pass structural guards before later rejection.
The provider implementation of JSON Schema string-length semantics and whitespace generation has not yet been measured.

Task #60 operationalized the pre-spend gate as a compact schema-stress maximum.
That interpretation is superseded.
The original normal-run objective qualifies success by a working model,
so pre-spend acceptance covers provider-returnable responses that deterministic guards can admit.
Candidate G and Candidate H are not rejected by their schema-stress witnesses alone.
Their lack of a qualified author and complete-page quality evidence remain independent blockers.

#### I1: transport-limited deterministic-admission boundary

Treat manifested provider completion tokens as the hard transport maximum.
Keep fixed-key immutable-shell slot JSON and require realistic plus adversarial deterministic-admission witnesses,
model-specific completion tokenization with framing reserve,
and full-page author acceptance before spend.
Schema-valid but semantically invalid floods may truncate and become no-effect output.

Pros:
publication prose and contributor authority remain unrestricted inside the actual provider envelope;
existing structure and restart guards survive.

Cons:
not every abstract schema-valid compact stress instance can complete;
model-specific token accounting and provider behavior become part of evidence.

#### I3: measured repository prose budget

Add per-slot or aggregate limits measured from source and accepted outputs.
This mechanism can be combined with I1,
but it cannot define correctness by itself.

Pros:
simple preflight arithmetic and smaller unusable-output region.

Cons:
cap can reject the only publication-quality wording;
retained rejected pages do not prove an acceptable-page ceiling.

#### I2: token-id experiment

Ask authors to print tokenizer ids and decode them into prose.
This is not an exact provider-token envelope as previously stated:
decimal digits,
commas,
JSON framing,
whitespace,
special tokens,
and invalid sequences consume provider completion tokens independently of decoded ids.

Pros:
model-specific decoded length is explicit.

Cons:
wire accounting remains nontrivial;
models are not calibrated for reliable tokenizer-id authorship;
provider and model swaps change encoding.
No implementation is warranted without a zero-spend reliability prototype.

Ranking:
I1 > I3 > I2.
I1 ranks above I3 because it preserves prose authority and matches truncation-as-unusable semantics.
I3 ranks above I2 because a measured aggregate budget is implementable despite editorial risk,
while token-id authorship has neither reliable generation nor exact provider accounting.

Provider research completed on 2026-08-31:

- Hyper documents Anthropic Messages `max_tokens` as required maximum output tokens;
- `buildAnthropicBody` applies the package-wide 32,000-token measured answer bound to every Hyper call,
  lowered only by model or caller cap;
- tool input streams as model-produced partial JSON strings and finalizes as an object;
  neither Hyper nor Anthropic promises compact whitespace;
- streaming usage reports cumulative output tokens;
- `stop_reason: max_tokens` explicitly means truncation.

The normal-run objective is qualified by a working model.
Accordingly,
future architecture work adopts the intersection of provider-returnable responses and deterministic admission,
not every abstract schema-valid stress instance.
Schema stress witnesses remain robustness evidence and can prove overpermissiveness,
but they are not by themselves a normal-run architecture rejection.

This interpretation removes Candidate G and Candidate H envelope-only rejections.
It does not reverse their independent blockers:
Candidate G has no qualified author and its four-candidate verifier interface remains unmeasured under the planned model tokenizers;
Candidate H is implemented and targeted-lint-clean,
but its completed live calibration admitted no verifier ballot and produced no publication-eligible candidate.
Live Hyper and Synthetic use is authorized without another permission checkpoint.

Consumer hardening is required first.
`readJsonOutcome` currently admits parseable guard-valid JSON even when `finishReason` is `max_tokens`.
That contradicts truncation-as-no-effect semantics and is documented in
`doc/troubleshooting/charm-hyper-max-tokens-tool-json.md`.
The shared provider boundary now rejects Anthropic `max_tokens` and OpenAI-compatible `length` before parsing.
Commit `44ed76c59` added dist-based controls,
and GFP showed both parseable truncation cases fail when the predicate is removed.

### Candidate H implementation status

Prototype commits `7ceaec899` and `f2e35fd2c` implement Candidate H with:

- three immutable-shell whole-document authors and three all-candidate verifiers in two dependency waves;
- one Hyper-only vision roster used in both roles:
  Qwen3.8-27B,
  Kimi K3,
  and MiniMax M3;
- canonical roster identities mapped to exact Hyper wire ids
  `qwen3.8-27b`,
  `kimi-k3`,
  and `minimax-m3`;
- self-model clean certification excluded while self-model defect remains dissent;
- complete manifest-ordered obligation and global status arrays;
- eight bounded exact findings per candidate and exact `overflow === D > 8` algebra;
- obligation-indexed omissions,
  non-overlapping UTF-16 target anchors,
  and raw duplicate-member refusal;
- complete author-wave settlement,
  whole-ballot abstention,
  hidden priority,
  family-aware evidence floors,
  and no-dissent production eligibility;
- frozen provider binding,
  one-payload prompt claims,
  process-incarnation lease,
  deterministic restart,
  indeterminate-transmission quarantine,
  and exact caller-abort propagation after sibling settlement;
- Hyper-only provider masking,
  all page-image carriage,
  blocked internal node subpaths,
  and private output only.

Targeted Candidate H controls pass.
Type lint passes.
The fresh restored package run reports 876 passing suites and no failure lines.
GFP removal of overflow algebra and family-floor checks fails at their intended Candidate H controls;
restored targeted and full runs pass.

Candidate H's synthetic structurally admitted field-count-maximum Carena verifier witness measured:

- 23 immutable-shell slots,
  134 obligations,
  three candidates,
  eight findings per candidate,
  and three anchors per finding;
- 13,339 compact bytes and 4,447 tokens under the project three-bytes-per-token estimate;
- 7,473 raw JSON tokens under exact Qwen3.8 tokenizer;
- 5,716 raw JSON tokens under exact Kimi K3 tokenizer;
- 5,816 raw JSON tokens under exact MiniMax M3 tokenizer;
- 10,284 tokens of lowest raw-wire arithmetic reserve under Kimi's 16,000-token model cap;
- compact wire SHA-256
  `85d8eeee934173552d5b631f47a580f7f1f47039102302eff966b51629befbc8`.

The witness is field-count-maximum,
not byte-maximum.
It passed full Candidate H admission for all planned verifier identities.

A realistic complete Carena author response measured 21,412 compact bytes with SHA-256
`bb61c6dcb2cde515e04748cccabb99e15579edf9091e634b156e633c3159ef08`.
Its exact raw JSON counts are 4,585 Qwen3.8 tokens,
4,594 Kimi K3 tokens,
and 4,524 MiniMax M3 tokens.
The lowest author-witness raw-wire arithmetic reserve is 11,406 tokens under Kimi's 16,000-token model cap.
The retained response is used only as realistic size evidence,
not as evidence that its rejected author identity is qualified.

Current roster tokenizer artifacts are pinned at:

- Qwen3.8 `1d4bf0f2ff6012fd82039f2fa52739d0dd7c60c0`;
- Kimi K3 `a590ce090cb049c93a33dfe8c208ec652aa20503`;
- MiniMax M3 `f0e1c1e04d40177e4673a22097036854f536e9c0`.

Raw-wire headroom does not include model reasoning or tool-call framing.
Provider `usage.output_tokens` and finish reason were therefore required and are recorded by live calibration.
`doc/troubleshooting/charm-hyper-token-count-evidence.md` records the endpoint research,
artifact hashes,
and exact method.

Envelope measurement code is committed as `be0c178d6`.
The fresh package run after that change reports 876 passing suites and no failure lines;
type lint passes.

Candidate H targeted lint is clean in prototype-branch commit `ebf5a7f7d`:
18 files checked against 484 Oxlint rules report no warnings or errors,
type lint passes,
and the fresh full run reports 876 passing suites and no failure lines.
The migration also split verifier schema from parsed guard,
moved lifecycle fixture construction to the dist-importing unit test,
and preserved exact `IndeterminateTransmission` operational evidence.

Fresh Hyper `GET /v1/models` evidence confirms current vision capability and provider maxima:
Qwen3.8-27B true and 128,000,
Kimi K3 true and 16,000,
and MiniMax M3 true and 512,000.
Both prior DeepSeek verifier ids remain vision false and are rejected by Candidate H manifest.
The package applies 32,000-token project cap where model maximum is higher.

H-specific GFP rejects removal of global-omission refusal,
Hyper vision filtering,
Hyper-only provider binding,
canonical-to-Hyper reach,
self-certification exclusion,
prompt claims,
fresh raw-member refusal,
spent-node restart,
runtime lease identity,
and all-settled abort behavior.
Task #69 already GFP-proves shared truncating-finish refusal.
Restored targeted lint and types pass;
the restored full run reports 876 passing suites and no failure lines.

### Rejected Candidate H live calibration

Private Carena calibration ran from prototype commit
`5f3ca0946e690dcef7cabeb2e3482c951d915679`
against corpus commit
`a80634a674f94861ea3b7056fba054ca9eab1a2c`.
Manifest SHA-256 was
`c289fbb230e28cd29ab94deee4dbd13778f76556fc3d9a5d7349169c91825353`.
The fresh Hyper catalog still reported all three manifested models as vision-capable with expected maxima.
An injected transient control made exactly one transport attempt under `retryPolicy.limit = 0`.
Forced-tool controls bound all three canonical model ids to expected Hyper wire ids,
tool name,
and tool choice.

The run sent exactly six exchanges,
one for each durable node row,
and completed in 1,025,027 milliseconds.
Every exchange record carries request and response digests,
bytes,
role,
wire model,
HTTP status,
stop reason,
and provider usage without retained raw payload text.
All six responses returned HTTP 200:

- Qwen author reported `end_turn` and 59,438 output tokens;
  complete response was admitted.
- Kimi author reported `max_tokens` and exactly 16,000 output tokens;
  truncation guard made it spent-unusable.
- MiniMax author reported `tool_use` and 18,680 output tokens;
  complete response was admitted.
- Qwen verifier reported `end_turn` and 47,553 output tokens;
  extracted tool answer was not parseable JSON and whole ballot abstained.
- Kimi verifier reported `max_tokens` and exactly 16,000 output tokens;
  truncation guard made it spent-unusable.
- MiniMax verifier reported `tool_use` and 9,031 output tokens;
  parsed response failed exact caller guard and whole ballot abstained.

The two admitted authors demonstrate bounded candidate retention in this run despite one author failure.
The three verifier abstentions left no clean or dissenting ballot.
Private fixed-priority fallback selected Qwen with `evidenceFloorMet: false` and
`productionEligible: false`;
it did not authorize publication.

Complete-page review read source,
archive,
both admitted candidate pages,
and page-referenced `photo1.webp` at 2,048 by 2,048 pixels.
The selected document preserved front matter,
node order,
link target,
media reference,
contributor attribution,
and all source passages.
It was not publication-ready.
The footnote boundary omitted required English whitespace between link and following prose.
Both admitted candidates carry same boundary defect,
so fixed priority cannot avoid it.
Complete reading also found unresolved English idiom in flight-ticket comparison.

Candidate H is rejected rather than retried or repaired from spent outputs.
Its complete status-array idea remains useful,
but current all-candidate response interface did not yield one admissible ballot from three vision models.
Raw-wire arithmetic reserve also failed to predict Kimi's reasoning-inclusive truncation.
Private artifacts remain at
`~/temp/agent/prototype-Carena-H-bounded-verdict-20260831/`.
Complete source,
archive,
image,
and both-candidate review record is
`complete-page-review.json` with SHA-256
`efe09e54068c7aa9b45557818119b4fb88b53cf6b363446857ace4f3b322c646`.

The user has authorized Hyper and Synthetic calls without another permission checkpoint.
Advisor review preceded live calibration as engineering evidence ordering,
not as an authorization gate.

## Candidate I plan: candidate-scoped compact ballots

Candidate I keeps immutable-shell whole-document authorship but replaces all-candidate verifier matrices.
Its fixed graph manifests:

1.  one Qwen3.8-27B author and one MiniMax M3 author in concurrent first wave;
2.  one Qwen,
    one GLM 5.3 Flash,
    and one MiniMax verifier for each author ordinal in concurrent second wave;
3.  deterministic selection and private publication after every dispatched sibling settles.

Maximum payload count is eight in two provider waves.
An unusable author writes durable terminal state and deterministically skips its three candidate-scoped verifier nodes.
It does not cause replacement author,
retry,
or generated verifier work.
All nodes remain Hyper-only and receive every page-referenced image.
One model and one canonical substantive prompt still produce at most one provider payload.
Canonical verifier prompt includes manifested candidate ordinal,
opaque candidate id,
and candidate digest;
ordinal keeps prompts distinct even if two candidate documents and digests are byte-identical.

Every verifier receives one complete anonymous candidate,
complete source,
archive,
immutable shell,
closed-world obligation ledger,
and all images.
Its response keeps complete manifest coverage in compact strings:

```ts
export type CandidateIVerification = {
  readonly candidateId: CandidateId;
  readonly candidateDigest: string;
  readonly obligationStatuses: string;
  readonly globalStatuses: string;
  readonly overflow: boolean;
  readonly findings: readonly CandidateHFinding[];
};

export type CandidateIVerifierProfile = {
  readonly modelId: ModelIdentity;
  readonly responseRoute: 'anthropic-tool';
};
```

`obligationStatuses` has exactly one `p` or `d` code per manifested obligation.
`globalStatuses` has exactly one `c` or `d` code per global criterion.
Runtime expands strings into durable manifest-indexed audit rows before selection.
Candidate and digest binding,
finding anchors,
duplicate-member refusal,
and Candidate H overflow algebra remain exact.
A malformed status length or character abstains atomically and records privacy-safe guard category,
not raw provider text.

For Qwen-authored candidate,
clean GLM and MiniMax ballots satisfy two-family nonself evidence floor.
For MiniMax-authored candidate,
clean Qwen and GLM ballots satisfy same floor.
Clean self-model ballot never contributes to floor;
a valid self-model defect or overflow still vetoes candidate.
No admitted dissent is allowed.
Private fixed-priority fallback may preserve complete candidate below floor,
but cannot claim production eligibility.

Candidate I does not set non-default thinking,
reasoning-budget,
temperature,
or effort parameters.
It still sends protocol-required or live-catalog output ceiling.
Package contracts record owner's standing instruction against reasoning and sampling knobs after measured serving-stack
failures.
Focused Kimi vet rejected every available route under those constraints.
K2.6 and K2.7 Code returned non-JSON content despite strict Hyper OpenAI schema request;
K3 returned parseable JSON that failed exact caller guard.
Repository transport source already records that Hyper OpenAI route accepts and ignores `response_format`.
Focused third-family vet selected Hyper `glm-5.3-flash` over Anthropic forced-tool route for prototype validation.
Manifest persists canonical id,
wire id,
output cap,
and exact route-table digest for every verifier.
Unsupported,
truncated,
or malformed output abstains without route or parameter fallback.

A bounded Hyper probe sent one image and one forced tool with low effort to Kimi K3.
It returned HTTP 200,
`tool_use`,
96 output tokens,
214 reasoning characters,
and expected parseable verdict after 2,955 milliseconds.
This proves route combination acceptance only.
It does not prove lower reasoning and cannot override owner no-effort policy.
Candidate I will not use low-effort path.

Candidate I intends to move target-language separators out of model authority.
Immutable shell manifest gains AST-derived boundary atoms for target text adjacent to locked inline syntax.
For each such relation,
manifest states exact separator and neighboring syntax roles.
Prototype compiler must insert separator before candidate hashing;
verifier input and persisted candidate must use exact post-insertion document.
It does not trim or normalize arbitrary prose.
Positive and negative fixtures cover link-to-prose,
footnote,
punctuation,
code,
URL,
media,
and block boundaries.
This addresses Candidate H's shared footnote boundary defect without editing model prose.

Candidate I replaces coarse caller-guard detail with finite privacy-safe union:

```ts
export type CandidateIGuardFailure =
  | 'key-set'
  | 'candidate-binding'
  | 'status-length'
  | 'status-alphabet'
  | 'finding-shape'
  | 'anchor'
  | 'overflow'
  | 'raw-duplicate'
  | 'json-syntax';
```

Every caller-guard rejection records one category without retaining reviewer wording.
Transport,
abort,
truncation,
and indeterminate-transmission failures remain separate operational classes.
Finish reason,
usage,
response route,
request and response digests,
and attempt count become durable node evidence.

### Candidate I posture after Kimi route rejection

Candidate I uses two authors and six candidate-scoped verifiers.

Pros:

- removes demonstrated Kimi author truncation from producer wave;
- preserves three verifier-family positions needed for two-family nonself evidence;
- reduces each ballot from all candidates to one candidate;
- isolates one malformed candidate response to one ballot;
- keeps eight-payload,
  two-wave finite graph.

Cons:

- only two author wordings enter selection;
- GLM remains load-bearing and lacks complete Carena translation-review evidence;
- Hyper OpenAI structured output is unavailable as candidate route;
- neither retained Candidate H author passed complete-page publication review.

Candidate J's Kimi third-author expansion is rejected.
It adds demonstrated truncating producer and twelve-payload graph while no Kimi route satisfies verifier hard gate.

Candidate I remained a private prototype,
not production selection,
until the calibration recorded in the next section.
GLM route was manifest-bound and transport-validated before consumer validation.
It inherits Candidate H's zero-retry and pre-dispatch exchange cap,
Hyper-only provider binding,
indeterminate-transmission quarantine,
spent-node restart,
process lease,
exact caller-abort identity after sibling settlement,
prompt claims,
and complete-candidate no-effect publication guards.
Prototype HEAD `28d548d84fe1fcb51765b0e9a845c6361bf6359c` implements the fixed graph.
It passed targeted type-aware lint with zero diagnostics,
type checking,
and full `buildAndTest` with 877 suite `PASS` lines and no suite `FAIL` line.
GFP mutations failed after rebuild for family exclusion,
pre-verifier abort,
atomic guard category,
spent restart,
compiled anchor slots,
AST roles,
punctuation spacing,
all three wire mappings and caps,
route digest,
status length and alphabet,
finding algebra,
status-row expansion,
and unusable-author verifier skips.
Every mutation was restored before final green run.

### Candidate I pinned-Carena disposition

Candidate I is rejected after its one zero-retry pinned-Carena calibration.
Harness SHA-256
`90455077ef109c50750728dc9cb975acde33459d91a4d55ca6ace5dc881e37ff`
ran prototype commit `28d548d84fe1fcb51765b0e9a845c6361bf6359c` against corpus commit
`a80634a674f94861ea3b7056fba054ca9eab1a2c`.
Its fixed manifest dispatched all eight nodes once,
then proved restart with zero additional transport calls.
Every request carried `photo1.webp` and used default reasoning and sampling.

Both authors completed and admitted complete candidates.
Verifier settlement was:

- Qwen candidate:
  Qwen failed JSON syntax,
  GLM was cut at the 360-second call deadline,
  and MiniMax returned a clean ballot;
- MiniMax candidate:
  Qwen returned a clean ballot,
  GLM was cut at the 360-second call deadline,
  and MiniMax reached `max_tokens` before a tool ballot.

Four nodes completed and four became spent-unusable.
No node was skipped.
Each candidate had only one clean nonself family.
Neither candidate met the two-family nonself floor.
The deterministic private fallback selected the Qwen candidate with
`evidenceFloorMet: false` and `productionEligible: false`.

Complete-page review read the source,
archive,
both candidate documents,
private selected document,
and page image before interpreting selection metadata.
Both candidates preserved front matter,
block order,
media syntax,
contributor identity and link,
and source footnote link.
The Qwen candidate still contained an opening punctuation defect,
an incomplete possessive construction,
and one source-and-image relation mistranslation.
The MiniMax candidate contained sentence fragments,
several unidiomatic constructions,
and one omitted contrast.
Neither document was publication ready.
No output was published.

Metadata summary SHA-256 is
`48c4607d29a90aebacc27f5130e7e45d8d83f4529958419bfd937b35afe3e115`.
Private complete-page review SHA-256 is
`f5368388a184fbe394eccedb923933bd69e0deeef1c40e702e743b821c62373b`.
Candidate I must not be retried,
repaired,
or integrated.

The next finite design must reduce the opaque review burden without dropping any of the 134 clause-and-relation
obligations or ten global criteria.
Deterministic admission already owns syntax,
structure,
links,
media,
identity,
and boundary separators.
A successor should ask model verifiers only for source-to-target semantic and language judgments over compiled
translation slots plus page-level criteria.
It must retain complete source,
archive,
candidate,
and image access,
family-aware nonself evidence,
exact status coverage,
and the same zero-retry lifecycle.
This is a new substantive verifier contract and cannot redispatch any Candidate I prompt.
