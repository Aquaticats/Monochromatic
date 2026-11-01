# Translation repair history 2026-08-15 to 2026-08-19: the fidelity trial, its three fixtures and the relocation hazard they turned up

The trial that gives the judges a question with a right answer,
the deletion,
insertion and alteration fixtures,
the edit-mark in the deletion arm and the rerun,
what each result rules out about position,
length and source reading,
and the finding that a relocated passage reads as a fabrication to a per-slice judge.

Part of [Translation repair history: 2026-08-15 to 2026-08-19](translation-repair-history-2026-08-15-to-19.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The fidelity trial and the relocation hazard it turned up

### The judges now have a question with a right answer

`#84` asks whether the translate lane's judges can tell a faithful rendering from
a fluent one,
and it could not be answered from any corpus run,
for a reason
worth stating plainly:
IN PRODUCTION NOTHING KNOWS WHICH CANDIDATE IS BETTER.
That is what the judges are being asked.
A run therefore reports how often they
replaced the archive and never how often they were right to.

SO THE ANSWER IS CONSTRUCTED.
A trial takes a real archive slice,
deletes one
whole sentence with `applySeededErrors`,
and puts the two texts on the ballot
`selectBestCandidate` builds.
The deletion is word for word the clean text minus
a sentence,
so it cannot lose on fluency,
register or house style,
only on
coverage,
which is the first criterion the sheet names.
A judge that picks it has
ranked something above saying what the original says.

IT ASKS THE SHEET THAT SHIPS,
not a copy of it.
The task and criteria moved into
`translate-selection-sheet.ts` and both the stage and the trial import them,
so
the measurement cannot drift from the thing measured the first time either is
edited.

FOUR ARRANGEMENTS PER PAIR:
clean text as incumbent and as proposal,
each listed
first and second.
A judge that keeps whatever it is handed scores half,
a judge
that prefers position one scores half,
and only a judge that reads scores four.

A DECLINE IS NOT A HIT,
even in the direction where declining happens to leave
the clean text standing.
The judges did not find the deletion,
they abstained,
and counting that as correct is exactly how `#66`'s silent probe came to look
reliable.

TWO DEFECTS THE TESTS CAUGHT IN THE NEW CODE,
which is what they are for.
`SelectionBallot.best` and `SelectionOutcome.selectedIndex` are BOTH ONE-BASED,
and both were compared against a zero-based array position,
so every correct
choice read as wrong.
`CANDIDATE_NONE` is zero,
so a declining ballot had to be
read before that comparison or it would land in whichever branch was not the
clean one.
The trial now derives the clean text's one-based position once and
compares both readings against it.

WHAT IT DOES NOT ANSWER,
recorded in `#84` rather than implied:
self-preference,
since the fixture is attributed to a composite with no contributors so no ballot
is discounted;
and the HARD case,
a fluent paraphrase that quietly drops a
qualifier.
A deletion is a blunt defect,
so this is a floor:
a roster that cannot
see a missing sentence will not see a missing hedge.

### First reading from the fidelity trial: the judges see a deleted sentence

Sixteen trials over FOUR PAIRS,
one from each of `AmbeR_the_anpa`,
`Arita`,
`Chinatsu_Suzuki` and `CuspariaKLSY`,
four arrangements each:
EVERY TRIAL CHOSE
THE COMPLETE TEXT.
Preserving when the clean text was the incumbent,
replacing
when it was the proposal,
and both with the clean text listed first and second.
Deletions ran 75 to 158 characters.

COUNT THE PAIRS,
NOT THE TRIALS.
Sixteen trials are four independent questions
asked four ways each,
not sixteen independent questions,
because the four
arrangements of a pair share their text.
The earlier eight-trial reading came
from two slices of ONE entry,
which is why it was rerun.

THE BALLOTS SAY MORE THAN THE VERDICTS.
Ninety-five ballots were cast across the
sixteen trials:
66 for the complete text,
3 for the deletion,
26 declining to
choose.
Per judge:

- `hf:moonshotai/Kimi-K3`:
  16 clean,
  0 damaged,
  0 declined
- `hf:zai-org/GLM-5.2`:
  15 clean,
  0 damaged,
  1 declined
- `hf:Qwen/Qwen3.6-27B`:
  15 clean,
  0 damaged,
  1 declined
- `hf:zai-org/GLM-4.7-Flash`:
  10 clean,
  3 damaged,
  2 declined
- `hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4`:
  5 clean,
  0 damaged,
  11 declined
- `hf:openai/gpt-oss-120b`:
  5 clean,
  0 damaged,
  11 declined

EVERY VOTE FOR THE DELETION CAME FROM ONE JUDGE,
`GLM-4.7-Flash`,
and never
enough of them to carry a trial.

TWO JUDGES ABSTAIN ON TWO THIRDS OF TRIALS,
and the reason is legible in what
they wrote:
both hold the pair to the sheet's exact-names criterion,
which the
ARCHIVE ITSELF violates by romanising the handle `AmbeR_the_安帕` to
`AmbeR_the_anpa`,
and refuse both candidates on that ground.
That is a property
of the source slice rather than of the damage,
so on slices like these the
effective roster is four voices and not six.
It is worth carrying into `#93` and
into any quorum decision:
a criterion the archive cannot satisfy silently removes
a third of the panel.

WHAT THAT RULES OUT.
A roster that keeps whatever it is handed scores half,
and
so does one that prefers the first candidate;
the rotation was built to separate
exactly those.
Neither pattern is what came back.

WHAT IT DOES NOT RULE OUT,
and this is a limit of the FIXTURE rather than of the
run:
PREFER THE LONGER TEXT scores sixteen of sixteen here too.
The deletion is a
strict subset of the clean text,
so the complete candidate is the longer one in
every arrangement and rotating the ballot cannot touch that.
Longer-is-better is
not a harmless habit either:
in production it favours a padded fresh rendering
over a tight archive one,
which is the sheet's "nothing added" criterion failing
quietly.

THE ONE PIECE OF EVIDENCE AGAINST A LENGTH HABIT is what the judges wrote.
They
name the missing propositions specifically:
"Candidate 2 completely omits the
first sentence containing the handle,
birth date,
and hometown".
A length rule
cannot produce that sentence.
It is still a SELF-REPORT and reasons can be
written after the fact,
so it is evidence and not proof.

WHAT SETTLES IT IS AN INSERTION FIXTURE,
where the correct answer is the SHORTER
candidate:
splice a sentence from elsewhere in the same document into the clean
text,
so it is in register and fluent and unsupported by this slice's original.
`SeededErrorKind` already carries `insertion`.
A roster that passes deletion and
insertion both cannot be reading length.
That is the next build under `#84`.

ONE LEAK CHANNEL IS ALREADY CLOSED,
checked by dumping a real judge sheet:
the
producer labels do not reach the model.
It sees `CANDIDATE 1` and `CANDIDATE 2`
and nothing saying which is the archive's.
So length is the remaining channel,
which is why the insertion fixture is the whole answer rather than a refinement.

AND THE FIXTURE IS A FLOOR BY CONSTRUCTION.
A deleted sentence is a blunt
coverage defect.
What the lane actually risks is a fluent paraphrase that drops a
qualifier while reading better than the archive,
and nothing here speaks to that.
The reading is:
a roster that could not see a missing sentence would be
disqualified,
and this one is not.

### The insertion fixture, which is what settles the length question

BOTH FIXTURES NOW LIVE IN `src/fidelity-damage.ts`,
and `--damage
deletion|insertion` chooses;
the probe runs both by default and reports per
defect as well as overall,
because a combined figure hides the reading that
matters.
Every trial and every row carries which defect it was built with.

WHAT THE INSERTION IS:
a sentence taken from ANOTHER SLICE OF THE SAME DOCUMENT,
spliced into the clean text after its own longest unique sentence.
Same
translator,
same register,
same subject,
genuinely fluent prose,
and unsupported
by the original this slice shows the judges.
The correct answer is therefore the
SHORTER candidate,
which is the one thing a deletion can never ask.

WHY THE DONOR IS ORDERED RATHER THAN CHOSEN,
and this is worth knowing because
the first version was wrong on the corpus rather than in principle.
Taking
strictly the furthest slice refused FIFTEEN OF SIXTEEN attempts:
the last slice
of a memorial entry is often a credit line,
a short list,
or a single sentence,
and none of those offers a borrowable sentence.
Refusing there would have sampled
only documents that happen to end in prose,
which is a selection rule nobody
chose.
`donorTextsFor` now returns every other slice carrying English,
furthest
first,
and the fixture takes the first that offers a sentence this slice does not
already carry.
Distance is still the preference;
it is no longer a veto.

WHAT THE FIXTURE REFUSES,
each for a reason worth keeping:
a donor sentence the
slice already carries,
since that adds nothing while scoring a judge wrong for
keeping a text that says the same things;
a slice with no sentence long or unique
enough to splice after;
and an entry with no other slice at all.

THE TESTS STATE THE INVERSION rather than describing it:
the deletion leaves the
complete text longer,
the insertion leaves it SHORTER,
and both leave every other
word alone.
Writing them turned up that an insertion anchor carries an EMPTY text
rather than an absent one,
since both members of `DocumentChunk` declare `text`,
so the probe's `cleanText === undefined` branch was unreachable and the donor
helper repeated the same misreading.
Both now rely on the length floor.

### The insertion result: the length habit is ruled out

SIXTEEN TRIALS,
FOUR PAIRS,
the same four entries the deletion spread used,
so
the comparison is within-document.
EVERY TRIAL CHOSE THE COMPLETE TEXT,
which
here is the SHORTER candidate.
A roster preferring length would have scored
sixteen of sixteen on the deletion and ZERO of sixteen here.

THE BALLOTS ARE STRONGER THAN THE TALLY.
Ninety-five ballots:
64 for the complete
text,
31 declining,
and NOT ONE for the text carrying the borrowed sentence.
Every judge that chose at all chose correctly.
Per judge:

- `hf:Qwen/Qwen3.6-27B`:
  16 clean,
  0 damaged,
  0 declined
- `hf:moonshotai/Kimi-K3`:
  15 clean,
  0 damaged,
  1 declined
- `hf:zai-org/GLM-4.7-Flash`:
  14 clean,
  0 damaged,
  1 declined
- `hf:zai-org/GLM-5.2`:
  13 clean,
  0 damaged,
  3 declined
- `hf:openai/gpt-oss-120b`:
  5 clean,
  0 damaged,
  11 declined
- `hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4`:
  1 clean,
  0 damaged,
  15 declined

THE REASONS NAME THE ADDITION AS UNSUPPORTED,
which is the reading that matters:
"adds an entirely fabricated sentence at the end that has no basis in the
original Chinese text",
"appends a completely unrelated,
hallucinated sentence",
"appends a sentence that does not exist anywhere in the original".
28 of the 95
reasons QUOTE THE CHINESE directly while comparing (`占有欲很强`,
`温柔可爱`),
which is evidence the judges consulted the original rather than only weighing the
English against itself.

WHAT THE PAIR OF FIXTURES NOW LICENSES:
the roster is not running a status-quo
reflex,
a position preference,
or a length preference.
Each of those scores half
or zero across the two fixtures,
and the roster scored both in full.

WHAT IT STILL DOES NOT LICENSE,
and this is the honest ceiling:
"verified against
the source" is a stronger claim than the fixtures support.
A judge that never
reads the Chinese but prefers whichever English READS BETTER passes both,
since
the deletion leaves a gap in the argument and the borrowed sentence is a topical
non-sequitur.
The quoted-Chinese reasons are evidence against that reading rather
than proof,
because a reason is written after the choice.
What would settle it is
a fixture whose damaged candidate reads BETTER than the archive's English while
saying something the original does not,
which is the hard case `#84` still
carries.

### The deletion arm was leaving an edit-mark, and the rerun says it did not matter

FOUND BY LOOKING RATHER THAN BY A FAILURE.
A sentence is stored trimmed and prose
separates sentences on both sides,
so cutting the sentence alone leaves BOTH
separators.
Measured on the fixture text:
a mid-paragraph deletion left a DOUBLE
SPACE at the join,
and deleting a trailing paragraph left THREE CONSECUTIVE
NEWLINES.
Either is a typographic edit-mark visible without reading a word of the
original,
which would have let the damaged candidate lose on tidiness while the
run was recorded as a coverage reading.

`src/fidelity-splice.ts` removes the sentence and ONE of the two whitespace runs,
keeping whichever the structure needs:
the trailing run at the end of the text,
the leading one at the start,
the STRONGER one in the middle.
Written beside the
fixture rather than inside `applySeededErrors`,
because the recall benchmark and
the introduced-defect probe are measured against what that shared primitive does
today.

THE ARM WAS RERUN CLEAN,
and the number holds:
16 of 16 again,
over the same four
entries.
92 ballots this time:
60 for the complete text,
2 for the deletion (both
`GLM-4.7-Flash` again),
30 declining,
with three voices lost to the grace period.
So the judges were not reading the join,
which is the one thing the first run
could not say about itself.

TWO OF THE GUARDS I WROTE FOR THE FIX WERE PASSING FOR THE WRONG REASON,
which is
worth recording because it is the same failure GFP exists to catch,
one level in.
`deriveOmissionSeeds` picks the LONGEST sentence,
and in both paragraph fixtures
that sentence sat in the first paragraph,
so the cut landed at the START of the
text and neither test ever produced the middle or trailing join it was named for.
Both passed under a bare cut.
The fixtures now put the longest sentence where the
test needs the cut,
and all three guards fail without the splice.

### The third fixture: a number only the original can adjudicate

WHAT DELETION AND INSERTION CANNOT DO between them is show that the source was
READ.
Both are decidable by an English-only reader with taste,
which is exactly
the reading the two results cannot exclude.

AN ALTERATION CHANGES A NUMBER THE ORIGINAL ALSO STATES.
Digits survive
translation:
a birth year written `2004年` in the Chinese is `2004` in the
English,
so the same run of characters sits on both sides.
Changing it in the
English alone leaves a candidate of the SAME LENGTH,
equally fluent,
equally in
register,
and wrong about a fact that no amount of reading the English reveals.

GROUND TRUTH IS VERIFIED RATHER THAN ASSUMED,
which is what makes this stronger
than a dropped qualifier:
the number is used only when the English states it
exactly once and the Chinese carries it,
and the replacement only when NEITHER
side carries it.
So the clean text is supported by the original and the damaged
text is supported by nothing.

WHAT IT WILL NOT PROVE even if the roster passes:
that judges consult the source
on ordinary slices.
It shows whether they CAN,
on a slice where nothing else can
decide.
That is still the strongest reading available without human grading.

### The alteration result, and the thing it turned up underneath

TWELVE OF SIXTEEN chose the complete text,
over `AmbeR_the_anpa`,
`Arita`,
`Chinatsu_Suzuki` and `Dethelly` (the fourth entry differs from the other two
arms because `CuspariaKLSY` states no number both sides share).
96 ballots:
55
for the complete text,
41 declining,
and AGAIN NOT ONE for the damaged text.

SO NO JUDGE EVER PICKED THE WRONG NUMBER.
Every miss is a roster that declined
rather than a roster that chose wrongly.

THE REASONS ARE THE POINT HERE,
because this is the fixture built to ask whether
the source is read at all:
"Candidate 1 correctly preserves the birth year 2004
from the original",
"Candidate 2 incorrectly alters the birth year from 2004 to
2005,
violating the faithfulness criterion regarding time",
"a direct
faithfulness error on a number/time fact".
They name the number the ORIGINAL
states.
86 of 96 reasons appeal to the original explicitly.

ALL FOUR MISSES ARE ONE SLICE,
`Dethelly/0`,
in all four arrangements,
and the
roster was RIGHT to refuse it.
That slice is where the interesting thing is.

### A relocation reads as a fabrication, and the judges condemn the archive twice

WHAT IS ON THAT SLICE,
read directly rather than taken from the judges.
The
Chinese `## 简介` is one sentence:
Sugar is a girl from Chongqing who came to
Hangzhou in autumn 2023 for university.
The English `## Description` carries that
sentence WITHOUT Chongqing,
plus four sentences about being introverted,
caring
too much about others' feelings,
being too shy to express confusion,
having
nobody to confide in,
and looking intimidated.

THE JUDGES CALLED THAT HALLUCINATED.
They are right about the slice and wrong
about the document:
内向,
焦虑,
倾诉,
困惑 and 在意 all appear in the Chinese,
and
the NEXT Chinese slice is exactly that material.
The English moved it up a
section.
Alignment is not at fault:
24 slices,
zero findings,
every heading
pairs.

THE RATIOS CORROBORATE IT,
which is what makes this evidence rather than a
reading:

- whole document:
  3.06,
  which is ordinary Chinese-to-English expansion
- median slice:
  3.31
- slice 0:
  11.51,
  content moved IN
- slice 1:
  2.08,
  content moved OUT

Two complementary anomalies on adjacent slices is what displacement looks like.
Fabrication would raise one without lowering its neighbour.

WHY THIS MATTERS FAR BEYOND THE TRIAL,
and why it is now `#107`.
Every judge sees
ONE slice pair and nothing else.
Wherever a translator moved material across a
section boundary,
the archive looks like it invented content on one slice and
dropped content on the next,
and the roster will condemn it on BOTH while
preferring a fresh rendering that says only what its own slice's original says.

THAT IS A SYSTEMATIC BIAS AGAINST THE ARCHIVE,
in the direction that matters
most for a memorial archive,
and some unknown share of the three-in-four
replacement rate on the bench may be exactly this rather than the archive being
worse.
Nothing in the pipeline can currently see it.
`#107` carries a free
deterministic probe for it:
per-slice ratios against each document's own median,
looking for adjacent high-low pairs.

AND IT MEANS THE ALTERATION NUMBER IS A FLOOR.
On the twelve trials where the
fixture's premise holds,
the roster was right twelve times;
the four it "failed"
are four where the archive's own English is not a faithful rendering OF THAT
SECTION,
so declining was the correct answer and the trial has no way to score it
as one.
