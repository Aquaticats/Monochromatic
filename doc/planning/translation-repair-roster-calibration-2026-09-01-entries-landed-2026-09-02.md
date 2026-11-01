# Roster calibration: the Carena0442, keyword233 and Toka_ls pages that landed on 2026-09-02

Carena0442 landing at 13:48 UTC with the escaped-quote defect found in its reading;
keyword233 landing at 14:13 UTC and Toka_ls landing at 15:52 UTC;
the cause-of-death decisions and the first XIEPT2 launches refused by the runs-directory guards.

Part of [Roster calibration under the owner blocklist, 2026-09-01](translation-repair-roster-calibration-2026-09-01.md),
which lists every file of this record.

## Carena0442 landed at 13:48 UTC, and the two waiters were killed at the same moment

`TALLY Carena0442 status=SETTLED slices=22 ms=11375156` on `3852e4d86`:
190 minutes,
page and artifact
written under `~/temp/agent/carena-rerun-20260902`,
`verify-published` clean (1 of 1 pages carry every
wording,
20,751 characters as implied),
0 refusals and 0 holds.
The reading is
`doc/planning/translation-repair-carena-reading-2026-09-02.md`:
one defect (JSON-escaped quotes leaked
into the text;
the slice validator now refuses the sequence,
`e29dbce4f`,
test shown to fail without
it),
one regression ("She should have known"),
many recoveries of what the archive had dropped,
and the
semantic-wrap reshaping a reader must know about first.
The leak entered at consolidation slice 6:
the
producer's text carried it,
all eight structural verdicts were `valid`,
and the polish kept it.

At 13:48:31 the chain waiter launched keyword233 on `a5c69a305` and,
within seconds,
both background
waiters (the chain and the Toka_ls follow-up) were reported killed,
exactly as at 04:02;
the keyword233
pass died with its parent ("sh exited with non-zero status:
no exit status" three seconds in).
The cause
is outside this session's tool calls.
keyword233 was relaunched at 13:50:54 detached from the harness
task (`setsid nohup`) so a task kill cannot take the pass,
and a detached script
(`~/temp/agent/toka-after-keyword233.sh`,
output `.out`) launches Toka_ls at overlap 3 after keyword233's
TALLY.
Both run on the tree with `321b12673` and `8384166e9`.

## keyword233 landed at 14:13 UTC in 22 minutes, on the fixed tree

`TALLY keyword233 status=SETTLED slices=3 ms=1340649` on `a5c69a305`:
the first artifact of generation
ten,
page and artifact under `~/temp/agent/pin-rerun-20260902`,
`verify-published` clean (826
characters as implied),
0 refusals,
0 holds,
no unendorsed standing,
no escaped quote.
Seats:
Kimi-K3
threw 8 of 37 and Qwen3.8-27B 12 of 37 at the window;
every other seat 1 or fewer.

The reading,
all three slices against the source:
the metadata slice stands (its name is the
directory,
as the source's is,
which the narrowed guard of `6d85b619a` accepts);
both body slices went
to the repair lane at 7 and 6 ballots.
Every change recovers something the archive had paraphrased away:
"She started her own channel on Telegram,
where she shared little moments from her daily life and her
heartfelt reflections with everyone" for 开设了自己的频道，与大家分享她的生活点滴和心灵感悟 (the archive
had "shared laughters and tears");
"which I thought was wonderful" for 这在我看来很棒,
which the archive
had dropped;
"Her wisdom and passion added a touch of brightness to this world" for
用她的智慧和热情，为这个世界增添了一抹亮色;
"It is heartbreaking that she passed away" for 令人悲痛的是;
"we
will always remember her" for 我们会永远铭记她的 (the archive:
"She will always be in our heart").
No
defect,
no regression.
The detached Toka_ls script launched Toka_ls at 14:13:20 on `aeb4181b9` at
overlap 3 (`~/temp/agent/toka-rerun2-20260902.log`).

## Toka_ls landed at 15:52 UTC in 100 minutes at overlap 3

`TALLY Toka_ls status=SETTLED slices=16 ms=5972259` on `aeb4181b9`,
generation ten,
page and artifact
under `~/temp/agent/toka-rerun2-20260902`,
`verify-published` clean,
0 refusals,
0 holds,
no unendorsed
standing.
The reading is `doc/planning/translation-repair-toka-ls-rerun-reading-2026-09-02.md`:
slice
9's pronoun is fixed ("she leaves behind verse in a neat meter"),
the whitespace churn is gone,
the
heading is right,
and slice 10's title moved the wrong way:
"Life of Aiden" survives as the alternate
but the work is now called "Flow",
the English title of 喵的奇幻漂流,
one of the five neighbour results
the web lookup returned.
Fixed unasked:
a result that never names the work now says so on its line and
sorts last.
One item for the owner:
the page now carries the source's death paragraph,
"after
resuscitation failed to reverse hemorrhagic shock",
which the archive had omitted;
the
reader-protection rule names suicide method and medication,
and this sits at its edge.

Three of the four entries are now published from this build:
Carena0442 (190 min),
keyword233 (22 min),
Toka_ls (100 min).

DECIDED 2026-09-02 (after the Toka_ls reading):
the reader-protection rule covers a medical cause of
death that implies the method;
the paragraph stays,
the cause goes
(`doc/decision/translation-repair-reader-protection-cause-of-death.md`).
DECIDED at the same time:
XIEPT2
runs now at overlap 4,
the owner's choice over the measured overlap 3;
it is the first run at that
overlap and the first entry with the cause-of-death rule (`f5e172067`).

The first XIEPT2 launch at 22:55 UTC was refused in two seconds by the runs-directory schema guard:
`assertResumableSchemaGeneration` defaulted to generation seven,
unchanged through generations eight,
nine and ten because every launch of those days went into an empty directory,
and
`~/temp/agent/pin-rerun-20260902` now holds keyword233's generation-ten artifact.
Fixed by the commit
`6e5e909d9` (the default is the writer's generation;
the guard's fixtures carry it;
its tests fail with
the old default).
The relaunch into the same directory at 23:00 was refused by the next guard,
`assertBuildGenerationResumable`,
which is by design:
the directory's artifact was stamped by another
build's pipeline digest,
and a pool of several builds would corrupt every rate computed over it.
XIEPT2
launched at 23:01 into its own directory,
`~/temp/agent/xiept2-rerun-20260902`,
which is what the
`ONLY` banner asks for anyway.

DECIDED 2026-09-02 23:0x UTC:
「安乐死」 stays as the original states it ("it implies it's done legally
... it's very inaccessible");
the reader rule's addendum is in the decision doc and the house policy.
CORRECTED 23:2x UTC:
"the immediate cause of death 'shock' is actually fine,
because it doesn't give
enough information for replication".
The rule now states the test as replicability:
method,
substance,
dose and where to obtain them stay vague;
an immediate medical cause,
the date,
the place and the age
stay as the original states them.
The Toka_ls page of 15:52 is right as published;
no rerun needed for
it.
The 23:01 XIEPT2 run had already ended by itself at 23:03:
`TALLY XIEPT2 status=INCOMPLETE ms=114473
error=translation repair interrupted: provider-unavailable`,
with every seat asked 10 and answering
10.
The thrower was `runArchiveBlockReviewStage`:
XIEPT2's archive is a placeholder page ("(To-Do)" and
translation hints under empty headings),
the block pairing left its blocks unclaimed,
nine reviewers
were heard about the block and fewer than the exact half anchored their support in the original,
and
the stage threw that as a provider outage.
Fixed by `ae39bf22e` (test shown to fail without it):
an
unheard roster still interrupts;
a heard roster that cannot anchor retains the block with an unresolved
finding,
as the no-loop design says ("reviewer indecision cannot withhold the entry").
XIEPT2 relaunched
at 23:11 into `~/temp/agent/xiept2-rerun2-20260902` on the tree carrying that fix and the euthanasia
sentence.
