# OpenRouter and the corpus passes: 2026-09-08, 16:07 to 22:42 UTC

What was launched,
read and decided on 2026-09-08 16:07 to 22:42 UTC.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## The E2B calibration prints its standing; the translator seat, 2026-09-08, 16:07 UTC

`producer-calibrate 40 --candidates google.gemma-4-e2b` (pid 87207,
launched 13:07:14 UTC on `5462257b4`,
log `~/temp/agent/producer-calibrate-e2b-20260908.log`,
runs dir beside it holding only the candidate ledger)
printed `STANDING over 40 rounds` at 16:07:09 UTC,
10795 s wall clock,
about four and a half minutes a slice.
Read by the scratch `read-standing.mjs`,
proven first on the 2026-09-01 log
(pooled null 13.0 percent,
z -4.53 and -4.55 for the two writers dropped that day,
threshold 2.77 for nine comparisons,
all as the seating decision records).

Standing as printed,
best first,
with the availability-adjusted share where it differs and the z against the pooled null of 12.8 percent
(308 disinterested wins over 2413 ballots,
Bonferroni two-sided threshold for ten comparisons z 2.81):

-   `hf:Qwen/Qwen3.8-27B`:
    24.4 percent (55 of 225 disinterested ballots,
    over 35 candidates),
    adjusted 21.4,
    z +5.25
-   `gemma-4-26b-a4b-it`:
    21.3 (56 of 263,
    over 40),
    adjusted 21.3,
    z +4.14
-   `hf:moonshotai/Kimi-K3`:
    16.2 (42 of 260,
    over 40),
    z +1.64
-   `minimax-m3`:
    11.6 (30 of 258,
    over 40),
    z -0.55
-   `hf:zai-org/GLM-5.3-Flash`:
    11.1 (24 of 217,
    over 35),
    adjusted 9.7,
    z -0.75
-   `glm-5.3`:
    10.7 (17 of 159,
    over 25),
    adjusted 6.7,
    z -0.78
-   `google.gemma-4-e2b`:
    10.1 (30 of 298,
    over 39),
    adjusted 9.8,
    z -1.40
-   `deepseek-v4-pro-0813`:
    7.8 (19 of 243,
    over 39),
    adjusted 7.6,
    z -2.31
-   `deepseek-v4-flash-0731`:
    7.6 (18 of 236,
    over 40),
    z -2.36
-   `hf:openai/gpt-oss-120b`:
    6.7 (17 of 254,
    over 40),
    z -2.90

Slice-clustered (37 winner-bearing rounds over 37 slices,
top-three inclusion over 4000 resamples of whole slices):
Qwen3.8-27B 91.7 percent,
gemma-4-26b-a4b-it 83.8,
Kimi-K3 83.0,
E2B 12.9,
GLM-5.3-Flash 12.2,
minimax-m3 7.1,
glm-5.3 7.0,
the deepseeks under 2.

`SEAT` lines,
asked and usable:
minimax-m3 87 and 87;
glm-5.3 87 and 46 (41 thrown);
deepseek-v4-flash-0731 88 and 88;
google.gemma-4-e2b 94 and 94;
deepseek-v4-pro-0813 88 and 85;
gemma-4-26b-a4b-it 86 and 86;
gpt-oss-120b 89 and 89;
Qwen3.8-27B 84 and 71;
Kimi-K3 88 and 84;
GLM-5.3-Flash 87 and 73.
E2B's 94 completed streams:
p50 1.6 s,
p90 2.5 s,
max 5.1 s;
no warning names it.
The warnings the run did carry:
the cut-mid-reply abandonments the thrown counts sum,
one Qwen3.8-27B and one GLM-5.3-Flash `StreamCutShortError` retried,
eight `InStreamProviderError` 504s on OpenRouter streams (Together six,
Reka one,
Wafer one) between 15:25 and 15:59 UTC,
each retried at attempt 1 and answered,
and one gpt-oss-120b schema mismatch in the last slice's repair round.

THE SEAT,
by the rule as applied on 2026-09-01
(a writer leaves the seat when its z crosses the Bonferroni threshold below the pooled null;
the two dropped that day sat at -4.5):
E2B at -1.40 is not separated from the null,
so it takes the translator seat,
and with it the consolidation seat every measured writer holds.
Landed as `169a86173`:
`WRITER_UNMEASURED` empty,
`RUN_WRITERS` filtered off `RUN_ROSTER`,
translators eight (quorum 4),
consolidation writers ten;
the guard in `run-seats.unit.test.ts` shown to fail first on the old build
(`expected false to equal true` at the translators check);
oxlint,
types,
954 `PASS` and 0 `FAIL`.
Two fixtures moved with the fact:
on Bedrock alone the translators bench now reaches the pair
(`gemma-4-26b-a4b-it` and E2B),
so the class-fifteen cases stop at the editors instead,
and the one-translator shape of the eighth pass is kept on the roster of that day.
Decision:
the addendum of 2026-09-08 in `translation-repair-roster-seating-2026-09-01.md`.
Not acted on:
gpt-oss-120b below the null again (z -2.90),
deepseek-v4-flash-0731 not this time (-2.36),
deepseek-v4-pro-0813 at -2.31;
the 2026-09-01 drops stand on their own measurement.

## The front-matter branch merges; the tenth hakureico launch, 2026-09-08, 16:18 UTC

With the standing printed,
the throwaway branch was rebased onto `ab0e9f0e9` as `d11f36799`
(built,
types and oxlint clean,
954 `PASS` and 0 `FAIL` in the throwaway;
the 955 written earlier was a miscount of the same set),
fast-forwarded into `translation-repair-rebased` and pushed,
then built and proven again in the main worktree (954 and 0);
the worktree `~/temp/agent/tr-frontmatter-20260908` and both branches are gone.
The seat landed on top as `169a86173`.
`hakureico` launched at 16:18:30 UTC on `169a86173`
(pipeline `697d9d67`),
plain invocation,
runs dir `~/temp/agent/hakureico10-20260908`,
log beside it,
pid 276432.
The first lines the rule and the seat owe:
`FRONT MATTER entry=hakureico authority=archive: the archive translated it, so it ships as it stands and no lane
writes it`,
and `JUDGE SEATS phase=preparation synthetic=dry bedrock=wet hyper=dry openrouter=wet wide=7 select=7 late=8
slate=8 checkers=3 translators=7 readers=5 writers=9 roster=9 withheld=hf:moonshotai/Kimi-K3 waited=0ms`:
Synthetic's window at zero after the calibration and Hyper at zero credits,
so Bedrock and OpenRouter serve the pass,
Kimi-K3 withheld as the OpenRouter-cost seat,
and every count is the seated roster less that one
(translators 8 less one,
writers 10 less one,
roster 10 less one).
What the page must show when it settles:
front matter equal to the archive's byte for byte
(`read-page.mjs` prints `frontMatterEqualsArchive`),
`artifactSchemaVersion: 11` with `frontMatterAuthority: 'archive'` in the preparation,
and E2B's candidates and ballots in the translate lane.

## The tenth pass is killed at 37 minutes; the seventeenth class, 2026-09-08, 16:55 UTC

The tenth pass lost five deepseek-v4-flash-0731 voices between 16:37 and 16:49 UTC,
one critic and four select ballots,
every one a `schema-mismatch` whose raw opening doubled:
`{   "issues":{     "issues": [`,
`{"best": 1{"best": 1, "reason": ...`,
`{"{"best": 1, "reason": ...`,
`{"{"best": 1, ...`,
`{"best": 2{"best": 2, ...`.
Each stream was served by OpenRouter's Makora endpoint and carried reasoning
(391189 raw characters and 11635 reasoning characters on the critic,
153070 and 4214 on the first select),
where the model's clean replies that same quarter hour came from Together with no reasoning and 2000 raw characters.
Read back through the ninth pass (all four providers wet):
of its eight mismatches,
six were gpt-oss-120b through Bedrock with the same doubled opening
(`{ {   "choice`,
`{  {"choice"`,
`{"{"resolution`,
`{"{ "resolut`),
every one on a stream with reasoning characters;
the sixth pass,
on Synthetic and Hyper with no reasoning streams,
had none of that shape.
THE SEVENTEENTH CLASS:
a reasoning stream opens its answer,
abandons the opening,
and writes the whole object after it,
and the reply ladder refused the whole as unparseable.
The object after the fragment is the answer,
and the fragment's whitespace differs from the object's
(`{   "issues":` against `{     "issues": [`),
so this is the model writing twice,
not a client appending a retry to a partial buffer:
the retries the log carries are minutes away from the mismatches and name other models.

Landed as `8bf9deec0`:
`json-false-start.ts` reads the object past the opening
(`readJsonPastFalseStart` tries each brace inside the first 256 characters as the start until one parses to the end;
`parseAnswerJson` parses the whole first and reads past only when the whole fails),
the reply ladder in `chat-json-outcome.ts` calls it where it called `parseModelJson`
and warns `json false start: read the object past an abandoned opening of N chars`,
and the caller's guard still judges what was read.
Guards:
the five observed shapes read with their abandoned lengths (10,
2,
2,
3,
13),
a cut reply,
a reply that opens no object and an opening past the window read nothing,
and the ladder case (`READS PAST an abandoned opening ahead of the object`) failed first on the old build
(`expected 'schema-mismatch' to equal 'ok'`).
Types and oxlint clean;
956 `PASS` and 0 `FAIL`.

The tenth pass was killed at 16:55:44 UTC (pid 276432,
37 minutes,
in the repair lane at its sixteenth chunk) under the kill-and-relaunch rule.
What it had shown before the kill:
`FRONT MATTER entry=hakureico authority=archive` on its first reading;
Synthetic's rolling window returned at 16:27:29 UTC,
nine minutes into the pass,
and served 186 calls after it;
Bedrock 191 calls for 0.09 USD;
OpenRouter 275 calls for 0.57 USD;
one minimax-m3 select reply ended at its content bound (32003 characters against 32000,
`StreamOverrunError`,
voice lost),
one gpt-oss-120b `StreamCutShortError` retried,
one in-stream 504 retried.

## The eleventh hakureico launch on the false-start build, 2026-09-08, 16:56 UTC

`hakureico` launched at 16:56:22 UTC on `8bf9deec0`
(pipeline `f39a482e`),
plain invocation,
runs dir `~/temp/agent/hakureico11-20260908`,
log beside it,
pid 297691.
`JUDGE SEATS phase=preparation synthetic=wet bedrock=wet hyper=dry openrouter=wet wide=8 select=8 late=9 slate=9
checkers=3 translators=8 readers=6 writers=10 roster=10 withheld=none waited=0ms`:
the first seat line with eight translators and ten writers,
every bench whole on three providers,
Hyper at zero credits.
`FRONT MATTER entry=hakureico authority=archive` on the same reading.
What the page must show when it settles:
the front-matter and artifact checks of the tenth launch,
E2B's candidates and ballots in the translate lane,
and every `json false start` line a voice kept where the tenth pass lost one.

## The eleventh pass is killed at 85 minutes; the eighteenth class; the twelfth launch, 2026-09-08, 18:21 UTC

What the eleventh pass showed before it was killed.
`FRONT MATTER entry=hakureico authority=archive` at 16:56:23 UTC.
The pictures phase lost one GLM-5.3-Flash reading of `photo1.webp` to the 360000 ms deadline after
3.5 M characters (17:04:29 UTC);
five readers remained.
Synthetic's rolling week ran dry at 17:21:15 UTC inside the lanes phase
(`markRefused synthetic: refused us while its meter reads dry ... held out for 300000ms`,
six calls in ten seconds,
each routed to the next provider),
and the translate lane opened at 17:51:13 UTC on
`synthetic=dry bedrock=wet hyper=dry openrouter=wet wide=7 select=7 late=8 slate=8 checkers=3 translators=7
readers=5 writers=9 roster=9 withheld=hf:moonshotai/Kimi-K3`.
E2B wrote and voted in that lane:
`candidate 1 from google.gemma-4-e2b won weight 3 across 7 ballots` at 18:03:02 UTC,
the lane's other winners Qwen3.8-27B (twice),
a GLM-5.3-Flash and glm-5.3 composite,
GLM-5.3-Flash,
minimax-m3 and gemma-4-26b-a4b-it (twice).
Four `json false start` lines,
every one Bedrock's gpt-oss-120b (`served=openai.gpt-oss-120b`,
2291 reasoning characters on the first),
abandoned openings of 2,
3,
2 and 3 characters,
and no `schema-mismatch` beside any of them:
the seventeenth class's fix read live.
Two transport failures retried on the first attempt (a 503 from Together at 17:31:59 UTC,
a 502 from NextBit at 17:39:42 UTC).

The one lost voice,
18:08:45 UTC in the lane contest:
`lane-contest deepseek-v4-pro-0813: schema-mismatch (content is not valid JSON: SyntaxError: Unexpected end of
JSON input (model stopped with finish_reason=error)) raw="", voice lost`,
on a stream served by CoreWeave that completed after 24291 ms with 315414 raw characters,
8284 reasoning characters,
0 content characters and a cost of 0.
The provider had marked the generation failed and the ladder counted the failure as the model's answer.
Read back across the logs since 2026-09-03 with `finish_reason=error`:
seven replies in four passes
(`openrouter-live3-20260903` two GLM-5.3-Flash translations on Together,
`mtf_0615-shapes-20260904` one GLM-5.3-Flash refiner on Together,
`Uekawakuyuurei-pictures-20260904` one gpt-oss-120b probe and one GLM-5.3-Flash consolidation on Together,
and this one),
every one through OpenRouter,
every one with no content,
every one a lost voice.
OpenRouter's API reference (read 18:10 UTC) normalizes `finish_reason` to `stop`,
`length`,
`tool_calls`,
`content_filter` and `error`;
its errors page draws the mid-stream failure as a chunk with a top-level `error` object,
`finish_reason: "error"` and the stream terminated,
which `requireNoStreamError` has read since 2026-09-04.
These seven passed that check (no error object) and the terminator check (`[DONE]` present),
so the wire shape is one the reference does not draw:
the choice closed on the error finish and the stream terminated normally.
THE EIGHTEENTH CLASS.

Fixed as `bb04656ef` (18:21 UTC):
`openrouter-error-finish.ts` reads any choice whose `finish_reason` is `error`,
carrying `native_finish_reason` when the gateway forwarded one;
`openRouterStreamErrorOf` asks it when no chunk carried an error object and reports `code unnamed`,
the native reason or `error-finish` as the kind,
and the endpoint;
`requireNoStreamError` throws the same `InStreamProviderError`,
so the call rides the retry ladder (five attempts) under its own name.
The guard was shown to fail first against the old build
(`expected { found: false } to deeply equal { found: true, code: 'unnamed', ... }`,
and the new module's suite could not import its export);
oxlint 0 and 0,
types clean,
957 `PASS` and 0 `FAIL`.

The eleventh pass was killed at 18:21 UTC under the rule,
85 minutes in,
inside its consolidation
(the translate stage of a slice had just settled on the incumbent;
three select judges abandoned 120000 ms after quorum).
`hakureico` relaunched at 18:21 UTC on `bb04656ef` (pipeline `920ee70b`) into `~/temp/agent/hakureico12-20260908`,
log beside it,
pid 369919;
`JUDGE SEATS phase=preparation synthetic=dry bedrock=wet hyper=dry openrouter=wet wide=7 select=7 late=8 slate=8
checkers=3 translators=7 readers=5 writers=9 roster=9 withheld=hf:moonshotai/Kimi-K3 waited=0ms`
and `FRONT MATTER entry=hakureico authority=archive` at 18:21:58 UTC.
What the page must show when it settles:
the eleventh's checks,
and no `schema-mismatch` line naming `finish_reason=error`;
an `InStreamProviderError` line with `code unnamed` is the class caught.

## The twelfth pass settles in 108 minutes under the three checks and rewrites the letter, 2026-09-08, 20:10 UTC

`TALLY hakureico status=SETTLED slices=17 repairStatus=repaired repairIssues=78 repairAccepted=52 repairResolved=47
repairFindings=223 repairChanged=13 translateStatus=complete translateChanged=13 documentsDiffer=16 pageChanged=15
pageSilent=0 alignmentFindings=4 selection=contested ms=6504858`
at 20:10 UTC:
108.4 minutes,
against the ninth's 83.3 on a whole bench.
Every seat line of the six phases read
`synthetic=dry bedrock=wet hyper=dry openrouter=wet wide=7 select=7 late=8 slate=8 checkers=3 translators=7
readers=5 writers=9 roster=9 withheld=hf:moonshotai/Kimi-K3 waited=0ms`;
Synthetic's rolling week returned at 19:48:52 UTC (`METERS synthetic=wet`,
1.23 percent used at the tally) and the per-chunk re-seating of the thirteenth class routed 159 calls to it
before the end
(gpt-oss-120b 71,
Qwen3.8-27B 68,
GLM-5.3-Flash 20).

The three checks.
The page's front matter is the archive's seven lines byte for byte
(`read-page.mjs` prints `frontMatterEqualsArchive: true`;
`name: Hanasaka`,
alias `Kagurazaka Hanasaka, Hakureico`),
the artifact carries `artifactSchemaVersion: 11` and `frontMatterAuthority: "archive"`,
the log `FRONT MATTER entry=hakureico authority=archive` at 18:21:58 UTC,
and the blank line after the front matter that the ninth page lacked is back,
since the archive's is what ships.
E2B in the translate lane:
18 ballots (`google.gemma-4-e2b chose candidate N at weight 1`,
one at weight 0.5),
two `translate-repair: google.gemma-4-e2b revised its candidate`,
and one win,
in the consolidation (`translate stage: google.gemma-4-e2b won weight 6.5` at 20:02:53 UTC);
the translate-stage winners across lane and consolidation were GLM-5.3-Flash 8,
minimax-m3 5,
Qwen3.8-27B 5,
gemma-4-26b-a4b-it 3,
deepseek-v4-flash-0731 2,
and one each for the incumbent,
gpt-oss-120b,
E2B,
glm-5.3 and deepseek-v4-pro-0813.
Eight `json false start` lines
(seven deepseek-v4-flash-0731 through OpenRouter at openings of 2 to 12 characters,
one Synthetic gpt-oss-120b at 19:45:04 UTC),
no `schema-mismatch` beside any of them:
eight voices kept.
No line names `finish_reason=error`,
no `InStreamProviderError`,
and no `retrying in` at all
(the eleventh had two transport retries;
this pass had none),
so the eighteenth class did not occur and its guard was not exercised live;
it stands on its unit suite.
The pass's one `schema-mismatch`,
19:45:21 UTC,
`produceConsolidations deepseek-v4-flash-0731: ... Unexpected end of JSON input (model stopped with
finish_reason=stop) raw=""`,
is an empty answer on a normal stop,
one writer of a consolidation slate that settled without it.

The seven steps.
`verify-published` matched 1 of 1 at the length the artifact implies
(`wordings=17 silent=0 chars=4864/expected 4861+separators missing=0`,
exit 0).
`DESTINATIONS hakureico source=0 page=0 dropped=0`;
one `PhotoScroll`,
single-quoted paths as the archive's;
four footnote markers (two references,
two definitions) on an archive with none;
three headings as the source;
the archive's three translator notes carried (the source has none);
Chinese punctuation 11 and CJK runs 14,
equal to the archive's (the notes and the Japanese lyric).
Refusal vocabulary 0 (the one `attempt` hit is a ballot's prose).
Straight apostrophes after a letter 0,
curly 9 (archive 8);
straight double quotes 0,
curly 16 (archive 18);
three-dot ellipses 1,
U+2026 0 (archive 0 and 0;
the source's one `……` closes the solo-journeys sentence).
No run of two or more blank lines:
the seam of three blank lines before the closing sentence that the ninth page carried is gone.
`cost=` fields summed 6.49;
the meters moved OpenRouter 185.59 to 177.62 USD (7.97),
Bedrock 197.85 to 197.37 (0.48),
Synthetic weekly 0 to 1.23 percent;
Hyper 0.

What the prose read found,
beside the source and the archive.
THE LETTER.
The archive's note above it
(`这段话以下全部，包括结尾的两句祝愿，原文都是英文，中文是反向翻译的，请仅修可能造成误解或明显的非刻意语法错误，不大修`:
everything from here,
the two closing wishes included,
was written in English,
the Chinese is a back-translation,
fix only what misleads or is plainly unintended grammar,
no heavy revision)
ships on the page,
and the page still rewrote the English original in five places:
`I am never gone` became `I am never really gone`;
`If you happen to see this` became `If you happen to see this little poem`;
`And who’s by your side w` became `And no matter who’s by your side w`;
`I will always be with you` moved from before the two `No matter` lines to after them,
the back-translation's order;
and the archive's two closing wishes
(`Now these accounts are free. May she be free too.` and `Time to sleep friends, and keep smiling, stay alive.`)
became one sentence rendered from the source
(`It’s time to say goodnight. May everyone keep smiling and live well.`),
the source carrying only the second.
The ninth page had gone further
(`I actually never left`,
`Just changed places`,
`From the earth to heaven above owo`,
`Cause cats have nine lives`);
the twelfth keeps `From ground to heaven owo`,
`Cause neko has 9 lives =w=`,
`Aug. 8th, 2018` and the signature.
The page's first note
(`本文的大部分引用原文都是英文，引用部分请仅修语法和可能造成误解的错误`:
most quoted passages were English,
fix only grammar and what misleads)
is followed for the two short quotes,
which ship verbatim
(`Finally went to hell as a devil.` with `updated 18-08-09`,
and `May the world treat you well.` with `At least better than me.`),
and not for the letter.
The notes reach the slices (the sixth class's fix carries them with their position),
so this is the models' judgment on a long quoted span,
not a missing note;
nothing on the pipeline's side makes the archive the authority for a span whose note says the archive is the
original,
which is the front-matter rule's shape (the archive's text is the original and ships as it stands) applied to a
span instead of the front matter.
Measured across the pinned corpus:
22 of the 93 archive pages carry a translator note,
and 2 carry one saying the English is the original
(this page's,
and `cheonwoomaeng`'s,
saying the whole page is the author's English and must not be touched).
A DESIGN QUESTION for the owner,
recorded in the handover under "The twelfth page and the letter";
nothing landed.
Smaller:
`“Mayday”[^1]` puts the footnote marker outside the closing quote where the source has it inside the brackets
(`「Mayday[^1]」`) and the ninth page had `“Mayday[^1]”`,
the English convention;
one em dash (`rhythm games — she also focused`) on an archive page with none,
where 38 of the corpus's 93 archive pages carry one;
seven lines ending in a space (the Mayday quote's blank `> ` lines) where the archive has 23 and the ninth page 21,
the letter's blank quote lines being bare `>`;
the `osu!` line break inside its sentence (`her *osu!*` then `account was still online.`),
recorded at the sixth pass;
`feeling down` for 精神不好 where the ninth had `struggling mentally`,
softer than the source.
The rest reads true to the source:
the Giftia paragraph,
the transponder quote with three and two Maydays as the source (the archive's four and three) and the archive's
bold,
`Hanasaka’s Letter` for 千歌的信 under the name rendering,
the departure paragraphs.

The straggler count against the ninth (`SEAT ... threw=`):
Qwen3.8-27B 35 of 245 (ninth 17 of 258),
deepseek-v4-pro-0813 21 of 241 (0 of 248),
deepseek-v4-flash-0731 9 of 193 (0 of 198),
glm-5.3 8 of 121 (3 of 120),
GLM-5.3-Flash 7 of 73 (7 of 75),
gemma-4-26b-a4b-it 1 of 245 (0 of 219),
minimax-m3 0 of 230 (1 of 240),
gpt-oss-120b 0 of 227 (0 of 242),
E2B 0 of 223 (0 of 180),
Gemma 31B 0 of 2 (0 of 2),
Kimi-K3 withheld (3 of 258).
81 lost voices against 38,
by round:
select 26,
critic 16,
panel 15,
lane contest 7,
translate 6,
consolidation writers 5 (one of them the empty answer),
refiner 3,
editor 1,
checker 1,
naturalness review 1;
every one but the empty answer abandoned after quorum at the straggler window.
With five readers and nine seats against the ninth's six and ten,
quorum came from the fast seats and the window cut the long reasoners
(Qwen3.8-27B through CoreWeave at 2.9 to 3.2 M delivered characters per cut,
deepseek-v4-pro-0813 at 0.85 M);
that is a reading of the lines,
not a measurement of who answered first.
Quorum held in every round (`quorum-not-met` 0,
`one-reader-only` 0,
43 corroborated),
so no seat moves;
the 25 minutes over the ninth are the window's price on a shorter bench.
GLM-5.3-Flash on `photo1.webp`:
`stream cut after 7127626 characters (CallTimeoutError: ... exceeded its 360000ms deadline)`,
eight passes of eight;
four readers corroborated the picture at overlap 0.598.

`yuki418330012` launched at 20:18 UTC on `bb04656ef` (tree `ba7ce85a9`) into `~/temp/agent/yuki418330012-20260908`,
log beside it,
pid 393298,
on three providers
(`JUDGE SEATS phase=preparation synthetic=wet bedrock=wet hyper=dry openrouter=wet wide=8 select=8 late=9 slate=9
checkers=3 translators=8 readers=6 writers=10 roster=10 withheld=none waited=0ms`)
and `FRONT MATTER entry=yuki418330012 authority=archive` at 20:18:17 UTC.
Watched by the filtered poller that reports only the tally,
a stop,
a crash,
an error-finish regression or the process exit;
read by the seven steps plus the three checks when it settles.

## The letter sealed and the yuki page read and the nineteenth class fixed, 2026-09-08, 22:10 UTC

The owner answered the letter question after the twelfth page was reported (20:20 UTC):
"1",
span authority from the note,
and "in this specific case our pipeline should refuse to 'repair' that specific entry" for `cheonwoomaeng`,
whose note says the whole page is the author's own English.
Landed as `439667ec3` (the seal and the decline) and `58f647ab9` (the fixtures and
`doc/decision/translation-repair-archive-original.md`).
The seal:
`archive-original-note.ts` reads the archive's HTML comments for the marks
(`以下` with `原文` and `英文` for a span,
`原文即英文` or `不要动本篇` for the whole page),
a span runs from the note's end to the next heading or the end of the page,
`groupNodesSealed` keeps the sealed translation blocks and the originals paired with them as `SealedRun`s through
merge and anchor and drops them before slicing
(an original the source carries behind the seal becomes an insertion at the seal's end),
the preparation records `archiveOriginalSpans` (artifact generation twelve),
`assertArchiveOriginalComplete` refuses a page that does not carry every sealed span byte for byte,
and the pass logs `ARCHIVE ORIGINAL entry=<id> span=<k> [start, end) of the archive ships as it stands under the note`.
The decline:
`runEntryPipeline` reads the note before any purchase,
writes `declined/<id>.json`,
prints `TALLY <id> status=DECLINED reason=archive-original`,
the scheduler counts the id as done from the artifacts and the declines together,
and `verify-published` prints `declined=N` and flags `DECLINED AND PUBLISHED ANYWAY`.
Verified live:
`cheonwoomaeng` into `~/temp/agent/cheonwoomaeng-20260908` declined in 41 ms,
`declined/cheonwoomaeng.json` written and no page;
the thirteenth `hakureico` launch (`439667ec3`,
`~/temp/agent/hakureico13-20260908`,
21:46 UTC) logged `ARCHIVE ORIGINAL entry=hakureico span=0 [3966, 4561)` under the letter's note at 21:46:19 UTC
and reached the lanes at 21:52 UTC.
One thing `verify-published` still does:
a runs dir holding only declines prints `NOTHING VERIFIED` and exits 2,
since no page was published;
a whole-corpus pass always has pages beside its declines,
so the exit is right there and wrong only on a decline-only dir.

The `yuki418330012` page (`bb04656ef`,
20:18 to 21:22 UTC,
`~/temp/agent/yuki418330012-20260908`),
read by the seven steps plus the three checks.
`TALLY yuki418330012 status=SETTLED slices=9 repairStatus=repaired repairIssues=68 repairAccepted=46
repairResolved=44 repairFindings=107 repairChanged=5 translateStatus=complete translateChanged=6 documentsDiffer=6
pageChanged=5 pageSilent=1 alignmentFindings=8 selection=contested ms=3862504`:
64.4 minutes on three providers
(`synthetic=wet bedrock=wet hyper=dry openrouter=wet wide=8 select=8 late=9 slate=9 checkers=3 translators=8
readers=6 writers=10 roster=10 withheld=none` at the preparation;
Synthetic refused at 20:33 UTC and was held out for 300 s under the fourteenth class's reading,
14 `HTTP 429` retries at 20:32 to 20:33 UTC).
The three checks passed:
the front matter is the archive's byte for byte (`FRONT MATTER entry=yuki418330012 authority=archive`),
30 `json false start` reads (26 of them deepseek-v4-flash-0731) every one kept with no `schema-mismatch` beside it,
and no error finish occurred.
`verify-published` matched at length (`wordings=8 silent=1`),
destinations 0,
0 and 0.
Seats that threw at the straggler window:
Qwen3.8-27B 28 of 118 calls,
deepseek-v4-flash 9 of 90,
Kimi-K3 6 of 67,
GLM-5.3-Flash 3 of 48,
deepseek-v4-pro 2 of 115,
E2B 1 of 101.
Meters at the tally:
OpenRouter 172.94 USD,
Bedrock 197.20 USD.

The prose read found three things.
THE NINETEENTH CLASS,
the footnote label swap:
the original writes `洲洲[^2]` and `真理[^1]`;
the archive had renumbered by first appearance,
`Zhouzhou[^1]` and `Zhenli[^2]`,
with its two definitions numbered to match
(`[^1]: Yuki's substitute parent ... Zhouzhou`,
`[^2]: She's younger ... sister`).
Every lane judged the body against the original,
so the page's body follows the original's labels,
while the archive's definitions stood as they were:
the roster paired neither
(section 1 paired 6 of 8 original and 10 of 12 translation blocks),
the original's definitions came through as an insertion and were withdrawn as duplicates,
`pageSilent=1`.
The page carries `Zhenli[^1]` above `[^1]: Yuki's substitute parent`:
each marker points at the other's note.
The assembly guard (`introducedFootnoteFindings`) diffs only unresolved,
orphan and duplicate findings against the incumbent,
and a swap is none of those.
Second,
`自切`,
community slang for self-surgery,
rendered `self-harmed by cutting` where the archive had `attempted self-surgery`.
Third,
`超天酱`,
the KAngel character of *Needy Streamer Overload*,
rendered `Choco-chan` where the archive named the game.
The second and third are regressions of correct archive renderings by a bench that does not know the community's
words;
they are the glossary question put to the owner in the handover.

The fix for the nineteenth class,
landed as `1ba94c27a` and corrected as `ea07a1512`,
sits upstream of every lane:
once the archive's labels are rewritten to the original's,
label equality is the correspondence,
a definition pairs with the definition of the same label,
and a definition that renders the wrong note is a fidelity defect the lanes see.
`archive-footnote-relabel.ts`:
`referenceLabels` lists a text's distinct GFM reference labels in first-appearance order,
definition openers left out;
`footnoteRelabelOf` reads a positional map off each paired slice
(the k-th distinct marker on one side is the k-th on the other),
leaves a slice whose sides carry different counts out of the reading with a note naming it,
and refuses as ambiguous where two slices map one label two ways;
`applyFootnoteRelabel` rewrites references and definition openers of the mapped labels in one pass.
`pass-footnote-relabel.ts` applies it after the first preparation and `pass-prepare.ts` prepares again over the
relabelled archive before the block correction round
(one preparation,
at most one relabel with its re-preparation,
at most one correction round with its re-preparation),
logging `FOOTNOTES entry=<id> relabelled [^1]->[^2], [^2]->[^1]`,
`FOOTNOTES entry=<id> left out of the relabel reading: slice N references ...`,
or `FOOTNOTES entry=<id> archive labels stand, since the slices disagree: ...`.
The suite failed first on the old build (`does not provide an export named 'applyFootnoteRelabel'`),
then passed;
the full suite on `1ba94c27a` printed 967 `PASS` and 0 `FAIL`.

The correction:
the fourteenth `hakureico` launch (`1ba94c27a`,
22:05 UTC) read
`FOOTNOTES entry=hakureico archive labels stand, since the slices disagree: slice 7 references 1 distinct notes in
the original and 0 in the archive`,
because the archive of `hakureico` carries no `[^2]` at all
(the source's `HOSTED__WITH__GAE[^2]` inside the letter and its definition `即 Google App Engine` were never
translated).
An omitted or added note is the fidelity defect the lanes see,
not a labelling conflict,
so the first build's abort on a count mismatch was wrong;
`ea07a1512` leaves that slice out of the reading,
named,
and reads the map off the rest.
Under the kill-and-relaunch rule the thirteenth pass was killed at 22:04 UTC for `1ba94c27a`,
the fourteenth and the second `yuki418330012` (22:05 UTC) at 22:07 UTC for `ea07a1512`,
and the fifteenth `hakureico` (`~/temp/agent/hakureico15-20260908`,
pid 461533) and the third `yuki418330012` (`~/temp/agent/yuki3-20260908`,
pid 461634) launched at 22:08 UTC on `ea07a1512`,
both on Bedrock and OpenRouter
(`synthetic=dry bedrock=wet hyper=dry openrouter=wet wide=7 select=7 late=8 slate=8 checkers=3 translators=7
readers=5 writers=9 roster=9 withheld=hf:moonshotai/Kimi-K3`),
each watched by the filtered poller.
What their pages must show:
`hakureico`,
the letter byte for byte the archive's under the seal
(`assertArchiveOriginalComplete` would have refused the page otherwise),
the source's footnote definitions after the span,
`[^1]` agreeing on both sides,
slice 7 named as left out;
`yuki418330012`,
`FOOTNOTES entry=yuki418330012 relabelled [^1]->[^2], [^2]->[^1]`,
`Zhouzhou[^2]` above `[^2]: ... substitute parent` and `Zhenli[^1]` above `[^1]: ... sister`,
and whether `自切` and `超天酱` regress again.

## The nineteenth class's second face where the crossing definitions starve the pairing, 2026-09-08, 22:36 UTC

The third `yuki418330012` launch (`ea07a1512`,
22:08 UTC) did not relabel.
Its log read
`FOOTNOTES entry=yuki418330012 left out of the relabel reading: slice 4 references 0 distinct notes in the original
and 2 in the archive`
and no relabel line,
because the roster's second pairing round had starved:
`paired 3 of 8 original and 3 of 12 translation blocks across 3 relations, from 2 usable voices of 8 heard`,
with six voices refused,
three as `pairing moves backwards on the original side at position 10` and three as
`pairing moves backwards on the translation side at position 10`.
Position 10 is the definitions.
The original defines `[^1]` (the sister) then `[^2]` (the substitute parent);
the archive defines `[^1]` (the parent) then `[^2]` (the sister);
every voice that paired the two definitions by content paired them crossed,
and the wire reader's rule that a correspondence never steps backwards refused each of them.
The rule is a rule about prose,
and a footnote definition is an order-free block:
a page renders its notes by reference order,
so two documents may define the same notes in different orders and both be right.
The first `yuki418330012` run had paired 6 of 8 and 10 of 12 in that section for the same reason,
the two definitions unpaired on each side,
which is how the archive's definitions came to stand unrepaired under the original's markers.
So the swap wrecks the pairing before the relabel can read it:
the same class,
seen from the other side.

The fix (`e5ff6c4f8`),
in four pieces.
`readBlockPairing` takes the chunk-local indices of the definition blocks on each side as `freeOrder`,
reads the order rule over the body pairs alone,
and refuses a pair that joins a definition with a body block;
the stage threads the indices through and `prepareDocumentPairWithRoster` computes them from the nodes' zones.
`splitDefinitionPairs` (`pair-definition-order.ts`) reads each chunk's agreed pairing after the media claim:
where the definition pairs cross they are kept out of what the slicer walks
(its runs must be in document order on both sides),
named as `block-pairing section N: the footnote definitions cross, kept out of the slicing and read for the relabel`,
and every definition pair is handed on by label as `footnoteDefinitionPairs` on the paired preparation.
`footnoteRelabelOfDefinitions` reads the map off those pairs,
the exact evidence
(the archive's definition of a note carries the archive's label for it,
the original's carries the original's),
and the positional slice reading of `1ba94c27a` stays as the fallback where the roster paired no definition.
`reorderFootnoteDefinitions` (`archive-footnote-order.ts`) then moves the archive's definition blocks into the
original's definition order,
a contiguous run only,
so the second preparation pairs them in order like any other block and the slices carry both sides;
the relabel step logs `FOOTNOTES entry=<id> relabelled [^2]->[^1], [^1]->[^2] off the definitions the roster paired`
and `FOOTNOTES entry=<id> definitions moved into the original's order: [^1], [^2]`.
The wire suite failed first on the old build
(the crossing pairs refused,
the mixed pair accepted),
then passed;
the footnote exports moved to `footnote-barrel.ts` at the pipeline barrel's line budget;
full suite 973 `PASS` and 0 `FAIL`,
oxlint and types clean.
The fifteenth `hakureico` and the third `yuki418330012` were killed at 22:16 UTC under the rule
(the last reading before the kills:
Bedrock 197.01 USD,
OpenRouter 170.95 USD),
and the sixteenth `hakureico` (`~/temp/agent/hakureico16-20260908`,
pid 489831) and the fourth `yuki418330012` (`~/temp/agent/yuki4-20260908`,
pid 489956) launched at 22:36 UTC on `e5ff6c4f8`,
each watched by the filtered poller.
What their logs must show before their pages are read:
`yuki418330012`,
the second pairing round with every voice usable,
`block-pairing section 1: the footnote definitions cross`,
the relabel off the definitions the roster paired,
the definitions moved,
and a re-preparation whose section 1 pairs the definitions in order;
`hakureico`,
slice 7 still named as left out (the archive carries no `[^2]`),
no definition moved,
the seal as before.

## The relabel must close over the archive's labels, 2026-09-08, 22:42 UTC

The fourth `yuki418330012` launch (`e5ff6c4f8`,
22:36 UTC) showed the pairing whole again
(`paired 6 of 8 original and 10 of 12 translation blocks across 10 relations, from 8 usable voices of 9 heard`,
the one refusal `google.gemma-4-e2b returned an unusable pairing: pairing pairs a footnote definition with a
body block at position 5`,
the right refusal)
and then a relabel bug:
`FOOTNOTES entry=yuki418330012 relabelled [^2]->[^1] off the definitions the roster paired`,
one pair,
not two.
The roster agreed on the sister's definitions (`[^1]` in the original,
`[^2]` in the archive) and not on the parent's,
so the map moved the archive's `[^2]` onto `[^1]` and said nothing about the archive's own `[^1]`,
and the rewrite gave the archive two `[^1]` notes.
A map whose target label the archive already carries must also move that label away,
or it is a merge.
Fixed in `2da6e7f22` (`archive-footnote-closure.ts`):
`documentLabels` lists every distinct label a text carries,
references and definition openers alike;
`closeFootnoteRelabel` completes the one pair elimination forces
(exactly one label left unmapped on each side and not the identity)
and refuses as open any map that lands on a label the archive carries and does not move,
naming the labels and how many stand unaccounted for on each side;
the pass applies only a closed map,
skips the reorder when the map is open,
and logs
`FOOTNOTES entry=<id> archive labels stand, since the map read off <basis> does not close: ...`.
The suite failed first (`does not provide an export named 'closeFootnoteRelabel'`),
then passed;
full suite 975 `PASS` and 0 `FAIL`,
oxlint and types clean.
The sixteenth `hakureico`,
the fourth `yuki418330012` and the first `Arita` (launched 22:38 UTC beside them,
the next entry in the queue) were killed at 22:39 UTC under the rule,
and the seventeenth `hakureico` (`~/temp/agent/hakureico17-20260908`,
pid 500384),
the fifth `yuki418330012` (`~/temp/agent/yuki5-20260908`,
pid 501073) and the second `Arita` (`~/temp/agent/arita2-20260908`,
pid 501893) launched at 22:42 UTC on `2da6e7f22`,
each watched by the filtered poller.
What `yuki418330012`'s log must show:
`relabelled [^2]->[^1], [^1]->[^2] off the definitions the roster paired`
(the second pair by elimination where the roster agrees on one),
`definitions moved into the original's order: [^1], [^2]`,
and a re-preparation whose section pairs both definitions in order.
