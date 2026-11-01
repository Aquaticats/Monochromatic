# OpenRouter and the corpus passes: 2026-09-16, 15:58 to 22:58 UTC

What was launched,
read and decided on 2026-09-16 15:58 to 22:58 UTC.
Entries:
hulicaijia5,
Mio20,
Mio21,
Mio22,
Mio23,
Mio24.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## hulicaijia5 launches on the tip, 2026-09-16, 15:58 UTC

Frozen `1cfd9d2fe` (code tip `099b69787`,
the model-card refactor,
value-identical to `a88f52d5b`),
pass pid `1470018`,
runs `~/temp/agent/hulicaijia5-20260916`,
log `~/temp/agent/hulicaijia5-20260916.log`.
Meters at launch:
Synthetic 62.7 percent,
Hyper 0 (dry until the 07:31 UTC reset),
Bedrock 184.95 USD,
OpenRouter 264.76 USD.
Read first the three shapes no read page has met:
the inline ruby annotation,
the Sakura component and the footnote;
then the archive block review
(its 2026-09-09 INCOMPLETE was class twenty-six),
then the seven steps and the three checks.

## Class thirty-five, 2026-09-16, 16:43 UTC: the sheets delete accurate detail the translator took from a cited page

The owner asked about the Mio19 wording note
("The archive's 'older sister who is also trans' lost the clause on source authority"):
"It is true though?
And the Chinese source does imply that,
if I remember correctly?"
Measured:
the Chinese page says 有一个姐姐 in all seven of its revisions
(`git log --follow` on the corpus),
the English clause arrived in the translation commit `c7cc3259`,
and the blog the page cites
(`stblog.penclub.club/posts/InMemoryOfMio/`,
fetched from its archived copy)
says 「Mio 的姐姐也是 MtF。她大我们八岁，在小学五年级时就第一次穿了女装」.
True,
and stated only by the reference.

Four critics filed the clause as `accuracy/addition` on absence alone
("not present in the source text"),
the panel supported all four,
the editor deleted it,
against the rule both sheets already carry
(the critic sheet:
accurate detail a translator added is not an addition,
report it only when wrong;
the adjudication sheet:
vote unsupported on a claim whose whole case is that the ORIGINAL does not carry it).
Measured over the repair lane's shipped addition edits:
Mio19 20 of 24,
Mio16 14 of 24,
Mio12 9 of 22,
hakureico23 11 of 12,
noname3 7 of 9;
among Mio19's twenty the sister,
the backpacks in `photo5`,
"no one was hurt",
"in mainland China",
"lucky" and the closing farewell.

Four options put to the owner
(only what the source contradicts;
facts stand and wording may go;
feed the cited reference to the critics;
keep source-only authority),
ranked A > C > D > B.
The owner chose the third:
"3,
and we have Exa.
Such Exa fetched results need to be cached too to avoid hitting reference links too much",
and while it was built:
"Cache Exa results semi-permanently on disk since the reference links rarely change their content."
Recorded in
[the cited-references decision](../decision/translation-repair-cited-references-2026-09-16.md).

The fix (`dff8c91d2`):
every link the original writes
(its own site and people's profiles left out,
at most eight)
is bought once through the Exa contents endpoint
(4,000 characters of page text)
into `~/.cache/translation-repair/lookup/reference/`,
keyed by the url's digest and never expiring,
and a `CITED REFERENCES, EVIDENCE ONLY` block with its rule rides after the pair on the critic sheet
and after the claims on the panel sheet;
the repair slice key folds it in only where the original links somewhere.
Verified live on the Mio source
(the blog carries 姐姐也是 in its block,
the archived copy timed out and is cached as such,
a GitHub user page and a bilibili space came back as chrome and are now excluded),
and the three guard suites fail with the block disabled.
Measured over the corpus:
59 of 92 originals link somewhere,
116 links.

`hulicaijia5` was killed at 16:43 UTC under always-kill-and-relaunch,
45 minutes in
(Bedrock 184.95 to 184.74 USD,
OpenRouter 264.76 to 264.37 USD);
it had reached the lanes and shipped nothing.

## Mio20 launches on class thirty-five, 2026-09-16, 16:43 UTC

Frozen `dff8c91d2`,
pass pid `1551070`,
runs `~/temp/agent/Mio20-20260916`,
log `~/temp/agent/Mio20-20260916.log`.
Meters at launch:
Synthetic 58.3 percent,
Hyper 0 (dry until the 07:31 UTC reset),
Bedrock 184.74 USD,
OpenRouter 264.37 USD.
The log opens with `REFERENCES cited=2 cached=2 bought=0`:
the blog read back from the cache at 2,729 characters,
the archived copy as its cached failure,
the two profile links never asked.
Read first whether `She has an older sister who is also trans` ships,
then the addition claims that shipped an edit against Mio19's 20 of 24
(`lanes.repair.result.issues` in `artifacts/Mio.json`),
then the seven steps and the three checks.

## Mio20 read, 2026-09-16, 18:44 UTC: the clause the panel now keeps is lost one stage later

Frozen `dff8c91d2`,
SETTLED in 121 minutes,
18 slices,
78 repair issues (45 accepted,
33 rejected),
selection contested,
1,278 seats asked.
Spend:
Bedrock 184.74 to 184.63 USD,
OpenRouter 264.37 to 263.97 USD,
Synthetic weekly 58.3 to 47.7 percent
(and 37.0 at the next launch twenty minutes later with nothing of mine running:
the owner was using DeepSeek V4.1 Flash on Synthetic for other work,
so Synthetic meter moves between passes are not pipeline spend),
Hyper dry throughout.
The log opened `REFERENCES cited=2 cached=2 bought=0`.

The page still says `She has an older sister.`
Traced through `artifacts/Mio.json`:

- The panel rejected all three addition claims on the clause,
  3 unsupported to 2 supported
  (Qwen3.8-27B,
  Kimi-K3 and minimax-m3 unsupported;
  gpt-oss-120b and gemma-4-26b-a4b supported),
  against Mio19's 4 to 1 the other way.
  Addition claims by adjudicated status:
  Mio20 10 rejected and 11 accepted of 21;
  Mio19 6 rejected and 30 accepted of 36.
  Class thirty-five did what it was built to do.
- The repair lane's final text for slice 2 kept
  `she has an older sister who is also trans`.
- The lane contest lost it:
  three of five judges named the repair candidate unsupported for the clause
  ("incorrectly states the sister is trans"),
  one chose translate for keeping 生前 while noting it "drops the archive's 'who is also trans' clause",
  and one voted neither with the right reading
  ("the archive-carried detail that the older sister is also trans,
  which the Chinese does not contradict and so must be kept").
  Translate won 3 to 0 with 2 neither.
- The consolidation wrote from the translate candidate and its ballots
  and shipped the sentence without the clause,
  five gate judges valid.

Class thirty-six:
the lane contest,
the consolidation writer,
the consolidation gate and the translate judge
(which also judges consolidation slates)
never see the cited references;
only the critic and panel sheets did.

The rest of the reading is clean and,
where it differs from Mio19,
better:
the archive's own wording survives where Mio19 rewrote it
(`musculine`,
`Wechat`,
`These are the last text messages she sent:`,
`> . . .`,
the multi-sentence lines of the Bei Yan Yun Yi paragraph and of the last chat block are all the archive's),
`In mainland China` stays where Mio19 deleted it as an addition
(the archive says `In China mainland`),
and the citation title is `Beiyan Yunyi's blog · In Memory of Mio` with no Chinese gloss.
Front matter equals the archive;
two headings both sides;
six photo scrolls;
no straight apostrophes or quotes;
four unicode ellipses both sides;
no footnotes;
no forbidden runs;
117 lines differ from the archive,
143 from Mio12,
188 from Mio19.
Seats:
wide 8,
select 8,
late 9,
slate 9,
translators 8,
readers 6,
writers 9 on three wet providers;
no holds,
no payment refusals,
no quorum shortfall.
Repair changed 11 slices,
translate 12,
the page 10.
The repair lane dropped 生前 in slice 2
("before her passing"),
which the contest judges caught and the consolidation restored;
noted,
not a class.

## Class thirty-six, 2026-09-16, 19:03 UTC: the candidate sheets read the cited references too

`20a7272dc`.
`referenceContext` from the prepared pair now reaches
`runPassContest` and the lane contest subject
(block after the candidates,
before the damage claims),
`runPassConsolidation` and the consolidation subject
(writer sheet block between the candidates and the findings;
the settle context's identity object carries it to the slate judge and the gate),
and the translate slice input through the stage and the round input into `judgeTranslateSlate`
(one labelled evidence entry after the declared names).
A candidate-sheet rule (`CITED_REFERENCE_CANDIDATE_RULE`) sits beside the critic rule:
a detail the archive rendering or a candidate carries that a cited page states
is never counted unsupported and never a reason to prefer the rendering that drops it;
the references never license adding,
never outrank the original,
never license a finding elsewhere.
The three run shapes
(`laneContestRunShape`,
`consolidateRunShape`,
`translateRunShape`)
fold the block in only when non-empty,
so the 33 entries that cite nothing key as before.
Guards on the three sheets,
the translate judge,
the three keys and the rule module;
all eight shown red with the block or the folds disabled
(`~/temp/agent/red-c36-20260916.log`),
green restored.
Build,
oxlint,
types and the unit suite green.

## Mio21 launches on class thirty-six, 2026-09-16, 19:04 UTC

Frozen `20a7272dc`,
pass pid `1635211`,
runs `~/temp/agent/Mio21-20260916`,
log `~/temp/agent/Mio21-20260916.log`.
Meters at launch:
Synthetic 37.0 percent,
Hyper dry,
Bedrock 184.63 USD,
OpenRouter 263.97 USD.
The log opens `REFERENCES cited=2 cached=2 bought=0`.
Read first whether the sister clause ships,
then the slice 2 contest ballots and consolidation,
then the addition claims by status against Mio20's 10 rejected of 21,
then the seven steps and the three checks.

## Mio21 read, 2026-09-16, 20:40 UTC: the clause ships by the archive standing, not by judgment

Frozen `20a7272dc`,
SETTLED in 87 minutes,
17 slices,
10 changed,
1,280 seats asked,
selection contested.
Spend:
Bedrock 184.63 to 184.34 USD (0.29 logged),
OpenRouter 263.97 to 262.62 USD (0.50 logged against 1.35 on the meter;
Mio20 logged 0.46 against 0.40,
so 0.85 USD of this pass is unexplained),
Hyper dry,
Synthetic wet at launch and dry from the translate lane on
(the owner was using it too),
so Kimi-K3 was withheld from the translate lane,
the lane contest and the consolidation,
and the checker and introduced-defect-probe rounds closed on two voices sixteen times
("short of quorum,
reachable 1 of 3").

The page carries `She has an older sister who is also trans.`,
but not because anything judged it accurate.
The panel supported the addition claim 3 to 2
(minimax-m3,
gemma-4-26b-a4b and gemma-4-e2b supported;
deepseek-v4.1-flash and mercury-2.5 unsupported;
Mio20 rejected it 3 to 2),
the repair lane dropped the clause,
the translate lane never had it,
the contest settled on neither
(one neither,
two repair,
two translate;
one judge:
"the archive rendering invents the sister being trans"),
the consolidation slate was declined,
and slice 2 shipped the archive unchanged.
Addition claims accepted 23,
rejected 5,
of 28
(Mio20:
11 and 10 of 21).
The references were on every sheet:
the judges' own replies cite them,
four select judges writing
"this is true per the cited reference ... but is NOT stated in the ORIGINAL passage,
so it is an addition that violates faithfulness"
and "background context not in the ORIGINAL passage".
The sheet's addition category outranks the rule block beside the references.
Class thirty-seven.

The first chat block was revised by the archive block review
(candidate 4 from minimax-m3 won 3.5 of 5;
in Mio20 the judges tied at weight 1 and the archive stood):
the revision restores the pat and the `mtf` question the transcript carries,
corrects the archive's `musculine`,
and drops the "WeChat post" line the transcript does not carry.
It shipped with 14 straight apostrophes inside words and 10 straight double quotes
on a page whose archive has none
(Mio20:
none):
the block review never called the typography restoration.
Class thirty-eight.

The rest read as Mio20 did:
headings 2 of 2,
no comments,
six PhotoScroll lines,
four unicode ellipses on four,
no footnotes,
no Chinese punctuation,
the one CJK run and the one refusal-pattern hit are the archive's own
(`春の芽工作室`,
"I can't delay it forever"),
no code fence,
no errors,
no holds,
no payment refusals.
202 lines differ from the archive
(Mio20:
117;
the revised chat block accounts for the rise).
The Mio20 note on `生前` in slice 2 stands
(repair "during her lifetime",
translate "before her death").

## Class thirty-eight, 2026-09-16, 20:46 UTC: the block review restores the archive's quote style

`6b853e5ce`.
Every revision candidate the archive block review builds is restored against the block it replaces
with the whole archive as convention before the judges read it,
so the selection and the gate read the shipped bytes.
Class seven restored the translate and consolidation candidates;
the block review was added later (class twenty-five) and never called `restoreTypography`.
Guard shown red without the fix.

## Class thirty-seven, 2026-09-16, 20:58 UTC: reference-backed archive details are attested word for word and screened before the panel

`35e9a0785`,
whitespace fix `887e5262c`.
Prose beside the references does not survive the sheets' own addition category,
so the mechanism is now partly mechanical.
At preparation,
when the original cites a page,
the bench is asked one focused question
(`reference-attest-wire.ts`):
which details the ARCHIVE RENDERING states that the ORIGINAL does not but a CITED REFERENCE does,
each as a verbatim archive quote plus a verbatim reference quote.
Both quotes are checked as substrings with every whitespace character removed
(`reference-attest-match.ts`;
Mio22 showed a voice spacing the reference's Chinese around its Latin tokens),
items from distinct voices whose archive quotes overlap merge into one detail,
and a detail is kept when at least half the heard voices gave it
(`rosterQuorumSize`).
The details ride the prepared pair as `attestedDetails`
and as `- attested: ...` lines under the references in `referenceContext`,
so every sheet and every cache key sees them;
both rule constants name the attested lines.
In the repair chunk
(`reference-attest-claims.ts`)
an `accuracy/addition` claim whose archive-side quote overlaps an attested quote
is recorded rejected with an empty tally and never reaches the panel or the editor;
the chunk logs `addition claim rejected before the panel, reference-attested`.
The log prints `ATTESTED heard=H answered=A verified=V needed=N details=D`,
one `ATTESTED item <seat>` line per verified item,
and one `- attested:` line per detail.
Four guards shown red with the verification,
the quorum filter and the screen disabled.

## Mio22 launches on class thirty-seven, 2026-09-16, 21:00 UTC, killed at 21:01 UTC

Frozen `35e9a0785`,
pid 1765887.
Its attestation read `ATTESTED heard=6 answered=4 verified=3 needed=3 details=0`:
three voices attested the sister clause against the blog
(`Mio的姐姐也是MtF。`),
one of them writing `Mio 的姐姐也是 MtF。`,
which the folding check rejected,
one voice attested the wrong archive line
("gentle big sister"),
two attested nothing.
Two voices against a quorum of three,
nothing attested.
Killed under ALWAYS KILL AND RELAUNCH once `887e5262c` landed.

## Mio23 launches on class thirty-seven, 2026-09-16, 21:05 UTC

Frozen `887e5262c`,
pid 1793041,
runs `~/temp/agent/Mio23-20260916`,
log `~/temp/agent/Mio23-20260916.log`.
Read first the `ATTESTED` lines
(expect the sister detail with three or more voices),
then slice 2's rejected issue with the empty tally and the chunk's reference-attested finding,
then whether the repair lane keeps the clause and what the contest and consolidation do,
then the first chat block's quote style,
then the seven steps and the three checks.

## Mio24 launches on class thirty-nine, 2026-09-16, 22:58 UTC

Frozen `27109ca8b`,
pid 2354397,
runs `~/temp/agent/Mio24-20260916`,
log `~/temp/agent/Mio24-20260916.log`,
Synthetic and Hyper dry.
Read first the `ATTESTED` lines
(expect the sister detail again),
then slice 2's translate lane text
(`lanes.translate.result.sliceTexts` by index;
expect the sister clause carried),
then the contest ballots and what ships,
then the first chat block,
then the seven steps and the three checks.

## Class thirty-nine, 2026-09-16, 22:55 UTC: the translate writer never sees the attested details

The translate writer sheet
(`buildTranslateMessages`)
shows the ORIGINAL,
the EXISTING TRANSLATION as evidence,
the pictures and the declared names,
and nothing about the references:
`referenceContext` reaches the translate judge
(class thirty-six)
and never the writer.
So a translate candidate is written from the ORIGINAL alone,
and an archive detail the ORIGINAL does not state is dropped by construction,
attested or not.
The contest then compares whole slices:
when the translate lane reads better elsewhere,
it wins and the page loses the detail,
whatever the candidate-sheet rule says about dropping it.

`27109ca8b`:
`attestedLines`
(the `- attested:` lines of the prepared pair's `attestedDetails`)
travel from `translateSliceInput` through `runTranslateStage`,
`runTranslateRepairs` and `produceTranslateSlate` into the writer sheet,
which carries them as an `ATTESTED DETAILS` block after the EXISTING TRANSLATION
and adds `TRANSLATE_ATTESTED_RULE` above the reply rule:
each is accurate detail the translator took from the ORIGINAL's own references,
carry every one,
the ORIGINAL not stating it is no reason to drop it,
and they license nothing else.
The lines are folded into `translateRunShape` only when there are any,
so Mio23's cached translate slices are not served back to Mio24
and every entry that attests nothing keeps its key.
Guards red first at `e9f90c485`:
the sheet carries the block,
the line and the rule when given and nothing when not;
the run shape separates the two.
Build,
lint,
types and the unit suite green
(`~/temp/agent/check-package-c39b-20260916.log`).

## Mio23 read, 2026-09-16, 22:40 UTC: the attestation holds and the translate lane wins without the clause

Frozen `887e5262c`,
SETTLED in 94 minutes,
18 slices,
repair 49 issues,
30 accepted,
selection contested,
page changed 12.
Bedrock 0.43 USD,
OpenRouter 0.67 USD by meter against 0.70 logged
(no gap this time;
Mio21's 0.85 gap did not recur),
Synthetic and Hyper dry throughout.

The attestation:
`ATTESTED heard=6 answered=10 verified=9 needed=3 details=1`.
Five of six voices
(gpt-oss-120b,
minimax,
gemma-4-26b,
deepseek-v4.1-flash,
mercury)
attested "She has an older sister who is also trans."
against the blog's "Mio的姐姐也是MtF。";
minimax's other four items
(the university,
the driving,
the programming,
the accident)
verified word for word and fell to the quorum as one voice each.
One item was discarded unverified.

Slice 2:
the critic filed two mistranslation claims
("because of",
"accepted" for 就读)
and no addition claim on the sister,
so the pre-panel screen had nothing to reject
(no `reference-attested` finding anywhere in the log).
The repair lane kept the clause.
The translate lane rendered "Her father is a high school teacher,
her mother is a housewife,
and she has an older sister."
The contest chose translate 3 to 2.
The two repair ballots name the attested detail as their reason
("translate leaves out the confirmed detail that Mio's older sister is also trans").
Of the three translate ballots,
one names the 985,
the local university and "before she passed away" that repair dropped
and adds "though it wrongly keeps the reference-attested 'also trans' sister";
one names the housewife and the study context;
one lists "who is also trans" under dropped in its raw form,
and votes translate anyway.
The consolidation slate was declined,
translate stood,
the polish gate chose neither,
and the page reads "and she has an older sister."
The mechanism is class thirty-nine
(the heading dated 2026-09-16 22:55 UTC):
the translate writer never sees the attested lines,
so its candidate cannot carry the detail,
and the contest is a whole-slice choice.

The first chat block:
the block review's winner drew 1.5 of 5 with two abstentions against a floor of 2.00
and the archive block stood
(Mio21's revision won 3.5 of 5),
so the pat,
the mtf question and the musculine typo stand as the archive has them
and class thirty-eight had no revision to restore quotes on;
the page carries no straight quote anywhere
(`mio21-typo.mjs`:
every slice 0/0).
Slice 3's translate lane added "> hehe~ " for the 嘿嘿~ the archive dropped,
with one trailing space the wrap kept;
the closing poem's two-space hard breaks are the original's own.
Front matter identical to the archive.
Slice 14 declined-standing,
slice 16 consolidated;
the diff against Mio12 is the Mio21 diff again
(the archive's wording kept where Mio19 rewrote it).
