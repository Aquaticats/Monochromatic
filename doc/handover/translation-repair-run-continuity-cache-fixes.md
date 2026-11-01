# Translation repair run continuity: cache fixes

Part of the [run continuity index](translation-repair-run-continuity.md).

## Current stop condition

Historical evidence only.
No continuity supervisor or corpus pass is running,
and this file does not authorize recreating one.

## The fix: the naturalness lane caches in its own namespace

LANDED 2026-08-22 as `fda817aaf`,
with tests in `0f781c687`.

### Why the per-stage cache and not the other option

The fork recorded earlier was whether the cached unit becomes the slice AFTER
 refinement,
or whether refinement gets a cache of its own keyed on the accuracy result.

The first option is not merely worse,
it does not work.
Folding refinement into the existing record means a resumed slice must skip the
 lane,
and the only marker available to skip on is `refined`.
`refine-phase.ts` set that flag on exactly one path,
where a rewrite both changed the text and kept every confirmed issue,
so it reads false both for a slice refinement declined and for a slice
 refinement never saw.
Skipping on it would still rebuy the lane at precisely the slices that flipped
 between the two runs,
which is where the divergence came from.

The second option also keeps a refinement re-askable without rebuying the
 accuracy pass,
which matters because the rewriter roster has churned before.

### What the key covers

`refine-slice-key.ts` keys on the slice source,
the accuracy text,
the declared names,
the filed issues whole rather than by identifier,
the confirmed subset,
the non-translation verdict,
and the rewriter,
judge and checker rosters.

It also keys on the DEFINITIONS of the whole assembled document.
Those are collected across every slice so a paragraph's references resolve
 while it is gated,
which means a neighbour settling differently changes what this rewriter is
 shown.
A key blind to that resumes a stale rewrite,
which is the failure `#126` already recorded once at the accuracy window.

The checkers are in the key even though they never rewrite anything,
because they decide whether a rewrite is rolled back for breaking a confirmed
 repair,
so a different checker roster ships wording this one refused.

### What a resumed slice does not carry

`askedRewriters` is re-derived rather than stored.
It says whether THIS run reached a rewriter,
and the driver reads it to decide whether a run overtaken by an abort may still
 call itself finished.
A slice resumed from disk asked nobody anything,
so carrying the stored answer would report a previous run's purchase as this
 one's.

### How it was verified

Three phase cases and twelve key cases,
all passing,
with the package at 504 passing tests and zero lint findings.

The phase cases COUNT MODEL CALLS rather than compare text.
The scripted client answers the same way every time by construction,
so identical text proves nothing about whether anything was bought.

Shown to fail per GFP.
With the resume removed and the package rebuilt,
the second run makes 8 calls instead of 0 and `askedRewriters` reads true
 instead of false.
The first run asserting it bought something is the positive control:
without it,
a second run buying nothing would prove only that the lane never ran.

### What is still owed

The band pair has not been re-run under the fix.
That is the user-boundary verification,
and it is the next thing to do:
two passes over the same six entries,
comparing published text slice by slice,
with the criterion this file now states exactly,
that a resumed slice must ship its cached text once blockquote markers are
 normalised.

## The capture poller reads directories only

The method that makes this verification possible nearly produced nothing,
and it would have failed silently.

`vub-cache-capture.mjs` walks its source directory,
skips every entry that is not itself a directory,
and copies the files one level down.
Pointed at `slice-cache`,
that is exactly right,
because each entry id is a directory and its cache files sit inside.
Pointed at `slice-cache/<id>`,
every entry it sees is a plain file,
so it skips all of them and copies nothing,
forever,
while still logging as a healthy process.

Run one of the pair was launched with the source pointed one level too deep.
The cache held eight files,
the capture held zero,
and the poller was alive the whole time.
Nothing in the poller reports the difference between
a directory that is empty
and a directory whose every entry it refuses.

Corrected mid-run,
with no loss,
because the poller keys what it has already copied by size and modification time
rather than by having watched it appear,
so a restart recaptures everything present.
All eight files were captured on the first poll after the restart,
which is the evidence that the walk now reaches them.

The markers came with them,
and that matters more than it looks.
`openNamespacedCache` reads each lane's marker
and discards the whole namespace when it is absent or does not match,
so a restore missing them would delete every restored file before anything resumed,
buy the lane again,
publish different text,
and present as this fix not working.
A capture that copies only the slice files is not a capture that can be restored.

The general shape,
which is the reusable part:
a poller that filters by a structural predicate reports the same silence
for nothing to do
and for everything filtered out.
A capture is not verified by the process still running.
It is verified by naming a file that must be in it,
and finding that file.

## The refinement resume verified at the user boundary

Run the same entry twice,
capture the first run's cache before it is discarded,
restore it,
and compare what the second run publishes.
`Zha_Ke` under pipeline `sha256-tree-v1:851f8020`.

The result.
Every published slice is byte-identical across the pair,
in both lanes,
four repair slices and four translate slices,
zero differing.
Before the refinement cache existed the same instrument found
7 of 18 repair-lane slices publishing different text on identical inputs.

The second run made 19 model calls against the first run's 301,
settled in 207 seconds against 4838,
and discarded no namespace.
Both runs recorded the same pipeline digest
even though `tip` moved,
because a documentation commit landed between them:
the digest reads the built directory rather than the commit,
which is the case `pipeline-digest.ts` was written for
and which this pair demonstrates rather than assumes.

The direct evidence for the lane under test is an absence.
The first run's log carries three `runRefineStage` decisions,
two rewrites that won on weight
and one panel tie that kept the repaired text.
The second run's log carries no refinement line at all.
The lane ran once,
was read back the second time,
and published the same words.

### The one slice that differs is the instrument rather than the pipeline

Consolidation slice 3 shipped 277 characters in the first run and 281 in the second.
That difference is the capture,
and the timestamps say so exactly.

The first run gated three consolidations,
at 10:10:01,
10:14:22 and 10:18:50.
The capture holds three consolidation records,
written at 06:10:01,
06:14:23 and 06:15:34 local,
so its newest predates the final gate by more than three minutes.
The artifact was written at 06:18:50,
and `discardSliceCache` runs immediately after it,
so the record for the last gated slice was created and deleted
inside one 200 millisecond poll.
The second run resumed the three it had
and bought the one it did not,
which is one purchase,
matching its single gate line.

This was scoped before the comparison was read,
not after it,
which is the only reason it can be called an artifact rather than a result.
A difference confined to a consolidation slice the capture demonstrably lost
is the measurement failing to record,
not the pipeline failing to resume.
Any other difference would have been a defect.

### A resumed run under-reports its own findings

Found by the same comparison,
and unrelated to the resume under test.

The first run records `alignmentFindings` twice and 34 repair findings.
The second records one and 33.
The missing entry is the same string in both places,
`block-pairing section 0`,
and it is missing because the pairing stage emits it when it BUYS
and does not persist it with the record it caches.

So a resumed artifact's findings list is not a faithful account of what the pipeline determined.
It is an account of what this particular run happened to pay for.
Anything reading findings to characterise an entry
reads a different answer depending on cache state,
which is the same class of defect as `#171`
with telemetry in place of published text.

## The refinement key now covers the incumbent

Landed 2026-08-22 as `f8d747f9c`,
the obligation `#172` left open.

### A field no model reads still belongs in a key

`refineSliceKey` covered the source,
the repaired text,
the definitions,
the declared names,
the issues,
the confirmed set,
the non-translation verdict and the roster.
It did not cover the archive wording.

That absence reads as correct on the first pass through the stage,
because nothing shown to a rewriter,
a judge or a checker carries the incumbent.
Every prompt in the lane is built from the source and the repaired text.

The stored RECORD is a different question from the prompt.
`refine-slice-settle.ts` sets `changed` by comparing its rewrite against the incumbent,
and drops `resolvedIssueIds` wherever the two match,
on the rule the accuracy stage already applies:
a resolution credited to text the document does not carry is a repair no reader saw.
So two runs over one source and one repaired text
but different archive wording settle differently,
and shared a key.

### The failure is a hard stop rather than a wasted purchase

A key too narrow usually costs correctness quietly.
This one does not.
`repair-refine-step.ts` asserts over every refined outcome,
resumed ones included,
that the stored `changed` agrees with the incumbent the current run computed.
There is no discard path.
A resumed slice carrying the other run's verdict throws,
and it throws on every later resume of that entry rather than once.

### Nothing pinned the two texts together

The obvious way to close this without touching the key
is to establish that no path yields a moved incumbent under an unchanged repaired text,
which would make the omission harmless.
That check was not run,
deliberately.
Even a true answer would be a coincidence of what other stages happen to do
rather than an invariant anything asserts,
and a later change to slicing or pairing would break it with no test failing.
`consolidate-key.ts` already covers the standing text for the same reason,
which makes this the in-repo precedent rather than a new idea.

### The phase hands over its resolved incumbent

The key is given the same variable the settlement compares against,
not a re-derivation of it.

`refine-phase.ts` computes the incumbent as the prepared slice's target text
falling back to the outcome's repaired text,
and the fallback fires wherever no prepared slice sits at an index.
A key that re-read the prepared slice itself would cover an absent incumbent
while the settlement below compared against the repaired text,
so the two would answer different questions
on exactly the path that has no archive wording to check.

### Shown to fail without it

The key-movement test was committed first,
then the pre-change `refineSliceKey` was restored over it and the package rebuilt.
Exactly one test failed,
the new one,
and the suite passed again once the field was put back.

No cache generation bump was needed.
The hashed array gains an element,
so no key written before this change can collide with one written after it,
and any source edit moves the pipeline digest regardless.

One consequence worth stating for the next session:
the `vub171` capture taken for `#172` is no longer restorable,
because its stored markers name a digest this change moved.
That is expected and costs nothing.

## The pairing cache now stores what the round reported

The pairing namespace held a bare list of correspondences.
A resumed entry makes no calls for a cached section,
so everything that section reported the first time was reported by nothing the
 second time:
the per-section pairing counts,
the `block-pairing section N fell back to scoring` notice,
and every voice-level finding the round produced.

The stored record now carries the findings beside the pairs,
matching `RefinedSliceSettlement`,
which is the shape that already got this right.
`isCachedPairing` refuses a bare array,
though nothing depends on that refusal:
the namespace is discarded whenever the stored generation differs from the
 running pipeline digest,
and editing these files moves that digest.

### The two gates were never the same gate

Three findings leave a section,
under three different conditions.
The voice-level findings arrive unconditionally.
The pairing-count line is filed where any voice was usable,
which is also what decides whether the round may be stored at all.
The fallback notice is filed where the roster agreed on nothing,
which is independent of both.

Collecting them into one list and storing it invites a regression that the
 resume test cannot see:
if the document's own list is fed only where the record is persisted,
a round nobody answered loses its findings on the COLD run,
and no cached section exists to notice.
The list is therefore fed on every path,
and only the persist stays gated.

### The fallback notice had to move

It was filed after the store wrote,
so storing the collected list would have stored everything except the one
 finding the comparison originally caught.
It is now filed before the write.
The warning beside it is emitted again on a resumed run rather than only the
 first time,
because keeping the deterministic aligner is what that run is doing,
not something that merely happened once.

### Roster reachability is stored on purpose

`block-pairing unusable (<model>: <message>)` names a call that failed.
Replayed off disk it describes a call the resumed run never made,
which is a fair objection and it is stored anyway.
The findings say what buying this pairing cost,
and a resume that dropped them would report a healthier roster than the one
 that produced the stored pairs.
`RefinedSliceSettlement` keeps its `refine-candidates (N/M heard)` line for the
 same reason.

### Shown to fail

The driver had no cache coverage at all before this,
so nothing would have noticed the findings going missing.
The new test runs the same document twice over one map and compares the two
 findings lists whole,
rather than sampling a string,
because a replay that keeps some findings and drops others is the shape this
 defect actually had.
It asserts the cold run said something first,
since two empty lists compare equal.
Removing the replay line and rebuilding fails that test and no other.
