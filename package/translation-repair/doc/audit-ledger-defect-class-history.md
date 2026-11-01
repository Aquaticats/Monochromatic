# Audit ledger: history of the numbered defect classes

The defect classes found run by run before the audit,
classes 1 to 185 and the ones before numbering.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## History, classes 93 to 185

Notes:
`~/temp/agent/audit-history-93-185/notes.md`
(families F1 to F19,
a per-class list,
probes `probe-siblings.mjs`,
`fixture-corpus.mjs`,
`guard-order.mjs`,
`verify-hashes.mjs`).
All 95 recorded guard commits precede their fixes;
of 221 cited hashes only `0e0bd1a8c` (named in `811d908a9`'s message) does not resolve.

### H1: a rejection of every valid proposal still stops the entry

Status:
fixed in `ee6d31e88`,
guarded in `37128bf6c` (owner,
2026-09-27,
"Preference + polish");
only a slate with no candidate or no voice heard still stops,
having nothing to ship.
`translate-runoff-tie.ts` breaks only a challenge round's tie;
any other decline throws,
and `consolidate-settle.ts` turns it into `ConsolidationStandingIneligibleError` over an ineligible standing
(the hulicaijia14 shape).

### H2: the lane contest winner is validated without `lineStructured`

Status:
fixed in `a0fd3cc7c`,
guarded in `b867b3180`.
`lane-contest-eligibility.ts` passes `declared` only;
with the class one hundred two fixture the floor says invalid and `laneContestChoiceVerdict` says it may ship.

What that cost,
read from the code rather than assumed:
the verdict gates only persistence and the log line in `lane-contest-driver.ts`,
and the consolidation's `readStandingVerdict` does pass `lineStructured`,
so the merged winner never shipped unrefused;
it was written to the slice cache and twin memo as warm-run evidence,
which is what the verdict exists to prevent.

The fix makes `lineStructured` required on `laneContestChoiceVerdict` and `laneContestChoiceMayShip`,
and threads `prepared.lineStructuredSliceIndices` through `runPassContest` and `contestDocumentLanes`.
Required rather than defaulted,
because the defect was an optional flag reading false where a caller left it off.
The guards are a verdict case and a driver case (merged winner of a governed slice not persisted;
the same winner persisted where no rule governs;
the line-keeping lane persisted where one does);
removing the flag from the validation call turned both red at their `mayShip` and `persisted` assertions.

The same family was checked across the package:
every other production `validateTranslatedSlice`,
`laneTextsForSlate`,
`repairInvalidCandidates` and
`buildTranslateMessages` caller passes `lineStructured`,
the two front-matter calls in `lane-contest-eligibility.ts` return before any line check,
and `corpus-run/translate-probe.ts`,
the translate-first prototype of `f87dda263` still runnable as the `translate-probe` task,
omitted it.
The probe now decides governance with the pipeline's own `governedSliceIndices` over its section and slices.
Replayed offline over its entry,
none of its three probed slices is governed,
so its sheet is unchanged there;
the positive control (a governed two-line slice) adds 882 characters of line rule to the sheet.
The probe calls providers and prints no sheet,
so no live run was made for this.

### H3: the 治愈 misquote

Status:
fixed in `357f534b7` (the 安慰 contrast) and `f6cf6679b`
(the reasons,
now given as the change's own);
the owner was told 2026-09-27.
The comment also called the reasons the owner's,
though the owner wrote only "I kinda disagree here".

### H4: the repair lane's own sheets never see the archive dispute

Status:
fixed (`d91b338d7`).
The editor and the checker read `ADDITION_IS_REMOVED_NOT_SOFTENED` (`addition-repair-rule.ts`):
an accepted addition is fixed only by removing the detail,
and a softer restatement keeps it,
which the checker answers not-fixed.
Class one hundred eight measured the repair lane softening an accepted invented event into milder wording.
Guard:
`addition-repair-rule.unit.test.ts`.

### H5: the translate lane and the consolidation never re-seat under a hold

Status:
fixed.
Translate half in `29a424b76` (guard `3d9079ea7`;
mutation checked with a control,
three mutants caught).
Consolidation half in `84aa1ce86` (red guards `01c8bbad8`,
further guards `2f8499b4d` and `81555e22e`):
`consolidationHooksFor` (`corpus-run/pass-consolidate-reseat.ts`) reads the seats while a hold runs,
builds the writers,
slate judges and naturalness roles `pass-consolidate.ts` builds,
and keeps them once the hold ends;
`consolidateDocument` seats each slice on that roster and keys it by it (`shapeFor`),
so a slice nobody re-seated keys as before and no cache version moves.
Mutation checked with a control:
eight H5 mutants caught
(the same runs caught three X11 mutants,
two of them on the self-certification fold M37 later reverted).
Three survived the first guards (the driver handing a re-seated slice the starting judges or polish roles,
and the run shape keying the starting polish),
because no judging or polish round ran under a client
that answered nothing usable;
a role-answering client with a no-hook control now proves those rounds are asked.
The lane contest,
the preparation and the picture readings have the same gap:
X12.

Found as:
`corpus-run/pass-reseat.ts` and `corpus-run/pass-consolidate.ts` only wait.
The fourth stage of one family:
classes one hundred three,
one hundred nine and one hundred thirteen
fixed re-seating for one repair stage at a time.
The per-slice hook now reads the seats while a hold runs for either lane and keeps each lane's latest roster;
the lanes driver passes a translate roster through,
and `translateDocument` seats the slice on it
and keys it by that roster (`shapeFor`),
so no cache version moves:
a slice nobody re-seated keys as before.
The consolidation half waits on a design:
`consolidateRunShape` folds the roster into every key once,
so per-slice seating there must fold each slice's bench into its key.
Measured 2026-09-28 (`~/temp/agent/audit-glossary-fix/h5-census.mjs`):
no hold began inside a consolidation phase
in any TianqiChen666 run (9 holds in 20 logs,
none inside),
against 983 inside in 25 of 4,121 logs overall.

### H6: the absolute naturalness review is shown wrapped prose and no house rules

Status:
fixed with S13 (`3e4cf0441`).
The subject takes a required `lineStructured`;
on prose the wire folds each paragraph for display only.
The subject's texts stay exact:
`absolute-naturalness-review-stage.ts` digests them
and `corpus-run/artifact-two-lane-read-naturalness-digest.ts` recomputes those digests from the shipped text,
so folding the subject would have broken every artifact read.

### H7: the consolidation writer and the translate judge see picture transcripts with no scope rule

Status:
fixed with S16 (`efa502d21`).

### H8: the introduced-defect probe has no declared names or glossary

Status:
fixed (`b6df6d5ee`).
The probe shows the declared names with their rules and the community renderings over each region's BEFORE and AFTER;
`runIntroducedDefectProbe` and `proveRepairedChunk` take a required `identityContext` string,
empty for none,
since the repo forbids nullish unions and an optional field would let a caller omit it silently.
Guard:
`probe-page-rules.unit.test.ts`.

### H9: observations left unbuilt after the owner said to fix everything

Status:
re-checked 2026-09-28 on XingZ6014's page (the latest XingZ60 page,
2026-09-26);
the one that stood is fixed in `b1dcdc00b`,
and the gap behind it is H16.
Every observation is XingZ60's.
同类 now reads "the same kind",
and 更多人 is not in the original:
the nearest clause (更多其他的相似点,
`page.md:311`) is rendered in full.
The album's title reads the same in its heading and its attribution.
Both 雨狸 signatures still give a bare-pinyin handle and one keeps 妄想症 in Han,
but today's floors refuse both renderings (`validateTranslatedSlice`:
Han residue and signer handle,
then signer handle),
so the next XingZ60 run cannot ship them.
螐儿 still took two treatments,
a bird in one slice and an unnamed crawling creature in another,
which no floor sees;
the rendering glossary now carries 螐 with its dictionary meaning
and a why saying the author's note carries the bird reading
(guard `rendering-glossary-rare-word.unit.test.ts`;
disabling the entry fails it,
the control survives).
Only XingZ60 writes the character,
so only its slices re-key.
The observations as first recorded:
更多人 omitted,
螐 in three treatments and the album in two forms,
"Yuli" with no literal gloss,
同类 narrowed.

### H10: stale statements against the code

Status:
fixed in `8c1681836`.
Both also missed that since class one hundred six a tie over an eligible standing every contest ballot
called flawed is run off,
that a run-off repeats while it narrows (class eighty-two),
and that a challenge declining with nothing left to narrow ships the judges' preference (owner,
2026-09-27);
the TSDoc was in `consolidate-settle-judge.ts`.
`doc/slice-context.md` (moved on 2026-10-18 to `package/translation-repair/doc/slice-context.md`) says a slate over an
eligible standing keeps its single round on a decline
and that a tie or rejection is re-asked once;
the TSDoc in `consolidate-settle.ts` says the same.

### H11: the address floor counts the 你 in 迷你

Status:
fixed with F-1 in `d0c788468` (`addressesNobody` in `translate-address-original.ts`,
guarded in `address-drop.unit.test.ts`).

### H12: minimax-m3 ignores OpenRouter endpoints one at a time and names none measured

Status:
fixed in `db1d285c6` (red guard `6f31a3479`):
the card sends `provider.order` Together then CoreWeave,
fallbacks allowed,
as the DeepSeek and GLM cards name theirs.
Measured 2026-09-28 (`~/temp/agent/audit-glossary-fix/h12-endpoints.mjs`,
pass-run logs only):
since the 2026-09-04 ignores the price sort sent 59,745 streams to CoreWeave,
9,177 to Together and 8 to Venice.
The listing of 2026-09-28 marks structured outputs on CoreWeave,
Together and ModelRun alone,
and OpenRouter routes a `response_format` request only to endpoints supporting it (provider-selection docs);
all three are FP4 (Together's own model table,
where OpenRouter lists it unknown),
and the fp8 endpoints,
the model's own among them,
list no structured outputs.
Over 2026-09-21 to 2026-09-28 Together served 7,601 streams at p50 1.9 s and p90 5.5 s,
18 not completed;
CoreWeave 37,870 at p50 2.7 s and p90 9.7 s,
99 not completed;
observed,
not controlled for load.
Precision being equal,
speed chose the order;
Together's listed price (0.30 and 1.20) is the card's.
The ignore comment said both ignored endpoints "cut a quarter or more of at least twenty streams";
`d55d83082` measured Parasail answering into the reasoning channel with content empty
and ModelRun timing out 119 of 300 streams in-stream,
and the comment now says so.
No cache version moves:
routing is in no key.
Mutation check:
emptying the named endpoints and swapping their order were each caught by four cases
across the catalog and client tests;
the control survived;
the full suite then ran green (0 of 1260 failing).

### H13: the seeds test pins the whole term list

Status:
fixed in `992d6c984` (properties and named renderings,
the owner's 药娘 and 治愈 included).
`community-glossary.unit.test.ts` went red three times on additions.

### H14: Freud's "id" becomes "ID"

Status:
fixed in `a17296fec` (guard `6527efaf2`),
with its neighbours in `1873b23dc` (guards `8ef3bdaec`,
`18c2bc00f`).
A text naming the ego,
the superego,
Freud or psychoanalysis keeps a bare "id" as written;
"id" inside a longer word ("idée",
with a separate accent too),
beside a digit or in a footnote label also stays.
A mutant dropping the Freudian hold fails the guard.

### H15: fixtures carry corpus text

Status:
fixed in `98054d72b` (every file this entry lists rewritten with invention).
`address-drop.unit.test.ts` (two fixtures),
`consolidation-polish-gate.unit.test.ts`,
`rendering-glossary-idiom.unit.test.ts`,
`suicide-drop.unit.test.ts`,
`community-glossary.unit.test.ts`,
`consolidate-gate-wire.unit.test.ts`.

### H16: a term repeated in slices judged apart has no shared rendering where the archive has none

Status:
fixed 2026-09-28;
found fixing H9.
Each slice is written and judged alone.
Where the archive renders a passage,
its English anchors every slice;
where it does not (a partial archive,
as XingZ60's poem),
a term the original repeats
reaches each slice's bench with nothing saying how the others rendered it.
The page-name glossary covers names and linked titles the archive pairs,
and the rendering glossary covers
dictionary words someone has entered;
a coined or rare term outside both can ship in as many forms as slices.
H9's entry closes the one instance read;
the general mechanism waits on a measurement of how often
a repeated term ships in more than one form.
Measured 2026-09-28 (`~/temp/agent/audit-glossary-fix/h16-count.mjs`,
`h16-terms.mjs`):
42 entries have a settled artifact,
and on each one's newest,
768 slices carry 75 with no archive English;
only XingZ60 (32 of 121) and shi_Yumiaoya (11 of 18) have two or more
(XIEPT2's 25 of 35 is a stub run of 2026-09-03).
The deterministic preparation cannot widen the count to all 92 pairs:
an unanchored slice is an insertion chunk,
made only after the roster's pairing and the insertion admission.
Over those two pages,
103 rare Han n-grams (on at most two pinned originals) recur across two or more unanchored slices;
read against the shipped text,
most are ordinary phrasing,
and the repeated terms are names,
titles
and a few community words,
most shipped in one form.
The divergences are concentrated in title-marked spans no archive anchors:
one of XingZ60's section titles ships in three forms (heading,
attribution and footnote,
one of them bare pinyin and one garbled),
another differs by an article between heading and attribution,
a pseudonym in brackets is kept in Han in one signature and rendered in the other (today's floors refuse the Han),
and a handle and a community's name vary in case (`z60` and `Z60`,
`limelight` and `Limelight`);
shi_Yumiaoya's 同居者 turns plural in the slice after the one that names a single person.
So the defect is real and bounded:
repeated titles and names on unanchored slices,
which the page-name glossary would cover if the archive paired them.
Bound over all 92 pairs (`h16-bound.mjs`):
8 pages carry 16 title-marked spans
(ATX or HTML heading text,
《…》,
【…】,
and 「…」 equal to one of those) on two or more places
that the page-name glossary does not pair;
XingZ60 carries 6 of them.
Decided 2026-09-28 for quality,
against no ruling:
the preparation settles one English rendering per such span once per page,
asking its roster
through the hook the attestation uses (X12),
cached,
and carries it as evidence lines after the page-name block;
it restores and enforces nothing,
since the judges keep deciding headings (owner,
2026-09-21,
`page-name-glossary.ts`).
Kept out of H16:
the Latin case drift,
which is X16,
and 同居者 turning plural,
one page's translation slip rather than a missing mechanism.
Built 2026-09-28,
pieces `ea61cbd4c` and `5df6fc632`,
red guard `31c7d3f00`,
wiring `d80f56866`:
`repeatedTitleSpans` (`page-title-spans.ts`) reads the spans off what the page shows,
listed by where each first stands
(`d9a306602`,
M44);
`settlePageTitles` (`page-title-lexicon-stage.ts`) asks the roster once under the house rules,
compares renderings without case,
spacing or wrapping quotes,
keeps the one most voices gave
(the earliest seat breaking a tie) and leaves out a title no voice rendered;
`passPageTitles` (`corpus-run/pass-page-titles.ts`) reads the hook only when the page repeats such a title,
and stores a heard round in its own registered namespace (`page-titles.`) under `PAGE_TITLE_CACHE_VERSION` 1,
keyed by the original,
the titles and the roster that answered;
a round nobody answered is not stored.
The preparation carries the lines after the corpus-name block,
and the round's finding on every way out (X18).
Mutation checked with a control:
34 of 37 mutants caught on the first run,
then the full suite (0 FAIL);
the three left (a nested HTML heading read whole,
the key dropping its version,
the key naming the starting roster)
had guards that could not fail,
and `5f64cbe91` makes each fail,
all three caught on the second run.
X16 corrects one line of the `h16-count.mjs` measurement:
`Z60` opens a quoted line,
and within a page each name-like token ships in one casing.

### Process mistakes, classes 93 to 185

- Passes ran a stale build (a nested `cp` copy),
    launched from uncommitted trees,
    or launched on builds whose full suite was red
    (hulicaijia30,
  shi_Yumiaoya38,
  TianqiChen66614,
  TianqiChen66618,
  TianqiChen6663,
  yingying9).
- A failing test was reported inside a `[PASS]` count and never fixed
    (the `lane-contest-stage` grace case,
  five times).
- A suite ran on a build other than the one committed (class one hundred fifty).
- `[PASS]` counts before 2026-09-26 counted describe blocks,
  not tests.
- A commit message names a hash that does not exist (`811d908a9` names `0e0bd1a8c`).
- Doc facts not taken from the source (launch times,
  counts,
  a stale handover line).
- Class one hundred forty-eight's fix has no planning-doc entry.
- Launch mistakes:
  a wrong entry id,
    a waiter watching the old pid,
    a build in the run's cgroup killed by systemd-oomd.
- Results claimed without a validated probe (the en_CA scan's false positives;
    class one hundred ten not replayed on real texts).
- Evidence misquoted to the owner and rulings misread (治愈's 安慰;
  class fifty-four's reading of "else fail").
- Shell rule slips in this session:
  a `;` in a suite command and in a test command;
    on 2026-09-28 two more,
  before a `PIPESTATUS` echo and between two `rg` probes;
    then a `;` between two `rg --count` probes of the built declarations,
    a heredoc that wrote a script followed by an unchained command that ran it,
    and a `;` before an `echo` closing a TianqiChen666 refusal count;
    after the crash that heredoc slip repeated during L11 (the edit script for `refine-slice-settle.ts`).
    Prevention:
  write a script with the Write tool and run it in its own call.
    During L5 the heredoc slip came back once more:
    a heredoc wrote the sheet-wiring script `l5-sheets.mjs` with an `echo` on the next line,
    after the prevention was already recorded;
  the script then ran in its own call.
    A prevention the same session repeats is not yet a habit:
    a Bash call that holds `<<` holds nothing else,
  and a script goes through the Write tool.
    After the restart,
  during L5's footnote fix,
  two more:
    a foreground `sleep 1 && true` as a placeholder while a mutation run finished,
    and `build > log 2>&1 ; tsc | rg` so the type check would run even if the build failed.
    Neither was needed:
  a background task notifies when it ends,
    and a build failure is itself the answer the type check would have given.
    Then the placeholder came back as `sleep 0 2>/dev/null ; true` after a doc edit,
  a sleep and a `;` in one call
    that did nothing at all.
    Prevention:
  a call that would do nothing is not made;
  while waiting on a background task,
  make no tool call.
    During P9 a `;` joined a lint count to the commit of the red guard,
    and the staging read only the count of expected type errors,
  not the full summary (read afterwards:
  those six only).
    During L14(a) three more:
    `build && run-files ; tail` put a `;` before reading the build log,
    a heredoc edit script was followed in the same call by `sed ... ; rg`,
    and the runner's exit status was trusted as a test verdict (`run-files.ts` exits 0 on failing files;
    its per-file `exit=` lines are the verdict).
    The heredoc repeat came after the rule "a Bash call that holds `<<` holds nothing else" was recorded,
    so the prevention stays the same and is now applied without exception:
    edit scripts go through the Write tool and run alone.
    During L12 three more:
    a call chained four `&&` steps where three is the limit;
    a `git add && git commit` ran from the package directory,
  which cli-git's `require-root` guard refused
    (git commands run from the repository root or with `git -C`);
    and a two-line test fix went through `python3 -c` in the call rather than a script written with the Write tool.
    Prevention:
  count the `&&` before sending,
    start every git call with `cd -- <repo root>` or `git -C`,
    and treat an inline `-c` script as the heredoc slip in another form.
    During P1 the do-nothing placeholder came back as a background `sleep 1` while a mutation run finished.
- A census read logs it should not have (QIV):
  the first P7 card ratios and P1 counts read every `.log`
    under the agent directory,
  where unit-test suites and prototype test logs carry fixture `SPEND` and stream lines;
    mimo's median moved from 126 to 93 once kept to pass-run logs.
    Prevention:
  a log census keeps logs that open with `START tip=` and says so in its header.
- The mutation harness (`mutants.ts`) counted only failures that said `AssertionError`,
    so a mutant that made a test throw anything else read as a survivor (P1's no-usage mutant,
  a `TypeError`).
    Survivals reported before 2026-09-28 may be false;
  catches were not.
  It now counts every failing test line.
- The Write tool also strips trailing spaces,
  so an edit script's literal holding a TSDoc blank line
    that carries spaces never matches the file;
  the scripts build those lines in code.
- The Write tool decodes `\u2028` and `\u2029` in the content it is given into the raw characters
    (it left `\u0085` as text),
  so a source file written with those escapes held raw line separators,
    which `no-multi-str` flagged in `sheet-line-text.ts` and which reached `sheet-line-text.unit.test.ts` unflagged.
    Found by reading the bytes;
  fixed by a Python pass writing the escape text.
    Prevention:
  a file whose source must spell a line separator as an escape is written or patched by a script
    that builds the escape from its parts,
  then checked with `rg` for the raw characters,
    against a positive control that proves the search can match one.
- A capped search taken as complete (QRY):
  before `7ceffe055` changed two card prices,
    the search for tests pinning the old ones ran through `head --lines=10`,
    which cut off `deepseek-v41-admission.unit.test.ts`;
  the full suite caught it and `bc69e2336` fixed it.
    Prevention:
  a search whose empty or short result licenses a change runs uncapped.

## History, classes 1 to 92 and before numbering

Notes:
`~/temp/agent/audit-history-1-92/`
(`notes-classes-01-44.md`,
`notes-classes-45-76.md`,
`verify-hashes.out`:
213 cited hashes,
none missing).
Families found there:
narrow fix leaving siblings,
a gate or tie stopping the entry over a valid proposal,
two readers of one text applying different rules,
sheet text contradicting another rule or the code,
a rule's sample wording steering output,
a check placed after the last decision (publish-only),
a decision made without evidence the pipeline holds,
silence counted as a vote,
an enumerated list where a general rule was needed,
a fallback hiding a failure,
missing log context,
slice-by-slice judging blind to page consistency,
a fix introducing a regression,
provider replies misclassified.

### E1: destination loss is caught only at publish, and the refusal names neither destination nor slice

Status:
fixed.
The diagnostic half in `7668c6ce7`;
the remedy in `cc6762f46` (red guard `d8009bfb1`),
following the owner's ruling of 2026-09-27 that a settled page ships with its defects reported.
Every publish-time content check (front matter,
archive original,
contributor names,
destinations,
headings)
now runs through `corpus-run/publish-defects.ts`:
its own refusal becomes a defect that is logged and printed as a `DEFECTS` line beside the tally,
anything else it throws still stops the entry,
and the page ships.
A page that does not parse still refuses.
A page that disagrees with its artifact refused too until the owner ruled the same day that it ships reported
(`page-agreement`,
fixed in `e5bd9a95b`,
red guard `87aa58047`).

Found as:
`corpus-run/publish-fixed.ts` threw after all spending;
no page-assembly pass names a destination;
`corpus-run/destinations-line.ts` and `corpus-run/pass-entry.ts` claim the addresses are in the run log,
false on the refusal path.
XingZ6011 lost 96.3 min to it on 2026-09-26.
The refusal now logs each dropped address with the slices whose original,
archive span and shipped text carry it,
and its message names the source slices.

Measured on 2026-09-27 over the 246 stored artifacts whose carve reproduces,
461 source destinations inside a slice:
302 carried by both the archive span and the shipped text,
151 missing from the archive span and restored by the shipped text,
5 missing from both and dropped by the page,
all from August runs before the either-rendering rule of 2026-09-04
(wangzihao980 slices 3 and 4 written by a lane,
Toka_ls slice 13 left to the archive),
3 missing from both and carried elsewhere on the page,
and none carried by the archive and lost by the shipped text.
So the live failure is an archive sentence without the link shipping,
and a take-back to the archive would never have helped.
Refused entries leave no artifact,
so they are counted from run logs:
seven real entries were refused at publish for one dropped destination each
(luxuanwen3,
Mio,
shi_Yumiaoya twice,
XingZ60 three times),
after 53 to 136 minutes each,
about 10.3 hours in all.

### E2: the reader-protection bullet prescribes "she ended her life" for deaths and attempts alike

Status:
fixed in `e8f0b0369`.
The bullet opens on "a death or an attempt" and its sample says "the page says that she ended her life",
the class seventy-nine shape;
class one hundred twenty-four's later sentence contradicts it.

### E3: stages running their own rounds count unreachable seats in their quorum

Status:
fixed with X8 in `29baade8f` (prep `a107c7486`,
red guards `a6cbd8fc5`,
legacy fixture `78d6540f8`),
recorded as the second 2026-09-27 addendum of `doc/decision/translation-repair-short-bench-share.md` at the repo root.
The review now decides on the reachable share and records `unreachable` in its round;
the reader recomputes the verdict from it and reads a round without it as none out of reach.
The agreement census over the 214 stored pages was unchanged by the reader change (137 agree,
77 disagree,
0 refused).

Found as:
`absolute-naturalness-review-stage.ts` sizes on `modelIds.length`,
so a false "quorum not met" skips the confirmation;
`pair-blocks-stage.ts` and `pair-sections-stage.ts` time their grace the same way.
`reachableQuorum` exists (`stage-reachable-quorum.ts`).
Measured on 2026-09-27 over the 265 stored artifacts:
59 of 4567 naturalness reviews closed `quorum-not-met`,
every one with 2 to 4 usable seats,
an upper bound on the harm,
since a review record cannot tell a seat the router refused from one lost in transport.
XingZ624 shows the shape live:
a review of 5 seats taken over a bench of 8 (a confirmation asks only the discovery's seats)
closed 3 of 5 usable against a quorum of 4,
with `glm-5.3` and Qwen3.8-27B refused as out of budget during it.
The lane contest and both gates decide on two ballots,
so there the bench quorum only drives how long rounds chase seats;
the review alone decides on it,
and its reader recomputes the verdict from `quorumOver`,
so a fix changes the record and the reader together.

### E4: an empty standing with no valid lane text fails at publish, not at once

Status:
fixed,
in two steps.
First in `d9cb910e4` (red guard `d2f0e07ef`),
which made the slice fail at once under the owner's rule of 2026-09-04;
then the owner ruled the same day that such a slice keeps the archive and the page ships
("Keep archive,
ship",
`doc/design-commitments.md` (moved on 2026-10-18 to `package/translation-repair/doc/design-commitments.md`)),
fixed in `7f79ada48`,
`002f21f43` and `e7d409fdd` (red guards `4acd8bd40`),
recorded as the eighteenth addendum of `doc/decision/translation-repair-ineligible-standing.md` at the repo root.

Found as:
the `no-standing-text` exit in `consolidate-settle.ts` was the one exit keeping the standing
that skipped `requireShippableTerminal`,
so a slice with no valid proposal anywhere settled with no polish,
and `assertFinalNaturalnessComplete` stopped the entry at persist
after every later slice had been bought.
XingZ60 lost about 3.0 hours that way on 2026-09-22 (slice 89),
on the variant with a lane text on offer that class eighty-seven fixed.
An empty standing passes the rule only over a blank original,
which the pinned corpus never carves (0 of 1259 deterministic slices,
2026-09-27),
and such a slice still keeps nothing.
Since the naturalness check of 2026-08-28,
no stored artifact carries a `no-standing-text` slice;
all 15 that do are from August.

The first step also fixed a vacuous check beside it:
the `incumbent-only` case read its served counter off a return a throw never delivers,
so its "no judge bought" assertion held whatever was bought.

### E5: floor refusals and panel votes the log cannot explain

Status:
fixed.
The reason per panel verdict in `f2cd70ece` (red guard `413a441a0`),
after the owner chose "Reason before vote":
the sheet asks for the reason first and shows it first in the reply shape,
the schema requires it and declares it before the vote,
a missing or blank reason is the finding `missing-reason (n)` while the vote still counts,
every stored panel ballot keeps it,
and the log has one line per ballot beside the issue's decision.
Repair slice cache version 33,
since the sheet is not in the run shape;
no slice had been cached under 32.
Floors and the repair turn in `d69a456de` (red guard `58b993f81`):
the translate floor warns per withheld candidate with the rule's reason,
the consolidation floor names proposals withheld beside survivors (it logged only the all-refused case),
and `repairOneCandidate` logs every branch that changes a candidate's fate,
where it had logged only a taken revision.
The panel in `b8aea8ecf` (red guard `f12a29a3d`,
lint repair `f83c4a6b6`,
see M23):
one line per issue with its status,
severity and every member claim's weights,
where the stage logged only its packet count;
a claim with no tally says so rather than reading as zeros.
Who voted what was already in the artifact (`AdjudicatedIssue.readings`).

Found as:
`translate-produce.ts` returns floor refusals as findings with no log line;
a panel verdict carries no reason.
The second half is a wire change (`PanelVerdictWire` has claim,
vote and severity only),
measured on 2026-09-27 over the 265 stored artifacts:
about 783 panel verdicts per entry (207,394 ballots),
and the reasons other stages' ballots carry average 167 characters.
A reason placed before the vote can move votes,
and no panel bench exists to measure that:
the last panel-sheet change was judged by paired entry runs,
one run per arm
(`doc/audit/the-damage-no-instrument-was-catching.md` at the repo root).

### E6: a malformed archive competes in repair selection as though it parsed

Status:
not a defect in behaviour;
the docs that claimed one are corrected,
and the equality is pinned in `c62aa9ac8`.
`repair-chunk-verdict.ts` passes `UNCHANGED_MEASUREMENTS` with `integrityOk` true,
and that is the honest measurement:
`measurePatchedCandidate` asks whether a candidate is no worse than the archive on grammar downgrades and broken footnotes,
and the repair lane's baseline is the archive itself (`repair-chunk-evidence.ts`),
so the archive against itself is intact however it parses.
The claim came from `selectRepairCandidate`'s TSDoc (`2e2694680`),
which read `integrityOk` as absolute,
as its own field doc did ("still parses",
"front matter intact",
which nothing measures).
The guard in `chunk-measure.unit.test.ts` fails when integrity is made absolute (mutation checked).
A patch that repairs an archive's structure gains no integrity rank for it;
it gains rank only through an accepted issue the checkers find resolved,
which is the lexicographic order as settled.
See M22.

### E7: the sheet-leak label list has fallen behind the sheets

Status:
fixed with F-8;
a fenced block no longer depends on the list.
`translate-sheet-leak.ts` lists seven labels;
`REJECTED CANDIDATE N`,
`WHAT THE JUDGES FOUND`
and `PRIOR FAILED CONSOLIDATION STRATEGY` are fenced but not listed.

### E8: more fixtures paraphrase corpus content

Status:
fixed in `98054d72b` (every file this entry lists rewritten with invention).
`archive-footnote-relabel.unit.test.ts`,
`pair-definition-order.unit.test.ts`,
`coverage-verdict.unit.test.ts` (names from the corpus).

### E9: tests pin roster sizes to literals

Status:
fixed in `87f95d62f` (no provider spelling for two roster models;
the contract's checker floor).
`roster-reach.unit.test.ts`,
`corpus-run/owner-cull.unit.test.ts`.

### E10: the page-name glossary reads raw documents

Status:
fixed in `7a434cb1b` (guard `5640c50e7`).
`page-name-glossary.ts` does no front-matter split and no comment or code-fence masking.
Measured over the 92 pinned entries before the fix:
5 read differently.
Four gained a pair only from an editor's comment or the front matter (two contributor links,
two note lines read as headings),
and windward0032 lost three real heading pairs because a heading-like line inside a comment broke the count alignment.
The glossary now reads `visibleText` (`src/page-visible-text.ts` (moved on 2026-10-18 to
`package/translation-repair/rendering-audit/src/page-visible-text.ts`)):
the body with HTML comments,
JSX comments and fenced code blanked.
After it,
raw and masked readings agree on all 92.
Page-name lines are identity context,
which every stage that shows them hashes,
so only those entries re-key.
A mutant reading the raw text fails the guard.

### E11: the retry-wait parser is case-sensitive and knows only h, m and s

Status:
fixed in `7a285b2f2` (guard `041e0a0d4`);
latent.
Every provider refusal in the run logs is Hyper's lower-case h/m/s form.
The parser read the first letter after a number as the unit,
so a glued "500ms" would have read as 500 minutes,
and an existing case pinned "12 minutes" as no wait.
Units are now whole letter runs (ms,
s,
m,
h and their words),
with decimals and joined parts,
in `src/retry-stated-wait.ts` (moved on 2026-10-18 to `package/translation-repair/provider/src/retry-stated-wait.ts`).
A mutant dropping case folding fails the guard.

### E12: the English-original recognizer is a list of exact wordings

Status:
fixed in `f91f561b9` (guard `3d028c76b`);
latent.
The module comment promised that every note read is logged with its reading;
no code logged one (the M20 family).
A note naming an original and English in a wording no mark knows now reads `unmarked-original-claim`,
seals nothing,
since a guessed seal could decline a whole page,
and is warned about by the pass
(`entryArchiveOriginalOf`,
which logs every note's reading).
Over the 92 pinned entries every decision is unchanged,
61 notes are logged,
and the one unmarked claim is hakureico's quotes note,
which names no span and still seals nothing.
A mutant that never reads the claim fails the guard.

### Process mistakes, classes 1 to 92 and before

- A suite called green when red (`e7307d304` for `d70087757`) and a miscounted `[PASS]` total.
- Runs launched on red builds (CuspariaKLSY3 on a real defect;
  others red on fixtures),
    on stale builds (two green runs that never executed the new code),
    or with the wrong roster or entry id.
- Results claimed without reading the output (the owner:
  "You didn't even look at its actual output.")
    and reads that declared no defect where one stood.
- Early fixes committed their tests inside the fix;
    red-first commits start with class five.
- A test file in the wrong shape registered nothing and exited 0.
- A mechanical lint autofix changed behaviour (`92c5192ee`).
- A question put to the owner whose example contradicted its label (class eighty-three).
- Observations recorded and not fixed that later became classes
    (84,
  85,
  75 and 76,
  79,
  67 and 68,
  71,
  106).
