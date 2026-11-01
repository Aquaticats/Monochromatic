# Translation repair history 2026-08-12 to 2026-08-15: the overnight census of unread signals and the open decisions

The autonomous stretch of 2026-08-13 that censused every signal the deterministic core emits,
the defects it fixed,
the quote-anchoring and silent-refiner findings,
the aligner fix that was never wired in,
and the consolidated list of every open decision at the end of it.
The same night is also recorded in `translation-repair-history-first-handover-to-2026-08-15-overnight-2026-08-13-alignment-rebuild-and-round-three-grading-steps.md`,
in the first handover group,
which records the alignment fix refuted and rebuilt.

Part of [Translation repair history: 2026-08-12 to 2026-08-15](translation-repair-history-2026-08-12-to-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Overnight autonomous stretch, 2026-08-13 (03:20 to 05:00 UTC)

The user was asleep with a standing instruction to keep working,
to land
 certainly-good pipeline changes immediately,
and to restart runs as needed.
24 commits.
Everything below is committed and pushed;
the tree is clean and
 212 test suites pass with lint clean apart from the ignored rule.

### What is running right now

`pass12`,
into `node_modules/.monochromatic/translation-repair-runs-pass12`,
 started 04:35 UTC under every fix listed here EXCEPT the naturalness
 eligibility one.
It settles roughly one entry per 45 minutes,
so a full pass is
 days rather than hours.

`pass10` (3 settled entries,
old pipeline) and `pass11` (nothing settled) were
 stopped.
`pass10`'s artifacts are kept and are a CONSISTENT old-pipeline
 population,
not a mixed one.
Full reasoning in
 `doc/troubleshooting/translation-repair-run-invalidation.md`.

### Four decisions waiting, none blocked on further work

-   `#70` and `#71`,
    the same question.
    Its two genuinely untranslated sections
    need a destination and both available answers are bad:
    an empty target
    produces the unrepairable 915-characters-against-nothing shape,
    and
    skipping contradicts the decided output goal.
    `doc/planning/translation-pipeline-redesign.md`.

    CORRECTED 2026-08-13:
    this entry previously said the section aligner now
    pairs `XingZ60` correctly.
    That is true of `alignHeadings` and FALSE of the
    production path,
    which still merges source sections from the front and
    slides the document by two.
    `alignHeadings` is called by nothing.
    `doc/planning/wire-the-heading-aligner.md`.
-   `#65`,
    the unit precision is denominated in.
    570 of 2650 accepted issues
    share a span with another,
    but every duplicate pair shares one repair
    envelope,
    so the harm is counting rather than wasted work.
    Ranked C > B > A.
    `doc/planning/duplicate-accepted-issues.md`.
-   Naturalness lane reach.
    620 blocks of plain soft-wrapped prose are excluded
    by a `multi-line` check;
    admitting them would triple the lane's reach.
    Ranked B > C > A.
    `doc/planning/naturalness-lane-reach.md`.
-   The damage sheet still wants human grading,
    which `#66` and `#68` both wait
    on.

### The method, because it is the transferable part

`#71` was found because the artifact had recorded `alignment
 structure-mismatch` for weeks and nothing read it.
Treating that as a PATTERN
 rather than an incident is what produced everything else:
census every signal
 the deterministic core emits,
then check whether each is correct and whether
 anything consumes it.

That chain ran end to end.
The census found the footnote graph was wrong about
 10 of its 15 findings;
fixing it made the graph trustworthy;
trusting it
 revealed that 4 of 56 settled repairs had shipped broken footnotes;
those are
 now gated.
Each step was only possible because of the one before.

The second productive question was whether a built feature actually FIRES.
 Typography restoration was wired in and doing almost nothing.

### Defects found and fixed

-   Invisible-line masking had two holes:
    a line of non-ASCII space welds
    paragraphs exactly as a byte-order mark does and was not caught,
    and masking
    did not know about fenced code.
    `7b5dbf6b4`.
-   `---\r\n---` was refused outright,
    the one CRLF front-matter shape the
    earlier fix still missed,
    because its guard assumed a one-character
    terminator.
    `462b1690a`.
-   A closing fence was accepted on `trim()`,
    so ```` ```<U+FEFF> ```` read as a
    terminator.
    A bug in code committed an hour earlier,
    in the corrupting
    direction.
    `908e39285`.
-   The `#71` aligner slid every zero-evidence document by putting gaps at the
    FRONT,
    reproducing the defect it exists to remove.
    `XingZ60` hid it because
    three of its headings share names across the boundary.
    `110fc3909`.
-   The footnote graph walked the RAW tree while the node list walked the
    flattened one,
    so a definition inside a disclosure container was invisible
    and every `nodeId` after a container named the wrong block.
    `08d92eb41`.
-   Integrity did not notice broken footnotes,
    because it counted only MDX
    grammar downgrades.
    `e10ece178`.
-   Typography restoration learned its convention from the replaced REGION,
    which at a median of 75 characters usually holds no quote at all.
    `846f9ff6d`.
-   Naturalness eligibility counted a repaired parse as a degraded one,
    a
    regression introduced by the parse-finding work earlier the same night.
    `99e9b2c94`.

### Corrections made to earlier claims in this document and to the agent's own

-   The duplicate-issue cost claim was WRONG and is corrected in `af8abf895`.
    All 567 duplicate pairs share one repair envelope,
    so no model work is
    duplicated;
    the ranking reversed with the claim.
-   `#71`'s blast radius is one entry,
    but the first reasoning was wrong.
    SEVEN entries emit `structure-mismatch`;
    six pair by index anyway.
    The
    earlier check counted HEADINGS where the aligner compares CHUNKS.
-   A running pass cannot see a rebuild,
    so `pass10` was never the mixed
    population it was called at the time.

### Three verification traps hit and caught, worth knowing about

-   `git grep --extended-regexp '^\s*(\`\`\`|~~~)'` reported fence markers in
    nearly every file.
    Backslash-backtick is the buffer-start anchor in GNU
    regex.
    Fixed-string search shows the corpus has NO fenced code blocks.
-   `rg FAIL` over test output matched the NAME of a test containing "FAILS".
    Use `^\[error\]` or `AssertionError`.
-   A refinement-eligibility probe returned zero eligible for every input
    INCLUDING the clean control,
    because the fixtures were under the
    120-character minimum.
    The positive control is what caught it.

### Bounded and deliberately not built

-   CRLF documents get no invisible-line masking,
    blockquote-payload welds are
    not masked,
    and fence indentation is read from the line rather than the
    container.
    All three are real,
    all three are inert at this pin,
    and all
    three are recorded in
    `doc/troubleshooting/translation-repair-invisible-characters.md`.
-   An automatic slice-cache key over a hash of `src/` was considered and
    rejected:
    it would invalidate on comment and test changes,
    and a pass takes
    days.

## Overnight autonomous stretch, continued (05:00 to 05:20 UTC)

### What is running

`pass13`,
PID 1371047,
into
 `node_modules/.monochromatic/translation-repair-runs-pass13`,
running slice
 cache version 9.

Cache version is the quickest way to tell whether two passes are comparable,
 so for the record:
`pass10` ran version 5,
`pass12` ran version 7,
`pass13`
 runs version 9.
Artifacts from different versions answer different questions
 and should not be pooled.

`pass12` was stopped at 05:14 UTC with nothing settled.
The reasoning,
and the
 rule that a replacement pass must be confirmed working before its predecessor
 is signalled,
are in
 `doc/troubleshooting/translation-repair-run-invalidation.md`.

### What this stretch found

The method carried over unchanged:
census every signal the deterministic core
 emits,
check each is correct,
check something consumes it.
Applied this time
 to the full 56-entry artifact population rather than to one stage.

The finding:
quote anchoring silently discards 398 critic claims.
`locateQuote`
 fails,
`repair-stages.ts` pushes the reason and returns an empty array,
and
 the claim never reaches adjudication.
`quote-not-found` alone accounts for 225
 of those,
across 45 of 56 entries.

A mechanism explains part of it.
The corpus soft-wraps prose,
so a critic
 quoting across a wrap returns a space where the document holds a line break,
 and neither the byte-exact search nor the punctuation-normalized fallback can
 match that.
The full argument,
including the competing explanation that was
 ruled out first,
is in
 `doc/troubleshooting/translation-repair-unread-signals.md`.

### What was landed, and what was deliberately not

Landed (`a6bbeca50`):
telemetry only.
`quote-not-found` now names whether a
 soft-line-break collapse would have located the quote uniquely,
ambiguously,
 or not at all.
No claim changes fate.

NOT landed:
admitting those quotes.
That is blocked,
not deferred.
A repair
 anchored to a quote spanning a wrap replaces several lines with one,
which is
 the same line-structure question `doc/planning/naturalness-lane-reach.md`
 leaves for the user.
Landing the fix would decide it without asking.

So the fix waits on the same decision the naturalness lane waits on,
which
 raises that decision's value:
it now governs two changes rather than one.

### Corrections to earlier claims

-   A first pass at the duplicate-issue mechanism blamed dropped adjudication
    merge opinions.
    Refuted by measurement:
    36.8% duplicate with them,
    35.1%
    without.
    Recorded so the hypothesis is not raised again.
-   A ceiling argument ("43 dropped opinions cannot explain 956 duplicates")
    was drafted and dropped,
    because one dropped opinion can leave a
    multi-claim cluster unmerged and produce more than one duplicate.
    The
    empirical split carries the refutation without it.
-   `doc/troubleshooting/translation-repair-unread-signals.md` still claimed
    `footnoteGraph` was read by nothing.
    The chunk-integrity gate has read it
    since the footnote work earlier the same night.
    Corrected.

### A trap worth adding to the list

`readdir` on the slice cache returned zero `.json` files while `find` returned
 six,
because the cache sharded by entry id into subdirectories.
The empty
 result looked exactly like "nothing cached yet" and would have been reported
 as a starved pass.
`readdir` needs `{ recursive: true, }` there.

The general shape is the one already listed twice:
a filter that silently
 matches nothing reads identically to a genuine zero.

### What the next session should check first

Whether `pass13` has settled entries carrying `[line-break-collapsible]`
 suffixes,
and in what proportion.
That number is the incidence the whole
 investigation could not measure from existing artifacts,
because a claim
 discarded at anchoring never becomes a retained issue.

If the proportion is small,
the quote-anchoring lead is closed.
If it is large,
 it becomes an argument the user should hear when they settle the
 line-structure question.

### A second finding from the same census: the refiner goes silent

CORRECTED LATER THE SAME DAY,
and the correction closed it.
Read the paragraph
 below as the finding AS FIRST WRITTEN,
then the correction under it.

The naturalness refiner was a ONE-model stage on the population measured.
Across 129 invocations it heard nothing 34 times,
and the partition by entry is
 exact:
29 entries heard from it on every invocation,
7 heard from it on none,
 and none sat in between.
Independent per-call failure cannot produce that,
so
 the cause is a function of the entry rather than transport flakiness.

Those blocks were eligible,
were selected,
and were never rewritten,
so the
 lane's real reach on that population is below the 12.0% eligibility figure.

THE CORRECTION:
that population ran 2026-08-06 to 08-11,
and `eb21ffa6b` on
 08-12 took the lane from one refiner to three.
Quorum is now two,
so a single
 lost voice cannot empty the stage,
and `pass13` confirms it in practice:
every
 one of its `refine-candidates` findings reports `3/3 heard`,
with Kimi-K3
 losing the refiner voice and being retried back each time.

So the reach deficit is real for the OLD figures and gone going forward,
and
 the objection this paragraph originally raised against Option B,
that it would
 triple the eligible blocks against a single voice,
IS WITHDRAWN.
What misled
 the first reading was a stale comment in `run-config.ts`,
since corrected.
Recorded as `#73`,
closed.

The cause is not yet known,
and getting it needs no code change.
The warning in
 `attemptStageCall` names the reason but is discarded before it reaches any
 finding;
the historical logs are gone,
checked rather than assumed.
So
 `pass13`'s voice losses are being appended to
 `node_modules/.monochromatic/translation-repair-runs-pass13/voice-loss.log`,
 and because the failure is entry-determined `pass13` should reproduce it and
 name the cause.

Threading the reason into the finding was considered and deferred:
it needs a
 `StageVoice` union change,
cache version 10 and another restart,
and the log
 route answers the question without any of them.

Tracked as task #73;
the quote-anchoring work is task #72.

### Standing invariants for whoever picks this up

-   Every `*.unit.test.ts` imports `../dist/final/node/index.mjs`,
    so always
    run `buildAndTest`,
    never bare `test:unit`.
-   A running pass has a frozen module graph,
    so rebuilding cannot affect it,
    and a pass must be confirmed working before its predecessor is signalled.
-   Any pipeline-behaviour change bumps `SLICE_CACHE_VERSION` in the same
    commit.
    It has been missed once,
    on the very commit that added a gate.
-   Corpus content is unlicensed:
    artifacts,
    sheets and logs stay under
    `node_modules/.monochromatic/`,
    never in git,
    and grading sheets never go
    to a third-party model.

### A third finding: the aligner fix was never wired in

`alignHeadings` in `align-sections-order.ts` is a Needleman-Wunsch aligner that
 allows gaps on either side,
is covered by its own test file,
is exported from
 `index.ts`,
and carries the gap-placement fix from `110fc3909`.
Nothing calls it.
A search across the repository finds it only in its own
 definition,
its own tests,
and the re-export.

The pipeline aligns through `alignDocumentSections` in `chunk-document.ts`,
 which on a structure mismatch still merges adjacent source sections from the
 front and aligns the rest proportionally by character fraction.
Two merges
 shift the document by two,
which is the offset `#71` recorded.

Run against the current build,
`alignHeadings` produces the correct `XingZ60`
 pairing outright,
with affinity 1.00 on the three shared Latin names and the
 two gaps landing on the two sections that genuinely have no translation.
So the fix is wiring,
not new logic.

Blast radius re-measured on the current build,
which `#71` asked for:
7 of 92
 entries fall back to proportional,
but five of those have EQUAL chunk counts
 where proportional lands on index pairing anyway,
and `XIEPT2` was inspected
 pair by pair and is correct.
Only `XingZ60` is wrong,
confirming the earlier
 count by a different route.

NOT landed,
and the reason is not caution:
alignment feeds every later stage,
 and wiring it produces explicitly unpaired sections,
which is exactly the
 destination question `#70` leaves open.
Landing it would answer a question the
 user owns.
Options and ranking (B > C > A) are in
 `doc/planning/wire-the-heading-aligner.md`.

The general lesson is the one this handover already names:
a committed fix with
 a passing test suite is not evidence that the pipeline uses the code it fixed.
Asking whether a built feature FIRES caught typography restoration in the
 previous stretch and caught this in the current one.

## Every open decision, consolidated 2026-08-13

Supersedes the earlier "Four decisions waiting" list,
which is now incomplete.
Ordered by how much else is blocked behind each.

REWRITTEN at the end of the stretch rather than amended again.
Three
 amendments had stacked on this list and two of its items had gone false,
which
 is the same accretion this session diagnosed elsewhere.
What follows is the
 current state,
not a history of it.

### 1. Choose the pipeline shape: translate or repair (`#70`, blocks `#71`)

CORRECTED FRAMING.
An earlier version of this list called this "what an
 unpaired source section is for",
as though a destination policy had to be
 invented.
`doc/planning/translation-pipeline-redesign.md` already settles that
 it is not a separate decision:
the destination falls out of which pipeline
 shape is chosen,
and it offers three options ranked B > C > A.

-   B,
    translate every slice and select against the existing text.
    An unpaired
    section is then just a slice whose existing translation is empty.
-   C,
    fill coverage gaps first,
    then repair the completed draft.
    The coverage
    check fills it.
-   A,
    route barely-covered sections to a translate stage.
    The section is the
    sparse case the classifier routes.

The ranking's reasoning is that C rests on judging whether source content is
 represented anywhere,
the same judgement that already produced five false
 positives,
and a coverage check that misses is invisible.
B replaces detection
 with selection,
which is built and auditable.

FOUR PREREQUISITES are listed there,
and one is already on this board:
`#31`'s
 judge crosscheck,
deferred since milestone three,
because B stakes everything
 on judges preferring a good human translation over a fluent machine one and
 nothing has measured that.
The others are what replaces the introduced-defect
 differential,
what pairs 7 and 5 actually are,
and cost.

What this session ADDS is scale,
not framing:
there are three unpaired sections
 at this pin,
one of which is a seven-character `(To-Do)` placeholder needing no
 decision,
leaving TWO genuinely untranslated sections of 915 and 1459
 characters BOTH IN ONE ENTRY.
The 915-character one opens with an HTML
 disclosure block rather than prose.
So whatever shape is chosen,
the section
 content it has to handle today is small and markup-bearing.

Still the highest-value decision,
because the aligner fix cannot land without
 it.
`doc/planning/translation-pipeline-redesign.md` and
 `doc/planning/wire-the-heading-aligner.md`.

### 2. Naturalness lane reach: does changing line structure count as damage

Ranked B > C > A,
with two things that moved since the ranking was written.

The reliability objection to B is GONE:
`#73` closed,
because the refiner is a
 three-model stage since `eb21ffa6b` and `pass13` confirms every invocation at
 full roster.

The line-structure cost is now measured rather than abstract:
admitting the 620
 blocks removes 1329 line breaks and turns paragraphs of median 3 lines into
 single lines of median 212 characters,
p90 413,
max 1063.
That WEAKENS B over
 C,
because B's argument rests on the corpus wrapping inconsistently and inside
 these blocks it does not (median line 65,
p90 130).

It governs ONE change now,
not two.
`#72` closed,
so wrap-spanning quotes are
 no longer part of this question.
`doc/planning/naturalness-lane-reach.md`.

### 3. Whether the critic stage should retry to a full roster (from `#75`)

New,
and separable from everything else.
The editor and refiner retry lost
 voices until the roster is whole;
the critic retries only to quorum,
so a
 voice lost after quorum is met is never recovered.
At 7 settled entries that
 is 18.6% of critic invocations running one voice short,
permanently,
on the
 stage where claims are DETECTED.

Cost of fixing it,
measured:
about 3% more critic calls.
The exposure is
 measured;
the harm is not,
and measuring it needs a paired run.

### 4. Which unit precision is denominated in (`#65`)

Issue or envelope.
570 of 2650 accepted issues share a span with another,
but
 every duplicate pair shares one repair envelope,
so the harm is counting
 rather than wasted work.
Ranked C > B > A.

One half of it CANNOT be answered from existing data:
whether a duplicate is
 independent corroboration depends on the two copies coming from different
 critics,
and artifacts do not record which critic raised a claim.
`doc/planning/duplicate-accepted-issues.md`.

### 5. The damage sheet still wants human grading (`#66`, then `#68`)

`#68` is blocked behind `#66`:
which prober is right when they disagree cannot
 be settled from telemetry.

### Not decisions, just waiting on the running pass

-   `#72` is CLOSED with a verdict rather than a null.
    Soft wrapping explains
    about 3% of quote misses:
    1 of 33 wrap-explained,
    one-sided 95% ceiling
    13.5%,
    on a sample verified to wrap as much as the rest of the corpus (69%
    against 69%).
    At that rate the change recovers roughly 7 of the 225 misses
    in the settled population,
    which is not worth a behaviour change that also
    decides the line-structure question.

    It was first closed at 30 misses with NONE wrap-explained and a 9.5%
    ceiling.
    One appeared an hour later,
    which loosened the bound rather than
    tightening it.
    The decision did not move;
    the wording did,
    from "no effect"
    to "a small real effect".
    Full record in
    `doc/troubleshooting/translation-repair-unread-signals.md`.

    So the line-structure decision governs ONE change,
    the naturalness lane,
    rather than two.
    What stays open is the other 398 discarded claims,
    whose
    dominant cause is unknown.
-   `#73` is CLOSED,
    not waiting.
    It was already fixed by `eb21ffa6b` before it
    was found.
    `#75` replaces it,
    and needs no decision until a debug capture
    names which schema-mismatch sub-kind Kimi-K3 is hitting.

Neither needs anything from the user,
and both are now answered.
`#72` was
 unanswerable from the existing 56-entry population,
which is why `pass13` was
 restarted rather than left.
That restart paid for itself.
