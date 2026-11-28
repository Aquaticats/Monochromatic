# A line break at no semantic boundary: a finding, or not

Status:
first written 2026-11-27 by the round that fixed issue 6642
(`fix/monochromatic-lint-issue-6642-the-line-break-fix-joins-what-the-old-wrap-cut-and-leaves-no-lone-word`)
and revised in the same session by the round that reran its survey
(`docs/planning-line-break-proposal-revised-against-its-gate-review-with-its-survey-kept`)
after a same-vendor gate read.
**Unapproved and non-final:**
nothing here is built,
the rule reports exactly what it reported before,
and the owner declares completion.
The owner's decision this proposal asks for is a yes or a no to a new finding class,
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
so a break before a code span or an emphasized phrase is a wrap the repository already allows.
The six characters the rule breaks after are the rule's own set,
not a definition MD1 gives.
The survey below classifies every break by the first token of the next line,
so the owner can say which kinds MD1 permits;
this proposal does not answer that question.

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

Four classes are counted apart because they are not one defect:

- A line ending in two or more spaces or a backslash is a Markdown hard break.
  The survey counts it and every option below leaves it alone.
  The survey found none.
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
- A line ending `e.g.`,
  `etc.` or `...` is treated as a boundary here,
  although `break_offsets` never breaks after an abbreviation or an ellipsis.
  The survey undercounts breaks at no boundary by that small class.

## The measurement, and what it counts

The survey ran at commit `c8545f91b8c79a8064490bcffa667d6ea79b83df`
(2026-11-27T22:56:16-05:00)
over the `10,472` tracked `.md`,
`.mdx` and `.rs` paths in
`package/linter/monochromatic-lint/doc/a-survey-of-line-breaks-at-no-semantic-boundary/files.txt`.
The harness reads each path's bytes with `git show <commit>:<path>`,
so the counts are that commit's and not a later worktree's.
The configuration ignores `66` of them,
and turns the rule off for `1`;
`10,272` host files yielded a parsed source.
The corpus includes the earlier version of this proposal,
because the list was taken before this revision;
the records added after it are not counted.
The whole output is
`package/linter/monochromatic-lint/doc/a-survey-of-line-breaks-at-no-semantic-boundary/output.txt`,
and the command and temporary edits are in `command.txt` beside it.

The command,
with the two temporary source edits
that the folder's `README.md` names and that this round reverted:

```sh
# from package/linter/monochromatic-lint
SURVEY_LIST="$PWD/doc/a-survey-of-line-breaks-at-no-semantic-boundary/files.txt" \
  SURVEY_ROOT="$(git rev-parse --show-toplevel)" \
  cargo nextest run --locked --lib --no-capture survey_
```

The positive control ran in the same command:
`survey_probe_sees_every_class_on_its_fixture` failed until the classifier saw
each class on a fixture of its own,
and its `PROBE` line is in `output.txt`.

The counts add up:

```text
breaks inside paragraphs:                                         1,541,282
  whose line ends at a break point:                               1,320,034
  at no boundary:                                                   221,248
    in paragraphs the rule reports nothing for:                     209,657
      ending in a closing bracket or quote:                           9,587
      ending otherwise:                                             200,070
    in paragraphs the rule already reports:                          11,591
      ending in a closing bracket or quote:                             260
      ending otherwise:                                              11,331
```

`209,657` is the population every option below starts from.
It includes `9,847` closer-ended breaks and `3,383` emphasis endings.
It excludes the `11,591` in paragraphs the existing rule already reports,
because the existing fix rewrites those paragraphs.

Each break is classified by the first token of the next line,
in this order:
`block-marker`,
`parenthesis`,
`inline-span`,
`conjunction` (a fixed list of conjunctions and subordinators),
`dash` (the line before ends in `--`,
`—` or `–`),
and `ordinary` for the rest.
The counts,
all in the `209,657`:

```text
block-marker    1,275
conjunction    19,647
dash               20
inline-span    24,994
ordinary      127,298
parenthesis    36,423
```

The `ordinary` class is `60.7` percent of the population;
the rest sit at one of the kinds MD1 may or may not permit.

Each break also carries the width of its **clause**:
the run of lines from the previous line ending at a break point or a hard break
to the next one,
joined with single spaces,
after each line's own indentation,
plus the paragraph's continuation indent.
`58,788` of the `209,657` sit in a clause at or over `120` characters,
in `47,103` clauses.
No single line may hold such a clause under MD1's bound,
and it holds no punctuation to break at,
so its break is **forced**:
a reader cannot rewrap it into a line under `120` without adding words.
That is the conflict between MD1's two halves in the text this repository holds.

The class issue 6642 names is
**a line of one or two words that itself ends at no boundary**:
`39,501` breaks in `4,745` files,
of which `36,599` are removals the shipped join's guards allow.
The reverse shape,
a break that leaves a one-or-two-word line after it,
is `72,913` in `6,998` files,
of which `62,283` are removable.
They are not the same class.

## Examples, by the rule that drew them

Each example is the first break of its class in sorted path order,
quoted whole from the line it names.
One break can be the first of two classes.

Conjunction,
`.agents/skills/choosing-technology/SKILL.md:152`:

```text
(not necessarily Haiku)
or run it themselves.
```

Parenthesis,
`.agents/skills/choosing-technology/SKILL.md:151`:

```text
because the user may delegate the enumeration to a cheaper model of their choosing
(not necessarily Haiku)
```

Inline span and forced clause,
`.agents/skills/android-emulator/SKILL.md:22`:

```text
 Briefs for agents that use an emulator repeat the headful rule and name the launch form in
 `doc/troubleshooting/android-emulator-37-software-renderer-sigsegv.md`;
```

Dash,
`.agents/skills/project-code-review/PHILOSOPHY.md:24`:

```text
The alternative -- referencing `AGENTS.md` sections by name ("apply the rules from the TypeScript Standards section") --
assumes the agent will cross-reference accurately across documents under context pressure.
```

Ordinary,
`.agents/skills/choosing-technology/SKILL.md:688`:

```text
Clone every serious alternative and finalist under a private `~/temp/agent/` root using the repository
cloning convention.
```

Block marker,
`doc/audit/2026-05-14-repo-assessment.md:3`:

```text
> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
```

Emphasis ending,
`doc/audit/deepseek-harness-node-domexception.md:182`:

```text
- **Install and audit surface:**
   present for every published CLI install.
```

The issue's own class,
`.agents/skills/css/SKILL.md:170`:

```text
 user-configurable properties
where the fallback represents the default user choice.
```

The wrapper's own file,
`package/git-policy/cli/src/native/worktree_clone_add_route/mod.rs:6`,
holds the shape the issue names,
and is quoted in the section on shipped defects below.

## What the records say, and what still stands

`doc/planning/repository-linter-stays-at-zero.md` records:

- A repository-wide run at `136a40fe69` reported `174,916` findings,
  of which this rule made up `171,262` (line 16).
- The owner's words,
  moved verbatim on 2026-11-23:
  "Put the semantic line break findings on hold."
  and "Put the semantic line break findings on hold.
  They aren't the most serious issues right now."
  (lines 40 to 43).
- The fixer "changed rendered text in `58` of the `498` files it changed
  in the documentation families" (line 186).
- `doc/handover/cli-git-rust-implementation.md` records a sweep of
  `4,062` findings in `480` files that had to wait
  because "the branches that were changing them could not take the diff" (lines 2797 to 2806).

What still stands:
the hold is the last recorded owner instruction about this rule,
and no later record lifts it or renews it.
The rule is still at `error` severity in `monochromatic-lint.config.jsonc`,
and the fixer's earlier rewrite of rendered text remains the reason
a blind sweep of quoted records is not safe.
Whether the hold still governs this question is the owner's to say,
not this proposal's.

The repository is not green today,
and no enforced gate stands behind the "commit path" this proposal used to name:

- `monochromatic-git-policy-cli.config.jsonc` line 10 runs
  `"markdown/autofix": ["warn", { "rules": ["lfs-image-url"], ... }]`:
  `git commit` through the wrapper runs no semantic-break check.
- `mise.toml` lines 1210 to 1215 make `format:monochromatic-lint` run
  `monochromatic-lint --fix` over `.` when no paths are given.
- `mise.toml` lines 1024 to 1025 say the linter
  "Exits 1 on error findings and 2 when the run could not be completed;
  warnings alone exit 0."
- The installed `0.5.2-alpha.24` on `package/git-policy/cli` at the survey commit
  reports `14` findings in `14` files and exits `1`:
  `13` `rust/max-lines` errors and one `rust/avoidable-closure` warning.
  The command is
  `/home/user/Monochromatic/.cache/toolchain-bin/monochromatic-lint package/git-policy/cli`.

So "returns to green" has no green baseline,
and a warn-severity release exits `0` while still applying `--fix`.

## What a removal would be, and what is not measured

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
The counts below therefore say what a removal would reach under the shipped guards,
not that a `--fix` run clears them.

## Options

Each option's first-run count is in the `209,657` population,
in files.
"Removable" means the shipped join's guards allow the removal.
"Commit" names what the Git wrapper's `git commit` sees,
which is nothing for every option,
and what an agent's `mise run format:monochromatic-lint` and
`mise run lint:monochromatic-lint` do.

### A: report every break at no boundary

`209,657` findings in `8,643` files,
including `9,847` closer-ended breaks and `3,383` emphasis endings.
If only the emphasis endings are excluded,
the count is `206,274`.
If the closer-ended breaks are excluded too,
because they read as boundaries of their own,
the count is `196,427`.

Pros:

- It answers the widest reading of MD1,
  not only the break-point half.
- One definition,
  and the `58,788` forced breaks are visible rather than hidden.

Cons:

- `58,788` findings sit in a clause that reaches `120`,
  where no join under MD1's bound exists.
  A reader cannot rewrap those into compliance without adding words.
- `3,383` emphasis endings contradict the existing rule:
  A asks for a join the rule breaks again.
- The diff reaches `8,643` files.
- At `error`,
  a scoped run exits `1` on lines the author did not write,
  and the removal fix would leave the forced and emphasis findings red.

At `warn`,
the finding exits `0` and the fix still rewrites every joined line
unless the fix is withheld.
`git commit` through the wrapper sees nothing.

### B: report only a break a removal can join

`162,285` findings in `8,109` files.
`2,997` of them are emphasis endings the existing rule breaks again,
so the durable count is `159,288` in `8,088` files.

Pros:

- Every durable finding has one join the shipped guards allow.
- It is A minus `47,372` findings the shipped guards refuse,
  so it asks for fewer joins no command can clear,
  though it still holds some forced breaks and the emphasis class.

Cons:

- The diff is the same order as A:
  `8,109` files,
  and a removal fix rewrites every one.
- It includes `2,997` emphasis endings unless that class is excluded first,
  and each cycles against the existing rule.
- It ties the report to what a fix can do,
  and no removal fix exists yet.
- It still reports the deliberate style MD1 may permit:
  `19,647` conjunction,
  `24,994` inline-span and `36,423` parenthesis breaks are in the population.
- The rule already declines block-start and ambiguous breaks
  (`markdown_semantic_breaks.rs` skips `starts_block_construct`;
  `markdown_break_points.rs` declines abbreviations and ellipses),
  so B is not "backwards from every other rule" as an earlier draft said.

At `error`,
the durable findings clear with one removal run if that fix ships;
the emphasis class does not.
At `warn`,
it exits `0` and the fix still rewrites the tree unless withheld.

### C: report nothing, as today

No new finding.

Pros:

- No churn and no new operation.
- The fragment join the earlier round shipped still repairs the fragment an inserted break cuts.

Cons:

- The `39,501` breaks of the issue's own class stay,
  and nothing tells an author about one.
- The wrapper's own lone `the` stays after the fixed linter ships:
  the join only asks about a fragment an inserted break cut,
  not about an author's line elsewhere in the paragraph.
  The reproduction is in the section on shipped defects.

At `warn` and at `error` the rule reports nothing new,
so the exit status is what it is today.

### D: report an unforced break at clause level

One finding per clause under `120` characters
with at least one break at no boundary,
no hard break,
and no emphasis ending,
in a paragraph the rule reports nothing for.
The fix joins the whole clause.
`144,483` findings in `8,015` files.
The `47,103` forced clauses are not reported.

Pros:

- The predicate is MD1's own bound,
  not what a fix can do.
- One finding per clause,
  so a run of consecutive breaks is one edit and no chain of joins.
- Every finding has an author action:
  the whole clause fits under `120`.

Cons:

- It still reports `36,423` parenthesis and `24,994` inline-span breaks,
  which MD8 permits and the owner has not excluded.
- It still reports `19,647` conjunction breaks,
  whose status MD1 does not settle.
- The clause fix is a new operation that joins more than two lines;
  its guards,
  its Rust projection and its width are unmeasured because no fix exists.
- At `error` it exits `1` in `8,015` files;
  at `warn` it exits `0` and the fix rewrites them unless withheld.

### D2: D without the span, parenthesis and dash edges

D,
but a clause is not reported when its first break opens a parenthesis or an inline span,
or when the line before that break ends in a dash.
`107,302` findings in `7,329` files.

Pros:

- It removes the kinds MD8 permits and the dash the owner may permit,
  so its findings are the plain mid-clause wraps.
- It turns the kinds question from a surprise into a decision the owner can make.
- It keeps D's clause-level fix and its one-edit-per-clause property.

Cons:

- It still reports `19,647` conjunction breaks,
  which the owner has not decided.
- It adds a second definition to maintain beside the rule's own break-point test.
- At `error` it exits `1` in `7,329` files;
  at `warn` it exits `0` and the fix rewrites them unless withheld.

### E: a warn-first ramp

E is not a separate predicate.
It is a release order for any report above,
instantiated here on F2:
report the issue's class at `warn`,
with the removal fix withheld,
sweep or accept the findings,
then ship the fix at `error`.
`38,405` durable findings in `4,508` files at the first run,
the same as F2 after its emphasis endings are excluded.

Pros:

- The first release changes no exit status and rewrites nothing,
  because the fix is withheld.
- It gives the owner the finding count in the first release
  before any branch pays for a sweep.
- The second release can be scoped to the files a sweep did not reach.

Cons:

- The defect stays in the text until the second release.
- A warn finding that no fix clears still adds output to every run.
- It needs two releases and a recorded decision between them.
- If the fix ships at `warn` instead of being withheld,
  `--fix` still rewrites the tree on the first day.

### F1: report a break that leaves a one-or-two-word next line

`72,913` findings in `6,998` files,
of which `62,283` are removable.

Pros:

- It names a visible orphan:
  a short line after the break.
- The shipped join already counts words,
  so the class is not a new language heuristic in the code.
- It is smaller than D and D2.

Cons:

- It is not the class the issue names;
  the issue's line is short before its break,
  not after it.
- `10,630` findings are not removable by the shipped guards,
  so a fix cannot clear the class.
- A short next line can be a line that itself ends at a rule-wanted boundary;
  the fix moves that line's start,
  not its ending,
  so it leaves the boundary in place,
  but the author still sees a join where the rule wanted a line end.
- At `error` it exits `1` in `6,998` files;
  at `warn` it exits `0` and the fix rewrites them unless withheld.

### F2: report a one-or-two-word line that ends at no boundary

This is the class issue 6642 names.
`39,501` findings in `4,745` files,
of which `36,599` are removable.
`1,096` are emphasis endings and must be excluded first,
which leaves `38,405` durable findings in `4,508` files,
of which `35,503` are removable.

Pros:

- It is the smallest class that names the defect the issue reports,
  and the one the shipped join already targets.
- `92.4` percent of its durable findings have a join the shipped guards allow.
- It is `38,405` durable findings in `4,508` files,
  about one fifth of A and about one third of D2,
  so a commit that runs `--fix` on changed files sees a bounded diff.
- It rarely reports a forced clause:
  a two-word line plus its next line is usually under `120`,
  and `2,902` findings are not removable.

Cons:

- It is a word count,
  and a word count is a heuristic in scripts that do not separate words with spaces.
- It leaves a three-word fragment alone,
  and it misses a short line that ends at a break point,
  which the rule wants to keep.
- `2,902` findings are not removable by the shipped guards,
  so a fix cannot clear the whole class.
- At `error` it exits `1` in `4,745` files;
  at `warn` it exits `0` and the fix rewrites them unless withheld.

### G: C plus a one-time repair of the wrapper's own class

Report nothing new,
and repair the `4,077` F2 breaks in `1,503` files under
`package/git-policy/cli` in a round of its own,
by a removal run limited to that package or by reverting the affected files and fixing again.

Pros:

- It fixes the file the issue was raised on without adding a finding class.
- It gives C's empty commit path.
- It is a bounded one-package rewrite.

Cons:

- It prevents nothing:
  the next old-style wrap stays.
- It rewrites `1,503` files in a package other branches change.
- It needs the owner to accept a one-time scoped rewrite.

At `warn` and at `error` it changes no finding.

## Ranking

1. F2.
1. E on F2.
1. D2.
1. F1.
1. D.
1. G.
1. C.
1. B.
1. A.

F2 above E:
F2 repairs the class in the release that reports it,
while E withholds the fix and leaves the defect until a second release.
E costs less day-one churn,
but the writer gets no repair from it,
and the commit path is the same for both because the wrapper runs no semantic-break check.

E above D2:
E's `38,405` findings are the issue's own class,
and `35,503` of them have a join the shipped guards allow;
D2's `107,302` are broader,
and `19,647` of them are before a conjunction whose status the owner has not decided,
so D2 asks a writer to change a style the owner may permit.

D2 above F1:
D2's predicate is MD1's own `120` bound with the permitted span,
parenthesis and dash edges removed,
so every finding is a clause that could fit on one line;
F1's predicate is a short next line,
which can be a line the rule wants to end at a boundary,
and `10,630` of its findings no shipped guard can remove.

F1 above D:
F1 is `72,913` findings,
D is `144,483`;
D includes the `36,423` parenthesis and `24,994` inline-span breaks MD8 permits,
so it asks for more joins of legal wraps than F1 does.

D above G:
D reports and can repair the unforced class in the text a writer is changing;
G only repairs the wrapper's existing class and prevents nothing.

G above C:
G removes the `4,077` wrapper breaks the issue was raised on;
C leaves them.

C above B:
B rewrites `8,109` files and still includes `2,997` emphasis endings that cycle,
for a class C leaves alone;
the status quo is cheaper than a repo-wide churn whose fix does not exist yet.

B above A:
B is A minus `47,372` findings,
a mix of forced breaks,
block-marker refusals and width refusals,
so B asks for fewer joins no command can clear.
It still holds some forced breaks and the `2,997` emphasis endings,
where A holds more of both.

## What the owner has to decide

1.
Which kinds of break does MD1 permit:
before a conjunction,
before a parenthesis,
between inline spans,
after a dash?
The owner needs the counts above (`19,647` conjunction,
`36,423` parenthesis,
`24,994` inline span,
`20` dash),
the examples quoted whole,
and MD8's sentence that a hard wrap may fall between whole inline spans.
This proposal does not answer it.

2.
Does the rule report the issue's class (F2),
the broader unforced clause (D2 or D),
every removable break (B),
or nothing (C)?
The owner needs the first-run counts in files,
the `58,788` forced breaks that no join clears,
the `3,383` emphasis endings in the population that contradict the existing rule,
and the record that this rule once held `171,262` findings and a hold.

3.
At what severity and with what fix timing:
`error` with the fix,
`warn` first with the fix,
or `warn` first with the fix withheld (E)?
The owner needs the exit statuses:
`error` exits `1` on any finding,
`warn` exits `0` and still applies `--fix` unless the fix is withheld,
and the tree already exits `1` on `14` fix-less findings in `package/git-policy/cli`.

4.
Which bound does a join use:
the rule's own,
which has none,
or MD1's `120`?
The owner needs the fact that the shipped join counts the virtual line the rule sees,
so a joined Rust doc line can be up to four characters over `120` in the host file,
and the alternative bounds the earlier round refused:
no bound,
and a host-aware bound.

5.
Does the owner's recorded hold,
"Put the semantic line break findings on hold",
still govern this rule?
The owner needs the record's date and the fact that no later record lifts or renews it.

6.
If C is chosen,
does the wrapper's existing `4,077` breaks get G's one-time repair,
and in which round?

## Shipped defects reproduced here

These are defects of shipped code,
not of this proposal,
and this round changed no line of them.
They bear on the options above.

### The boundary test does not climb emphasis delimiters

The installed `0.5.2-alpha.24` on a scratch copy of
`- **Naming every status.** The status filter is the next thing.`
writes:

```text
- **Naming every status.**
  The status filter is the next thing.
```

The rule wants the break after `.**`,
because `delimiter_tail` climbs `Strong`.
`ends_at_break_point` strips only bracket and quote closers,
so the survey and the removal model call the line a break at no boundary.
A removal join of that line is broken again on the next pass.
On `Aaa bbb, **cc dd.**` followed by `eee fff.`,
`--fix` inserts the break after `bbb,`,
joins `**cc dd.**` to the next line,
then breaks after `.**` again,
and settles at the break.
This is churn,
and under a removal rule the pair is unsatisfiable
until the boundary test climbs the same delimiters.

### The join does not ask about an author's line

The installed linter reports no finding in
`package/git-policy/cli/src/native/worktree_clone_add_route/mod.rs`,
and `--fix` on a copy changes no byte.
Lines 6 to 9 hold:

```text
//!  The registration half beside this folder owns the record family,
//! the
//!  managed lock's words,
//! the finder,
```

The lone `the` ends at no boundary and stays.
On `Aaa bbb, ccc ddd eee` / `the` / `fff ggg hhh.`,
`--fix` writes four lines:

```text
Aaa bbb,
ccc ddd eee
the
fff ggg hhh.
```

The join only looks at the fragment an inserted break cut.
This is why C leaves the issue's defect in place.

### The README's "never rewrites a line the author wrote"

`package/linter/monochromatic-lint/README.md` line 2623 says
"The fix never rewrites a line the author wrote."
On a copy of
`Owns the record family, the` / `managed lock's words, the finder.`,
the installed fix writes:

```text
Owns the record family,
the managed lock's words,
the finder.
```

The author's second line gained the fragment `the` at its start.
The sentence is already untrue of the shipped join,
and would be false of every joined line under B.

Line 2624 says
"Removing the deepest indent of a rustdoc run re-projects every line of that run".
`run_margin` (`processors_docs.rs`,
lines 116 to 131)
takes the smallest indent among the run's non-empty lines,
so the case that re-projects is removing or lowering the only line at the smallest indent.
This round read the source and did not run a removal,
because no removal fix exists.

## What this round built, and what it did not

Built:
the survey and its positive control,
the kept file list,
the whole output,
and this revision.
Changed nothing that ships:
no rule,
no fix,
no sweep,
no configuration,
no workflow.

The earlier round built the forward join,
where a fragment the fix's own break cut joins the clause that continues it,
and the indent a paragraph's first continuation line carries.
It measured `package/git-policy/cli/src` as it stood before the merge,
where the released fix left `834` doc lines of one or two words among the `4,613` it wrote,
and this fix leaves `732` among the `4,577` it writes.

Measured by this round:
the counts above,
the installed binary on `package/git-policy/cli` (`14` findings in `14` files,
exit `1`),
the exit statuses on a one-rule fixture (`warn` exits `0`,
`error` exits `1`,
and `--fix` applies at both),
and the reproductions in the section on shipped defects.
Inferred,
not run:
the removal counts,
the clause-width joins,
the Rust projection under a removal,
and the ten-pass fix loop's behaviour on a clause fix.
Unknown:
how many of the `38,405` F2 lines a sweep would change in files other branches hold,
and what the owner's kind list is.
