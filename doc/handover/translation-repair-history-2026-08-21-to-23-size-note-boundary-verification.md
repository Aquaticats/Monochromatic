# Translation repair history 2026-08-21 to 2026-08-23: the size note's boundary verification

The far-longer precondition that did not reproduce after the pairing change,
the corpus-wide tail census with its paired same-entry comparisons,
the conclusion that overreached and was corrected later the same day,
and the far-shorter case that reproduced and exposed the settled-neither fallback.
The correction its verdict says is made later is recorded in `translation-repair-history-2026-08-21-to-23-far-longer-tail-and-the-note-floor.md`.

Part of [Translation repair history: 2026-08-21 to 2026-08-23](translation-repair-history-2026-08-21-to-23.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## 2026-08-23: #157 removed the far-longer case #163's note was built to catch

OVERREACHED,
corrected later the same day.
The measurement in this section is sound;
the conclusion it licensed was not.
See "2026-08-23:
the far-longer tail is alive,
and the note's own floor is why nobody saw it".

`dogesir_` settled at 00:58:55Z in the `#163` boundary run.
The far-longer
precondition DID NOT REPRODUCE,
and the cause is upstream rather than random.

WHAT CHANGED,
with the control run first because the obvious alternative is a
corpus change:

-   `corpusSha` is identical across the two runs,
    `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`.
    Same archive,
    same source.
-   Sliced SOURCE characters are identical,
    1196 in both.
    The Chinese side is
    sliced the same way.
-   Sliced ARCHIVE characters fell from 3475 to 1818.
    The whole archive is 3716
    characters,
    so 1898 of them now sit OUTSIDE the paired region.

So the English side lost 1657 characters from the sliced set while the Chinese
side did not move at all.
That is `#157`,
whose declined blocks now leave the
slice.

THE CONSEQUENCE FOR `#163`.
Slice 3 still carries the same 114-character
Chinese,
but its archive rendering is now 226 characters,
1.98 times,
where the
08-18 run had 1766 characters at 15.49 times.
The ratio tail is gone,
the note
stays silent,
and it is CORRECT that it stays silent:
there is no longer a
disproportion to report.

This narrows an earlier reading without overturning it.
`contest-size-note.ts`
records that both far-longer trips were page-only content rather than
displacement,
established by profiling the document and finding no donor slice.
That holds.
What it did not say is that the surplus was English the pairing had
no business pulling into the slice at all,
so the right fix was never to have a
judge reason about it.
`#157` does that at the pairing.

ALSO SEEN,
and it tightens a margin `#163` already called thin:
at slice 2 the
TRANSLATE lane produced 2170 characters against a 232-character source,
9.35
times.
That is a PRODUCED candidate closer to the 10 endpoint than the 9.27 the
earlier census recorded as its maximum.
It carries `block-count-gap`,
which is
excluded from the ratio-tail reasons,
so no note fired.
The judges caught it
without one:
five of six chose repair and four marked translate `unsupported`.

WHAT IS STILL DECIDABLE.
`wangzihao980` slice 3 is the FAR-SHORTER case,
102
Chinese characters against 66,
and `#157` cuts in the opposite direction there:
removing unpaired English makes a slice's English side SHORTER,
which can only
make a far-shorter tail more reachable,
not less.
That entry has not started
yet and is now the whole of the remaining verification.

Nine of the ten slices reached the lane contest.
Slice 5,
at 276 characters
against 244,
was not contested and appears in `comparison` but not in
`laneSelection`.

### Corpus-wide tail census: the extreme archive ratios are gone, in BOTH directions

Run while waiting on `wangzihao980`,
over every settled artifact on disk
(`${HOME}/temp/agent/163-tail-census.mjs`,
ids and ratios only).
It reports
ratio tails on the ARCHIVE side per artifact,
dated by the artifact's own
timestamp.

THE AGGREGATE IS CONFOUNDED AND IS NOT THE EVIDENCE.
43 artifacts,
10
far-longer and 5 far-shorter overall;
15 settled on 2026-08-22 or later with
zero of either.
But the recent set is mostly two-to-four-slice entries settled
by verification runs,
while the older set holds the long ones,
so entry size
alone could produce that split.

THE PAIRED SAME-ENTRY COMPARISONS ARE THE EVIDENCE.
Three entries settled both
before and after the pairing work,
and every one lost its tail:

-   `Zha_Ke`:
    maximum archive ratio 94.51 to 5.15,
    far-longer 1 to 0.
-   `dogesir_`:
    15.49 to 3.40,
    far-longer 1 to 0.
-   `lintong`:
    7.98 to 3.75,
    far-SHORTER 1 to 0,
    slice count 5 to 3.

A CORRECTION TO WHAT THIS FILE SAID EARLIER TODAY.
The `dogesir_` section
reasons that `#157` cuts TOWARD the far-shorter tail,
since removing unpaired
English shortens a slice's English side.
`lintong` refutes that:
its
far-shorter case cleared as well.
The pairing work is not directional in the
way that argument assumed,
because it changes the slicing itself rather than
only trimming one side,
so a ratio can move either way.
Treat the far-shorter
prediction for `wangzihao980` as OPEN,
not as favoured.

ATTRIBUTION IS LIMITED TO WHAT WAS MEASURED.
The census shows the effect,
not
which commit caused it:
`#131`,
`#157` and `#159` all landed in that window.
Only `dogesir_` carries a specific mechanism,
and it is `#157`'s signature:
sliced archive characters fell 3475 to 1818 while sliced source characters held
at exactly 1196.

WHAT THIS MEANS FOR THE SIZE NOTE.
Its far-longer half now serves no case that
still occurs in anything settled here,
and its far-shorter half has lost its
only demonstrated instance outside `wangzihao980`,
which has not settled yet.
Do not act on that until it does.
The note costs nothing when silent,
so there
is no urgency in removing it,
and the question is whether it earns the policy
text it adds to every contest message.

## 2026-08-23: the boundary verification's answer, and the gap it found

`wangzihao980` settled and the pass exited 0.
The far-shorter precondition
REPRODUCED,
twice,
so this half of `#163` is a real verification rather than a
vacant one.

-   Slice 3:
    Chinese 102,
    archive 66,
    0.65 times,
    `target-far-shorter`,
    note FIRED.
    Candidates 186 and 182.
-   Slice 4:
    Chinese 141,
    archive 88,
    0.62 times,
    `target-far-shorter`,
    note FIRED.
    Candidates 167 and 171.
    In the 08-18 run slice 4 was the 8.71
    times FAR-LONGER case;
    under the current pairing it is far-shorter.

Two of five contested slices fired,
against the 1.7 percent the design measured
over the older corpus.
The rate on this entry is not the corpus rate,
but it is
worth knowing that a single entry can carry two.

THE NOTE REACHES THE JUDGES AND THEY ACT ON IT,
at slice 4:
translate won four
votes to one,
two judges marked repair `unsupported` or `dropped`,
the
consolidation gate was ASKED,
and the fuller text shipped.

AT SLICE 3 THE UNDER-RENDERING SHIPPED ANYWAY,
and the reason is structural
rather than a judge ignoring evidence:

    laneSelection  settled-neither
    consolidation  terminal=no-standing-text, gate=not-asked, shipped=unchanged

No lane won,
so there is no standing text,
so the gate is never asked and the
archive stands.
The archive is the 66-character rendering the note flagged.
The
judges did NOT endorse it:
two of six marked BOTH candidates as `dropped`,
which
is a rejection of the candidates,
not an endorsement of the incumbent.
The
fallback simply happens to be the shortest of the three.

SO THE GAP IS THIS.
`settled-neither` means "neither candidate is better",
and the pipeline reads it as "keep the archive".
At a slice where the archive is
measurably under-rendering,
those are different claims,
and nothing currently
separates them.
The size note is the only thing in the system that knows the
incumbent is short,
and it is shown to judges who cannot vote for or against the
incumbent at all:
`LaneChoice` is `repair`,
`translate` or `neither`,
and the
archive is not a candidate.

That is worth stating plainly,
because it also explains why the far-shorter half
of `SIZE_NOTE_POLICY` cannot do what it says.
It instructs a judge to "put the
DROPPED question to that rendering in particular",
and at slice 3 that rendering
is the one the ballot has no way to name.

WHAT THIS DOES NOT SHOW.
Nothing here says the archive at slice 3 is wrong,
only
that it is short and that no candidate beat it.
Deciding it is wrong needs the
passage read,
which is a separate act.
The finding is that the mechanism cannot
currently distinguish the two cases.

VERDICT FOR `#163`:
the estimator and the note are LANDED and the note is
verified to reach judges and to move an outcome where a lane wins.
The
far-longer half now serves no case that still occurs (WRONG,
corrected below:
it serves a case at 185x that the note's floor hides).
The far-shorter half is
demonstrated live,
and it exposed the `settled-neither` fallback rather than
being consumed by it.
