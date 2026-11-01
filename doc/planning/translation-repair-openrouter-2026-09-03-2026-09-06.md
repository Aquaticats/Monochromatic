# OpenRouter and the corpus passes: 2026-09-06

What was launched,
read and decided on 2026-09-06.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## The providers come back, 2026-09-06: two defaults move and a rule is written

The owner returned with Synthetic refilled to a third
(`METERS synthetic=wet hyper=wet openrouter=wet syntheticWeekly=33.18% syntheticFiveHour=2750/2750
hyperBalance=2734 openrouterUsd=0.32` at 19:51 UTC),
OpenRouter still uncharged and expected within days,
and the instruction not to wait on it.

- **`yulianNyanner` launched at 19:53 UTC** on tip `14231350f` with all three providers configured and the
  2026-09-04 dials set (overlap 4,
    writer grace 180000),
    runs dir `~/temp/agent/yuliannyanner-20260906`.
    Asked whether the launch was necessary,
    the honest split:
    the double-quoted-path shape it was picked for had already been read on `MTF_0615` and `BI4PBV`;
    what it does test is the first run on this build with all three providers configured,
    the first chance at a consolidation that completes unstarved,
    and a fresh page to read.
- **The production default had never shipped a page.**
  `PASS_OVERLAP` in `corpus-run/pass-overlap.ts` was 1 until the overlap dial was decided;
    every page that shipped ran at 4 through `TRANSLATION_REPAIR_SLICE_OVERLAP`.
    The overlap dial had been measured on four matched pairs on 2026-08-27 and 2026-08-28
    (`keyword233`,
    `Toka_ls`,
    `Zha_Ke`,
    `Weideriche_`;
    normalized wall down 0.091,
    0.273,
    0.239 and 0.183 against a 0.03 band,
    voices never worse)
    and never read into a default because the 2026-09-01 hold froze the pipeline.
    Guard `bb5e97e0e` shown to fail on the old build (`value=1 source=fallback` at
    `pass-entry.unit.test.ts:1927`);
    fallback moved in `e50be2299`;
    record `doc/decision/translation-repair-pass-overlap.md`,
    flagged for veto.
- **The owner's rule is always kill and relaunch.**
  A source change while a pass runs means kill the pass by pid,
    build,
    and relaunch the same entry into a fresh runs dir on the new build;
    a page finished on a superseded build is not readiness evidence;
    a known fix lands before the launch.
    Written into the package README,
    the runbook's launch and restore steps,
    the handover hub,
    the 2026-09-04 snapshot,
    the run-continuity and overlap-dial handovers (`0d2203abd`),
    and deliberately not into the root `AGENTS.md`.
    The 19:53 UTC run was killed at 20:01:44 UTC under it,
    264 calls in (148 Hyper,
    116 Synthetic),
    still in the lanes phase.
- **The writer window was the same defect.**
  The straggler-grace record reserved it for the owner.
    Every shipped page ran its writers at 180000 ms through `TRANSLATION_REPAIR_WRITER_GRACE_MS` while the
    round window was the owner's 120000 ms.
    Measured off the four shipped logs of 2026-09-04:
    writer-round cuts at 180 s were 4,
    6,
    11 and 20 (almost all `produceConsolidations`) against 28,
    35,
    26 and 135 reader-round cuts at 120 s;
    writers at 120 s has no matched arm.
    Offered build it in,
    keep the dial,
    or writers at the round window,
    the owner chose build it in.
    `7a2bdbedf`:
    `WRITER_GRACE_MS = 180_000`,
    never shorter than the round window so the calibration keeps its 300000 ms;
    guard shown to fail neutralised (4 `FAIL`) and pass restored (0);
    oxlint 0 and 0,
    types clean.
- **`yulianNyanner` relaunched at 20:21 UTC** on `7a2bdbedf`,
  pipeline `cb326421`,
    the plain invocation with no dial set,
    runs dir `~/temp/agent/yuliannyanner2-20260906`,
    log beside it.
    The log opens `OVERLAP yulianNyanner value=4 source=fallback` and `WRITER GRACE built in`,
    all three providers wet,
    weekly 32.19 percent after the killed run,
    Hyper 2722,
    roster 9,
    `withheld=none`.
    The reading follows.

## The first plain-invocation run, 2026-09-06, 20:21 UTC: the fifth class, and no page

`yulianNyanner` ended `INCOMPLETE` at 20:50:19 UTC,
1,715,943 ms (28.6 minutes) and 742 calls in
(Hyper 55 percent,
Synthetic 45 percent,
no OpenRouter call),
with `slice 1: the standing text failed the deterministic publication rule and the consolidation left nothing
valid to ship (incumbent-only)`.
Weekly Synthetic went from 32.19 to 31.40 percent and Hyper from 2722 to 2686 credits,
which prices one stopped attempt at about 0.8 percent of the week and 36 credits.

- **What the log said.**
  Eleven `MdxParseError` lines,
    eight at `3:2`,
    two at `5:2`,
    one at `9:2`,
    every one `micromark-extension-mdx-jsx/unexpected-character`,
    every one on the ORIGINAL:
    `contest winner fails publication invariants and remains retryable: no comparison was possible: original
    could not be read` on slices 1,
    2,
    4,
    8,
    11 and 12 at the contest,
    then `consolidation standing text fails the deterministic publication rule and is withheld from the slate`
    on slice 1,
    then `ConsolidationStandingIneligibleError`.
    Translation itself had finished:
    `translated 17 slices (0 resumed): 13 changed, 0 refused by a guard`.
- **What the source carries.**
  Fourteen HTML comment lines,
    translator notes between paragraphs
    (a glossary and section-voice markers),
    and the column the grammar names is the `!` of `<!--`.
    Measured at pin `a41fc607`:
    17 of 92 sources carry a comment,
    34 comment lines in all,
    14 of them here;
    22 of 92 archives carry one,
    and this archive renders the fourteen as twelve English comments.
    The three pages that shipped on 2026-09-04 carry none,
    and the one comment `keyword233` and `Toka_ls` each carry is the contributor credit,
    which preparation handles apart.
- **The class.**
  `parse-document.ts` masks HTML comments to same-length whitespace before its strict MDX parse,
    because the grammar refuses `<!--` outright.
    `readSliceSkeleton` in `translate-skeleton.ts` handed the strict grammar the raw slice.
    So an original with a note was `unparseable`,
    `validateTranslatedSlice` answered `unknown`
    (its own comment says an unreadable original "is not a candidate's fault"),
    and both consumers of the floor,
    `lane-contest-eligibility.ts` and `consolidate-ineligible-standing.ts`,
    treat `unknown` as inadmissible.
    Every commented slice was therefore unshippable on any roster,
    and the entry could never settle.
    The candidates were fine:
    142 slice-cache files carry a rendered comment.
- **The fix,
  `259708e79`.**
  The slice reader masks comments the way the document reader does,
    on both sides of the comparison;
    the mask keeps offsets,
    so atoms after a comment still index the unmasked text.
    Whether a candidate carries the note rendered is left to the judges as wording,
    since the archive itself does not keep parity (fourteen to twelve).
    Guards `ebf6524de` in `translate-skeleton.unit.test.ts` and `translate-validate.unit.test.ts`,
    shown to fail on the unfixed build (2 `FAIL` lines in each suite) and pass on the fixed one;
    `translate-skeleton-page.unit.test.ts` 0 `FAIL` as well;
    oxlint 0 and 0,
    types clean.
    `inspect-paragraph.ts` parses strictly too but is handed paragraphs the document reader already produced,
    which never contain a comment,
    so it is not a sibling.
- **`yulianNyanner` relaunched at 21:03:43 UTC** on `3ab2d318a`,
  pipeline `597fa3e0`,
    plain invocation,
    fresh runs dir `~/temp/agent/yuliannyanner3-20260906`,
    log beside it,
    under the kill-and-relaunch rule (the 20:21 UTC run had already exited on its own).
    The reading follows.

## The third launch, 2026-09-06, 21:03 UTC: the first page on the plain invocation, and the sixth class

`yulianNyanner` SETTLED at 21:48 UTC in 2,694,071 ms (44.9 minutes),
17 slices,
1,180 calls (Hyper 673,
Synthetic 507,
OpenRouter none),
14 straggler cuts (7 at the 120 s reader window,
7 at the 180 s writer window),
6 schema mismatches,
8 transport retries,
3 recovery rounds,
every meter wet throughout,
and the consolidation completed unstarved,
which no run had done since 2026-09-02.
Weekly Synthetic went from 31.37 to 27.48 percent and Hyper from 2686 to 2622 credits,
so one settled entry costs about 3.9 points of the week and 64 credits at zero USD.
`verify-published`:
matched 1,
wordings 17,
silent 0,
`chars=6020=expected`.
`DESTINATIONS source=0 page=0 dropped=0`,
since the source links nowhere.
Three pictures gathered,
all three textless by the deterministic reader,
no model asked.

- **What the page got right,
  read beside the source and the archive.**
  The front matter `desc` is the source's sentence again,
    where the archive had replaced it with the page's closing line;
    the alias carries the source's 涟 beside the archive's romanizations;
    the Japanese lyric line reads as the source writes it where the archive had altered a syllable;
    高二 is "second year of high school" where the archive said "until high school";
    both `PhotoScroll` lines are intact,
    the two-line single-quoted array and the double-quoted one inside a blockquote;
    every one of the fourteen source comments is an English comment in its place,
    including the lyrics attribution the archive dropped;
    no bare pronoun,
    no corner bracket,
    the site's name absent from this source.
- **The defect.**
  The page carries `## Dysphoria` twice:
    for 烦躁,
    whose source comment says the English word for this title is dysphoria,
    and for 桎梏,
    which the archive renders "Shackles".
    Ledger contest 000031 over the 桎梏 slice:
    six candidates,
    three headed Shackles and three Dysphoria,
    the judges chose Dysphoria,
    and seven of eight ballots give the same reason,
    that "the declared editor comment fixes the heading as Dysphoria";
    the one dissent read 桎梏 literally.
    Every one of the twelve prompts carrying that slice carried the note,
    as an identity-context line with no position:
    `- ORIGINAL editor comment: 这里标题对应的英文词是 dysphoria`.
    The note sits under the third heading and speaks of "this title";
    carried into every slice,
    "this" pointed at every heading at once.
- **Smaller readings.**
  "My mind always drifts" in the Introduction,
    a section whose own comment says it is objective narration,
    where the archive has "her mind";
    the translator-instruction comment rendered into English as a comment,
    harmless and pointless;
    the alias listing the name itself.
    Wording,
    for the judges,
    and recorded.
- **The class,
  six.**
  A positional note carried without its position.
    Fixed in `459b2007f` and `7a01c9048`:
    each comment line names "under heading X" or "before the first heading";
    the critic sheet and the house policy say that a note about "this title" or "here" speaks of that heading
    and no other;
    and the publisher refuses a would-ship page on which two headings that differ in the source read the same
    (`CollapsedHeadingError`,
    `corpus-run/heading-distinctness.ts`),
    measured at pin `a41fc607` to refuse nothing:
    no source repeats a heading and no archive collapses two.
    Guards `01b896ea7` shown to fail on the unfixed build (entry-notes 3 `FAIL` lines;
    the floor did not exist),
    0 `FAIL` on the fixed one,
    the publisher,
    critic and preparation suites 0 as well;
    oxlint 0 and 0 after `147c417b6`,
    types clean.
    The floor's own fail-first proof:
    refusal inverted,
    5 `FAIL`;
    restored,
    0.
- **`yulianNyanner` relaunched at 22:06:22 UTC** on `147c417b6`,
  pipeline `30cd442b`,
    plain invocation,
    fresh runs dir `~/temp/agent/yuliannyanner4-20260906`,
    and killed at 22:07:59 UTC,
    26 calls in,
    when the full suite on that build (934 `PASS`,
    2 `FAIL`) showed the names-only message inventory did not know `CollapsedHeadingError`;
    the inventory is a test file,
    fixed in `e5bd6bf0f` with no rebuild,
    and the rule was applied as written.
- **`yulianNyanner` relaunched at 22:10:19 UTC** on `e5bd6bf0f`,
  the same pipeline `30cd442b`,
    fresh runs dir `~/temp/agent/yuliannyanner5-20260906`,
    log beside it.
    The reading follows,
    with the five headings first.

## The fifth launch, 2026-09-06, 22:10 UTC: the headings hold, and the seventh class

`yulianNyanner` SETTLED at 22:50 UTC in 2,362,951 ms (39.4 minutes),
17 slices,
1,150 calls (Hyper 651,
Synthetic 499,
OpenRouter none),
6 straggler cuts,
no refusal of any kind,
no collapse,
every meter wet,
consolidation unstarved.
Weekly Synthetic 27.41 to 23.54 percent,
Hyper 2622 to 2559.
`verify-published` matched,
`chars=6068=expected`.
`DESTINATIONS source=0 page=0 dropped=0`,
three pictures textless,
all fourteen comments rendered in place,
both `PhotoScroll` shapes intact,
"her mind" in the objective section where the 21:03 UTC page had slipped to "my".

- **The headings hold.**
  Introduction,
    Shackles,
    Dysphoria,
    Wishes,
    Final Chapter.
    The translate contest over the 桎梏 slice offered three candidates and all three were headed Shackles;
    the consolidation ballots now reason from the anchor
    ("the passage sits under the heading 烦躁,
    which the editor comment marks as first-person"),
    which is the line the fix added doing what it was for.
- **By design,
  not a defect.**
  The alias reads `Lyna, 涟, Yulian, Nyanner` because the translate sheet's identity rule requires the
    translated name among the alias renderings when the original declares name and alias as one identity,
    which this source does;
    a ballot cites the rule by number.
- **The seventh class.**
  Five contractions on the page carry a straight apostrophe
    (I'd,
    can't,
    What's,
    wouldn't,
    Let's)
    against thirty curly ones and an archive that is curly throughout;
    the 21:03 UTC page had none,
    and measured on the pages that shipped on 2026-09-04,
    `Uekawakuyuurei` carries five straight against two curly and nobody read it.
    `restore-typography.ts` exists for exactly this and runs on every editor and refiner replacement;
    a translate-lane wording,
    a consolidation proposal and a polish rewrite never passed through it.
    Fixed in `bc42fe330` and `b669363b6`:
    the would-ship reading,
    which every publisher and checker derives the page from,
    puts each non-archive wording through the restoration against the row's incumbent and the stored archive
    text,
    so the artifact keeps what the stages wrote and the page and its checks agree.
    Guard `fd7701f49` shown to fail on the unfixed build (2 `FAIL` lines),
    0 on the fixed one with the page-check,
    publish,
    final-selection and critic suites;
    types clean.
- **Wording,
  for the judges,
  recorded.**
  贴贴 kept in Chinese with a gloss where the archive had "get close to someone online":
    the community-vocabulary rule was applied to a term that has an everyday English equivalent,
    and the house policy now says so with this example (`bc42fe330`).
    自慰 rendered by its blunt literal sense in a quoted despairing thought where the archive and the 21:03 UTC page
    read it as self-consolation;
    a polysemous word,
    the judges' call,
    and a stochastic one across two runs of the same slice.
- **`yulianNyanner` relaunched at 23:11:16 UTC** on `c69a256bd`,
  pipeline `7ec38326`,
    plain invocation,
    fresh runs dir `~/temp/agent/yuliannyanner6-20260906`,
    log beside it,
    after the full suite on that build read 935 `PASS` and 0 `FAIL`,
    oxlint 0 and 0,
    types clean.
    Weekly Synthetic 23.53 percent at launch.
    The reading follows,
    with the apostrophe count first.

## The sixth launch, 2026-09-06, 23:11 UTC: the apostrophes hold, and the eighth class

`yulianNyanner` SETTLED at 23:56 UTC in 2,647,492 ms (44.1 minutes),
17 slices,
1,228 calls (Hyper 702,
Synthetic 526,
OpenRouter none),
no refusal of any kind,
every meter wet,
consolidation unstarved.
Weekly Synthetic 23.53 to 19.13 percent,
Hyper 2559 to 2491,
zero USD.
`verify-published` matched,
1 of 1 at the length the artifact implies.
Five distinct headings,
all fourteen comments rendered in place,
three pictures textless,
`DESTINATIONS source=0 page=0 dropped=0`.

- **The apostrophes hold.**
  Zero straight apostrophes inside words against forty curly ones,
    where the 22:10 UTC page had five against twenty-nine and the archive has none against thirty-six.
    The one straight quote left after a letter is `girls'`,
    a plural possessive,
    which the restoration's apostrophe rule skipped because a quote with a space on one side may be
    closing a quotation.
- **The eighth class.**
  The blockquoted component line reads
    `> <PhotoScroll photos={[“${path}/photos/photo3.webp”]} />`:
    the class-seven fix put every non-archive wording through the restoration,
    the restoration protected backtick spans and nothing else,
    and a JSX string literal in typographic quotes compiles nowhere.
    Every slice floor had passed the straight-quoted slice,
    since the would-ship reading runs after them,
    and nothing between that reading and the disk read the page as a document.
    The log had said it and nobody refused:
    `publish: destinations-mdx-downgraded (page)`,
    a warning from the destination check,
    which parses the page and names the fall to plain markdown as a finding.
    Measured at the pin,
    19 of 92 sources carry a tag with a double-quoted attribute
    (`<p style="text-align: end;">` in `XingZ60` and `luxuanwen3` among them),
    so the class was waiting on every one of those.
- **Fixed guard first.**
  `2079c8c99`:
    the publisher reads the assembled page the way every document is read,
    `parseDocument`,
    and refuses one whose strict parse fell back (`UnparseablePageError`,
    names only,
    entry id and the parser's refusal site).
    `ba91c5587`:
    `typography-prose-mask.ts` marks backtick spans and tags,
    `<` followed by a letter or `/` through the `>` that is not a blockquote marker,
    as not prose,
    and every rule of the restoration reads that one mask;
    a trailing apostrophe converts when the replacement holds no straight single quote shaped like an
    opening one,
    which is `girls'`.
    Guards shown to fail on the neutralised build:
    the floor inverted,
    5 and 7 `FAIL` lines in the page-grammar and publisher suites;
    the mask never entering a tag,
    4;
    the trailing rule removed,
    2;
    0 on each restore.
    Full suite 936 `PASS`,
    0 `FAIL`;
    oxlint 0 and 0;
    types clean.
- **The reading gains two greps.**
    `mdx-downgraded` and `would ship a page` join the refusal vocabulary,
    and the apostrophe count now reads any straight quote after a letter,
    not only one inside a word.
- **A page from a superseded build no longer verifies.**
  `verify-published` against the 22:10 UTC runs dir on the class-seven build reads 0 of 1,
    because the would-ship reading moved under it;
    the tool now says what the rule says.
- **Recorded,
  not fixed.**
  The blank line after the front matter is dropped on every `yulianNyanner` page and on `Hangmster`,
    a splice property of the front-matter slice,
    not a rendering defect,
    since the site strips the front matter before the body is compiled.
    贴贴 now reads "cuddle online" under the house rule;
    自慰 reads "masturbation fantasies" again,
    the same literal sense as the 22:10 UTC page,
    the judges' call.
    The alias and the `desc` are as before,
    the one by the identity rule and the other from the source.
- **`yulianNyanner` relaunched at 00:12:39 UTC on 2026-09-07** on `ba91c5587`,
  pipeline `4d000da8`,
    plain invocation,
    fresh runs dir `~/temp/agent/yuliannyanner7-20260907`,
    log beside it,
    weekly Synthetic 21.05 percent at launch.
    The reading follows,
    with the component line and the apostrophe count first.
