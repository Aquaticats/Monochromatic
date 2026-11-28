# A line break at no semantic boundary: a finding, or not

Status:
this page is unapproved and non-final.
It was written 2026-11-27 by the round that fixed issue 6642,
and revised 2026-11-28T08:09:21-05:00 by the round that answered the fourth gate read.
Nothing here is built,
the rule reports exactly what it reported before,
and the owner declares completion.

It has had four same-vendor gate reads,
none of them an independent review:
the first found 16 findings (5 High,
8 Medium,
3 Low),
the second 22 (2 High,
13 Medium,
7 Low),
the third 21 (3 High,
8 Medium,
10 Low),
and the fourth 25 (4 High,
11 Medium,
10 Low).
The fourth read's findings and what answered each are in that round's report,
not here,
because a page the owner reads is for the decisions and not for the history of its own drafts.

## What is asked

Fourteen decisions,
not one.
Five of them are about the kinds,
so they are numbered 2a to 2e.
Each decision can be answered on its own,
in this order:

- Decision 1,
  the hold:
  still governs,
  governs today's findings only,
  or lifted.
- Decision 2a:
  may a break whose next line opens a code span,
  an emphasis span,
  a link or an image go unreported.
- Decision 2b:
  may a break whose next line opens a parenthesis go unreported.
- Decision 2c:
  may a break whose next line opens one of 31 listed conjunctions or subordinators go unreported.
- Decision 2d:
  may a break whose line before it ends in a dash go unreported.
- Decision 2e:
  may a break whose line ends in a closing bracket or quote go unreported.
- Decision 3,
  the bound a join uses:
  the host line's,
  the line the rule sees,
  or none.
- Decision 4,
  which of five predicates reports the class.
- Decision 5,
  what mechanism repairs the breaks already in the tree.
- Decision 6,
  which release first reports the class.
- Decision 7,
  whether a repair fix would run under the repository's format task.
- Decision 8,
  which files the first repair round covers,
  if it would not.
- Decision 9,
  whether the class has a rule id of its own or shares the existing one.
- Decision 10,
  at which severity the class first reports.

The recommendation,
as the set it is,
with what each item rests on:

- Decision 1:
  the hold governs today's findings only,
  and not a new class.
  `[read]`
- Decision 2a:
  a no;
  the break before a span stays reported.
  `[measured counts, read rule text]`
- Decision 2b:
  a yes;
  a break before a parenthetical goes unreported.
  `[measured counts, walker]`
- Decision 2c:
  a yes;
  a break before a listed conjunction goes unreported.
  `[measured counts, walker]`
- Decision 2d:
  a no;
  a break after a dash stays reported.
  `[read rule text, measured count]`
- Decision 2e:
  a no;
  a closer-ended break stays reported.
  `[measured, and one figure not counted]`
- Decision 3:
  keep the host line's bound,
  and no veto.
  `[measured widths, read rule text]`
- Decision 4:
  the joinable break,
  `B`.
  `[measured]`
- Decision 5:
  a removal fix built on the shipped join's guards.
  `[inferred: the fix is a model]`
- Decision 6:
  report first,
  and ship the fix in a later release.
  `[inferred]`
- Decision 7:
  no;
  keep the fix out of the format task.
  `[read task text]`
- Decision 8:
  the whole tree,
  in one round.
  `[measured, in the unit of decision 4's predicate]`
- Decision 9:
  a rule id of its own.
  `[read configuration, one severity per id]`
- Decision 10:
  `warn` first.
  `[measured exit statuses]`

The three marks:
`[measured]` is a row of the survey's `output.txt` or a figure whose command and output are kept beside it,
`[read]` is a sentence of a record or a rule quoted with its path,
and `[inferred]` is a claim no measurement here settles.

What a yes to the whole set would do,
in one sentence:
the rule would gain a second code at `warn`,
which reports the `124,259` breaks a join's guards allow in `8,185` files
and reports nothing for the `29,166` breaks those guards refuse,
a removal fix would be built for it outside the format task,
and the first round would run that fix over `package/git-policy/cli`.

The one fact that frames all fourteen is that the rule reports nothing for any of these
breaks today.
The `210,207` breaks below sit in paragraphs the rule reports nothing for,
so no finding of the class exists,
and a yes adds findings to text that passes today.
Every one of them is in the count the survey measured,
and no command in the tree clears one.

What the owner may skip:
if decision 1 answers that the hold still governs this rule,
then decisions 2a to 10 cannot take effect,
and the rest of this page is a plan for later.

What is already adopted,
and is decided here only as a veto:
the join counts the host line,
a change released in `0.5.2-alpha.25` (decision 3).

## What the numbers are

Every figure below is one of three things,
and the sentence that uses it says which:

- a row of the survey's `output.txt`,
  at the commit named under "The measurement";
- a figure the survey does not count,
  whose command and its whole output are kept in
  `package/linter/monochromatic-lint/doc/a-survey-of-line-breaks-at-no-semantic-boundary/figures-the-proposal-derives/`;
- another record's own measurement,
  named as that record's and not rerun here.

The counts are the survey commit's,
`bc5b4ca2ba`,
and they are the counts of `monochromatic-lint 0.5.2-alpha.24`,
the release installed when the survey ran.
`0.5.2-alpha.25` (release commit `6f52a414fc`,
2026-11-28T09:43:57Z) carries one change that moves some of them,
which "What the tip changes,
and what it does not" states.

## Why the question is open

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
The survey below classes every break by the first token of the next line's prose,
so the owner can say which kinds MD1 permits.
This proposal does not answer that question,
and the kinds it offers are the author's proposal,
not a list MD1 or MD8 holds.

No rule of the 29 this linter ships bounds a prose line's width:
`monochromatic-lint --rules` lists 29 ids,
and the kept record
`figures-the-survey-does-not-count/no-rule-bounds-a-prose-line.txt`
names each and the width a run reached.
A 123-character prose line with no break point inside it passes the repository's
own configuration with exit 0,
and only `markdown/semantic-line-breaks`,
which reads where to insert a break,
reads a prose line at all.
The rules named `max-lines` in the Rust and shell sets bound a file's line count.
So MD1's 120 is a bound nobody enforces,
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
  so one condition of the clause predicate tests nothing.
- A line whose last visible character is a bracket or a quote
  is counted apart as a **closer-ended** break,
  because a reader may read such a line as ending at a boundary of its own.
  The survey still counts it at no boundary,
  and every predicate below reports it:
  decision 2e says what a yes would take and which figure is not counted.
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
- A line ending `e.g.`,
  `etc.` or `...` is treated as a boundary here,
  although `break_offsets` never breaks after an abbreviation or an ellipsis.
  The survey undercounts breaks at no boundary by that small class.

A continuation line of a quotation keeps the `>` its container writes.
The survey strips the marker before it classes the line,
counts its words or measures its clause width,
so a quoted line is filed by its prose and not by the marker a reader does not see.
The run classes `5,602` breaks whose next line opens a quote marker this way,
and `65` breaks remain in the `block-marker` class.
That class is a first-character test:
a line whose first character is `>`,
`#` or `|`,
or which opens a fence or a list marker,
is `block-marker`.
It is not a statement that the shipped join refuses the break:
of the `65`,
the join removes `38`.
The example quoted under "The kinds" is one of the `27` it does not remove.## The measurement,
and what it counts

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
None of those `210,207` is a finding today,
because the rule reports nothing in the paragraphs they sit in.

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
  A scale for how many lines could hold it:
  the survey counts `3,423` breaks whose line ends inside emphasis in the whole population,
  `3,383` of them in the quiet one.
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
  a rewrap usually clears the finding even though the clause stays forced,
  but not always:
  "What clears a finding" counts the clauses that no rewrap clears.
- All forced counts are widths of the line the rule sees.
  A Rust doc comment's host line is longer by the item's indent and the comment marker,
  so a bound on the host line would move some clauses into the forced set;
  decision 3 says how far.

The class issue 6642 names is a line of one or two words that itself ends at no
boundary.
There are `39,485` of them in `4,728` files.
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
Those are the counts under decision 2's answers of "no" to all five kinds;
a permitted kind takes its own breaks out of every file count,
and `output.txt` prints no per-kind file count,
so a file count under another answer to decision 2 is not known.

## What the tip changes, and what it does not

The survey commit is before `fc679b9392`
(committed 2026-11-28T03:02:49-05:00,
released in `0.5.2-alpha.25`),
which does two things:

- The join counts the host line,
  its comment marker and host indent included.
  Decision 3 asks for a veto of that.
- The boundary test climbs `*`,
  `_` and `~`,
  so a break whose line ends inside emphasis is no longer counted at no boundary,
  and the emphasis class is empty at the tip.

`output.txt` prints a durable row for three predicates,
which is the predicate with the emphasis endings taken out:
`A` `206,824` in `8,717` files,
`B` `159,710` in `8,164` files,
and `F2` `38,322` in `4,700` files.
It prints none for `D` or `F1`,
so neither of their counts can be moved to the tip here.
What that does to the one figure the predicate ranking uses,
and to the removable counts decision 5 rests on,
is said at those places.
A rerun of the whole survey is not part of this proposal.

## The kinds, and one quoted break of each

Decision 2 asks which kinds MD1 permits.
Each kind is one row of the class table,
each is shown by a real break at the survey commit,
and each example is the kind it is shown for.
The quoted text is the file's own,
read by
`figures-the-proposal-derives/the-examples-checked-against-their-files.py`,
whose output is `the-examples-checked-against-their-files.output.txt` beside it,
so it is not retyped here.

- `block-marker` (`65` breaks):
  a line whose first character is `>`,
  `#` or `|`,
  or which opens a fence or a list marker.
  From `doc/audit/tech-mold-linker-vet-2026-10-05.md:786`,
  a line opening `#1696`,
  which the shipped join does not refuse:
  that branch of `starts_block_construct` needs a `#` followed by a space,
  a tab or the end of the text.

  ```text
    #1695 closed unmerged with follow-up issue
    #1696 open (thread-pool tuning discussion);
  ```

- `parenthesis` (`37,033`):
  the next line opens with `(`.
  From `.agents/skills/choosing-technology/SKILL.md:151`,
  clause `128` characters,
  forced:

  ```text
  because the user may delegate the enumeration to a cheaper model of their choosing
  (not necessarily Haiku)
  ```

- `inline-span` (`25,355`):
  the next line opens with a backtick,
  a `*`,
  an `_`,
  a `~`,
  a `[` or a `!`.
  From `.agents/skills/android-emulator/SKILL.md:22`,
  clause `162` characters,
  forced:

  ```text
   Briefs for agents that use an emulator repeat the headful rule and name the launch form in
   `doc/troubleshooting/android-emulator-37-software-renderer-sigsegv.md`;
  ```

- `conjunction` (`19,749`):
  the next line opens with one of the 31 words listed below.
  From `.agents/skills/css/SKILL.md:170`,
  clause `83` characters,
  not forced:

  ```text
   user-configurable properties
  where the fallback represents the default user choice.
  ```

- `dash` (`20`):
  the line before the break ends in `--`,
  `—` or `–`.
  From `.agents/skills/project-code-review/PHILOSOPHY.md:24`,
  clause `211` characters,
  forced:

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
which is why the list is here and why decision 2c asks about it by name.

A seventh kind is not a row of that table:
the **closer-ended** break,
whose line ends with a bracket or a quote (`9,603` breaks).
It is orthogonal to the six:
a break can be closer-ended and conjunction-class at once,
so `output.txt` prints its total without a split among the classes,
and what a yes to decision 2e would remove from each predicate is not counted.
It is offered as a decision anyway,
because every predicate reports it and leaving it out of the questions
was the fourth read's `L7`.## What each predicate would report

Five predicates report,
and a sixth answer reports nothing.
Each is counted in its own population,
in clauses where it reports clauses and in breaks where it reports breaks.
`output.txt` labels the first five `A`,
`B`,
`D`,
`F1` and `F2`.

Every predicate takes decision 2's answers as a filter:
a kind the owner permits leaves every predicate.
So each predicate has two counts below,
under "no" to all five kinds and under the recommended set
(2b and 2c answered yes,
the other three no),
and the per-kind table in decision 4 gives every other set.

The examples are real files at the survey commit,
one from `doc/` and one from a Rust doc comment for each predicate that reports,
with the class,
the clause width and the forced flag named
so that a reader can see which other predicate leaves the same break.
Each file's paragraph was checked to carry no finding of the shipped rule,
by a run of the installed binary recorded for this round.

### Report every break at no boundary (`A`)

`210,207` findings in `8,721` files,
or `153,425` under the recommended set,
whose file count is not counted.

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

A yes here leaves undecided which clause of a forced pair is the finding's position,
and that `A` at `error` can never reach zero.

### Report only a break a removal can join (`B`)

`162,707` findings in `8,185` files,
or `124,259` under the recommended set,
whose file count is not counted:
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

The one guard the survey lifts is `FRAGMENT_WORDS`,
which the shipped join sets to two words;
every other guard is the shipped one.
So `B` is not a count of what the shipped join does today,
and a yes here leaves undecided that the fix does not exist.

### Report an unforced break at clause level (`D`)

One finding per clause under `120` characters
with at least one break at no boundary,
no hard break,
and no emphasis ending,
in a paragraph the rule reports nothing for.
`144,863` findings in `8,085` files,
covering `148,203` breaks,
or `111,457` under the recommended set.
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
A yes here leaves both undecided.

### Report a break that leaves a one-or-two-word next line (`F1`)

`73,414` findings in `7,072` files,
or `52,338` under the recommended set,
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

A yes here leaves undecided how many of its `14,694` forced findings a rewrap clears,
which "What clears a finding" counts.

### Report a one-or-two-word line that ends at no boundary (`F2`)

The class issue 6642 names.
`39,485` findings in `4,728` files,
or `33,883` under the recommended set,
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

A yes here leaves undecided that a rewrap clears `5,613` of its findings,
and that the `1,163` emphasis endings are the rule's own inserted break.

### Report nothing

No finding,
no churn,
and the `39,485` short lines stay.

A yes here leaves nothing undecided about this rule,
and answers the whole proposal:
the class is not reported.

## A finding as an author meets it

The rule prints one JSON object per finding,
and the class would print one in that form.
This is a real run of the installed binary (`0.5.2-alpha.25`)
over two fixtures,
with the survey's one-rule configuration
and the command and its whole output kept in
`figures-the-proposal-derives/a-finding-as-an-author-meets-it.txt`:

```text
{"message":"A line break belongs here, after a prose break-point character.","code":"markdown/semantic-line-breaks","severity":"error","causes":[],"filename":"a-lone-short-word-at-no-boundary.md","labels":[{"span":{"offset":8,"length":0,"line":1,"column":9}}],"related":[]}
```

That fixture's text before the run was:

```text
Aaa bbb, ccc ddd eee
the
fff ggg hhh.
```

The finding is at line 1,
column 9:
the rule wants a break after the comma,
which is a finding of the rule as it stands.
The second line,
`the`,
ends at no boundary and no predicate of the rule reports it:
that is the class,
and it is the shape the rule's own fix writes.
`--fix` on that file leaves:

```text
Aaa bbb,
ccc ddd eee
the
fff ggg hhh.
```

On the other fixture,
`Aaa bbb, the` / `ccc ddd eee.`,
`--fix` writes:

```text
Aaa bbb,
the ccc ddd eee.
```

So the rule's own fix produces the shape issue 6642 names,
and the class is what would report it.

For each predicate,
on the first fixture above,
what a yes would print:

- `A`:
  one finding at line 2,
  column 1,
  the whole line,
  at the severity decision 10 answers.
- `B`:
  the same finding,
  because the joined line `the fff ggg hhh.` is under `120`.
- `D`:
  one finding for the clause `Aaa bbb, ccc ddd eee the fff ggg hhh.`,
  anchored at its first break (line 1,
  column 9) or at the clause,
  which decision 4 says is undesigned.
  Its message is undesigned too,
  and no rule id can carry it before one exists.
- `F1`:
  no finding,
  because the next line `fff ggg hhh.` holds three words.
- `F2`:
  one finding at line 2,
  column 3,
  the end of `the`.
- `nothing`:
  no finding,
  which is what the rule does today.

The fixed text under each:
`A`,
`B`,
`D` and `F2` all want the same join,
`Aaa bbb, ccc ddd eee the fff ggg hhh.` on one line,
and `F1` and `nothing` want nothing.
That join is what a removal fix would write,
and no such fix exists.

The predicates' own output is a design,
not a capture:
no rule id,
message,
or anchor exists for any of them,
so the shapes above say where a finding would land and not what a shipped program prints.

## What clears a finding

A rewrap of the clause is the one repair a person can make without a command,
and it does not clear every finding:

- `A`:
  a break in a forced clause is ordinary or of a kind the owner reports,
  and no rewrap clears it,
  because the clause cannot go on one line and has no punctuation to move the break to.
  Under "no" to all five kinds that is at least `47,811` findings and at most `58,959`.
- `B`:
  every finding is one the modeled join clears,
  so `162,707` of `162,707` under "no" to all five kinds,
  and `124,259` of `124,259` under the recommended set.
  That is by construction and not a measurement of a shipped command.
- `D`:
  a rewrap that puts the clause on one line clears it,
  because every one of its clauses is under `120`.
  So all `144,863` are clearable by hand and none by a command.
- `F1` and `F2`:
  a rewrap usually clears the finding,
  but not always.
  A forced clause of three words or fewer cannot be rewrapped to clear either:
  every line of such a clause holds one or two words and ends at no boundary,
  so every line is a finding of both.
  Counted over the `163` Markdown files of `doc/audit` at the survey commit by
  `figures-the-proposal-derives/forced-clauses-a-rewrap-cannot-clear.py`:
  `8,234` clauses of two or more lines,
  `2,331` of them forced,
  `21` forced clauses of three words or fewer,
  and `1` of the `21` whose every break position is ordinary-class.
  That one is
  `doc/audit/dry.md` from line `2151`,
  a clause of `135` characters in three words,
  and no answer to decision 2 clears it:
  a rewrap can only leave a finding of `F1` or of `F2`.
  The other `20` have a position of a class some answer to decision 2 permits,
  so a yes to that kind clears them.
  How many of the `14,694` and `5,613` such clauses there are in the whole tree
  is not counted:
  the per-clause widths are not in `output.txt`,
  and this proposal does not rerun the survey.## What the records say,
  and what still stands

`doc/planning/repository-linter-stays-at-zero.md` records,
at lines 14 and 16 of the rerun commit:

- A repository-wide run at `136a40fe69` (2026-11-07T11:36:20-05:00) reported `174,916` findings,
  of which this rule made up `171,262`.

That is the last recorded count of the rule's own findings.
No later record counts them,
and this proposal does not run the linter over the repository,
so **how many findings the rule reports today is not known.**
It is below `171,262`,
because the run is before every sweep listed next,
but by how much is not measured.
The `210,207` breaks this proposal is about are not among them:
they sit in paragraphs the rule reports nothing for.

The same record quotes the owner's words,
moved verbatim on 2026-11-23,
at lines 40 to 43:

```text
"Put the semantic line break findings on hold."
then,
"Put the semantic line break findings on hold.
They aren't the most serious issues right now."
```

The words were spoken on 2026-11-04 and 2026-11-05,
as the old handover says where it holds them
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`,
"Owner words the dropped state subsections held").

The sweeps of this rule's fix that the records hold,
with the commit's own date:

- 2026-11-07T12:26:57-05:00 `e3174c31b0`,
  the linter's whitespace fixes for the documentation families outside `doc/decision`.
- 2026-11-07T12:33:53-05:00 `09d670555d`,
  the linter's semantic line breaks in four rules of `AGENTS.md`,
  whitespace only.
- 2026-11-07T12:33:59-05:00 `fce04655a0`,
  those fixes for every package outside the open branches,
  `AGENTS.md`,
  `README.md` and the skills.
- 2026-11-07T14:16:16-05:00 `aa559080b7`,
  the fixes for Markdown and rustdoc in every package outside the open branches.
- 2026-11-07T14:30:00-05:00 `46a85668cb` and 2026-11-07T14:38:23-05:00 `cc34dc6faf`,
  two later merges of the same branch.
- 2026-11-23T21:43:12-05:00 `23cb42e24a`,
  the whitespace-only sweep of 2026-11-23,
  with its own note at 2026-11-23T21:45:28-05:00 `c2701d58a5`.
- 2026-11-24T01:19:33-05:00 `96fe5ec804`,
  01:22:58 `2b748f3748`,
  01:23:27 `fcaa7b840b`,
  01:24:19 `9ad03b7693` and 02:31:50 `f12e93fa30`,
  the line breaks of the files the sweep above left out.
- 2026-11-27T18:15:47-05:00 `ee8871b190`,
  `76` findings in `package/mpm-shadow/cli`;
  18:16:07 `f6b4dc0947`,
  `11` in `package/cargo-shadow/cli`;
  18:22:37 `e6026dbd46`,
  `778` in five Markdown files of `package/pi-shared/subagent-driver`;
  18:38:01 `beb61ec46b`,
  `4,062` findings in `480` files of `package/git-policy/cli`.
- 2026-11-27T22:35:41-05:00 `96c930201f`,
  which repaired `771` doc paragraphs in `367` files
  by running the installed fix over the pre-sweep copies,
  netting `90` fewer doc lines.

Those five merges of 2026-11-07 are two days after the hold was spoken,
and they are the largest,
because they are repository-wide.
The list is the sweeps the records name,
not every commit that changed a wrap:
a subject search with the regex in `figures-the-proposal-derives/the-sweeps-and-the-hold-in-the-records.txt`
finds `325` commits since 2026-11-05 whose subject names line breaks,
which is a search and not a count of sweeps,
so whether each of those was a sweep the hold covers is the owner's to say.

At line 186 the same record says the fixer
"changed rendered text in 58 of the 498 files it changed in the documentation families",
which is why a blind sweep of quoted records is not safe.

Two more passages of that record bear on the decisions below,
and neither is the owner's word:

- At lines 254 to 259,
  moved from the old handover,
  it counts what a sweep of `doc/decision` would meet:
  "`doc/decision` 1,785 (answered on 2026-11-10:
  form-only prose fixes,
  and moved-path notes only);".
  Those `1,785` are findings of the rule as it stands,
  not breaks of the quiet population,
  and the record does not say who answered.
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

`doc/handover/cli-git-rust-implementation.md` counts the `4,062` sweep,
at line 2749 of that file at the survey commit,
and at line 3410 of that file at `b1529e79e1`,
the commit this revision was written at:

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
(lines 258 to 260),
and the fixer's earlier rewrite of rendered text remains the reason
a blind sweep of quoted records is not safe.

### What runs the linter, and at which severity

- `git commit` through the wrapper runs no check of this rule:
  `monochromatic-git-policy-cli.config.jsonc` line 10 runs
  `"markdown/autofix": ["warn", { "rules": ["lfs-image-url"], ... }]`,
  and that rule list is the policy's default,
  stated there.
- `mise run lint:monochromatic-lint` runs the linter over the paths it is given,
  or the whole tree when it is given none,
  and `mise.toml` lines 1024 to 1025 say it
  "Exits 1 on error findings and 2 when the run could not be completed;
  warnings alone exit 0."
  That is a person's or a script's command,
  and it fails that command.
- `.github/workflows/advisory-core.yml` line 213 runs that task over the changed `.md`,
  `.mdx` and `.rs` files of a pull request,
  and the step is `continue-on-error: true` (line 192),
  so it reports and does not fail the job.
- The installed `0.5.2-alpha.25` on `package/git-policy/cli`
  reports `26` findings in `20` files and exits `1`:
  `14` `rust/max-lines` errors,
  `11` `rust/folder-uses-links` errors,
  and one `rust/avoidable-closure` warning.
  It reports no finding of this rule there.
  The command and its whole output are kept beside the survey.

So "returns to green" has no green baseline for this rule,
and `error` fails a run a person or a script starts,
not a commit.
At `error` a run without `--fix` exits `1`;
at `warn` the same run exits `0`.
Both severities apply `--fix` when it is passed.

`mise run format` runs `monochromatic-lint --fix` over `.`
(`mise.toml` lines 1184 to 1188 and 1210 to 1215),
with no paths,
which is decision 7's subject.
`markdown/semantic-line-breaks` has `fixClass: whitespace`
(`monochromatic-lint --rules`),
and the linter applies a whitespace-class fix under plain `--fix`.

## What the rule files' own prose shows

`AGENTS.md` is the repository's own rule file,
and it is the only long corpus whose wrapping this round can read in a form that predates the class.
It is not evidence of the owner's hand:
`555` commits touch it,
`100` of them carry a `Co-Authored-By` trailer naming Claude models or a bot,
and rule GAP tells agents to edit it.
So the wrapping below is the file's,
written by whoever made those commits,
and this proposal does not attribute it to the owner.

Part of it is the linter's own output:
`09d670555d` rewrote four of its rules with that fixer,
and `fce04655a0` landed the fixer's changes on it.
A break the fixer could have written is one after `,` `.` `;` `:` `?` or `!`,
because those six are the only characters it breaks after;
a break before `and` or before `(` is not its work.
The two classes the walker finds most of are exactly those two.

A line walker over that file at the survey commit,
kept as `figures-the-proposal-derives/the-rule-files-prose-walked.py`
with its output beside it,
finds `1,030` breaks inside paragraphs and `21` at no boundary:
`13` before a conjunction,
`7` before a parenthesis,
`1` in the ordinary class,
and none in the other three.
No line of the file ends in a dash.
The walker reads no syntax tree and its paragraph model is its own,
so the `1,030` is indicative:
two earlier walkers with different paragraph models read `1,312` and `1,325` breaks in the same file.
All three agree on the `21` and on how they divide.

What that supports:
the file's own wrapping breaks before a conjunction and before a parenthesis,
`20` times of `21`,
and never before a span and never after a dash.
What it does not support:
a claim that those are the only kinds the owner permits,
or a claim about how many breaks of each kind a repository-wide answer should permit.
The kinds' counts in the tree are the class table's,
and they are not proportionate to this file's `21`.## The decisions,
in the order they can be answered

Each decision says what earlier answers fixed for it,
what a yes would change for good,
what it gives up,
and what a yes leaves undecided.
No decision ties an answer to an edit of the owner's rule text:
where a rule's words would have to change for the answer to be recorded,
the decision says so beside the answer,
and the edit is the owner's to make or not.

### 1. Does the hold still govern this rule

What it counts.
The hold is on working this rule's findings,
and the rule reports them today whether or not it stands:
the rule is at `error`,
so lifting the hold changes who may clear findings and not what the rule reports.
The count it holds back is the rule's own findings,
last recorded as `171,262` at `136a40fe69`,
before the sweeps listed under "What the records say".
How many there are today is not known,
and the `210,207` breaks this proposal is about are not among them.
Decision 1 gates every other decision.

The options:

#### It still governs this rule
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
and makes decisions 2a to 10 a plan for later.

#### It governs today's findings only, and not a new class
The rule's own findings keep the hold,
and a class that decision 4 picks may ship and report its own population.
What it changes for good:
a second code would report the class,
so the hold's words still hold for the findings they were spoken about.
What it gives up:
two codes to keep in step,
and today's findings keep the queue the hold left them in.
A yes here leaves undecided whether the sweeps of 2026-11-07 and later were within the hold
(the records do not say),
and what the class's first report meets in the files those sweeps touched.

#### It is lifted
Today's findings and any new class report at whatever severity decision 10 answers.
What it changes for good:
the rule's own findings become work again,
and whatever number stands today is the queue.
What it gives up:
a queue of that size with no fix in the tree,
against the owner's recorded principle that lint issues are taken care of asap.
A yes here leaves undecided what clears them (decision 5),
and whether the sweeps count as having pre-empted the hold.

Ranking:
the second > the first > the third,
because:

- The second over the first:
  the second lets decisions 2a to 10 take effect,
  and the first delivers nothing the tree does not already have.
- The first over the third:
  the third reopens a queue of findings no command in the tree clears,
  and the first keeps that queue where the owner put it.

### 2. Which kinds does MD1 permit

Five yes-or-no decisions,
one per kind,
in any combination.
They are not a chain:
an owner may permit the conjunction kind and not the parenthesis,
or the reverse.

Each of them changes one thing in the linter:
a break of a permitted kind is not reported by the class,
so it leaves every predicate.
It changes nothing else,
and in particular it does not edit `AGENTS.md`.
A permitted kind widens what a maintainer may write without a finding,
and narrows what the class can tell an author about.

The counts each answer takes out of each predicate,
from `output.txt`'s per-class rows,
recomputed by
`figures-the-proposal-derives/figures-for-each-kinds-answer.py`:

```text
kind              A          B          D         F1         F2
block-marker         65         38         37         39         13
conjunction      19,749     12,392     10,537      1,703      2,706
dash                 20         17         14          6          2
inline-span      25,355     17,512     14,769     12,829      5,137
ordinary        127,985    106,692     96,637     39,464     28,731
parenthesis      37,033     26,056     22,869     19,373      2,896
TOTAL of the six 210,207    162,707    144,863     73,414     39,485
```

Each column adds to that predicate's own row in `output.txt`,
so an owner may choose any set of the six and read each predicate's count by subtraction.
The `ordinary` row is not offered:
it is the class the other five are defined against.
The `block-marker` row is not offered as a sixth decision:
`38` of its `65` breaks are ones the join removes,
and a kind that is neither a boundary nor a repair is not one this proposal puts to the owner.
A yes to a kind is a yes to that row leaving,
and nothing more.

### 2a. May a break whose next line opens a code span, an emphasis span, a link or an image go unreported

`25,355` breaks in `A`,
`17,512` in `B`,
`14,769` in `D`,
`12,829` in `F1`,
`5,137` in `F2`.

#### A yes: a break before a span stops being reported
Mechanism:
a break whose next line's first character is a backtick,
`*`,
`_`,
`~`,
`[` or `!` stops being reported.
What it changes for good:
a wrap before a span is permitted in prose.
What it gives up:
`25,355` wraps MD8 does not bless.
MD8's words are about where a hard wrap may fall,
and a break before a span is not one of the places MD8 names.
If the owner wanted it recorded,
MD1 would have to say "a line may end before a code span,
an emphasis span or a link".
A yes here leaves the rule text untouched.

#### A no: a break before a span is still reported
Mechanism:
those breaks stay findings.
What it changes for good:
a break before a span is still reported.
What it gives up:
`25,355` findings at a shape no line of `AGENTS.md` ends a line at.

Recommendation:
**no**.
The repository's own rule file breaks before a span `0` times,
in `1,030` breaks,
and MD8's "only between whole inline spans" is written about avoiding a split,
not about blessing one.
That is a reason from a read of the rule and from a measured `0`,
not a reason to report what the owner cannot see:
if the owner's own prose ever does it,
the finding is one the owner would want.

### 2b. May a break whose next line opens a parenthesis go unreported

`37,033` breaks in `A`,
`26,056` in `B`,
`22,869` in `D`,
`19,373` in `F1`,
`2,896` in `F2`.
That is the second-largest kind in the tree and the second most common shape in `AGENTS.md`.

#### A yes: a break before a parenthetical stops being reported
Mechanism:
a break whose next line opens with `(` stops being reported.
What it changes for good:
a wrap before a parenthetical is permitted.
What it gives up:
`37,033` findings,
and a parenthetical a reader would rather see on the previous line is never named.
If the owner wanted it recorded,
MD1 would have to say "before a parenthesis".
A yes here leaves the rule text untouched.

#### A no: a break before a parenthetical is still reported
Mechanism:
those breaks stay findings.
What it changes for good:
a break before a parenthetical is still reported.
What it gives up:
`37,033` findings at a shape the rule file uses `7` times of its `21` no-boundary breaks.

Recommendation:
**yes**.
The rule file breaks before a parenthesis `7` times of the `21` breaks at no boundary,
and the fixer could not have written those,
because it breaks only after its six characters.
So the file's own wrapping uses the shape,
and a class that reports it would report the repository's own rule text.

### 2c. May a break whose next line opens one of 31 listed conjunctions go unreported

`19,749` breaks in `A`,
`12,392` in `B`,
`10,537` in `D`,
`1,703` in `F1`,
`2,706` in `F2`.

#### A yes: a break before a listed conjunction stops being reported
Mechanism:
a break whose next line opens with one of the 31 words listed under "The kinds"
stops being reported.
What it changes for good:
a wrap before a clause-opening conjunction or subordinator is permitted,
and the list becomes part of what the linter knows,
not of the rule text.
What it gives up:
`19,749` findings,
and a line opening with `that` or `which`,
which many readers call a bad wrap,
is never named.
If the owner wanted it recorded,
MD1 would have to say "before one of a listed set of conjunctions and subordinators",
and that list would have to be maintained in `AGENTS.md` beside the linter's copy.
A yes here leaves the rule text untouched.

#### A no: a break before a listed conjunction is still reported
Mechanism:
those breaks stay findings.
What it changes for good:
a break before `and`,
`because` or `which` is still reported.
What it gives up:
`19,749` findings at the commonest no-boundary shape the rule file itself uses
(`13` of its `21`).

Recommendation:
**yes**.
`13` of the rule file's `21` no-boundary breaks are before a conjunction,
the largest shape it uses,
and the fixer cannot have written them for the same reason as 2b.
The 31-word list is the proposal's and not MD1's,
so a yes is a yes to that list;
`that`,
`which` and `as` are in it,
and an owner who wants a shorter list is asking a different question,
which this decision does not offer.

### 2d. May a break whose line before it ends in a dash go unreported

`20` breaks in `A`,
`17` in `B`,
`14` in `D`,
`6` in `F1`,
`2` in `F2`.

#### A yes: a break after a dash stops being reported
Mechanism:
a break whose previous line ends in `--`,
`—` or `–` stops being reported.
What it changes for good:
`20` findings go.
What it gives up:
a break after a character WR2 forbids in prose at all,
so the class would say nothing about a line the repository's own prose rule forbids.
WR2 would have to change for the answer to be recordable:
it would need a prose exception for a dash at a line's end.
A yes here leaves WR2 untouched and contradicted.

#### A no: a break after a dash is still reported
Mechanism:
those `20` breaks stay findings.
What it changes for good:
a line ending in a dash is reported,
and with it the character WR2 forbids.
What it gives up:
`20` findings,
which is the smallest kind by two orders of magnitude.

Recommendation:
**no**.
The rule file ends no line in a dash anywhere,
and WR2 forbids the character in prose.
A yes would hide findings about a construction the repository has already ruled out,
for `20` breaks.

### 2e. May a break whose line ends in a closing bracket or quote go unreported

`9,603` breaks in the quiet population,
and the per-predicate count is not counted:
the survey prints the total and not its split among the six classes,
because a break can be closer-ended and of another class at once.
So a yes here removes at most `9,603` from each predicate,
and the exact removal is unknown.

#### A yes: a break after a closer stops being reported
Mechanism:
a break whose line's last visible character is one of `" ' ) ] } ” ’ » ›`
stops being reported.
What it changes for good:
a reader who reads such a line as ending at a boundary of its own is not contradicted.
What it gives up:
every predicate loses an uncounted share of its findings,
and no figure in this proposal can say how much.
If the owner wanted it recorded,
MD1 would have to name a closing bracket or quote as a boundary.
A yes here leaves the rule text untouched.

#### A no: a break after a closer is still reported
Mechanism:
those breaks stay findings.
What it changes for good:
a break after an unmatched closer is still reported.
What it gives up:
`9,603` findings a reader may think are blessed.

Recommendation:
**no**,
and the reason is that nothing measured here separates a closer that closes a parenthetical
from one that closes a sentence fragment,
so a yes would be a yes to an uncounted share of every predicate's findings.
An owner who wants it counted needs a rerun that prints the joint table,
which this proposal does not offer.

### 3. Which bound does a join use

What earlier answers fixed:
nothing;
this is a veto of a change already released,
and it is the one decision that stands apart from the class.

The shipped join measures the line the rule sees:
the paragraph's continuation indent,
the fragment and the following line,
against `120`.
In a Rust doc comment the host line is longer by the item's indent and the comment marker.
A joined virtual line of `119` characters is a host line of
`123` characters at item indent 0,
`127` at indent 4,
and `131` at indent 8,
measured with the installed fix on copies of one fixture
(`figures-the-survey-does-not-count/host-width-of-a-joined-doc-line.txt`).

This decision is adopted and veto open.
`fc679b9392` (2026-11-28T03:02:49-05:00,
released in `0.5.2-alpha.25`)
made the join count the host line,
its comment marker and host indent included,
so a joined Rust doc line stays under `120` in the host file,
and it settles in one pass.
`package/linter/monochromatic-lint/doc/second-pass-reads-of-released-versions.md`
records the choice,
naming the cost of the way not taken:
"it costs the rule writing lines over the limit it enforces,
which the owner's own bound in `AGENTS.md` rule MD1 forbids",
and then says "The veto stays open."

The options:

#### Keep the host line's bound (the tree as it stands)
Mechanism:
the join measures the line as the file holds it,
marker and indent included.
What it changes for good:
nothing,
it is what the tree does,
and what `0.5.2-alpha.25` releases.
What it gives up:
a join is refused when the virtual line is under `120` but the host line is not.
A yes here leaves undecided nothing;
it is the state of the tree.

#### Veto it and return to the virtual line's bound
Mechanism:
revert `fc679b9392`,
correct the linter `README.md` sentence that says the host line is the limit,
and decide what the tests `the_host_prefix_is_part_of_the_width`,
`the_indent_the_fix_writes_is_part_of_the_width` and
`a_joined_doc_line_stays_under_the_limit_in_its_host_file` become.
It needs a release,
because `0.5.2-alpha.25` already carries the host bound:
a veto reverts released behaviour.
What it changes for good:
a join is measured by the line the rule sees,
as `0.5.2-alpha.24` measures it.
What it gives up:
the rule writes lines over the `120` it enforces,
by up to `11` characters at an indent of 8,
and the removal counts move:
`B`'s `162,707`,
the `47,500` the guards refuse,
`F1`'s `62,521` and `F2`'s `36,540` removable are the virtual bound's counts,
and the survey's own `command.txt` says a run at the tip gives lower ones
"where a Rust doc comment's host line is over the bound".
By how much is not counted;
the bound on that move is each predicate's Rust doc share:
`A` `19.18` percent,
`B` `21.23`,
`D` `22.52`,
`F1` `18.26`,
`F2` `14.80`.
A yes here leaves undecided which of those three tests survive in what form.

#### Veto it and set no bound at all
Mechanism:
a join may write any width,
so the width guard is removed.
What it changes for good:
no joined line is refused for its width.
What it gives up:
the rule enforces no bound in the line it writes,
and MD1's `120` becomes advice in a second place.
A yes here leaves undecided how a join chooses what to join.

Ranking:
keeping the host line's bound > vetoing to the virtual line's bound > vetoing to no bound,
because:

- The first over the second:
  the first is the only one where the bound the rule enforces holds on the line a reader opens,
  and the second writes lines up to `11` characters over it.
- The second over the third:
  the second keeps a bound of `120` on the line the rule sees,
  and the third keeps none.

The forced counts and both clause predicates are the line the rule sees under every answer,
so they do not move:
the survey's widths never included the host prefix.
A host-line bound moves some Rust doc clauses into the forced set,
bounded above by each predicate's Rust doc share,
which is the list above.

### 4. Which predicate reports

What earlier answers fixed:
decision 2's five answers take their rows out of every predicate,
so the counts below are printed under two sets and the per-kind table gives every other.
Decision 3's answer moves the removable counts and so the figure this decision ranks on,
and this round did not measure the order under it:
each predicate's removable count falls by up to its Rust doc share,
between `14.80` and `22.52` percent,
so an owner who vetoes reads the table as a bound on each row and not as a new order.

The five predicates and "nothing" are in "What each predicate would report",
each with its counts,
its file count and a real example from `doc/` and from a Rust doc comment.

#### The criterion, and the workload it serves
The workload is the repository's own principle,
"Lint issues need to be taken care of asap":
what a predicate delivers is findings a command clears,
because those are the findings the repository can return to zero on.
So a predicate is ranked by the number of its findings a modeled removal join clears,
and where two agree,
by the number it reports that no join's guards allow,
which is work only a person can do.
What is not a reason:
how much a fix would cost to build,
which files it touches,
and how many findings a review would have to read.

Two things about that criterion,
said here rather than in a note:

- The fix it counts is a model,
  not a shipped command:
  it is `markdown_fragment_joins::fragment_join` with `FRAGMENT_WORDS` lifted,
  and the survey says so.
  "The removal count" below is what it measures.
- No reader judged any break,
  and no figure here says a finding is a defect a reader would agree with.
  The one measured proxy for that is the `ordinary` class,
  which a person defined,
  and it appears in this decision only as a cost.
  A judged sample of breaks,
  one per class,
  by a reader of another model vendor,
  is the measurement that would test it,
  and this round did not make it.

The ranking,
under "no" to all five kinds and under the recommended set,
from `figures-for-each-kinds-answer.py`:

```text
                            no to all five kinds        the recommended set
                            cleared   of which no join    cleared   findings
B every joinable break      162,707        0             124,259   124,259
A every break               162,707   47,500             124,259   153,425
F1 short next line           62,521   10,893              41,445    52,338
F2 short line                36,540    2,945              30,938    33,883
D unforced clause                 0  144,863                   0   111,457
nothing                           0        0                   0         0
```

`F1`'s and `F2`'s figures under the recommended set are their permit-none joinable totals
less the joins the permitted kinds' own rows remove,
which is the best bound the rows give;
the exact split of a shape predicate's joinable findings among the classes is not printed.

Ranking:
`B` > `A` > `F1` > `F2` > `D` > nothing,
the same order under both kinds answers,
because:

- `B` over `A`:
  both clear `124,259` findings under the recommended set,
  and `A` also reports `29,166` findings (`47,500` under "no" to all five kinds)
  that no join's guards allow,
  each of which stays until a person rewraps the line or edits words.
  `B` reports none of them.
- `A` over `F1`:
  `A` clears `124,259` and `F1` at least `41,445` (`62,521` under "no" to all five kinds),
  and `A` puts `127,985` ordinary-class breaks in front of an author against `F1`'s `39,464`.
- `F1` over `F2`:
  `F1` clears at least `41,445` against `F2`'s at least `30,938`,
  and `F2`'s cost is that it covers `28,731` ordinary breaks against `F1`'s `39,464`
  while it reports the shape issue 6642 names
  and its ordinary share is higher
  (`84.79` against `75.40` under the recommended set).
- `F2` over `D`:
  `F2` clears at least `30,938` findings and no command clears a `D` finding,
  because joining a clause is an operation that does not exist and is not designed;
  `D`'s cost is the `111,457` findings,
  one per clause
  (`144,863` under "no" to all five kinds),
  each needing a hand rewrap.
- `D` over nothing:
  `D` reports `111,457` findings at breaks the rule's own definition reads,
  and nothing reports none,
  so the last pair compares a finding class with the absence of one.

Recommendation:
**`B`**,
with what it leaves undecided:
the exact guards of the removal fix it needs (decision 5),
the anchor and message of a finding (no rule id exists yet),
and that the issue's own class is `92.5` percent inside it:
`36,540` of `F2`'s `39,485` breaks are ones `B` reports.

The per-kind table above also answers "which kinds",
so an owner who answers 2b and 2c yes
can read every predicate's count by subtraction and see this order does not move:
what moves is how many findings each predicate has,
and `D` moves from `66.71` to `86.70` percent ordinary against `B`'s `65.57` to `85.86`,
a difference inside the shift the tip's emphasis change can make (`D` has no durable row,
and `B`'s ordinary share at the tip is at most `66.80` percent at permit none).
That is why the criterion is not the ordinary share:
under the recommended set it separates nothing,
and this decision would have no basis.

### 5. What mechanism repairs the breaks already in the tree

What earlier answers fixed:
decision 4 picks the predicate,
and a clause predicate needs a fix that joins a whole clause,
which does not exist and is not designed.
Decision 3 moves the removable counts this decision's ranking reads.

The tree holds `210,207` quiet breaks at no boundary,
of which `162,707` are joinable by the shipped join's guards.
No shipped command repairs any of them.

The options:

#### A removal fix built on the shipped join's guards
Mechanism:
it joins each break's two lines where the guards allow,
`162,707` of the `210,207`,
with `FRAGMENT_WORDS` lifted as the survey lifts it.
What it changes for good:
new fixes of this kind become automatic,
and the lines that were already alone are reached.
What it gives up:
it does not exist,
the `47,500` breaks the guards refuse stay,
a clause of three lines needs more than it does,
and it has to be shown clear of the three fixer behaviors the record lists.
A yes here leaves undecided who designs it,
its guards' exact widths,
and when it ships.

#### The revert-and-fix route of `96c930201f`
Mechanism:
run the installed fix over pre-sweep copies of the files a round changes.
It ran that way and repaired `771` paragraphs in `367` files.
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
in that package that prose follows with room under `120`.
A yes here leaves undecided what reaches the `458` and the `3,367`,
and which round runs it.

#### Both, in this order: a removal fix first, then the route for the files a later sweep cuts
Mechanism:
the removal fix reaches the lines already alone,
and the route keeps a sweep from leaving new ones.
What it changes for good:
both reaches are covered,
and a sweep after the fix does not reopen the class.
What it gives up:
the route's pass runs twice over the lines a wrap cuts,
once in the fix and once in the round's copies.
A yes here leaves undecided which round runs the second pass,
and whether the fix's own fixes make it unnecessary.

#### Nothing
The breaks stay,
and the next old-style wrap adds more.
What it changes for good:
nothing.
What it gives up:
every break in the tree stays,
and a new class's findings stay alongside them.
A yes here leaves undecided nothing about repairs,
and makes decision 8 moot.

Ranking under decisions 4's `A`,
`B`,
`F1` or `F2`:
a removal fix > both,
in this order > the route > nothing,
because:

- The removal fix over both:
  the fix already reaches every break the route reaches,
  so "both" delivers the fix's `124,259` cleared findings and the route's `90` of `548`
  at the price of running the join twice.
- Both over the route alone:
  the route alone leaves the `458` lines of that package that were already alone,
  and the fix's reach covers them.
- The route over nothing:
  the route repairs what a new wrap cuts,
  and nothing repairs nothing.

Under decision 4's `D` the order changes:
the route joins two lines and a `D` finding is a clause,
so no option here clears one;
ranking is by what each does for the next wrap,
where the route and the fix are both silent,
and "nothing" is the honest description of all four.
An owner who picks `D` should read this decision as "a clause fix is undesigned",
and the fourth read's `M3` is why that is said here.

### 6. Which release first reports the class

What earlier answers fixed:
decision 5 says whether a fix will exist,
and this decision says whether the class waits for it.
It also decides which severity ranking in decision 10 holds.

The options:

#### Report now, fix in a later release
Mechanism:
the class ships at decision 10's severity,
and `--fix` changes nothing for its findings until the fix ships.
What it changes for good:
the class is measured in the tree while the fix is designed,
and that measurement is the one this proposal does not have.
What it gives up:
the findings wait for a fix,
and a person meets them with no command to clear them.
A yes here leaves undecided which release carries the fix and what it does.

#### Report only when the fix ships, both in one release
Mechanism:
no finding of the class exists until a release carries the fix.
What it changes for good:
no finding appears that nothing can clear.
What it gives up:
nothing of the class is measured until a fix exists,
and no fix exists,
so the class does not ship in this round either way.
A yes here leaves undecided when the fix exists,
and,
for a clause predicate,
what a fix that joins a clause would be.

Ranking:
reporting now > reporting with the fix,
because:

- Reporting now over reporting with the fix:
  the first measures the class in the tree,
  which is the only way the `output.txt` counts become the tree's counts,
  and the second delivers nothing until a fix that does not exist is designed.

### 7. Whether a repair fix would run under the repository's format task

What earlier answers fixed:
decision 5 says whether a fix will exist at all.
This decision is asked only if it will,
and it decides whether decision 8 is the owner's to answer.

The repository's format task runs `monochromatic-lint --fix` over `.`
(`mise.toml` lines 1184 to 1188 and 1210 to 1215),
and the linter applies a `fixClass: whitespace` fix under plain `--fix`.
Joining two lines is a whitespace change:
the only bytes it removes are a newline and the next line's indent,
and the `>` or `///` markers of the container become syntax rather than words.
So a removal fix in the existing rule's class would be applied by the format task
over the whole tree,
the next time anyone runs `mise run format`,
at either severity,
with no separate decision.

The options:

#### Yes: the fix is a whitespace-class fix, and the format task applies it
Mechanism:
the fix joins every break its guards allow,
wherever `--fix` is pointed,
and the repository's own task points it at `.`.
What it changes for good:
the class leaves the tree on the next format run,
without a round.
What it gives up:
the owner does not choose the scope,
and the record's own warning applies:
an earlier fixer run "changed rendered text in 58 of the 498 files it changed in the documentation families".
A yes here leaves undecided how the next format run is proved not to change rendered text,
and makes decision 8 moot.

#### No: the fix gets a class the format task does not run
Mechanism:
the fix is not applied by plain `--fix`,
so a round applies it to paths it names,
and decision 8 sets those paths.
What it changes for good:
the scope stays a decision,
and a format run never joins a line by surprise.
What it gives up:
a person running `--fix` does not get the class's joins,
and the class needs a round per scope.
A yes here leaves undecided how a round applies a fix the task does not,
which is a task of its own.

Ranking:
no > yes,
because:

- No over yes:
  yes hands the scope to whichever format run happens next,
  over a tree of `162,707` joinable breaks,
  and the record already holds one sweep that changed rendered text in `58` files;
  no keeps the first round a decision the owner makes.

### 8. Which files the first repair round covers

What earlier answers fixed:
decision 7 says whether this is the owner's to answer at all,
and decision 5 says whether there is a fix.
The counts below are in the unit of whichever predicate decision 4 picked,
and they are the counts under "no" to all five kinds,
because `output.txt`'s family rows are per predicate and not per class:
under the recommended set they fall by the rows decision 2 gives,
and the per-family split of those rows is not printed.

The options:

#### The whole tree
`A`:
`210,207` findings in `8,721` files.
`B`:
`162,707`.
`D`:
`144,863`.
`F1`:
`73,414`.
`F2`:
`39,485`.
What it changes for good:
the class leaves the tree in one round.
What it gives up:
the largest rewrite any option here proposes.
A yes here leaves undecided how the rewrite is proved not to change rendered text.

#### The documentation families, then the package in that order
`A`:
`140,090` under `doc/`,
then `69,575` under `package/`.
`B`:
`106,496`,
then `55,740`.
`D`:
`93,494`,
then `50,956`.
`F1`:
`50,836`,
then `22,427`.
`F2`:
`29,489`,
then `9,894`.
What it changes for good:
the largest single block leaves first,
in files whose fixes a round can check family by family.
What it gives up:
the `471` breaks elsewhere for `B`,
and a second round for the package.
A yes here leaves undecided when the second part runs.

#### The documentation families only
`A`:
`140,090`;
`B`:
`106,496`;
`D`:
`93,494`;
`F1`:
`50,836`;
`F2`:
`29,489`.
What it changes for good:
the largest single block leaves.
What it gives up:
the package's `55,740` for `B`,
`19.2` percent of the population in Rust doc comments among them.
A yes here leaves undecided what covers the rest.

#### The one package a repair route has run over, `package/git-policy/cli`
`F2`:
`4,010` findings in `1,481` files.
No row of `output.txt` counts another predicate inside one package,
so for `A`,
`B`,
`D` or `F1` this scope's count is not known.
What it changes for good:
the smallest queue,
and the only scope a route has actually run over.
What it gives up:
the rest of the tree stays,
and for four of the five predicates the round's size is unknown before it starts.
A yes here leaves undecided when the other families get a round.

Ranking,
by the findings a round clears in the unit of decision 4's predicate,
with each option's risk stated beside it:
the whole tree > the documentation families then the package > the documentation families only > the one package,
because:

- The whole tree over the families then the package:
  the whole tree clears the `162,707` findings `B` reports under "no" to all five kinds in one round,
  and the two-block scope clears `162,236` of them and needs two rounds,
  whose second runs over files the first left as the tree's.
- The families then the package over the families only:
  both clear the same `106,496` first block,
  and the two-block scope also reaches the package's `55,740`.
- The families only over the one package:
  the families clear `106,496`,
  and the one package's count is printed for `F2` alone
  (`4,010` findings in `1,481` files),
  so for four of the five predicates its clearing is unknown before the round starts.

Each option's risk is the record's own:
an earlier fixer run "changed rendered text in 58 of the 498 files it changed in the documentation families".
That is a cost of every option here and not a reason to rank one lower,
because the round that applies the fix can check each file the way the repository's sweeps did,
by comparing it with its old text once whitespace and comment markers are taken off.

### 9. Whether the class has a rule id of its own

What earlier answers fixed:
nothing.
This decision and decision 10 fix each other in one direction:
a shared id fixes the severity.

The rule takes one severity per run
(`semantic_line_breaks(context, severity)`),
and the configuration names one severity per rule id
(`monochromatic-lint.config.jsonc` lines 258 to 260),
so a class reported under `markdown/semantic-line-breaks` takes that rule's severity.

The options:

#### A rule id of its own
Mechanism:
the class gets its own code and its own severity.
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

#### The existing rule id
Mechanism:
the class shares whatever severity `markdown/semantic-line-breaks` has.
What it changes for good:
one definition and one configuration entry.
What it gives up:
the class cannot be `warn` unless today's findings are,
and the notices-file rule-off switches the new class off with them.
A yes here leaves undecided the severity (decision 10) and whether a `warn` answer is the
loosening `AGENTS.md` rule LN7 forbids without prior approval.

Ranking:
a rule id of its own > the existing rule id,
because:

- The first over the second:
  the first lets the class report at a severity of its own while today's `error` findings stay
  where they are,
  and the second can only do that by demoting them,
  which is the loosening LN7 asks the owner to approve.

### 10. At which severity the class first reports

What earlier answers fixed:
decision 9 says whether the severity is the class's to choose,
and decision 6 says whether a fix exists when the class first reports.
The ranking below is given under both answers to decision 6,
because it reverses.

At `error` a run exits `1` on any finding;
at `warn` it exits `0`.
Both severities apply `--fix` when a fix exists,
and today none exists.

The options:

#### `warn`
Mechanism:
the class's findings are printed and no run fails on them.
What it changes for good:
every run reports the class and no run fails on it.
What it gives up:
the tree stays green while the predicate's findings go unreported
by anything that stops a commit or a run.
A yes here leaves undecided when the severity rises,
and whether today's findings move with it.

#### `error`
Mechanism:
the class's findings fail a `lint:monochromatic-lint` run.
What it changes for good:
the finding stops a run a person or a script starts,
so it is acted on.
What it gives up:
every run over a file holding one fails until someone clears it,
and with no fix in the tree nothing clears it automatically;
the repository's own principle that lint issues are cleared asap
meets a queue with no repair.
A yes here leaves undecided what clears the findings (decision 5),
and,
if decision 9 answers with the existing rule id,
that the class then carries that id's severity and today's findings keep theirs.

Ranking under decision 6's "report now,
fix in a later release":
`warn` > `error`,
because:

- `warn` over `error` while no fix exists:
  `warn` reports every finding and fails no run,
  and `error` fails every run over a touched file
  with nothing to repair it.

Ranking under decision 6's "report only when the fix ships":
`error` > `warn`,
because:

- `error` over `warn` once the fix ships with the report:
  the fix clears the `124,259` findings `B` reports under the recommended set,
  so `error` turns that clearing into a rule the tree holds,
  and `warn` leaves the same findings unread.

## What cannot be built, what needs an approval, and what needs an edit

Three lists,
because the fourth read's `M11` was that one list mixed them.

Answers that cannot exist:

- "Report nothing" with any answer to decisions 9 or 10:
  a class that reports nothing carries no severity and no rule id.
- "Report nothing" with any mechanism:
  a class with no finding has nothing to repair.
- `A` at `error` with zero findings:
  `26,558` ordinary breaks sit in forced clauses,
  and a forced clause keeps one break,
  so that answer cannot reach zero under any answer to decision 2 short of permitting the ordinary class,
  which this proposal does not offer.
- The existing rule id with the class at `warn` while today's findings stay `error`:
  one rule id carries one severity in the configuration,
  so that combination is not expressible.
- `B` with a finding the join's guards refuse:
  `B` is defined as the breaks the guards allow,
  so the combination is a contradiction and not an option.

Answers that need the owner's approval under `AGENTS.md` rule LN7
("Never loosen lint rules without prior approval."):

- The existing rule id at `warn`,
  which demotes today's `error` findings to `warn`.
  Nothing else here loosens a rule:
  a rule id of its own loosens nothing,
  because today's findings keep their severity.

Answers that need an edit of the owner's rule text,
which no decision above makes:

- Recording any permitted kind in MD1.
  Each of decisions 2a to 2e says what MD1 would have to say,
  and a yes to the kind leaves MD1 as it is.
- Recording the dash answer:
  WR2 gains a prose exception,
  or the class reports a line the rule text forbids.
- Recording the closer-ended answer:
  MD1 names a closing bracket or quote as a boundary.

Combinations an owner could want,
which this set of decisions reaches and the earlier version did not:

- The hold governing today's findings only,
  with the existing rule id.
  Reachable now:
  the class shares `error`,
  and decision 9's ranking says what that costs.
- A clause predicate with any answer to decision 2.
  Reachable now:
  `D`'s counts are the per-kind table's `D` column subtracted from `144,863`.
- Any set of kinds,
  including the conjunction kind alone or the parenthesis kind alone.
  Reachable now,
  because the five are separate decisions and not a chain.
- Decision 3's veto with a removal fix.
  Reachable now:
  the fix's counts move by the Rust doc share decision 3 prints,
  and the order of decision 4's ranking does not change.

"Both,
in this order" is reachable at decision 5,
as its own option.
At decision 8 it is the second option,
the documentation families then the package.
At the kinds it is not a pair of alternatives at all:
the five are independent.

## What is not known, and what this round did not run

Not known,
and said so where it is used:

- How many findings the rule reports today.
  The last recorded count is `171,262` at `136a40fe69`,
  before the sweeps.
- The exact removal counts under the host line's bound,
  bounded by each predicate's Rust doc share.
- The per-predicate split of the closer-ended breaks,
  and so what a yes to decision 2e removes.
- How many forced clauses of three words or fewer the whole tree holds,
  and how many of those no answer to decision 2 clears.
  `doc/audit`'s `163` files hold `21` and `1`.
- The per-kind file counts,
  so a predicate's file count under any answer to decision 2 but "no" to all five.
- The joinable split of `F1`'s and `F2`'s findings among the classes,
  so their cleared counts under the recommended set are bounds.
- Whether the sweeps the records name were within the hold.
- Whether the fixer's three recorded defects are fixed.
- Whether a reader would agree that an `ordinary`-class break is a defect.

Not run,
and why:

- The survey was not rerun,
  and the kept harness was not compiled:
  the fourth read found that none of what it asked needs a rerun,
  and a rerun takes about twenty-two minutes of build and run.
- The linter was not run over the repository,
  which is what "how many findings today" needs.
- No fix was built,
  and the removal counts are the survey's model.
- No sample of breaks was judged by a reader.
- Nothing was measured on a guest,
  in a container,
  or against a network service.

## Figures, and where each command and its output are kept

Every figure above is a row of the survey folder's `output.txt`,
`package/linter/monochromatic-lint/doc/a-survey-of-line-breaks-at-no-semantic-boundary/`,
except these,
whose command and output are in that folder's `figures-the-survey-does-not-count/`
or in `figures-the-proposal-derives/` beside it:

- The per-kind table,
  the two predicates' counts under each kinds answer,
  the clears and the coverage figures:
  `figures-the-proposal-derives/figures-for-each-kinds-answer.py`
  and `figures-for-each-kinds-answer.output.txt`.
- The rule file's `1,030` breaks and its `21`,
  with their classes:
  `figures-the-proposal-derives/the-rule-files-prose-walked.py`
  and `the-rule-files-prose-walked.output.txt`.
- Each example's class,
  clause width,
  forced flag and word counts,
  and the quoted text as the file holds it:
  `figures-the-proposal-derives/the-examples-checked-against-their-files.py`
  and `the-examples-checked-against-their-files.output.txt`.
- The `8,234` clauses,
  the `2,331` forced ones,
  the `21` and the `1`:
  `figures-the-proposal-derives/forced-clauses-a-rewrap-cannot-clear.py`
  and `forced-clauses-a-rewrap-cannot-clear.output.txt`.
- A finding as the installed binary prints it,
  with the fixed text:
  `figures-the-proposal-derives/a-finding-as-an-author-meets-it.txt`.
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
- The sweeps' commits and dates are read from `git log` with the commands in
  `figures-the-proposal-derives/the-sweeps-and-the-hold-in-the-records.txt`.

## What the removal count is

The removal count asks the shipped join,
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
It is a model of a fix that does not exist,
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
