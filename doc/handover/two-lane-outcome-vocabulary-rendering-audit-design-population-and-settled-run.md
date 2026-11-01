# Lane outcome vocabulary handover: the rendering audit, from design to the settled-artifact run

The rendering audit for the translate lane:
the four artifacts that form its whole input,
the design decisions and the review that preceded its rebuild,
the live arms that exposed the matcher,
and the run over settled artifacts (#115) with its refusal to compare.

Part of [Handover: the lane outcome vocabulary and what is mid-flight](two-lane-outcome-vocabulary.md),
which lists every file of this record.

## Item 5's whole population is four files, and they were sitting in scratch

THE ONLY VERSION 2 ARTIFACTS THAT EXIST are four,
being two entries settled twice.
Measured rather
than assumed:
the default runs directory holds 56 settled artifacts and NOT ONE carries an
`artifactSchemaVersion` field,
so every one of them predates version 2 and none can answer a
two-lane question.

That matters because queue item 5 is the rendering audit run "standalone over EVERY settled version
2 artifact".
Every is four,
from two distinct entries.
That is still worth running,
being the
instrument's first application to real output rather than fixtures,
but it is not the census the
phrase suggests,
and nothing about particular entries can be settled from two of them.

THEY WERE BOTH IN `~/temp/agent`,
which is the agent scratch root and is named to be deleted.
A
cleanup or a REISUB would have destroyed item 5's entire input,
and re-settling those two entries
costs about 100 minutes of real quota.
They are now copied,
about 1.8 MB in total,
to:

```text
~/translation-repair-v2-archive/grace-remeasure-2026-08-17/{Aniloviraw,zheermao101}.json
~/translation-repair-v2-archive/grace-remeasure-2026-08-17/remeasure.log
~/translation-repair-v2-archive/two-lane-cost-2026-08-16/{Aniloviraw,zheermao101}.json
~/translation-repair-v2-archive/two-lane-cost-2026-08-16/two-lane-cost.log
```

THE LOGS WENT TOO,
3.7 MB with the artifacts.
They are the evidence behind every table in
`doc/audit/straggler-grace-remeasure.md`:
the per-stage voice completeness split by stage kind and
roster size was computed from them and cannot be recomputed from the artifacts alone.

OUTSIDE THE REPOSITORY ON PURPOSE:
they carry corpus text,
which does not enter this repo.

THE DEFAULT RUNS DIRECTORY IS GENUINELY DURABLE,
checked rather than worried about,
because it sits
under `node_modules/` and `resolveRunsDir` calls itself durable.
The 56 settled artifacts there are
dated 2026-08-06 to 08-11 and `node_modules/.modules.yaml` is dated 2026-08-13,
so an install ran
after they were written and did not remove them.
Nothing to fix;
recorded so it is not re-raised.

## What the rendering audit is for, and the design decisions inside it (`#85`)

The absent-baseline instrument the translate lane needs,
since a slice rendered from scratch has no
BEFORE text for the differential probe to compare against.
It sits BESIDE the introduced-defect probe rather than replacing it:
the repair lane still has a
baseline and `#66` still measures the old instrument.
The file inventory and the measured behaviour live under "The audit is rebuilt,
run live,
and
measured";
this section keeps only the reasoning that survived the rebuild,
so it is not
re-litigated.

Three decisions inside it are worth not re-litigating:

-   **Each category proves itself from the side it CAN.**
    An omission has nothing in the candidate to
    quote,
    its absence being the claim,
    so a symmetric rule would make the likeliest defect of a
    from-scratch rendering unprovable.
    An unsupported addition is the mirror.
    Everything that changes
    something both sides state must quote both.
-   **The archive is irrelevant by construction,
    not by instruction.**
    A source quote is searched in
    the original and nowhere else,
    so a claim resting on another translation's wording anchors
    nowhere and is dropped.
    The prompt says so too;
    the prompt is not what enforces it.
-   **Corroboration counts VOICES,
    not claims**,
    threshold two rather than a majority:
    the first
    question is whether a defect is there,
    and a majority over six would discard a defect four of
    them missed.
    The rows keep the counts,
    so a stricter rule can be applied to an existing run.

STILL WIRED TO NOTHING:
no lane calls it,
and where it should run is Question 8,
which is the
user's to answer.
Measured,
though,
on its own fixtures.

The second of those three decisions is now stated too strongly,
and the code has been left as it is
while the wording is corrected:
nothing in the anchoring makes a voice reason independently of an
archive it may have seen elsewhere,
and the `reason` field is not screened at all.
What is true is
narrower and still worth having:
the archive is not serialized into the prompt,
and archive wording
cannot serve as an anchor.
That is data-flow exclusion at the caller,
not a property enforced by
quote anchoring.

## The audit is rebuilt, run live, and measured (`#85`)

Both arms ran on 2026-08-17,
three times,
through `mise run
//package/module/translation-repair:audit-sensitivity` (moved on 2026-10-18 to `//package/translation-repair/probe:audit-sensitivity`).

THE CONTROL IS CLEAN,
in every run:
three auditors,
three verdicts of `no-defect-found`,
zero
claims,
zero corroborated,
zero near misses.
Nothing was invented about a faithful rendering.

THE POSITIVE ARM FINDS THE PLANTED DEFECT,
also in every run.
The auditors are not the weak part:
they located a dropped negator,
named it `altered-polarity`,
and anchored it.

WHAT THE ARMS EXPOSED is the matcher,
and only because the arm reports three measurements rather
than one count.
Run by run,
on the same fixture and the same roster:

-   Three voices,
    three oracle hits,
    no dropped claims,
    ONE corroborated defect,
    three near misses.
-   Three voices,
    three oracle hits,
    no dropped claims,
    ZERO corroborated defects.
    All three named
    `altered-polarity`;
    they quoted `不吃`,
    `吃` and `不吃罐头`.
    Unanimous agreement,
    reported as
    nothing.
-   Three voices,
    two oracle hits,
    one claim dropped as `unanchored-locator (source)`,
    zero
    corroborated,
    one agreed group of two voices.

Which span a voice picks is not stable between runs,
so a count keyed on exact spans is a coin flip.
That is why the report now carries a SECOND TIER,
`agreed`,
beside the strict `corroborated`:
it
groups claims of one category whose focus spans overlap,
pairwise throughout,
so two claims about
different words of one sentence never join and a wide claim touching two narrow ones pairs with each
rather than forming a trio.
Both counts are reported;
neither is folded into the other.

READ BOTH,
and read the per-voice rows under them.
The strict count is the conservative fact;
the
second is the useful one;
the oracle hits and the drop list say which component to blame when they
disagree.

## What the review found before any of that ran, kept because the reasoning still applies

An adversarial review of the three audit files on 2026-08-17 returned a NO-GO for the live arms,
and it was right.
The matcher cannot distinguish agreement on ONE DEFECT from agreement on ONE SENTENCE:

-   FALSE SPLIT,
    which was already suspected:
    `defectKey` demands both evidence strings match
    exactly,
    so two voices quoting different-width spans of the same dropped negation do not
    corroborate,
    and neither do two voices calling it `omission` and `altered-polarity`.
-   FALSE MERGE,
    which was not suspected and is worse:
    the quote floors (12 Latin characters,
    4 CJK)
    FORCE a voice to pad a short trigger with context,
    since `not`,
    `three` and a three-character
    Chinese negation all fail the floor.
    Two DISTINCT defects in one sentence then arrive as the
    same padded quote under the same category,
    and are reported as one twice-confirmed defect.
    The
    floors cause the defect they were meant to prevent.

Had the positive arm run first it would have failed,
and the failure would have been unattributable
between the auditors,
the anchoring and the matcher.

The rebuild separates a LOCATOR span,
which identifies one occurrence uniquely,
from a FOCUS span,
which carries the alleged change and need only be unique inside its locator,
keeps document offsets
for both,
and corroborates on the focus interval rather than on the text a voice typed.
`#85` carries the full scope,
including the one review finding deliberately not adopted (schema
enums for verdict and category,
which would turn vocabulary noncompliance into voice loss) and why.

Two things the review predicted arrived exactly as described,
which is worth recording because it
says what that kind of review is worth:
the false merge it found was real and unsuspected,
and the
false split it warned would spoil the positive arm did spoil it,
twice,
before the second tier
existed.
One thing it recommended was not taken and should stay untaken:
schema enums.

WHAT WAS NOT REBUILT,
and is recorded rather than done,
because each needs a decision or more than
one fixture to settle:

-   Cross-category agreement.
    Two voices naming one span `altered-polarity` and `omission` are
    reported as a near miss and counted as neither.
    Deciding whether they agree is a question about
    the taxonomy,
    and the precedence rule in the prompt is the current attempt to prevent it arising.
-   `broken-structure` still requires both sides.
    The review argued for splitting it;
    the precedence
    rule routes the one-sided cases to `omission` and `unsupported-addition` instead,
    which answers
    the objection by definition rather than by adding categories.
    Untested against a real structural
    defect.
-   `identityContext` is never anchored.
    An `altered-identity` claim resting on licensed evidence
    cannot point at that evidence,
    only at the two texts.

## Queue item 5 is under way: the rendering audit over settled artifacts (`#115`)

The FREE HALF is done,
committed and green,
and it changed the plan in three ways worth carrying:

-   The population is 40 subjects,
    not "roughly 40 per run".
    Every translate delivery row in all
    four archived artifacts is `decided`,
    so the whole ledger is the population.
-   16 of those 40 audit the ARCHIVE's own English rather than a fresh rendering.
    That split is
    recorded per subject and must be reported apart,
    or the instrument's first real measurement
    blurs its own denominator.
-   The audit needs NO corpus text.
    Delivery rows carry `sourceText`,
    and `outcome.acceptedText`
    equals `shippedText` on all 40.
    Re-preparation is still done,
    but only for `identityContext`
    and for provenance.

Everything measured,
with the numbers,
is in `doc/audit/rendering-audit-settled-population.md`.
That file also records the `#107` reading rule,
fixed BEFORE the run so the tally cannot move it.

Landed so far:

-   `src/corpus-run/rendering-audit-settled-input.ts`,
    the zero-quota reader.
-   `src/corpus-run/rendering-audit-settled-input.unit.test.ts`,
    eight cases on a throwaway git
    corpus of cat-themed invention.

A TRAP WORTH REMEMBERING:
the reader's first name,
`readSettledArtifact`,
already existed in
`artifact-read.ts` and reached the package through `sheet-barrel.ts`.
`export *` drops an ambiguous
name SILENTLY,
so `corpus-barrel.ts` exported one of the two new functions and quietly did not
export the other.
The only symptom was an import error in a test.
Anything added to a barrel should
be checked against the package's runtime exports,
not just against a type-check.

The fixture had to resolve the REAL git binary through `resolveGit()`.
This repository's PATH
exposes the policy shim,
which rejects fixture staging patterns and settles worktree copies against
the real repository;
concurrent cases then raced on `.git/cli-git-worktree-copy/v1/settlement.lock`
and `git init` failed with `ENOTEMPTY`.
`corpus-source.unit.test.ts` already had this right and
says so in a comment.

STILL OWED on `#115`:
the driver itself (`rendering-audit-settled.ts` plus a mise task),
which must
carry an `import.meta.main` guard from the start,
take `--cap` and `--only`,
treat `--cap 0` as the
zero-quota boundary check,
buy a small cap first and have it read in full before the rest,
and
persist through `persistProbeRun({ probeName: 'rendering-audit-settled', ... })`.
The probe name
deliberately does NOT say "census":
the write-up must not let the word carry an implication two
entries cannot support.

### Where `#115` stands at 14:40Z

The full run is IN FLIGHT:
`node dist/final/node/rendering-audit-settled.mjs`,
logging to
`~/temp/agent/audit-settled-full.log`,
18 of 40 subjects bought.
It persists to the real runs
directory on completion.
If it dies,
rerun it;
nothing is cached,
so a rerun buys all 40 again.

Landed since the last note:

-   `rendering-audit-settled.ts` plus `-args.ts`,
    the probe and its command line,
    with the
    `import.meta.main` guard and a `--cap 0` zero-quota mode that was run FIRST,
    on a throwaway runs
    directory:
    four artifacts read,
    all four verified,
    40 subjects counted,
    nobody asked anything.
-   `rendering-audit-settled-row.ts`,
    the shared row shape.
-   `rendering-audit-settled-read.ts` and `-relocation.ts`,
    the readings,
    plus eight cases.
-   A `mise` task,
    `//package/module/translation-repair:rendering-audit-settled` (moved on 2026-10-18 to
    `//package/translation-repair/probe:rendering-audit-settled`).

TWO THINGS THE SMALL FIRST BUY PAID FOR,
which is the entire argument for capping the first run:

-   The rows were storing COUNTS.
    One subject read `corroborated=0 agreed=0 near=1` over two voices
    claiming two defects each,
    and the file could not say what any of them claimed.
    That makes
    `#66`,
    `#68` and the `#107` relocation rule all unanswerable from the artifact the probe exists
    to produce.
    Fixed:
    the whole report is persisted,
    uninterpreted.
-   It priced the rest at roughly 30 seconds a subject.

ALSO OBSERVED,
and worth carrying:
the SAME subject (`grace-remeasure/Aniloviraw#0`) read
`claimed=1 corroborated=0` on the capped buy and `claimed=5 corroborated=1 agreed=1` on the full
run.
Identical text,
identical roster,
minutes apart.
Whatever the final tally says,
a single audit
of a slice is not a measurement,
and the six character-identical repeat subjects in the archive are
the only principled way to put a number on that.

STILL OWED when the run lands:
apply `splitFor`,
`rateByVoice` and `auditRelocationPairs` to the
persisted rows and write the result into `doc/audit/rendering-audit-settled-population.md`,
which
already holds the population and the reading rules.
Report the archive and fresh halves apart,
and
say plainly that two entries settle nothing about any entry.

### Where `#115` stands at 15:30Z

The first full run LANDED,
all 40 subjects,
as
`node_modules/.monochromatic/translation-repair-runs/rendering-audit-settled/2026-08-17T14-25-51.584Z-b7d84b5b.json`.
Its numbers are written up in `doc/audit/rendering-audit-settled-population.md`.

A SECOND full run over the same 40 is IN FLIGHT,
logging to `~/temp/agent/audit-settled-run2.log`.
It buys two things at once:
a pipeline digest that is actually the build that ran,
and the
run-to-run band.
Both matter because `QNB` says a comparison resolves nothing narrower than the
spread on unchanged input,
and the headline of `#115` is a comparison.
When it lands:

```sh
mise run //package/module/translation-repair:rendering-audit-settled-report -- \
  --against node_modules/.monochromatic/translation-repair-runs/rendering-audit-settled/2026-08-17T14-25-51.584Z-b7d84b5b.json
```

Moved on 2026-10-18 from `//package/module/translation-repair:rendering-audit-settled-report`
to `//package/translation-repair/probe:rendering-audit-settled-report`.

Then write the band into the population doc beside the halves.
That doc already says,
in its own
words,
that the archive-versus-fresh difference is not quotable without it.

Landed since the last note:

-   `rendering-audit-settled-repeat.ts`,
    the two pairings:
    within one run,
    where two artifacts of
    one entry carry identical characters at one slice;
    and across two runs,
    where every subject
    appears in both.
-   `rendering-audit-settled-band.ts`,
    `repeatBandOf`,
    which reads the spread off those pairs.
-   `rendering-audit-settled-digest.ts`,
    the recorded text identity.
    Rows now carry a DIGEST of the
    two texts each audit saw,
    never the texts:
    a run file is read,
    grepped and quoted,
    and the
    corpus goes only to the production provider.
    It is a tagged union so rows that predate it read
    as "cannot say" rather than pairing through their shared absence.
-   `rendering-audit-settled-print.ts`,
    split out of the report to keep it under its line budget.
-   `--against <run>` on the report,
    plus the fix that made the report print the archive the RUN
    named rather than the one this invocation defaulted to.
-   `rendering-audit-settled-repeat.unit.test.ts`,
    thirteen cases,
    mostly refusals.

TWO TRAPS FOUND,
both worth carrying:

-   NUL BYTES IN COMMITTED SOURCE.
    Six of them,
    where spaces were intended,
    between the
    interpolations of every composite key these readings build;
    one had been there since the `#107`
    key builder landed.
    Nothing misbehaved,
    because a NUL is a better separator than a space and
    both sides built keys the same way.
    `rg` and `grep --perl-regexp` report a CLEAN FILE.
    Full
    write-up and the searches that do work:
    `doc/troubleshooting/nul-bytes-land-invisibly-in-source.md`.
    The separator is now
    `SLOT_SEPARATOR`,
    a shared constant written as an escape.
-   ANOTHER BARREL COLLISION,
    `bandOf`,
    already in `band-order.ts`.
    This one failed LOUDLY as
    TS2308,
    unlike `readSettledArtifact`,
    which was also a value and was dropped in silence.
    Why
    the two differ is not established,
    so
    `doc/troubleshooting/barrel-star-export-drops-ambiguous-names.md` keeps its warning:
    a clean
    type-check does not prove there is no collision.

### `#115` IS DONE, and its answer is a refusal

Three audits of the same 40 subjects,
all persisted in
`node_modules/.monochromatic/translation-repair-runs/rendering-audit-settled/`:

```text
run 1  2026-08-17T14-25-51.584Z-b7d84b5b   digest stamped WRONG, see below
run 2  2026-08-17T15-06-37.885Z-c157db5b
run 3  2026-08-17T15-44-00.750Z-eeda8b9b
```

The comparison the audit exists to make CANNOT BE MADE on this population,
for two
independent reasons,
either sufficient alone:
it is confounded with the entry,
and the
band is wider than the effect.
Over 40 digest-verified pairs of identical text the
corroborated total moved from four to ten,
and ten of forty subjects flipped between
claiming something and claiming nothing.
Full write-up and every number:
`doc/audit/rendering-audit-settled-population.md`.
Task `#115` carries the summary.

WHAT UNBLOCKS IT:
more ENTRIES,
not more slices.
Slices tighten each cell without adding
a degree of freedom.
That means more settled version 2 artifacts,
which is what `#108`
would produce,
and `#108` now records this as a second reason to run.

RE-READING ANY RUN COSTS NOTHING:

```sh
cd /var/home/user/worktrees/translation-repair
R=/var/home/user/worktrees/translation-repair/node_modules/.monochromatic/translation-repair-runs/rendering-audit-settled
mise run //package/module/translation-repair:rendering-audit-settled-report -- \
  --run $R/2026-08-17T15-44-00.750Z-eeda8b9b.json \
  --against $R/2026-08-17T15-06-37.885Z-c157db5b.json
```

Moved on 2026-10-18 from `//package/module/translation-repair:rendering-audit-settled-report`
to `//package/translation-repair/probe:rendering-audit-settled-report`.

Paths must be ABSOLUTE:
the mise task runs from the package directory,
so a repo-relative
path gives `ENOENT` naming a path that exists.
Run 1 pairs against nothing,
because its
rows predate the recorded text identity;
the report says so in its own sentence rather
than reporting it as an archive that moved.

TWO TASKS CAME OUT OF THIS AND ARE OPEN:
`#116`,
stamp runs with the runner's own
dependency closure rather than a whole-tree digest,
since the closure is what a band
measurement needs.
`#117`,
every stage round leaks a live 180-second timer,
so any CLI
running one outlives its own work;
NOT fixed because `stage-round.ts` is in the producing
path `#115` was forbidden to touch.

### Where `#115` stood at 15:45Z, kept for the trail

RUN 2 LANDED as `2026-08-17T15-06-37.885Z-c157db5b.json` and is written up.
RUN 3 IS RUNNING,
launched DIRECTLY in background task `bveiyodmy`,
log `~/temp/agent/audit-settled-run3.log`.

THE CHAIN THAT WAS MEANT TO START IT DID NOT WORK,
and the reason is worth carrying:
`pgrep --full 'node dist/final/node/rendering-audit-settled.mjs'` matches the SHELL RUNNING THE
PGREP,
because that pattern is inside the shell's own command line.
The wait loop found itself and
never exited.
It was killed by PID and run 3 started by hand.
Separately,
the probe holds its
process open for about three minutes after printing `kept at`,
on sockets left by voices abandoned
after quorum,
so `kept at` is the completion signal and process exit is not.
Both are written up in
`doc/troubleshooting/chaining-a-command-behind-a-long-run.md`.

RUN 2 AND RUN 3 ARE NOT ON THE SAME BUILD,
and the doc says why that is acceptable rather than
pretending otherwise.
`rendering-audit-settled-digest.ts` gained `isDigested` between them,
and
that module IS in the runner's closure,
so the chunk changed.
`digestAuditedText`,
the only part of
it the runner calls while writing rows,
did not change,
so both runs produce rows by identical
logic.
The population doc records the closure and the distinction.

REBUILDING NOW IS SAFE:
run 3 loaded its code at startup and reads its digest at start,
so neither
its behaviour nor its recorded identity can move under it.

Why three runs and not two:
run 1 has no recorded text identity,
so pairing it against anything
yields 40 subjects that match by position and cannot be vouched for.
Run 2 alone supplies six
digest-verified pairs from the character-identical subjects.
Run 2 against run 3 supplies forty.

WHEN RUN 3 LANDS:

```sh
cd /var/home/user/worktrees/translation-repair
R=/var/home/user/worktrees/translation-repair/node_modules/.monochromatic/translation-repair-runs/rendering-audit-settled
mise run //package/module/translation-repair:rendering-audit-settled-report -- \
  --run $R/<run3>.json --against $R/<run2>.json
```

Moved on 2026-10-18 from `//package/module/translation-repair:rendering-audit-settled-report`
to `//package/translation-repair/probe:rendering-audit-settled-report`.

Paths must be ABSOLUTE.
The mise task runs from the package directory,
so a repo-relative path
gives `ENOENT` naming a path that exists.

Then write the band into `doc/audit/rendering-audit-settled-population.md`,
which already fixes how
to read it:
the headline gap is a CORROBORATED gap,
so the band on corroborated counts decides
whether it survives,
and the doc says in advance that about four means noise and nought or one
means it stands.
It also predicts `auditRepeatsWithin` will find exactly the six subjects the
population count named,
by a different route through different data.

Landed since the last note:

-   The three-way split in `auditRepeatsAcross`.
    It would have reported all 40 subjects of run 1 as
    "the archive moved between the two runs" in exactly the `--against` command this handover
    prescribes.
    The archive did not move;
    run 1 predates the field.
    Now `paired`,
    `textMoved` and
    `unverifiable`,
    worded apart by the printer,
    with the new case shown to fail first.
-   The relocation candidates measured structurally,
    no text:
    both anchor against their shared slice
    boundary,
    which is the geometry `#107` predicts,
    and that is in the population doc.
-   A double count found in the same data:
    on `zheermao101` slice 0,
    Qwen anchored BOTH an
    `altered-actor` and an `unsupported-addition` to the identical candidate span.
    `claimed` counts
    claims,
    not distinct problems.
-   `#116`,
    a task:
    stamp runs with the runner's own dependency closure instead of a whole-tree
    digest,
    since the closure is what a band actually needs and the built entry already names it.

VERIFIED,
so nobody re-does it:
full unit suite green at FINAL HEAD,
371 passing describes,
zero
failures,
zero assertion or type errors,
`~/temp/agent/suite-final.log`.
Checked from the FILE with
a positive control rather than from an exit code:
the six new reading modules appear in the log as
having run,
and so does the fourteenth case,
`SEPARATES "the archive moved" FROM "nobody recorded
what this saw"`,
which was added after the previous green suite.

That positive control is not ceremony.
An earlier suite run in this session was piped through
`rg ... | head --lines=30`,
and `head` closed the pipe on a stream whose first thirty matching lines
were all `Error` fixtures,
so nothing of the actual result survived and the exit code was `head`'s.
Grep the file,
never the pipe.

GFP was satisfied for the sharpest guard:
removing the unrecorded-rows refusal from
`sameAuditedText`,
rebuilding and running gave `expected true to equal false`,
and the guard was
restored and re-verified.
The run aborted at that describe,
so the OTHER refusals in
`auditRepeatsWithin` were not exercised under the broken guard and have not been shown to fail.
