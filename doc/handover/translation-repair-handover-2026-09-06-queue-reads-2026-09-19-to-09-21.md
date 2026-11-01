# Translation repair handover 2026-09-06: the queue, page reads of 2026-09-19 to 2026-09-21

The oldest page reads of the numbered queue,
from CuspariaKLSY5 back to XingZ619,
with the order of reads and the spend list of the XingZ60 passes.

Part of [Translation repair handover snapshot: 2026-09-06](translation-repair-handover-2026-09-06.md),
which lists every file of this record.

## Page reads of 2026-09-19 to 2026-09-21

CLASS SEVENTY-SIX IS LIVE ON CUSPARIAKLSY5
(the pass log's heading dated 2026-09-21 23:20 UTC):
`CuspariaKLSY5` (frozen `d70087757`) SETTLED at 23:08 UTC in 41 minutes,
0.24 USD by the meters,
one attempt,
369 seats,
17 retry rounds;
the seven lines of the life ship in the past,
the slice 0 contest went to the translate lane 3 of 5
(two ballots cite the house rule's past tense,
none names a rule holding narrative in the present),
no unendorsed standing,
three gate-kept (slices 0,
2,
3),
1 heading,
1 marker,
front matter equal,
no TA,
no error finish,
no double blank line,
the publish weight check silent,
9 abstention lines.
Two bench choices recorded as observations,
not classes:
吃不上饭 ships as "could not afford a meal"
(the ballots faulted repair's "can't afford to eat" on the same reading),
and 起的 on the circle name ships as "was given to her"
where `CuspariaKLSY4` and the archive have her coining it;
the archive's translator gloss
("means fish in clear water",
not in the original)
is dropped as on `CuspariaKLSY4`.
CLASS SEVENTY-SIX WAS FOUND ON CUSPARIAKLSY4
(the pass log's heading dated 2026-09-21 22:40 UTC):
`CuspariaKLSY4` (frozen `c5470163d`) SETTLED at 22:24 UTC in 38 minutes,
0.22 USD by the meters,
class seventy-five live (no double blank line,
the publish weight check silent),
but the seven present-tense lines of the life shipped as the archive has them:
the repair lane left slice 0 unchanged,
the contest split 2 to 2 to 1 with one ballot citing
"the tense rule that holds narrative in the present",
and the unendorsed standing shipped.
That ballot read the judge tail's own sentence
("the tense the text being replaced had already chosen")
against the house rule's past-tense line;
fixed in `d70087757` (both tails in `house-policy.ts` put the house rule's past tense first),
guard red first at `20f344da3`,
lint and types clean,
the full suite red at `d70087757` on the older tense-authority guard,
which asserted the removed sentence and now asserts the new one (`03cc8e448`);
the full suite green at `03cc8e448` (exit 0,
no FAIL line).
CLASS SEVENTY-FIVE IS FIXED
(the pass log's heading dated 2026-09-21 21:50 UTC):
`CuspariaKLSY2` (frozen `574fce2d1`) SETTLED at 21:32 UTC in 36 minutes,
0.20 USD by the meters,
and tells the life in the past tense on every line the archive had in the present
(35 judge reasons cite the house rule),
but shipped a double blank line between the list intro and its list:
the slice's shipped text ended in a newline and `spliceSlices` wrote it verbatim
ahead of the archive's block separator.
Fixed in `c9e267d3a` (`span-edge-match.ts`,
`matchSpanEdges`:
a content replacement carries the replaced span's line-ending edges),
guard red first at `727fbc171`.
The full suite was red at `c9e267d3a` on the publish test
(the page weight counted the lane's newline the splice no longer writes);
fixed in `c5470163d` (`sliceDelta` weighs through `matchSpanEdges`),
guard red first at `01b54f2a9`,
lint and types clean,
the full suite green at `c5470163d` (exit 0,
no FAIL line).
`CuspariaKLSY3` (frozen `c9e267d3a`) was killed at 21:45 UTC six minutes in.
HULICAIJIA9 CONFIRMED THE HEADING EVIDENCE
(the pass log's heading dated 2026-09-21 21:00 UTC):
`hulicaijia9` (frozen `574fce2d1`) SETTLED at 20:53 UTC in 82 minutes on one attempt,
4.10 USD by the meters,
every check as the archive,
and the fifth heading ships as the archive's "Conflict"
where `hulicaijia7` and `hulicaijia8` shipped "Left and Right":
no candidate proposed the literal heading once the identity block listed
`左右 (heading): "Conflict"`,
and the one critic claim against it lost 1 to 3.
`CuspariaKLSY2` (frozen `574fce2d1`,
pid 1637234,
launched 20:56 UTC)
runs to read the tense line against `Cusparia1`:
read the life narrative's tense first
(the archive's present-tense lines that `Cusparia1` kept),
then the seven steps and three checks,
and record it.
THE OWNER ANSWERED THE THREE OBSERVATIONS
(the pass log's heading dated 2026-09-21 19:35 UTC):
headings stay with the judges
("Keep the judges deciding"),
and the archive's heading renderings reach every sheet's identity block as evidence,
aligned by order where both documents carry the same heading count
(`574fce2d1`,
guard `351e68ef8`,
fixtures rekeyed on the marked heading in `8e2b06bc9`);
the em-dashes are left as they are,
observation closed;
the house policy tells the life on these memorial pages in the past tense
("these people are already dead and obviously past tense is better English here";
`574fce2d1`,
guard `0262bf031`).
`hulicaijia9` (frozen `574fce2d1`,
pid 1612708,
launched 2026-09-21 19:31 UTC)
runs to read the heading evidence live;
read it against `hulicaijia8`,
the 左右 heading and the judge reasons citing the heading line first,
then launch `CuspariaKLSY` on `.frozen-dist-574fce2d1` to read the tense line against `Cusparia1`.
The full suite is rerun at `8e2b06bc9`;
read its log before the next fix.
THE QUEUE WAS RUN THROUGH ON THE CURRENT BUILD
(the pass log's heading dated 2026-09-21 19:00 UTC):
`XingZ620` (frozen `364b83557`) SETTLED in 3h59m on one attempt,
10.24 USD by the meters,
every check as the source or the archive,
slower than `XingZ619` only because Synthetic and Hyper are wet and slow.
No pass is running and no class is open.
THREE OBSERVATIONS ARE OWNER QUESTIONS,
each about who decides,
with a recommendation:
(a) the archive's heading wording:
`左右` ships as "Left and Right" over the archive's "Conflict" on `hulicaijia7` and `hulicaijia8`;
the judges weigh the archive's rendering as a candidate and choose the literal one;
recommendation:
restore a heading to the archive's wording wherever the pairing aligns a source heading with an archive heading,
the way class sixty-seven restores contributor names,
since a heading is a title the human translator chose;
(b) a spaced em-dash on a page whose archive carries none
(`zheermao101`,
`hulicaijia8`);
recommendation:
a house-policy line that the page's own dash convention is kept,
judge-enforced,
since replacing a dash deterministically picks a comma or a colon nobody chose;
(c) tense drift across slices on `CuspariaKLSY`
(the archive's present against a lane's past);
recommendation:
leave it,
the house policy already binds tense per passage and no rule can pick the page's tense.
Until the owner answers,
the next work is whatever a new read surfaces;
the four-entry queue can be rerun on any later build the same way.
YINGYING2 IS READ AND XINGZ620 RAN
(the pass log's heading dated 2026-09-21 14:50 UTC):
`yingying2` SETTLED in 25 min on one attempt,
0.04 USD,
every check as the archive.
`XingZ620` (frozen `364b83557`,
pid 1573994,
launched 2026-09-21 14:48 UTC) runs as the queue's last entry;
read it against `XingZ619`.
After it every entry the queue names has run on the current build;
the next work is whatever its read surfaces,
else the open observations
(Cusparia's tense drift,
the spaced em-dashes,
the 左右 heading,
the Bird in a Cage attribution).
SHI_YUMIAOYA7 IS READ AND YINGYING2 RAN
(the pass log's heading dated 2026-09-21 14:25 UTC):
SETTLED in 62 min on one attempt,
0.19 USD by the meters,
the attribution once,
the source's 小药娘 footnote glossed inline under the house rule and its definition withdrawn as an orphan,
which matches the archive;
class seventy-four stands on its guards,
no live refusal yet.
`yingying2` (frozen `364b83557`,
pid 1571244,
launched 2026-09-21 14:21 UTC) runs as the queue's next entry;
read it against `yingying1`,
then launch `XingZ620` on `.frozen-dist-364b83557`.
CLASS SEVENTY-FOUR IS BUILT AND SHI_YUMIAOYA7 RAN
(the pass log's heading dated 2026-09-21 13:25 UTC,
`364b83557`,
guard `f97250977`):
the line guard refuses a rendering that repeats a line more often than the original repeats any,
because `shi_Yumiaoya6` shipped the farewell quote's attribution twice.
`shi_Yumiaoya7` (frozen `364b83557` at `node_modules/.frozen-dist-`,
pid 1557601,
launched 2026-09-21 13:17 UTC) runs to read it live;
then `yingying` and `XingZ60` on `.frozen-dist-364b83557`,
one at a time.
SHI_YUMIAOYA6 IS READ
(same heading):
SETTLED in 58 min on one attempt,
0.18 USD by the meters,
every other check as the archive,
Synthetic and Hyper wet again and slower.
HULICAIJIA8 IS READ
(the pass log's heading dated 2026-09-21 12:15 UTC):
`hulicaijia8` (frozen `155c6aa65`) SETTLED in 59 min on one attempt,
4.90 USD by the meters,
every check as the archive,
class seventy-one and the abstention log live,
the 左右 heading still literal (observation).
`shi_Yumiaoya6` (frozen `155c6aa65`,
pid 1546977,
launched 2026-09-21 12:11 UTC) runs as the queue's next entry;
read it against `shi_Yumiaoya5`,
then launch `yingying` and `XingZ60` on the same dist,
one at a time.
MIKAELA5 IS READ AND THE ABSTENTION LOG IS BUILT
(the pass log's heading dated 2026-09-19 23:20 UTC):
SETTLED at 23:15 UTC in 27 min on one attempt,
1.13 USD by the meters;
class seventy-two on the page ("Mika was outed against her will."),
classes seventy and seventy-one holding,
slice 28 settled by the slate on its first round so class seventy-three's run-off path is covered by its guards alone.
An abstaining or stray ballot now logs its reason beside the choosing ballots
(`155c6aa65`,
guard `9f7218396`);
the next pass on any entry reads it live.
CLASS SEVENTY-THREE IS BUILT
(the pass log's heading dated 2026-09-19 22:50 UTC,
`6a9a7ce27`,
guards `49b9dccca`,
the sixth ineligible-standing addendum):
a run-off between valid finalists is decided on the two-ballot floor under the weight minimum,
because `mikaela4` stopped at slice 28 with two judges agreeing on a finalist and two abstaining on grounds the rule had answered.
MIKAELA4 IS READ
(same heading):
INCOMPLETE at 22:32 UTC in 24 min,
1.17 USD by the meters;
class seventy-two reached the lane
(the repair select chose "Mika was outed",
the ballots citing the glossary's meaning),
the page never assembled.
Open:
an abstaining ballot's reason is not logged.
CLASS SEVENTY-TWO IS BUILT
(the pass log's heading dated 2026-09-19 22:10 UTC,
`c807572ca`,
guard `7582175c8`):
炸柜 (being outed against one's will) is seeded in the community glossary,
because `mikaela3` shipped it as "came out" where `mikaela1` had "blown out of the closet"
and no sheet said which reading the word carries.
MIKAELA3 IS READ
(same heading):
SETTLED at 22:03 UTC in 34 min on one attempt,
1.18 USD by the meters;
class seventy-one live
(the 铨 line probed off the frozen dist,
two judge reasons cite the DECLARED NAMES block,
the dialogue line renders `𝓠𝓾𝓪𝓷`),
class seventy live
(the closing excerpt curled),
one gate-kept and one unendorsed standing (slice 16).
`prompt-payloads/` holds replies,
not sheets,
so a sheet block is read off the frozen dist and the judges' reasons.
Synthetic read dry again at 22:01 UTC with its five-hour window spent.
CLASS SEVENTY-ONE IS BUILT
(the pass log's heading dated 2026-09-19 21:30 UTC,
`cc088bc4b`,
guard `3f88a6841`):
every sheet carries the page's own renderings of its people and linked titles,
read off same-href links and aligned signatures at preparation.
THE THREE SHAPES ARE READ AND CLASS SEVENTY IS FIXED
(the pass log's heading dated 2026-09-19 21:25 UTC):
`CuspariaKLSY`,
`zheermao101` and `mikaela_khara` SETTLED on frozen `bb4bb7556` on their first attempts,
1.95 USD together;
the ordered list,
the inline code span and the `CapDownQuote` component ship as the archive has them;
the shape census is exhausted.
Class seventy
(a possessive apostrophe after a link,
an emphasis span or a tag stays straight)
is fixed in `f664e1165`,
guard `5599f4752`.
`mikaela2` (frozen `f664e1165`) was launched at 21:20 UTC to read it live and killed at 21:29 UTC for class seventy-one;
`mikaela3` reads both.
Three observations are open in that pass log entry,
none built:
tense drift across slices (Cusparia),
the author's handle rendered "Quan" once against `𝓠𝓾𝓪𝓷`
(mikaela;
a per-page name glossary into the sheets is the mechanism),
spaced em-dashes on an archive page without them (zheermao).
XingZ619 IS READ
(frozen `bb4bb7556`,
SETTLED at 20:17 UTC on 2026-09-19,
the pass log's "XingZ619 read" heading dated 2026-09-19 20:40 UTC):
classes sixty-seven to sixty-nine live,
the seating of `9f8dcccd2` live,
the seven steps and the three checks clean.
The V4.1 Flash producer calibration is read
(the pass log's heading dated 2026-09-19 16:55 UTC):
V4.1 Flash writes,
Mercury leaves the translator seat.
XingZ60 is read
(XingZ616 SETTLED,
the pass log's "XingZ616 read" heading dated 2026-09-19 14:55 UTC);
the four queued entries are all read.
Then step 6,
the components no read page has met.
XingZ616 was to be read for the closing poem's slice
(91 on XingZ615's roster):
its contest winner passing the invariants,
no `slate-declined-standing`,
the poem on the page with the attribution inside the quote,
and the `translate-candidate-refused` findings in the artifact;
XingZ608 ran the whole pipeline in 2h16m and stopped INCOMPLETE at publish
(the pass log's "XingZ608 read" heading dated 2026-09-19 02:35 UTC),
XingZ613 stopped INCOMPLETE at 3h02m in consolidation.
Read first the consolidation of the `其九：空白` slice
(84 on XingZ613's roster):
`translate-runoff (finalists 2 of N)` where the first round splits,
a run-off that decides,
no `slate-declined-standing`;
then the admission
(the tail admitted on the bound,
no `placement unresolved`),
then the memorial disclosure block on the page with all four of the original's paragraphs,
then the translate lane's assembly
(no `assembly-footnote-reverted unresolved-reference`,
no `assembly-withdrew-every-replacement`),
then the bilibili link in the poem attribution on the page,
then the nine definitions on the page under the original's labels
and the orphan trim's findings for any definition no marker references,
then the run time against XingZ608's 2h16m.
Then the rest by the seven steps and the three checks:
the five archive-absent disclosure blocks on the page,
then the funeral and teachers paragraphs
(no TA left standing,
the `consolidate gate ballot` lines with their reasons),
then Jev's select ballots
(reason prefix `typed decision`)
against the chat judges' on the same rounds,
then the pictures line
(`image0.webp` corroborated by 4 of 4 readers at 0.570 on XingZ602),
then the nine definitions' labels
(class nineteen's relabel,
class twenty-one's orphan guard),
then the `PhotoScroll`,
`Hexagon`,
`DottedNumber`,
`BlurBlock` and `TextRing` components as the archive has them,
then the `details` block whose summary carries inline HTML and a blockquote,
then the profile link to another entry and the two commented-out empty paragraphs on nobody's page,
then the page assembly's findings
(`repeats another section's heading` in the log,
`pageAssembly.trimmed` in the artifact;
class forty-five's restoration has not fired live yet),
then the shapes hulicaijia7 met:
the inline ruby annotation,
the Sakura component and the footnote
(its 2026-09-09 INCOMPLETE was class twenty-six,
fixed in `40aba2fdb`;
hulicaijia5 was killed on 2026-09-16 before its page),
then any `carries the sheet's own` refusal in the translate log
(class forty-four's first live measurement;
Mio27 had four such candidates of 68),
then the `ATTESTED` and `ATTESTED CONFIRM` lines if the entry cites a page,
then the archive block review,
then the seven steps and the three checks,
then the spend
(XingZ616 to its SETTLED publish at 2h04m:
9.19 USD by the meters,
8,216 seats,
225 retry rounds;
XingZ615 to its INCOMPLETE at 2h41m in consolidation:
10.34 USD by the meters,
8,070 seats,
246 retry rounds;
XingZ614 to its kill 12 minutes in,
in preparation:
0.04 USD by the meters;
XingZ613 to its INCOMPLETE at 3h02m in consolidation:
7.81 USD by the meters,
8,025 seats,
262 retry rounds;
XingZ612 to its ERROR at 34 minutes,
three attempts:
2.19 USD by the meters,
3,010 seats,
146 retry rounds;
XingZ611 to its kill 1 hour and 42 minutes in,
both lanes over:
6.91 USD by the meters,
5,675 seats,
111 retry rounds;
XingZ610 to its kill 29 minutes into the repair lane:
2.08 USD by the meters,
2,074 seats,
26 retry rounds;
XingZ609 to its kill 35 minutes into the repair lane:
3.25 USD by the meters;
XingZ608 to its INCOMPLETE at publish:
10.08 USD by the meters,
7,958 seats,
284 retry rounds,
2 hours and 16 minutes;
XingZ607 over two attempts to the deadline and its INCOMPLETE:
12.42 USD by the meters,
8 hours and 41 minutes;
XingZ606 to its INCOMPLETE at slice 33:
6.92 USD by the meters,
6446 seats,
155 retry rounds,
3 hours and 4 minutes;
XingZ605 to its INCOMPLETE at slice 13:
5.41 USD by the meters,
6220 seats,
108 retry rounds,
4 hours and 8 minutes;
XingZ604 to its INCOMPLETE at slice 13:
7.97 USD by the meters,
6655 seats,
401 retry rounds,
4 hours and 53 minutes;
XingZ603 to its kill in the repair lane:
1.24 USD by the meters,
1238 seats,
28 retry rounds,
60 minutes;
XingZ602 to its ERROR and the killed reattempt:
9.76 USD by the meters,
10034 seats,
348 retry rounds,
5 hours and 57 minutes;
XingZ601 to its INCOMPLETE:
0.04 USD by the meters,
90 seats,
3 retry rounds,
7 minutes;
yingying1 to its SETTLED page:
0.18 USD by the meters,
345 seats,
12 retry rounds,
15 minutes;
shi_Yumiaoya5 to its SETTLED page:
0.77 USD by the meters,
969 seats,
45 retry rounds,
40 minutes;
shi_Yumiaoya4 to its SETTLED page:
0.79 USD by the meters,
899 seats,
51 retry rounds,
52 minutes;
shi_Yumiaoya3 to its INCOMPLETE:
0.57 USD by the meters,
948 seats,
31 retry rounds,
78 minutes;
shi_Yumiaoya2 to its INCOMPLETE:
0.69 USD by the meters,
840 seats,
39 retry rounds,
48 minutes;
shi_Yumiaoya1 to its INCOMPLETE:
0.67 USD by the meters plus Synthetic,
923 seats,
37 retry rounds,
53 minutes;
hulicaijia7:
4.41 USD by the meters,
4,694 seats,
146 retry rounds,
150 minutes;
Mio27:
1.03 USD,
1,314 seats,
46 retry rounds,
117 minutes).
If the page reads clean,
the queue continues with `yingying` and `XingZ60` on the same frozen dist.
ALWAYS KILL AND RELAUNCH if the source moves while it runs.
