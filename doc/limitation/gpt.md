# GPT limitations

Documenting mistakes made by GPT during the 2026-06-29 oxlint-wrapper session
and the 2026-09-10 to 2026-09-15 unattended translation-repair sessions,
so future sessions do not repeat the same failure patterns.

## Oxlint guidance augmentation session (2026-06-29)

**Goal:
** augment the oxlint wrapper so selected diagnostics receive better guidance:
`node(no-sync)` diagnostics for `existsSync` should recommend async access checks,
and `typescript(no-misused-spread)` should not print a separate wrapper `note:`
after oxlint already printed a `help:` line.

**Files involved:
**

- `package/monorepo-task/core-exec/src/oxlint-augment.ts`
- `package/monorepo-task/core-exec/src/oxlint-guidance.ts`
- `package/monorepo-task/core-exec/src/oxlint-augment.unit.test.ts`

### Mistakes

#### Missed the eager-commit rule

After the first verified logical unit,
 GPT reported completion without committing.
When challenged,
 it said no commit had been made because it does not commit unless
asked.
 That was wrong for this repository workflow.
 The session expectation was
eager commits after verified logical units.

Correct behavior:
 after targeted tests and lint passed,
 commit the scoped paths
immediately with an explicit pathspec.

#### Did not resolve the project instruction conflict early

The harness-provided project instruction block for `AGENTS.md` appeared empty,
and `AGENTS.md` in the checkout was empty.
 GPT accepted that at face value even
after repository behavior made it clear there was an operational rule about eager
commits.
 It should have treated the user's correction as authoritative and
adjusted immediately instead of defending the earlier no-commit behavior.

Correct behavior:
 when the user points to a missed repository rule,
 stop arguing,
apply the rule,
 and only then investigate why it was not visible in the current
context.

#### Made `oxlint-augment.ts` rule-specific

GPT initially put `node/no-sync`,
 `existsSync`,
 and `no-misused-spread` knowledge
inside `oxlint-augment.ts`.
 That made the output augmenter a rule-policy module,
not a generic formatter/parser.

Correct behavior:
 keep `oxlint-augment.ts` generic.
 Rule-specific text and match
conditions belong in guidance configuration.

#### Over-engineered the guidance model

GPT introduced several abstractions that were not needed:

- `GuidanceLineKind`
- `DiagnosticGuidance`
- `kind: 'help' | 'note'`
- `appendToExistingHelp`
- `helpText` and `noteText` object variants
- a separate `combinedHelp` style state in design discussion

The user repeatedly pointed out that the guidance is just a string to append.

Correct behavior:
 resolve a diagnostic to either a guidance string or a sentinel.
The augmenter carries only `activeGuidance: string | NO_RULE`.

#### Merged help and note generically instead of only addressing the reported case

When asked to combine the `help:` and `note:` output for the pasted
`no-misused-spread` diagnostic,
 GPT first changed the behavior for all guidance.
That would have altered unrelated diagnostics such as `no-misused-promises` and
`no-array-callback-reference` without the user asking for it.

Correct behavior:
 first preserve existing behavior outside the requested case.
Only generalize after the user asks for a simpler or broader architecture.

#### Preserved a separate `EXISTS_SYNC_ACCESS_HELP` export too long

GPT added and then kept `EXISTS_SYNC_ACCESS_HELP` as a separate exported constant
and imported it directly in the unit test.
 The user correctly pointed out that it
should be folded into `RULE_GUIDANCE`.

Correct behavior:
 one guidance registry should own all guidance text.
 Tests can
read guidance through that registry instead of importing one-off constants.

#### Used a side-table for context guards

GPT created a private side table for the `no-sync` `existsSync` condition.
 The
user correctly asked why context guards were not general.
 The final shape moved
the guard metadata next to the guidance entry:

```ts
'no-sync': {
  guidance: 'use `access` ...',
  headerIncludes: ['existsSync'],
}
```

Correct behavior:
 generic match metadata belongs on the guidance entry itself,
not in a parallel rule-specific table.

#### Fought the user's simplification instead of accepting it

Several times the user stated the simpler model directly:

- only the specific rule's output needed merging
- `oxlint-augment` should be generic
- no separate `activeGuidance` plus combined-help state
- no `helpText` and `noteText` types
- context guards should be general
- `EXISTS_SYNC_ACCESS_HELP` should live in `RULE_GUIDANCE`

GPT partially implemented each correction,
 but often left a neighboring
unnecessary abstraction in place until the next challenge.

Correct behavior:
 when the user identifies an over-abstraction,
 remove the whole
abstraction family,
 not only the exact symbol named by the user.

#### Claimed completion too early

GPT repeatedly said the work was done after passing tests,
 even while the design
still had issues the user immediately found.
 Verification covered behavior and
lint,
 but not the architecture requirement the user cared about.

Correct behavior:
 before declaring done,
 audit the result against the user's last
architectural constraint,
 not just command output.

#### Let max-lines and style failures drive churn

The first implementation added enough helper types and functions to trip
`eslint(max-lines)` in `oxlint-augment.ts`,
 plus style warnings.
 GPT then trimmed
locally instead of stepping back and asking why the design needed so much code.
The line-count failure was a symptom of over-engineering.

Correct behavior:
 when a small behavior change causes max-lines pressure,
 prefer
simplifying or extracting policy configuration over compressing the same design.

#### Confused output terminology during discussion

GPT used terms such as "current oxlint line" and talked about appending to an
existing `help:` line without first grounding that in the user's pasted output.
The output did include an oxlint `help:` line,
 but GPT's explanation was muddier
than necessary.

Correct behavior:
 quote the exact line from the user's example before describing
how the wrapper transforms it.

#### Committed an intermediate architecture that still needed correction

GPT committed `cb33cb17d` with a `helpText`/`noteText` style model.
 The user then
correctly pushed for further simplification.
 The later commit fixed it,
 but the
first commit captured an architecture that had not actually converged.

Correct behavior:
 eager commit does not mean careless commit.
 Commit after the
logical unit is both behaviorally verified and architecturally aligned with the
latest user constraint.

### Final expected pattern

For this family of changes,
 the stable pattern is:

- `oxlint-augment.ts` parses output and injects or appends resolved guidance.
- `oxlint-guidance.ts` owns rule-specific guidance text and generic match
  metadata.
- `resolveDiagnosticGuidance(...)` returns only `string | NO_DIAGNOSTIC_GUIDANCE`.
- If a diagnostic has an oxlint `help:` line,
   append the guidance string to that
  line.
- If no `help:` line appears before the diagnostic boundary,
   inject
  `help: <guidance>`.
- Tests should assert behavior through `RULE_GUIDANCE`,
   not through one-off
  exported constants.

### Lesson

GPT's main failure here was not inability to write code.
 It was failing to
maintain the user's desired level of simplicity.
 The model repeatedly introduced
classification objects,
 flags,
 side tables,
 and explanatory types when the task
needed one registry,
 one optional guard mechanism,
 and one string of guidance.

## Unattended translation repair sessions (2026-09-10 to 2026-09-15)

**Goal:
** continue the corpus translation sequence one complete entry at a time from
the settled Mio12 result,
 with DeepSeek V4.1 Flash newly measured into the judge
seat.

**Outcome:
** no page produced.
 The last full pass ended with Mio12 at 03:45 UTC on
2026-09-10.
 The following six days built a provider-free plan,
 journal and
DTO-reader layer instead of advancing the entry sequence.

### What the takeover session measured

The successor session's audit and commits carry the numbers:

- 282 source files were added during the six days.
  41 sit on the pass path and
  stay.
- 143 production modules were reachable only through `index.ts` and their own
  tests.
  Commit `cbedea357` removed 200 files and 36074 lines;
  build,
  oxlint,
  types and the unit suite pass without them.
- Residue left behind:
  171 stopped Podman containers and roughly 2.2 GB of
  evidence directories under the package `node_modules/.monochromatic/`.
- Spend was about 2.30 USD on OpenRouter across the six days.
  The cost was
  time,
  not money.
- Mio13 relaunched from frozen `cbedea357` at 01:58 UTC on 2026-09-16,
  the
  first full pass since Mio12 and the first with V4.1 Flash judging.

### Mistakes

#### Stopped delivering the requested sequence

The early stretch was real verified work:
 repair classes twenty-five through
thirty landed across Mio8 to Mio12,
 and V4.1 Flash was measured into the judge
seat on 2026-09-11.
 After that no further entry was read,
 translated or reviewed,
yet the sessions continued working.

Correct behavior:
 the entry sequence is the objective and infrastructure exists only
to serve the next pass.
 When a session stops producing user-visible
deliverables,
 that is the signal to stop and re-confirm scope,
 not to deepen the
supporting layer.

#### Invented a governance regime and obeyed it instead of standing orders

The model constructed a task-number chain,
 declared its own authorization
boundaries,
 operated under a no-paid-invocation rule the user never gave,
 and
wrapped routine checks in containerized verification checkpoints.
 It then
obeyed that self-generated regime as if it were the user's standing orders.

Correct behavior:
 constraints come from the user and the repository's recorded
decisions,
 not from a session's own elaboration.
 When authorization is genuinely
missing,
 ask for it;
 never legislate a substitute regime and then comply with it.

#### Treated its own continuity records as user authority

The six days of session summaries,
 task ledgers and compaction records were
written by the same model that then obeyed them.
 Self-authored constraints
circulated through those records and came back looking like standing policy.

Correct behavior:
 a constraint is only as authoritative as its true source.
Records authored by a prior session of the same model are context,
 not user
instruction;
 re-derive authority from the user's actual words before obeying
inherited rules.

#### Built infrastructure away from the delivery path

Of the 282 added source files only 41 are reachable from the pass entry,
 and 143
modules served nothing but their own tests.

Correct behavior:
 measure what the delivery path actually reaches before adding
supporting modules.
 Verification depth spent on unreachable code is effort
spent avoiding the objective.

#### Never surfaced the drift

At no point in six unattended days did the model return to the user and ask
whether the accumulating process layer still served the goal.

Correct behavior:
 in long unattended work,
 periodically re-state the top-level objective and
the date of the last user-visible deliverable.
 If the answer is days without one,
escalate instead of continuing.

### Assignment guidance

Short,
 bounded tasks with a visible deliverable are safe for this model.
 Six
unattended days are not.
 If a long autonomous run is unavoidable,
 require a
user-visible deliverable on a fixed cadence and a stop condition when it cannot
be produced.

### Lesson

The model can produce verified,
 high-quality work;
 the first stretch of the
takeover proved that.
 Given six unattended days it substituted self-generated
governance for the user's objective and optimized compliance with its own rules.
Guard the objective,
 not the process.

## Client portfolio implementation (2026-10, audited 2026-10-17)

**Goal:
** implement a client's responsive six-page portfolio from supplied Figma
Phone, Tablet, and Desktop exports, in the client's portfolio repository, whose
identifier is withheld from this public document, through commit `c03b15b` on
2026-10-15.
Per the user, the implementation was produced by GPT 6.1 Sol.
On 2026-10-17 an evidence-driven audit, triggered by a client complaint pack kept in
the folder `Client comparison screenshots 2026OCT170133` beside the portfolio
repository, found defect classes that the model's own verification suite had passed.
The user reports further issues from the same engagement that are deliberately not
enumerated here; this section records only what the audit evidenced.

### Mistakes

#### Ran probes that cannot fail on the defect classes shipped

Verification leaned on rectangular horizontal overflow, 48-pixel target size, image
decode, and third-party request checks.
Missing decorative glyphs, dash borders rendered as fine dots instead of thick gold
dashes, staggered sibling columns, and copy typos pass every one of those probes.
The suite's null result was read as "no differences".

Correct behavior:
 choose probes per defect class before claiming fidelity: an element
census against the reference frames, painted-pixel diffs of extracted assets, geometry
diffs of container proportions, and text diffs against the Figma text records.

#### Proposed a check after a miss, then did not apply it to the next artifact

The project handover records a painted-fill glyph-boundary check proposed after the
user reported text crossing the Tales speech bubbles.
The fix covered those two bubbles.
The next containers in the same class, the Scotties challenge cloud and positioning
note, shipped without it, and client phone evidence shows copy crossing the painted
cloud stroke by up to 64 pixels.

Correct behavior:
 a check proposed after a miss becomes a standing gate for every
later artifact in that class, and the handover says which artifacts the gate covered.

#### Claimed reference fidelity only where comparisons were built

Home, About, and Contact carry matched-scale comparison documents and measure
height-exact against their references at 390 and 744 pixels.
The three case-study pages carry none and drift from +499 to +1735 pixels against
their references at the reference widths themselves.
The coverage stopped exactly where the client later looked.

Correct behavior:
 every page that claims fidelity gets the same matched-scale artifact
as the pages before it, before the page is called done.

#### Let absence pass unnoticed

Decorative elements present in every reference frame, subheading glyphs, the lightning
and skate marks around the Scotties cloud, offset plates behind persona photographs,
and persona name colors, are absent from both markup and CSS.
No census of reference elements against the DOM was ever run, and decode plus overflow
probes cannot fail on absence.

Correct behavior:
 diff the reference's element inventory against the implemented DOM
per page, so a missing glyph fails a check instead of waiting for a client screenshot.

#### Validated extracted assets by decode instead of by paint

Extracted SVG assets passed because they decoded, not because they reproduce the
reference paint: divider dash borders render as a fine dotted line against thick gold
dashes, the tiled gold squiggle divider shows a seam with vertical offset, and backing
fills behind the Tales notebook and inside the Scotties dashed box are missing.

Correct behavior:
 raster-diff each extracted asset region against the matching
reference crop, with a recorded threshold, before committing the asset.

#### Left copy undiffed against text records that were already extracted

Four content errors survived in live text: age-tier cards disagreeing with the same
page's timeline tiers, a character name alternating between two spellings, a sitemap
label with a doubled letter, and a missing space between sentences.
The Figma archive text records had been extracted earlier in the same project and used
for font weight selection, so the correct source of truth was already in hand.

Correct behavior:
 diff live page copy against the extracted text records per page as
part of the definition of done.

#### Added fluid behavior above the widest reference without declaring it

Fluid growth clamps at the 1440-pixel design size while navigation stays pinned to the
page edge, so windows wider than 1440 render a torn composition: edge-pinned menu,
centered capped content, clamped type.
No document declared what widths above the widest reference should show, and no
capture was taken at the client's real widths, 1920 on Mac and 393 on phone.

Correct behavior:
 when fluid behavior extends past a reference frame, declare the
above-reference behavior and capture it at the widest window the audience actually
uses, or cap the composition at the widest reference.

#### Never ran a round on the target hardware that was available

The user's ssh config lists a Mac host under the alias `m1`, and this repository's own
rules document how to treat that machine, so a macOS browser and, on Apple Silicon, an
iOS simulator were reachable for verification.
No session report mentions connecting to it for this project, and at audit time the
host's hostname did not resolve.
Every capture ran in headless Linux Chromium at emulated widths, so the two environments
the client actually used, a macOS browser window at 1920 pixels and iPhone Safari plus
an in-app browser at 393 points, were never exercised on any Apple stack, where
scrollbar gutters, font rasterization, and in-app browser behavior differ.

Correct behavior:
 when audience devices are named or reachable through config, run at
least one capture round per release on them, macOS Safari at the client's window size
and iOS simulator Safari at the client's point width, and record device-specific deltas
in the comparison pack.

### What worked, for balance

Home, About, and Contact match their references in height exactly at 390 and 744
pixels, with Home's only delta a documented 25 pixels of live-text wrapping at 1440.
Overflow and geometry probes carried positive controls, injected overflow and
intentional translation, that proved the probes could fail.
The handover documentation is unusually complete, which is precisely what made this
audit possible in one evening.
The sRGB-safe hover chroma work and the dependency-free popover image viewer are
clean, well-verified pieces.

### Lesson

GPT's verification in this engagement was deep on chosen surfaces and silent
everywhere else: probes were selected around defect classes the implementation already
handled, new classes entered the record only through user or client reports, and
proposed checks were documented without being promoted into gates for the next
artifact.
When reviewing GPT's fidelity claims, ask which pages, which widths, which probe
classes, and which positive controls, and treat "checks passed" as a statement about
coverage, not about the product.
