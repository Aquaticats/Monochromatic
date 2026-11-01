# Audit ledger: found while fixing, names, headings and math on the page

Defects met while fixing another finding,
in how a page's names,
HTML headings and math are read.

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
