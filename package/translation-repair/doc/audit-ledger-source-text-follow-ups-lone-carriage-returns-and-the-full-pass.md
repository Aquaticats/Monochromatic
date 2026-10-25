# Audit ledger: the source text under a note, the ninth round of follow-ups

The ninth round of the follow-ups of B832 (the original of an untranslated passage written as literal text):
lone carriage returns,
a wider sample of the generated sets and a full pass over them.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.
Rounds 1 to 8 are in
[the file of the follow-ups on parser gaps and step searches](audit-ledger-source-text-follow-ups-parser-gaps-and-step-searches.md),
and the entry they follow is in [the page grammar](audit-ledger-source-text-under-a-note-and-the-page-grammar.md).

## B832 follow-ups, the ninth round

This round follows the re-check of round 8.
It answers three findings of that re-check and four smaller ones.
The commits are in the order they were made;
each fix comes after a red commit,
and the failing lines of every red commit were kept as files apart from the commit message.

- `06630dabea8dc7c6f785f0c164438571926c65b9`:
  the pair and text readers of the sampled check move into a fixture.
- `19338352b89a6294a8096077fcd04c9679d38a5f`,
  red:
  a text with lone carriage returns for line endings (68 failing leaf cases, 3 controls).
- `c72dc715f9c51cff425551edf6e99b1350b808eb`,
  the fix.
- `6fa76a85c0593b561dd02b370dd35ff050679f7f`,
  red:
  the walk fixture called through an object property, the module namespace and `Function.prototype.call`
  (7 failing leaf cases).
- `6dafae15297c41fbf9450bc2520a6d1ca15664f4`,
  red:
  a definition holding a brace before an indented fence, a caret label with white space, and a tab fence beside lone
  carriage returns (9 failing leaf cases).
- `f99b48691280f178cb81120dfb6468786e216622`,
  the fix of those.
- `13e53207b72e7dfc6f9bef886dd79770aed9d4d7`,
  the fix of the walk fixture.
- `b59a090e344f00183f2f22af1686101f5fe66fc3`,
  red:
  a code span across a footnote definition line in a block quote or a list item (4 failing leaf cases),
  and the pin of the limit on footnote labels with a brace.
- `693c4b67ca341a55d26b6cffaf637e8a8b36cc21`,
  the fix of those, and the source of a regression found by the full pass.
- `d6cdc13b5114623d8a489b297db2c41128ffd43d`:
  articles in TSDoc and lines over 120 columns.
- `e4871f05840ce82edef1eb2d4c6f6e1e64883cb1`,
  red:
  the regression (2 failing leaf cases); its three cases asked for reader equality that no encoding gives for
  a footnote in a container, which `83a717b45dca19419e4ce70e104cf1468682807a` corrects.
- `83a717b45dca19419e4ce70e104cf1468682807a`,
  red:
  the same three texts, asked for no live construct (1 failing leaf case).
- `99adc6093c69dd5eff714ee25e3f208822340adc`,
  the fix.
- `bed83bb5c3c8cd0ff8e7329f4b80e15b72a93040`:
  the sampled check, the full task and the invariant for the last step.

### A lone carriage return

A carriage return that is no part of a Windows line ending is a line ending to the grammar and to the plain parse,
and the readers of the encoder split lines on the line feed alone.
The re-check found 18 text-level ids and 13 page-level ids that leaked a live expression,
and the shapes `$$CR{a}CR$$CR` and a fence that swallowed the page.
Two designs were open:
read a carriage return as a line ending in every reader that splits,
or write each one as a line feed where the text enters.
Ten modules split lines on the line feed
(`source-ships-nested.ts`,
`source-ships-open-flow.ts`,
`source-ships-readable.ts`,
`source-ships-lines.ts`,
`source-ships-fence.ts`,
`source-ships-fenced.ts`,
`source-ships-footnote.ts`,
`source-ships-addresses.ts`,
`source-ships-module-line.ts`
and `source-ships-lazy.ts`),
against one place where the text enters the encoder,
so the second was taken.
`foldLoneCarriageReturns` keeps the length,
so every offset the parse gives stands,
and it leaves a Windows line ending alone,
so a text of mixed endings keeps its Windows ones.
The commit message of `c72dc715f9c51cff425551edf6e99b1350b808eb` counts nine modules where the list has ten.

The company check of the composition (`source-ships-company.ts`) reads the text of each run for an indented fence mark,
a declared name and a flow left open, and it splits lines on the line feed too.
It reads the run folded in the same way,
and the page keeps the run as it was written.
The first version of the check left a run with a tab fence mark and lone carriage returns as written,
and the later run's brace stood outside the fence the mark opens;
the pair case in `source-ships-carriage-return.unit.test.ts` pins it.

The corpus reader folds CRLF and leaves a lone carriage return alone
(`slice-reading/src/line-endings.ts`),
so no page the corpus brings changes.
`doc/design-commitments-shipping-and-the-archive.md` states what the page does with one.

### The sampled check and the full pass

The sampled check guarded the class it was tuned on.
The pool now holds lone carriage returns,
footnote labels other than `1`,
tilde fences at the first column,
a footnote in a list item and in a block quote,
and a definition line after a footnote that is a tag alone.
The footnote product is 32256 texts.
The rate is one pair in four:
a class of nine pairs among 40000 is missed with chance 0.75 to the power 9,
so it is caught with chance 0.925,
where one in six catches it with chance 0.81.
The sampled case takes 40.2 s and 51.4 s in the two measurements made.
A task,
`test:live-full` of `translation-repair-page-check`,
runs `source-live-full.expensive.unit.test.ts`,
all 40000 pairs of each of the seeds 31415926 and 27182818 and all 32256 texts,
and takes 318 s under a cap of three threads
(one seed alone, 143 s and about 360 MB).

The invariant for the last step read the nesting of the pair texts with a function that splits lines on the line feed
and gave false alarms.
It now reads what the grammar refused of each page the composition tried,
and a page that ships without the original of a run is reported unless one refusal says the page nests too deeply.

The widened pool found,
in turn:

- A definition holding a brace before an indented fence,
  whose fence mark was joined to the definition line,
  which made the line no definition;
  the join is left out after a line that ends a definition.
- A link reference definition with a caret label that holds white space,
  which a footnote label cannot,
  read as a footnote definition in three places;
  `isFootnoteLabel` and `hasFootnoteLabel` now say it once.
- A code span across a footnote definition line in a block quote or a list item,
  one pair of the seed 31415926.
- The ten pairs of the two seeds that the fix of the last one made live,
  by cutting a paragraph of a container at a footnote of the container itself;
  the cut now takes only a definition that starts a line.
  The sampled case of that round did not keep any of the ten.

### The walk fixture

The fixture for a parse that returns a tree raising at the caller's first read told the grammar's own reads from the
caller's by looking for the function name in the call stack.
V8 writes a frame called with an object as its receiver as `Object.parseMdxBody`,
so a call through an object property,
the module namespace or `Function.prototype.call` with an object hid the name and the grammar's own read raised.
`parseMdxBody` and `parseMarkdownBody` now call a hook last
(`document-structure/src/parse-end-hook.ts`),
which stands on the global object under a registry symbol
and does nothing where no test sets it;
the fixture sets it for the span of one call and raises once it has fired for a tree.
The package of the fixture needs no import of the grammar's package,
and a source copy and a built copy of the module find the same hook.
The 14-site test of `translation-repair-artifact` passes with it.
Other parse sites (`parse-markdown-math.ts`,
`gfm-block-ranges.ts`,
`module-line-starts.ts`)
do not call the hook:
none of them reads the tree it builds before it returns it.

### Limits named

- A footnote label that holds a brace or a less-than sign (`[^{a}]`,
  `[^a}]`,
  `[^<b>]`):
  the reference is written with a backslash and the definition keeps its label,
  so the page holds no expression and the reference shows as text.
  `source-ships-encoding.unit.test.ts` pins three labels,
  and the module comment of `source-ships-encoding.ts` names the limit.
  Writing the same backslash into both labels would make them agree again.
- A footnote definition inside a block quote or a list item,
  a tag alone on the next line and a definition line with a brace:
  the page parses and holds no expression,
  and the definition line shows as text where the source reads a definition.
  This is the reading difference round 8 named for a tag alone and a definition in a container.

### Mutants

Run once each on a scratch tree of `bed83bb5c3c8cd0ff8e7329f4b80e15b72a93040`,
with the sampled case and the unmutated tree as the control (it passes):

- The JSX leak,
  a less-than sign before `Foo` left unescaped:
  the sampled case fails.
- The ES-module leak,
  module lines left as they are for a text with a quote mark and an import after a blank line:
  the sampled case fails.
- The revert of `f97ac62bcf6843d1a3907ae53428691b68538075`
  (`source-ships-nested.ts` as it was before it):
  the sampled case fails.
- The fold of lone carriage returns removed from the encoder,
  the caret label rule reverted,
  and the join after a definition restored:
  the sampled case fails for each.
- The cut of a paragraph of a container reverted,
  and the fold removed from the company check:
  the sampled case passes,
  and the full task fails for each.

### Checks

`build`,
`test:unit` (147 `[PASS]` lines and no `[FAIL]` for the page-check package,
59 for document-structure,
7 for test-fixture,
121 for artifact),
`lint:types` and `lint:oxlint` of the packages touched.
No cache version moved:
`git log --oneline --extended-regexp -G '_CACHE_VERSION = [0-9]+' bcff23b8ce..HEAD` names no commit,
and the same command over `c265e8df95~1..c265e8df95` names `c265e8df95`.
The fold is in the encoder of the page,
and no package that declares a cache version imports `translation-repair-page-check`;
the hook of the grammar returns the tree it was given and changes no value.
