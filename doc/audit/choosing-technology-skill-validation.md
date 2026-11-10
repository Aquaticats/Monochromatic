# Choosing-technology skill validation

Status:
PASS.

Execution date:
2026-07-09.

Execution environment:

- selected Pi model:
  `openai-codex/gpt-5.6-sol`;
- thinking level:
  `xhigh`;
- execution mode:
  direct walkthrough by the selected model,
  without delegated scenario execution;
- skill path:
  `.agents/skills/choosing-technology/SKILL.md`;
- skill commit:
  `3b6d1bd6ac0c6eb5704152ddb00e2b69ddcf653b`;
- skill SHA-256:
  `71c50a51d0f0086f789e350ef43824f8aead66435f9ab92d94aae751d16d8359`;
- documentation-policy revision:
  `91c92d54881fa6ba9ab8eb3f14b9b3f193960dbe`;
- plan revision at validation start:
  `c03575520bf2e3169bddd77d2b0623997966fded`.

This document contains synthetic evidence and simulated artifact content.
It creates no fake production vet report or decision record.
Each scenario starts from the rewritten skill and independently follows its declared lifecycle.

## Shared fixture contract

Synthetic evidence records use these fields:

- `source` identifies a fictional primary document,
  source path,
  command log,
  or independent corroboration;
- `claim` states the evidence-bearing fact;
- `gate` states the affected base or overlay gate;
- `confidence` is high,
  medium,
  or low;
- `outcome` is pass,
  fail,
  scored,
  low-signal,
  irrelevant,
  or not-applicable.

Every simulated report delta is embedded here.
`<audit-date>` represents the execution date selected by an evaluator.
No simulated report path is written to the repository.

## S01: saturated discovery with two survivors

### Prompt

> Choose an inspectable TypeScript archive reader for Node.
> It must support streaming reads and the repository license policy.

### Synthetic evidence bundle

- Registry queries `archive stream`,
  `streaming archive reader`,
  and one taxonomy expansion complete every result page.
- Repository-host and web schedules each complete;
  two consecutive complete pages add no survivor.
- Repository inspection finds candidate A and candidate B category-compatible,
  licensed,
  inspectable,
  and plausibly streaming.
- Incumbent and hand-rolled repository searches find no additional survivor.

### Expected route

Both candidates transition through discovered,
screened,
serious alternative,
and hard-gate confirmed.
Both become finalists.
No soft evidence may eliminate either candidate.

### Walkthrough

1.  Classify A and B as inspectable open-source local technologies.
2.  Activate no extra overlay.
3.  Freeze streaming and license as hard constraints.
4.  Record every query and the one expansion round.
5.  Mark every source class saturated.
6.  Promote both hard-gate survivors to finalist validation.
7.  Create the vet report when targeted repository evidence promotes serious alternatives.

### Simulated report delta

```markdown
status: finalist validation
terminal discovery result: saturated with at least two survivors
survivors: [A, B]
exits: []
next legal action: equal-depth finalist validation for A and B
```

### Assertions

- PASS:
  the route requires all discovery source classes.
- PASS:
  saturation uses the recorded query and page rules.
- PASS:
  both hard-gate survivors become finalists.
- PASS:
  the report starts before recommendation.
- PASS:
  no product mutation occurs.

Result:
PASS.

## S02: provider-cap discovery fails closed

### Prompt

> Recommend a hosted artifact registry for a regulated workload.

### Synthetic evidence bundle

- The required vendor directory returns a fixed cap of 100 results and no exhaustion marker.
- The final returned page adds one screening survivor.
- Official API,
  export,
  sitemap,
  and independent enumeration attempts cannot expose uncapped results.
- Other required source classes complete.

### Expected route

The capped class is blocked rather than saturated.
The evaluation finishes a report and recommends none.

### Walkthrough

1.  Classify candidates as managed service or SaaS.
2.  Activate sensitive-data,
    compliance,
    and geography overlays.
3.  Record the cap and every alternate enumeration attempt.
4.  Refuse to apply the two-empty-page rule because the provider never exposed complete pages.
5.  Enter the discovery-blocked terminal outcome.

### Simulated report delta

```markdown
status: complete, no recommendation
terminal result: discovery blocked
blocked source: fictional vendor directory
attempted alternatives: API, export, sitemap, independent enumeration
recommendation: none
```

### Assertions

- PASS:
  a provider cap is not saturation.
- PASS:
  alternate enumeration is attempted and recorded.
- PASS:
  the blocked required class fails closed.
- PASS:
  the report is completed before the response.

Result:
PASS.

## S03: saturated discovery with one survivor

### Prompt

> Choose a self-hosted parser that accepts format Q and runs without native code.

### Synthetic evidence bundle

- All frozen registry,
  repository-host,
  web,
  incumbent,
  and repository queries saturate.
- Candidate A supports format Q and has no native boundary.
- Candidate B fails format Q.
- Candidate C downloads a native binary.
- No other screening survivor appears.

### Expected route

Discovery records one survivor.
A continues through targeted confirmation and full finalist validation.
The workflow does not invent a second live option.

### Walkthrough

1.  Classify all candidates as local technologies.
2.  Apply native/prebuilt overlay to C.
3.  Exit B for hard-constraint failure.
4.  Exit C for the no-native hard constraint.
5.  Record `saturated with one survivor`.
6.  Promote A and run every applicable finalist gate.

### Simulated report delta

```markdown
terminal discovery result: saturated with one survivor
finalist: A
hard-gate exits:
  - B: format Q unsupported
  - C: native artifact violates hard constraint
```

### Assertions

- PASS:
  one survivor continues rather than terminating discovery.
- PASS:
  failed candidates retain evidence-backed exits.
- PASS:
  no invented alternative appears.
- PASS:
  A still receives the full validation bar.

Result:
PASS.

## S04: saturated discovery with zero survivors

### Prompt

> Choose a browser library that supports baseline R,
> has license L,
> and performs no network access.

### Synthetic evidence bundle

- Every source class saturates.
- A fails baseline R.
- B has incompatible license M.
- C performs mandatory network calls.
- No survivor remains.

### Expected route

The evaluation creates and completes a terminal vet report,
recommends none,
and identifies the named constraints that could be changed.

### Walkthrough

1.  Activate the browser-baseline overlay.
2.  Screen each candidate against all hard constraints.
3.  Record three hard-gate exits.
4.  Enter the no-serious-alternative terminal outcome.
5.  Ask only whether the user wants to change a named hard constraint.

### Simulated report delta

```markdown
status: complete, no recommendation
terminal discovery result: saturated with zero survivors
exits:
  - A: baseline R failed
  - B: license L failed
  - C: no-network constraint failed
changeable constraints: [baseline R, license L, no network access]
```

### Assertions

- PASS:
  zero survivors still produces a completed report.
- PASS:
  soft scoring never begins.
- PASS:
  recommendation is none.
- PASS:
  the follow-up names actual constraints rather than proposing unverified custom code.

Result:
PASS.

## S05: managed SaaS routing and sensitivity

### Prompt

> Choose a managed build service.
> Data must stay in region Z.
> Reliability,
> export,
> and support matter,
> but I have not ranked them.

### Synthetic evidence bundle

- Service A and service B publish region-Z processing and storage commitments.
- Service C's primary terms permit processing outside region Z.
- The full 24-month layoff,
  review,
  12-month outage,
  ownership,
  signup,
  and security domains are inspected for A and B.
- A's fictional layoff event affected an unrelated sales region and has no causal link to the proposed use.
- A rates reliability 4,
  export 3,
  support 2,
  residency 4.
- B rates reliability 3,
  export 4,
  support 3,
  residency 4.
- Every initial soft weight is 1.
- Raising reliability weight to 3 changes B over A into A over B.
- Synthetic user preference after the sensitivity question is export and support over incremental reliability.

### Expected route

C fails the residency hard gate.
The unrelated layoff finding is reported with no score effect.
The sensitivity change triggers one controlling preference question,
a rubric refreeze,
and a complete rerun.
B remains the stable recommendation after the resolved preference.

### Walkthrough

1.  Classify A,
    B,
    and C as SaaS.
2.  Activate sensitive-data,
    geography,
    and residency overlays.
3.  Exit C on primary terms.
4.  Inspect every retained and direct SaaS domain for A and B.
5.  Mark the unrelated layoff event `irrelevant` rather than scored.
6.  Compute initial totals:
    A is 13 of 16,
    B is 14 of 16.
7.  Run the complete one-at-a-time matrix and detect the reliability-weight reversal.
8.  Ask only the controlling priority.
9.  Refreeze weights to reflect export and support priority.
10. Rerun baseline,
    every equal-default weight,
    every medium/low confidence adjustment,
    and every range endpoint.
11. Recommend B only after stable ordering.

### Simulated report delta

```markdown
hard-gate exit: C, region-Z residency failed
irrelevant evidence: A layoff event, no operational score effect
sensitivity: reliability weight 3 reverses initial order
resolved preference: export and support over incremental reliability
rerun result: B > A, stable for every defined one-at-a-time test
```

### Assertions

- PASS:
  every historical and direct SaaS domain is inspected.
- PASS:
  residency is a non-compensable hard gate.
- PASS:
  inspected irrelevant evidence receives no points or penalty.
- PASS:
  an outcome-changing weight asks a controlling preference.
- PASS:
  the entire matrix reruns after refreezing.

Result:
PASS.

## S06: pure TypeScript replacement parity

### Prompt

> Evaluate replacing incumbent I with a pure TypeScript parser.
> Keeping I is allowed.

### Synthetic evidence bundle

- Repository consumers exercise sync parse,
  async parse,
  error locations,
  and format-preserving output.
- I,
  candidate A,
  and candidate B contain no native or Wasm artifacts.
- I's reported defect is malformed location output for nested token T.
- Source audit shows I's current revision already fixed T.
- A passes sync but fails async error locations.
- B passes consumed behavior but adds nine runtime dependencies versus I's two.
- I passes every upstream and consumer-boundary fixture.

### Expected route

The incumbent is a full candidate.
Equal-depth parity proves I is the valid winner.
No replacement is recommended merely because the prompt mentions replacement.

### Walkthrough

1.  Classify I,
    A,
    and B as inspectable open-source local technologies.
2.  Apply replacement parity to every candidate.
3.  Inventory every consumed incumbent path.
4.  Audit all candidates to the same source,
    dependency,
    maintenance,
    upstream,
    and consumer depth.
5.  Exit A for a consumed-path failure.
6.  Score I and B only after both validate.
7.  Rank I over B based on measured dependency and migration surface.

### Simulated report delta

```markdown
incumbent defect: fixed in evaluated revision
hard-gate exit: A, async error-location consumer path failed
ranking: I > B > A
recommendation: keep I
```

### Assertions

- PASS:
  keeping the incumbent remains eligible.
- PASS:
  parity covers every consumed path.
- PASS:
  A's consumed failure cannot be scored away.
- PASS:
  the workflow can recommend no replacement.

Result:
PASS.

## S07: native provenance success and failure

### Prompt

> Choose between two image decoders.
> Native code is acceptable only with verifiable provenance.

### Synthetic evidence bundle

- Candidate A maps package checksum `sha256:a1` to source tag `v1`,
  compiler flags,
  release workflow,
  source archive,
  signatures,
  and target matrix.
- Rebuilding A in the declared image reproduces checksum `sha256:a1`.
- Candidate B downloads binary `b.bin` from a release CDN.
- B provides no source mapping,
  compiler flags,
  source archive,
  signature,
  or reproducible build.

### Expected route

A passes the native overlay.
B fails unknown provenance before runtime validation.

### Walkthrough

1.  Classify A and B as local technologies.
2.  Activate native/prebuilt overlay for both.
3.  Record A's source-to-artifact chain and reproducible checksum.
4.  Exit B on unknown build provenance.
5.  Continue A through full validation.

### Simulated report delta

```markdown
native provenance:
  A: pass, rebuilt checksum sha256:a1
  B: fail, downloaded artifact has no source mapping
finalist: A
```

### Assertions

- PASS:
  successful provenance requires more than a published checksum.
- PASS:
  unknown provenance is a hard failure.
- PASS:
  isolation would not rescue B.

Result:
PASS.

## S08: high-trust plugin auditability

### Prompt

> Choose an agent plugin that handles repository credentials.

### Synthetic evidence bundle

- Open plugin A has 1,200 non-test source lines,
  14 files,
  two runtime dependencies,
  and one concentrated credential boundary.
- Open plugin B has 8,900 non-test source lines,
  71 files,
  18 runtime dependencies,
  generated protocol code,
  and three credential/network boundaries.
- Proprietary plugin C exposes no relevant source.
- A and B pass equivalent functional and consumer-boundary checks.
- The user uses none of B's extra UI features.

### Expected route

C fails inspectability.
A and B both receive human-auditability measurement and full validation.
A may outrank B because unused breadth receives no positive score.

### Walkthrough

1.  Apply high-trust execution and sensitive-data overlays.
2.  Exit C before finalist validation.
3.  Measure source,
    dependency,
    control-flow,
    generated-code,
    and credential surfaces for A and B.
4.  Validate A and B equally.
5.  Score measured auditability and exclude unused feature breadth from benefit.
6.  Rank A over B.

### Simulated report delta

```markdown
hard-gate exit: C, proprietary high-trust execution
human auditability:
  A: 1200 source lines, 14 files, 2 runtime dependencies
  B: 8900 source lines, 71 files, 18 runtime dependencies
ranking: A > B > C
```

### Assertions

- PASS:
  proprietary high-trust execution is rejected.
- PASS:
  both open survivors receive equal-depth validation.
- PASS:
  auditability uses measurements.
- PASS:
  unused breadth receives no positive weight.

Result:
PASS.

## S09: proprietary local exception

### Prompt

> Choose a desktop signing client.
> It does not handle repository or CI credentials.

### Synthetic evidence bundle

- Open candidate A fails the required hardware-token protocol.
- Open candidate B supports the protocol only on an unsupported operating system.
- Proprietary candidate P supports the protocol on the required system.
- P publishes signed artifacts,
  documented update behavior,
  vendor security policy,
  and a bounded consumer test path.
- No high-trust overlay applies to P's role in this synthetic context.

### Expected route

P is eligible only because every open-source candidate fails a named hard constraint.
The exception is explicit and does not claim P is inspectable.

### Walkthrough

1.  Apply open-source precedence.
2.  Exit A and B with exact hard-constraint evidence.
3.  Record the proprietary exception naming both failures.
4.  Apply proprietary local gates to P.
5.  Validate P without relabeling it open source.

### Simulated report delta

```markdown
open-source exception:
  - A: required token protocol unsupported
  - B: required operating system unsupported
proprietary finalist: P
inspectability claim: not made
```

### Assertions

- PASS:
  proprietary evaluation follows exhausted open-source failures.
- PASS:
  the exception names hard constraints.
- PASS:
  P is not eligible for a high-trust role.
- PASS:
  P remains explicitly proprietary.

Result:
PASS.

## S10: license and security failures

### Prompt

> Choose a server library compatible with license L and safe for untrusted input.

### Synthetic evidence bundle

- Candidate A's primary license text conflicts with L.
- Candidate B's parser source performs an unchecked allocation from attacker-controlled length and
  its security advisory confirms exploitable denial of service in the evaluated version.
- Candidate C has compatible license text,
  bounded parsing,
  adversarial tests,
  and a patched advisory history.

### Expected route

A exits on license.
B exits on the security boundary.
Only C advances.

### Walkthrough

1.  Freeze license L and untrusted-input safety as hard constraints.
2.  Use primary license text for A.
3.  Use source plus advisory evidence for B.
4.  Record both hard failures outside scoring.
5.  Validate C.

### Simulated report delta

```markdown
hard-gate exits:
  - A: incompatible primary license text
  - B: evaluated parser version fails untrusted-input security boundary
finalist: C
```

### Assertions

- PASS:
  license failure is non-compensable.
- PASS:
  security failure is non-compensable.
- PASS:
  primary and source evidence control the exits.

Result:
PASS.

## S11: platform and browser routing

### Prompt

> Choose a client library for Linux x64,
> macOS arm64,
> Windows x64,
> and the repository browser baseline.

### Synthetic evidence bundle

- Candidate A claims all targets and has upstream jobs for each.
- Candidate B omits macOS arm64 from its target matrix.
- Synthetic validation logs pass A on all named platform routes and required browsers.
- The Markdown skill itself remains platform-neutral.

### Expected route

The platform and browser overlays activate.
B fails a hard platform gate.
A routes to every available required target and browser check.

### Walkthrough

1.  Record exact required platforms and repository browser baseline.
2.  Apply multi-platform and browser overlays.
3.  Exit B on its published target matrix.
4.  Route A to Linux,
    macOS,
    Windows,
    and browser validation.
5.  Record that scenario routing,
    not cross-machine Markdown execution,
    is under test.

### Simulated report delta

```markdown
required routes: [linux-x64, macos-arm64, windows-x64, repository-browser-baseline]
hard-gate exit: B, macos-arm64 unsupported
A validation: every required route passed
```

### Assertions

- PASS:
  every required platform becomes a gate.
- PASS:
  browser baseline becomes an overlay gate.
- PASS:
  unsupported platform failure occurs before scoring.
- PASS:
  validation semantics do not require the skill file to execute on each platform.

Result:
PASS.

## S12: low-signal tracker activity

### Prompt

> Evaluate a focused open-source serializer with three source files.

### Synthetic evidence bundle

- The tracker has two issues in 12 months and no pull request.
- Both issues are user questions with no maintainer comment.
- The maintainer published four signed releases,
  authored compatibility commits,
  updated dependencies,
  and ran required CI in the same period.
- Source and consumer fixtures pass.

### Expected route

The sparse tracker remains low-signal.
Release,
ownership,
source,
and validation evidence can still support a maintenance score.
Silence is not called health.

### Walkthrough

1.  Inspect both issues because the sample is at most 20.
2.  Record zero maintainer tracker responses.
3.  Separately measure maintainer-authored work and release cadence.
4.  Mark tracker evidence low-signal.
5.  Score maintenance using the broader evidence with stated confidence.

### Simulated report delta

```markdown
tracker evidence: low-signal, 2 inspected issues, 0 maintainer comments
release evidence: 4 signed releases with required CI
maintenance conclusion: supported by release and ownership evidence, not tracker silence
```

### Assertions

- PASS:
  every issue is inspected under the sample rule.
- PASS:
  zero tracker activity is not proof of health.
- PASS:
  other maintenance evidence remains usable.

Result:
PASS.

## S13: equal-depth finalist validation

### Prompt

> Compare three inspectable task runners.

### Synthetic evidence bundle

- A,
  B,
  and C all pass targeted hard-gate confirmation.
- Preliminary documentation makes A appear richer,
  B appear leaner,
  and C appear less maintained.
- Each exposes default CI,
  integration,
  Windows,
  and consumer-boundary suites.

### Expected route

All three become finalists.
Each receives the same applicable source and execution depth before scoring.

### Walkthrough

1.  Promote A,
    B,
    and C after hard-gate confirmation.
2.  Ignore preliminary soft ranking for promotion.
3.  Audit source,
    maintenance,
    default CI,
    integration,
    Windows,
    and consumer boundaries for all three.
4.  Score only after all validation records are complete.

### Simulated report delta

```markdown
finalists: [A, B, C]
validation matrix:
  A: source, maintenance, default CI, integration, Windows, consumer boundary
  B: source, maintenance, default CI, integration, Windows, consumer boundary
  C: source, maintenance, default CI, integration, Windows, consumer boundary
```

### Assertions

- PASS:
  every hard-gate survivor becomes a finalist.
- PASS:
  preliminary soft evidence eliminates none.
- PASS:
  each finalist receives the same applicable validation set.

Result:
PASS.

## S14: upstream failure outside every relevant surface

### Prompt

> Choose a library for its Node parser only.
> Browser widgets are not consumed or claimed.

### Synthetic evidence bundle

- Candidate A's full default CI fails a browser-widget visual snapshot.
- Source trace proves the widget package is separately built,
  separately published,
  and unreachable from the Node parser package.
- Node unit,
  integration,
  package,
  and consumer-boundary suites pass.
- No requested claim mentions browser widgets.

### Expected route

The failure is diagnosed and may be excluded only with exact out-of-surface proof.
A can remain validated for the scoped claim.

### Walkthrough

1.  Run the complete default CI-equivalent path and record failure.
2.  Diagnose the exact failing job and source boundary.
3.  Prove the failed widget path is outside every claimed and consumed path.
4.  Record exact command,
    output,
    path trace,
    and rationale.
5.  Continue relevant validation.

### Simulated report delta

```markdown
upstream failure: browser-widget visual snapshot
scope proof: separately built and published; unreachable from consumed Node parser
claimed or consumed effect: none
candidate status: validated for scoped Node parser use
```

### Assertions

- PASS:
  the default failure is not ignored.
- PASS:
  exclusion requires exact scope proof.
- PASS:
  all relevant suites still run.

Result:
PASS.

## S15: unavailable relevant validation

### Prompt

> Choose a native database driver whose transaction path must work on Windows.

### Synthetic evidence bundle

- Candidate A is the only hard-gate survivor.
- Its Windows transaction integration suite requires an unavailable proprietary test server.
- The suite source shows it exercises the exact consumed transaction path.
- No inspectable substitute,
  replay fixture,
  or bounded execution path exists.

### Expected route

A fails because relevant validation cannot run.
With no survivor,
the evaluation recommends none.

### Walkthrough

1.  Confirm the unavailable suite is relevant by reading its source.
2.  Attempt inspectable substitute paths and record their absence.
3.  Fail A rather than treating unavailability as a soft concern.
4.  Enter the relevant-execution-unavailable terminal outcome.

### Simulated report delta

```markdown
candidate: A
failed gate: relevant Windows transaction validation unavailable
survivors: []
recommendation: none
```

### Assertions

- PASS:
  a relevant unavailable suite fails the candidate.
- PASS:
  no score can rescue A.
- PASS:
  zero remaining survivors yields no recommendation.

Result:
PASS.

## S16: zero criteria and exact tie

### Prompt

> Choose between two validated formatters.
> They meet every requirement and I have no further preference.

### Synthetic evidence bundle

- A and B pass identical hard constraints.
- No decision-level soft criterion applies.
- Their factual integration,
  migration,
  and exit tradeoffs are equivalent in the synthetic evidence.

### Expected route

Both receive `score: not applicable`.
The exact factual tie asks for a controlling preference.
The model does not invent points or a tiebreaker.

### Walkthrough

1.  Validate both finalists equally.
2.  Remove criteria irrelevant to the whole decision.
3.  Record score not applicable for both.
4.  Inspect factual tradeoffs and confirm they do not order candidates.
5.  Ask the unresolved preference.

### Simulated report delta

```markdown
scores:
  A: not applicable
  B: not applicable
ordering: exact tie
next action: ask controlling preference
```

### Assertions

- PASS:
  the zero-denominator case avoids arithmetic.
- PASS:
  both candidates receive the same score status.
- PASS:
  no tiebreaker is invented.

Result:
PASS.

## S17: sensitivity resolution and unresolved instability

### Prompt

> Compare open tools A and B.
> I have not ranked startup speed against auditability.

### Synthetic evidence bundle

- A rates speed 4 and auditability 2.
- B rates speed 2 and auditability 4.
- Both initial weights are 1,
  producing an exact tie.
- Route R1 supplies the user preference `auditability over startup speed`.
- Route R2 supplies `no preference`.
- Each candidate also has one medium-confidence reliability rating,
  and A has a low-signal range from 2 through 3.

### Expected route

R1 refreezes weights and reruns the complete matrix.
R2 completes conditional rankings and recommends none.

### Walkthrough

1.  Run every equal-default weight from 1 through 5.
2.  Move both medium-confidence exact ratings down and up one step.
3.  Test both endpoints of A's range.
4.  Detect order changes.
5.  For R1,
    record the preference,
    refreeze,
    rerun baseline and every sensitivity input,
    and recommend only if stable.
6.  For R2,
    publish which weights produce A and B,
    then recommend none.

### Simulated report delta

```markdown
R1:
  resolved preference: auditability over startup speed
  action: rubric refrozen and complete matrix rerun
  result: B > A, stable
R2:
  unresolved input: startup speed versus auditability
  conditional ranking: A wins when speed dominates; B wins when auditability dominates
  recommendation: none
```

### Assertions

- PASS:
  the defined matrix includes weights,
  confidence moves,
  and range endpoints.
- PASS:
  resolution triggers a complete rerun.
- PASS:
  unresolved instability produces conditional rankings and no recommendation.

Result:
PASS.

## S18: fingerprint and slug determinism

### Prompt

> Reopen the compatible vet report for `Café / Build`.

### Synthetic evidence bundle

Normalized fingerprint input:

```json
{
  "baseCategories": ["local"],
  "deployment": {"arch": "x64", "os": "linux"},
  "hardConstraints": ["NFC café"],
  "incumbent": null,
  "overlays": ["native"],
  "schemaVersion": 1,
  "scope": "build tool",
  "subject": "Café / Build"
}
```

Expected RFC 8785 bytes,
shown as consecutive fragments that are concatenated without a separator or newline:

```text
{"baseCategories":["local"],"deployment":{"arch":"x64","os":"linux"},
"hardConstraints":["NFC café"],"incumbent":null,"overlays":["native"],
"schemaVersion":1,"scope":"build tool","subject":"Café / Build"}
```

Expected SHA-256:
`fecd6f8e299a48add12f7cd948b25360f8e2534793a7a715ce484067a54b1a09`.

Additional fixtures:

- NFD `Café / Build` normalizes to the same NFC input.
- Expected subject slug is `tech-caf-build`.
- Fingerprints F1 and F2 share eight prefix characters but differ within the next four.
- A path storing a different full fingerprint under an identical 64-character qualifier represents corruption.
- Status,
  owner,
  last-updated date,
  and skill revision vary without changing the compatibility fingerprint.

### Expected route

Canonical bytes and normalization match.
Collision suffixes extend deterministically.
Mutable metadata is excluded.
Corruption blocks writing.

### Walkthrough

1.  Normalize every string to NFC.
2.  Apply subject/scope outer trimming only.
3.  Sort set-valued arrays and deployment keys as specified.
4.  Serialize to the exact bytes shown.
5.  Verify the expected SHA-256.
6.  Scan the slug by ASCII policy and obtain `tech-caf-build`.
7.  Extend F2's qualifier from eight to 12 characters.
8.  Block the full-hash corruption fixture.
9.  Recompute after mutable metadata changes and confirm unchanged identity.

### Simulated report delta

```markdown
subject slug: tech-caf-build
compatibility fingerprint: fecd6f8e299a48add12f7cd948b25360f8e2534793a7a715ce484067a54b1a09
collision behavior: 8 characters, then 12 characters
corruption behavior: blocked
mutable metadata effect: none
```

### Assertions

- PASS:
  canonical bytes match the expected UTF-8 content.
- PASS:
  NFC and NFD inputs converge.
- PASS:
  the slug excludes unsafe path syntax.
- PASS:
  collision extension is deterministic.
- PASS:
  full-hash disagreement blocks the write.
- PASS:
  mutable metadata does not change compatibility.

Result:
PASS.

## S19: report reuse and concurrent ownership

### Prompt

> Continue the build-tool evaluation from its existing report.

### Synthetic evidence bundle

- Reports R1 and R2 share the full compatibility fingerprint.
- R1 has `last-updated: 2026-06-01`.
- R2 has `last-updated: 2026-06-02`.
- R3 has the same subject and date as R2 but an incompatible fingerprint.
- R3's eight-character qualifier is already occupied by another fingerprint,
  while its 12-character qualifier is free.
- A live lock records an existing process and start time 10 minutes ago.
- A second lock records an absent process but start time 10 minutes ago.
- A third lock records an absent process and start time 31 minutes ago.

### Expected route

R2 is reused.
R3 receives a 12-character qualified same-day path.
The live and young-absent locks block.
Only the old absent lock is stale.

### Walkthrough

1.  Match reports by full fingerprint.
2.  Select R2 by greatest last-updated date.
3.  Record R1 as a duplicate.
4.  Create R3's incompatible path with progressive qualifier extension.
5.  Refuse deletion of the first two locks because both stale conditions do not hold.
6.  Permit stale handling only for the third lock.
7.  Rerun selection after lock acquisition and compare pre-edit hashes before rename.

### Simulated report delta

```markdown
selected compatible report: R2
compatible duplicate: R1
incompatible same-day path: 12-character fingerprint qualifier
lock outcomes:
  live-young: blocked
  absent-young: blocked
  absent-old: stale
```

### Assertions

- PASS:
  compatible reuse is deterministic.
- PASS:
  incompatible same-day work receives a separate path.
- PASS:
  stale status requires absent process and age over 30 minutes.
- PASS:
  uncertain ownership blocks rather than overwrites.
- PASS:
  update protocol includes a compare-before-rename check.

Result:
PASS.

## S20: evaluation writes only documentation

### Prompt

> Evaluate archive libraries and recommend one.

### Synthetic evidence bundle

- The substantial-evaluation threshold is crossed after repository evidence promotes candidate A.
- The main worktree contains product source,
  package manifests,
  lockfiles,
  configuration,
  generated output,
  docs,
  and audit reports.
- No adoption action appears in the prompt.

### Expected route

The model may update the plan,
validation documentation,
and compatible vet report in the main worktree.
It must not edit product source,
dependencies,
lockfiles,
configuration,
generated output,
or decision records.

### Walkthrough

1.  Apply the decision-verb policy.
2.  Create the vet report at threshold crossing.
3.  Update and commit report checkpoints.
4.  Keep recommendation separate from adoption.
5.  Reject every simulated product or decision mutation.

### Simulated scoped diff

```diff
+ doc/audit/tech-archive-reader-vet-<audit-date>.md
~ doc/audit/tech-archive-reader-vet-<audit-date>.md
- no product source change
- no package or lockfile change
- no configuration or generated-output change
- no decision-record change
```

### Assertions

- PASS:
  documentation and reports write directly in the main worktree.
- PASS:
  product,
  dependency,
  configuration,
  and generated-output mutation remains forbidden.
- PASS:
  recommendation does not imply adoption.

Result:
PASS.

## S21: adoption after separate authorization

### Prompt sequence

First prompt:

> Evaluate archive libraries and recommend one.

Later separate prompt:

> Adopt candidate A from the completed archive-library vet report.

### Synthetic evidence bundle

- The completed compatible report recommends A over B.
- It records version `1.2.3`,
  revision `abc123`,
  ranking `A > B`,
  migration steps,
  exit path,
  rollback,
  and revisit triggers.
- The first prompt contains no action authorization.
- The later prompt explicitly authorizes adoption.

### Expected route

The first prompt creates no decision record.
The later prompt may update product integration and the decision record with provenance.

### Walkthrough

1.  During evaluation,
    finish only the vet report and recommendation.
2.  On the later action request,
    verify it targets the completed report and candidate A.
3.  Create or update the project or package decision record.
4.  Include candidate version,
    revision,
    adoption date,
    authorizing request,
    linked report,
    skill revision,
    constraints,
    weights,
    full ranking,
    rejected alternatives,
    integration,
    migration,
    exit,
    rollback,
    and revisit triggers.

### Simulated decision-record content

```markdown
## Archive reader

adopted: candidate A 1.2.3 at abc123
authority: later explicit `adopt candidate A` request
vet report: doc/audit/tech-archive-reader-vet-<audit-date>.md
ranking: A > B
integration: package-local reader boundary
migration: replace incumbent calls through the boundary
exit: restore incumbent adapter
rollback: revert integration commit and lockfile revision
revisit: security advisory, unsupported runtime, failed consumer boundary
```

### Assertions

- PASS:
  the evaluation prompt creates no decision record.
- PASS:
  only the later action prompt authorizes adoption.
- PASS:
  the decision record carries report,
  ranking,
  integration,
  migration,
  exit,
  rollback,
  and revisit provenance.

Result:
PASS.

## Coverage audit

The selected-model walkthroughs cover:

- every discovery terminal result;
- provider-cap failure;
- every base category;
- sensitive-data,
  geography,
  replacement,
  high-trust,
  native,
  platform,
  and browser overlays;
- license,
  security,
  residency,
  provenance,
  inspectability,
  and validation hard failures;
- universal SaaS inspection with relevance-gated scoring;
- open-source source,
  maintenance,
  parity,
  and human-auditability routing;
- equal-depth finalist validation;
- out-of-surface upstream failure proof;
- unavailable relevant execution;
- weighted scoring,
  zero criteria,
  tie,
  sensitivity resolution,
  full rerun,
  and unresolved instability;
- RFC 8785 bytes,
  Unicode normalization,
  safe slugging,
  compatibility,
  collision,
  corruption,
  reuse,
  and concurrent ownership;
- automatic report creation;
- authorized documentation writes;
- recommendation-versus-adoption authority.

Every scenario passed every declared assertion.
No scenario exposed a missing route or contradictory terminal state in the rewritten skill.
