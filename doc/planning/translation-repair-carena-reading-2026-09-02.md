# Reading the Carena0442 page the pipeline published on 2026-09-02

The first entry landed by the seated no-loop build:
a page and an artifact,
verified.
This is the
task 3 reading,
the actual output read against the source and the archive before any readiness claim.
Numbers off `~/temp/agent/carena-rerun-20260902.log`,
the artifact
`~/temp/agent/carena-rerun-20260902/artifacts/Carena0442.json` (schema 9,
4.3 MB) and the page diff
`~/temp/agent/carena-page-diff-20260902.diff`.

## Outcome

- `TALLY Carena0442 status=SETTLED slices=22 ... ms=11375156`:
  190 minutes on `3852e4d86` (before the
  completeness-guard fix `321b12673` and the generation-ten reviewer change `8384166e9`),
  launched
  10:38:50 UTC,
  written 13:48:28.
  Under `FIT` that is three times the hour;
  the earlier Carena pass took
  94 minutes on the same fixture,
  so the roster's speed,
  not the entry,
  is what moved.
- `verify-published`:
  1 of 1 pages carry every wording the artifact promised,
  at the length it implies
  (20,751 characters,
  22 wordings,
  0 silent,
  0 missing).
  `DESTINATIONS source=2 page=2 dropped=0`.
  The
  fixture clone is untouched;
  the page sits under the runs directory's `fixed/` tree.
- Refusals:
  0 HTTP 429,
  0 holds,
  0 both-dry.
  The 474 fix was never exercised.
- Seats:
  Qwen3.8-27B threw 113 of 298 calls and Kimi-K3 59 of 298 at the 60-second round window;
  minimax-m3 18 of 275;
  every other seat 5 or fewer.
  GLM-5.3-Flash was asked 80 times (editor and
  refiner only,
  after its unseating).
- Lane contest:
  15 slices lane-won (8 translate,
  7 repair),
  3 settled neither (1 archive declined,
  2
  archive endorsed),
  slices 12,
  19 and 20 uncontested (identical lanes);
  ballots 6 to 8 usable per
  slice.
  Comparison:
  16 both-differ,
  3 archive-stands,
  2 translate-only,
  1 repair-only.
- Repair lane findings the reader should know exist:
  `stage-voice-lost` 85,
  `verdict-index-out-of-range`
  64,
  `duplicate-verdict` 586,
  `stage-roster-incomplete` 60.
  The lost voices are the window cuts above.

## The shape of the page

The archive was 61 lines;
the page is 390,
and 208 of them end in a comma.
That is the semantic-wrap
stage (`src/semantic-wrap.ts`,
`MD1`),
which puts one clause per line so the Markdown renders the same
(soft breaks are spaces) and the source stays maintainable;
the-third-rendering plan records the rule.
It is by design,
not a defect,
but a reader of the diff must know it before reading anything else,
because it makes every changed paragraph look rewritten.

## What changed, read paragraph by paragraph

- Front matter:
  `alias: Carena` became `alias: 飞猫, Carena`,
  the translate lane winning 6 ballots.
  The source declares both handles;
  the archive had dropped the Chinese one.
  Faithful,
  and the
  identity rule passed;
  whether an English page should carry a Chinese handle in `alias` is a house
  question,
  not a defect.
- Epigraph (slice 1):
  both lanes converged on the same wording;
  "so today,
  a year later,
  I am here to
  make up for it" for 我来重新补上 is closer than the archive's "fulfill that promise".
- The Guangzhou paragraph:
  "for family reasons" (家庭原因) corrects the archive's "family conflicts";
  "head teacher" for 班主任 corrects "teacher";
  the archive's stray "had had" is gone;
  "teaching
  part-time under a fabricated university student identity" renders 制造大学生的身份兼职任教 where the
  archive had "working part-time jobs while posing as a university student".
  All better.
- "whenever water was cut off,
  food ran out,
  or she was short of travel money" renders 断水绝粮、路费缺少
  where the archive summarised.
  Better.
- The overdose call:
  "go on enjoying",
  "shuttling between part-time jobs and restaurant work in various
  places",
  "the few plane tickets she flew back and forth on" (来回机票):
  closer than the archive.
- "I always got along fairly well with Carena" for 一直和飞猫相处较好:
  closer than "maintained a good
  relationship".
- "a long-standing close friend" for 长期而且至今仍在保持联系的密友:
  the archive had lost 长期.
- "world-weary and scornful" (厌世而轻蔑) added where the archive had dropped it.
  Recovered.
- "completed her repeat year and the gaokao in 2022":
  "gaokao" enters where the archive said "repeat
  exam";
  community vocabulary the house rule allows,
  and the page uses "college entrance exam" elsewhere,
  so the term is now inconsistent within the page.
  Minor.
- The rooftop paragraph:
  "she was only panicked by the responsibilities that had already been settled
  and that she had no choice but to shoulder—not that she couldn't carry them,
  and not that she wouldn't"
  renders 不是不能承担，也不是不愿 where the archive had "not incapable of carrying them".
  Recovered.
- "would only leave her weary and fed up" for 只会让她厌倦 replaces the archive's "invited immediate
  burnout".
  Closer.
- "she would be queer for life,
  she would be queer for life":
  the source repeats itself
  (她将终身是酷儿，她将终生是酷儿);
  the archive had varied it to "she was queer for life".
  Faithful now.
- "what mattered was that she was queer at her core" for 重要的是她本是酷儿:
  closer than "this was who
  she was".
- "She should have known."
  for 她应该知道 replaces the archive's "She must have known."
  REGRESSION:
  应该
  here is certainty,
  and "should have known" reads as a reproach in English.
- "the floor-to-ceiling glass,
  tall as if no one were there" for 高若无人的落地玻璃:
  literal and awkward;
  the archive's "high,
  clear glass" dropped 若无人.
  Neither is right;
  the page's is the worse read.
- "A youth that takes nothing to heart,
  a life that takes nothing to heart" for 不在乎的青春，不在乎的一生
  replaces the archive's "A life lived on its own terms,
  lived indifferently".
  Closer.
- Footnote [^1]:
  the page completes the definition with what the archive had dropped,
  "It received
  light edits when it was compiled into this entry (including minor text polishing,
  correction of a few
  typos,
  and revisions made under the principle of protecting readers,
  etc.)".
  The source footnote says
  exactly that.
  Recovered,
  and the notes line carried it to every sheet.
- Pronouns:
  she 84 to 93,
  her 127 to 134,
  they/them/their unchanged (2,
  10,
  5):
  the partner's
  neutral ta stays neutral,
  the person stays she.
  No neutralisation.
- Reader protection:
  "she died by suicide on a live stream" is the archive's wording,
  kept;
  no method
  or substance named anywhere the archive did not.

## The one defect

Line 95 of the page reads `so-called \"common sense.\"` with literal backslashes,
where the archive has
“common sense.”
A model answered with doubly escaped quotes inside its JSON string,
the intake kept
the backslashes as text,
and every later guard passed them.
The artifact carries the escaped form in 6
recorded texts and the shipped one.
Tracked as task 26:
refuse or unescape at intake,
with a test,
and
check the other landed pages.

## What this run did not test

The completeness-guard fix and the generation-ten reviewer change landed while it ran;
keyword233 and
the queued Toka_ls rerun are the first runs of those.
No slice ended with an unendorsed standing here,
so the guard would not have fired on this entry either way.
