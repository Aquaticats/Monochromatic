# Audit ledger: tags, nesting and paired marks

Readers of tags,
attributes,
paired marks,
front matter and nesting depth,
each reading the structure its own way.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B38: readers of paired marks took an unclosed mark's next title into its span

Status:
fixed 2026-09-30 (UTC) with the sixth T8 batch,
found writing a case for `translate-latin-title.ts` 100,
the reader's stop at an opening mark with no closing mark after it.
Five readers paired each opening mark with the next closing mark:
the Latin title floor and the Han title reader (`han-title-read.ts`),
which read candidates and originals for the translate floors,
the page-title spans (`page-title-spans.ts`),
the work-title scan (`work-title-scan.ts`),
and the title-reference locator (`corpus-run/title-reference-marks.ts`).
An opening mark that never closed took the next title into its span.
Given `《Purr, and 《Cats Are Liquid》`,
the Latin title floor told the model to write `“Purr, and 《Cats Are Liquid”`
beside a second finding for the real title;
the Han title floor named a title holding the stray mark;
the work-title scan looked up the stray span and never the title inside it;
the page-title spans counted a stray span as a title of its own;
and the locator,
reading a page's `《Purr 《Cat Talk》`,
rewrote `Purr 《Cat Talk` whole with the heading's rendering,
dropping the words before the title.
Three of the readers scanned forward from every opening mark,
so their time grew with the square of the unclosed marks:
the old reader shape took 225 ms at 10,000 unclosed marks and 14,153 ms at 80,000,
where the shared reader takes under 2 ms at 80,000 (`t8b6-b38-timing.mjs` in the audit's scratch folder).
The pinned corpus (279 Markdown files at `a41fc607e`) holds no unclosed 《 or 【,
five unclosed 「 in five files,
and four unclosed “ in two files,
each of those four with its next opening mark on the same line
(`t8b6-b38-corpus.mjs` and `t8b6-b38-corpus2.mjs`),
so no English page there uses the multi-paragraph convention of opening each paragraph and closing only the last.

Fix:
`closed-mark-spans.ts` holds the one pairing rule every reader now uses:
a closing mark answers the last opening mark before it,
so an opening mark followed by another before any closing one encloses nothing,
a closing mark with no opening mark before it is text,
and identical marks (straight quotation marks) pair in turn.
It reads each stretch of text once forward and once backward.
On text whose marks all close,
every reader returns what it returned before.
`closed-mark-spans.unit.test.ts` pins the rule,
and each reader has a stray-mark case:
the Latin and Han title floors' verdicts,
`workTitlesOf`,
`repeatedTitleSpans` with the same stray mark twice,
and `unifyTitleReferences` keeping the words before the title.
Run against the build from before the change,
all five reader cases failed.
A multi-paragraph quotation on a page would now show its last paragraph to the locator as a quoted span,
as any quoted line of dialogue is;
before,
it was read as one span crossing lines and left out.
The Latin and Han title findings reach the model in the revision round,
and the page-title spans and the work-title scan feed preparation,
so the change joins the translate and preparation cache accounts before a launch.

## B72: the italic-title pass read structure off the text

Found reading `italicizeSlice` for B70's splice,
which left open that a quoted title inside an emphasis span was rewritten in asterisks there too.
The archive side read structure the same way:
`archiveItalicSpans` (`corpus-run/archive-italic-spans.ts`) split each line of a slice at its stars,
so words inside an HTML comment or an MDX expression read as italics,
a span across two lines read as its first line's words,
a linked title read as link markup and never as a title,
and a span set with underscores was never read.

Measured first with the site's compiler
(`@mdx-js/mdx` with `remark-math`,
comments rewritten as the site's build does;
`italic-in-emphasis-census.mjs`)
over the 92 archives at the pin and the 273 settled pages,
of which it parses 272.
Its control,
one quoted title inside emphasis,
one inside bold and one outside,
counted 3 found,
1 inside emphasis and 1 inside bold.
The pass finds one quoted archive title in the archives and two in the settled pages,
none inside emphasis or bold.
The two readings of the archives' titles differ by nine
(`italic-title-reading-diff.mjs`,
`italic-star-only-where.mjs`):
five only the split reads,
four of them inside MDX expressions in `XIEPT2` and `shihai4h`
and one the first line of a span in `Susiethegamer`;
four only the parse reads,
three linked titles in `Futajuhuacha` and `zhangyubaka`
and that whole span in `Susiethegamer`.
None of the nine stands quoted in any archive or settled page
(`italic-reading-effect.mjs`),
so the fix changes no current output.

Fixed (`db01484f7` red,
`f0bad6d25`,
then cases in `d38620f38` and `3429c8229`):
`emphasisSpans` (`corpus-run/emphasis-spans.ts`) blanks HTML comments to spaces of the same length,
reads the text with `parseBodyTolerant`,
and returns each emphasis node's offsets and words,
its text and inline code in document order with a hard break read as a line feed,
on one line.
`archiveItalicSpans` reads those words,
and `italicizeSlice` parses a slice only when it quotes an archive title in prose,
then leaves quoted any title that overlaps an emphasis span.
`treeNodes` moved to `mdast-tree-nodes.ts`,
shared with the formula floor in `translate-formula.ts`.

Four calls made here are open to veto.
The parse is the tolerant one,
strict MDX first and plain Markdown where the strict grammar refuses,
so the pass has no refusal arm:
with the package's `parseMdxBody`,
2,251 top-level blocks of the archives and 10,724 of 272 settled pages each parse alone,
and one settled page (`yulianNyanner`) is refused whole,
so a refusal is reachable on shipped pages,
while inline emphasis reads the same in both grammars.
Only emphasis is left alone,
not bold:
italics inside bold render bold italic,
which still sets a title apart.
A quoted title is left when it overlaps an emphasis span at all,
not only when it lies inside one.
And `oneLine` folds only the space and the line feed,
though its TSDoc said every whitespace run:
the pass writes the page's own words back,
so folding a no-break space would change the page's spacing.
Measured at the pin
(`b72-whitespace-probe.mjs`,
its control counting a tab,
a carriage return and a no-break space),
no quoted span of an archive or settled page holds whitespace other than those two,
nor does any emphasis span of the 272 settled pages the compiler parses,
and one archive emphasis span holds a narrow no-break space.
The TSDoc now says what the code folds.

With the emphasis skip,
the comment mask and the hard break's line feed each turned off in one build,
the case covering each failed and nothing else in the file.
The case for words inside an HTML comment passes with the mask off,
since plain Markdown reads a comment as an HTML node,
which is why `d38620f38` puts a JSX comment beside one:
the strict grammar reads it as an expression only once the HTML comment is blanked.
The readers' own tests failed as named with `treeNodes` made recursive
(the deep tree,
a `RangeError`),
the parse made strict alone
(the refused text,
an `MdxParseError`)
and `oneLine` folding every whitespace character
(the no-break space),
the last in a build of its own (ledger M89).
Each was restored,
and the tree read clean.

Its census at `3429c8229`
(`census-apleN5`,
1,475 passes,
taken clean)
reads against `census-vi1FHB`:
ran 0,
still cold 1,103,
cold since then 4,
not loaded 6,
claimed sources with no stretch there 0,
sources edited since then 2
(`archive-italic-spans.ts` loaded with 0 cold stretches,
and `archive-italic-title-restore.ts` with 1,
its `break` on a quote that never closes,
cold at the baseline too).
Library source went from 747 stretches over 1,435 lines in 275 files to 746 over 1,432 in 274:
the star split's stretch went with it.
The suite prints one pass line per top-level suite,
so the 3 more passes are the new test files' three suites,
and the cases added to an existing file add none.
The 4 cold since then are the other packages' stretches `census-vi1FHB` read as run
(`with-timeout.ts` 57-59,
`create-logger.ts` 240-247 and 366-373,
`error-format.ts` 31-58),
and no commit since touched those packages,
so they come and go with run timing:
cold in `census-DapwaX`,
run in `census-vi1FHB`,
and cold in `census-apleN5` and in the census one commit before it (`census-NUrxKe`).

Recurrence:
a reader of a page's structure,
block or inline,
reads the parse the site's build reads,
with comments masked as that build masks them,
never a split of the text at a delimiter;
and a helper moved into a module of its own gets a unit test of its own in the same change
(ledger M88).

## B133: a parenthesis that was no gloss took the handle's gloss beside it

Red in `5f931f0a2`,
fixed in `754e50dd2`.
Found on 2026-10-05 (UTC) by a review of the T8 trial's cases,
behind a case named "PLACES NO GLOSS" that asserted a prefix of the damaged heading.
`appearancesIn` (`corpus-run/handle-gloss-place.ts`) read a handle followed by a parenthesis that was no gloss
(empty,
never closed,
or closed only by a mark on a later line)
as an appearance with no gloss.
Where that appearance was the handle's earliest,
`placeOne` wrote the gloss another appearance carried in front of the open parenthesis,
and stripped it from the appearance that had it:
a heading `### Ten: Jinmao (Brocade Cat` over a signature carrying `Jinmao (Brocade Cat)` became
`### Ten: Jinmao (Brocade Cat) (Brocade Cat`.
The same happened across two slices and for `Jinmao ()`.
The defect is older than the trial,
whose case pinned it.
Whether a shipped page ever carried a doubled gloss was not measured.

The fix:
such an appearance is left out,
as the module's rule for a parenthesis holding markup already had it (ledger A6),
and the scan goes on from the handle's end,
since the closing mark may belong to another appearance's gloss.
The case asserts the three pages come back whole with no finding,
and was red at `5f931f0a2` (1 [PASS] and 2 [FAIL] lines);
a second case asserts whole that a handle inside a longer word is no appearance,
which the old `toContain` could not tell from a gloss written after it.

Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail".

## B154: a contributor label or list item with spaces read the spaces, or the link, as the name

Red in `df0bcb85a`,
fixed in `ba243b3c3`.

Found 2026-10-05 (UTC),
checking B126's fix for the same shape.
`contributorForm` (`contributor-name-authority.ts`) took the text after a list marker as it stood
and the label between `[` and `](` as it stood.
A label of spaces read as the name `" "`,
a padded label as `" Whisker "`,
and a marker followed by two spaces left a space before `[`,
so `*  [Pebble](https://example.test/pebble)` read as its whole markup,
B126's shape again.

The fix:
the spaces after a marker are dropped,
and the label is trimmed,
so a label of spaces is empty and dropped as B126's empty label is.
Three cases hold the three inputs.
A reference link such as `[Whisker][w]` still reads as markup;
only the parse can tell whether it shows as a link.

No cached decision changes,
measured over the pinned corpus with the built package:
a digest of the forms each of 92 entries declares is byte-identical before and after,
and no entry holds an untrimmed or link-carrying form.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse".

## B222: the lone container tag masker read whitespace inside a tag otherwise than the strict grammar does

Red in `81ad0add4`,
fixed in `0b687fbdf`.

Found on 2026-10-05 (UTC)
by the agent applying the first review of the trial's added cases,
which probed the installed parser on that review's note about a closer written `</details >`.
`tagOnLine` (`mask-container-tags.ts`) ended an element name at the first space
and refused any closer carrying text past its name.
The strict grammar steps over whitespace between a name and the closing bracket:
it reads `</details >` as closing the element,
and refuses that closer standing alone.
The masker left such a lone closer unmasked,
so the parse refused a slice holding it,
the failure the masker exists to prevent.
With a tab the name itself went wrong.
A closer with a tab before its bracket got a name holding the tab and paired with nothing,
so a whole element closed that way read as two lone tags and was masked,
and an opener whose attribute follows a tab got a name holding the tab and the attribute.
How many pages of the pinned corpus hold either spelling was not counted.

The fix:
`nameEndOf` ends a name at the first space or tab,
and a closer may carry spaces and tabs after its name and nothing else,
so `</p x>` is still no tag.
Three cases failed on the lead's red tree,
each file run by name against the build of the red commit's parent:
"MASKS A LONE CLOSER WITH A SPACE BEFORE ITS BRACKET as the closer of its element,
since the strict grammar reads that spelling as closing the element and refuses it alone",
"MASKS A LONE CLOSER WITH A TAB BEFORE ITS BRACKET under the element's name alone,
and an opener whose attribute follows a tab under its name alone"
and "PAIRS AN OPENER WITH A CLOSER WRITTEN WITH A SPACE OR A TAB BEFORE ITS BRACKET,
so a whole element is masked nowhere".

The agent's measurements against `parseMdxBody`,
not rerun by the lead:
a tab,
a no-break space,
an ideographic space,
a form feed,
a vertical tab and a carriage return before the bracket all parse,
and a zero-width space is refused.

Open when this entry was written,
closed in B262 (`68df23b05`):
the masker matches a space and a tab alone,
so it still differs from the grammar on the other whitespace the grammar takes;
and `inline-container-tags.ts` reads tags its own way and was not examined.
A follow-up for both was launched on 2026-10-06 (UTC)
and merged in that commit,
which gives both readers the grammar's whitespace through one predicate.

The whole suite ran on `f7653073b` in a side checkout,
which holds the changes of B222 to B227:
1,621 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 36 source scans passing.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse":
a reader of a tag's shape is shown the strict parser's verdict for each whitespace spelling.

## B262: the container tag readers stepped over fewer whitespace characters than the grammar

Red in `5abbb9c66`,
fixed in `68df23b05`.

Found on 2026-10-05 (UTC) by a review of the masker's space and tab fix (B222),
by the report of the agent that fixed it,
and fixed by the follow-up B222 records as launched on 2026-10-06 (UTC),
closing the note B222 left open.
The masker ended a tag's name at a space or a tab,
and the inline reader at a space,
a tab,
a line feed or a return.
The strict grammar (`micromark-extension-mdx-jsx`,
`unicodeWhitespace`,
the ECMAScript `\s`) steps over twenty-five characters between a name and an attribute or the bracket
(`U+0009` to `U+000D`,
`U+0020`,
`U+00A0`,
`U+1680`,
`U+2000` to `U+200A`,
`U+2028`,
`U+2029`,
`U+202F`,
`U+205F`,
`U+3000` and `U+FEFF`)
and refuses every other,
a zero-width space among them.
So a tag with an ideographic space before its bracket was another element's name,
and a whole element read lone at both its tags.
A tag that crosses a line break was not read as a tag line at all,
and a lone one stayed in the slice for the grammar to refuse.

The fix:
one predicate,
`isTagWhitespace`,
the grammar's twenty-five,
is the whitespace of both readers,
and the masker reads a tag across lines when no blank line parts it and nothing follows its bracket.

The agent's measurement,
not rerun by the lead:
asked of the grammar at run time over thirty-four spellings in five forms,
the masker disagreed on 110 before and on none after.

Open:
a tag name holding a character the grammar refuses is still part of the name.

Open to the owner's veto:
the masker's reading of a tag across lines,
which goes beyond the spellings the review named.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse":
two readers of one construct share one predicate,
and a test asks the grammar for its verdict.

## B301: one reply nested thousands of markers deep ended the translate stage for every voice

Red in `c720721a9`,
fixed in `41d5f9267`.

Found on 2026-10-06 (UTC) by a follow-up to the cost B212 left open.
The Markdown parser descends once per nesting level,
and so does the tree transform it runs on its own output,
so a reply of thousands of nested quotation markers exhausted the stack.
Whether it did depended on how deep the caller already stood
(the agent measured 5,989 quotation markers parsed from a shallow caller and 5,957 through the translate stage),
so one reply could pass in one place and fail in another.
The strict entry turned the overflow into its own refusal,
with the message "at RangeError (RangeError)" and no cause.
The plain entry let the engine's `RangeError` out,
`requireMarkdownRefusal` took any `RangeError` for it,
and two readers of the plain entry had no catch:
the fold that lays a paragraph out as it renders
(`soft-break-fold.ts`,
read by `pageLayout` and two judge sheets)
and the floor that reads a rendering's atoms (`translate-unwrapped-link.ts`).
One voice's reply of 16,000 nested markers left `runTranslateStage`,
by the agent's trace through the first of the two,
and the voices that wrote ordinary renderings lost their work with it.
Other shapes never overflowed and cost minutes:
a link nested in a link,
an image in a link,
a list nested by indentation
(the agent's timings,
in its report).

The fix:

- `nesting-bound.ts` reads a body in one linear pass before any parse,
  against one bound:
  256 levels of the container markers opening a line,
  of a line's indentation,
  of a paragraph's open brackets and,
  under the strict grammar,
  of open tags and open braces;
  and 1,024 emphasis delimiters in a block.
  Fenced code and a line of rule characters are skipped for what they cannot nest.
- Both entries refuse a body past the bound with the refusal their callers already read:
  `MdxParseError`,
  or for plain markdown a new `MarkdownParseError`,
  a `RangeError` by class.
  The message names the measure,
  the bound,
  the line and the column,
  and quotes nothing.
- Both entries catch the engine's stack exhaustion,
  for a shape the lexical measure does not see,
  and raise the same refusal with the exhaustion as its cause.
  Anything else propagates unchanged.
- `requireMarkdownRefusal` reads `MarkdownParseError` alone,
  so another `RangeError` is no longer taken for the grammar's refusal.
- `foldSoftBreaks` returns a refused text as written,
  and the atoms floor reads no atoms from one;
  the floor that reads the candidate's shape refuses the same text with its own finding.

The lead's edits on the agent's work:
the byte order mark the scan drops is written as an escape,
where the agent's source held the invisible character itself;
and `MarkdownParseError` carries no `line` or `column` field,
since nothing read them and they were not adjusted for a leading mark as the strict refusal's are.

Measured by the lead on the red tree:
five test files fail as described and in no other case
(`translate-deep-reply` one case of two,
`wording-key` one,
`unwrapped-link` one,
`translate-validate` one,
`translate-floor-ground` two).
The agent's measurements,
not rerun by the lead:
the lowest overflow over every shape it tried is 2,995 bullet markers from a shallow caller;
the deepest page of the pinned corpus nests 3 container levels,
2 brackets,
2 open tags,
2 open braces and 36 delimiters in a block
(184 pages of 93 entries,
read through the package's own corpus readers);
a parse at the bound takes 8 to 330 ms for every shape measured,
and a body one past it is refused in at most 5 ms;
a scan of `src` for functions that reach themselves found 12,
none over text a model or a page wrote.
Its two cases at the stack's edge prove the catch at one measured edge each,
on one runtime build.
The whole suite ran on `41d5f9267` in a side checkout:
1,909 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 41 source scans passing.

Open,
with no fix in any commit through `f8e99b84d`:
callers of `parseDocument` that take text assembled from model output and have no catch
(`consolidation-polish-round.ts`,
`refine-phase.ts`,
`refine-slice-settle.ts`,
`repair-chunk-verdict.ts`,
`assembly-regressions.ts`,
`archive-footnote-order.ts`),
where the agent did not trace whether a floor refuses a deep reply first;
and,
found on the lead's reading,
inline counts are skipped inside what the scan reads as a fence,
and its fence rule is looser than the parser's,
so nested brackets after such a line are not counted.
There the catch still turns an overflow into the refusal,
and the slow bracket shapes are not bounded.

Open to the owner's veto:
the bound's values;
`MarkdownParseError` as a `RangeError`;
a cause on `MdxParseError` for the stack exhaustion alone;
an indented line of 514 or more columns refused;
markers and indentation counted inside a fence;
brackets and delimiters counted inside code spans and addresses;
a refused text folded as written and given no atoms,
where propagating is the alternative.

Recurrence:
`mistake-prevention.md`,
"Refusals inside a composed operation":
text one voice wrote is bounded before a parser that descends once per level reads it,
and every reader of that parser has its refusal.

## B320: an editor's edit nested past the bound ended the repair chunk after its checkers were paid

Red in `d0aebf8a7`,
fixed in `71702c550`.

Found on 2026-10-06 (UTC) by the agent following up B301,
over every place a voice's text reaches a parse no stage catches.
The editor's wire carries free text,
and the apply gate checked what an edit kept and quoted,
never whether a grammar reads what it leaves.
An edit of 300 nested list markers,
or of 300 open brackets,
passed the gate,
was judged and checked,
and was then parsed by `settleChunkFromChecks`,
which raised the plain grammar's `MarkdownParseError` out of `repairChunk`.
In a document of two slices,
both slices' repairs were lost,
and the chunk's checkers had been paid for nothing.
Quotation markers never reach the parse,
since `clampQuoteDepth` reduces them to one level.

The fix:
`applyPatchOperations` (`apply-patch.ts`) reads,
for each operation it accepts,
the text the accepted edits and the candidate leave
(`nestingWithEdit`,
`patch-nesting.ts`,
which runs `firstNestingExcess` under the plain grammar),
and rejects an operation past the bound as `unreadable-replacement` with the measure's reason,
through the gate's existing rejection path;
`editor-candidates.ts` records the reason against the voice.
The refine lane (`refine-stage-replies.ts`) names the same reason `refine-unreadable-rewrite` instead of throwing.
A case pins the boundary:
256 markers apply.
The lead moved the nesting exports out of `index.ts` into `nesting-barrel.ts`,
since the agent's lines would have taken `index.ts` past the 300 lines `eslint/max-lines` allows.
The whole suite on `71702c550`,
run in a side checkout:
2,010 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and 44 source scans passing.

Open to the owner's veto,
the agent's:
the gate reads the whole spliced text for each operation,
quadratic in a slice's operations;
the bounds of 256 levels and 1,024 delimiters stay;
no parse in the gate for exhausting the stack below the bound,
since by the agent's measurement the parser needs 82 to 1,175 frames against 13,899 available;
the parses that read lane output after a floor stay throws.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse":
a voice's text is refused as that voice's by its stage's own gate,
before judges and checkers.

## B321: the nesting scan skipped what a fence holds where the parser read no fence

Red in `d0aebf8a7`,
fixed in `71702c550`.

Found on 2026-10-06 (UTC) by the agent of B320.
The measure B301 added skipped every line after a fence line,
where the parser reads no fence for a backtick fence with a backtick in its info,
a fence indented four columns or a tab,
a fence in a quotation or list item the next line leaves,
or a fence line in raw html,
a math block or an indented code block.
Each passed texts the parser then nested:
by the agent's measurement,
to a tree depth of 303 for open tags and 3,003 for opened emphasis,
and in 700 to 2,200 ms for a link in a link 3,200 deep and emphasis 3,000 deep.

The fix:
the scan skips a fenced stretch only where the parser certainly reads a fence
(`certainFenceOf`,
`nesting-fence.ts`):
the line begins with the fence outside every container,
a backtick fence has no backtick in its info,
no raw html block is open (`nesting-html-block.ts`),
and under the strict grammar no math block has been seen and no brace or tag is open.
A fence line the scan cannot be certain of opens a loose fence whose lines are counted until a closing line.
`nesting-fence-differential.unit.test.ts` reads a generated family of texts against the grammar's own parser.
The agent's version pinned how many texts the scan passes (60,
36 and 36);
the lead's asserts that the passed texts are exactly those its stated rule skips,
and its oracle rethrows any failure of the plain grammar's parser.
By the agent's measurement,
the same family at its base showed 422 passed texts nested past the limit under the plain grammar
and 290 under each strict one,
and none after.

Open,
the agent's stated over-refusals:
a fence indented one to three columns,
a closing line indented four,
a fence in a container,
a fence line in an html comment or `pre` block,
every fence after a `$$` line under the strict grammar,
and a raw-text html block left open before a fence holding 256 or more brackets,
which the assembly guard then withdraws.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse":
the measure may refuse a text the parser reads,
never skip a stretch the parser nests.

## B322: the nesting scan read a line mixing stars and hyphens as a thematic break

Red in `d0aebf8a7`,
fixed in `71702c550`.

Found on 2026-10-06 (UTC) by the agent of B320.
`isRuleLine` took any line of stars,
underscores,
hyphens and blanks as a thematic break and skipped its emphasis delimiters,
where the grammar's break is three or more of one character.
By the agent's measurement,
`*-` repeated 24,000 times passed the scan and parsed in 3.7 to 6.6 s over four runs,
growing faster than the text.

The fix:
`isRuleLine` is the grammar's thematic break exactly,
three or more of one of `*`,
`_` and `-` with only blanks between,
and a mixed line is counted.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse".

## B323: a page nested past the bound by the join of two readable slices ended the document

Red in `d0aebf8a7`,
fixed in `71702c550`.

Found on 2026-10-06 (UTC) by the agent of B320.
An unclosed fence in one slice is closed by the fence line of the next,
whose open brackets then nest on the page,
so each slice read alone and the page did not.
`guardFootnoteAssembly` parsed the page and raised `MarkdownParseError` out of `repairPreparedDocument`;
the same guard runs for `translate-assemble.ts` and `page-assembly-rounds.ts`.

The fix:
`introducedStructuralRegressions` (`assembly-regressions.ts`) returns `unreadable-page`
when the assembled page is refused,
reading the incumbent first,
so an incumbent the grammar refuses still throws;
`assembly-integrity.ts` skips the footnote reading of an unreadable page,
and the guard's existing single-withdrawal proof withdraws the replacement that made it.
The lead's change to the agent's patch:
the function pairs each kind with the incumbent's count of it,
where the agent's version looked the count up by position with a `?? 0`,
which would have read a missing count as zero.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse":
the assembly guard reads the page,
not only the slices;
and "Defaults that stand in for an input" for the positional zero.

## B326: three readers of a tag name each kept a name of their own

Red in `75ea14bff`,
fixed in `3ce947dce`.

Found on 2026-10-06 (UTC),
the residual B262 left on its container tag readers.
The lone container tag masker took an ASCII letter and then everything up to the first whitespace,
the inline reader took that name and looked one character past it,
and the attribute restorer (`corpus-run/tag-attributes.ts`) took an ASCII letter and ASCII alphanumerics with `-`,
`.` and `:`.
By the agent's differential against `parseMdxBody`,
over 148,252 spellings in each of three places of a name,
the masker refused 49,018 starts the compiler accepts
(`$`,
`_` and every letter outside ASCII)
and read 97,431 characters inside a name the compiler refuses
(a zero-width space,
a bidirectional control,
a soft hyphen,
an `@`),
so a lone tag of such a name was masked as an element of that name where the slice should have been refused;
and over 282 spellings the restorer read 24 the compiler refuses and missed 84 it accepts.
The compiler hands the grammar UTF-16 units,
so a letter past U+FFFF starts and continues no name.

The fix:
one `readTagName` (`mdx-tag-name.ts`),
written from the states of `micromark-extension-mdx-jsx` 3.0.2
and the classes of `estree-util-is-identifier-name` 3.0.0,
used by the masker,
the inline reader and the restorer,
its start set shared with `opensMdxTag` (`mdx-tag-start.ts`).
`tag-name-spellings.test-fixture.ts` holds the differential set,
and each reader's test holds the reader to the strict grammar over it.
The agent's case pinned how many spellings the grammar accepts (106);
the lead's asserts that both verdicts occur,
and a check of a list's length against itself is gone.
The lead's change to the module comment:
it names all three readers,
where the agent's named two.

Open:
`nesting-inline-count.ts` reads a tag name of its own,
not brought to the definition and not measured;
`tag-attributes.ts` still reads attribute names with its ASCII set,
unmeasured.
Both closed since:
the nesting count by B344,
the attribute names by B345.
Corrected on 2026-10-06 (UTC):
the spelling set this entry counts as 282 yields 283 spellings,
277 of them distinct
(the lead's count of `tagNameSpellings()`,
unchanged since `75ea14bff`).

Open to the owner's veto:
`opensMdxTag` stays start-only,
so an angle autolink stays a protected span,
and its start set is the Basic Multilingual Plane's;
the restorer reads tags named in any script and refuses 24 shapes it read.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse":
one definition of a construct's name,
held by a differential case over every class of character against the compiler.

## B344: the nesting count read tag names its own way

Red in `de6ff17`,
fixed in `a7eca7f`.

Found on 2026-10-06 (UTC),
the nesting count's own tag-name reader B326 left open.
`nesting-inline-count.ts` opened a tag at `<` followed by an ASCII letter.
By the agent's differential over the 283 spellings of the tag-name set,
of the 106 the strict grammar opens a tag for it missed 8
(names starting with `$`,
`_` or a letter of another script),
so a body nesting such tags past the bound passed the measure,
and it opened 146 the grammar refuses.

The fix:
the count reads a tag through `readTagName`,
in the line followed by `LINE_GOES_ON`
(a line ending,
a name letter and a bracket),
since the grammar steps over a line ending inside a tag
and the next line is out of the scan's sight.
By the agent's probe the grammar reads a tag split after its name,
after a member's dot or a colon,
or after `</`,
as one element.
A closer begun with `</` at a line's end now closes on the next line,
where the count kept the tag open.
The count and the grammar agree on all 283.
The opener half of the line-end case passed on the old count,
and the agent showed it failing with `LINE_GOES_ON` empty.

Open to the owner's veto:
`LINE_GOES_ON` is an invented continuation,
measured against the grammar,
where the alternative hands the scan the rest of the body and changes `nesting-bound.ts`.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse".

## B345: the attribute restorer read attribute names its own way

Red in `de6ff17`,
fixed in `a7eca7f`.

Found on 2026-10-06 (UTC),
the attribute-name reader B326 left open.
`corpus-run/tag-attributes.ts` read an attribute name as ASCII letters,
digits,
`-`,
`.` and `:`.
By the agent's differential,
each of the 283 spellings written as an attribute's name (`<Cat NAME="1"/>`),
the strict grammar reads one attribute on 75;
the reader dropped 61 of those tags whole,
read a name on 32 the grammar refuses,
and read the prefixed `a : b` as `b`.

The fix:
`readAttributeName` in `mdx-tag-name.ts` reads a name as the grammar's states do
(`micromark-extension-mdx-jsx` 3.0.2,
`factory-tag.js`,
`attributeBefore` to `attributeLocalNameAfter`):
a primary name,
then optionally `:` and one local name,
with whitespace allowed around the colon,
each part taking the characters a tag name's parts take,
and no `.` member.
The restorer reads through it,
keeping a prefixed name without the whitespace around its colon,
and the reader and the grammar agree on all 283.
The lead probed a name starting with a letter past U+FFFF:
the compiler refuses it as a tag name and as an attribute name,
and `readTagName` reads none,
as B326 found for tag names;
the spelling set holds no such character.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse".
