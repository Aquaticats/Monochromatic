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

## The classes

Each break is classified by the first token of the next line,
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
  the command and the three temporary source edits,
  which were reverted after the run.
- `output.txt`:
  the whole output of the run named in its first lines,
  including the positive control and every count.
- `README.md`:
  this file.

`output.txt` names the root,
the list,
the commit and every path the configuration ignored.
A later rerun at a later commit will differ by the files that changed.

## How to run it again

The harness must sit inside the crate to call the shipped join.
Copy it in and wire it temporarily;
none of these edits is committed.

1. Copy the kept harness to the crate source:

   ```sh
   # from package/linter/monochromatic-lint
   cp doc/a-survey-of-line-breaks-at-no-semantic-boundary/survey_line_breaks_at_no_semantic_boundary_tests.rs.keep \
     src/markdown_fragment_joins_survey_tests.rs
   ```

1. Append this module declaration to `src/markdown_fragment_joins.rs`:

   ```rust
   //! Wire the survey test in for one run.
   /// Keep the survey test out of a release build.
   #[cfg(test)]
   #[path = "markdown_fragment_joins_survey_tests.rs"]
   mod survey_tests;
   ```

1. Change this constant in `src/markdown_fragment_joins.rs`
   from `2` to `usize::MAX`.
   This is the only lifted guard,
   and this edit is the reason the count is a model:

   ```rust
   //! Read the shipped fragment limit.
   /// The most words a shipped join accepts from a fragment.
   const FRAGMENT_WORDS: usize = 2;
   ```

1. Run the probe and the survey from the package folder,
   with the list and the worktree root named:

   ```sh
   # from package/linter/monochromatic-lint
   SURVEY_LIST="$PWD/doc/a-survey-of-line-breaks-at-no-semantic-boundary/files.txt" \
     SURVEY_ROOT="$(git rev-parse --show-toplevel)" \
     cargo nextest run --locked --lib --no-capture survey_
   ```

   The same command also runs `survey_probe_sees_every_class_on_its_fixture`.
   That test is the positive control:
   it fails unless the survey sees each class on its own fixture.
   Its `PROBE` line is the evidence that the counters can see a class at all.

1. Undo the two temporary edits and delete `src/markdown_fragment_joins_survey_tests.rs`.
   `git status --short` must name none of them before a commit.

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
- The removal count uses the shipped join's width,
  which adds the paragraph's continuation indent,
  not the joined line's own indent.
  A removal fix that keeps another indent would measure a few characters differently.
- The survey does not run a removal fix,
  because none exists.
  It measures each break pair by pair;
  a run that joins a whole run of lines at once,
  and the Rust doc-comment projection,
  are not measured here.
- The list is the tracked corpus at its recorded commit,
  before this folder and the revised
  proposal were added,
  so the records themselves are not in the counts.
