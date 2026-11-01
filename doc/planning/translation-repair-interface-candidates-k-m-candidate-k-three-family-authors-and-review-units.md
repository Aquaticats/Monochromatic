# Translation repair interfaces K to M: Candidate K, three-family authors with conjunctive review units

Candidate K's plan,
its static graph,
review-unit compiler,
ballot algebra,
alternatives and acceptance controls,
its structural prototype status,
and the calibration that rejected it as an author protocol.

Part of [Translation repair interfaces: Candidates K to M](translation-repair-interface-candidates-k-m.md),
which lists every file of this record.

## Candidate K plan: three-family authors with conjunctive review units

Candidate K addresses both Candidate I failure surfaces.
It adds GLM as a third independent whole-document producer,
so a strong GLM candidate can qualify from Qwen and MiniMax evidence without depending on a GLM self-verdict.
It also replaces the opaque digest-only verifier ledger with readable compiled review groups.
It retains one explicit status for every clause and relation obligation.
The unit compiler retains every original obligation,
its semantic evidence,
and its exact mapping.

### Static graph

The manifest fixes twelve node templates before any provider contact:

1.  Qwen3.8-27B,
    GLM 5.3 Flash,
    and MiniMax M3 each receive one substantively new complete-candidate author prompt in the concurrent author wave;
2.  Qwen,
    GLM,
    and MiniMax each verify every usable candidate with one combined fidelity-and-English ballot in the concurrent
    verifier wave;
3.  deterministic selection runs only after every dispatched sibling settles.

Maximum payload count is twelve in two waves.
If an author is unusable,
its three statically named verifier nodes become durable deterministic skips.
Actual payload count is therefore three plus three times the number of usable authors.
Dependency-independent nodes remain concurrent.
No response creates a node,
retry,
route fallback,
or correction request.

Every node remains Hyper-only and receives every page-referenced image.
One model and one canonical substantive prompt produce at most one payload.
A local request deadline remains finite and manifest-bound;
the initial prototype value is 900,000 milliseconds.
That value is provisional validation input,
not evidence that GLM will complete.
Deadline-aborted outputs are spent-unusable and have no selection effect.
No model request sets thinking,
reasoning budget,
temperature,
effort,
or another sampling knob.
The initial route output ceiling remains the vetted 32,000 tokens for each model.
A higher ceiling requires a separate new-protocol route vet before implementation may rely on it.

### Substantively new producer contract

Candidate I's Qwen and MiniMax author prompts are spent and cannot be sent again.
Candidate K author protocol changes both instruction and packet semantics:

- it receives the compiled review-unit plan instead of the raw per-clause status ledger;
- it treats source text as semantic authority and archive text only as permitted wording evidence;
- it explicitly owns complete meaning,
  every actor and relation,
  idiomatic sentence boundaries,
  unambiguous references,
  terminology,
  chronology,
  memorial register,
  and image-related meaning;
- it returns every immutable-shell slot exactly once and returns no audit or repair claims;
- it must produce one complete publication candidate or have no effect.

GLM has not received a Candidate I author prompt.
Qwen and MiniMax may enter Candidate K only under this genuinely changed protocol and packet,
not a digest-only rename.
All three producers are peers;
none reads or edits another producer's output.
This preserves the ban on serial editor layering.

### Review-unit compiler

The compiler template is fixed before authors run.
Candidate binding after author settlement instantiates already-manifested templates with candidate id,
candidate digest,
and deterministic-proof digest;
it does not create work.

Candidate K bounds one manifest at 192 slot groups,
192 clause subjects,
191 relation subjects,
six global subjects,
and 64 retained findings.
For pinned Carena,
the compiler must produce:

- 23 slot groups containing all 112 clause subjects and one status per clause;
- 22 relation subjects preserving every relation obligation as its own status;
- 23 slot-language subjects;
- six page-level global subjects.

Each slot group stores its slot key,
source-slot range,
source text,
digest,
authority,
and ordered clause subjects.
Each clause subject carries obligation id,
canonical source ranges and text,
authority,
allowed target slot keys,
and evidence digest.
The ballot returns one clause status for every member,
so no clause disappears behind a slot conjunction.
Grouping makes evidence readable without reducing status granularity.

Each relation subject carries obligation id,
`adjacent-source-slot` kind,
ordered left and right clause endpoints,
canonical endpoint source ranges and text,
authority,
allowed target slot keys,
and evidence digest.
Relation findings may use multiple target anchors across authorized endpoint slots.
Slot-language subjects bind exactly one candidate slot and its complete candidate text.
All readable plan structure is included in `reviewPlanDigest`;
opaque digests never substitute for evidence the verifier must inspect.
The six global subjects are:

- cross-slot actor identity and coreference;
- cross-slot chronology and semantic relations;
- technical and legal terminology consistency;
- document-wide grammar,
  tense,
  register,
  and rhetorical coherence;
- contributor voice and authority;
- source,
  image,
  and target relation.

Ownership intentionally overlaps.
Terminology,
coreference,
register,
and image-alt prose may be rejected by either fidelity or language evidence.
A verifier cannot defer a defect because another responsibility also owns it.

The compiler persists an exact coverage map from Candidate I criteria into Candidate K subjects:

- unsupported addition,
  identity attribution,
  actor reference,
  chronology,
  and technical or legal meaning map to slot fidelity,
  relation,
  and overlapping global subjects;
- grammar and usage,
  tense,
  register,
  and source-language calque map to slot-language and overlapping global subjects;
- paragraph relations map to relation units and the chronology-and-relations global subject.

Tests must prove that every 112 clause obligation belongs to exactly one slot group and has exactly one status,
every 22 relation obligation remains exactly once,
every translatable slot has a language subject,
and every prior global criterion has at least one explicit successor owner.
They must also prove every readable source excerpt matches its bound range and digest,
every relation endpoint and direction match the original ordered obligation,
and no compiler output exceeds the static bounds.

### Deterministic proof boundary

Deterministic admission owns only mechanically decidable properties:

- raw duplicate-member refusal and exact response key sets;
- exact slot-key set,
  nonempty values,
  and compiled-document envelope;
- immutable syntax and locked-range survival;
- runtime-owned separators and target boundaries;
- front-matter key shape;
- link destination,
  media path,
  contributor identity and URL,
  and footnote destination survival;
- candidate,
  manifest,
  review-plan,
  document,
  slot,
  image,
  and proof digests.

It does not claim that translated front matter,
link labels,
image-alt prose,
visual relations,
contributor voice,
or any other target wording is semantically correct.
Those remain model-reviewed.
The candidate-bound proof is recomputed before verifier dispatch and before selection.

### Combined ballot and finding algebra

Each verifier returns:

```ts
export type CandidateKBallot = {
  readonly candidateId: CandidateId;
  readonly candidateDigest: string;
  readonly reviewPlanDigest: string;
  readonly deterministicProofDigest: string;
  readonly clauseStatusesBySlot: readonly string[];
  readonly relationStatuses: string;
  readonly slotLanguageStatuses: string;
  readonly globalStatuses: string;
  readonly overflow: boolean;
  readonly findings: readonly CandidateKFinding[];
};

export type CandidateKFinding = {
  readonly scope: 'c' | 'r' | 'sl' | 'g';
  readonly subjectIndex: number;
  readonly defectClassIndex: number;
  readonly sourceEvidenceIndexes: readonly number[];
  readonly imageEvidenceIndexes: readonly number[];
  readonly targetAnchors: readonly TargetAnchor[];
};
```

`clauseStatusesBySlot` contains one string per slot group,
and every string length equals that group's clause count.
Clause and relation alphabets are `p` or `d`;
language and global alphabets are `c` or `d`.
Raw duplicate members,
wrong characters,
wrong outer or inner lengths,
stale bindings,
invalid evidence indexes,
and unbound anchors abstain atomically.

A status is one Boolean subject state,
not a defect count.
Canonical subject order is clause groups and members in manifest order,
then relations,
slot language,
and globals.
Let `D` be the number of defective subjects and let fixed `C` equal 64.
`overflow` must equal `D > C`.
Without overflow,
exactly one witness finding binds every defective subject.
With overflow,
findings must bind exactly the first `C` defective subjects in canonical order.
Multiple defects inside one subject do not change `D` and do not create an unverifiable finding-count claim.

Every finding field is present;
scope controls whether its arrays must be empty or populated:

- clause findings cite the subject's source evidence;
  omission findings have no target anchor,
  while every other clause defect anchors inside an allowed target slot;
- relation findings cite both ordered endpoint source records and one to four disjoint target anchors across authorized
  endpoint slots;
- slot-language findings have empty source and image evidence arrays and one to three target anchors inside their slot;
- nonvisual global findings require target anchors and no image evidence;
- source-image-target relation findings require at least one exact manifest image index and target anchor,
  plus any relevant source evidence.

Defect-class scopes are closed in the manifest.
An omission class is legal only for clause scope;
an image-relation class is legal only where image evidence is permitted.
All source evidence indexes resolve to readable plan records with exact range and digest.
All image indexes resolve to manifest-bound page images.

Seeded controls must include multiple defects in one slot,
a cross-slot relation defect,
terminology drift,
actor-coreference ambiguity,
register failure,
image-relation failure,
and a mixed fidelity-and-language defect that both roles may veto.

### Evidence and selection

For a Qwen candidate,
clean MiniMax and GLM ballots provide two nonself families.
For a MiniMax candidate,
clean Qwen and GLM ballots provide two nonself families.
For a GLM candidate,
clean Qwen and MiniMax ballots provide two nonself families.

One candidate is production eligible only when:

- deterministic proof passes at every boundary;
- both nonself family ballots are valid and clean across every status string;
- no admitted ballot from any family contains a defect or overflow;
- contributor,
  structure,
  media,
  prompt-claim,
  provider,
  restart,
  and cancellation controls pass.

Self-clean evidence never contributes to the floor.
A valid self defect or overflow vetoes.
Malformed,
partial,
truncated,
late,
or stale ballots abstain with no selection effect.
Selection among eligible candidates uses fixed hidden priority only after all siblings settle.
A complete private fallback remains explicitly below floor and cannot authorize publication.

### Alternatives

#### Option A: three authors with combined ballots (recommended)

Pros:

- adds a genuinely new GLM producer and gives that candidate Qwen plus MiniMax nonself evidence;
- performs one complete-page review per family and candidate;
- retains separate fidelity,
  relation,
  language,
  and global acceptance predicates inside one atomic ballot;
- fixes Candidate I finding algebra without duplicating full-page inputs.

Cons:

- one malformed combined ballot loses both fidelity and language evidence from that family;
- three concurrent GLM verifier calls remain unmeasured under the smaller contract and longer deadline;
- Qwen and MiniMax producer quality may remain correlated with prior family defects;
- no finite selection rule can publish when all three independent whole candidates are defective,
  so producer quality remains a calibration hard gate.

#### Option B: three authors with separate fidelity and language ballots

Pros:

- isolates role instructions and schema failures;
- one failed role does not erase valid evidence from the other role.

Cons:

- has 21 statically named nodes and sends six concurrent full-page verifier calls per model when all authors succeed;
- has the same family-aware acceptance predicate as Option A;
- repeats complete source,
  archive,
  candidate,
  shell,
  proof,
  and images for each responsibility;
- specialization benefit has no measured consumer evidence.

#### Option C: one GLM author with Qwen and MiniMax ballots

Pros:

- has three payloads when every node completes;
- avoids using GLM as its own qualifying verifier.

Cons:

- one unusable author leaves no complete candidate;
- offers no alternate wording when the complete candidate has a defect;
- cannot satisfy the normal-run resilience objective.

Ranking:
Option A > Option B > Option C.
Option A ranks over Option B because both enforce the same conjunction,
while Option A avoids duplicated full-page load and same-model concurrency.
Option B ranks over Option C because its producer redundancy and isolated evidence survive one producer failure;
Option C has a single producer point of failure.
A first-defect-only ballot is excluded before ranking because it cannot demonstrate complete review coverage.

### Prototype acceptance controls

Candidate K cannot enter live calibration until:

- the manifest fixes three authors,
  nine candidate verifier templates,
  twelve-payload ceiling,
  route ids,
  vetted 32,000-token caps,
  provisional 900,000-millisecond local deadlines,
  and two dependency waves;
- review-plan compilation proves exact obligation and prior-criterion coverage,
  readable evidence binding,
  static unit bounds,
  and relation direction;
- deterministic proof excludes every semantic claim named in its boundary section;
- combined-ballot schema and guard pass seeded positive and negative controls;
- family floor,
  self-veto,
  prompt uniqueness,
  image reach,
  exact provider mask,
  duplicate-member,
  abort identity,
  spent-node restart,
  lease,
  and deterministic-skip guards GFP-fail when removed;
- type-aware lint,
  types,
  targeted rebuilt tests,
  and full `buildAndTest` pass;
- advisor reviews the exact implementation and the zero-retry calibration harness;
- complete source,
  archive,
  every candidate,
  selected document,
  and every page image are read before any production-eligibility claim.

Candidate K is selected for structural prototyping only.
It is not selected for production.
It carries no permission to reuse Candidate I prompts or outputs as new provider work.

### Candidate K structural prototype status

Candidate K passed structural implementation at prototype commit
`399d0d686524818cd78e91bdb9417496901f9880`.
Built `prototype-review-unit.mjs` SHA-256 is
`bc64696ca4d7e1066e099718f5c83624eafc1be388d635191214f1c6f4678a83`.
Built test-support SHA-256 is
`1a5db330fed4bfd94e2907cbc447dcab958eb524821c6461c1e3c479c1016cd2`.
Built package index SHA-256 remains
`d67e90eab0b7f1d9c4073f4b33754c6b6fc92b2ec78e2f006217c6c892f0ca75`.

The implementation fixes three Qwen,
GLM,
and MiniMax author nodes plus nine candidate-scoped verifier nodes before provider contact.
It persists readable review plans,
mechanical proof,
manifest and route identities,
model-facing finding rules,
author settlement,
verifier plan,
node records,
responses,
ballots,
selection,
and prompt claims.
An unusable author deterministically skips its three verifier nodes.
A candidate qualifies only with two clean nonself families and no valid self or nonself dissent.
A vetoed higher-priority candidate cannot mask a lower-priority eligible candidate.

The readable pinned-Carena plan contains four semantic front-matter subjects,
112 individual clause statuses in 23 slot groups,
22 ordered relation statuses,
23 slot-language statuses,
and six page-level globals.
Every source excerpt,
range,
digest,
relation endpoint,
direction,
front-matter shape,
supported scalar,
synthetic target slot,
and old-to-new global owner remains manifest-bound.
Scope-specific model-facing rules and caller admission share one digest.
Canonical overflow retains the first 64 defective subjects.

Type-aware lint returned zero warnings and zero errors across 38 scoped files.
Types passed.
The rebuilt targeted suite has 34 cases.
Full `buildAndTest` exited zero with 878 suite `PASS` lines and no suite `FAIL` line.
The post-commit GFP script,
SHA-256 `82ed0f3b8dbaa66e161e1c1d6e3ea4301efcd70fc3d858f73e25511112f9541c`,
mutated 22 load-bearing controls.
Each mutation rebuilt,
made the targeted suite fail,
was restored immediately,
rebuilt,
and passed.
Mutations covered subject-specific global rules,
synthetic-slot collision,
front-matter structure and scalar identity,
proof and cancellation before dispatch,
narrow anchors,
canonical finding prefix,
route cap and deadline,
family floor,
candidate dissent veto,
sibling all-settlement,
exact abort identity,
between-wave cancellation,
indeterminate conversion,
spent restart,
author-dependent skips,
rule-table and image carriage,
provider mask,
and prompt claims.

Offline pinned-Carena request measurement used script SHA-256
`a7974f497ed539e5ba92eb161705a5b227c339c5b48d446a0c35e148fbf4aa30`.
Metadata summary SHA-256 is
`6824feaedd6ea6214e1ad5f19f888c1a7bcc77c3714297344f3dc75ab8656fe5`.
Verifier request bodies measured 1,145,716 to 1,145,719 bytes,
with one exact image,
one forced tool,
and 32,000 `max_tokens`.
The largest constructed admitted 64-finding response measured 37,389 characters.
Prior official-tokenizer artifacts counted it at 20,237 Qwen,
16,102 GLM,
and 14,907 MiniMax tokens.
Token-count metadata SHA-256 is
`7d7ad7bc6049c4414dad4df8863bb80419470ae629051a4805a11d07511a0577`.
These counts are screening evidence,
not hosted-route guarantees.
A separate 84,411-character conservative static superset intentionally violates dynamic subject enums and per-subject
admission bounds;
it is not an admitted maximum.
Its metadata SHA-256 is
`af305cb703e6d8813120e9cf3cabae3546a98f02b69525ce20e7d47a9bf3b8a3`.

The local 900,000-millisecond deadline remains a provisional calibration hypothesis.
Candidate I's GLM rejection applies to its 134-plus-ten candidate-ballot verifier contract;
it does not prove Candidate K's new author or readable-review contracts complete.
Only one zero-retry pinned-Carena run can establish consumer behavior.

Front matter remains a shared archive-derived immutable-shell input,
not independently authored prose.
Every verifier reviews its semantic string values and exact deterministic shape,
but a valid front-matter defect vetoes every candidate and no Candidate K author can repair it.
That common-mode limitation is a calibration hard gate.
It blocks any broader normal-run completion claim until production front-matter authority or authorship is redesigned.

Candidate K may proceed to an advisor-reviewed zero-retry pinned-Carena harness.
It remains ineligible for production integration before calibration and complete-page review.

### Candidate K calibration disposition

Candidate K is rejected after its one zero-retry pinned-Carena calibration.
The exact harness SHA-256 was
`41094dded81174b49bec30b5b7c14966362470524180c6fe44daadb77897886a`.
The output root is
`/var/home/user/temp/agent/prototype-Carena-K-review-unit-20260901`.
The retained summary binds prototype commit,
built artifacts,
corpus commit,
source,
archive,
image,
manifest,
review plan,
verifier rules,
provider routes,
and live catalog.
Every retained file is mode `0600` and every directory is mode `0700`.

The concurrent author wave made three exact Hyper Anthropic Messages exchanges:

- GLM returned HTTP 200 after 648,545 milliseconds,
  emitted only a thinking block,
  and stopped at 32,000 output tokens with `max_tokens`;
- Qwen returned HTTP 200 after 511,425 milliseconds,
  emitted thinking plus a tool-use block,
  and ended with `end_turn`,
  but accumulated tool input was not parseable JSON;
- MiniMax returned HTTP 200 after 184,139 milliseconds,
  emitted thinking plus a tool-use block,
  and stopped at 32,000 output tokens with `max_tokens`.

All three author nodes became `spent-unusable`.
The runtime created no candidate,
deterministically skipped all nine dependent verifier nodes,
and selected nothing.
Fresh and restart results were equal,
the restart transport made zero calls,
and the private artifact tree was unchanged by restart.
The result has three network exchanges,
three spent nodes,
nine deterministic skips,
and twelve terminal static nodes.

There is no candidate document to review or publish.
The source,
archive,
and exact WebP copy match the completely read Candidate I inputs by SHA-256,
but input identity cannot substitute for a candidate.
No complete-page candidate review can begin when candidate admission produced zero documents.

This rejection is an author-protocol result,
not a verifier result.
No Candidate K verifier payload was dispatched.
The readable combined-verifier protocol remains unspent and uncalibrated.
The calibration does not authorize a retry,
continuation,
output-ceiling increase,
partial JSON repair,
reconstruction from hidden reasoning,
or production integration.
Candidate K's author prompts and terminal outputs are permanently spent evidence.

The failure separates two design concerns:

- whole-document slot production remains viable because Candidate I admitted Qwen and MiniMax candidates with the same
  output schema;
- adding the readable audit plan to producer requests did not produce an admitted author in this calibration.

Candidate K therefore keeps its review-unit compiler,
ballot algebra,
and verifier lifecycle as reusable prototype evidence,
but its producer contract is retired.
