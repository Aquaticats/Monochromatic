# Translation repair history 2026-08-21 to 2026-08-23: the far-longer tail and the note's own floor

How a runaway emission of 185 times its source surfaced through the repetition check,
the source-length floor that hid it from the judges,
the census of silenced candidates,
what the earlier conclusion got wrong,
and the floor fix with the reason no cache version bump was needed.

Part of [Translation repair history: 2026-08-21 to 2026-08-23](translation-repair-history-2026-08-21-to-23.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## 2026-08-23: the far-longer tail is alive, and the note's own floor is why nobody saw it

This corrects two claims recorded earlier the same day,
in "2026-08-23:
#157 removed the far-longer case #163's note was built to catch"
and in the `#163` verdict beneath it.
Both said the far-longer half of the size note serves no case that still occurs.
It serves one at 185 times,
in an artifact settled after `#157` landed.

### How it surfaced, which is the part worth keeping

Checking whether `#68`'s document-scale repetition check existed turned up `src/assembly-repetition.ts`,
already built,
already wired into both assemblers,
already measured.
Reading what it had FOUND rather than whether it existed
is what led here.

`vub171-20260822`'s `Zha_Ke` artifact carries 866 `introduced-repetition` findings.
The token shape is the diagnosis:
all 866 are exactly 12 words,
archiveCount 0,
shippedCount 2.
Twelve is `MAX_PHRASE_WORDS`,
so that is one long verbatim span duplicated in a lane's assembled document,
reported once per 12-word window.

In the artifact,
`comparison` chunk 2 holds incumbent 56 characters,
source 56 characters,
and a `translateText` of 10381.
The translate lane ran away on a 56-character slice and emitted 185 times its length,
looping.

The runaway did NOT ship.
That slice settled `slate-kept-standing`,
`gate=not-asked`,
`shipped=unchanged`,
so the archive's 56 characters stood.
The instrument caught it and the downstream guard held.

### The defect

`contest-size-note.ts:162` returns the empty string when the source is under
`MIN_RATIO_SOURCE_CHARS`,
which is 80 (`displacement-ratio.ts:97`),
BEFORE it looks at any candidate.
So the judges deciding that slice were never told a candidate was 185 times its source.

`sliceImplausibility` is not floored:
it returns `target-far-longer` at 56 against 10381.
The suppression exists only in the note the judges read.

The floor's stated justification is that "a ratio over a twenty-character line reports rounding".
That holds for `target-far-shorter`.
It does not hold for `target-far-longer`,
where 56 against 10381 is a runaway,
not rounding.
One floor is applied symmetrically to a rule that needs it asymmetrically.

### Measured over every settled artifact

Instrument:
`~/temp/agent/182-floor-census.mjs`,
ids and ratios only.

    far-longer candidates across every settled artifact   40
    the note TOLD judges about                             3
    the 80-char floor SILENCED                            37

    distinct slices silenced                              11
    distinct entries                 saurikissa, Zha_Ke, zheermao101

The floor inverts the instrument.
Silenced ratios include 185.4,
137.0,
100.8,
99.1,
94.5,
69.7,
23.8 and 19.2.
The three it does report are 15.5,
15.5 and 10.1,
the three mildest in the population.
The note is quiet exactly where the evidence is strongest.

### What the earlier conclusion actually got wrong

The earlier census was not wrong about what it measured.
It compared the ARCHIVE side against the source,
grouped by artifact date,
and its numbers stand.
It was wrong about what those numbers licensed concluding:
one side of a comparison does not settle a question about both sides.
The far-longer tail had moved to the candidate side,
where that census never looked.

`#182` carries the fix shape,
its measured cost,
and the two questions still open before building.

### The floor fix, landed and verified against the shipped function

`cf1c547c7`.
`contest-size-note.ts` now applies its source-length floor to
`target-far-shorter` alone;
`target-far-longer` is reported at any original length.

GFP,
done the cheap way rather than by juggling files:
the tests import from `dist`,
so writing the new surplus case and running it BEFORE rebuilding
runs it against the pre-fix build.
It failed there with `expected '' to include 'SIZE NOTE'`,
and passes after the rebuild.
Both shortfall cases pass either way,
which is the point of an asymmetric floor.

The test that used to sit in that spot asserted silence for an 800-character rendering
of a 79-character original.
That is 10.1 times,
exactly the shape the note exists to surface,
so the suite had been pinning the defect in place.

Verified by asking the shipped `contestSizeNote` itself,
with the three labels `lane-contest-wire.ts` passes,
over every comparison row on disk (`~/temp/agent/182-verify.mjs`):

    eligible rows          267
    note fired before        6
    note fires now          23   (8.6%)

The 23 rows are 9 distinct slices across 5 entries,
`saurikissa`,
`zheermao101`,
`Zha_Ke`,
`wangzihao980` and `dogesir_`,
counted more than once because several entries were settled in more than one run directory.
`vub171-20260822`'s `Zha_Ke` chunk 2,
the 185x runaway,
is among them.

Gate:
`lint` 0 warnings 0 errors,
`lint:types` exit 0,
suite 531 PASS / 0 FAIL / exit 0,
identical to baseline.

`displacement-class.ts:361` keeps its blanket floor deliberately.
It excludes short slices from SETTING the document baseline,
which is right in both directions:
a twenty-character slice should not get to define what normal expansion is.

#### Whether the fix reaches a judge, and why no key bump was needed

Verifying `contestSizeNote` proves the function speaks.
It does not prove a judged round will carry the note,
because the lane contest caches its ballots across runs
and #163 threaded the note with an explicit no-cache-bump decision,
taken when the note moved 2 rows rather than 23.
So the fix could have been inert exactly where it matters.

It is not,
and the reason is structural rather than lucky.
`pass-entry.ts` opens every cache with `generation: pipelineDigest`,
`openNamespacedCache` (`slice-cache-namespace.ts:462`) resumes only when `cached === generation`
and calls `discardNamespace` otherwise,
and `digestPipeline` digests the executable files under `dist/final/node`.
The lane contest owns its own prefix and marker,
because `LANE_CONTEST_NAMESPACE` and `EVERY_SLICE_NAMESPACE`
both read out of `CLAIM_BY_ROLE` in `slice-cache-claims.ts:56`,
so the discard is its own files rather than another lane's.

Editing `contest-size-note.ts` changed the built bundle,
which is not inferred:
the guard test failed against the un-rebuilt `dist` and passed after the rebuild,
which is direct evidence those bytes moved.
Different bytes mean a different digest,
a different generation,
and a discarded contest cache.
The 9 newly covered slices are re-judged with the note on the next run.

THE GENERAL RULE THIS ESTABLISHES,
worth keeping for every later behaviour change:
a change to executable source needs no manual cache bump,
because the digest covers it.
A cache key bump is needed only when the INPUT to a stage changes
while the code computing it does not,
which is what #173 and #178 were about:
a window or an incumbent that the key never covered.
