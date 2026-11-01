# OpenRouter and the corpus passes: 2026-09-26, classes 160 to 178

What was launched,
read and decided on 2026-09-26 classes 160 to 178.
Entries:
TianqiChen66610,
TianqiChen6669,
TianqiChen6668,
TianqiChen6667,
TianqiChen6665,
TianqiChen6664,
TianqiChen6663,
TianqiChen6662.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## TianqiChen66610 read, 2026-09-26: SETTLED, classes one hundred seventy-five to seventy-eight

TianqiChen66610 ran on `.frozen-dist-7390294ff` and SETTLED in 17.0 min.
Class one hundred seventy-four was not exercised (no tie).

- Class one hundred seventy-five:
  slice 16 shipped a straight stray closing quote the curly stray-quote floor missed;
    fixed in `c3c96f289` (guard `3f04e53cb`),
  the floor reading a straight double quote by its shape.
- Slice 13 shipped the archive's gloss of the "high-performance robot" image
    as "a cute character she cosplayed as":
    the repair lane's adjudicators accepted major mistranslation claims against it,
    but only additions disputed the archive (class one hundred seventy-six),
    every contest ballot called the archive flawed and the gate tied 2 to 2,
    which kept the standing (class one hundred seventy-seven),
    and nothing named the character (class one hundred seventy-eight).
- Owner's answers of 2026-09-26:
    "Major+ accuracy,
  also "high-performance robot" is referring to Atri (search)."
    and "Slate's choice (Recommended)".
    Atri is the robot heroine of ATRI -My Dear Moments-,
  whose catchphrase is "Because I'm high-performance!".
- Class one hundred seventy-six:
  `25b9eaec7` (guard `bd6ad35e8`),
  the eleventh addendum of
    `doc/decision/translation-repair-ineligible-standing.md`.
- Class one hundred seventy-seven:
  `325c448ba` (guard `6243c68d0`),
  the twelfth addendum.
- Class one hundred seventy-eight:
  高性能机器人 seeded in `community-glossary-fandom.ts`
    with Atri named in its note (`55d6a72db`,
  guard `754c8211f`,
  glossary pin `842ffe399`).
- Observation,
  not built:
  slice 1's critics missed the omission of 更多人.
- Full suite after class one hundred seventy-seven:
  1,166 PASS,
  0 FAIL.

## TianqiChen6669 read, 2026-09-26: INCOMPLETE on a run-off tie, class one hundred seventy-four

TianqiChen6669 ran on `.frozen-dist-fce655de0` and stopped INCOMPLETE after 1,458 s (24.3 min),
Bedrock and OpenRouter alone with Synthetic and Hyper dry,
at the consolidation of slice 4 (`slate-declined-standing`);
no page and no artifact.

The glossary refusals of classes one hundred sixty-one,
sixty-two,
sixty-four,
sixty-seven and seventy-two fired live
("she has healed",
"type ii diabetes",
"made her met with",
"added a lot",
and "she’s a" three times);
classes one hundred seventy,
seventy-one and seventy-three were not reached on a page.

Class one hundred seventy-four:
the class one hundred seventy-two floor refused the archive's "So,
she’s a girl of incredible perseverance",
so the consolidation standing on slice 4 was ineligible;
the contest had split 2 repair to 2 translate over "on" against "with" for 在最虚弱的身体上;
the slate of three valid renderings tied 1.5 to 1.5
("bore the greatest pressure on her frailest body" against "with the frailest body"),
the run-off over those two tied 1.5 to 1.5 again,
and a run-off of two has nothing left to narrow,
so the entry stopped over two valid sentences.
Fixed in `8db508b0a` (guard red first `8666d2480`):
`settleAbsentDecline` in `translate-runoff-tie.ts`,
called from `judgeTranslateSlate` wherever the slice has no incumbent,
ships one candidate when a challenge round ties across every candidate it asked about,
the repair lane's text first,
then the translate lane's,
then slate order,
with the finding `translate-runoff-tie-broken (<basis>)`;
a partial tie still narrows,
and a slice with an incumbent keeps it.
Recorded as the tenth addendum of `doc/decision/translation-repair-ineligible-standing.md`.

The class fifty-three guard that expected a repeated tie to raise now expects the slate-order ship (`7390294ff`).
Lint 0/0,
types clean,
full suite 1,166 PASS,
0 FAIL (`suite-class174b.log`).
TianqiChen66610 launched on `.frozen-dist-7390294ff` (scope `pass-TianqiChen66610`,
pid 877799).

## TianqiChen6668 read, 2026-09-26: class one hundred sixty-eight live, classes one hundred seventy to seventy-three

TianqiChen6668 ran on `.frozen-dist-5f7fa6742` and SETTLED in 2,005 s (33.4 min),
one attempt,
1,861 calls,
0.97 USD by the spend lines,
12 abandoned streams (0.05 USD),
Bedrock and OpenRouter alone with Synthetic and Hyper dry
(glm-5.3 threw 117 of 117 on Hyper dry;
Qwen threw 231 and Kimi 99 with no reachable provider).
Front matter equal,
three headings,
six em-dashes left as the owner said,
no TA,
one unendorsed standing,
38 abstention lines.

Class one hundred sixty-eight is live:
the log carries `page assembly: blockquote-quotes-unwrapped (slice 16: “My... wish? / … unwrapped …`
and no quoted blockquote paragraph is on the page.
Class one hundred sixty-nine was not exercised:
the bench wrote "She used this handle" and no prose `id` reached the page.
Classes one hundred sixty to sixty-seven hold:
type 2 diabetes,
the headpiece,
Arona and Atri,
"deserved a better life",
"met with family misunderstandings",
"the draw cards really piled up",
"a small box"
and "through that narrow opening".

Classes one hundred seventy to seventy-two,
three wording slips on terms the census finds on TianqiChen666 alone:
"Then everything will have a chance."
for 这样一切都会有机会的 (source line 67,
class 170);
"and when I leave us,
it will end with my memories of Kigurumi" for 我会在离开我们的时候 (source line 49,
class 171);
and "So,
she’s a girl of incredible perseverance…" and "So,
she’s a girl who can be trusted…",
the archive's present tense kept on two of the four parallel 所以她是个…女孩吧 lines (source lines 17,
25,
33 and 37)
beside "So she was a girl…" on the others,
against the house rule's past tense (class 172).
Fixed in `2151fbbc8` (guard red first `91d3fd5d3`):
three `GRAMMAR_GLOSSARY` entries in `rendering-glossary-grammar.ts`
seed 一切都会有机会 ("there will be hope",
"everything will still be possible";
refused "everything will have a chance"),
离开我们的时候 ("when I leave",
"when I leave you all";
refused "when I leave us")
and 所以她是个 ("so she was";
refused "she’s a",
"she's a" and "she is a"),
so a candidate writing a refused form is refused before any judge.

Class one hundred seventy-three:
the page wrote the anime's title as “Anohana:\nThe Flower We Saw That Day”,
in curly quotes broken after the colon,
where the archive sets *Anohana:
The Flower We Saw That Day* in italics.
Fixed in `fce655de0` (guard red first `cdb756270`):
`restoreArchiveItalicTitles` in `corpus-run/archive-italic-title-restore.ts`,
with the span reading in
`corpus-run/archive-italic-spans.ts`,
runs after `unwrapBlockquoteQuotes` in the page assembly.
It reads the archive's single-star italic spans that open with a capital letter
(a bold span's words fall outside the reading) and,
where a page slice quotes the same words in prose
with whitespace runs read as one space,
writes the archive's italic span back,
moving a period or comma the quotes held outside;
finding `archive-italic-title-restored`.
Any other quotation stands,
since it may be speech.
Replayed over the real TianqiChen6668 page:
the one span restored,
"from *Anohana:
The Flower We Saw That Day*."
on one line.

Not a class:
"That glow is like" in the present stays,
a timeless simile rather than the life's narration.

Lint 0/0,
types clean,
full suite after classes 170 to 172 1,165 PASS,
0 FAIL (`suite-class170.log`),
after class 173 1,166 PASS,
0 FAIL (`suite-class173.log`).
TianqiChen6669 launched on `.frozen-dist-fce655de0` (scope `pass-TianqiChen6669`,
pid 704034).

## TianqiChen6667 read, 2026-09-26: classes one hundred sixty-four to sixty-seven live, sixty-eight and sixty-nine

TianqiChen6667 ran on `.frozen-dist-e146633e1` and SETTLED in 918 s (15.3 min),
0.78 USD by the meters,
Bedrock and OpenRouter alone with Synthetic and Hyper dry.
Front matter equal,
three headings,
six em-dashes left as the owner said,
no TA.

Classes one hundred sixty-four to sixty-seven are live on the page:
"she was met with family misunderstanding and mounting pressure" (164),
the final message's blockquote with no stray closing mark (165),
"whenever she glimpsed,
/ through the gap,
/ those she had comforted" (166)
and "the draw cards really piled up" (167).
Classes one hundred sixty to sixty-three hold:
type 2 diabetes,
the headpiece,
Arona and Atri,
"deserved a better life" and "a small box".

Class one hundred sixty-eight:
the original quotes each of her messages with 「」 and the archive sets all nine blockquote paragraphs bare,
yet the page wrapped two paragraphs of one blockquote in curly quotes ("My... wish?
… rely on."
and
"Please don't distance yourself from me …") and left the other nine bare,
so one page quoted her messages two ways.
Fixed in `39a4a06fd` (guard red first `c530bbf58`):
`unwrapBlockquoteQuotes` in `corpus-run/blockquote-quote-unify.ts`,
with the line reading in
`corpus-run/blockquote-paragraphs.ts`,
runs after `unifyQuoteStyle` in the page assembly.
Where the archive's bare blockquote paragraphs outnumber its quoted ones,
it unwraps each page blockquote paragraph that is one quotation from its first mark to its last,
finding `blockquote-quotes-unwrapped`.
A paragraph holding any other mark of the pair (a quotation inside it,
or two quotations) stays as written,
and the pass never adds marks,
since that would need to know which blockquotes are speech.
Replayed over the real TianqiChen6667 page:
exactly those two paragraphs unwrapped,
no quoted blockquote line left.

Class one hundred sixty-nine:
"she used this id on basically all of her social media platforms" for 这个id.
The pinned archive writes ID on three pages and lowercase id on none,
and the psychoanalytic id appears nowhere in the corpus.
Fixed in `5f7fa6742` (guard red first `1e825e277`):
the Canadian forms pass writes a prose `id` as `ID`;
an `id=` attribute and longer words such as "idle" stay as written.

Not a class:
"She loved Menma (めんま)" keeps the kana because the original itself writes 面码（即めんま）
and names めんま as the ID she used everywhere,
so the kana is the handle the sentence is about.

Lint 0/0,
types clean,
full suite 1,165 PASS,
0 FAIL after each (`suite-class168.log`,
`suite-class169.log`).
TianqiChen6668 launched on `.frozen-dist-5f7fa6742` (scope `pass-TianqiChen6668`,
pid 4003551).

## TianqiChen6665 read, 2026-09-26: classes one hundred sixty-two to sixty-seven

TianqiChen6665 ran on `.frozen-dist-c75738ea1` and SETTLED in 17.8 min,
one attempt,
24 slices,
1,428 calls,
1,649 seats,
56 retry rounds,
0.71 USD of OpenRouter cost,
1 abandoned stream (0.004 USD).
Front matter equal,
3 headings,
no TA,
no error finish,
no triple blank line,
4 unendorsed standings (slices 6,
8,
15,
23),
21 abstention lines,
5 em-dashes against the archive's 6.
Synthetic and Hyper stayed dry:
Qwen3.8-27B threw 252 of 252 and glm-5.3 106 of 106.

Class one hundred sixty-two is live:
the page reads "Living with type 2 diabetes".
Class one hundred sixty-three is live:
the kigurumi slice shipped "in her headpiece"
and "Wearing this headpiece" with no refusal of the correct rendering.
Classes one hundred sixty and sixty-one hold (Arona and Atri,
"those she had comforted",
"turned into a small box",
"deserved a better life,
didn't she?").

Class one hundred sixty-four:
slice 6 shipped the archive's "God,
in His sense of 'fairness,'
made her met with family misunderstandings"
for 积极上进遇到的却是家人的不理解.
The contest split 2 to 2 (minimax and gemma-4-e2b for the repair text,
deepseek and Mercury for translate;
deepseek named "made her meet with" as carried over from the archive),
so the archive stood unendorsed with its slip.
Fixed in `c5f664396` (guard red first `32e4213cd`):
the grammar glossary seeds 遇到的却是 with "met with",
"was met with" and "faced"
and refuses "made her met with",
so that standing reads ineligible and a lane text replaces it.
Lint 0/0,
types clean,
full suite 1,162 PASS,
0 FAIL (`suite-class164.log`).
TianqiChen6666 launched on `.frozen-dist-c5f664396` (scope `pass-TianqiChen6666`,
pid 3048898)
and was killed right after launch under always-kill-and-relaunch,
since three more renderings on this page could be better and the owner's standing instruction of 2026-09-25
("whenever you see anything that can be translated better do it") makes each one a fix,
not an observation.

Class one hundred sixty-five:
the archive's final-message blockquote carries one closing quotation mark and no opening one
(the source quotes each line with 「」),
shipped as the archive on an unendorsed standing.
Fixed in `e146633e1` (guard red first `a1c44e652`):
`translate-quote-balance.ts` refuses,
before any judge,
a candidate carrying a closing ”,
」 or 』 with nothing of its kind open,
wired into `sourceCarryFindings` after the marker floor,
so such an archive rendering is an ineligible standing.
It stands aside where the original itself closes a quotation it never opened (a slice boundary inside a quotation),
and an opener with no closer is no fault,
since a quotation over several paragraphs opens each and closes the last.

Class one hundred sixty-six:
"whenever she caught a glimpse through the gaps of those she had comforted" for 在隙中得以窥见有被她治愈的人
reads the gaps as belonging to the people seen through them.
Fixed in `e146633e1`:
the grammar glossary seeds 在隙中 with "through the gap" and "through the narrow gap"
and refuses "gaps of those" and "gap of those".

Class one hundred sixty-seven:
the farewell's 这次的 uno，真的加了很多呢 stayed the archive's "the game of Uno,
truly,
we've added a lot",
which says nothing in English;
in UNO,
加 is stacking the +2 and +4 draw cards.
Fixed in `e146633e1`:
`rendering-glossary-slang.ts` (spread into `RENDERING_GLOSSARY`) seeds
`uno，真的加了很多` with "draw cards",
"piled up" and "+2 and +4 cards" and refuses "added a lot".

Lint 0/0,
types clean,
full suite 1,164 PASS,
0 FAIL (`suite-class165-167.log`).
TianqiChen6667 launched on `.frozen-dist-e146633e1` (scope `pass-TianqiChen6667`,
pid 3464291);
Synthetic and Hyper still dry at launch.

## TianqiChen6664 read, 2026-09-26: TALLY ERROR, class one hundred sixty-three

TianqiChen6664 ran on `.frozen-dist-358a6bdcb` and ended TALLY ERROR after three attempts
(about 20 min,
2 min and 2.5 s),
with no page.
Every attempt stopped on chunk 9 (the kigurumi slice) because the absent slice had no candidate:
the floor refused every rendering the bench wrote for it.

Class one hundred sixty-three:
the class one hundred sixty refused form `inside her head` is the opening of the accepted rendering
"inside her headpiece",
and `refusedFormIn` in `translate-community-term.ts` matched the form with `includes`,
so a candidate carrying the correct rendering was refused as the misreading.
Class one hundred sixty-two was working in the lanes before the stop
(critics flagged "type II",
writers wrote "type 2 diabetes"),
but no page shipped to read it on.

Fixed in `c75738ea1` (guard red first `bd12b556f`):
`refusedFormIn` scans every occurrence of each refused form with a linear `indexOf` cursor
and excuses an occurrence where an accepted rendering of the same entry starts inside it and runs past its end.
A rendering that ends inside the occurrence excuses nothing,
so "a minor trans girl" stays refused on `minor trans` although "a minor" is a rendering,
and the prefix refusals that rely on matching inside a longer word (`spiritual bull`,
`cohabitant`) still refuse.
A standalone "inside her head" beside "inside her headpiece" in the same candidate stays refused.
Lint 0/0,
types clean,
full suite 1,162 PASS,
0 FAIL (`suite-class163.log`).
TianqiChen6665 launched on `.frozen-dist-c75738ea1` (scope `pass-TianqiChen6665`,
pid 2520043).

## TianqiChen6663 read, 2026-09-26: classes one hundred sixty and sixty-one live, class one hundred sixty-two

TianqiChen6663 ran on `.frozen-dist-7b87c5a4e` and SETTLED in 39.2 min,
one attempt,
24 slices,
1,565 calls,
1,822 seats,
67 retry rounds,
0.84 USD of OpenRouter cost,
10 abandoned streams (0.05 USD).
Front matter equal,
3 headings,
no TA,
no error finish,
no triple blank line,
1 unendorsed standing,
31 abstention lines;
7 em-dashes against the archive's 6,
left as the owner said.
Synthetic and Hyper stayed dry:
Qwen3.8-27B threw 284 of 284 and glm-5.3 112 of 112.

Classes one hundred sixty and sixty-one are live on the page:
"within her headpiece" and "Her kigurumi headpiece",
"her portrayals of Arona and Atri",
"someone she had comforted",
"Her memory and imprint became a small box" and
"deserved a better life,
didn't she?".
None of the six refused forms is on the page.

### Class one hundred sixty-two: type 2 diabetes

身患II型糖尿病 shipped as the archive's "type II diabetes".
Current English,
Diabetes Canada's among it,
writes the type with an Arabic numeral.
The pinned corpus carries the term on this entry alone (and its zh_hant page).
II型糖尿病 joins the rendering glossary through `rendering-glossary-medical.ts`,
spread into `RENDERING_GLOSSARY`,
refusing "type ii diabetes" and "type-ii diabetes" (guard red first `a9562aaba`,
fix `358a6bdcb`;
full suite 1,162 PASS,
0 FAIL).
TianqiChen6664 runs on `.frozen-dist-358a6bdcb` to read it live.

## TianqiChen6662 read, 2026-09-26: classes one hundred sixty and sixty-one

TianqiChen6662 (entry TianqiChen666,
Menma) ran on `.frozen-dist-141598127` and SETTLED in 19.7 min,
one attempt,
24 slices,
1,512 calls,
1,754 seats,
61 retry rounds,
0.74 USD of OpenRouter cost,
4 abandoned streams (0.02 USD).
Front matter equal,
3 headings,
no TA,
no error finish,
no triple blank line,
4 unendorsed standings,
17 abstention lines;
7 em-dashes,
left as the owner said.
Synthetic and Hyper stayed dry:
Qwen3.8-27B threw 266 of 266 and glm-5.3 109 of 109 with no provider.

### Class one hundred sixty: fandom words

头壳 (a kigurumi performer's head mask) shipped "within the smallest space inside her head",
reading it as the wearer's own head;
the archive renders 头壳 "headpiece" three times on the same page.
阿洛娜 and 亚托莉 shipped as the archive's "Alona and Atori";
they are Arona of Blue Archive and Atri of
ATRI -My Dear Moments-,
and no candidate on the run wrote either name (the artifact carries neither).
The first full suite on `7b87c5a4e` ran 1,160 PASS and 1 FAIL:
the community glossary's SEEDS test pins the
term list,
extended to the nine terms in `52e1280a1`.
All three join the community glossary through `community-glossary-fandom.ts`,
spread into `COMMUNITY_GLOSSARY`,
refusing "inside her head",
"alona" and "atori" where the source carries the term
(guard red first `069690562`,
fix `7b87c5a4e`).

### Class one hundred sixty-one: grammar slips

Three ungrammatical lines reached the page.
"turned into in a small box" (化作) and "those she has healed" (被她治愈,
present perfect in a life told in the past)
were the archive's own and no lane repaired them.
"should have had a better life,
didn't she?"
(应该会有更好的生活，不是吗) was the bench's regression:
the archive had written "deserved a better life,
didn't she?",
and the tag does not match the clause.
All three join the rendering glossary through `rendering-glossary-grammar.ts`,
spread into `RENDERING_GLOSSARY`
(same guard and fix commits).
TianqiChen6663 runs on `.frozen-dist-7b87c5a4e` to read them live.

### Observations not built

The UNO line 这次的 uno，真的加了很多呢 shipped "we've added a lot",
which loses the draw-card stacking the line
jokes about;
blockquotes carry curly quotes on some quoted lines and none on others where the original quotes
every line with 「」.
