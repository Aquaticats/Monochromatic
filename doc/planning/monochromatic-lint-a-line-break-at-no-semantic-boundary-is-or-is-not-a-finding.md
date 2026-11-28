# A line break at no semantic boundary: a finding, or not

Status:
written 2026-11-27 by the round that fixed issue 6642
(`fix/monochromatic-lint-issue-6642-the-line-break-fix-joins-what-the-old-wrap-cut-and-leaves-no-lone-word`),
revised the same day by the round that reran its survey,
revised on 2026-11-28 by the round that reran the survey for the second gate read,
and revised on 2026-11-28 by the round that answered the third gate read.
**Unapproved and non-final:**
nothing here is built,
the rule reports exactly what it reported before,
and the owner declares completion.

It has had three same-vendor gate reads,
none of them an independent review:
the first read the version at `cdf69930fd` and found 16 findings
(5 High,
8 Medium,
3 Low),
the second read the version at `5bb8813b28` and found 22 findings
(2 High,
13 Medium,
7 Low),
and the third read the version at `8638a53928` and found 21 findings
(3 High,
8 Medium,
10 Low).
The third read's scratch expires a day after it was written,
so "Answers to the third gate read" restates each of its findings
before it says what answered it.

Every count below is one of three things,
and the text says which at each use:
a row of the survey's `output.txt`,
a figure the survey does not count whose command and output are kept beside it,
or another record's own measurement,
named as that record's.

The owner's decision this proposal asks for is a yes or a no
to a new finding class,
and a yes adds findings to text that passes today.

## Why this question is open

`markdown/semantic-line-breaks` answers one question:
does a break belong after one of `,` `.` `;` `:` `?` `!`
when prose follows that character on the same line
(`package/linter/monochromatic-lint/src/markdown_break_points.rs`,
`break_offsets` and `is_break_point`)?
It never looks at a break an author already wrote.
So a paragraph hard-wrapped at a spot with no break-point character in front of the break
passes the rule.

`AGENTS.md` rule MD1 says,
and this is the whole rule:

```text
Break lines at semantic boundaries,
under 120 chars,
so text reads without editor wrapping.
```

It names no set of characters.
It names "semantic boundaries" and a bound,
and it never says which boundaries count.

`AGENTS.md` rule MD8,
in the same section,
says,
and this is the whole rule:

```text
Hard wraps fall only between whole inline spans;
keep code spans,
emphasis,
and link syntax on one line,
or the closing delimiter renders literally.
```

MD8 names inline spans,
code spans,
emphasis and link syntax.
It names neither a parenthesis nor a dash.
It is also a condition on where a hard wrap may fall,
not a grant that a wrap before a span is a semantic boundary:
"only between" forbids a wrap elsewhere,
and forbids nothing it does not name.

`AGENTS.md` rule WR2,
in the section on prose style,
says,
and this is the whole rule:

```text
Prose never uses em-dashes,
en-dashes,
or ASCII substitutes for them:
use paired commas/parentheses,
colon,
semicolon,
or period;
"to" for ranges.
Hyphenated compounds and CLI `--flags` are fine.
```

So a break after a dash is a break after a character
the repository forbids in prose at all.

The six characters the rule breaks after are the rule's own set,
not a definition MD1 gives.
The survey below classifies every break by the first token of the next line's prose,
so the owner can say which kinds MD1 permits.
This proposal does not answer that question,
and the kinds it offers are the author's proposal,
not a list MD1 or MD8 holds.

No rule of the 30 this linter ships bounds a prose line's width.
A 123-character prose line with no break point inside it passes the repository's
own configuration with exit 0,
and only `markdown/semantic-line-breaks`,
which reads where to insert a break,
reads a prose line at all.
The rules named `max-lines` in the Rust and shell sets bound a file's line count.
So MD1's 120 is a rule nobody enforces,
and this proposal does not propose to enforce it.

## What "at no boundary" means in this measurement

One break is counted when all of these hold:

- The break sits inside a paragraph,
  so a blank line,
  a fence,
  a heading,
  a table and a list marker end the walk.
- The line the break ends holds at least one visible character.
- The last visible character of that line,
  after any closing bracket or quote
  (the nine characters `" ' ) ] } ” ’ » ›`),
  is not one of `,` `.` `;` `:` `?` `!`.

The harness is kept in
`package/linter/monochromatic-lint/doc/a-survey-of-line-breaks-at-no-semantic-boundary/`.
It reads Markdown and Rust doc comments through the linter's own
`MarkdownSource` and `processors::extract`,
so a paragraph and a doc comment are the ones the rule sees.
It resolves each path's configuration through `config_lookup` and `config_match`,
so a path the configuration ignores is not in the count.

Four kinds of line are counted apart because they are not one defect:

- A line ending in two or more spaces or a backslash is a Markdown hard break.
  The survey counts it and every predicate below leaves it alone.
  The run found none,
  which is why one condition of the clause predicate tests nothing (see "Answers to the third gate read",
  `L10`).
- A line whose last visible character is a bracket or a quote
  is counted apart as a **closer-ended** break,
  because a reader may read such a line as ending at a boundary of its own.
  The survey still counts it at no boundary,
  and every predicate below reports it:
  question 2 says what a permit of that kind would take and why its exact count is not printed.
- A line that ends a sentence inside emphasis or strong emphasis,
  such as `word.**`,
  is an **emphasis ending**.
  `markdown_semantic_breaks.rs` climbs `Strong`,
  `Emphasis` and `Delete`
  through `delimiter_tail`,
  so the rule wants the break after the delimiter,
  while `ends_at_break_point` strips only bracket and quote closers.
  A run of the installed fix on
  `- **Naming every status.** The status filter is the next thing.`
  writes the break after `.**` and settles there.
  A predicate that reported this ending would ask for a join the rule breaks again.
  The merge named in question 7 makes the boundary test climb `*` `_` and `~` too,
  so a rerun at this branch's tip would count such a line as a boundary
  and this class would be empty there.
- A line ending `e.g.`,
  `etc.` or `...` is treated as a boundary here,
  although `break_offsets` never breaks after an abbreviation or an ellipsis.
  The survey undercounts breaks at no boundary by that small class.

A continuation line of a quotation keeps the `>` its container writes.
The survey strips the marker before it classes the line,
counts its words or measures its clause width,
so a quoted line is filed by its prose and not by the marker a reader does not see.
The run classes 5,602 breaks whose next line opens a quote marker this way,
and 65 breaks remain in the `block-marker` class,
where a following line still opens what the shipped join refuses,
such as a line that begins `#1696` inside a list.

## The measurement, and what it counts

The survey ran at commit `bc5b4ca2bae7fc5bb80475448c31d4d57f51f83c`
(2026-11-28T03:19:11-05:00)
over the `10,552` tracked `.md`,
`.mdx` and `.rs` paths of that commit in
`package/linter/monochromatic-lint/doc/a-survey-of-line-breaks-at-no-semantic-boundary/files.txt`.
The harness reads each path's bytes with `git show <commit>:<path>`,
so the counts are that commit's and not a later worktree's.
The configuration ignores `66` of them,
and turns the rule off for `1`;
`10,352` host files yielded a parsed source,
and none failed to parse.
The corpus includes the earlier version of this proposal,
because the list was taken before this revision;
the records added after it are not counted.
The whole output is
`package/linter/monochromatic-lint/doc/a-survey-of-line-breaks-at-no-semantic-boundary/output.txt`,
and the command and temporary changes are in `command.txt` beside it.

The command,
with the three temporary changes
that the folder's `README.md` names and that the round reverted:

```sh
# from package/linter/monochromatic-lint
SURVEY_LIST="$PWD/doc/a-survey-of-line-breaks-at-no-semantic-boundary/files.txt" \
  SURVEY_ROOT="<the worktree the list came from>" \
  SURVEY_M13_DIR="<a folder outside the repository>" \
  SURVEY_LINT_BIN=/home/user/Monochromatic/.cache/toolchain-bin/monochromatic-lint \
  cargo nextest run --locked --lib --no-capture survey_
```

The run took `1,297.955` seconds for its five tests.

The counts add up:

```text
paths in the kept list:                                         10,552
paths the configuration ignores:                                    66
paths the configuration turns the rule off for:                      1
paths surveyed:                                                 10,352
paths that failed to parse:                                          0

breaks inside paragraphs:                                    1,545,884
  whose line ends at a break point:                          1,324,125
  at no boundary:                                              221,759
    in paragraphs the rule reports nothing for:                210,207
      ending in a closing bracket or quote:                      9,603
      ending otherwise:                                        200,604
      ending in a hard break:                                        0
    in paragraphs the rule already reports:                     11,552
      ending in a closing bracket or quote:                        254
      ending otherwise:                                         11,298
      ending in a hard break:                                        0
```

`210,207` is the population every predicate below starts from.

The `133` paths the rows above do not name are Rust files holding no `///` or `//!` line:
the rustdoc processor makes no document for them,
so the harness never counted them as surveyed.
`git grep --files-without-match --extended-regexp '^[[:space:]]*(///|//!)' bc5b4ca2ba -- '*.rs'`
lists `134` such files,
all `134` of them are in `files.txt`,
and one of the `134` (`package/linter/monochromatic-lint.fuzz/seed/rust_style/sidecars.rs`)
is one of the `66` the configuration ignores:
`10,552 - 66 - 1 - 10,352 = 133`.

The `ADDITIVE` line of `output.txt` is a section label,
printed just before the assertions that add each cell of the block above;
the numbers after it are the cells whose sums the harness checks.

The line `M13 batch exit=Some(1) finding lines=0` says that one batch of the after-the-fix
step exited `1` and that the harness counted `0` lines on its stderr,
which is where the harness looked for findings:
the findings themselves go to stdout,
so the line means "nothing on stderr",
not "nothing reported".

The population includes `9,603` closer-ended breaks and `3,383` emphasis endings.
It excludes the `11,552` in paragraphs the existing rule already reports.
That exclusion is a subtraction of two measurements,
not a lower bound:

- After the shipped fix ran over copies of the `222` files that hold those paragraphs,
  `10,028` of the `11,552` breaks are still counted at no boundary
  and `1,524` are gone.
- The `10,028` is an upper bound on what a full fix run would leave,
  and the `1,524` is a floor on what it would join:
  at the survey commit `ends_at_break_point` does not climb `*`,
  `_` or `~`,
  so a break the fix inserts after `.**` is counted at no boundary by this survey
  even though the rule put it there on purpose.
  The proposal's own `emphasis-boundary-reproduced.txt` is that shape.
  A rough scale for how much of the `10,028` could be such breaks:
  `269` lines in `.md` and `.mdx` and `2` in `.rs` hold a break-point character,
  an emphasis closer and prose on one line.
- `221` of the `222` files carry no finding after the fix,
  and one still holds `64`,
  whose breaks may sit in paragraphs the rule still reports.

A run of the fixed tool over the whole tree would therefore leave
`210,207` breaks plus at most `10,028` more for every predicate to start from,
which is `220,235` as an upper bound.
The measurement and the findings left on the copies are in `output.txt`
and in the file `findings-left-on-the-fixed-copies.jsonl` beside it.

Each break is classified by the first token of the next line's prose,
in this order:
`block-marker`,
`parenthesis`,
`inline-span`,
`conjunction` (a fixed list of conjunctions and subordinators),
`dash` (the line before ends in `--`,
`—` or `–`),
and `ordinary` for the rest.
The counts,
all in the `210,207`,
with the breaks whose clause reaches the limit and the breaks a removal can join:

```text
block-marker      65       forced      6       removable     38
conjunction   19,749       forced  8,849       removable 12,392
dash              20       forced      5       removable     17
inline-span   25,355       forced 10,091       removable 17,512
ordinary     127,985       forced 26,558       removable 106,692
parenthesis   37,033       forced 13,450       removable 26,056
```

The `ordinary` class is `60.9` percent of the population.

Each break also carries the width of its **clause**:
the run of lines from the previous line ending at a break point or a hard break
to the next one,
joined with single spaces,
after each line's own indentation,
plus the paragraph's continuation indent.
`58,959` of the `210,207` sit in a clause at or over `120` characters,
in `47,811` clauses.
No single line may hold such a clause under MD1's bound,
and it holds no punctuation to break at,
so its break is **forced**:
no rewrap of that clause brings it under `120` on one line.
`26,558` of the forced breaks are ordinary-class,
`13,450` parenthesis-class,
`10,091` inline-span-class,
`8,849` conjunction-class,
`6` block-marker-class and `5` dash-class.
The conflict between MD1's two halves therefore exists
under the reading that permits none of those kinds;
an owner who permits a kind removes that kind's forced breaks with it.

What "forced" says and does not say:

- It is a fact about a clause,
  not about a finding:
  a forced clause holds `58,959 / 47,811 = 1.233` breaks on average,
  and it needs at least one,
  so a rewrap must keep one break of it and may remove the others.
- For the predicate that reports every break (`A`) the surviving break is a finding no rewrap clears,
  unless the author adds punctuation or words.
- For the predicates that report a shape (a break before a short line,
  or a short line of its own)
  a rewrap does clear the finding even though the clause stays forced:
  the clause keeps a break,
  but not at a spot the predicate reports.
- All forced counts are widths of the line the rule sees.
  A Rust doc comment's host line is longer by the item's indent and the comment marker,
  so a bound on the host line would move some clauses into the forced set;
  question 7 says how far.

The class issue 6642 names is
**a line of one or two words that itself ends at no boundary**:
`39,485` breaks in `4,728` files.
`36,540` of them are removals the shipped join's guards allow,
`1,163` are emphasis endings,
which leaves `38,322` durable findings in `4,700` files,
and `5,613` sit in a clause that reaches `120`.
The reverse shape,
a break that leaves a one-or-two-word line after it,
is `73,414` in `7,072` files,
of which `62,521` are removable.
They are not the same class.

The shape of three of the issue's own four examples is not the class the issue names.
`393,916` quiet one-or-two-word lines end at a break point,
which the rule wants,
and `39,485` end at none.
`/// 4,` and `/// held,` end at a comma,
so they keep their lines and no predicate below reports them.

Where the findings sit,
for the population:

```text
doc/planning        45,636      doc/handover        29,755
doc/troubleshooting 32,347      doc/audit           17,400
doc/decision        10,168      doc/research         2,464
doc/runbook          1,103      doc (every family) 140,090
package, Markdown   29,259      package, Rust doc comments 40,316
the rest               542
```

So `66.6` percent of the population is under `doc/`,
`13.9` percent is Markdown under `package/`,
`19.2` percent is Rust doc comments under `package/`,
and `0.3` percent is elsewhere.

Asked the other way,
an author meets at least one finding of a predicate
in this share of the `10,352` surveyed files,
which is each predicate's file count over the surveyed files:
`A` `84.24` percent (`8,721`),
`B` `79.07` percent (`8,185`),
`D` `78.10` percent (`8,085`),
`F1` `68.32` percent (`7,072`),
`F2` `45.67` percent (`4,728`).
At `error` that is the share of existing files where an edit would meet a finding
its author did not write.

## What each predicate would report

Five predicates report,
and a sixth answer reports nothing.
Each is counted in its own population,
in clauses where it reports clauses and in breaks where it reports breaks.
`output.txt` labels the first five `A`,
`B`,
`D`,
`F1` and `F2`.

Every predicate takes the owner's answer to question 2 as a filter,
so the counts below are the counts under "permit none of the six kinds".
The ranking section gives the same counts under the other answers to question 2.

The examples are real files at the survey commit,
one from `doc/` and one from a Rust doc comment for each predicate that reports,
with the class,
the clause width and the forced flag named
so that a reader can see which other predicate leaves the same break.
Each file's paragraph was checked to carry no finding of the shipped rule,
by a run of the installed binary recorded for this round.
The class and width of each were measured by a script kept with this round's probes
and checked against the three examples `output.txt` prints:
it reproduces `83` characters and the `conjunction` class at
`.agents/skills/css/SKILL.md:170`,
`162` and `inline-span` at
`.agents/skills/android-emulator/SKILL.md:22`,
and `122` and `ordinary` at
`.agents/skills/choosing-technology/SKILL.md:688`.

### Report every break at no boundary (`A`)

`210,207` findings in `8,721` files.

Reports,
from `doc/philosophy/agents.md:313` (class `ordinary`,
clause `107` characters,
not forced,
next line two words):

```text
AD3 is a narrow architecture rule:
 prefer calling async functions directly over building descriptor objects that another
interpreter executes.
```

Reports,
from the Rust doc comment at
`package/git-policy/cli/src/native/service/wait_ceilings.rs:71`
(class `ordinary`,
clause `48` characters,
not forced,
next line one word):

```text
/// (`doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md`);
/// the service greeted the next time it was
///  asked,
```

Leaves a break in a paragraph the rule already reports,
as this fixture shows:

```text
Aaa bbb, ccc ddd eee
the
fff ggg hhh.
```

At `error` a scoped run exits `1` on any of its findings;
at `warn` it exits `0` and `--fix` still runs.

A yes here leaves undecided which clause of a forced pair is the finding's position,
and that `A` at `error` can never reach zero (see "Which answers fix which,
and what cannot be built").

### Report only a break a removal can join (`B`)

`162,707` findings in `8,185` files:
every `A` break whose two lines the shipped join's guards allow,
with the one guard the survey lifts stated below.
`2,997` of them are emphasis endings the existing rule breaks again,
so the durable count is `159,710` in `8,164` files.

Reports,
from `doc/philosophy/agents.md:606`
(class `ordinary`,
clause `41` characters,
not forced,
own line two words,
next line five words):

```text
 The available-skill description supplies the trigger;
 the enumerated
detail lives in the skill.
```

Reports,
from the Rust doc comment at
`package/git-policy/cli/src/native/service/wait_ceilings.rs:192`
(class `ordinary`,
clause `54` characters,
not forced,
own line two words):

```text
/// and so the longest a commit waits behind one:
/// the longest
///  self-released wait with a margin of five,
```

Leaves the break at `doc/philosophy/agents.md:180`
and the one at `wait_ceilings.rs:63`,
because the joined line would reach the `120` bound.

A yes here leaves undecided that a fix does not exist,
and that the survey's count lifts the join's two-word fragment limit,
so `B` is not a count of what the shipped join does today.

### Report an unforced break at clause level (`D`)

One finding per clause under `120` characters
with at least one break at no boundary,
no hard break,
and no emphasis ending,
in a paragraph the rule reports nothing for.
`144,863` findings in `8,085` files,
covering `148,203` breaks.
The `47,811` forced clauses are not reported,
and neither are the `3,045` breaks that sit in unforced clauses this predicate leaves out
for an emphasis ending (`210,207 - 58,959 - 148,203 = 3,045`).

Reports the clause of `doc/philosophy/agents.md:313` quoted above,
and the clause of the Rust doc comment at `wait_ceilings.rs:71` quoted above.

Leaves the forced clause of `doc/philosophy/agents.md:180`:

```text
Any competent agent or developer already knows how to use `ripgrep` or when to reach for official documentation
over source code.
```

Two things about this predicate are not designed and are not asked here:
where one finding per clause is anchored (its first break,
or the clause),
and whether a clause of three breaks prints one finding or three.
A yes here leaves both undecided,
and leaves undecided what a permitted kind does to a clause
(question 2 states the two readings and which one the survey counted).

### Report a break that leaves a one-or-two-word next line (`F1`)

`73,414` findings in `7,072` files,
of which `62,521` are removable,
and `14,694` sit in a clause that reaches `120`.

Reports the break at `doc/philosophy/agents.md:313` quoted above
(next line:
`interpreter executes.`),
and the break at `wait_ceilings.rs:71` quoted above
(next line:
`asked,`).

Leaves the break at `doc/philosophy/agents.md:606` quoted above,
whose next line holds five words,
and the break at `wait_ceilings.rs:192` quoted above,
whose next line holds seven.

A yes here leaves undecided that a rewrap clears every one of its findings,
the `14,694` forced ones included.

### Report a one-or-two-word line that ends at no boundary (`F2`)

The class issue 6642 names.
`39,485` findings in `4,728` files,
of which `36,540` are removable.
`1,163` are emphasis endings and are excluded first,
which leaves `38,322` durable findings in `4,700` files,
and `5,613` sit in a clause that reaches `120`.

Reports the break at `doc/philosophy/agents.md:606` quoted above
(own line:
` the enumerated`),
and the break at `wait_ceilings.rs:192` quoted above
(own line:
`/// the longest`).

Leaves the break at `doc/philosophy/agents.md:313` quoted above,
whose own line holds eleven words,
and the break at `wait_ceilings.rs:71`,
whose own line holds eight.

A yes here leaves undecided that a rewrap clears every one of its findings,
and that the `1,163` emphasis endings are the rule's own inserted break.

### Report nothing

No finding,
no churn,
and the `39,485` short lines stay.

A yes here leaves undecided nothing about this rule,
and answers the whole proposal:
the class is not reported.

## The removal count, and what "removable" means

No removal fix exists in the tree.
The removal count in this survey asks the shipped join,
`markdown_fragment_joins::fragment_join`,
about each break at no boundary.
The caller passes the line the break ends as the fragment.
The only guard lifted for the call is `FRAGMENT_WORDS`,
which the shipped join sets to two words;
every other guard is the shipped one:
the hard break,
the break-point ending,
the following block marker,
and the joined width against `120` characters.

That is a model of a fix that does not exist,
not a run of a shipped command.
It measures each break pair by pair.
It does not measure a run that joins a whole clause,
the ten-pass fix loop (`fix_loop.rs`,
`MAX_PASSES`),
the Rust doc-comment projection (`processors_docs.rs`,
`run_margin`),
or the cycle an emphasis ending creates.

The shipped join is narrower than that model in one way that matters here:
it answers only about the fragment one of the rule's own inserted breaks cut
(`markdown_semantic_breaks.rs` builds the `breaks` list and asks `fragment_joins` about it).
A line that was already alone in the input is never joined,
which is why `10,028` of the `11,552` breaks in paragraphs the rule reports
survive a run of the installed fix on copies of their files.

A detail of the guards that the class table does not show:
`starts_block_construct` refuses a following line whose first byte is `>`,
`<` or `|`,
so a break whose next line opens a quote marker is never removable,
and the `block-marker` class does not list `<`,
so such a line is filed under another class while the join refuses it.

## What the records say, and what still stands

`doc/planning/repository-linter-stays-at-zero.md` records,
at lines 14 and 16 of the rerun commit:

- A repository-wide run at `136a40fe69` reported `174,916` findings,
  of which this rule made up `171,262`.

The same record quotes the owner's words,
moved verbatim on 2026-11-23,
at lines 40 to 43:

```text
"Put the semantic line break findings on hold."
then,
"Put the semantic line break findings on hold.
They aren't the most serious issues right now."
```

At line 186 the record says the fixer
"changed rendered text in 58 of the 498 files it changed in the documentation families",
which is why a blind sweep of quoted records is not safe.

Two more passages of that record bear on the questions below,
and neither is the owner's word:

- At lines 254 to 259,
  moved from the old handover,
  it counts what a sweep of `doc/decision` would meet:
  "`doc/decision` 1,785 (answered on 2026-11-10:
  form-only prose fixes,
  and moved-path notes only);".
  The record does not say who answered,
  so this proposal does not attribute the answer to the owner.
- At lines 123 to 137 it lists the three behaviors that made the fixer's run refuse files:
  it breaks `///` doc comments inside fenced Rust listings,
  it wraps a URL that sits inside a code span,
  and it converts a pipe table to an HTML table,
  and it says they "belong in the linter package"
  and are "listed in the report of the branch that fixed the findings".
  That file does not say whether they are fixed,
  and this round did not find that report.
  So any fix,
  a new removal fix included,
  has to be shown clear of them,
  and this proposal does not claim they are fixed.

`doc/handover/cli-git-rust-implementation.md` counts a later sweep,
at lines 2749 and 2750 of that file at the survey commit,
and at lines 2865 and 2866 after the merge:

```text
Both are fixed (`beb61ec46b`):
4,062 findings in 480 files,
```

Both figures are that record's own measurements,
made at their own commits by the rounds that ran them;
this round did not rerun them,
and their passages are kept beside the survey in
`figures-the-survey-does-not-count/`.

What still stands:
the hold is the last recorded owner instruction about this rule,
and no later record lifts it or renews it.
The rule is still at `error` severity in `monochromatic-lint.config.jsonc`
(lines 258 to 260 of the rerun commit),
and the fixer's earlier rewrite of rendered text remains the reason
a blind sweep of quoted records is not safe.

## What the owner's own prose does

The owner's own `AGENTS.md` is the one corpus where the owner's wrapping is on record.
A walker over that file at the survey commit,
kept with this round's probes,
counts `1,325` breaks inside paragraphs and `21` at no boundary:
`13` before a conjunction,
`7` before a parenthesis,
and `1` in the ordinary class.
The walker reads no syntax tree and skips headings and fences by their first character,
so the figure is indicative and not a measurement of the rule's own paragraphs.
It is evidence for question 2,
and it says the owner's own prose wraps before a conjunction
and before a parenthesis far more often than anywhere else.
It also says the owner's prose does not end a line at a dash,
which WR2 forbids.

Two sweeps merged while the hold stood,
and the owner needs them to answer that question:

- `beb61ec46b`,
  a merge dated 2026-11-27T18:38:01-05:00,
  which ran the whitespace-only fix over `package/git-policy/cli`;
  the handover passage above counts its result as `4,062` findings in `480` files.
- `96c930201f`,
  dated 2026-11-27T22:35:41-05:00,
  which repaired `771` doc paragraphs in `367` files
  by running the installed fix over the pre-sweep copies,
  netting `90` fewer doc lines.

No enforced gate stands behind the rule:

- `monochromatic-git-policy-cli.config.jsonc` line 10 runs
  `"markdown/autofix": ["warn", { "rules": ["lfs-image-url"], ... }]`:
  `git commit` through the wrapper runs no semantic-break check.
- `mise.toml` lines 1210 to 1215 make `format:monochromatic-lint` run
  `monochromatic-lint --fix` over `.` when no paths are given.
- `mise.toml` lines 1024 to 1025 say the linter
  "Exits 1 on error findings and 2 when the run could not be completed;
  warnings alone exit 0."
- The installed `0.5.2-alpha.24` on `package/git-policy/cli` at the rerun commit
  reports `26` findings in `20` files and exits `1`:
  `14` `rust/max-lines` errors,
  `11` `rust/folder-uses-links` errors,
  and one `rust/avoidable-closure` warning.
  It reports no finding of this rule there.
  The command and its whole output are kept beside the survey.
  This proposal claims no repository-wide figure;
  that one package is what it measured.

So "returns to green" has no green baseline,
and a warn-severity release exits `0` while still applying `--fix`.
At `error` a run without `--fix` exits `1`;
at `warn` the same run exits `0`.
Both severities apply `--fix` when it is passed.

## The kinds, and one quoted break of each

Question 2 asks which of these kinds MD1 permits.
Each kind is one row of the class table,
each is shown by a real break at the survey commit,
and each example is the kind it is shown for:

- `block-marker` (`65` breaks):
  the next line opens a marker the shipped join refuses.
  From `doc/audit/tech-mold-linker-vet-2026-10-05.md:786`:

  ```text
    #1695 closed unmerged with follow-up issue
    #1696 open (thread-pool tuning discussion);
  ```

- `parenthesis` (`37,033`):
  the next line opens with `(`.
  From `.agents/skills/choosing-technology/SKILL.md:151`:

  ```text
   buying because the user may delegate the enumeration to a cheaper model of their choosing
   (not necessarily Haiku)
  ```

- `inline-span` (`25,355`):
  the next line opens with a backtick,
  a `*`,
  an `_`,
  a `~`,
  a `[` or a `!`.
  From `.agents/skills/android-emulator/SKILL.md:22`:

  ```text
   Briefs for agents that use an emulator repeat the headful rule and name the launch form in
   `doc/troubleshooting/android-emulator-37-software-renderer-sigsegv.md`;
  ```

- `conjunction` (`19,749`):
  the next line opens with one of the 31 words listed below.
  From `.agents/skills/css/SKILL.md:170`:

  ```text
   user-configurable properties
  where the fallback represents the default user choice.
  ```

- `dash` (`20`):
  the line before the break ends in `--`,
  `—` or `–`.
  From `.agents/skills/project-code-review/PHILOSOPHY.md:24`:

  ```text
  The alternative -- referencing `AGENTS.md` sections by name ("apply the rules from the TypeScript Standards section") --
  assumes the agent will cross-reference accurately across documents under context pressure.
  ```

- `ordinary` (`127,985`):
  none of the above.
  From `.agents/skills/css/SKILL.md:54`
  (class `ordinary`,
  clause `94` characters,
  not forced):

  ```text
  Anything supported in evergreen Firefox but not ESR 140
  is off-limits without a feature query.
  ```

The conjunction kind is a fixed list,
not a grammar,
and it is these 31 words:
`and`,
`but`,
`or`,
`nor`,
`yet`,
`so`,
`for`,
`because`,
`although`,
`though`,
`while`,
`when`,
`where`,
`which`,
`who`,
`whom`,
`whose`,
`that`,
`if`,
`unless`,
`until`,
`since`,
`as`,
`before`,
`after`,
`whether`,
`than`,
`then`,
`plus`,
`versus`,
`via`.
`that`,
`which`,
`as`,
`for`,
`so`,
`then`,
`than` and `via` are in it,
and a reader may not picture those as the boundaries MD1 means,
which is why the list is here.

A seventh kind is not in that table:
the **closer-ended** break,
whose line ends with a bracket or a quote (`9,603` breaks).
Every predicate reports it,
and `output.txt` prints its total without its split among the six classes,
so an answer that permitted it would remove at most `9,603` breaks from each predicate,
and its exact figure is not counted.
An owner who wants it counted needs a rerun that prints the joint table,
and this proposal does not offer it as a costed option.

## The questions, and which answers fix which

Nine questions follow,
and they are not independent.
Each is one decision,
each lists its options,
and each says what a yes leaves undecided.

The questions:

1. Does the hold still govern this rule?
1. Which kinds of break does MD1 permit?
1. Which predicate reports?
1. Which rule id carries the class?
1. At which severity does it first report?
1. Does the release that first reports it also carry a removal fix?
1. Which bound does a join use?
1. What mechanism repairs the breaks already in the tree?
1. Which files does the first repair round cover?

Which answer fixes which:

- Question 1 gates every other question.
  If the hold still governs this rule,
  then none of questions 3 to 9 can take effect:
  the class cannot ship,
  and their answers become a plan for later.
  It does not settle whether the two merged sweeps were within it,
  which stays the owner's to say.
- Question 2 fixes the population,
  and so fixes every count in question 3 and the order of the ranking:
  a kind the owner permits leaves every predicate.
  For a clause predicate it also fixes what a permitted kind does,
  and the survey counted only one of the two readings:
  it drops the whole clause (the `D2` row of `output.txt`),
  not the break,
  so under the three-edge answer `D` has `106,460` findings,
  where the same answer read at break level would give a number nobody counted.
- Question 3 fixes what a fix must do:
  a break predicate needs a fix that joins two lines,
  and a clause predicate needs one that joins a whole clause,
  which does not exist and is not designed.
- Questions 4 and 5 interact in one direction:
  if the class shares the existing rule id,
  then its severity is that id's,
  so a `warn` answer demotes today's `error` findings
  and is an approval under `AGENTS.md` rule LN7
  ("Never loosen lint rules without prior approval.").
  With a rule id of its own,
  question 5's answer loosens nothing.
- Question 6 depends on question 8:
  a release cannot carry a fix that no answer of question 8 builds.
- Question 7 fixes the forced split for Rust doc comments:
  the survey's widths are the line the rule sees,
  and a bound on the host line would move some of their clauses into the forced set.
- Question 9 depends on question 8:
  a mechanism that does not exist has no scope yet.

Combinations that cannot be built,
or that can never reach zero:

- `A` at `error` cannot reach zero under any answer to question 2 short of permitting the ordinary class.
  `26,558` of the `58,959` forced breaks are ordinary-class,
  a forced clause must keep at least one break,
  and that break stays at no boundary unless the author adds punctuation or words.
  Under "permit none" the irreducible number is between `47,811` (one per forced clause) and `58,959`;
  under any other answer it is between one and the ordinary forced breaks the answer leaves reportable.
  So an owner who answers `A` and `error` is choosing a class that
  `doc/planning/repository-linter-stays-at-zero.md` lines 8 to 10 calls "taken care of asap",
  and that no author action clears.
- A clause predicate with a release that carries a removal fix cannot be built:
  the only fix modelled is the pairwise join,
  and a clause fix does not exist and is not designed.
- The existing rule id at `warn` cannot be had without loosening today's `error` findings,
  so that pair needs the owner's approval as an LN7 approval,
  which question 5 states in its option.
- `D` with question 2's three-edge answer is `D2`,
  and `D2` with "permit none" is a contradiction:
  `D2` is defined with those three edges removed.
  This is why `D2` is not one of the predicates question 3 offers.
- "Report nothing" together with any answer to questions 4 to 9 is undefined:
  a class that reports nothing carries no severity,
  no fix and no bound.
- A rule id of its own cannot reuse the notices file's rule-off entry:
  that entry names `markdown/semantic-line-breaks`
  (`monochromatic-lint.config.jsonc` lines 339 to 341),
  so the notices file keeps the new class unless a second entry is added.

"Both,
in this order" is reachable at every question:
the kinds can be permitted in stages,
the severity can rise in a later release,
the fix can follow the report,
and the repair round can follow the mechanism.
## The ranking of the predicates

The criterion comes first,
and it is the author's proxy,
not a measurement of a defect.
A predicate is ranked by the share of its own findings
that fall in the class the harness calls `ordinary`,
and then,
among predicates that agree on that share,
by how many ordinary-class breaks it puts in front of the author.
What is built today,
how many files a change would touch,
and which fix a predicate needs are costs,
stated in "What each predicate would cost" and never used to place a predicate lower.

Three things about that proxy,
said here rather than in a note:

- No reader judged any break.
  The `ordinary` class is a definition of the harness's:
  a break whose next line opens
  with none of the markers,
  brackets,
  span openers,
  listed words or dashes.
  It stands for "the kind of break MD1 and MD8 do not bless",
  and that standing is the author's.
- The class is not all defect:
  `26,558` of its `127,985` breaks sit in a clause at or over `120` characters,
  which no rewrap of that clause clears,
  so the proxy counts as a defect `20.8` percent of breaks nobody can clear by hand.
- It is not the only criterion a reader would use.
  Four others reorder the list,
  each computable from `output.txt`:
  the share of each predicate's findings a pair join can repair
  (`B` `100`,
  `F2` `92.5`,
  `F1` `85.2`,
  `A` `77.4` percent,
  and no figure for `D`,
  which has no fix);
  the ordinary breaks put in front of the author
  (`A` `127,985`,
  `B` `106,692`,
  `D` `98,708`,
  `F1` `39,464`,
  `F2` `28,731`);
  the findings per file an author meets
  (`A` `24.1`,
  `B` `19.9`,
  `D` `17.9`,
  `F1` `10.4`,
  `F2` `8.4`);
  and the breaks left unreported out of the `210,207`
  (`A` none,
  `B` `47,500`,
  `D` `62,004`,
  `F1` `136,793`,
  `F2` `170,722`).
  The last of these puts the list in nearly the reverse order.

The share of each predicate's own findings that are ordinary,
under "permit none" of question 2,
each a fraction of a row of `output.txt`:

```text
the short line          72.76%  =  28,731 /  39,485   (F2)
the unforced clause     66.71%  =  96,637 / 144,863   (D, one finding per clause)
every joinable break    65.57%  = 106,692 / 162,707   (B)
every break             60.89%  = 127,985 / 210,207   (A)
the short next line     53.76%  =  39,464 /  73,414   (F1)
nothing                 no share: it reports no finding
```

`D`'s fraction is its finding unit,
one per clause:
`96,637` clauses whose first break is
ordinary,
out of `144,863` clauses.
At the break unit it is `98,708 / 148,203 = 66.60` percent,
which is the figure `output.txt`
also prints and which changes no place in the order.

A sixth row belongs to this table and not to the ranking:
the unforced clause with the three edges removed is `90.24` percent
(`96,066 / 106,460`),
first by construction,
because it drops exactly the clauses whose breaks the share excludes.
It is `D` under question 2's three-edge answer,
so it is not an answer to question 3.

The ranking:

the short line > the unforced clause > every joinable break > every break > the short next line > nothing,
because:

- The short line over the unforced clause:
  `72.76` percent against `66.71`,
  and the same unit separates them:
  `20.35` percent of the short line's findings (`8,035` of `39,485`) sit at the three edges,
  against `25.99` percent of the unforced clause's (`37,652` of `144,863`).
  The short line is also the shape issue 6642 names.
- The unforced clause over every joinable break:
  `66.71` against `65.57`,
  because the clause predicate leaves out every clause holding an emphasis ending,
  and `2,997` of the joinable break's `162,707` findings (`1.84` percent) are endings
  the rule would break again.
- Every joinable break over every break:
  `65.57` against `60.89`,
  because it leaves out the `47,500` breaks the join's guards refuse,
  and those refused breaks are less often ordinary than the population:
  `21,293` of the `47,500` (`44.83` percent) against `60.89` percent of all of them.
- Every break over the short next line:
  `60.89` against `53.76`,
  because `43.87` percent of the short next line's findings (`32,208` of `73,414`)
  sit at the three edges against `29.69` percent of every break's (`62,408` of `210,207`).
- The short next line over nothing:
  it reports `39,464` ordinary breaks,
  and nothing reports none,
  so the last pair compares a finding class with the absence of one.

How the order changes under each answer to question 2.
The shares below are computed at the break unit:
each permitted kind's breaks leave the predicate,
the ordinary count is untouched,
and the share is the ordinary count over what is left.
That is the reading of "a kind the owner permits is removed from every predicate";
for `D` and `D2` it is not their own rule,
which drops whole clauses,
and the clause-unit figure is printed only for the three-edge answer (`D2`).
Each line is a fraction of `output.txt` rows:

```text
permit none of the six kinds
  D2 90.24% =  96,066 / 106,460
  F2 72.76% =  28,731 /  39,485
  D  66.60% =  98,708 / 148,203
  B  65.57% = 106,692 / 162,707
  A  60.89% = 127,985 / 210,207
  F1 53.76% =  39,464 /  73,414

permit the inline-span kind
  D2 90.24% =  96,066 / 106,460
  F2 83.65% =  28,731 /  34,348
  D  74.13% =  98,708 / 133,159
  B  73.48% = 106,692 / 145,195
  A  69.24% = 127,985 / 184,852
  F1 65.14% =  39,464 /  60,585

permit the inline-span and parenthesis kinds
  F1 95.76% =  39,464 /  41,212
  F2 91.35% =  28,731 /  31,452
  D2 90.24% =  96,066 / 106,460
  D  90.06% =  98,708 / 109,600
  B  89.55% = 106,692 / 119,139
  A  86.58% = 127,985 / 147,819

permit those two and the dash kind
  F1 95.77% =  39,464 /  41,206
  F2 91.35% =  28,731 /  31,450
  D2 90.24% =  96,066 / 106,460
  D  90.07% =  98,708 / 109,585
  B  89.57% = 106,692 / 119,122
  A  86.59% = 127,985 / 147,799

permit those three and the conjunction kind
  B  99.96% = 106,692 / 106,730
  D  99.96% =  98,708 /  98,747
  D2 99.96% =  96,066 /  96,066
  F2 99.95% =  28,731 /  28,744
  A  99.95% = 127,985 / 128,050
  F1 99.90% =  39,464 /  39,503
```

The `D2` row never moves,
because those three edge columns are already absent from it.
The `D` row is the break-unit reading,
not `D`'s own rule.

What the table says:

- Under "permit none",
  the ranking above stands.
- Under permitting the inline-span kind,
  the order is unchanged and every share rises.
- Under permitting spans and parentheses,
  the short next line goes from last to first
  (`53.76` to `95.76`),
  and `A` stays last.
- Under permitting the three edges as well as the conjunction kind,
  every predicate is at or above `99.90` percent and the top four are within `0.06` points:
  the ranking carries no information at all there,
  and an owner who answers question 2 that way should answer question 3 on something else.
- The conjunction-only answer,
  which earlier versions of this proposal described,
  is not one of the options,
  and its order was `D2` `99.96`,
  `F2` `78.12` (`28,731 / 36,779`),
  `D` `71.86` (`98,708 / 137,365`),
  `B` `70.98` (`106,692 / 150,315`),
  `A` `67.21` (`127,985 / 190,458`)
  and `F1` `55.03` (`39,464 / 71,711`):
  the clause predicates do not both rise above the short line there,
  only the edge-free one does.

So the ranking in this proposal is a ranking under one answer to question 2,
and it is not a recommendation until question 2 is answered.

## What each predicate would cost

These are costs,
not reasons to place a predicate lower.

- No removal fix exists.
  Every predicate except "nothing" reports findings no shipped command clears today.
  A removal fix built from the shipped join's guards would clear every finding of every joinable break,
  and for the other predicates only the part of it the guards allow:
  `36,540` of the short line's `39,485`,
  `62,521` of the short next line's `73,414`,
  and every break's `162,707` of `210,207`.
  It would not clear `47,500` breaks,
  because the guards refuse them,
  and it would not clear a forced clause's last break.
  The clause predicate needs a fix that joins more than two lines,
  and that operation,
  its guards and its width are unmeasured because it does not exist.
- Files a change would touch:
  `8,721` for every break,
  `8,185` for every joinable break,
  `8,085` for the unforced clause,
  `7,072` for the short next line,
  `4,728` for the short line,
  and none for nothing.
- Findings the rule breaks again:
  `3,383` for every break,
  `2,997` for every joinable break,
  `1,163` for the short line,
  and none for the clause predicate.
  These are the survey commit's,
  where the boundary test did not climb the emphasis delimiters;
  after the merge named in question 7 it does,
  so the class is empty there.
- The exit status:
  at `error` a scoped run exits `1` on any finding,
  and at `warn` it exits `0` while `--fix` still rewrites the file.
- What clears a finding,
  predicate by predicate:
  - Every break,
    an ordinary-class forced one included:
    nothing a person can do short of adding
    punctuation or words clears it,
    and the join clears the `162,707` its guards allow.
    Under "permit none",
    at least `47,811` findings survive any rewrap.
  - Every joinable break,
    and the short line and the short next line:
    a rewrap clears the finding.
    The forced short-next-line example in this proposal is the proof:
    `android-emulator/SKILL.md:22` joins to `162` characters and stays forced,
    and it rewraps into a `66`-character line ending `... the headful rule and`
    and a `95`-character line that begins `name the launch form in`
    and ends with the path's code span and a semicolon,
    six words in all,
    whose next line is no longer short,
    so the forced `14,694` of the short next line and the `5,613` of the short line
    are clearable by hand even though their clauses stay forced.
  - The unforced clause:
    a rewrap that keeps the clause under `120` clears it.

## What the owner has to decide

### 1. Does the hold still govern this rule

The owner's recorded instruction is
"Put the semantic line break findings on hold.",
moved verbatim on 2026-11-23 into
`doc/planning/repository-linter-stays-at-zero.md` lines 40 to 43.
No later record lifts it or renews it.
While it stood,
two sweeps of this rule's findings merged:
`beb61ec46b` on 2026-11-27,
counted in the handover as `4,062` findings in `480` files,
and `96c930201f`,
which repaired `771` doc paragraphs in `367` files.
This question gates every other question in this proposal.

The options:

- **It still governs this rule.**
  Nothing of this proposal ships:
  no new class,
  no fix,
  no sweep.
  What it changes for good:
  nothing;
  the rule reports what it reports today.
  What it gives up:
  issue 6642 stays open,
  and the `210,207` breaks stay unreported.
  A yes here leaves undecided nothing about the class,
  and makes questions 3 to 9 a plan for later.
- **It governs today's findings only,
  and not a new class.**
  Today's `210,207` breaks stay unreported,
  and a class that question 3 picks may ship and report its own population.
  What it changes for good:
  the rule's existing findings keep the hold,
  while a new code reports the class the owner answered for.
  What it gives up:
  two codes to keep in step,
  and today's findings keep their `210,207`-finding queue.
  A yes here leaves undecided whether the merged sweeps were within the hold (it does not say),
  and what the class's own first report will meet in the files those sweeps already touched.
- **It is lifted.**
  Today's findings and any new class report at whatever severity question 5 answers.
  What it changes for good:
  the rule reports `210,207` breaks at that severity.
  What it gives up:
  a queue of `210,207` findings with no fix in the tree,
  against the owner's recorded principle that lint issues are taken care of asap.
  A yes here leaves undecided what clears them (question 8),
  and whether the two merged sweeps
  count as having pre-empted it.

Ranking:
the second > the first > the third,
because:

- The second over the first:
  the second lets the owner's answer to question 3 take effect while keeping `210,207` findings out of
  the queue,
  and the first delivers nothing it does not already have.
- The first over the third:
  the third creates a `210,207`-finding queue that no fix in the tree can clear,
  and the first keeps the tree as it is.

### 2. Which kinds of break does MD1 permit

`AGENTS.md` rule MD1 names "semantic boundaries" and a bound and no set of characters.
`AGENTS.md` rule MD8,
which is about hard wraps,
names inline spans,
code spans,
emphasis and link syntax,
and names neither a parenthesis nor a dash.
Rule WR2 forbids a dash in prose at all.
The survey files every break by the first token of the next line's prose,
and "The kinds,
and one quoted break of each" quotes one break of each kind.
The population of each kind,
and what each answer removes from it:

```text
block-marker      65
conjunction   19,749
dash              20
inline-span   25,355
ordinary     127,985
parenthesis   37,033
```

Every answer below says two things,
because a yes covers what the option's text says:
what the linter stops reporting,
and what `AGENTS.md` rule MD1 records.
An answer that leaves MD1 unchanged says so.

The options:

- **Permit none of the six kinds.**
  The linter reports all `210,207` breaks;
  MD1 keeps no enumeration.
  What it changes for good:
  nothing.
  What it gives up:
  `62,408` findings at shapes the owner's own prose uses
  (`13` conjunction breaks and `7` parenthesis breaks in `AGENTS.md` alone).
  A yes here leaves undecided which of the six the owner would permit later,
  and that the order in "The ranking of the predicates" is the order this answer gives.
- **Permit the inline-span kind.**
  The linter stops reporting a break whose next line opens with a code span,
  an emphasis span,
  a link or an image,
  which is to say a backtick,
  a `*`,
  an `_`,
  a `~`,
  a `[` or a `!`,
  `25,355` breaks;
  MD1 gains "a line may end before a code span,
  an emphasis span or a link".
  What it changes for good:
  those `25,355` breaks are never reported again.
  What it gives up:
  a break before a span that would have read better on the previous line
  is no longer reported.
  A yes here leaves undecided the parenthesis and dash kinds,
  and the rule text's exact wording.
- **Permit the inline-span and parenthesis kinds.**
  That is `62,408` breaks and `147,819` reportable;
  MD1 gains "before a parenthesis" as well.
  What it changes for good:
  `37,033` parenthesis breaks are never reported again.
  What it gives up:
  a parenthetical that a reader would rather see on the previous line
  is no longer reported.
  A yes here leaves undecided the conjunction and dash kinds,
  and whether a clause predicate
  drops the whole clause or only the break (see "The questions,
  and which answers fix which").
- **Permit those two and the conjunction kind.**
  That is `82,157` breaks and `128,050` reportable;
  MD1 gains "before one of a listed set of conjunctions and subordinators",
  and the list becomes part of the rule text:
  the 31 words quoted under "The kinds,
  and one quoted break of each",
  `that`,
  `which`,
  `as`,
  `for`,
  `so`,
  `then`,
  `than` and `via` among them.
  What it changes for good:
  `19,749` conjunction breaks are never reported again,
  and the repository's rule text holds a 31-word list that has to be maintained.
  What it gives up:
  a line opening with `that` or `which` is a shape many readers call a bad wrap,
  and it is no longer reported.
  A yes here leaves undecided the dash kind and the closer-ended kind.
- **Permit those three and the dash kind.**
  That is `82,177` breaks and `128,030` reportable,
  and it needs a change to `AGENTS.md` rule WR2,
  because every one of the `20` dash breaks is a line ending in a character WR2 forbids in prose.
  What it changes for good:
  `20` breaks are never reported again,
  and WR2 either gains a prose exception or is contradicted by the rule.
  What it gives up:
  the repository's own prose rule.
  A yes here leaves undecided the closer-ended kind and what WR2's new text says.

Ranking:
the fourth > the third > the second > the first > the fifth,
because:

- The fourth over the third:
  it removes `19,749` conjunction findings at a shape a line-opening subordinator makes normal,
  and it takes nothing away that the other answers keep:
  all `127,985` ordinary breaks stay reportable under every answer here.
- The third over the second:
  it removes `37,033` parenthesis findings,
  and a wrap before a parenthetical is the
  commonest way a writer keeps a long sentence readable,
  which `AGENTS.md` itself does `7` times.
- The second over the first:
  it removes `25,355` inline-span findings,
  the one kind MD8 names by name.
- The first over the fifth:
  the fifth removes `20` more findings than the first,
  and each of the `20` is a line ending in a dash that WR2 forbids in prose,
  so the fifth hides findings instead of blessed wraps.

The seventh kind,
the closer-ended break (`9,603`),
is offered by no option here because `output.txt` prints its total without its split among
the six classes:
a yes to permitting it would remove at most `9,603` from each predicate and its exact count is not measured.
A yes to any option here leaves that kind undecided.

### 3. Which predicate reports

The five predicates and "nothing" are in "What each predicate would report",
each with its counts,
its file count and a real example from `doc/` and from a Rust doc comment.
Each takes question 2's answer as a filter,
so the counts below are under "permit none",
and "The ranking of the predicates" gives the order under every answer.

The options:

- **The short line (`F2`),
  `39,485` findings in `4,728` files.**
  One finding per line of one or two words that ends at no boundary.
  What it changes for good:
  the shape issue 6642 names is reported,
  `36,540` of its findings are joinable by a fix whose guards allow them,
  and a rewrap clears every one of them.
  What it gives up:
  `4,728` files gain a finding,
  `36,540` of its findings wait for a fix that does not exist,
  and `5,613` sit in a clause that reaches `120` and need a rewrap,
  not a join.
  A yes here leaves undecided the severity,
  the rule id,
  the fix,
  and which kinds question 2 permits.
- **The unforced clause (`D`),
  `144,863` findings in `8,085` files.**
  One finding per clause under `120` with a break at no boundary.
  What it changes for good:
  one finding covers a whole clause,
  so `148,203` breaks become
  `144,863` findings,
  and no finding sits in a forced clause or on an emphasis ending.
  What it gives up:
  the finding's position and message are undesigned,
  no fix can clear it,
  and a clause of three breaks prints one finding or three,
  undecided.
  A yes here leaves undecided the anchor,
  the message,
  the fix,
  and the reading of a permitted kind.
- **Every joinable break (`B`),
  `162,707` findings in `8,185` files.**
  What it changes for good:
  every finding is one a join's guards allow,
  so one fix would clear all of them.
  What it gives up:
  it reports `2,997` breaks the rule itself would break again,
  and `47,500` breaks are not reported at all.
  A yes here leaves undecided the fix (which does not exist),
  the severity and the rule id.
- **Every break (`A`),
  `210,207` findings in `8,721` files.**
  What it changes for good:
  nothing at no boundary is missed.
  What it gives up:
  at least `47,811` findings survive any rewrap,
  `47,500` are unjoinable
  by the guards,
  and `3,383` are emphasis endings the rule breaks again.
  A yes here leaves undecided the severity (which cannot make it reach zero) and the fix.
- **The short next line (`F1`),
  `73,414` findings in `7,072` files.**
  What it changes for good:
  `62,521` are joinable and a rewrap clears every one.
  What it gives up:
  `43.87` percent of its findings sit at the three edges question 2 may permit,
  so the widest answer there removes `32,208` of them.
  A yes here leaves undecided the severity,
  the rule id and the fix.
- **Nothing.**
  No finding,
  no churn,
  and the `39,485` short lines stay.
  What it changes for good:
  nothing.
  What it gives up:
  issue 6642 stays open.
  A yes here ends the proposal:
  no other question applies.

Ranking:
the short line > the unforced clause > every joinable break > every break > the short next line > nothing,
with the reasons in "The ranking of the predicates".

### 4. Which rule id carries the class

The rule takes one severity per run
(`semantic_line_breaks(context, severity)`),
and the configuration names one severity per rule id
(`monochromatic-lint.config.jsonc` lines 258 to 260),
so a class reported under `markdown/semantic-line-breaks` takes that rule's severity.

The options:

- **A rule id of its own.**
  The class gets its own code and its own severity.
  What it changes for good:
  it can be `warn` while today's findings stay `error`,
  it can carry a path list that only shrinks,
  and the notices-file rule-off
  (`monochromatic-lint.config.jsonc` lines 339 to 341) needs a second entry beside it.
  What it gives up:
  a second definition of "a break at no boundary" beside the rule's own
  break-point test,
  and two codes to keep in step.
  A yes here leaves undecided the code's name,
  its allow list,
  and whether the notices file
  is covered by it.
- **The existing rule id.**
  The class shares whatever severity `markdown/semantic-line-breaks` has.
  What it changes for good:
  one definition and one configuration entry.
  What it gives up:
  the class cannot be `warn` unless today's findings are,
  and the notices-file rule-off switches the new class off with them.
  A yes here leaves undecided the severity (question 5) and whether a `warn` answer is the
  loosening `AGENTS.md` rule LN7 forbids without prior approval.

Ranking:
a rule id of its own > the existing rule id,
because:

- The first over the second:
  the first lets the class report at a severity of its own while today's `error` findings stay
  where they are,
  and the second can only do that by demoting them,
  which is the loosening LN7 asks the owner to approve.

### 5. At which severity does it first report

At `error` a scoped run exits `1` on any finding;
at `warn` it exits `0`.
Both severities apply `--fix` when a fix exists,
and today none exists,
so a first release reports and rewrites nothing either way.

The options:

- **`warn`.**
  What it changes for good:
  every run reports the class and no run fails on it.
  What it gives up:
  the tree stays green while `210,207` (or the predicate's own count) go unreported
  by anything that stops a commit.
  A yes here leaves undecided when the severity rises,
  and whether today's findings move with it.
- **`error`.**
  What it changes for good:
  the finding stops a run,
  so it is acted on.
  What it gives up,
  with no fix in the tree:
  every file an author touches fails that run until the author
  rewrites the line by hand,
  and the repository's own principle that lint issues are cleared asap
  meets a queue with no automatic repair.
  A yes here leaves undecided what clears the findings (question 8),
  and,
  if question 4 answers with the existing rule id,
  that the class then carries that id's severity and today's findings keep theirs.

Ranking:
`warn` > `error`,
because:

- `warn` over `error` while no fix exists:
  `warn` reports every finding and fails no run,
  and `error` fails every touched file
  with nothing to repair it.
  Once question 6 ships a fix,
  `error` becomes the answer that delivers clearing over reporting,
  and the two swap.

### 6. Does the release that first reports it also carry a removal fix

The options:

- **Report now,
  fix in a later release.**
  What it changes for good:
  the class is measured in the tree while the fix is designed.
  What it gives up:
  the findings wait for a fix,
  and `--fix` changes nothing for them.
  A yes here leaves undecided which release carries the fix and what it does.
- **Report only when the fix ships,
  both in one release.**
  What it changes for good:
  no finding appears that nothing can clear.
  What it gives up:
  nothing of the class is measured until a fix exists,
  and no fix exists,
  so the class does not ship in this round either way.
  A yes here leaves undecided when the fix exists,
  and,
  for a clause predicate,
  that the fix cannot be built at all.

Ranking:
reporting now > reporting with the fix,
because:

- Reporting now over reporting with the fix:
  the first delivers the measurement this proposal exists to get,
  and the second delivers nothing until a fix that does not exist is designed.

### 7. Which bound does a join use

The shipped join measures the line the rule sees:
the paragraph's continuation indent,
the fragment and the following line,
against `120`.
In a Rust doc comment the host line is longer by the item's indent and the comment marker.
A joined virtual line of `119` characters is a host line of
`123` characters at item indent 0,
`127` at indent 4,
and `131` at indent 8,
measured with the installed fix on copies of one fixture.

**This question is adopted and veto open.**
Commit `fc679b9392` (committed 2026-11-28T03:02:49-05:00,
merged after the survey commit)
made the join count the host line,
its comment marker and host indent included,
so a joined Rust doc line stays under `120` in the host file,
and it settles in one pass;
the same commit makes the boundary test climb `*` `_` and `~`.
`package/linter/monochromatic-lint/doc/second-pass-reads-of-released-versions.md` records the choice,
naming the cost of the way not taken:
"it costs the rule writing lines over the limit it enforces,
which the owner's own bound in `AGENTS.md` rule MD1 forbids",
and then says "The veto stays open."
So the owner is asked for a veto,
not for a first choice.

The options:

- **Keep the host line's bound (the tree as it stands).**
  A joined Rust doc line stays under `120` in the file a reader opens.
  What it changes for good:
  nothing,
  it is what the tree does.
  What it gives up:
  a join is refused when the virtual line is under `120` but the host line is not.
  A yes here leaves undecided nothing;
  it is the state of the tree.
- **Veto it and return to the virtual line's bound.**
  That means reverting `fc679b9392`,
  correcting the linter `README.md` sentence that says the host line is the limit,
  and deciding what the tests `the_host_prefix_is_part_of_the_width`,
  `the_indent_the_fix_writes_is_part_of_the_width` and
  `a_joined_doc_line_stays_under_the_limit_in_its_host_file` become.
  What it changes for good:
  a join is measured by the line the rule sees,
  as `0.5.2-alpha.24` measures it.
  What it gives up:
  the rule writes lines over the `120` it enforces,
  by up to `11` characters at an indent of 8.
  A yes here leaves undecided which of those three tests survive in what form.
- **Veto it and set no bound at all.**
  A join may write any width.
  What it changes for good:
  no joined line is refused for its width.
  What it gives up:
  the rule enforces no bound in the line it writes.
  A yes here leaves undecided how a join chooses what to join.

The forced counts and both clause predicates are the line the rule sees under every answer,
so they do not move:
the survey's widths never included the host prefix.
A host-line bound would move some Rust doc clauses into the forced set,
bounded above by each predicate's Rust doc share:
`A` `19.18` percent,
`B` `21.23`,
`D` `22.52`,
`F1` `18.26`,
`F2` `14.80`.

Ranking:
keeping the host line's bound > vetoing to the virtual line's bound > vetoing to no bound,
because:

- The first over the second:
  the first is the only one where the bound the rule enforces holds on the line a reader opens,
  and the second writes lines up to `11` characters over it.
- The second over the third:
  the second keeps a bound of `120` on the line the rule sees,
  and the third keeps none.

### 8. What mechanism repairs the breaks already in the tree

The tree holds `210,207` quiet breaks at no boundary,
of which `162,707` are joinable by the shipped join's guards,
and the class question 3 picks is a smaller set inside it.
No shipped command repairs any of them.
The options:

- **A removal fix built on the shipped join's guards.**
  It joins each break's two lines where the guards allow,
  `162,707` of the `210,207`.
  What it changes for good:
  new fixes of this kind become automatic,
  and the lines that were already alone are reached,
  which the sweep route below did not reach.
  What it gives up:
  it does not exist,
  the `47,500` breaks the guards refuse stay,
  a clause of three lines needs more than it does,
  and it has to be shown clear of the three fixer behaviors the record lists.
  A yes here leaves undecided who designs it,
  its guards' exact widths,
  and when it ships.
- **The revert-and-fix route of `96c930201f`.**
  It ran the installed fix over pre-sweep copies and repaired `771` paragraphs in `367` files.
  Its own measurement (`81650758c4`) records what it reached and what it did not:
  of `548` one- and two-word doc lines that ended at no break point
  in the `471` swept files of `package/git-policy/cli/src`,
  it repaired exactly the `90` the sweep itself cut,
  and the `458` that were already alone stayed.
  What it changes for good:
  the lines a new wrap cuts are joined in the same pass.
  What it gives up:
  the lines already alone,
  and `3,367` one- and two-word doc lines
  in that package that prose follows with room under `120`,
  stay.
  A yes here leaves undecided what reaches the `458` and the `3,367`,
  and which round runs it.
- **Nothing.**
  The breaks stay,
  and the next old-style wrap adds more.
  What it changes for good:
  nothing.
  What it gives up:
  every break in the tree stays,
  and a new class's findings stay alongside them.
  A yes here leaves undecided nothing about repairs,
  and makes question 9 moot.

Ranking:
a removal fix > the revert-and-fix route > nothing,
because:

- A removal fix over the revert-and-fix route:
  the first reaches every break the guards allow (`162,707`) including the ones that were already
  alone,
  and the second reached `90` of `548` in the one package it ran over.
- The revert-and-fix route over nothing:
  the second at least repairs what a new wrap cuts,
  and the third repairs nothing.

### 9. Which files does the first repair round cover

This question has nothing to cover until question 8 answers with a mechanism.

The options:

- **The whole tree.**
  `8,721` files and `210,207` findings,
  `66.6` percent of them under `doc/` and `19.2` percent in Rust doc comments.
  What it changes for good:
  the class leaves the tree in one round.
  What it gives up:
  the largest rewrite any option here proposes,
  and the fixer's own record that it changed rendered text in `58` of the `498` files of one
  documentation sweep.
  A yes here leaves undecided how the rewrite is proved not to change rendered text.
- **The documentation families.**
  `140,090` findings under `doc/`,
  including the `1,785` in `doc/decision` that a recorded answer covers with form-only prose fixes
  and moved-path notes.
  What it changes for good:
  the largest single block leaves,
  in files whose edits the record
  already describes.
  What it gives up:
  the `40,316` Rust doc comment findings and the `29,259` package Markdown ones stay.
  A yes here leaves undecided what covers the rest.
- **The one package the route has run over,
  `package/git-policy/cli`.**
  `4,010` findings in `1,481` files,
  and `3,367` one- and two-word doc lines with room under `120`.
  What it changes for good:
  the smallest queue,
  and the only scope a route has actually run over.
  What it gives up:
  the other `206,197` breaks stay,
  `66.6` percent of them under `doc/`.
  A yes here leaves undecided when the other families get a round.

Ranking:
the whole tree > the documentation families > the one package,
because:

- The whole tree over the documentation families:
  the first reaches the `40,316` Rust doc comment findings and the `29,259` package Markdown ones,
  and the second leaves both.
- The documentation families over the one package:
  the second reaches `140,090` findings and the third `4,010`.

A yes to any of them leaves undecided where the round's commits land
and which other branches are changing those files,
which `doc/planning/repository-linter-stays-at-zero.md` names as a cost of its own option A.

## Shipped behaviour, and the commit each statement holds at

- The boundary test strips the nine closers and does not climb `*`,
`_` or `~`
  (`markdown_break_points.rs`,
`ends_at_break_point`),
  while the rule climbs `Strong`,
`Emphasis` and `Delete`
  (`markdown_prose_context.rs`,
`delimiter_tail`).
  This holds at `bc5b4ca2ba` and in `0.5.2-alpha.24`,
  where the emphasis fixture above is rewritten as shown.
- The join answers only about the fragments one pass's own planned breaks cut,
  so a line that was already alone in the input is never joined.
  This holds at `bc5b4ca2ba` and in `0.5.2-alpha.24`,
  where `Aaa bbb, ccc ddd eee` / `the` / `fff ggg hhh.`
  keeps its lone `the` after `--fix`.
  The wrapper's own file,
  `package/git-policy/cli/src/native/worktree_clone_add_route/mod.rs`,
  held such a line at `c8545f91b8`;
  at `bc5b4ca2ba` it does not,
  because the merge of `96c930201f` joined it.
  The defect remains reachable and its exhibit is now a fixture.
- The linter's `README.md` line 2629 at the survey commit said
  "The fix never rewrites a line the author wrote."
  That sentence held at `bc5b4ca2ba`,
  and a run on a copy shows an author's second line gaining a fragment at its start.
  The round `monochromatic-lint-alpha-24-read-the-line-break-fix-settles-in-one-pass-and-keeps-the-line-limit`
  removed or reworded it:
  the README on the branch after that round's merge holds no such sentence.
- The width a join counts was the line the rule sees
  at `bc5b4ca2ba` and in `0.5.2-alpha.24`,
  which the widths in question 7 measure,
  and it is the host line after the merge named there.
  The survey was not rerun for that change,
  and its counts are the commit's it names.
- No rule reports a fence that is never closed,
  and CommonMark runs an unterminated fence to the end of its container,
  so every Markdown rule,
and this survey,
skips what follows.
  That holds at `bc5b4ca2ba`
  (`package/linter/monochromatic-lint/README.md` lines 2635 to 2639 at the survey commit,
  and lines 2724 to 2728 after the merge,
  recorded by `7cfdc9199e`).
- At `error` a run without `--fix` exits `1` and at `warn` it exits `0`,
  and `--fix` applies at both severities.
  That holds in `0.5.2-alpha.24`,
  measured on one fixture.
- No shipped rule bounds a prose line's width.
  That holds at `bc5b4ca2ba` for all 30 rule ids,
  and a 123-character prose line passes the repository configuration with exit 0.

## How a line dodges each predicate

These routes follow from the definition and the guards and were not measured in use,
except the hard break,
of which the run found none:

- A line ending in two spaces or a backslash is a hard break.
  Every predicate leaves it alone,
  and one invisible edit silences a report while it changes the rendered text.
- A trailing comma,
period or colon moves a line to a break point,
which no predicate reports.
  The issue's own `/// 4,` and `/// held,` are that shape.
- A third word on a short line moves it out of the short-line and short-next-line predicates
  and leaves the break where it was.
- A non-breaking space or a full-width punctuation mark moves a line off the six-character test.

Whether a hard break or a trailing comma should itself be reported is not asked here.

## What this round built, and what it did not

The survey and its harness were built by the round that ran it,
and their record is `package/linter/monochromatic-lint/doc/a-survey-of-line-breaks-at-no-semantic-boundary/`.
The harness that produced `output.txt` is that folder's
`survey_line_breaks_at_no_semantic_boundary_tests.rs.keep` as commit `bc5b4ca2ba` holds it;
the kept file is a later form,
and `command.txt` names the difference.

This round,
the one that answered the third gate read,
built and ran,
each under its own scope:

- the recomputation of the ranking's shares under each answer to question 2,
  from the rows of `output.txt` (in "The ranking of the predicates");
- the derived figures this revision prints,
  each as a fraction of `output.txt` rows;
- the walker over the owner's own `AGENTS.md` that counts its breaks (in "What the owner's own prose does");
- a scanner that measures one break's class,
  clause width and forced flag,
  checked against the three examples `output.txt` prints,
  and used for the real examples in "What each predicate would report";
- a run of the installed binary over the three files those examples come from,
  which shows the paragraphs they sit in carry no finding of the shipped rule;
- the corrected after-the-fix control,
  run once on fixtures with the installed binary:
  its unfixed copy holds `1` break at no boundary,
  a fix run changes it to `Aaa bbb,` / `the ccc ddd eee.`,
  which holds `0`,
  and a second run reports nothing and exits `0`;
  the old control form's expectation of `2` held both before and after a fix run,
  which is why it could not fail;
- the count of the `133` paths that yield no source.

Changed nothing that ships:
no rule,
no fix,
no sweep,
no configuration,
no workflow.
The records this round changed are the survey folder's `README.md` and `command.txt`,
its `figures-the-survey-does-not-count/reported-paragraph-control.txt`,
and this proposal.

Inferred,
not run:
the removal counts,
the clause-width joins,
the Rust projection under a removal,
the ten-pass fix loop's behaviour on a clause fix,
and the two readings of a permitted kind for a clause predicate,
of which the survey counted only one.
Unknown:
how many of the `38,322` durable short-line findings a sweep would change
in files other branches hold,
how many breaks a forced clause must keep under the best rewrap
(the harness's per-clause widths are not in `output.txt`,
so this revision gives the bounds `47,811` and `58,959` and not the figure),
what the owner's kind list is,
whether the sweeps that merged while the hold stood were within it,
and whether the fixer's three recorded defects are fixed.

## Answers to the second gate read

Each item names the finding and what answered it.

- H1 asked for reasons and cons that use counts from another population or unit.
  Every reason now uses its predicate's own population,
  the ordinary share is stated for each,
  D and D2 are given in breaks beside clauses,
  and the short line's overlap with the kinds question is measured:
  `2,706` conjunction-class findings and `8,035` at the edges MD8 names.
- H2 asked for the repair route and the exhibit to be restated at the current commit.
  The status quo,
  the repair and question 8 are restated at the rerun commit,
  with what the revert-and-fix route reached,
  and the lone `the` no longer stands in the wrapper's file.
- M1 asked for the emphasis count beyond the removable branch.
  Every short-line emphasis ending is counted,
  and the durable files are counted over every durable finding.
- M2 asked for a closer-ended figure that matches the output.
  It is `9,603`,
  and the count without it is `200,604`.
- M3 asked for a reported and a left line for each predicate.
  Each one has both,
  and the edge-free clause's test reads every break of the clause.
- M4 asked that the decisions be separable.
  They are nine questions now,
  and "The questions,
  and which answers fix which" says what each answer fixes.
- M5 asked for the ranking criterion.
  It is stated first,
  with its three defects named,
  and build effort,
  files touched and fix needs are a cost section.
- M6 asked how a new class gets a severity of its own.
  Question 4 offers a rule id of its own and the shared severity,
  with their effects on today's error findings,
  the notices block and a shrinking allow list.
- M7 asked for a measured width and questions that stand alone.
  The width is measured (`123`,
  `127`,
  `131`),
  the questions spell out their options,
  and the conjunction list is given.
- M8 asked for the sweeps that merged during the hold.
  Question 1 names both sweeps with their commits and counts,
  and the handover citation is lines 2749 and 2750 at the survey commit,
  and lines 2865 and 2866 after the merge.
- M9 asked for "forced" relative to the kinds.
  The forced count is given per class,
  and "What \"forced\" says and does not say" states what it holds under and which predicates clear it.
- M10 asked for the issue's own examples to be counted.
  The one-or-two-word lines ending at a break point are counted (`393,916`)
  beside those ending at none (`39,485`).
- M11 asked for a split by file family.
  Each predicate's family rows are in `output.txt`,
  and the population's split is stated.
- M12 asked for quoted lines classed by their prose and for the missing controls.
  A quoted line is classed after its marker,
  three fixtures assert three splits,
  and the positive control sees the block-marker class.
- M13 asked whether the reported paragraphs are a lower bound.
  They are measured after the shipped fix ran on copies,
  `10,028` of the `11,552` remain,
  and the figure is stated as an upper bound on what remains and a floor on what joins.
- L1 asked whether an unterminated fence is reported.
  No rule reports it,
  and the answer names its commit and the README passage.
- L2 asked for the ways a line dodges a predicate.
  They are named in "How a line dodges each predicate".
- L3 asked for a pinned rerun and a snippet that compiles.
  The README names the commit and the worktree,
  the module text is the compiled `//` form,
  and `command.txt` and the README agree on three temporary changes.
- L4 asked for figures that trace.
  Every figure the survey does not count has its command and output beside it,
  and the two records of other rounds are named as theirs.
- L5 asked that the tree's state rest on evidence.
  The sentence claims only that package,
  with its run kept.
- L6 asked for paired figures in one unit.
  Paired figures are in one unit,
  and the clauses holding the forced breaks are counted.
- L7 asked where the README sentence holds.
  It holds at the rerun commit,
  and its change belongs to the round that is changing the fix.

The first read's three findings it called answered in part:

- H4 asked for a shrinking allow list and a rule id of its own.
  Both are weighed in question 4.
- Its M1 asked for the reported paragraphs to be measured.
  They are,
  as M13 above.
- Its M7 asked whether any rule holds prose under `120`.
  None does.
  The width is `123`,
  `127`,
  `131`.

## Answers to the third gate read

The third read's scratch expires,
so each finding is restated before the answer.
Every one was checked against the source,
the records and the kept measurements,
and every one held:
this revision changed the proposal for each of the three High and eight Medium findings,
and corrected the text or the record for each of the ten Low ones.
None was dismissed.

- `H1`:
  the ranking's criterion is the author's proxy,
  the order flips under the proposal's own kinds answer,
  and the pair reasons mix units.
  Checked by recomputing the shares from `output.txt`.
  Changed:
  the criterion is named as a proxy with three stated defects,
  the order under each answer is printed with its fractions,
  the "both counts" sentence is replaced,
  and every adjacent pair now gives its reason in one unit.
- `H2`:
  decisions 1 and 2 are not separable,
  the hold gates the rest,
  and the combinations that cannot be built are not stated.
  Checked against the harness's `D2` rule (it drops whole clauses) and question 1's fourth answer.
  Changed:
  `D2` is no longer a predicate,
  question 2 states both readings of a permitted kind,
  the hold is question 1,
  and "The questions,
  and which answers fix which" lists what fixes what and what cannot be built.
- `H3`:
  MD8 is made to name a parenthesis and a dash,
  and WR2 is never cited.
  Checked by reading `AGENTS.md` at the version the read pinned.
  Changed:
  MD1,
  MD8 and WR2 are quoted whole,
  the three-edge kind is called the author's proposal,
  and the dash option says it needs WR2 changed.
- `M1`:
  "forced" is counted per break and called "no author action",
  which is false for the
  shape predicates,
  and the removal sentence says "every break".
  Checked by arithmetic (`58,959 / 47,811 = 1.233`) and by rewrapping the proposal's own example.
  Changed:
  "What \"forced\" says and does not say" states the per-clause bound,
  the clearing per predicate is stated,
  and the removal sentence now names `162,707` and `47,500`.
  Not changed,
  because it cannot be counted from the record:
  the exact number of breaks a forced
  clause must keep,
  which needs the harness's per-clause widths.
- `M2`:
  question 4 presents a merged,
  veto-open adoption as an open choice and does not say what
  a veto does;
  the forced split rests on the virtual line.
  Checked against `second-pass-reads-of-released-versions.md` lines 336 to 340 and the harness.
  Changed:
  question 7 is "adopted and veto open",
  names what a veto reverts,
  and gives each
  predicate's Rust doc share as the bound on the forced split.
- `M3`:
  question 3 hides its mechanism,
  its options and what a yes leaves.
  Changed:
  it is three questions now (the rule id,
  the severity,
  the fix's release),
  each with its options and its costs,
  and LN7 is named where a demotion would need it.
- `M4`:
  question 1 gives no example for some kinds,
  says nowhere what "permit" does,
  and drops `65` breaks from two sentences.
  Changed:
  "The kinds,
  and one quoted break of each" quotes one break of every kind,
  the dash included;
  question 2 states what "permit" changes and which reading the survey counted;
  and the `65` block-marker breaks are named in the kinds section and in question 2.
- `M5`:
  the option sets other than the predicates have no pros,
  cons or ranking,
  and questions 5 and 6 carry several decisions.
  Changed:
  every option set has its options,
  its costs and a ranking with a reason per adjacent
  pair,
  and the two questions are split into four.
- `M6`:
  the kept harness is not the harness that produced `output.txt`,
  the corrected control had
  never run,
  and the README's rerun is circular.
  Checked by diffing the kept harness at `bc5b4ca2ba` and at this branch (five hunks:
  three for the
  control,
  two for the host prefix) and by running the corrected control once on fixtures.
  Changed:
  `command.txt` and the survey `README.md` say which commit's harness produced `output.txt`
  and name the two source changes a rerun at that commit needs,
  `reported-paragraph-control.txt` holds this round's run of the corrected control,
  and the README's rerun steps name a worktree under `.claude/worktrees/` and the scope limits.
- `M7`:
  the examples illustrate the opposite of what they are shown for,
  and all come from
  `.agents/skills/`.
  Checked by measuring each example's class,
  clause width and forced flag.
  Changed:
  each predicate has one real example from `doc/` and one from a Rust doc comment,
  each with its class and its forced flag,
  and the example that was shown as ordinary is gone from that place.
- `M8`:
  three pieces of evidence are missing:
  the owner's own prose,
  the share of files an author
  meets,
  and the recorded `doc/decision` answer with the fixer's recorded defects.
  Changed:
  "What the owner's own prose does" holds the walker's count,
  the file shares are printed with their arithmetic,
  and "What the records say" carries the `1,785` answer and the three fixer behaviors,
  with the statement that this round did not find whether they are fixed.
- `L1`:
  two sentences claimed more than the record supports.
  Changed:
  the status paragraph says which of the three kinds each count is.
- `L2`:
  the decisions and the questions were numbered two ways.
  Changed:
  there is one list of nine questions and nothing else is numbered.
- `L3`:
  "after it" was given with an earlier date.
  Changed:
  question 7 says "committed 2026-11-28T03:02:49-05:00,
  merged after the survey commit".
- `L4`:
  the path sum did not add up,
  `output.txt` had unexplained lines,
  and one figure had no
  recorded command.
  Checked:
  `134` Rust files hold no doc-comment line,
  all are in `files.txt`,
  and one is among the `66` ignored,
  so `133` is explained exactly.
  Changed:
  the `133`,
  the `ADDITIVE` line and the `M13 batch` line each have a sentence,
  and the figures `README.md` gives the command that produced the `.jsonl` file.
- `L5`:
  the closer-ended breaks were called a boundary of their own and were in every predicate,
  and no question asked about them.
  Changed:
  they are named as a seventh kind in question 2,
  with the reason their exact count is not printed and therefore not offered as a costed option.
- `L6`:
  figures the preface promised were missing,
  and a pointer named a `C` the output does not have.
  Changed:
  the file shares are printed for every predicate,
  the two shares the harness does not count are named as uncounted,
  and the `C` pointer is gone.
- `L7`:
  the README's rerun steps broke rule WTU and omitted the limits `command.txt` states.
  Changed:
  the README's worktree is under `.claude/worktrees/`,
  and its run command carries the same scope and limits `command.txt` records.
- `L8`:
  the subtraction's soundness argument is false for one class at the survey commit.
  Checked against `ends_at_break_point` at `bc5b4ca2ba`.
  Changed:
  the `10,028` is an upper bound on the remaining breaks and the `1,524` a floor on the
  joins,
  the mechanism is stated,
  and the survey README's sentence is corrected.
- `L9`:
  three figures for one package are not reconciled.
  Changed:
  question 9 names each population:
  `4,010` quiet short lines at no boundary in the package,
  `3,367` one- and two-word doc lines that prose follows with room under `120` after the sweep,
  and the `4,079` of issue 6642's own comment.
- `L10`:
  the clause predicate's "no hard break" test is a no-op.
  Checked in the harness:
  a hard line is skipped before the clause record is touched.
  Changed:
  "What \"at no boundary\" means" says the condition is vacuous at zero hard breaks.
  Testing every line of the clause instead would need the harness rerun.

## Figures and where each command and its output are kept

Every figure above is a row of the survey folder's `output.txt`,
`package/linter/monochromatic-lint/doc/a-survey-of-line-breaks-at-no-semantic-boundary/`,
except these,
whose command and output are in that folder's `figures-the-survey-does-not-count/`:

- The `26` findings in `20` files on `package/git-policy/cli`
  are in `repository-configuration-on-package-git-policy-cli.jsonl`
  beside that folder.
- The host widths `123`,
`127` and `131` are in `host-width-of-a-joined-doc-line.txt`.
- The absent width rule and the passing 123-character line are in
  `no-rule-bounds-a-prose-line.txt`.
- The exit statuses and the fix at each severity are in
  `exit-status-and-fix-at-each-severity.txt`.
- The emphasis boundary test is in `emphasis-boundary-reproduced.txt`.
- The corrected after-the-fix control,
  run once on fixtures,
  is in `reported-paragraph-control.txt`.
- The sweeps',
the repair's and the fence's commit records are in
  `96c930201f-the-revert-and-fix-repair.txt`,
  `81650758c4-what-the-repair-reached.txt`,
  `beb61ec46b-the-sweep-merge.txt`,
  `a287f226a0-the-merge-that-brought-the-repair.txt` and
  `7cfdc9199e-no-rule-reports-an-unterminated-fence.txt`.
- The quoted passages of
  `doc/handover/cli-git-rust-implementation.md` and
  `doc/planning/repository-linter-stays-at-zero.md` are in
  `cli-git-rust-implementation-the-4-062-passage.txt`,
  `repository-linter-stays-at-zero-the-repository-wide-run-and-the-58-of-498.txt` and
  `repository-linter-stays-at-zero-the-hold.txt`.
