# A survey of line breaks at no semantic boundary

Status:
unapproved and non-final.
This folder is a record and a harness,
not a rule,
a fix or a test the crate runs.
The owner declares completion.

The proposal it serves is
`doc/planning/monochromatic-lint-a-line-break-at-no-semantic-boundary-is-or-is-not-a-finding.md`.

## What it measures

A break is counted when all of these hold:

- The break sits inside a paragraph.
  A blank line,
  a fence,
  a heading,
  a table and a list marker end the walk.
- The line the break ends holds at least one visible character.
- The last visible character of that line,
  after any closing bracket or quote
  (the nine characters `" ' ) ] } ” ’ » ›`),
  is not one of `,` `.` `;` `:` `?` `!`.
  That is the set `markdown_break_points.rs` breaks after.

The harness reads Markdown and Rust doc comments through the linter's own
`MarkdownSource` and `processors::extract`,
so a paragraph and a doc comment are the ones the rule sees.
It resolves each path's configuration through `config_lookup` and `config_match`,
so a path the configuration ignores or turns the rule off for is netted out.
It reads each path's bytes with `git show <commit>:<path>`,
so a later edit to a record cannot move a count.

A line ending in two or more spaces or a backslash is a Markdown hard break.
The survey counts it apart and every option leaves it alone.

A line whose last visible character is a bracket or a quote is counted apart,
because it reads as a boundary of its own.

A line that ends a sentence inside emphasis or strong emphasis,
such as `word.**`,
is counted apart as the **emphasis ending**:
the rule climbs `Strong`,
`Emphasis` and `Delete` through `delimiter_tail`,
so it wants the break after the delimiter,
while `ends_at_break_point` strips only bracket and quote closers.
An emphasis ending is counted whether or not the shipped join would remove it,
so that a durable count subtracts every one of them.

A continuation line of a quotation keeps the `>` its container writes.
The survey strips the marker before it classes the line,
counts its words or measures its clause width,
so a quoted line is filed by its prose and not by the marker a reader does not see.
The `block-marker` class stays for a following line the shipped join refuses,
such as `#tag`,
which is not a heading and does continue the paragraph.

## The classes

Each break is classified by the first token of the next line's prose,
in this order:
`block-marker` (`>`,
`#`,
`|`,
a fence,
a list marker),
`parenthesis` (`(`),
`inline-span` (`` ` ``,
`*`,
`_`,
`~`,
`[`,
`!`),
`conjunction` (a fixed list of conjunctions and subordinators),
`dash` (the line before the break ends in `--`,
`—` or `–`),
and `ordinary` for the rest.
The list is a fixed list,
not a grammar;
whether these are the boundaries `AGENTS.md` rule MD1 means is the owner's question.

Each break also carries the width of its **clause**:
the run of lines from the previous line ending at a break point or a hard break
to the next one,
joined with single spaces,
after each line's own indentation,
plus the paragraph's continuation indent.
A clause at or over 120 characters is **forced**:
no single line may hold it under MD1's bound,
and it holds no punctuation to break at.

## The findings each predicate would draw

The survey counts the findings of six predicates,
each in its own population:

- `A`,
  every quiet non-hard break at no boundary.
- `B`,
  the `A` breaks the shipped join's guards allow.
- `D`,
  one finding per quiet clause under 120 characters holding no hard break
  and no emphasis ending.
- `D2`,
  `D` without a clause that holds a parenthesis or an inline-span edge,
  or a break whose previous line ends in a dash.
  The test reads every break of the clause,
not only its first.
- `F1`,
  a break whose following line holds one or two words.
- `F2`,
  a line of one or two words that itself ends at no boundary.

Each predicate also reports the class of its own breaks
and whether the break's clause is forced,
and each is counted per family of file,
so a reason for one predicate can use that predicate's own numbers.

## The removal count

No removal fix ships in the tree.
The removal count asks the shipped join,
`markdown_fragment_joins::fragment_join`,
about each break at no boundary.
The caller passes the line the break ends as the fragment.
The only guard lifted for this call is `FRAGMENT_WORDS`,
which the shipped join sets to two words;
the word limit is the one thing that stops the shipped join from being asked about a long line.
Every other guard is the shipped one:
the hard break,
the break-point ending,
the following block marker,
and the joined width against the same 120 characters the shipped join uses.
The count is a model of a fix that does not exist,
not a run of a shipped command.

## The reported paragraphs after the shipped fix

A paragraph the rule already reports can hold breaks at no boundary the rule says nothing about,
because the fix inserts breaks and joins only a fragment one of its own inserted breaks cut.
The survey therefore runs the shipped fix over a copy of every host file that holds a reported break,
and counts what the reported paragraphs still hold afterwards.

The copies live in the folder `SURVEY_M13_DIR` names,
never in the repository.
The fix is the installed binary named by `SURVEY_LINT_BIN`,
run as `--config <the harness's own one-rule configuration> --fix --no-ignore`,
once per batch of copies.
The harness writes that configuration itself,
into the same folder,
and it is kept beside this file as `reported-paragraphs-after-the-fix-config.jsonc`.
A control copy goes through the same invocation as the copies,
so the measurement can be seen to see a fix run:
its text is
`Aaa bbb, the` / `ccc ddd eee.`,
the unfixed copy holds one break at no boundary and the fixed copy holds none,
and the harness asserts both that the copy changed and that it now holds zero.

One file of the run recorded in `output.txt` kept findings after the fix,
which the survey measured separately:
`65` findings remained across all the copies,
`64` of them in one file,
and one in the old control copy,
which that run's harness wrote but never passed to the fix.
The kept harness corrects that,
so its control can fail on an unfixed copy;
the recorded figure rests on all `222` copies changing bytes,
and on the control measurement kept beside this survey.
Part of the `10,028` remaining breaks may therefore sit in paragraphs the fix leaves reported,
and the population after a full fix run is at most `220,235`.

The reported breaks that remain are the post-fix count of each file
less the breaks that were already quiet in it.
That subtraction is sound in one direction only:
a join is attached to a break one pass plans,
so a fix never removes a break outside a paragraph the rule reports,
and that makes the remainders a true count of what the fix left;
but an inserted break falls after a break-point character,
and at this commit `ends_at_break_point` strips only the closing brackets and quotes,
so a break the fix inserts after an emphasis delimiter,
as `word.**` gets,
is counted at no boundary here even though the rule wrote it on purpose.
So the `10,028` is an upper bound on what a full fix run leaves,
and the `1,524` is a floor on what it joins.
`figures-the-survey-does-not-count/emphasis-boundary-reproduced.txt` is that shape.

## What is kept here

- `files.txt`:
  every tracked `.md`,
  `.mdx` and `.rs` path at the recorded commit,
  from `git ls-files '*.md' '*.mdx' '*.rs' | LC_ALL=C sort`.
- `survey_line_breaks_at_no_semantic_boundary_tests.rs.keep`:
  the harness.
  The `.keep` suffix keeps a build from compiling it
  and the repository linter from reading it as Rust.
- `command.txt`:
  the command,
  the three temporary changes,
  and the environment the run had,
  all as this run made them.
- `output.txt`:
  the whole output of the run named in its first lines,
  including the positive control and every count.
- `reported-paragraphs-after-the-fix-config.jsonc`:
  the one-rule configuration the after-the-fix measurement used,
  copied from the harness's own constant.
- `repository-configuration-on-package-git-policy-cli.jsonl`:
  what the repository's own configuration reports on `package/git-policy/cli`
  at the recorded commit,
which is the figure the proposal quotes for the tree not being green.
- `figures-the-survey-does-not-count/`:
  the command and output of every figure this proposal quotes that the survey does not count itself,
  and a `README.md` naming each one.
- `README.md`:
  this file.

`output.txt` names the root,
the list,
the commit and every path the configuration ignored.
A later rerun at a later commit will differ by the files that changed.

## How to run it again

The run recorded in `output.txt` was made at commit
`bc5b4ca2bae7fc5bb80475448c31d4d57f51f83c` (2026-11-28T03:19:11-05:00),
in the worktree the round was started in,
`.claude/worktrees/pi-line-break-proposal-revised-against-its-second-gate-read-with-its-survey-run-again`
of the repository,
and its `SURVEY_COMMIT` line names the same commit.
The join and the boundary test changed after it
(`fc679b9392`,
committed 2026-11-28T03:02:49-05:00 and merged after the survey commit):
the join counts the host line,
and the boundary test climbs `*` `_` and `~`.
The harness in this folder is the later form,
and it differs from the harness the recorded run used in five hunks:
three for the after-the-fix control
(the control text,
the count it expects,
and passing the control copy through the same fix run as the copies)
and two for the host prefix
(the join is asked with the host's own prefix,
and a Rust doc comment carries it).
The recorded run used this file as `bc5b4ca2ba` holds it,
which is what `git show bc5b4ca2ba:package/linter/monochromatic-lint/doc/a-survey-of-line-breaks-at-no-semantic-boundary/survey_line_breaks_at_no_semantic_boundary_tests.rs.keep` prints,
and a rerun at the recorded commit needs that form,
not the kept one,
which needs this branch's source to compile.
The survey was not rerun,
and every count here is the commit's it names.
Its list is `files.txt` at that commit.
A rerun must read a worktree at that commit,
because a listed path that the commit does not hold is tallied as a parse failure,
and the harness refuses to report on an empty commit or an empty survey.

The harness must sit inside the crate to call the shipped join.
Copy it in and wire it temporarily;
none of these three changes is committed.
The run below was made in the worktree this round was started in,
at the commit above;
a rerun needs a worktree at the same commit,
and this line makes one,
under `.claude/worktrees/` as `AGENTS.md` rule WTU requires of every worktree of this repository:

```sh
# from the repository root
git worktree add .claude/worktrees/line-break-survey-rerun bc5b4ca2bae7fc5bb80475448c31d4d57f51f83c
```

1.
   Copy the kept harness to the crate source:

   ```sh
   # from package/linter/monochromatic-lint
   cp doc/a-survey-of-line-breaks-at-no-semantic-boundary/survey_line_breaks_at_no_semantic_boundary_tests.rs.keep \
     src/markdown_fragment_joins_survey_tests.rs
   ```

1.
   Append this module declaration to `src/markdown_fragment_joins.rs`.
   The comments are `//` and not `//!`,
   because an inner doc comment after the file's items does not compile;
   the run below compiled this text as appended.

   ```text
   // Wire the survey test in for one run.
   // Keep the survey test out of a release build.
   #[cfg(test)]
   #[path = "markdown_fragment_joins_survey_tests.rs"]
   mod survey_tests;
   ```

1.
   Change this constant in `src/markdown_fragment_joins.rs`
   from `2` to `usize::MAX`.
   This is the only lifted guard,
   and this edit is the reason the removal count is a model:

   ```rust
   //! Read the shipped fragment limit.
   /// The most words a shipped join accepts from a fragment.
   const FRAGMENT_WORDS: usize = 2;
   ```

1.
   Run the survey from the package folder,
   with the list,
   the worktree root,
   the copies folder and the installed binary named,
   inside a scope with the limits `command.txt` records.
   The run took `1,297.955` seconds and peaked under the limits below;
   a rerun that has a whole host to itself may raise the memory and process caps,
   never the slice:

   ```sh
   # from package/linter/monochromatic-lint
   systemd-run --user --scope --slice=agent-work.slice \
     --property=MemoryMax=8G --property=CPUQuota=400% --property=TasksMax=256 \
     --property=IOWeight=1 --property=RuntimeMaxSec=3600 \
     env \
       SURVEY_LIST="$PWD/doc/a-survey-of-line-breaks-at-no-semantic-boundary/files.txt" \
       SURVEY_ROOT="<the worktree the list came from>" \
       SURVEY_M13_DIR="<a folder outside the repository for the fix copies>" \
       SURVEY_LINT_BIN=/home/user/Monochromatic/.cache/toolchain-bin/monochromatic-lint \
       cargo nextest run --locked --lib --no-capture survey_
   ```

   Every test whose name starts with `survey_` runs.
   `survey_probe_sees_every_class_on_its_fixture` is the positive control:
   it fails unless the survey sees each class on its own fixture,
   the `block-marker` class included.
   `survey_classes_a_quoted_line_after_its_marker`,
   `survey_d2_leaves_a_clause_with_a_later_parenthesis_edge` and
   `survey_counts_a_reported_paragraph_apart` are the three controls for the splits the predicates lean on;
   each asserts exact numbers on its fixture.

1.
   Undo the two edits and delete `src/markdown_fragment_joins_survey_tests.rs`.
   `git status --short` must name none of them before a commit.
   The run recorded here took 1,297.955 seconds for its five tests.

## Limits

- The boundary test is `ends_at_break_point`,
  which treats a line ending `e.g.`,
  `etc.` or `...` as a boundary
  although `break_offsets` never breaks after an abbreviation or an ellipsis.
  The survey therefore undercounts breaks at no boundary by that small class.
- The boundary test does not climb `*`,
  `_` or `~`,
  so the emphasis ending is counted apart rather than as a boundary.
  A report that includes it must settle the conflict with the rule first.
  Update,
  2026-11-28:
  the shipped boundary test now climbs those three delimiters
  (the fix for the emphasis-ended fragment of the alpha.24 second pass,
  `package/linter/monochromatic-lint/doc/second-pass-reads-of-released-versions.md`),
  so this harness counts an emphasis ending as a boundary on a rerun:
  the `emphasis` class and every count apart from it reach zero,
  and the emphasis ending the classes below describe is no longer a break at no boundary.
  The harness is updated with it:
  the join is asked with the host prefix,
  and a rustdoc comment's context carries the prefix of its projection,
  because the joined width the shipped join now checks is the host line's.
- The removal count uses the shipped join's width,
  which adds the paragraph's continuation indent
  and,
  since 2026-11-28,
  the characters the host writes before the line,
  not the joined line's own indent.
  A removal fix that keeps another indent would measure a few characters differently.
- The survey does not run a removal fix,
  because none exists.
  It measures each break pair by pair;
  a run that joins a whole run of lines at once,
  and the Rust doc-comment projection,
  are not measured here.
- The after-the-fix measurement runs the installed binary,
  which is a release of the crate at the recorded commit,
  and not a build of that commit.
  A later release that changes the join changes the figure,
  and the commit the figure holds at is the one `output.txt` names.
- The list is the tracked corpus at its recorded commit,
  before some of this folder's own records were added,
  so those records are not in the counts.
