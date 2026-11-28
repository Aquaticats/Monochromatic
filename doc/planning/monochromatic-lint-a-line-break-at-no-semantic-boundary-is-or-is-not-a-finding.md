# A line break at no semantic boundary: a finding, or not

Status:
written 2026-11-27 by the round that fixed issue 6642
(`fix/monochromatic-lint-issue-6642-the-line-break-fix-joins-what-the-old-wrap-cut-and-leaves-no-lone-word`),
revised the same day by the round that reran its survey,
and revised again on 2026-11-28 by the round that reran the survey
against the second gate read of this proposal.
**Unapproved and non-final:**
nothing here is built,
the rule reports exactly what it reported before,
and the owner declares completion.

It has had two same-vendor gate reads,
neither of them an independent review:
the first read the version at `cdf69930fd` and found 16 findings
(5 High,
8 Medium,
3 Low),
and the second read the version at `5bb8813b28` and found 22 findings
(2 High,
13 Medium,
7 Low).
Each finding of the second read,
and the three the first read called answered in part,
is answered in the section "Answers to the second gate read".
Every count in this proposal comes from one run of the survey
at the commit its "The measurement" section names.

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

`AGENTS.md` rule MD1 says:
"Break lines at semantic boundaries,
under 120 chars,
so text reads without editor wrapping."
Rule MD8,
in the same section,
says:
"Hard wraps fall only between whole inline spans;
keep code spans,
emphasis,
and link syntax on one line,
or the closing delimiter renders literally."

MD1 names no set of characters.
It names "semantic boundaries" and a bound.
MD8 permits a hard wrap between whole inline spans,
so a break before a code span or an emphasized phrase
is a wrap the repository already allows.
The six characters the rule breaks after are the rule's own set,
not a definition MD1 gives.
The survey below classifies every break by the first token of the next line's prose,
so the owner can say which kinds MD1 permits;
this proposal does not answer that question.

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
  The run found none.
- A line whose last visible character is a bracket or a quote
  reads as a boundary of its own.
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
  The merge named in question 4 makes the boundary test climb `*` `_` and `~` too,
  so a rerun at the branch's tip would count such a line as a boundary
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
such as `#tag`.

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
that the folder's `README.md` names and that this round reverted:

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
It includes `9,603` closer-ended breaks and `3,383` emphasis endings.
It excludes the `11,552` in paragraphs the existing rule already reports,
and that exclusion is a measured count,
not a lower bound:
after the shipped fix ran over copies of the `222` files that hold those paragraphs,
`10,028` of the `11,552` breaks are still there
and the fix joined `1,524` of them.
A run of the fixed tool over the whole tree would therefore leave
`210,207 + 10,028 = 220,235` breaks for every predicate to start from,
as an upper bound:
`221` of the `222` files carry no finding after the fix,
and one still holds `64`,
whose breaks may sit in paragraphs the rule still reports.
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
a reader cannot rewrap it into a line under `120` without adding words.
`26,558` of the forced breaks are ordinary-class,
`13,450` parenthesis-class,
`10,091` inline-span-class,
`8,849` conjunction-class,
`6` block-marker-class and `5` dash-class.
The conflict between MD1's two halves therefore exists
under the reading that permits none of those kinds;
an owner who permits a kind removes that kind's forced breaks with it.

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

## What each predicate would report

Six predicates are counted,
in each one's own population,
in clauses where it reports clauses and in breaks where it reports breaks.
`output.txt` labels them `A`,
`B`,
`D`,
`D2`,
`F1` and `F2`.
Each takes the owner's answer to "which kinds MD1 permits" as a filter:
a kind the owner permits is removed from every predicate.

### Report every break at no boundary

`210,207` findings in `8,721` files.

Reports,
from `.agents/skills/choosing-technology/SKILL.md:688`:

```text
Clone every serious alternative and finalist under a private `~/temp/agent/` root using the repository
cloning convention.
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

### Report only a break a removal can join

`162,707` findings in `8,185` files.
`2,997` of them are emphasis endings the existing rule breaks again,
so the durable count is `159,710` in `8,164` files.

Reports,
from `.agents/skills/css/SKILL.md:170`:

```text
 user-configurable properties
where the fallback represents the default user choice.
```

Leaves the break at `.agents/skills/android-emulator/SKILL.md:22`
because the joined line would reach the `120` bound.

### Report an unforced break at clause level

One finding per clause under `120` characters
with at least one break at no boundary,
no hard break,
and no emphasis ending,
in a paragraph the rule reports nothing for.
`144,863` findings in `8,085` files,
covering `148,203` breaks.
The `47,811` forced clauses are not reported.

Reports the clause of `.agents/skills/css/SKILL.md:170` quoted above.

Leaves the forced clause of `.agents/skills/android-emulator/SKILL.md:22`:

```text
 Briefs for agents that use an emulator repeat the headful rule and name the launch form in
 `doc/troubleshooting/android-emulator-37-software-renderer-sigsegv.md`;
```

### Report an unforced break at clause level, without the permitted edges

The clause predicate,
but a clause is not reported when any of its breaks opens a parenthesis or an inline span,
or when the line before any of its breaks ends in a dash.
The test reads every break of the clause,
not only its first.
`106,460` findings in `7,375` files,
covering `108,197` breaks.

Reports the clause of `.agents/skills/css/SKILL.md:170` quoted above.

Leaves this fixture's clause,
whose first break is ordinary and whose second opens a parenthesis:

```text
alpha beta gamma delta
continuation words begin here
(parenthesis opens this line
and this is the last line.
```

### Report a break that leaves a one-or-two-word next line

`73,414` findings in `7,072` files,
of which `62,521` are removable,
and `14,694` sit in a clause that reaches `120`.

Reports the break at `.agents/skills/android-emulator/SKILL.md:22`:

```text
 Briefs for agents that use an emulator repeat the headful rule and name the launch form in
 `doc/troubleshooting/android-emulator-37-software-renderer-sigsegv.md`;
```

Leaves the break at `.agents/skills/css/SKILL.md:170`,
whose next line holds eight words.

### Report a one-or-two-word line that ends at no boundary

The class issue 6642 names.
`39,485` findings in `4,728` files,
of which `36,540` are removable.
`1,163` are emphasis endings and are excluded first,
which leaves `38,322` durable findings in `4,700` files,
and `5,613` sit in a clause that reaches `120`.

Reports the break at `.agents/skills/css/SKILL.md:170` quoted above.

Leaves the break at `.agents/skills/android-emulator/SKILL.md:22`,
whose line holds many words,
and leaves the comma-ended `/// 4,` and `/// held,` of the issue itself.

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
Whether the hold still governs this question is the owner's to say,
not this proposal's.

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

## The decisions, and why they are apart

Four decisions are separate,
and a reader can choose one answer to each.
Nothing below bundles two of them into one option.

1.
   **Which kinds MD1 permits.**
   A filter every predicate takes as a parameter.
2.
   **Which predicate reports.**
   One of the six above,
   or none.
3.
   **The timing.**
   At which severity the finding first ships,
   and whether that release carries a removal fix.
4.
   **The repair.**
   What happens to the breaks already in the tree,
   and in which round.

The bound a join uses is a fifth question,
because the shipped join measures the line the rule sees
and the earlier round refused a host-aware bound.
The hold is a sixth,
because it is the owner's own instruction.

"Both,
in this order" is reachable at every one of them:
the kinds can be permitted in stages,
the severity can rise in a later release,
and the repair can follow the report.
Nothing in this list has to be answered before another.

## The ranking of the predicates

The criterion comes first.
A predicate is ranked
by the share of its own findings that a reader would call a defect under MD1 and MD8 as written,
which the survey measures as the ordinary class,
because each of the other classes names a kind MD1 or MD8 may permit;
and then,
among predicates that agree on that share,
by how many ordinary-class breaks it puts in front of the author.
What is built today,
how many files a change would touch,
and which fix a predicate needs are costs,
stated in "What each predicate would cost" and never used to place a predicate lower.

The ordinary share and the ordinary break count of each predicate,
both measured:

```text
the unforced clause without the permitted edges   90.2% of 108,197    97,635
the short line                                    72.8% of  39,485    28,731
the unforced clause                               66.6% of 148,203    98,708
every joinable break                              65.6% of 162,707   106,692
every break                                       60.9% of 210,207   127,985
the short next line                               53.8% of  73,414    39,464
nothing                                           reports no break
```

The ranking:

1.
   The unforced clause without the permitted edges.
1.
   The short line.
1.
   The unforced clause.
1.
   Every joinable break.
1.
   Every break.
1.
   The short next line.
1.
   Nothing.

The unforced clause without the permitted edges above the short line:
it puts `97,635` ordinary-class breaks in front of the author against the short line's `28,731`,
and none of its findings sits at an edge MD8 may permit,
while `20.3` percent of the short line's findings do
(`5,137` before an inline span,
`2,896` before a parenthesis,
`2` after a dash).
The short line's advantage is its reach in files:
`4,728` against `7,375`.

The short line above the unforced clause:
the two differ by agreement first,
`72.8` percent against `66.6`,
because the clause predicate keeps the `26.1` percent of its breaks
that sit at the parenthesis,
inline-span and dash edges,
and it is the shape issue 6642 names.

The unforced clause above every joinable break:
the clause predicate excludes every clause holding an emphasis ending,
so none of its `144,863` findings contradicts the rule,
while `2,997` of the joinable-break predicate's `162,707` do,
and the clause predicate's findings agree slightly more often (`66.6` against `65.6`).
The joinable-break predicate puts `7,984` more ordinary breaks in front of the author.

Every joinable break above every break:
the joinable-break predicate reports every break the shipped guards allow and leaves out the
`47,500` breaks the shipped guards refuse,
so it reports no finding without a join the guards allow,
and its ordinary share is higher (`65.6` against `60.9`).
Every break carries `58,959` forced breaks and `3,383` emphasis endings,
where a person has no action under MD1's bound.

Every break above the short next line:
`60.9` percent of `210,207` against `53.8` percent of `73,414`,
and `127,985` ordinary breaks against `39,464`;
`32,208` of the short next line's findings,
`43.9` percent,
sit at the edges MD8 may permit.

The short next line above nothing:
it puts `39,464` ordinary-class breaks in front of the author,
and reporting nothing puts none.

What this ranking does not settle:
if the owner permits conjunction breaks,
the short line loses `2,706` of its `39,485` findings and the edge-free clause
loses `10,523` of its `108,197`,
and the clause predicates rise above the short line on both counts.
The owner's kind list decides whether the order above holds.

## What each predicate would cost

These are costs,
not reasons to place a predicate lower.

- No removal fix exists.
  Every predicate except "nothing" reports findings no shipped command clears today.
  A removal fix built from the shipped join's guards would clear the findings of
  the short line,
the short next line,
every joinable break and every break
  (the survey's model).
  The clause predicates need a fix that joins more than two lines,
  and that operation,
its guards and its width are unmeasured because it does not exist.
- Files a change would touch:
  `8,721` for every break,
  `8,185` for every joinable break,
  `8,085` for the unforced clause,
  `7,375` for the edge-free clause,
  `7,072` for the short next line,
  `4,728` for the short line,
  and none for nothing.
- Findings with no author action under MD1's bound:
  `58,959` for every break,
  `14,694` for the short next line,
  `5,613` for the short line,
  and none for either clause predicate.
- Findings the rule breaks again:
  `3,383` for every break,
  `2,997` for every joinable break,
  `1,163` for the short line,
  and none for either clause predicate.
  These are the survey commit's,
  where the boundary test did not climb the emphasis delimiters;
  after the merge named in question 4 it does,
  so the class is empty there.
- The exit status:
  at `error` a scoped run exits `1` on any finding,
  and at `warn` it exits `0` while `--fix` still rewrites the file.

## What the owner has to decide

### 1. Which kinds of break MD1 permits

MD1 names "semantic boundaries" and a bound and no set of characters.
MD8 permits a hard wrap between whole inline spans.
The survey files every break by the first token of the next line's prose,
and the population of each kind is:

```text
block-marker      65
conjunction   19,749
dash              20
inline-span   25,355
ordinary     127,985
parenthesis   37,033
```

The answers,
and what each does:

- Permit none of them:
  every predicate reports the population above in full,
  `210,207` breaks for the widest.
- Permit the three MD8 names
  (a break before a parenthesis,
before an inline span,
after a dash):
  the report loses `62,408` breaks,
and the ordinary and conjunction kinds stay.
- Permit those three and a break before a conjunction:
  it loses `19,749` more,
`82,157` in all,
  and only the `127,985` ordinary breaks stay reportable.
- Permit an ordinary break too:
  no break is left to report,
  and this answer is the "nothing" predicate of question 2.
- Permit some other subset:
  the per-kind counts above say what each removal takes.

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

The examples of each kind are quoted in "What each predicate would report",
and the block-marker kind,
from `doc/audit/tech-mold-linker-vet-2026-10-05.md:786`:

```text
  #1695 closed unmerged with follow-up issue
  #1696 open (thread-pool tuning discussion);
```

The alternative reading the owner may prefer,
that a break at a word boundary an author judged semantic is permitted,
would leave no finding and is not distinguishable by the survey,
which counts a position and not an intention.

### 2. Which predicate reports

The owner needs,
for each:
its count in files,
its forced share,
its emphasis share and its reach.

- Report nothing (`C` in the output):
  no finding,
  no churn,
  and the `39,485` short lines stay.
- Report only a break a removal can join (`B`):
  `162,707` findings in `8,185` files,
  every one with a join the guards allow,
  `2,997` of them emphasis endings.
- Report every break at no boundary (`A`):
  `210,207` findings in `8,721` files,
  `58,959` of them forced,
  `3,383` emphasis endings.
- Report the unforced clause (`D`):
  `144,863` findings in `8,085` files,
  covering `148,203` breaks,
  none forced and none an emphasis ending.
- Report the edge-free clause (`D2`):
  `106,460` findings in `7,375` files,
  covering `108,197` breaks.
- Report a break before a short line (`F1`):
  `73,414` findings in `7,072` files,
  `14,694` forced.
- Report a short line that ends at no boundary (`F2`):
  `39,485` findings in `4,728` files,
  `5,613` forced,
  `1,163` emphasis endings.

A predicate reports at whichever severity question 3 answers with,
and reports the kinds question 1 leaves.

### 3. At which severity, and when a fix ships

The rule takes one severity per run
(`semantic_line_breaks(context, severity)`),
and the configuration names one severity per rule id
(`monochromatic-lint.config.jsonc` lines 258 to 260).
So a new class under `markdown/semantic-line-breaks` would take that rule's severity.
Two routes give it a severity of its own:

- **A new rule id.**
  The class gets its own code and its own severity,
  so it can be `warn` while today's findings stay `error`.
  The block that switches this rule off for the notices file
  (`monochromatic-lint.config.jsonc` lines 339 to 341) needs a second entry for the new code,
  unless the owner wants that file covered by the new class.
  A new code can also carry a path list that only shrinks,
  which a shared severity cannot express.
  It costs a second definition of "a break at no boundary" beside the rule's own break-point test,
  and two codes to keep in step.
- **The existing rule id.**
  The class shares whatever severity `markdown/semantic-line-breaks` has.
  Choosing `warn` demotes the findings that are errors today,
  and the notices block switches the new class off with them,
  since it names the rule and not the class.
  It keeps one definition and one configuration entry.

At `error` a scoped run exits `1` on any finding;
at `warn` it exits `0`.
Both severities apply `--fix` when a fix exists.
Today none exists,
so a first release reports and rewrites nothing either way,
and the timing question is which severity first,
and whether the release that
reports it also ships the removal fix.

### 4. Which bound a join uses

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

The answers:

- The rule's own bound,
which is none:
  a join may write any width,
and no rule enforces MD1's `120` today
  (a 123-character prose line passes with exit 0).
- MD1's `120` on the virtual line,
which is what `0.5.2-alpha.24` and the survey commit do:
  a host doc line may stand up to 11 characters over `120` at an indent of 8.
- MD1's `120` on the host line,
marker and host indent included:
  this is what the tree does after the merge named below,
  and the earlier round refused it,
  so the owner is being asked again with the measured widths above.

Update,
2026-11-28:
the join and the boundary test changed after the survey ran.
Commit `fc679b9392` (2026-11-28T03:02:49-05:00) makes the join count the host line,
its comment marker and host indent included,
so a joined Rust doc line stays under `120` in the host file,
and it settles in one pass;
the same commit makes the boundary test climb `*` `_` and `~`.
Both merged into this branch after the survey commit,
the survey was not rerun,
and the widths above are the release's.
The record of that choice and of what each option costs is in
`package/linter/monochromatic-lint/doc/second-pass-reads-of-released-versions.md`.

### 5. Does the hold still govern this rule

The owner's recorded words are "Put the semantic line break findings on hold.",
moved verbatim on 2026-11-23 into
`doc/planning/repository-linter-stays-at-zero.md` lines 40 to 43.
No later record lifts it or renews it.
While it stood,
two sweeps of this rule's findings merged:
`beb61ec46b` on 2026-11-27,
counted in the handover as `4,062` findings in `480` files,
and `96c930201f`,
which repaired `771` doc paragraphs in `367` files,
netting `90` fewer doc lines.
Whether those sweeps fell within the hold,
and whether the hold governs a new class,
are the owner's to say.

### 6. What repairs what is already in the tree

The tree holds `210,207` quiet breaks at no boundary,
and `4,010` of them are the short-line shape inside `package/git-policy/cli`
in `1,481` files.
What exists to repair them:

- The revert-and-fix route of `96c930201f`:
  it ran the installed fix over pre-sweep copies and repaired `771` paragraphs
  in `367` files.
  Its own measurement (`81650758c4`) records what it reached and what it did not:
  of `548` one- and two-word doc lines that ended at no break point
  in the `471` swept files,
  it repaired exactly the `90` the sweep itself cut,
  and the `458` that were already alone stayed.
  The whole package still held `3,367` such lines after it.
- A removal fix built on the shipped join's guards:
  it would reach every line,
including the ones that were already alone,
  and it does not exist.
- Nothing:
  the breaks stay,
and the next old-style wrap adds more.

Which tool repairs the rest,
whether the owner accepts a one-time rewrite of `1,481` files in one package,
and in which round,
are the owner's to say.

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
  which the widths in question 4 measure,
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

Built:
the harness with each predicate in its own population,
three fixtures asserting exact numbers,
the positive control that sees every class,
the after-the-fix measurement,
the corpus list at the rerun commit,
the whole output,
and this revision.
Changed nothing that ships:
no rule,
no fix,
no sweep,
no configuration,
no workflow.

Measured by this round:
every count above,
the installed binary on `package/git-policy/cli`
(`26` findings in `20` files,
exit `1`),
the exit statuses on a one-rule fixture,
the host width of a joined doc line at three indents,
the absence of a width rule,
the emphasis boundary test,
and the control of the after-the-fix measurement,
which the run recorded in `output.txt` used in a form that never reached the fix.
That control is corrected in the kept harness and measured separately;
the recorded figure rests on all `222` copies changing bytes.
Inferred,
not run:
the removal counts,
the clause-width joins,
the Rust projection under a removal,
and the ten-pass fix loop's behaviour on a clause fix.
Unknown:
how many of the `38,322` durable short-line findings a sweep would change
in files other branches hold,
what the owner's kind list is,
and whether the sweeps that merged while the hold stood were within it.

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
  the repair and question 6 are restated at the rerun commit,
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
  The kinds,
  the predicate,
  the timing and the repair are four decisions.
- M5 asked for the ranking criterion.
  It is stated first,
  and build effort,
  files touched and fix needs are a cost section.
- M6 asked how a new class gets a severity of its own.
  A new rule id and the shared severity are both stated,
  with their effects on today's error findings,
  the notices block and a shrinking allow list.
- M7 asked for a measured width and questions that stand alone.
  The width is measured (`123`,
  `127`,
  `131`),
  the questions spell out their options,
  and the conjunction list is given.
- M8 asked for the sweeps that merged during the hold.
  Question 5 names both sweeps with their commits and counts,
  and the handover citation is lines 2749 and 2750 at the survey commit,
  and lines 2865 and 2866 after the merge.
- M9 asked for "forced" relative to the kinds.
  The forced count is given per class,
  and stated as the reading that permits no kind.
- M10 asked for the issue's own examples to be counted.
  The one-or-two-word lines ending at a break point are counted (`393,916`)
  beside those ending at none (`39,485`).
- M11 asked for a split by file family.
  Every predicate is split by family,
  and the population's split is stated.
- M12 asked for quoted lines classed by their prose and for the missing controls.
  A quoted line is classed after its marker,
  three fixtures assert three splits,
  and the positive control sees the block-marker class.
- M13 asked whether the reported paragraphs are a lower bound.
  They are measured after the shipped fix ran on copies,
  and `10,028` of the `11,552` remain.
- L1 asked whether an unterminated fence is reported.
  No rule reports it,
  and the answer names its commit and the README passage.
- L2 asked for the ways a line dodges a predicate.
  They are named in "How a line dodges each predicate".
- L3 asked for a pinned rerun and a snippet that compiles.
  The README names the commit and the worktree,
  the steps are the ones run,
  the module text is the compiling `//` form,
  and `command.txt` and the README agree on three temporary changes.
- L4 asked for figures that trace.
  Every figure the survey does not count has its command and output beside it,
  and the two records of other rounds are named as theirs.
- L5 asked that the tree's state rest on evidence.
  The sentence now claims only that package,
  with its run kept.
- L6 asked for paired figures in one unit.
  Paired figures are in one unit,
  and the clauses holding the forced breaks are counted.
- L7 asked where the README sentence holds.
  It holds at the rerun commit,
  and its change belongs to the round that is changing the fix.

The first read's three findings it called answered in part:

- H4 asked for a shrinking allow list and a rule id of its own.
  Both are weighed in question 3.
- Its M1 asked for the reported paragraphs to be measured.
  They are,
  as M13 above.
- Its M7 asked whether any rule holds prose under `120`.
  None does.
  The width is `123`,
  `127`,
  `131`.

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
