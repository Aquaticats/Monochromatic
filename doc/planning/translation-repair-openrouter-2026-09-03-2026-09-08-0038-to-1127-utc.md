# OpenRouter and the corpus passes: 2026-09-08, 00:38 to 11:27 UTC

What was launched,
read and decided on 2026-09-08 00:38 to 11:27 UTC.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## The sixth pass settles; the second footnote page, 2026-09-08, 00:38 UTC

`TALLY hakureico status=SETTLED slices=18` at 00:38:13 UTC,
`repairIssues=181 repairAccepted=114 repairResolved=109 repairFindings=279 repairChanged=13`,
`translateChanged=16 documentsDiffer=18 pageChanged=16 pageSilent=0 alignmentFindings=5 selection=contested`,
`ms=3610303`:
60.2 minutes,
zero `[error]` lines,
`verify-published` matched 1 of 1 at the length the artifact implies.
The page is `~/temp/agent/hakureico6-20260907/fixed/people/hakureico/page.en.md`,
195 lines against the archive's 157,
on tip `49aca5770` and pipeline `6bb40f9b`.

### The footnotes, read first

Both on the page and both defined.
`Mayday[^1]` sits on the "In May" sentence,
`HOSTED__WITH__GAE[^2]____` on the last line of the transponder blockquote,
and the definitions are the page's last two lines:
`[^1]` names the internationally used radio distress signal,
which from an aircraft usually signals an extreme emergency requiring immediate rescue,
and `[^2]` reads `That is, Google App Engine`.
The third pass had shipped this page without them for the thirteenth class's reason;
this is the first hakureico page that carries them,
and the second read page with a footnote after `TLL1122`.

### The seats lines

Six,
one per phase,
every one `waited=0ms`;
no `short of quorum`,
no `chunk resumes`,
no `names its return`:
Hyper's daily limit did not close during the pass,
so the thirteenth class had nothing to wait for.
Synthetic read wet at one reading of the six,
`phase=lanes` at 23:43 UTC,
where the bench was whole
(`wide=8 select=8 late=9 slate=9 checkers=3 translators=7 readers=4 writers=9 roster=10 withheld=none`);
the other five read it dry with the two Hyper-slow judges withheld
(`wide=7 select=6 late=8 slate=7 checkers=3 translators=6 readers=3 writers=8 roster=9`).
The phases by their readings:
preparation 23:38:03 UTC,
pictures 23:38:38 UTC,
lanes 23:43:03 UTC,
translate lane 00:16:01 UTC,
lane contest 00:25:24 UTC,
consolidation 00:31:28 UTC,
tally 00:38:13 UTC.
The lanes took 33 minutes where the third pass's took 8 (21:52 to 22:00 UTC).

### The fourteenth class in production

At 00:36:13 UTC,
two minutes into consolidation,
Hyper answered 402 on seven concurrent calls
(`hyperBalance=0`;
277 at launch,
748 calls in this pass),
the router walked each to OpenRouter,
and OpenRouter answered 402 on the same seven
(`openrouterUsd=0.01`;
`You requested up to 131072 tokens, but can only afford 6762`).
Each call marked its provider once,
`hyper: refused us for payment while its meter reads dry (hyperBalance=0)`
and `openrouter: refused us for payment while its meter reads wet (openrouterUsd=0.01)`,
both with the tail `reads dry until that meter moves`,
fourteen lines in the same instant,
and neither provider was asked again for the rest of the pass,
where the fifth pass had walked back into OpenRouter's wall every 60 s.
From that instant every voice a model without a Bedrock seat owed was lost at once
(`no provider can take <model>: every provider serving this model is out of budget, voice lost`,
625 lines),
and the consolidation finished its fifteen remaining slices on the Bedrock seats alone,
00:36:14 to 00:38:13 UTC.

### The consolidation on two seats

Of eighteen slices,
three shipped consolidated (slices 0 and 2 and 16),
five gate-kept-standing,
seven slate-declined-standing,
two slate-unjudged-standing,
one incumbent-only;
the sixteen changed page slices come from the lanes.
Slices 1 and 2 were reviewed by a full bench and read `unacceptable`,
recorded as evidence with the gated text shipping.
From slice 3 on every absolute naturalness review read `quorum-not-met`
(`3/8 usable` while the Hyper voices were being lost,
then two valid verdicts of two from slice 7),
each recorded on its settlement as
`absolute naturalness review quorum not met; recorded as evidence`
while the gated or standing text ships.
The thirteenth class did not wait here,
by design:
a payment mark carries no hold,
`providerHolds()` reads zero for it,
and `awaitBenchQuorum` returns at once,
since there is no instant at which a spent balance comes back.
So a pass that loses a paid provider inside a phase finishes the phase on whoever is left,
and says so on every review line;
the fourth pass had stopped INCOMPLETE under the same reading only because the slice it declined
held a text the block floor refused,
and none of this pass's standing texts did.

### Meters and spend

Bedrock 199.34 to 198.99 USD on the meter;
511 `SPEND` lines summing 0.4006 USD
(`google.gemma-4-26b-a4b` 198 calls 0.14 USD,
`openai.gpt-oss-120b` 153 calls 0.22 USD,
`google.gemma-4-e2b` 160 calls 0.03 USD).
Hyper 277 to 0 over 748 calls,
spent out at 00:36:13 UTC;
the owner will not recharge it.
Synthetic 212 calls in its one wet window,
`syntheticWeekly=0%` throughout,
`syntheticFiveHour=2750/2750` at the tally.
OpenRouter 0.01 USD,
no call answered.
2118 calls asked by the `SEAT` lines;
16 voices abandoned after quorum.
`google.gemma-4-e2b` in its first seated pass:
asked 160,
usable 160,
threw 0.

### The read-page checks

Headings 3 of 3 against the source's three,
comments 3,
`PhotoScroll` kept,
0 straight apostrophes in words (5 curly against the archive's 8),
0 straight double quotes (16 curly against 18),
1 three-dot ellipsis and 0 unicode (archive 0 and 0),
every forbidden rendering 0,
every refusal grep 0.
The page differs from the third pass's page on 153 of its lines:
two passes' wordings,
not a class.
No class found on this page.

## The seventh hakureico launch on Bedrock alone stops at the pictures, 2026-09-08, 01:49 UTC

Launched at 01:49:33 UTC on `ed278cbf1`
(docs only past `49aca5770`,
pipeline `6bb40f9b` unchanged),
plain invocation,
runs dir `~/temp/agent/hakureico7-20260908`,
log beside it,
pid 3956345,
after `budget-sample` read
`synthetic=dry bedrock=wet hyper=dry openrouter=wet syntheticWeekly=0%`
and `bedrockUsd=198.94 hyperBalance=0 openrouterUsd=0.01`.
Launched on the owner's words that a single wet provider is normal operation
(`Run now, because only one provider is wet is just normal`),
to read what the pipeline does on Bedrock alone,
the state production is in tonight.

`TALLY hakureico status=INCOMPLETE ms=20233 aborted=false error=visual evidence incomplete for 2 referenced assets`,
20 seconds,
no Bedrock spend on the meter.

WHAT THE TWENTY SECONDS SHOWED:

-   OpenRouter's meter reads wet at 0.01 USD at every launch,
    so `phase=preparation` seated its six models
    (`wide=7 select=7 late=8 slate=8 checkers=3 translators=6 readers=3 writers=8 roster=9`,
    `withheld=hf:moonshotai/Kimi-K3`),
    the first block-pairing round lost all six to 402 in 300 ms,
    and the fourteenth class marked OpenRouter dry for the rest of the process.
    A fresh process starts unmarked,
    so every launch at this balance buys the same six refusals.
    Harmless and noisy;
    the 402 body names what the balance can afford
    (`can only afford 6762` tokens against a 131072 request),
    which is the reading a threshold would want,
    and the owner's rule against magic numbers keeps 0.01 USD out of the meter.
-   The block pairing paired every relation from `3 usable voices of 3 heard`,
    the three Bedrock seats,
    `3/9 heard` each round.
-   `phase=pictures` then read
    `synthetic=dry bedrock=wet hyper=dry openrouter=dry`
    `wide=8 select=8 late=9 slate=9 checkers=3 translators=7 readers=4 writers=9 roster=10`
    `withheld=none waited=0ms`:
    the fullest bench the log has ever printed,
    with three providers dry.
    `withheld=` names only the seats the Hyper-slow and OpenRouter-cost rules take,
    and with Hyper and OpenRouter dry neither rule applies,
    so nothing is withheld and nothing on the line says that seven of the ten have no provider.
    The bench a reader of the log sees is not the bench the router can reach.
    `reachableSeats` already computes the difference,
    but the seats line prints it only when a hold is running (the thirteenth class's clause).
-   The readers bench is `hf:zai-org/GLM-5.3-Flash`,
    `hf:Qwen/Qwen3.8-27B`,
    `hf:moonshotai/Kimi-K3` and `minimax-m3`,
    none of which Bedrock serves;
    both photos read `0 of 4 readers produced a reading`,
    `unavailable for a transient reason, so it is not cached and is read again next run`,
    and the entry stopped honestly.
    Bedrock's four cards all carry `readsImages: false` in `bedrock-catalog.ts`,
    by the rule the OpenRouter catalog set:
    a listing field alone cannot widen the reader roster,
    a transcription has to be measured first
    (`roster-reach.unit.test.ts`,
    `KEEPS gemma off the picture readers on OpenRouter and on Bedrock until a transcription is measured`).
    So on Bedrock alone no pass can pass its pictures phase until that measurement is taken.

WHAT FOLLOWS:
two pieces of work,
neither a design decision.
The seats line should say how many seats of each bench the router can reach at that reading,
hold or no hold,
so a phase that will run short of quorum says so before it starts;
and Gemma 4 on Bedrock should be measured as a picture reader against the readings the four seated readers gave
on pages already read,
which is the same shape as the judge-seat measurement of 21:19 UTC.
Until one of those lands or another meter moves,
a launch on Bedrock alone stops at the pictures in twenty seconds at no cost,
which is a fine thing to know and not a page.

### Gemma 4 on Bedrock as a picture reader, first probe, 02:05 UTC

Four calls through the production Bedrock client
(`bedrock-reader-probe.mjs` in the session scratch:
the reader stage's own instruction and first perspective,
the picture as the same `image_url` data-URI part the stage sends,
the ledger the meter reads),
against the OCR reading of each picture,
in the trigram overlap `readImagePair` corroborates readers with.
Bedrock's endpoint takes the image part for both Gemma sizes:
every call answered in 1.5 to 2.6 s.

-   `photo2.webp` (114 KB,
    a captioned photograph,
    OCR 33 characters):
    `google.gemma-4-e2b` 84 characters at overlap 0.893,
    `gemma-4-26b-a4b-it` 65 characters at 0.893,
    both `corroborated`;
    the seated three had corroborated it at 0.767 in the sixth pass.
-   `photo1.webp` (168 KB,
    a dense screenshot,
    OCR 4652 characters):
    `google.gemma-4-e2b` 423 characters at 0.129,
    `disagree`,
    and the text is an invented ticket template
    (`TrainTicketSummary= Date=2026/09/08 TicketID= PassengerName= FlightNumber=`),
    nothing from the picture;
    `gemma-4-26b-a4b-it` 231 characters at 0.376,
    `corroborated` at the threshold,
    and the text is the spreadsheet chrome
    (menu labels,
    column letters,
    row numbers),
    none of the cells.
    The seated three had corroborated it at 0.902.

WHAT THAT SAYS,
on two pictures:
the E2B size invents a reading for a dense picture and echoes the prompt's `READING RESPONSIBILITY:` label into it,
so it does not take a reader seat;
the 26B-A4B size reads a caption whole and a dense screenshot's furniture,
which is neither a seat nor a refusal on this sample.
The measurement that decides it is the same shape as the judge probe:
every picture the seated readers have already read,
each Gemma reading scored by overlap against the seated readers' corroborated text,
the way `readImagePair` scores a fourth reader.
That needs the seated readings as text,
which the pass records only as overlaps;
where they are cached is the next thing to find.

### The three Gemma sizes against the seated readers on nine pictures, 02:20 UTC

The instrument (`bedrock-reader-measure.mjs` in the session scratch):
every picture whose seated readers left a `corroborated` record in a runs dir's picture cache
(twelve unique records over `gqt`,
`Toka_ls`,
`hakureico`,
`luxuanwen3` and `XingZ60`),
matched to its corpus picture by the OCR reading's overlap with the record's texts
(floor 0.3;
`gqt/photo3.webp` matched nothing and is out),
nine pictures in all;
each candidate asked through the production Bedrock client with the reader stage's own prompt;
each reading scored by the trigram overlap `readImagePair` uses and by its `readingsCorroborate` verdict
against every seated reading of that picture.
The positive control is each seated reader scored the same way against the others.

THE SEATED BAND:
every seated reader is corroborated by every other seated reader on every picture it read
(Kimi-K3 8 of 8,
Qwen3.8-27B 7 of 7,
minimax-m3 6 of 6,
GLM-5.3-Flash 5 of 5,
Qwen3.6-27B 2 of 2),
none uncorroborated,
mean overlap against the others 0.80 to 0.92.
The records hold only the readings that were produced,
so a seated reader's failed or empty call is not in its count.

THE CANDIDATES:

-   `gemma-4-26b-a4b-it` (Bedrock `google.gemma-4-26b-a4b`):
    nine asked,
    eight produced,
    every one of the eight corroborated by every seated reader,
    mean overlap 0.668 (first run 0.683,
    a run-to-run band of 0.015),
    mean 5.9 s.
-   `google.gemma-4-31b`:
    nine asked,
    eight produced,
    every one of the eight corroborated by every seated reader,
    mean overlap 0.671,
    mean 8.7 s,
    up to 29 s on the two dense screenshots.
-   `google.gemma-4-e2b`:
    nine asked,
    eight produced,
    one cut at the 32000-character content bound
    (`StreamOverrunError`,
    a runaway on `Toka_ls`),
    six corroborated by every seated reader and two by none
    (an invented reading on `Toka_ls/photo1.webp` at overlap 0.006 and a drifted one on `photo3.webp` at 0.279),
    every reading opening with the prompt's `READING RESPONSIBILITY:` label,
    mean overlap 0.430.
-   The picture none produced:
    `gqt/photo1.webp`,
    1.27 MB,
    answered with an empty stream by all three sizes,
    no error;
    the seated readers read it at 2718 and 2748 characters.
    Recorded as a Bedrock limit on picture size to find;
    the reader stage already turns an empty reply into `empty-reply` for that reader and the others carry the picture.

THE RULE FOR A READER SEAT,
written after these numbers and before any others,
in the shape of the wide-seat rule:
a candidate joins the readers when,
over the same pictures,
each reading it produces is corroborated by every seated reader of that picture at least as often
as the seated reader corroborated least often,
and it produces a reading no seated reader corroborates no more often than the seated reader
uncorroborated most often;
a call that produces nothing (empty,
cut,
thrown) counts for neither clause,
as it is absent from a seated reader's record too.
On the nine:
`gemma-4-26b-a4b-it` and `google.gemma-4-31b` meet both clauses (8 of 8,
none uncorroborated,
against 100 percent and 0);
`google.gemma-4-e2b` meets neither (6 of 8,
two uncorroborated).
The two join the readers through Bedrock's cards
(`readsImages: true` on `google.gemma-4-26b-a4b` and `google.gemma-4-31b`,
OpenRouter's row for the same model unchanged),
which makes the readers bench six and gives a Bedrock-only pass two reachable readers,
enough for `readImagePair` to corroborate.
On record beside the seat:
their overlaps sit below the seated band
(0.67 against 0.80 to 0.92),
which is a less exact transcription that the seated readers still corroborate,
and every cached picture reading is re-read once,
since the cache key names the reader roster.

## The eighth hakureico launch on Bedrock alone, on the reader build, 2026-09-08, 02:19 UTC

Launched at 02:19:56 UTC on `56ccbdc23`
(code `f7f9c9136`,
pipeline `e069d942`,
the two Bedrock reader seats and the readers' pair quorum),
plain invocation,
runs dir `~/temp/agent/hakureico8-20260908`,
log beside it,
pid 3992895,
after the full unit suite read 950 `PASS` and 0 `FAIL` on the build.
`METERS` at launch:
Synthetic dry (`syntheticWeekly=0%`),
Bedrock 198.93 USD,
Hyper 0,
OpenRouter 0.01 USD reading wet.
`phase=preparation` read `readers=5` with Kimi-K3 withheld under OpenRouter;
`phase=pictures` at 02:20:11 UTC,
after OpenRouter's six 402s,
read
`synthetic=dry bedrock=wet hyper=dry openrouter=dry`
`wide=8 select=8 late=9 slate=9 checkers=3 translators=7 readers=6 writers=9 roster=10`
and no shortfall line,
since the readers bench holds two reachable seats against its pair quorum.
The reading follows as for the seventh launch:
whether both pictures corroborate on the two Bedrock readers,
then every phase's shortfall line
(`wide 3 of 8 reachable, quorum 4` is expected at the lanes,
the contest and the consolidation),
then whether a page on Bedrock's three judges and one translator ships at all,
with what its settlements say.

### The eighth pass settles in 4.4 minutes; what one provider buys, 02:24 UTC

`TALLY hakureico status=SETTLED slices=18 repairStatus=unchanged repairIssues=38 repairAccepted=29 repairResolved=0`,
`repairFindings=315 repairChanged=0 translateStatus=unfilled translateChanged=6 documentsDiffer=6`,
`pageChanged=5 pageSilent=1 alignmentFindings=4 selection=contested ms=266781`:
4.4 minutes,
zero `[error]` lines,
`verify-published` matched,
Bedrock 198.93 to 198.71 USD on the meter against 253 `SPEND` lines summing 0.22 USD
(`openai.gpt-oss-120b` 80 calls,
`google.gemma-4-26b-a4b` 98,
`google.gemma-4-e2b` 73,
`google.gemma-4-31b` 2).
2382 calls asked by the `SEAT` lines,
2129 of them thrown as `NoProviderForModelError` for the seven seats no wet provider serves.

WHAT THE TWO READER SEATS BOUGHT:
both pictures corroborated on the two Bedrock readers
(`photo1.webp` by 2 readers at overlap 0.653,
`photo2.webp` at 0.814),
where the seventh launch had stopped;
`google.gemma-4-31b` in its first seated calls:
asked 2,
usable 2.

WHAT THE SHORTFALL LINE SAID,
at every phase after OpenRouter's first 402:
`lanes`:
`wide 3 of 8 reachable, quorum 4; translators 1 of 7 reachable, quorum 4`;
`translate lane`:
`translators 1 of 7 reachable, quorum 4; select 3 of 8 reachable, quorum 4`;
`lane contest`:
`wide 3 of 8 reachable, quorum 4`;
`consolidation`:
`slate 3 of 9 reachable, quorum 5; wide 3 of 8 reachable, quorum 4`;
each ending `no provider has named its return, so the phase runs on what is reachable`.
The pictures phase printed none,
its two reachable readers meeting the pair.

WHAT THE PAGE IS:
162 lines against the archive's 158,
38 lines differing from the archive,
148 from the sixth pass's page,
no footnote
(`translateStatus=unfilled`:
the passage the archive lacks stayed unfilled,
as on the third pass).
The repair lane changed nothing:
its editor and refiner benches have no Bedrock seat,
so `editor round: 0/3 heard` 40 times and `refiner round: 0/3 heard` 52 times,
38 issues found by the three reachable critics and none resolved.
The five changed slices are the translate lane's,
written by the one reachable translator (`gemma-4-26b-a4b-it`,
`translate round: 1/7 heard` 17 times)
and chosen at the lane contest by three ballots each
(`lane-won:translate` with 3 ballots on five slices,
`settled-neither` with 2 on one):
the contest asks a minimum ballot weight rather than a majority of its bench,
so three judges of nine decide.
The six slices that reached consolidation all kept standing text
(five `slate-declined-standing`,
one `gate-kept-standing`,
every naturalness review `quorum-not-met`),
and slice 14 shipped with `standing lacks contest endorsement` recorded as a finding.
The changed prose reads as English
(the Giftia paragraph,
the IKEA sentence,
the former lover's answer),
by one writer,
unrepaired,
unpolished.

THE READING:
the pipeline on one provider is honest in every line it prints and ships a page anyway.
`repairStatus=unchanged translateStatus=unfilled` on the tally,
`short of quorum` before every phase,
`quorum-not-met` on every review;
and a page with five one-writer slices and no footnotes goes to `fixed/`,
where the sixth pass's page had fourteen changed slices,
three consolidated,
and both footnotes.
Nothing in the settlement distinguishes a page a whole bench made from a page one writer and three judges made,
except the findings.
Whether a pass whose editor,
refiner and translator benches have no reachable seat should ship at all,
or stop INCOMPLETE until a provider returns,
is the owner's to decide;
it is recorded as the open question in the readiness signal.
Until it is decided,
no further launch on Bedrock alone:
the next one would buy the same page for the same 0.22 USD.

## The ninth hakureico launch, four providers wet, 2026-09-08, 11:27 UTC

The owner topped OpenRouter up
(`openrouterUsd=200.01`)
and answered the single-provider question:
a pass whose writing bench has no reachable seat stops INCOMPLETE
([`translation-repair-writing-bench-floor.md`](../decision/translation-repair-writing-bench-floor.md)).
`budget-sample` at 11:26 UTC read every provider wet:
`syntheticWeekly=5.8%`,
Bedrock 198.71 USD,
Hyper 250,
OpenRouter 200.01 USD.
`hakureico` launched at 11:27:26 UTC on `061b46c0b`
(code `f7f9c9136`,
pipeline `e069d942`,
the eighth pass's build),
plain invocation,
runs dir `~/temp/agent/hakureico9-20260908`,
log beside it,
pid 4118147.
`JUDGE SEATS phase=preparation` read every provider wet and every bench whole:
`wide=8 select=8 late=9 slate=9 checkers=3 translators=7 readers=6 writers=9 roster=10 withheld=none waited=0ms`,
the first reading with six readers.
Launched before the writing-bench floor is built,
since a whole bench never meets it;
the floor is built in a throwaway worktree
(`~/temp/agent/tr-class15-20260908`,
branch `translation-repair-class15`)
and merged after the pass settles,
so the running worktree's source does not move under it.
The reading this page owes:
the consolidation by a whole bench,
which the sixth pass ran short of quorum from slice 3,
then the footnotes,
then every seats line and any shortfall line.

### The writing-bench floor, built in the throwaway while the ninth pass runs, 11:50 UTC

Built on branch `translation-repair-class15` in `~/temp/agent/tr-class15-20260908`
(commit `5ac7b6f49` on `1ddbcc75a`),
so the running worktree's source does not move under the ninth pass;
merged into `translation-repair-rebased` after that pass settles.

THE SHAPE:

-   `run-seats-floor.ts` (new):
    `WRITING_BENCHES` (editors,
    refiners,
    translators),
    `WRITING_BENCH_FLOOR = 2`
    (the pair a slate needs,
    the same number the readers are held to),
    `WritingBenchUnreachableError` (`messageNamesOnly`,
    as the visual-evidence stop),
    `benchesOf({ seats })` naming every bench the readings look at,
    and `unreachableWritingBenches({ benches, names, dry })` returning clauses like
    `editors 0 of 3 reachable, floor 2`.
-   `run-seats-wait.ts`:
    `BenchName` gains `editors` and `refiners`,
    and the lanes phase leans on wide,
    editors,
    refiners and translators.
    The quorum clauses keep their majority threshold,
    so the thirteenth class's wait is unchanged:
    a bench short of its majority under a named hold still waits.
-   `run-seats-read.ts`:
    every phase reading computes both the quorum shortfall and the floor shortfall.
    With no hold,
    a judge bench short of quorum runs and says so (as since `c4a9682fe`),
    and a writing bench below the floor throws the error,
    which the entry queue records as INCOMPLETE
    (`stage-local work remains; whole entry will not restart in this invocation`).
    Under a hold the reading waits once,
    reads again,
    and throws if a writing bench is still below the floor.
    The per-chunk reading still costs nothing while nothing is held,
    so a bench lost inside a phase with no hold finishes the phase,
    as the sixth pass did;
    that case is recorded as unsettled in the decision.
-   Tests:
    `run-seats-floor.unit.test.ts` (new,
    the eighth pass's view at the lanes and the translate lane,
    nothing at consolidation or the pictures,
    the error's message),
    two cases in `run-seats-read.unit.test.ts`
    (stop with nothing to wait for;
    wait once then stop,
    or seat when the wait brought the bench back),
    and the lanes clauses in `run-seats-wait.unit.test.ts` now name editors and refiners.
    Guards neutralised 2,
    2,
    2 and 3 `FAIL`;
    restored 0.

WHAT THE EIGHTH PASS WOULD HAVE DONE ON THIS BUILD:
stopped at 02:21:13 UTC at the lanes reading with
`writing bench unreachable at lanes: editors 0 of 3 reachable, floor 2; refiners 0 of 3 reachable, floor 2;`
`translators 1 of 7 reachable, floor 2`,
after its pictures and for the Bedrock cost of the pairing and the pictures alone.

### The ninth pass settles in 83 minutes on a whole bench; the name two rules render differently, 12:50 UTC

`TALLY hakureico status=SETTLED slices=18 repairStatus=repaired repairIssues=134 repairAccepted=90 repairResolved=85`,
`repairFindings=226 repairChanged=13 translateStatus=complete translateChanged=17 documentsDiffer=16`,
`pageChanged=16 pageSilent=0 alignmentFindings=4 selection=contested ms=4998984`:
83.3 minutes,
zero `[error]` lines,
`publish: wrote 18 slices into a page of 4878 characters` at 12:50:44 UTC.
Six `JUDGE SEATS` lines,
every provider wet on each,
`wide=8 select=8 late=9 slate=9 checkers=3 translators=7 readers=6 writers=9 roster=10 withheld=none waited=0ms`,
no shortfall line,
no `quorum-not-met`,
no `chunk resumes`,
no hold named.
The phases:
pictures at 11:28:41 UTC,
lanes at 11:35:04 UTC,
translate lane at 12:01:28 UTC,
lane contest at 12:16:47 UTC,
consolidation at 12:21:04 UTC.

WHAT A WHOLE BENCH BOUGHT,
the reading this page owed:
`photo1.webp` corroborated by 5 readers at overlap 0.878
(`hf:zai-org/GLM-5.3-Flash` failed outright on it,
its stream cut),
`photo2.webp` by 6 at 0.949.
The repair lane changed 13 slices
(134 issues,
90 accepted,
85 resolved),
the translate lane 17,
and 16 of 18 differed between the lanes.
The contest,
9 ballots on every slice but one (8),
gave 12 to translate,
2 to repair,
2 to neither;
the closest splits 4/4/1 on slice 5,
4/3/1 on slice 12 and 3/2/4 on slice 13.
The consolidation ran all 16 contested slices on 9 of 9
(two rounds on 8 of 9):
7 consolidated,
3 `slate-endorsed-standing`,
5 `gate-kept-standing`,
1 `slate-unjudged-standing`;
18 naturalness-review rounds,
every one 9 of 9 heard;
2 slices recorded `standing text lacks contest endorsement and remains retryable`.
The sixth pass's consolidation had run short of quorum from slice 3;
this one never did.

WHAT IT COST.
Synthetic's week went from 5.83 percent to 0 at 12:25 UTC
(615 calls);
at 12:26:22 UTC Synthetic answered HTTP 429 with its meter reading dry,
was held out 300 s and the call routed on.
Hyper's balance went from 250 to 0 at 12:45 UTC
(898 calls).
Bedrock 198.71 to 198.39 USD
(463 calls).
OpenRouter 200.01 to 199.56 USD:
33 calls,
every one after 12:45:53 UTC when Hyper's meter read dry,
the first paid OpenRouter calls that were not 402s.
Both went dry inside the consolidation;
the per-chunk reading costs nothing while nothing is held,
no provider named its return,
and the phase finished on Bedrock and OpenRouter.
The floor merged afterwards would not have stopped it,
and Bedrock with OpenRouter reaches every bench at every phase
(measured on the merged build:
no shortfall clause and no floor clause under `synthetic=dry hyper=dry`).
One entry of 18 slices consumed the remainder of a Synthetic week and a Hyper day.

WHAT THE WARNINGS SAID:
30 `abandoned after quorum, voice lost`
(`hf:Qwen/Qwen3.8-27B` 15 across select,
panel,
consolidate-gate,
produceConsolidations,
critic and polish-gate;
`hf:zai-org/GLM-5.3-Flash` 6;
`hf:moonshotai/Kimi-K3` 3;
`glm-5.3` 3;
`minimax-m3` 1),
8 `hf:openai/gpt-oss-120b` schema-mismatch,
1 select recovery round,
2 stream cuts retried,
1 `translate-refused-declared-name`.

THE MECHANICAL READING (`read-page.mjs`,
calibrated on the eighth pass's recorded page first):
196 lines against the archive's 158 and the sixth pass's 195;
176 lines differ from the archive,
122 from the sixth pass's page;
headings 3 of 3,
comments 3,
`PhotoScroll` kept,
0 straight apostrophes in words (5 curly against the archive's 8),
0 straight double quotes (14 curly against 18),
2 three-dot ellipses and 0 unicode
(archive 0 and 0;
the source's one `……` rendered twice around `Though... many of those journeys were ones she took alone...`),
both footnote markers and both definitions,
every refusal grep 0,
CJK punctuation and runs equal to the archive's (the comments and the Japanese quote).
The details block carries the Japanese and English lines as the archive does,
where the sixth pass's page had kept the source's Chinese lines beside the English.
One em-dash on the page,
none on the archive page;
38 of 92 archive pages carry 228 of them,
so it is within the archive's conventions.
A run of three blank lines between the letter's blockquote and the closing sentence,
where source,
archive and sixth pass have one:
a seam artefact that renders the same.
No defect class on this page.

THE NAME,
the sixteenth thing found,
a question and not a defect.
The archive's front matter reads `name: Hanasaka`,
alias `Kagurazaka Hanasaka, Hakureico`;
the source's,
`name: 神楽坂千歌`,
alias `千歌, Hanasaka, Hakureico`.
The translate lane wrote `name: Kagurazaka Chika`,
alias `Kagurazaka Chika, Chika, Hanasaka, Hakureico`;
the repair lane kept the archive (`incumbent-fallback`);
the contest gave the slice to translate 6 ballots to 3,
the three reasoning that the declared names settle the visible name as Hanasaka,
the six that 千歌 is Chika and the alias should carry the given name;
the consolidation slate endorsed it 9 of 9.
In the body the repair lane's judges chose Chika for the IKEA sentence 8 of 8 at 11:41 UTC,
the consolidation's refine stage chose it again,
and the guard refused it at 12:28 UTC:
`translate-refused-declared-name (slice 4: archive text carries "Hanasaka" and the replacement does not;`
`keeping the archive text)`;
the letter's signature stays `Kagurazaka Hanasaka`.
Both are the rules as written:
`document-preparation.ts` takes the declared forms from the target's front matter
("target-authoritative identity forms guards preserve wherever archive body already carries them"),
and `translate-slice.ts` exempts the front-matter slice from that guard
("front matter is where declarations themselves are corrected from source,
so protecting target values there would make metadata unrepairable").
So the page names her Kagurazaka Chika at the top and Kagurazaka Hanasaka at the letter's foot,
and no rule reads both.
Recorded in the readiness signal for the owner.

THE FLOOR MERGED.
After the tally,
`translation-repair-class15` was rebased onto `aa133492d` and fast-forwarded:
`d74ef4a43` (the floor) and `977c242c1` (the names-only inventory);
built in the main worktree,
954 `PASS` and 0 `FAIL`;
the throwaway worktree,
its local branch and its remote branch removed.

### The front-matter rule, the sixteenth thing decided and built in a throwaway, 13:45 UTC

The owner answered the name question with a question:
"We're not supposed to change front matter though?"
Measured before answering:
the front matter is slice zero,
written by the translate lane from the source
(the repair lane has always kept the archive's,
`frontMatterRepairOutcome`),
judged at the contest and endorsed or consolidated like every other slice,
and the guard checks structure only (the 2026-09-02 decision,
on the owner's "why are we caring about metadata being different vs Chinese source at all?");
13 of the 127 pages shipped under `~/temp/agent` changed the archive's front matter,
9 of the last 10 read pages among them,
and the read-page checks never diffed it,
so TLL1122's `desc` "Lulu loves you!",
yulianNyanner's "In the end,
we heard her voice."
and Hangmster's "They had brought us laughter."
were rewritten on pages read as "no class found".
The owner chose,
of three,
"publish the archive's front matter as is;
render it only where the archive never translated it"
(`doc/decision/translation-repair-front-matter-guard.md`,
addendum 2026-09-08).

BUILT ON `translation-repair-front-matter` in `~/temp/agent/tr-frontmatter-20260908` (`20e5135a6` on `5462257b4`),
since the E2B calibration runs on the main worktree's build and the kill-and-relaunch rule holds for it too;
merged after the calibration's standing is printed.
The shape:
`corpus-run/archive-front-matter.ts` decides `archiveFrontMatterStands` from the two documents
(the archive stands unless it shows the directory id while the source names the person and no 2026-09-07 clause makes
the id stand),
`pass-prepare.ts` decides it once per entry and logs `FRONT MATTER entry=<id> authority=<archive|rendered>`,
`prepareDocumentPair` makes no metadata slice under `frontMatterAuthority: 'archive'` and marks the preparation,
`assertFrontMatterComplete` recomputes the answer and refuses `archive-front-matter` for a changed page or a rendered slice,
and artifact generation eleven records `frontMatterAuthority` in its preparation,
which the rebuild reads off the file.
`namesDirectoryId` moved beside `directoryIdNameStands`,
and the artifact reader's two field helpers moved to `artifact-two-lane-read-fields.ts` to stay under the file budget.
Types and oxlint clean;
955 `PASS` and 0 `FAIL`;
guards shown to fail first (predicate 4,
7 and 2;
preparation 2,
2 and 2;
guard branch 7 and 2),
restored 0.

THE CENSUS,
measured on the merged build's predicate over the 92 pinned archives:
every one stands.
23 name their directory,
8 because the source does too,
and the other 15 all stand by the clauses of 2026-09-07.
So the lanes will render no front matter on this corpus,
every page from here carries the archive's metadata byte for byte,
and the mechanical reading gains a check for exactly that.
The ninth pass's page,
re-read under the rule,
would carry `name: Hanasaka` and the archive's alias,
and the body would agree with it.
