# Audit ledger: found while fixing, names, headings and math on the page

Defects met while fixing another finding,
in how a page's names,
HTML headings and math are read.
It also holds B1052,
the heading `Part Nine: __` that the corpus pass `XingZ6022` shipped,
and B1054,
a `<details>` block that the parser kinds as a heading.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## X16: a handle or a community's name varies in case across unanchored slices

Status:
measured 2026-09-28;
no floor,
decided for quality.
Found measuring H16:
on XingZ60's newest settled artifact a handle ships as `z60` and `Z60`,
and a community's name as `limelight` and `Limelight`,
on slices no archive English anchors.
Measured over the newest settled artifact of 39 entries (`~/temp/agent/audit-glossary-fix/x16-case-drift.mjs`):
of 279 Latin tokens the originals write that ship,
16 ship in two casings away from a sentence start,
and reading where each minority form stands (the character before it,
never the token) shows
most sit inside a URL or path,
or open a quoted line after `>` (XingZ60's `Z60` among them);
the mid-sentence ones are ordinary English words,
where casing follows grammar and titles.
Restricted to tokens that are not lowercase dictionary words (`x16-floor-reach.mjs`,
URLs left out):
73 ship,
7 ship in a casing the original never uses (on 6 pages),
and each is consistent within its page.
A deterministic floor forcing the original's casing cannot tell a person's handle,
which keeps its spelling (house rule:
HiYku,
wing,
Mikä),
from a brand an original types in lowercase,
whose English styling is capitalized;
it would enforce the wrong English as often as the right one.
The handle case is already a house rule every sheet carries,
so nothing is added.

## X20: the page-name glossary reads Markdown headings and not HTML ones

Status:
fixed in `ee39e2ba5` (red guard `84b9822d8`,
order and levels pinned in `197f8b811`),
2026-09-28.
`page-headings.ts` now holds the one Markdown and HTML heading reader the glossary and the page title spans share,
each heading placed where it stands,
and the glossary pairs each kind among itself under the same-count rule,
listed by where the original's heading stands,
so an HTML heading only one side carries costs no Markdown pair.
Over the 92 pinned pairs the built `pageNameLines` gains heading lines on `aiyysk` (2) and `mikaela_khara` (1)
and nowhere else (`~/temp/agent/audit-glossary-fix/x20-verify.mjs`).
Mutation checked with a control,
then the full suite (0 FAIL):
over X20 and H16's three earlier survivors,
nine of eleven mutants were caught at once;
the two left,
a heading the archive keeps in the original's words
and markers with no space after them,
had no fixture that could fail;
`801b09420` adds one for each,
and the rerun,
with its control,
caught both.

Found as:
Reading the glossary's heading reader beside the page title spans' while fixing H16.
`headingsOf` (`page-name-glossary.ts`) read ATX headings only,
so an archive's rendering of an HTML heading
never reached the sheets as a page name.
Measured at the pin (`~/temp/agent/audit-glossary-fix/x20-html-headings.mjs`):
3 of 92 pairs carry HTML headings with Han in the original,
13 headings;
on `aiyysk` (2) and `mikaela_khara` (1) the archive carries the same count of HTML headings,
so those three are pairs the glossary misses;
XingZ60's 10 against the archive's 6 pair nothing by the glossary's same-count rule either way.
No page has a line of seven or more `#`,
which `headingsOf` read as a heading and CommonMark does not,
so the shared reader,
which stops at six,
reads the corpus as the old one did.

## X21: a `$...$` pair is a formula on the site and prose here

Status:
closed 2026-09-29,
premise refuted by measurement;
found the same day rereading `doc/repetition.md` (moved on 2026-10-18 to `package/translation-repair/doc/repetition.md`)
("The site's grammar is not this one").
The site compiles each page under `remark-math` and `rehype-katex`,
so a `$...$` pair is a formula there,
while this package parses it as prose;
six source pages at the pin carry one.
No pass that rewrites prose
(the typography restore,
the Canadian forms passes,
the prose masks)
has been checked inside a formula,
so a prime curled or a word respelled inside one would ship unseen.
Fix:
census every math pair in the sources,
the archive English,
the settled pages and the artifact strings;
replay those passes over them;
if any rewrites inside a formula,
protect math pairs in `corpus-run/prose-ranges.ts` and `typography-prose-mask.ts` as the site's compiler reads them,
with a guard shown failing first.

What the census found:

-   No formula exists where the passes act.
    The site's own compiler
    (`@mdx-js/mdx` 3.1.1 with `remark-math` 6.0.0,
    the versions its lockfile resolves,
    installed in `~/temp/agent/x21-math/` and run after the site's own rewrite of HTML comments into JSX comments,
    `census.mjs`)
    finds no math node in any of the 92 sources or archive translations at the pin,
    nor in any of the 273 settled pages under the agent runs
    (the package's own runs roots hold artifacts but no written pages).
    A planted file holding a formula,
    a pair of dollar amounts and a display block showed the census finding all three.
-   The entry's "six source pages carry one" counted dollar signs rather than formulas.
    Of the 395 dollar signs in the pinned sources and archives,
    all but one open a template string (`'${...}'`) inside a JSX attribute such as a photo list,
    which the MDX compiler reads as code;
    the one left is a single currency sign in `mikaela_khara`'s archive translation,
    with nothing to pair with.
-   So no typography or spelling pass has ever run inside a formula,
    and a mask protecting formulas would guard nothing and could not be shown red.
    What the census did find is a different gap,
    recorded as X22.

## X22: this package's strict parse lacks the site's math grammar

Status:
fixed 2026-09-29,
found the same day closing X21.
`parse-mdx.ts` says it parses with the grammar family the site compiles with,
but the site adds `remark-math` and this package does not,
so a wording that forms a formula passes the strict parse the publisher runs (the eighth class)
and renders as TeX on the site.
Models write such wordings:
over every string of the 370 artifacts under the agent runs and the package's runs roots
(`~/temp/agent/x21-math/artifact-dollars.mjs`,
each candidate confirmed by the site's compiler),
44 form a math node.
43 are judges' reasons,
findings and claim summaries,
which reach no page;
one is page text,
a translate-slate candidate in `XingZ616` (slice 83) that wrote a TeX command between dollar signs
where the source has no formula.
It did not ship,
since no settled page carries a math node,
but nothing refused it.
Fix:
add `remark-math` at the site's major version to `parseMdxBody`,
first showing the parse tree unchanged over every current input
(so no cache version moves),
then a floor where the quote-balance floor stands that refuses a wording forming a math node its incumbent lacks,
with a guard shown failing first.

What was done:

-   `5ce9370ac` adds `remark-math` to the strict parse,
    pinned in the catalog to the site's major version with the reason beside it;
    the lockfile gains `remark-math` 6.0.0 and its own dependencies and nothing else.
    Over the 457 current inputs
    (the 92 sources and archive translations at the pin and the 273 settled pages),
    456 parse to the same tree with and without it,
    and the one left is refused both ways (the eighth class's page);
    a planted formula changed the tree,
    so the comparison could show a difference
    (`~/temp/agent/x21-math/tree-diff.mjs`).
    The tolerant fallback stays without it:
    with no MDX grammar,
    the template strings of JSX attributes read as prose there,
    and math changed 171 of the same 457 trees.
-   `9aecc5277` adds `translate-formula.ts` to the source-carry floors after the quote balance:
    a candidate forming more formulas than its original is refused before any judge,
    comments cut first,
    standing aside where the strict grammar refuses either text.
    `validateTranslatedSlice` runs it,
    so the translate lane,
    the consolidation stages and lane-contest eligibility all read it.
-   No cache version moves:
    the translate,
    consolidation and lane-contest version accounts record that no slice-cache file under the agent runs is newer than
    04:26 UTC on 2026-09-27,
    so no answer cached under the current numbers exists to be served.

Guard:
`src/translate-formula.unit.test.ts` (moved on 2026-10-18 to
`package/translation-repair/translate-floor/src/translate-formula.unit.test.ts`),
committed red in `deb3406b2`:
a pair of dollar amounts,
a TeX command between dollar signs,
and a formula in a paragraph whose comment stands outside it are refused;
escaped signs,
a single amount,
the original's own formula and dollar signs inside a JSX attribute pass.
Five mutants (the floor never refusing,
ignoring the original's formulas,
keeping comments,
counting block formulas only,
and the floor unwired) were each caught,
the comment mutant only after M56's fixture was corrected.

## B1052: `### Part Nine: __` shipped over `### 其九：空白` because both lanes fell back to the archive's `__`

Status:
recorded on 2026-10-29 (UTC),
reproduced offline from the source and archive bytes.
The owner question was answered on 2026-10-29 (UTC):
"Report it as a defect (Recommended)".
Fixed in `9eee8cb5d5` (the red cases) and `95f1a006bb` (the check),
with the cases moved to a test file of the module's own and the scans mended in `67ce369c99`.

The defect:
the page shipped by `XingZ6022` has `### Part Nine: __` at page line 410.
The original's line 334 is `### 其九：空白` ("Part nine:
blank").
The archive's line 330 is `### __`:
`od -c` over the pinned bytes (`a41fc607ea`) reads `#`,
`#`,
`#`,
a space,
`_`,
`_` and a newline,
two ASCII underscores (0x5f).

Which stage wrote the underscores:
none of the pipeline's.
The archive (a hand translation) holds them,
and the slice's three texts,
the archive's,
the repair lane's and the translate lane's,
are all `### __`
(comparison row 84 of the artifact).
No model wrote it and no page pass produced it.

How it survived the lanes (slice 84):
- The repair panel accepted three claims against the heading,
  among them the omission of 其九 and 空白
  (log lines 17857 and 17723).
  The editor round applied two candidates (line 18053),
  the select round tied at weight 1 (line 18051),
  and the checker round heard 1 of 3 (line 18101),
  so the slice closed `unchanged, the archive won, 0/3 served accepted issues resolved` (line 18201).
  The rulings of 2026-10-25 make that outcome (the read check of this pass,
  item 8:
  not a fault in
  `repair/src/repair-chunk-verdict.ts`).
- The translate lane excluded the archive as disputed (line 23746).
  Its two candidates both rendered the heading "Part Nine:
  Blank",
  and differed only in the greeting line that follows it.
  The two judges each backed a different candidate,
  four rounds ended 1 to 1 (lines 23801,
  23864,
  24032,
  24063),
  and the challenge round declined with
  "nothing ships past the decline and the wording the archive has stays" (line 24065;
  `translate-stage/src/translate-runoff-tie.ts:383` to `:392`).
  That is the rule "Two judges must name it" (2026-10-25),
  and the wording in place stays.
- Both lane texts then equalled the archive,
  so the slice's lane relation was `archive-stands`.
  `contestEligibleIndexes` (`lane-contest/src/corpus-run/artifact-two-lane-contest.ts`) admits only slices whose two
  lane texts differ,
  so the contest and the consolidation never saw it.
  The refused archive therefore left no consolidation record,
  and the publish defect `no-valid-wording` (`archiveKeptDefects`,
  `page-check/src/corpus-run/publish-defects.ts:164`
  to `:217`),
  which is read off consolidation records,
  does not name slice 84.
  The DEFECTS line (log line 29724) lists `page-footnotes,no-valid-wording` only.
- The assembly pass `heading-series-unified` then wrote the original's number onto the heading (log line 29652).
  `unifyHeadingSeries` gives every numbered heading of the original's series the house style `Part Nine:` and keeps
  the heading's own words.
  Reproduced offline:
  the function run over ten fixture slices holding the original's ten headings and the archive's,
  with the ninth as `### __`,
  returns the finding `"### __" to "### Part Nine: __"` for that slice.
  So the pass is not at fault,
  and `markup/src/heading-affinity.unit.test.ts:77` (which scores 其九：空白 against another heading) does not
  reach this path.

Measured:
9 of 118 slices were `archive-stands` (8,
22,
41,
47,
57,
61,
64,
76,
84),
and slice 84 is the only one of the nine where the translate lane excluded the archive
(a search of the log for `translate incumbent excluded` at each of the nine).

Should a floor refuse an underscore-only heading the original fills?
It would refuse the archive at slice 84,
but the archive was already refused there (disputed,
line 23746) and still shipped,
because the declined round keeps "the wording in place" and the slice never reached the consolidation.
A floor changes the shipped heading only together with a rule for what ships when the refused archive is the last
wording left.
That rule is for the owner.

Owner question (answered),
with the options as the lead ranked them:
- Option 1,
  a report-only publish defect `placeholder-heading`,
  read off the page and the original like `pageWithoutBodyDefects` (B1046):
  it names a page heading whose words are only underscores,
  after an optional series prefix,
  when the page holds more such headings than the original does.
  The page still ships,
  the defect is reported,
  and no wording changes.
  It follows the ruling of 2026-09-27 ("Ship with the defect") and can be written red first.
  It reports and repairs nothing.
- Option 2,
  send a slice whose lanes both kept an archive the translate lane refused through the consolidation as an
  ineligible standing,
  so it records the archive-kept reason,
  reaches `no-valid-wording`,
  and a slate or writer may propose a heading.
  It needs a refused-archive flag on the comparison row (a new artifact generation),
  and with a one-judge slate it would most likely keep the archive again.
- Option 3,
  keep today's behaviour.
  It breaks no ruling and changes nothing,
  and the placeholder ships unreported.
- Option 4,
  ship the original's heading under the note where a refused archive is the last wording left.
  It never ships `__`.
  It contradicts "Keep archive,
  ship" (2026-09-27) and "the wording in place stays" (2026-10-25),
  and puts Han characters on a page whose English archive exists.
- Ranking:
  Option 1 > Option 2 > Option 3 > Option 4.
  Option 1 beats Option 2 because it costs one check and no schema change,
  while Option 2 moves the artifact for one
  slice in 118.
  Option 2 beats Option 3 because it makes the kept archive visible and gives a writer a chance.
  Option 3 beats Option 4 because Option 4 overturns two rulings to avoid one placeholder.

Owner answer,
2026-10-29 (UTC),
question tool,
"Report it as a defect (Recommended)":
a new report-only page defect `placeholder-heading`,
for a shipped heading whose text is only underscores,
dashes,
blanks or similar placeholder marks where the original heading has words,
read off the page and the original as `pageWithoutBodyDefects` (B1046) reads them.
It changes no wording and never stops a run (the owner's rule of 2026-09-27:
ship with the defect reported).

What changed
(`9eee8cb5d5` red,
`95f1a006bb` fix):
- `page-check/src/corpus-run/publish-placeholder-heading.ts` (new):
  `placeholderHeadingDefects` is wired in `publishFixedPage` beside `pageWithoutBodyDefects`,
  and its check name `placeholder-heading` joins `PublishCheck`.
- A heading has words when its title holds a letter or a digit of any script
  (`isLetterOrDigitOfAnyScript`,
  the helper the markup package already has,
  since a regular expression needs a scoped lint disable).
  The title is what follows the first colon,
  in either width,
  when the text before it holds a letter or a digit,
  else the whole heading.
  So `### __`,
  `### ——`,
  `### ` (empty),
  `### - - -`,
  `### …`,
  `### ＿＿`,
  an ideographic space,
  `### Part Nine: __` and `### Part Nine: ——` have no words,
  and `### Part Nine: Blank`,
  `### Part Nine:` (nothing after its colon),
  `### 9` and `### Blank` have.
- Headings are paired by position where the original and the page hold as many,
  so a heading the original also leaves without words is no defect.
  Where the counts differ nothing is paired,
  and the page is reported when it holds more such headings than the original does
  (the form option 1 of the owner question gave).
  The message names the page heading positions and the counts and quotes no text.
- Run on the real case:
  the shipped page of `XingZ6022` against the pinned original reports one defect,
  `at page heading 12`,
  which is the `### Part Nine: __` at page line 410
  (the page holds 14 headings).
  The shipped page of `XingZ6021` reports none.
  The pinned corpus page of the same entry,
  whose archive holds `### __` itself,
  reports it too,
  unpaired because that page holds 12 headings against the original's 14.

Limits:
- Only the headings the parser gives as top-level are read.
  A heading inside an HTML block such as `<details>` is not read by this check or by `assertHeadingsStayDistinct`
  (B1054).
- The check names the defect and repairs nothing.
  Slice 84 still ships `### Part Nine: __` until a rule for what ships when a refused archive is the last wording left
  exists (options 2 and 4 of the owner question),
  and the owner chose neither.

Guards shown to fail first:
`page-check/src/corpus-run/publish-placeholder-heading.unit.test.ts`
(written first in `publish-defects.unit.test.ts`,
whose file did not load before the fix,
for the missing export;
five cases cover the marks-only headings in ten spellings,
the headings that are no defect,
the pairing by position,
the comparison of counts,
and a page with no heading).

Observation:
two judges each backed a different candidate whose heading was identical,
and no rule carries a line on which they agreed.
That is the same one-to-one split that the read of this pass records as D3 for slices 107 and 114,
and it is not changed here.

Cache versions:
none moved,
because no stage output changed:
the check reads the page and the original at publish,
and no stored file gains a key.

## B1054: a `<details>` block that holds a heading is a node of kind `heading`

Status:
recorded on 2026-10-29 (UTC),
reproduced offline on the built `shared-contract`,
open,
not changed.

Found while writing the check of B1052.
`parseDocument` gives an HTML block that holds a Markdown heading the kind `heading`,
and its text is the whole block:
the input `<details>`,
a blank line,
`### __`,
a blank line and `</details>` parses to one node of kind `heading` whose text is the five lines
(four inputs tried:
the block alone,
after `# a`,
after `# a` and an empty `## `,
and after `# a` and `## b`;
each gave the block as a node of kind `heading`).
`headingWords` of that node reads `<details> ### __ </details>`,
which has letters.

So the heading checks that read nodes of kind `heading`
(`assertHeadingsStayDistinct` and the new `placeholder-heading`)
count a `<details>` block as one heading,
read it by the words of the whole block,
and never see a heading inside it.
A placeholder heading inside a `<details>` block ships unreported,
and the block takes a position among the headings when the two documents are paired.
The `### Part Nine: __` of `XingZ6022` sits outside the `<details>` blocks (page lines 387 to 406 and 80 to 267),
so B1052's case is read.

Not changed:
whether the parser's kind is right for its other readers was not examined,
and a check that wants the headings inside HTML blocks needs a decision on how deep to read.

Cache versions:
none moved,
because nothing was changed.

Ids used:
B1052 and B1054.
