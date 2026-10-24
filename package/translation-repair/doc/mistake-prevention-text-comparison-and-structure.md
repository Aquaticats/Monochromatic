# Preventing mistakes: comparing text and reading structure

Rules for comparing text and reading a document's structure:
code points,
word boundaries,
folds,
blank text,
escapes,
paired marks,
the parse,
lists of shapes where the parser decides
and quotes.
Pairing and offsets by position are in
[their own file](mistake-prevention-pairing-and-offsets-by-position.md).

Part of [Preventing this package's mistakes](mistake-prevention.md),
which lists every file of this record.

## Text by code point

What happened:
scans handed a character test one UTF-16 unit at a time,
and neither half of a character beyond the first plane passes a test for Han,
a letter or case:
a quote's neighbour after a script handle (class ninety-six),
a declared handle in script letters,
an Extension B ideograph in the tokenizer and the Han residue floor,
a Deseret letter beside a Canadian date,
and a pinyin pair over such an ideograph (ledger B22).
Two readings of a cased letter disagreed about script letters,
and eleven fixed-length text cuts could keep half an emoji.
The shared readers shipped tested only through their callers,
and their own tests later found a lone second half counted as nothing
and an opening that dropped a lone first half although no pair was cut.
A benchmark draw broke its ties with `localeCompare`,
so which slices it drew followed the machine's locale,
and a sort of document words compared UTF-16 units (ledger B95).
That fix left 40 argument-less sorts in production source,
and four comparators written `left < right ? -1 : 1`,
which order by UTF-16 unit and never answer zero for equal keys (ledger B122).

One floor was compared with two counts,
a model's reading in UTF-16 units with its spaces
and the deterministic reader's in non-whitespace code points,
so one text was usable as a model's reading and no text as the deterministic reader's (ledger B186).
A constant is compared with one count,
defined beside it.

A picture reading's length was logged in UTF-16 units beside a verdict that counted solid characters,
so fifteen astral letters logged as thirty (ledger B252).
A length printed beside a verdict is counted the way the verdict counted it.

The settled audit printed what a page cites as `characters=` from the text's UTF-16 length,
so a character beyond the first plane counted twice (ledger B389).
A count of characters uses code points,
through `codePointLength` (`code-points.ts`).

The rule:
text is ordered with `compareCodePoints` from `code-points.ts`,
and a list of plain text with `textsInCodePointOrder` from the same file,
never `localeCompare`,
`Intl.Collator`,
the default sort
or a relational operator in a comparator,
and cased with `toLowerCase` and `toUpperCase`,
never their `toLocale` forms.
Numbers order by subtraction.
A code-unit order kept on purpose,
such as canonical JSON's key order,
is named in the scan's exemptions with why.
A test whose domain reaches past the first plane
(Han beyond the unified block,
`\p{L}` or any general category,
case)
takes a whole character:
`for...of` over the text,
or `codePointAt` and `codePointBefore` from `code-points.ts` where the scan keeps UTF-16 offsets,
stepping by the character's length.
A unit read is kept only where every test on it is ASCII or first-plane only,
and a comment says so.
A cased,
capital or small letter is read by general category (`cased-letters.ts`),
never by comparing a character with its case mapping.
A text's opening cut at a fixed length goes through `wholeOpening`.
A guard for such a scan runs its text in a script beyond the first plane
and compares the result with the same text in Latin letters.
A shared reader gets a unit test of its own when it is extracted,
with lone surrogate halves among its cases.
The rule recurred twice in new work on 2026-10-12 (UTC):
the tense detector took a letter to be a character whose cases differ,
read one UTF-16 unit at a time,
so 14 of the corpus's declared aliases could never match (ledger B432);
and the first draft of the repeat-overlap pass weighed text in UTF-16 units (ledger B435).
Any scan that tests a letter or weighs text applies this rule,
not only one that names a script.

What enforces it:
`package/translation-repair/markup/src/code-points.unit.test.ts` and
`package/translation-repair/typography/src/cased-letters.unit.test.ts` pin the readers themselves,
lone halves included;
`package/translation-repair/source-scan/src/fixed-length-cuts.unit.test.ts` fails on any `.slice(0, LIMIT)` or
`.slice(-LIMIT)` it does not list with a reason;
`package/translation-repair/source-scan/src/locale-orderings.unit.test.ts` fails on any locale-dependent ordering or
casing in source or tests,
on an argument-less sort in source outside its named exemptions,
and on a relational operator inside a comparator handed straight to a sort;
the Latin-twin cases in `corpus-run/canadian-forms.unit.test.ts`,
and the script and Extension B cases in the declared-name,
tokenizer,
Han residue and pinyin guards,
fail when a unit read returns.
The settled audit's case on a cited character beyond the first plane fails on a UTF-16 count (ledger B389);
no source scan finds a UTF-16 length printed as a count of characters,
by the names of the scans the `source-scans` task runs,
so elsewhere that is habit.

## Words inside words

What happened:
fixed words and phrases were looked for with a raw substring,
so a glossary term matched inside a longer word
(class one hundred sixty-three,
ledger C1),
"as an ai" read as a refusal inside "as an aide",
"load" read as an inability inside "download",
and a refusal written with a typographic apostrophe matched no marker stored with a straight one (ledger B23).
An address that only began with a source link's was read as that destination kept,
and the neutral pronoun was counted only between listed marks,
so a dash or a slash after han hid it (ledger B23).
The archive review's apparatus labels were matched as raw prefixes,
so "Translation byproducts" read as the "translation by" label,
and the first fix bounded the label with a test of its own rather than `wordStarts` (ledger B81).
Two fixes in that pass nearly regressed:
reading a rendering's destinations through the strict grammar alone went silent
where the grammar refuses the rendering,
which is the validator's no-grammar branch that answers unknown;
and a plain word-boundary reading for the pronoun,
chosen before its tests were read,
would have counted the handles,
paths and addresses those tests exclude.

The rule:
a word or phrase looked for in prose goes through `carriesWord` or `wordStarts` (`word-bounds.ts`),
or through `glossary-match.ts` where the forms are folded,
and the text's quotes are folded with `normalizePunctuation` first
when the needles are stored with straight ones.
A name compared on its letters and digits goes through `carriesName` (`name-projection.ts`),
never containment in a projection,
which has lost the spaces between words;
a handle or a source form inside a link text is matched at handle edges,
where a hyphen and an underscore join.
A word that is never a piece of an address,
a path,
a handle or a compound goes through `tokenStarts` (`word-bounds.ts`).
A label looked for at the start of a line is a start `wordStarts` returns at offset 0,
never a `startsWith`,
and a fix that bounds a match reaches for these readers before writing its own boundary test.
A link destination is read as the grammar reads it,
the skeleton's `link-url` atoms,
never as a substring of the rendering,
and a rendering the strict grammar refuses is read under plain markdown,
never as carrying nothing.
Before narrowing any reading,
read the floor's own tests for the exclusions they pin,
and read the caller's branch for a text the reading cannot parse.
A raw `includes` on prose is kept only for containment,
a quote or a span found whole inside a text,
and a comment says so.
Turning a substring list into a word-bounded one reads,
for each needle,
the words that carry it past their start
(`/usr/share/dict/words` holds them),
and lists on its own each such word that keeps the needle's meaning.

What enforces it:
`word-bounds.unit.test.ts` pins both edges,
the open end,
digits and combining marks,
Han and punctuation edges;
the refusal,
picture-sense and reading-refusal guards hold a word carrying each marker,
a typographic apostrophe,
and each prefixed form listed on its own;
`name-projection.unit.test.ts`,
the survival guard and the link-name guard hold a key inside a longer word,
a case change and a digit inside a handle,
and a handle running on at either end;
`word-bounds.unit.test.ts` pins the token reading's joiners,
its trimmed ends and the mention mark,
and the neutral pronoun guard holds the dashes,
the slash after han and every exclusion;
the unwrapped-link guard holds a longer address
and an unwrap in a rendering whose original is refused too.

## Which fold for which question

What happened:
text written with curly quotation marks on one side
and straight ones,
or English quotes for corner brackets,
on the other was compared byte for byte (ledger B24).
A model's quote of the archive failed to anchor or to verify,
and lost its vote;
a copy of the incumbent with straightened quotes stood on the slate as a second candidate
and split the stake of one wording;
two voices giving one title with different apostrophes split their lexicon votes;
and a title reference apart from its heading only in apostrophe style was reported ambiguous
or rewritten into the other style.
Guillemets,
which English prose never uses,
shipped on one page although a judge named them,
and nothing read them.
A proposal that was the archive with its soft line breaks elsewhere,
which the site renders as spaces,
shipped as a change,
split a slate's stake with the incumbent,
or came back onto a consolidation slate as a lane text,
because each site asked whether it changed anything by bytes,
or by bytes and one rewrap of the base (ledger B26).
A reviewer's sentence-long quote of a screenshot transcript was dropped as unanchored,
because the transcript keeps the picture's line breaks
(the readers are told to preserve them)
and the quote,
written as the sentence,
had none (ledger B715).

The rule:
first say which question the comparison asks.
A model's quote against the document it quotes takes the evidence fold (`normalizePunctuation`),
which also maps the corner brackets,
because a model quoting a Chinese passage paraphrases them.
A quote of a picture's transcript or a cited page takes `quoteIsIn` (`reference/src/reference-attest-match.ts`) as well,
which sets every whitespace character aside on both sides:
a transcript wraps where the picture does and a quote does not.
A new anchor of a quote against evidence text reuses it and does not write a second whitespace rule.
Two renderings compared as one wording take the typography fold (`straightenQuotes`),
which maps curly and straight only,
because a rendering that kept 「」 is another rendering;
a whole slice,
which may carry code,
takes `straightenProseQuotes`,
which leaves a quote in a code span or a tag as written,
the restoration's own reach.
Whether a proposal changes the wording that stands takes `sameWording`,
which reads both through the wrap,
folds a paragraph's soft line breaks where the line-structure rule does not govern,
and keeps a hard break,
a blank line,
an opening indent and front matter apart;
every site that ships,
demotes,
collapses or offers a text asks it,
and bytes stay only where they are the contract
(assembly,
offsets,
seals,
an artifact's persisted relation).
A key is total:
before one reads text through a parser,
the parser's thrown errors are read and each gets a defined answer,
since a comparison that throws turns a question into a crash (M61).
A title,
a phrase or a document span compared with another document span needs no fold.
Offsets taken in folded text are used only in folded text:
every fold here maps one unit to one unit,
and the comparison states which text its offsets index.
A case fold read back as offsets in the original maps one unit to one unit too:
`lengthKeepingLowerCase` (`corpus-run/archive-name-runs.ts`) lowers a code point only where its lower case keeps the length,
since a whole-text `toLowerCase` grows at an İ,
and a scan that then reads the text past finds nothing on the whole page (ledger B121).
A check written as `.every` over what a scan found holds over nothing,
so it first asks what makes that list empty,
and a scan that gives up returns a refusal its caller must read,
never an empty list.
A mark the pages never use is refused by a floor,
not left to the judges:
a judge who names it does not keep it off the page.
Before folding a site,
measure what the fold would move over the stored records,
with a positive control that must move,
and where the pinned corpus cannot reach the site,
record it as unexposed rather than widen a reading that brings a hazard of its own
(a single-quote pair cuts at an apostrophe;
a wider URL stop cuts an address).

What enforces it:
`quote-normalize.unit.test.ts` pins both folds and the prose reading's code span,
tag and astral characters;
the attestation,
archive-review,
introduced-defect,
Latin title,
lexicon,
slate-collapse and title-reference guards each hold a straight quote against a curly one,
and the guillemet and sheet-leak guards hold the marks,
the exemptions and the editor sheet's marker;
`wording-key.unit.test.ts` pins what the wording key folds and what it keeps apart,
and the slate,
both lane wraps,
the consolidation wrap,
the polish round,
the lane offer,
the repair turn's copy check and the archive block review each hold a layout-only twin;
`archive-name-casing.unit.test.ts` holds a page with an İ that must still veto a name and restore a row.

## Checks on the bytes that ship

What happened:
the translate and consolidation lanes folded invisible variants only when building the slate,
after the publication rule,
the repair turn and the floor had read each answer as the model wrote it (ledger B112).
A line holding only a no-break space is no blank line to the page parser,
so an answer in two paragraphs passed against a one-paragraph page and shipped as two;
a disputed wording written with a non-breaking hyphen passed the disputed-wording check
and stood on the slate as that wording once folded.
The repair lane had folded at its wire readers all along,
and the fold module's header said every lane folded where an answer becomes a candidate,
which was true and was the wrong place.

The rule:
a change made to a model's answer before it ships is made where the lane first holds the answer,
before any check reads it,
so every check reads the bytes that ship.
A step that must change the text after the checks (a wrap)
is held by a test to layout alone,
or the checks run again after it.
A module that says where it is applied names each call site,
and a lane that starts reading a wire adds itself there.

What enforces it:
`translator-answer-fold.unit.test.ts` drives the translate slate,
the consolidation and the repair turn with an answer whose hidden paragraph break only the fold reveals,
and the translate slate with a disputed wording carrying a non-breaking hyphen;
`buildTranslateCandidates` throws `UnfoldedTranslationError` on a voice the fold would still change,
which `translate-candidates-collapse.unit.test.ts` holds.

## Text that shows nothing

What happened:
the checks deciding whether a model's wording or reason said anything asked `trim()`,
which keeps every invisible character that is not whitespace,
so a reply of one zero-width space passed the reply guard,
the intake fold emptied it,
and an empty candidate reached the judges;
a zero-width reason met "Reason before vote" the same way (ledger B40).
The docs of the two backstops said no such text could reach them.
The introduced-defect screen asked whether a claim's side was blank of trimmed text,
so a one-sided claim with a zero-width space or a Hangul filler on its other side
read as anchored both ways and was set aside as a wire fault (ledger B128).
A census of every blankness test in the production source then found twenty more sites:
quotes anchored on a character no reader sees,
fields counted as said,
an archive block revision of one invisible character shipped as wording,
transport fields read as present,
and page-assembly passes taking an invisible heading or signature for a rendering
(ledger B189 to B194).
The census first classed page text as parsed text and missed the page passes.

More readers took text that shows a reader nothing for a name,
a quote or a heading's words:
the signer floor,
the source quote floor and the archive's signature authority (ledger B260),
a prior quote of one invisible character that dismissed every claim carrying it (ledger B261),
and an entry note under a heading of marks alone,
which named an empty heading (ledger B271).

The rule:
whether text a model wrote shows a reader anything is asked of `rendersAsNothing` (`renders-as-nothing.ts`),
never of `trim()`,
`length`
or a comparison with the empty string,
and a check that also folds or tidies the text asks it of the bytes that ship.
`trim()` stays where the question is spacing:
a parsed value's edges,
a setting,
one character's class.
Text read off the assembled page is a lane's wording wherever a lane replaced the slice,
and is asked the same question.
A test of such a check carries an invisible character `trim()` keeps
(`U+200B`)
and one no fold removes
(`U+3164`),
spelled as escapes.

What enforces it:
`blank-text-tests.unit.test.ts`,
which `source-scans` runs (ledger B195),
fails on a production comparison with the empty string,
written as a string or an empty template;
on the length of a trimmed text,
or of a name its function declares from one,
compared with anything;
on the length of a name annotated as a string compared with zero or asked whether it is less than one;
and on any of those tested for truth,
outside a held list keyed `file#enclosing function`,
each key carrying its count of tests and why its question is another one,
so a new function,
a second test in a listed function
and a stale entry each fail.
`renders-as-nothing.unit.test.ts` pins the reading,
and each check's invisible-only case fails if it returns to `trim()`
(the screen's is "READS A SIDE THAT SHOWS A READER NOTHING AS NO ANCHOR").
Out of the scan's reach,
and left to each site's case and to review:
the length or truth of a string no annotation names,
a constant holding the empty string,
a blank needle handed to a search,
a length floor on text that was never trimmed,
an offset comparison standing for an empty span,
a trimming helper whose result another function compares,
and a stage that never asks.

## Escapes and the layer that reads them

What happened:
a tool that writes a file from a model's parameters decodes a backslash-u escape written there
into the character it names,
so an escape meant to stay visible in a file lands as the character itself (ledger M135).
The lead's own edit meant to write `\u002D` wrote a hyphen,
and a commit message draft held a literal U+FEFF;
an agent's front matter cases held nine literal byte order marks,
its TSDoc examples two full-width commas where escapes were meant,
and another agent's comments a hyphen for the JSON escape of one.
By the lead's inference,
a soft hyphen and byte order marks standing literally in older test files came the same way;
a scan then found 31 such characters in 9 files,
tests and two production sets among them (ledger B336).
In the other direction,
comments wrote a backtick with a backslash before it,
as a template literal needs,
where a Markdown reader of the comment shows the backslash
and a fenced example no longer opens or shows valid code:
five lines in one agent's test,
mended before its merge,
and twelve in five older files (ledger M137,
B337).
The lead's first draft of the scan for them kept a span of one backslash only,
and found a span of two,
the JSON escape of one backslash,
which renders as written (ledger M138).
With that rule written,
a checker's regex written through the Write tool held the escape for U+2028 (line separator),
and the tool wrote the raw character,
so the checker failed to parse;
the notes describing the slip then wrote the same escape the same way and got the character again (ledger M140).

The rule:
no tool parameter holds a backslash-u escape,
a message's text and notes included:
the code point is named in words.
An escape that must stay in a file is written through a shell edit,
with its backslash doubled in the edit's replacement
(`sed` with `\\u` where the file is to hold `\u`),
never through a tool parameter;
every file the lead or an agent wrote is searched for invisible characters,
and for an escape that came out as its character,
before it is committed.
A character a reader cannot see is written in source as its escape.
A comment is written for the layer that reads it:
a code span holding a backtick is delimited by a longer run of backticks,
a code span may hold nothing but backslashes,
and a fenced example holds the code as it is written,
with no escape a template literal would need.
A script that writes escapes into source checks the character at each place it edits
and refuses a place holding another,
and each site is read for whether it is a plain string,
an untagged template or something that reads the escape otherwise
(`String.raw`,
a comment).

What enforces it:
two source scans,
which `source-scans` runs (`2339cf5a6`,
every finding mended in `0b3996534`):
`literal-unseen-characters.unit.test.ts` fails on a code point of the Unicode property `Default_Ignorable_Code_Point`
or `White_Space`,
or of the general category `Cc`,
other than space,
tab,
line feed and carriage return,
written as itself in any TypeScript file under `src`;
`comment-escaped-backticks.unit.test.ts` fails on a comment the parser reports
that writes a backtick with a backslash before it,
unless the run of backslashes it ends has a backtick just before it.
Neither reads the docs,
a commit message or a file outside `src`;
those are searched by hand before a commit.

## Paired marks

What happened:
five readers of paired marks each paired an opening mark with the next closing mark,
so an opening mark that never closed took the next title into its span,
and three of them grew with the square of the unclosed marks (ledger B38).
The repeat reader read corner brackets and curly marks as quotation but not ASCII `"`,
which the pinned corpus writes in 98 body paragraphs (ledger B427).
It keeps its own depth rule,
since a quotation in Chinese prose reopens its mark at each paragraph,
and the marks it lists were then counted against the corpus.

The rule:
a reader of paired marks (title marks,
quotation marks,
brackets) reads spans through `closedMarkSpans` (`closed-mark-spans.ts`),
never through an `indexOf` loop of its own,
and its tests carry an opening mark that never closed.
The marks a quotation reader lists are counted against the pinned corpus,
and a mark the corpus writes that the reader does not list is a defect or a stated choice.

What enforces it:
habit and review;
each reader's stray-mark case fails if the reader stops using the shared rule.

## Structure read off the parse

What happened:
the quote guard counted blockquotes as blank-line-separated chunks opening with `>`,
while the floor it stands beside reads the parse,
so a quote opening on the line after a paragraph's was a blockquote to the floor and none to the guard,
and a replacement keeping every quote was refused
(hulicaijia24 slice 2,
ledger B42).
The census stretch for the refusal looked unreachable because the floor was read as the guard's reading.
The container block deficit counted runs of non-blank lines on each side,
so a list written tight in the original and loose in the archive,
or a fenced block with a blank line inside,
moved the deficit and admitted or refused a passage wrongly (ledger B68).
The italic-title pass read the archive's italics by splitting each line at its stars,
so a comment's or an expression's words read as a title
and a linked or underscored one was never read,
and it wrote a quoted title in italics inside an italic span,
where nothing set it apart (ledger B72).
The Anthropic reader took a body as whole when any payload held the word `message_stop` in quotes,
so a stream cut inside its last frame passed the retry ladder and failed after it,
and it never read the error event a failing stream ends with (ledger B87).
Six more readers split on blank lines after B68 named the rule:
the line-structure predicate called a loose list line-structured,
the target-only run protected no transcript written on the line after its anchor,
the size classifier and the address floor counted a fence with a blank line inside as two blocks,
the suicide floor read a quotation written on the line after a paragraph as the paragraph's own words,
and the page passes closed a stray backtick past a heading
and took a capital opening a paragraph after a heading for a name (ledger B100).
The translate floor and the archive-revision shape check compared only the top-level block sequence,
so a candidate keeping a container tag while turning the quote inside it into prose passed both,
and only the quote guard after judging caught it,
keeping the whole archive where a floor finding would have sent the candidate back to its translator (ledger B110).

The footnote run scan read a link's own URL and a tag's attribute as marker text (ledger B144),
the marker walk read an autolink literal's text,
which is its URL (ledger B145),
and a contributor form kept the spaces and link markup the page shows no reader (ledger B154).
A raw region read in place of nodes is checked against those nodes' own values
before any span of it is acted on.

The footnote graph read every text node's position and threw on the nodes
the autolink-literal transform rebuilds without one,
so the document parse ended on such a page (ledger B159).
A walk that reads positions off phrasing nodes names which node types it reads
and refuses the rest loudly.

The footnote graph read a marker shape in the URL of an angle autolink (ledger B181),
and the mention scan counted one in any URL (ledger B182).
The lexical reader B182 added to skip URLs was then probed against the graph,
and disagreed with the parse in both directions:
where a bare literal may begin,
what a link label holds,
whether whitespace may stand before a destination.
A lexical stand-in for a parse is checked against the parse over a table of shapes before it is trusted,
and where the two can still differ it errs toward the side its guard can afford.
The stand-in was then replaced:
the mention scan asks a parse of its fragment where a marker shape is no reference,
under both grammars the page may be read by (ledger B212),
since correcting the lexical reader rule by rule was writing the parser again by hand.
A fragment is parsed without the steps only a whole document can take,
the front matter split among them,
and a region is skipped only where every grammar the page may be read under skips it.
The destination scan ended an address at its first closing parenthesis,
where the parse keeps one that balances an opening one (ledger B213).
A cut that stands for the parser's is written from the parser's own rule for it.

A contributor form shaped like a reference link read as its markup whether or not the text defined the reference,
and the first fix found the label's end from the length the parser reports for the reference's own label,
which is decoded and not what is written (ledger B201).
Whether a reference shows as a link is asked of the parse,
and an offset inside a node is read off its children's positions.
The slice census sliced a settled row by its recorded pairing
and counted its added blocks by the deterministic aligner (ledger B197).
A figure describing a carve is read off the steps that made the carve.

The lone container tag masker ended an element name at a space alone
and refused a closer carrying anything past its name,
where the strict grammar steps over whitespace before a tag's closing bracket,
so a lone closer written `</details >` stayed unmasked,
and a closer written with a tab left its whole element reading as two lone tags (ledger B222).
A reader of a tag's shape is shown the strict parser's verdict for each whitespace spelling,
the spellings it does not take among them.

The mention scan called a marker a definition after any indentation,
which the footnote graph reads under one grammar only (ledger B258);
the destination scan kept the emphasis marks the parse's trail rule sheds (ledger B259);
and the two container tag readers stepped over fewer whitespace characters than the strict grammar,
each a different set (ledger B262).
A role a scan gives a marker is asked of the parse the page is read under,
and two readers of one construct share one predicate that a test checks against the grammar.

The nesting measure skipped fenced stretches the parser reads no fence for,
and took a line mixing stars and hyphens for a thematic break (ledger B321,
B322);
an editor's edit and the join of two slices that each read alone
nested past the bound where no gate read them,
and each ended a chunk or a document (ledger B320,
B323).
A page opening with a byte order mark read its front matter as body (ledger B324),
the slice parse did not blank a line showing nothing as the page parse does (ledger B325),
three readers of a tag name each kept their own name (ledger B326),
and the destination scan read neither the character reference nor the bracket of the trail rule (ledger B327).
A measure may refuse a text its parser reads,
never skip a stretch the parser nests;
a voice's text is refused as that voice's by its stage's own gate;
one reader answers where a document starts and what a name is,
held by a differential against the compiler over every class of character;
two parses of one text apply the same masks in the same order;
and a generator for a differential draws every form of the rule it tests.

The readers ledger B326 and B327 left open were then brought to the definitions:
the nesting count had missed 8 tags the grammar opens and opened 146 it refuses (ledger B344),
the attribute restorer had dropped 61 of the 75 tags whose attribute the grammar reads (ledger B345),
and the destination scan had run an address past a no-break or an ideographic space (ledger B346).
A reader named open beside a definition it should share is brought to that definition,
with the differential extended to its form;
a scan that sees one line at a time reads a construct the grammar lets run on to the next
as the grammar reads it,
measured against the grammar on the split forms.

The rule:
a question about what a passage's blocks are
(how many quotes,
which kind a block is,
what a container holds)
is asked of the parse the floor reads
(`readPageSkeleton` or `readSliceSkeleton`),
never of a blank-line split or a first character.
A reader in a page pass reads the page grammar (`proseBodyTree`);
a reader beside the floor reads the slice grammar;
each says what it answers for a passage its grammar refuses.
Plain markdown refuses too,
where nesting exhausts its stack,
so a reader moved off a scan that read any text keeps reading any text:
its tests carry a passage neither grammar reads
(the page passes' code-span reader threw out of the Han residue floor on such a page,
ledger B100).
A split on the text stays where the question is the text itself,
and a held-out run is cut at the parse's own offsets,
so putting it back rebuilds the text byte for byte.
A guard's tests carry a case where the split and the parse disagree:
a block opening on the line after a paragraph's,
and one inside a container tag.
A reader that scans a paragraph's raw text first blanks what the parse hides there,
comments and the tags of a container the parse widened the paragraph over,
and the text inside inline code and bare URLs,
before it tests the paragraph for markup;
the repeat reader dropped whole paragraphs for a brace inside a comment (ledger B426).
Before calling a guard behind the floor unreachable,
check that both read the structure one way.
A floor comparing top-level blocks answers nothing about what they contain,
so a property a page has to keep inside its blocks,
its quoted passages among them,
is compared at every depth beside the top-level comparison,
not inside it.
Where a count is compared across two sides and then spent,
one reading serves all three:
the deficit's passages cost the blocks the container counted for their slices.
The same holds inline:
which words a text sets in italics,
in bold or in a link is read off the parse
(`emphasisSpans` for italics),
with HTML comments first blanked to spaces of the same length,
as `parse-slice-body.ts` does,
so that no offset moves and no comment's words are read.
The same holds on a provider's wire:
which frame a payload is
(the terminator,
an error event)
is read off the parsed frame's `type`,
never found as a word in the payload's text,
and a reader is checked against every frame kind its protocol's current documentation lists.
A text is parsed in its own grammar:
a front-matter slice is YAML,
where a line opening with `>` is a scalar's content,
so a Markdown reading of structure skips it,
as the declared-name guard and the floor already did (ledger B113).
The contributor reader found a link's label end with `indexOf('](')`
and took the whole token for a name when the label was empty (ledger B126).
A reader that parses a slice alone says what a slice alone cannot see:
a footnote or reference definition held in another slice,
and the text around an edit region.
Parsed alone,
`[^1]` and `[b's]` are plain text,
so the refusal run read a footnote reference as characters between two marks
and shipped a doubled mark,
and the prose mask curled a label at its reference and not at its definition;
a fragment of an edit region that cuts an inline element is refused by the grammar alone,
or is parsed from the middle of a tag.
Every reader that parses a slice takes the page's definitions with it
(`refine-envelope.ts` collects them),
an edit-region fragment gets its reading from the slice it lands in with the fragment in place,
and a reader that cannot do either names that blind spot in its header
(ledger B609,
B610,
B615;
built by n159 and landed with the fixes of n161).
The definitions the shared reader collects (`page-definitions.ts`)
still miss those inside a container and stop at a single newline,
which is an open item (ledger B620),
and a call site with no page at hand gets none.
A fragment landing that lies outside its slice is refused by throwing `FragmentLandingError`,
never read as no prose,
and a memory of readings is keyed so that no text can read another's (ledger B621,
B622).

What enforces it:
`blank-line-splits.unit.test.ts`,
among the source scans,
fails on a `split`,
`indexOf`,
`lastIndexOf` or `includes` handed a blank line in the package's source
(as a string,
a constant holding one,
or a pattern matching two line feeds);
a reader walking lines and treating an empty one as a boundary is out of its reach,
and is left to review.
`quote-preservation.unit.test.ts` carries both disagreeing shapes,
and `quoteBlockCount` no longer exists to be reached for;
`translate-validate.unit.test.ts` and `archive-revision-shape.unit.test.ts` carry a container tag whose nested quote is dropped and one where it is kept,
and `translate-slice-guards.unit.test.ts` a front-matter block scalar line opening with `>`;
`insertion-container-deficit.unit.test.ts` carries loose against tight lists and a fenced block with a blank line;
`emphasis-spans.unit.test.ts` and `archive-italic-title-restore.unit.test.ts` carry the inline shapes a split misreads:
a comment,
a JSX comment beside one,
a link,
underscores
and a span across lines.
Each B100 reader's tests carry the shape its split misread and the answer for a passage its grammar refuses.
`anthropic-completion.unit.test.ts` carries the word as a tool's name and the error event,
and `hyper-client.unit.test.ts` a body cut inside its terminator frame.
`nesting-fence-differential.unit.test.ts` holds the nesting scan to the grammar's own parser over a generated family;
`mdx-tag-name.unit.test.ts` and the masker's,
the inline reader's and the restorer's tests hold every reader of a tag name to the strict grammar
over `tag-name-spellings.test-fixture.ts`,
and `nesting-inline-count.unit.test.ts` and `corpus-run/tag-attributes.unit.test.ts` hold the nesting count
and the reader of attribute names to it over the same spellings (ledger B344,
B345);
and `corpus-run/dropped-destinations.unit.test.ts` holds the destination scan to the tree reader
over `destination-address-texts.test-fixture.ts`,
which draws whitespace past ASCII after a path and after a trail (ledger B346).

## Lists of shapes where the parser decides

What happened:
the source encoder had to know which lines the strict grammar starts a module at,
and a list of the shapes that change the state the tokenizer is in when it reaches a line grew by one shape in each round:
a list marker alone after a footnote definition,
a line of invisible characters the package masks,
a line after a setext underline that follows a restarted block (ledger B832 follow-ups).
Each round's search found the next shape the list did not hold,
and each list was right for the shapes it named and silent about the rest.
The fix read the lines off the strict grammar's own tokenizer with a construct that records and refuses,
on the text with every other rewrite made (`moduleLineStartsOf`),
and the same fix read where a table and a footnote definition stand from a parse with GFM on
(`tableRangesOf`,
`footnoteDefinitionRangesOf`),
where the plain parse's ranges had been cut by guesses at the same grammar.

The rule:
a guard or a reader that lists the shapes of a construct the parser decides is a lexical stand-in for the parser,
and it is replaced by a question put to the parser:
a tokenizer construct that records where it is entered,
a node range from a parse with the extensions the page is read under,
a position the parser reports.
Where the parser gives no hook,
the list stays with three things beside it:
a differential against the parser over a generated family that draws every form of the rule,
a positive control that removes one shape from the list and shows the differential fail,
and a comment that names the shapes it leaves.
A comment writes "every" only where two seeded searches over the finished code prove it,
and otherwise states the counts and the head each run was at.

What enforces it:
habit and review;
`source-ships-module-line.unit.test.ts` and `document-structure/src/module-line-starts.unit.test.ts` hold the module lines to the tokenizer,
and `document-structure/src/gfm-block-ranges.unit.test.ts` holds the table and footnote ranges to the parse.
No scan finds a list of shapes standing in for a parser.


## Quotes a judge gives back

What happened:
the consistency pass counted the judges' quotes of a place as agreed only when they were the same string,
and the first live run split them on a final period,
a line break against a space and a straight against a curly apostrophe,
so a 3 to 1 choice located nothing (ledger B449);
and a quote that did locate would have carried a word that renders source outside the repeat (ledger B450).
Both families were recorded before the module was written.
Then the locator widened a located span over a single mark it judged,
from the shape of nearby words,
to be an apostrophe,
and 194 of 1,020 generated texts gave a wrong span that deleted or stranded a quotation mark (ledger B463).
The proof that replaced the guess read only the paragraph of the mark,
and a closing quotation mark whose opener stood in another paragraph was taken for an apostrophe and deleted (ledger B492).
Then one mark was read by five readers that disagreed:
the counter,
the locator,
the proof,
the reader of the page's form and the reader of marks inside words.
An apostrophe after closing markup was an opener to the proof and a joint to the locator (ledger B504),
a lone `*` counted as closing emphasis by shape (ledger B506),
the letter beside a mark was a Latin letter in one reader and a letter of any script in another (ledger B507),
and a mark one markup character past the words was recorded by none of them (ledger B508).
Widening the shared letter test for the pass reached the glossary,
quote locating,
alignment and the other callers of the package (ledger B523),
and an oracle written from the rule text then found the pass reading a mark as an apostrophe
beside another script,
a symbol or a pictograph,
past a `)` or a `"`,
and past a lone `_` that the parse does not make emphasis (ledger B536 to B538).
The run that a gain looks across for a kept mark,
so that the page never shows two marks where the source had one,
was first stated as the characters the parse reads as markup,
then as three references,
three spaces and "on the line".
The tenth review found the page carrying a second mark past a footnote reference,
a tag,
a link and a space,
and the re-review found it past a reference that is itself a mark,
a backslash escape,
the other Unicode spaces and a soft line break,
and the fix of those found a fifth family,
the container markers of a block quotation or a list (ledger B557,
B564 to B567,
B574,
M208).
A leading straight mark before a word of a closed list of elisions was curled to `’` whether or not it was paired,
and a quotation that begins with such a word shipped `’Tis the season,'`
with its closing mark straight (ledger B563,
B568,
M207).
Then the word test at a span edge and the pairing context of typography read the written text
and not the text that the reader sees:
an edge was placed inside a word that a reference,
a plain ampersand or a zero-width space joined (ledger B596),
a quote ended or started beside emphasis,
link or tag delimiters inside one shown word and shipped half a word (ledger B597),
and a quotation opened by a tag end or by emphasis markup kept a straight mark beside a curly one
(ledger B602,
B605).
Typography curled marks inside link destinations,
titles,
addresses,
expressions,
code and math until its mask was built from the parse (ledger B606).
Then three judges quoted three nested spans of the chosen place,
no one text was a strict majority of the quotes,
and the group was lost as unlocated (ledger B642);
the rule that located it hung on a blank quote and read no combining mark (ledger B644,
B645).

The rule:
a span covers only what the quote matched,
and anything decided by a guess stays outside it.
A single mark at the edge of a span is part of the span only where the whole page's prose proves it an apostrophe
(no possible partner for it anywhere in that prose,
every single quotation character counting as a partner);
every other edge mark is recorded beside the span and compared at both places.
An apostrophe that arrives with the words is written in the form of the receiving place (ledger B494).
One mark has one reading in every reader of the pass:
the letter beside it is a Latin letter (accents included),
an ASCII digit or a letter or digit whose compatibility form is one,
and the pass keeps its own test for that and does not widen the shared `continuesLatinWord` (ledger B523),
`*`,
`_` and `~` close only where the parse makes them the closing delimiter of a node (ledger B536),
and a splice is built and then checked by reading every single mark of the block outside the span before and after,
a change making the place unavailable (ledger B505 to B507).
A mark beside another script,
a symbol,
a pictograph or another single mark reads free,
and the run it is read past holds the closing delimiters of the parse and a link's closing tail only (ledger B537 to B539).
A leading mark of any apostrophe-like form is a possible opener for the marks after it (ledger B540,
B559).
The run that a gain looks across for a kept mark is another run than the one a mark is read past.
A rule about what lies between two marks is stated by what renders between them,
in the model of the renderer,
and never by a list of characters:
escapes (a backslash and the character it escapes show as that character),
character references (one that decodes to a quotation mark makes the page unavailable for every gain),
every white space that the proof's own test holds,
line breaks inside a paragraph (a single one shows as a space),
and container markers at the start of a line (they show as nothing)
(ledger B557,
B564 to B567,
B574,
M208).
Before an option is accepted that changes the glyphs a page ships,
a paired and an unpaired quotation are probed with it,
a quotation that begins with a word the option treats specially among them (ledger B563,
B568,
M207).
The word test at a span edge and the pairing context of typography read shown text,
and what the parse does not show is transparent:
references,
escapes,
default-ignorable characters,
emphasis,
strike and code delimiters,
link brackets with their destination and title,
tags,
comments and expressions (ledger B596,
B597,
B605).
Not everything the parse hides is transparent:
a hidden stretch hides every unit in it,
whatever its width (ledger B616),
markup that separates or annotates ends a word
(line breaks with or without a slash,
block tags,
table cells,
images and the `rt` and `rp` of a ruby annotation),
while inline wrappers stay transparent (ledger B617),
and inline code is a shown word (ledger B618).
Typography touches only the text that the page parse shows as prose,
through the one mask that every typography reader shares (ledger B606).
A quote a model gives back is evidence of where it points,
not text to compare or splice:
it is located in the text it quotes,
with whitespace runs,
quotation-mark style and edge punctuation read as equal,
and ballots agree when they locate the same span;
what a later step writes comes from the text itself,
and a change a span cannot express goes through writers,
the bench and the floors,
never through a splice.
Quotes of one place that nest are evidence for every quote that contains them:
the place is located by the longest quote that stands once in its English
and that a strict majority of the usable ballots' quotes contain at the edges of words,
and the span so located can be shorter than the words a judge meant (ledger B642).
A search that can be given an empty text is guarded before its loop,
since an empty text is found at every offset and is contained only in another empty text (ledger B644),
and a word edge reads a combining mark as part of the word,
as the locator does (ledger B645).
The slot a judge gives beside a quote is evidence for every quote that contains the located words,
as the quotes themselves are (ledger B658).
Two readers of one apostrophe read one twin of the English a reader sees,
and a link counts as a closing bracket only when its label ends in a closing mark (ledger B659,
B660).

What enforces it:
the consistency pass's own tests,
built from the live run's texts;
`pass-consistency-shipped-marks.unit.test.ts` holds a page for each family of second mark that the re-review found
(references,
escapes,
spaces and line breaks),
and `restore-typography.unit.test.ts` holds paired and unpaired elisions;
no scan finds a quote compared as a string elsewhere,
so a new reader of judges' quotes is reviewed against this rule.
