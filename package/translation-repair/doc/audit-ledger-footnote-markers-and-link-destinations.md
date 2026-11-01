# Audit ledger: footnote markers and link destinations

Scans for footnote markers and link destinations that disagreed with the parse,
about where a marker is a reference or where an address ends.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B123: a link literal the parser builds without a position makes the footnote relabel refuse the page

Status:
fixed in `9d7bbbec2` (red case in `23137d784`),
2026-10-04.
Found 2026-10-02 (UTC) reviewing what the T8 bundle 1 agent left,
and written only into `23090a91d`'s commit message until 2026-10-03,
when the handover to the trial model copied it here.
`activeFootnoteMarkers` (`src/active-footnote-markers.ts` (moved on 2026-10-18 to
`package/translation-repair/shared-contract/src/active-footnote-markers.ts`)) walks the parsed body
and reads each text node's raw bounds off `node.position`;
a text node without one throws `FootnoteRewriteError` of kind `position`
(the check reading `start` and `end` in the text-node arm of the walk).
The commit message names the source of such nodes:
`mdast-util-gfm-autolink-literal`'s transform builds text and link nodes with no position
wherever its pattern finds a literal micromark did not tokenize,
such as `www.` after a quote mark,
a comma or CJK punctuation,
and any literal after an unbalanced `[`.
On a page holding a footnote and one such literal,
the relabel refuses the whole page and the archive is kept.
That session's census of the 279 pinned `.md` and `.mdx` files found none that triggers it
(`b1-review/corpus-unpositioned.mjs` in its scratch folder).
Bundle 1 left this stretch uncased on purpose,
since a case written against the code as it stands would pin the refusal.

Measured on 2026-10-03 against the build of `98e94e584`,
calling `applyFootnoteRelabel` with the one rewrite of label `1` to `2`:
`A cat[^1] naps.` with its definition relabels,
while the same text holding `"www.example.com"`,
`[www.example.com`,
`，www.example.com。`
or `,www.example.com`
throws `FootnoteRewriteError` with the `position` kind's message
(`footnote rewrite: parsed marker positions do not match raw syntax; ...`).

The fix reads the raw behind each unpositioned run (`9d7bbbec2`),
the run bounded between its positioned neighbours
(the parent's own bounds at either end)
and scanned with `gfmMarkerSpans`,
which is escape-aware and reads labels as
`micromark-extension-gfm-footnote` 2.1.0 does.
The installed tokenizer settles the design:
a call forms only when its identifier is defined somewhere in the document
(`callData`'s `defined.includes` gate at `]`),
so an undefined `[^9]` stays in the text
and the walk's literal-looking recovery exists for exactly it.
That category rides in unpositioned runs too
(the probe put `，www.example.com[^9]` in one),
so skipping unpositioned nodes would lose the recovery;
a check off node values would mistake an escaped opening for a marker,
since a decoded value erases the escape backslashes;
and reconstructing positions would authorize edits off offsets no token
carried.
Probes on the built package:
`~/temp/agent/mimo-trial/b123-probe.mjs` and `~/temp/agent/mimo-trial/b123-probe2.mjs`.
No cache version moves:
the pinned corpus at `a41fc607e` holds 279 `.md` and `.mdx` files,
68 with marker text,
and none of those parses into unpositioned nodes
(`~/temp/agent/mimo-trial/b123-corpus-scan.mjs`),
so the relabel changes nothing a cache entry keys on.
The red case and three contract cases
(the undefined reference riding with the literal relabels in both
measured shapes,
an escaped opening and the malformed ones stay byte-identical)
sit in `apply-footnote-relabel.unit.test.ts`.

## B126: a link's empty label returned its markup for a name

Red in `98e94e584`.
A T8 bundle agent's case in `contributor-name-authority.unit.test.ts` wrote it:
"READS NO NAME from a link whose label is empty,
which shows a reader nothing,
rather than its markup (T8)".
`contributorForm` (`contributor-name-authority.ts`) finds a link's label end with
`unmarked.indexOf('](')`
and returned the token unchanged when `labelEnd <= 1`.
That one check joined two cases:
`-1`,
no `](` at all,
where the token is a plain name such as `[Whisker`,
and `1`,
an empty label `[](...)`,
whose visible form is the empty string.
The empty label instead returned its whole markup:
the case expected `['Pebble']`
and measured `['[](https://example.test/whisker)', 'Pebble']`.

The fix:
the check returns the token only where no `](` exists (`labelEnd === (-1)`),
so an empty label slices to the empty string,
which `archiveContributorNameForms` drops through its `nonempty` filter,
and the `@returns` names the empty link label beside the empty token.
The reader keeps its scan;
moving it onto the Markdown parse,
which the recurrence's rule asks for,
is a larger change no finding here asks for.

No cached decision changes,
inference from one search of the pinned corpus:
`git -C ~/one-among-us/data grep --line-number --fixed-strings '[](' a41fc607ea5a70d8a7625cc67d5ed8c444f53379`
prints six lines and every one reads `Binary file ... matches`
for a photo under `people/`,
no text line,
so no contributor line (`CONTRIBUTOR_LABELS`) carries an empty-label link
and the contributor floor protects the same pages as before.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse".

## B144: the run scan read a link's own URL and a tag's attribute as footnote markers

Red in `adce3f9a8`,
fixed in `3ba21f95c`;
B123 continued.

Found reviewing the B123 fix (`9d7bbbec2`,
`d59d9851e`).
The fix scanned the whole raw region of an unpositioned run with `gfmMarkerSpans`,
its link members included,
and gave a run at the head or tail of its parent the parent's whole span.
So `A cat naps ，www.example.com[^9] tail.` under the rewrite of `9` to `10` came back `，www.example.com[^10]`,
though the parse holds `[^9` in the link's `url` and text and only `]` as text;
`[^www.example.com]` and `[^paw@cat.example]` were read as references with a link standing between their brackets;
and `<Cat title="[^1]">，www.example.com</Cat>` under `1` to `2` rewrote the attribute.
Each breaks the promise of `applyFootnoteRelabel` that URLs and attributes stay byte-identical,
and each inflated the label inventory `documentLabels` returns.
Three of the cases added with the fix asserted the rewritten URL.
The fix:
`footnote-unpositioned-runs.ts` pairs every square bracket of a run's raw region with the same bracket of the run's decoded text,
in order,
counting a character reference that spells a bracket as one,
and keeps a marker lexeme only where both its brackets fall in text nodes no link separates.
A run that opens an `mdxJsxTextElement` starts after the element's last attribute;
a run at the edge of any parent but a paragraph,
heading,
emphasis,
strong,
delete,
table cell or JSX text element,
a run member that is neither text nor a link of text,
and bracket counts that still differ
refuse with the `position` kind.
Measured:
56 constructs agree with the parse (`probe-3-matrix.mjs` in the fix's scratch),
12,000 generated pages agree with an oracle that aligns characters instead of counting brackets,
while the unfixed build differs on 2,942 of 5,978,
and none of the 279 Markdown files of the corpus clone reads differently.
Recurrence:
`mistake-prevention.md`,
"Structure read off the parse":
a raw region read in place of nodes is checked against those nodes' own values before any span of it is acted on.

Open when this entry was written,
fixed in B159:
`collectBlockHits` (`footnote-graph.ts`) reads the start and end of every `text` node through `nonNullishOrThrow`,
so `A cat ，www.cat.example naps.` makes `parseDocument` throw `Expected non-nullish value, got undefined`,
with or without a footnote on the page.
`reorderFootnoteDefinitions` and `footnoteProtectedRanges` call it,
so the relabel pass ends on a throw that is no rewrite refusal
for the very pages B123 was opened for.
The module is outside this fix's files.

## B145: a marker shape inside a tokenized autolink literal was relabelled

Red in `adce3f9a8`,
fixed in `3ba21f95c`.

Found with the B123 review,
older than the trial:
the walk at `28303c42a` read every positioned `text` node,
the one text child of an autolink literal among them,
and that text is the URL.
`A cat naps https://cat.example/[^9]x tail.` under the rewrite of `9` to `10` came back `https://cat.example/[^10]x`.
The fix:
`isAutolinkLiteral` (`active-footnote-markers.ts`) reads a link whose span starts where its first child starts,
or that carries no span,
as a literal,
and the walk does not descend into it.
A labelled link keeps its label read,
since a bracket sets the label off from the link's start.
No page of the corpus clone holds a literal whose text carries a marker opening
(count over 279 files),
so no cached decision moves.
Recurrence:
`mistake-prevention.md`,
"Structure read off the parse".

## B159: a page holding an untokenized link literal made the document parse throw

Red in `4cf768073`,
fixed in `b4030dd01`;
the lead B144 left open.

Found reviewing the B123 fix.
`collectBlockHits` (`footnote-graph.ts`) read the start and end of every `text` node through `nonNullishOrThrow`,
and the autolink-literal transform builds text and link nodes with no position,
so `parseDocument` threw `Expected non-nullish value, got undefined` on `A cat ，www.cat.example naps.`,
with or without a footnote on the page.
`parseDocument` runs on every page the pipeline prepares,
so `prepareDocumentPair`,
`reorderFootnoteDefinitions`,
`footnoteRelabelOf` and `relabelArchiveFootnotes` all threw on such a page:
thirty calls over six pages,
the fixing agent's probe;
the lead measured `parseDocument` throwing on three such pages before the red commit.

The fix:
the graph reads text raw in three shapes,
a positioned text node's span,
an unpositioned run's region through `footnote-unpositioned-runs.ts`,
whose GFM lexemes count only where both brackets fall in text no link separates,
and a tokenized literal's span for full-width markers alone.
A full-width marker counts wherever text shows it,
link text included,
since micromark and the transform both take a glued `〔N〕` into the link
and the graph already counted it there.
The graph no longer reads a GFM shape inside a tokenized literal's URL,
as the marker reader does not.
`computeFindings` moved unchanged to `footnote-graph-findings.ts` as `footnoteGraphFindings`,
since the fix took `footnote-graph.ts` past the line limit.
The fixing agent read every other reader of node positions over 57 modules:
none reads a position off a text or link node,
the two the transform rebuilds.

Measured by the fixing agent:
a differential fuzz over 12,000 generated pages,
under the strict and the plain grammar,
agrees with the B123 oracle and with a decoded-value scan,
while its control perturbed 2,688;
and the 279 Markdown files of the pinned corpus read the same before and after,
none holding an unpositioned node,
so no cached decision moves.

Open when this entry was written,
closed in B181 and B182,
with what B182 leaves open:
an angle autolink under the plain grammar still reads a marker shape in its URL,
since its text starts after the `<` and `isAutolinkLiteral` does not take it for a literal;
and `footnoteMentions` scans raw text,
counting a `[^9` inside a URL as a mention.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse":
a walk that reads positions off phrasing nodes names which node types it reads
and refuses the rest loudly.

## B173: an explicit link destination was cut where prose would end an address

Red in `5df296c5f`,
fixed in `25c33df4f`.

Found on 2026-10-05 (UTC),
as an item ledger B139 left open.
`markdownDestinations` (`corpus-run/dropped-destinations.ts`) sent every tree destination through `trimDestination`,
which cuts at the first stopper and sheds trailing sentence punctuation.
That is right for an autolink literal,
whose address prose ran into,
and wrong for a destination its author wrote out:
two explicit links differing only past a full-width comma read as one,
and a source linking both against a page keeping the first reported nothing dropped.

The fix:
a link is an address prose ran into when it carries no span,
or its first child starts where the link starts,
the test `isAutolinkLiteral` (`active-footnote-markers.ts`) makes for the footnote relabel;
`isProseAddress` applies it,
and only such a link is trimmed.
Every other link,
every image and every definition stands as written.
`trimDestination` then meets only addresses that open on a scheme's letter,
so its emptied arm is an `unreachable:` throw.
The red run printed three [FAIL] cases in `dropped-destinations.unit.test.ts`,
two of them these;
a case pinning the literal arm was green before and after,
and fails with that arm switched off.

No pinned page reads differently
(the fixing agent's measurements):
over the 276 page files at the pin neither reader's output differs between the builds,
where an explicit destination holding a full-width comma appended to each differs on all 276,
and original against archive drops 14 destinations on both builds.
Over 249 pages rebuilt from settled artifacts on the audit machine,
one entry's page reads one more destination,
an explicit one a model wrote that ends in a closing parenthesis,
and no dropped list changes.

Open to the owner's veto:
an explicit destination keeps its trailing sentence punctuation too;
and the scanner still reads such a destination's address off the raw text and stops at the stopper,
so a source writing an address with a closing parenthesis inside as an explicit link,
against a page writing the same address bare,
now reports the explicit destination dropped.

Open when this entry was written,
closed in B213:
the literal test stands in two files,
since `isAutolinkLiteral` is private to its module;
and a closing parenthesis is a run stopper,
so a bare address holding a balanced pair of parentheses reads short on the scanner and on a literal alike
(the fixing agent's observation).
A prose cut that kept a balanced closing parenthesis would remove the difference between explicit and bare
that this fix opened.

Recurrence:
`mistake-prevention.md`,
"Defaults that stand in for an input".

## B174: a dropped destination was traced to every slice holding it as a substring

Red in `5df296c5f`,
fixed in `25c33df4f`.

Found on 2026-10-05 (UTC),
as an item ledger B139 left open.
`traceDroppedDestinations` (`corpus-run/destination-completeness.ts`) asked `text.includes` of each slice.
A dropped `https://example.org/tabby` was traced to a slice holding only `https://example.org/tabby-album`,
and a dropped `.` to every slice holding a full stop,
the shipped text among them,
which the trace's own type says is empty since the page drops it.

The fix:
each slice's text on each side is read by `collectDestinations`,
the readers that found the drop,
and a slice carries an address when one of its destinations has the same `sameAddress` key.
Nothing is read when nothing was dropped.
The red run printed two [FAIL] cases in the new `traceDroppedDestinations` suite.

No real trace changed
(the fixing agent's measurement):
14 traces over the pinned corpus,
original against archive,
and 5 over 249 pages rebuilt from settled artifacts read the same under both rules.

Open when this entry was written,
closed in B217:
`DroppedDestinationError` still words its count as `destination(s)` and several slices as `slice 4, 7`;
the fix needs a line in the message inventory.

Recurrence:
`mistake-prevention.md`,
"Defaults that stand in for an input".

## B181: the footnote graph read a marker shape in the URL of an angle autolink

Red in `e3c217fb0`,
fixed in `ab11a69d6`.

Found on 2026-10-05 (UTC),
as an item ledger B159 left open.
`<https://cat.example/[^9]x>` on a page the strict grammar refuses,
so `parseDocument` reads it under the plain grammar,
produced a gfm reference `9` and an `unresolved-reference` finding,
since the link's span starts at the `<`,
one character before its text,
and `isAutolinkLiteral` takes only a link whose first child starts with it.

The fix:
`isAngleAutolink` (`footnote-unpositioned-runs.ts`) takes a positioned link whose raw starts at `<`,
and the graph treats it as a tokenized literal:
full-width markers in its span are read,
a GFM shape in its URL is not,
and its children are not descended.
The marker reader (`activeFootnoteMarkers`) is strict-only,
and the strict grammar refuses every angle autolink,
so it never meets one and is unchanged
(the fixing agent's probes;
that the refusal is the strict parser's is its inference).
A link with a label is told apart by its `[`;
a case pins a marker in a label beside an angle autolink.

Open to the owner's veto:
the raw `<` as what tells an angle autolink,
where its child's span and its `url` were the other way.

Recurrence:
none new;
`mistake-prevention.md`,
"Structure read off the parse",
already names the rule that a walk says which node shapes it reads.

## B182: a footnote marker shape inside a link's URL counted as a mention

Red in `e3c217fb0`,
fixed in `ab11a69d6`.

Found on 2026-10-05 (UTC),
as an item ledger B159 left open.
`footnoteMentions` scanned raw text,
so `[^9]` in `https://cat.example/[^9]x`,
`[a](https://cat.example/[^9]x)` or `<https://cat.example/[^9]x>` counted as a reference to note 9.
`suspectsFor` (`assembly-integrity.ts`) compares counts of one scanner and was unaffected,
while `droppedMarkerFindings`,
`definitionLeakFindings`,
`definesIdentifier` and `labelsOf` read the mentions of an original alone,
so a candidate that dropped or rewrote the link was told to keep a marker the page never had.

The fix:
`urlSpansOf` (`footnote-url-spans.ts`) finds the spans of inline destinations,
angle autolinks and bare literals (`http://` or `https://` or `www.`) in one linear pass
that stops each scan at whitespace or `<`,
and `footnoteMentions` drops GFM hits inside them;
a full-width marker in a URL still counts,
as the graph counts it.
A marker in a link label or beside a URL still counts.
The scan stays lexical because the module reads fragments and does not parse them;
a marker in a code span or a reference definition's destination still counts and is not part of this finding.

Open when this entry was written,
closed in B212,
found by the lead's probes of the build of `ab11a69d6` against the footnote graph,
28 invented texts of which 5 differ.
The span reader begins a bare literal only after whitespace or one of four characters,
where the installed parser begins an `http` or `https` literal after any character that is no ASCII letter;
so a marker shape in a URL that follows a Han character,
a digit or a full stop is still counted.
Whitespace before an inline destination hides the destination from the reader.
A scheme followed by no domain character is read as a literal,
and so is a URL-shaped run inside a link label;
in both the parse reads the marker as a reference,
so the scan drops a mention the graph counts,
which is the direction the guards reading mentions cannot afford.
Whether any page at the corpus pin holds a marker shape inside a URL was not measured.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse":
a raw scan that feeds a guard names the contexts it skips,
and a context the graph skips is skipped here too.

## B201: a reference link in a contributor form read as markup where the text defines it

Red in `6f61ba5dd`,
fixed in `72f5be638`.

Found on 2026-10-05 (UTC)
by the agent that fixed B154,
among what it left.
`contributorForm` (`contributor-name-authority.ts`) read `[Whisker][w]` as the whole markup,
whether or not the text defined `w`.
Only the parse tells:
a reference shows as a link where the document defines it,
and as literal text where it does not.

The fix:
`archiveContributorNameForms` parses its text once when it holds `[`
and reads each reference link,
full,
collapsed or shortcut,
as the label it shows;
a reference with no definition keeps its markup,
which is what the page shows.

The fixing agent's version read the label by arithmetic:
it counted back from the end of the markup by the length the parser reports for the reference's own label.
That length is of the decoded label,
not of what is written,
so a reference label holding an escape or an entity moved the cut.
The lead's probe of the agent's build read `Whisker]` out of `[Whisker][a\]b]` and out of `[Whisker][w\*x]`,
and `Whisker][w&` out of `[Whisker][w&amp;x]`
(`merge-w35/probe-reference-label.mjs` in the audit's scratch folder,
with a log for each build).
The label is read off the parse now:
it ends where the link text's last inline node ends,
and a node with no end offset throws `unreachable:`.
The lead's red case holds the three shapes.

No form changes at the pin
(the fixing agent's count):
the 92 archive pages hold no reference link.

Open to the owner's veto:
a text the parser cannot take,
nested past its stack,
is read as defining no reference,
with a debug line;
and the lead's replacement of the agent's label arithmetic.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse":
an offset inside a node is read off its children,
never worked out from the length of a decoded value.

## B212: the footnote mention scan disagreed with the parse about where a marker shape is no reference

Red in `9b872bbea`,
fixed in `cf50117c5`.

Found on 2026-10-05 (UTC)
by the lead's probes of the lexical URL reader B182 added,
as the item B182 left open.
`footnoteMentions` (`footnote-mentions.ts`) counts GFM marker shapes by scanning a fragment's raw text,
and asked that reader (`footnote-url-spans.ts`) which of them sit in a URL.
The reader began a bare literal only after whitespace or one of four characters,
where the parser begins one after any character that is no ASCII letter,
and it knew nothing of code spans,
images,
link reference definitions,
comments or the markup around a link's label.
Four guards read the scan:
`assembly-integrity.ts`,
`translate-marker-drop.ts`,
`translate-definition-leak.ts` and `corpus-run/insertion-referenced-definitions.ts`.
A mention wrongly counted tells a candidate to keep a marker its page never had,
so a good candidate is refused or withdrawn;
a mention wrongly dropped leaves a real reference unguarded,
which is the direction those guards cannot afford.

Measured by the fixing agent on the build before the fix,
as texts whose scanned references differ from the footnote graph's:
90 of the 210 texts of the lead's URL table,
6 of them lacking a reference the graph reads;
5 of 28 edge texts,
2 of them lacking one;
and 1,603 of 3,000 texts from a seeded generator,
166 of them lacking one.

Three designs were weighed.
Correcting the lexical reader rule by rule was rejected:
it means writing code spans,
link labels,
rebuilt literals and the Han boundary by hand,
which is the parser again.
Taking the document parse of the fragment as the mentions was rejected:
a fragment opening on a thematic break reads as front matter and loses its marker,
one opening on a malformed front matter block throws `FrontMatterParseError`,
and a fragment that opens a fence or a comment for the page loses the markers after it.
Chosen:
the lexical hits stay,
which keeps the roles,
the overflow refusal and the full-width markers as they were,
and a hit is dropped where a parse of the fragment reads no reference.

The fix:
`footnote-parsed-spans.ts` (`parsedSpansOf`,
`insideParsedSpan`) masks invisible lines and HTML comments as the document parse does,
splits no front matter,
parses the masked fragment under plain markdown and under strict MDX,
and keeps only the regions both grammars read as no reference.
Where strict MDX refuses the fragment,
plain markdown decides alone;
where the plain parser refuses it for its nesting,
no region is reported and every shape counts,
as `corpus-run/prose-ranges.ts` already answers that refusal.
The regions are a tokenized or angle autolink,
the marker shape of a literal the autolink transform builds,
the markup around a link's or a reference link's label,
an image,
a link reference definition,
an inline code span and a masked comment.
A shape in a code block,
a raw HTML node or a JSX attribute still counts,
since a fragment cannot settle those for the page,
and a full-width marker counts as the graph counts it.
`footnote-url-spans.ts` and its test file are deleted,
and the footnote barrel exports the new pair in their place.

Both grammars are asked because they differ in block structure.
A fence indented four spaces is a code block to strict MDX and a paragraph line to plain markdown,
where the backticks before it pair into a code span;
a parse under plain markdown alone dropped a marker the page,
read as MDX,
holds as a reference.
Keeping a region only where both grammars read it removed every such text the agent's generator found.

After the fix,
by the fixing agent's runs:
the lead's three probes agree with the graph,
the 28 edge texts among them;
seeded runs of 3,000 to 60,000 generated texts,
eleven runs,
found no text where the scan lacks a reference the graph reads,
and none where it counts more than the graph outside a code block or a raw HTML node;
over the 276 page files of the pinned corpus,
68 of which hold a marker shape,
the scan's GFM references equal the document parse's in every file,
and no file and none of 8,188 blank-line chunks differs between the scan before and after,
while the control texts do differ;
and the scan costs about 200 to 270 ms over those 68 files,
against 15 to 25 ms before,
a text holding no marker shape taking a path that parses nothing.

Cases:
`footnote-mentions-agreement.unit.test.ts` runs the scan and the graph on every text of the lead's table,
the edge texts and 3,000 seeded texts,
storing no expectation for them,
and names the two shapes where the scan counts more;
`footnote-parsed-spans.unit.test.ts` holds each region,
the unclosed comment,
the region only one grammar reads and the nesting the plain parser refuses;
and the edge case of `footnote-mentions.unit.test.ts`,
two of whose thirteen expectations the parse does not give,
is replaced by four.
Three of those cases failed on the lead's red tree.
The agent's own red run of them came after its fix,
against the restored base sources.

At the merge the lead removed the deleted file's two keys from the blankness scan's held list,
and corrected the header comment and one TSDoc of `footnote-mentions.ts`,
which still described one grammar and no masked comment.
The whole suite ran on `cf50117c5` in a side checkout:
1,607 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 36 source scans passing.

Cache:
no version moved.
The cache account audit of 2026-10-05 (UTC) found the newest slice-cache record written 2026-09-27T04:26Z,
with every cache version set since then,
so no stored record exists under a current version.

Not checked by the agent:
whether every site that slices a page cuts at a top-level block's offset,
which the module's reading of a fragment rests on and which was read from a comment in `parse-document.ts`.

Open to the owner's veto:
a marker shape in a masked comment is no longer a mention,
where the scan counted it;
a shape in a code block,
in raw HTML or in a full-width marker stays counted;
a nesting the plain parser refuses counts everything;
and the exports `urlSpansOf` and `insideUrlSpan` are gone.

Open:
a footnote definition indented four or more spaces is a definition to the scan and a reference to the graph,
for which a follow-up was launched and had not handed back when this was written;
and a text nested thousands of block quotes deep costs the parser itself,
3.2 s at 30,000 markers and 14.8 s at 60,000 by the agent's timing,
with no check made of whether another stage parses such a candidate first.
Still open at `94de8258e`:
by that timing the cost grows faster than the text,
about 4.6 times the time for twice the markers,
it is the Markdown library's own,
and no follow-up was launched for it.
Both are closed since:
the indented definition in B258 (`68df23b05`),
and the nested text in B301 (`41d5f9267`),
which bounds the nesting a parser is handed and does not make the parse linear.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse":
a scan of a fragment asks a parse of the fragment where its shapes are something else,
under every grammar the page may be read by.

## B213: an address was cut at its first closing parenthesis

Red in `9b872bbea`,
fixed in `cf50117c5`.

Found on 2026-10-05 (UTC),
as the two items ledger B173 left open.
`corpus-run/dropped-destinations.ts` ended an address at its first closing parenthesis (`firstStopper`),
on the scanner and on a literal alike,
so a bare `https://en.example/wiki/Cat_(animal)` read as the address without its last character,
and a source linking that address explicitly against a page writing it bare reported the destination dropped
(the fixing agent's probe on the build before the fix).
The test for an autolink literal also stood twice,
as the private `isProseAddress` beside the exported `isAutolinkLiteral` (`footnote-unpositioned-runs.ts`).

The parse's rule,
as the agent read it in the installed autolink-literal syntax (`tokenizePath` and `tokenizeTrail`):
a closing parenthesis is part of the path while fewer have closed than opened,
and a run of sentence punctuation and closing parentheses that reaches the end of the address
is shed from where it starts.

The fix:
`addressEnd` and `trailRunEnd` replace `firstStopper`.
An address keeps a closing parenthesis that balances an opening one,
ends at one that balances none,
and ends at sentence punctuation that only closing parentheses follow,
as in `(cat.)`.
The cut is one pass:
a run that turns out to be no trail is not scanned again.
`scanUrlRuns` and `trimDestination` both use it,
and `isProseAddress` is gone in favour of `isAutolinkLiteral`,
a change with no red case of its own.

After the fix,
by the fixing agent's runs:
the explicit and the bare writing of one address report nothing dropped in either direction;
none of the 276 page files of the pinned corpus reads differently through `collectDestinations`,
while a control text does,
and one page holds an address with an opening parenthesis;
and the destination the tree reader gives equals the parse's address
in 26,012 of 40,000 generated texts holding a link,
against 15,757 before.

Four cases in `corpus-run/dropped-destinations.unit.test.ts` hold the balanced parenthesis in a bare address,
the trail that ends a run,
the autolink literal with a `www` literal beside it,
and the explicit and bare pair in both directions.
All four failed on the lead's red tree;
the agent's run has no red log for the trail case,
which it wrote after its last change to the scanner.

Open to the owner's veto:
sentence punctuation standing before a closing parenthesis is shed where only closing parentheses follow it.

Open:
the scan does not shed a trailing underscore,
asterisk or tilde before a closing parenthesis as the parse's trail rule does,
which it did not before the fix either;
about 450 of 40,000 generated texts show the difference,
by the agent's count.
A follow-up was launched and had not handed back when this was written.
It has since:
B259 (`68df23b05`) sheds the three marks as the parse's trail rule does.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse":
a cut that stands for the parser's is written from the parser's own rule for it.

## B258: the mention scan read a definition after any indentation, which the graph reads under one grammar only

Red in `5abbb9c66`,
fixed in `68df23b05`.

Found on 2026-10-05 (UTC) by a review of `opensDefinition`,
the rule B212 added.
The scan called `[^1]:` a definition after any leading whitespace.
Strict MDX has no indented code and reads a definition at four spaces or a tab;
plain markdown reads an indented code block,
or a continuation line of a paragraph when no blank line parts the two.
The footnote graph reads the grammar the page is parsed under,
so on a page downgraded to plain markdown the scan and the graph disagreed,
and on any page for a label nested in a list item or in another definition,
which the graph does not read.

The fix:
`fragmentReadingOf` replaces `parsedSpansOf`:
one parse gives the spans and the offsets where the page's own grammar opens a top-level footnote definition,
strict MDX where it accepts the text and plain markdown where it does not,
containers dissolved as the footnote graph dissolves them.
`footnoteMentions` calls a marker a definition only at such an offset.
The definition starts live in the new `footnote-definition-starts.ts`.
Where plain markdown cannot read the text for its nesting,
the line alone decides,
as before.

The agent's measurement,
not rerun by the lead:
over 96 generated pages
(48 fragments,
each read as strict MDX and as plain markdown)
the scan disagreed with the graph on 36 before and on 0 after.

Open to the owner's veto:
a slice is read as strict MDX where the strict grammar accepts it,
an inference from the corpus compiling as MDX and not a measurement of a slice inside a page that downgraded;
passing the page's grammar down from `validateTranslatedSlice` is the fuller fix.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse":
a role a scan gives a marker is asked of the parse the page is read under.

## B259: the destination scan kept the emphasis marks the parse's trail rule sheds

Red in `5abbb9c66`,
fixed in `68df23b05`.

Found on 2026-10-05 (UTC) by comparing the scan with the tree reader on generated texts.
The trail rule of `micromark-extension-gfm-autolink-literal` (`tokenizeTrail`) reads `_`,
`*` and `~` beside the sentence punctuation as no part of an address,
and the scan's `RUN_TRAILERS` held only the punctuation.
`see https://c.example/a_)` therefore named `https://c.example/a_` to the scan
and `https://c.example/a` to the tree reader.
A scanned run was also cut a second time,
without the characters that follow the run.

The fix:
the three marks are trailers,
and `trimDestination` takes where the address ends,
so a scanned run is shed of its trailers and not cut again.

The agent's measurements,
not rerun by the lead:
over 40,000 generated texts of each of two mixes the two readers differed on 1,709 and 2,238 before,
on 2 and 33 with the marks added and the second cut still there,
and on none after;
the tree reader's agreement with the parser's own address did not move.

Open:
the scan does not read the character-reference and bracket forms of the trail rule.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse":
a reader of a destination takes the parse's own trail rule,
and a second reader is measured against the first on generated texts.

## B327: the destination scan read neither the character reference nor the bracket of the autolink trail rule

Red in `75ea14bff`,
fixed in `3ce947dce`.

Found on 2026-10-06 (UTC),
the forms B259 left open on the trail rule.
`micromark-extension-gfm-autolink-literal` 2.1.0 reads a well-formed character reference (`&amp;`)
and a closing square bracket as no part of an address when an end follows them,
and ends an address at a bracket before whitespace,
`(`,
`[` or the text's end.
The scan kept `&amp` in `see https://c.example/a&amp;`,
ended every address at the first `]`,
and the tree reader,
which takes the parse's own cut,
named the shorter address.
By the agent's differential over generated texts,
the two readers disagreed on 5,048 of 26,452.

The fix:
`readTrail` (`corpus-run/dropped-destinations.ts`) is the rule as one reading of what follows a candidate end,
`addressEnd` asks it at marks,
`&` and `]`,
`]` is no longer a stopper of a run,
and the tree reader's autolink literal is cut by the same function (`literalAddress`).
`destination-address-texts.test-fixture.ts` generates the texts.
The agent's generator moved its prefix and its path length in step with the suffix,
each chosen by the position's remainder alone,
so a suffix only ever met one prefix,
against its own comment;
the lead's pairs every suffix with every prefix,
and the case over 6,000 of its texts passes with no disagreement.
The lead's change too:
two TSDoc examples keep the escape of the full-width comma
that the agent's tool had written as the character (M135).

Open:
the scan still ends an address at ASCII whitespace only,
where the parse ends at any Unicode whitespace,
and the generator draws none.
Closed since by B346.

Open to the owner's veto:
an address now runs past a `]` the rule does not end it at.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse":
a second reader of an address is held to the first by a generated differential,
and the generator draws every form of the rule.

## B346: the destination scan ended an address at ASCII whitespace only

Red in `de6ff17`,
fixed in `a7eca7f`.

Found on 2026-10-06 (UTC),
the item B327 left open.
The autolink literal's path,
its trail and a trail's closing bracket end at `markdownLineEndingOrSpace` or `unicodeWhitespace`
(`micromark-extension-gfm-autolink-literal` 2.1.0;
`micromark-util-character` 2.1.1 defines `unicodeWhitespace` as `\s` over one UTF-16 unit),
but the scan ended a run only at a space,
a tab and the two line endings.
An address followed by a no-break or an ideographic space ran on into the words after it,
so a page that kept the address and rewrote those words read as dropping a destination.
With five whitespace suffixes added to the generator,
the scan and the tree reader disagreed on 4,620 of 31,106 texts,
each holding a space past ASCII.

The fix:
a run ends at every whitespace `isTagWhitespace` reads
(the same 25 units,
the predicate the tag readers step over),
and so does a trail's closing bracket.
The generator draws a no-break space and an ideographic space after a path,
and whitespace past ASCII after a trail's period,
its bracket and a character reference.
The two readers agree on every text,
and a case holds 18 texts to the parse's own address,
a next-line mark and a zero-width space among them as controls that end no address.

Open to the owner's veto:
the scan imports `isTagWhitespace` from the tag readers' module,
where the alternative is a neutral module both import.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse".

## B705: a footnote reorder carried a note under the last definition to another definition

Status:
fixed on 2026-10-20 (UTC),
red `02a3d4e855`,
fix `19c50a6cd4`,
gated with the fixes of B704 and B706 as g125
(2565 `[PASS]` lines,
no `[FAIL]` line)
and landed in `eaa250287d`.
No cache version moved.

`reorderFootnoteDefinitions` (`archive/src/archive-footnote-order.ts`) bounds each definition by the parse's block,
with HTML comments masked to whitespace,
and already let the order stand,
with a note,
when a gap between two definitions held content.
It never read the text after the last definition:
that text was copied through as the suffix.
A note between translators on the line under the last definition,
or in its indented continuation paragraph,
ends no block,
so it stayed behind and followed whichever definition the move put last.
Four cases failed for that reason (`expected '2' to equal '1'`).

The order now stands,
with a note,
when the first content after the last definition is an HTML or JSX comment that starts on the same line or the next,
or is indented four columns or more (a tab counting as four),
as a continuation paragraph is
(`tailContinuesDefinition`,
one linear pass).
A comment on its own line past a blank line and indented less is a block of its own and stays where it stands:
the pinned `mikaela_khara` page ends with such a block of translators' remarks after its last footnote,
which a refusal of every comment would have frozen.
Guard tests hold a tab-indented note,
a page-closing note,
a two-column note across CRLF ends,
and a heading under the last definition,
and each was shown to fail under a mutation of the rule.

Over the 92 pinned archives,
12 pages hold two or more definitions and 2 of those carry notes;
neither the original's order nor the reverse of each page's own order changes any page's output.
The other passes that touch definitions (`applyFootnoteRelabel`,
`closeFootnoteRelabel`,
`relabelArchiveFootnotes` before its reorder,
the label readers) rewrite marker bytes or read labels and move no note.

Open,
with no pinned case:
the note carry (`archive-note-carry.ts`) puts a lost note after the slice's last block,
so in a slice holding two definitions a note lost from under the first would follow the second;
none of the 34 pinned notes sits inside a definition block.
A note on its own line before the first definition stays in the untouched prefix and can precede a different
definition after a reorder;
it follows no definition,
so it is left.

Recurrence:
none of an earlier entry.
A pass that moves spans reads every stretch it copies through,
the text before the first span and after the last included,
for content that belongs to a span it moves.
