# Second-pass reads of this package's releases

Status:
unapproved and non-final.
The Pi run of 2026-11-28 wrote this file,
the owner declares completion.

## What this file holds

A second pass is a reader told to find problems in a release it did not write.
This file records each such read of this package,
what its findings were,
how each was reproduced,
which of the README and the code changed for it,
and what a round left alone.
A finding is reproduced here from the release's own source and from runs of it,
so this file stands once the reader's own scratch folder is gone.

## The read of `0.5.2-alpha.24`

The read is
`/home/user/temp/agent/monochromatic-lint-alpha-24-release-review-2026-11-27/keep/review.md`,
written at 2026-11-27T23:30:37-05:00 by Claude Sonnet,
the vendor of some of the authors,
so it is a same-vendor second pass and not an independent review.
It read release commit `40710212ed`.
Its folder expires a day after its last change,
so every quotation below is restated here.

The read found no High,
one Medium and nine Lows.
**The Medium and Lows 1 to 4 and 6 are the allow-list reader's**,
and the round
`monochromatic-lint-alpha-24-read-the-allow-list-reader-costs-one-pass-and-its-sentences-name-the-right-edit`
owns them.
This round owns the join of `markdown/semantic-line-breaks`:
the read's Low 5,
Low 7 and Low 8,
the third part of its Low 9,
and the three shipped defects of the line-break proposal.

The reproduction used a debug build of this branch at `4cf7cc74b8`,
whose `src` is byte-identical to release commit `40710212ed`
(`git diff --stat 40710212ed HEAD -- package/linter/monochromatic-lint/src` is empty),
over fixture files in a scratch folder with one configuration that selects
`markdown/semantic-line-breaks` at error severity for `**/*.md` and `**/*.rs`.
Each fixture was checked once,
fixed twice,
and the debug note `fix loop changed the source in N pass(es)` counted.
Every claim below marked as a run is from that invocation.

### A join that spends a second pass, or the whole budget (Low 5)

The read's finding,
quoted:

> One pass did not settle in twelve shapes:
> eight needed two to six changing passes where alpha.23 needed one,
> and four chains of fifteen lines hit the loop's ten-pass limit,
> left a finding after the first run,
> and changed on the second.

Reproduced,
three shapes of the read on the built executable:

- `One two, *held,*` followed by `four five.`:
  the first `--fix` exited `0` after **2 changing passes**,
  and the second changed nothing.
  The joined line `One two, *held,* four five.` is broken again on the next pass,
  which is why the first pass could not be the last.
- `clause one, x, y` followed by `next line here.`:
  **2 changing passes**,
  the second changing nothing.
  Both breaks sit on one line,
  so their fragments answer one and the same join range,
  and the grouped editor refuses a whole fix that starts where an accepted one starts.
- Fifteen lines,
  each ending `*x,*`:
  **10 changing passes,
  stopped as `Limit`**,
  the run exited `1` with one finding still reported on line 11,
  and the second `--fix` needed **5 more changing passes** before it settled.

Two mechanisms in the source,
read at `40710212ed`:

- `src/markdown_break_points.rs`,
  `ends_at_break_point`,
  stripped the closing brackets and quotes but not `*`,
  `_` or `~`,
  while `src/markdown_prose_context.rs`,
  `delimiter_tail`,
  reads a text node inside those delimiters through the closing one.
  So the rule wants a line to end after `*held,*` and the join did not know it.
- `src/markdown_fragment_joins.rs`,
  `fragment_joins`,
  asked each fragment on its own,
  and two fragments of one line answer the same line ending.
  `src/edits.rs`,
  `conflicts_before`,
  then rejects the second whole fix,
  because its edit starts where the accepted one starts.

Changed,
in the code:

- `ends_at_break_point` now trims the emphasis closers as well as the brackets and quotes,
  so a fragment that ends inside `*`,
  `_` or `~` stays alone.
- `fragment_joins` answers one join per line ending:
  the first fragment keeps it and a later fragment that answers the same range gets none,
  so the pass applies both breaks and the one join together.

The joins module's tests gained
`an_emphasis_closed_break_point_ends_its_line`
and `two_fragments_ending_at_one_line_ending_give_one_join`,
and the rule's tests gained
`a_fragment_ending_inside_closing_emphasis_stays_and_the_fix_settles`,
`two_breaks_sharing_a_line_ending_settle_in_one_pass`
and the changing-pass control
`both_shapes_change_one_pass_and_no_more`,
which runs the fix loop itself and requires one changed pass on each shape and on both in one document.
The control's red is the old count of 2 and the old `Limit`:
before the fix the same fixtures give the numbers in the list above.

After the fix,
the same three fixtures changed **1 pass** each,
and the second `--fix` changed nothing in any of them.
The fifteen-line chain settles in one pass with no finding left.

### The joined Rust doc line over the limit (Low 7)

The read's finding,
quoted:

> The width the join computes is the projection's,
> so the excess is three at column 0 and grows with the host indent (135 at twelve spaces).

Reproduced on three Rust files,
each a doc comment whose fragment is `the` and whose following word is 115 `x` characters,
so the projection is 119 characters wide and the join is accepted:

- the comment at column zero gives a joined host line of **123** characters,
- four spaces of module indentation give **127**,
- twelve spaces give **135**.
  At those two indents the following word's own line is already 123 and 131 characters wide,
  and no fix writes it:
  it is the author's line and the join would only have added the fragment to it.

After the fix no joined line is written in any of the three,
and each file keeps only the lines its author wrote.

The host file is what MD1 bounds,
and the README already said the joined line stays under 120 characters,
so the code was the record that was wrong.

Changed,
in the code:

- `MarkdownSource` carries `host_prefix_chars`,
  the characters the host writes before each line of a projection,
  which a physical Markdown file leaves at zero.
- `VirtualSource::host_prefix_chars` answers the widest prefix of the mapping's lines,
  which is the comment marker and the host indent of a Rust doc comment,
  and `run_check.rs` records it on the virtual document.
- `fragment_join` counts it in the width beside the indent the fix writes,
  so the bound is the line a reader of the host meets.

The tests gained `the_indent_the_fix_writes_is_part_of_the_width`,
`the_host_prefix_is_part_of_the_width`,
`a_joined_line_exactly_at_the_limit_is_declined`,
and the end-to-end `a_joined_doc_line_stays_under_the_limit_in_its_host_file`,
which fixes a Rust doc comment at three host indents and requires that no line of the result passes the limit,
and that the same width one character further is declined.

The way not taken keeps the code and corrects the README's number,
and it costs the rule writing lines over the limit it enforces,
which the owner's own bound in `AGENTS.md` rule MD1 forbids,
so this round took the brief's pick:
the width is the host line's.
The veto stays open.
`doc/planning/monochromatic-lint-a-line-break-at-no-semantic-boundary-is-or-is-not-a-finding.md`,
question 4,
records that an earlier round refused a host-aware bound and asks the owner which bound to use;
that note now says what the shipped code does and that the owner has not been asked since.

### The quote marker the README said was not written (Low 8)

The read's finding,
quoted:

> README line 148 says "A line break inside a blockquote carries no `>`",
> and both releases write `> ` on the line they insert.

Reproduced:
`> One two, so we` followed by `stop here.` fixes to
`> One two,` and `> so we stop here.` in one pass.

The package's own records support the code:

- `src/markdown_prose_context.rs`,
  `continuation_prefix`,
  copies `>` and explains that a line without the marker is only a lazy paragraph continuation
  that a later edit can move out of the quotation.
- `src/markdown_semantic_breaks_tests.rs`,
  `a_break_in_a_quotation_keeps_the_marker_of_every_depth`,
  requires the marker on every inserted line at every depth.
- `src/run_command_whitespace_fix_tests.rs`,
  `a_blockquote_break_keeps_the_marker_at_the_same_depth`,
  names the reason:
  the raw text keeps the quotation,
  and a later edit cannot move the line out of it.
- The code has copied the marker since `3af7894e01`,
  2026-10-04T20:35:38-04:00.

Changed,
in the README,
which was the record that was wrong:

- The bullet that said a blockquote break carries no `>` leaves the section
  "What `--fix` leaves alone",
  because the fix is not withheld there.
- The join paragraph states that a line broken inside a block quote is written with the paragraph's own markers,
  and why laziness alone is not enough.
- The `whitespace` fix class says what else a written line takes:
  the Rust comment marker,
  and the quote markers of the container the paragraph sits in,
  both of them container syntax and not tokens of the text.
  So "the fix changes whitespace only" is stated with that exception.

The same claim stood in a handover record,
`doc/handover/unified-linter-cutover.md`,
and it keeps its text with an in-place note giving what the tool does and since when.

### Controls that would stay green (the third part of Low 9)

The read,
quoted:

> every fragment in the join tests is one word,
> so a limit of two words written as one stays green;
> the too-long line in the tests is 124 wide,
> so `>=` written `>` stays green;
> the indent is the literal 0 in every call;
> a single trailing space is joined and held by no test.

All four are reproduced by reading the tests at `40710212ed`,
and each is closed by an assertion that the edit it names turns red:

- `two_words_are_a_fragment_and_three_are_a_clause` joins a two-word fragment and leaves a three-word one,
  so `FRAGMENT_WORDS = 1` fails it.
- `a_joined_line_exactly_at_the_limit_is_declined` puts a joined line of exactly the limit and one of one less,
  so `width >= JOINED_LINE_LIMIT` written `>` fails it.
- `the_indent_the_fix_writes_is_part_of_the_width` and `the_host_prefix_is_part_of_the_width` ask the join
  with a non-zero indent and a non-zero host prefix,
  so dropping either from the width fails them.
- `one_trailing_space_still_joins` joins a fragment whose line ends in one space,
  so `HARD_BREAK_SPACES = 1` fails it.

### The three shipped defects of the line-break proposal

`doc/planning/monochromatic-lint-a-line-break-at-no-semantic-boundary-is-or-is-not-a-finding.md`,
section "Shipped defects reproduced here",
names three defects of the shipped code.
This round answers them and decides none of that proposal's options.

**The boundary test does not climb emphasis delimiters** is the mechanism of Low 5 above,
and the fix is the same one.

**The join does not ask about an author's line** reproduced on a copy of
`package/git-policy/cli/src/native/worktree_clone_add_route/mod.rs`:
the check reports nothing,
and `--fix` changes no byte.
Its seventh line ends at `family,` and its eighth is the word `the` alone,
and no break of this pass cuts that word,
so no join reaches it.
`Aaa bbb, ccc ddd eee` followed by `the` and `fff ggg hhh.` fixes to four lines with `the` still alone.
The code and the README agree on that restriction,
which the README states as one fragment per break of the same pass,
and the tests hold it:
`a_fence_beside_a_fragment_keeps_its_bytes` is the same rule with a fence below.
What the README's first sentence promised was wider than the mechanism,
so it now names the restriction in that sentence instead of only in the next.
Joining an author's lone line is what the proposal's option C needs,
and that is the owner's to decide.

**The README's "never rewrites a line the author wrote"** reproduced on
`Owns the record family, the` followed by `managed lock's words, the finder.`:
the fix writes three lines and the author's second line gains `the` at its start.
The code is right,
because joining is the point of the fragment it cuts,
and the README sentence is not true of it.
The README now says that every word stays where it is,
that the line a fragment joins to gains that fragment at its start,
and that the fragment's own line ending becomes the space between them.
A test's doc comment said the same untrue thing about author-written lines,
and now says it about its own fixture,
whose tail holds three words and so carries no join.

## What this round did not change

- The allow-list reader's findings,
  the read's Medium 1 and Lows 1 to 4 and 6.
  Another round owns them.
- Any option of the line-break proposal:
  what the rule reports for a line break at no semantic boundary stays the owner's to decide,
  and this round changed only what is a defect under the rule as it stands.
- The ten-pass budget of the fix loop,
  which no shape of this round reached after the fix.
- The width's refusal of a fragment whose following line is already long,
  which is the same bound and not a new one.

## Checks run

Each once,
in this branch's worktree:

- `mise run //package/linter/monochromatic-lint:lint:rustfmt`.
- `mise run //package/linter/monochromatic-lint:test:host`,
  which the package defines as the closure check's host-only run,
  and `mise run //package/linter/monochromatic-lint:test`,
  which names this package's unit tests.
- `mise run //package/linter/monochromatic-lint:lint:types`,
  the task that runs Clippy with `-D warnings` over every target.
- The installed `monochromatic-lint` over this README,
  this file and the other documents the round changed,
  with no finding on a line the round wrote.
- A render of every Markdown passage the round changed,
  through `pandoc --from gfm --to html`,
  read by eye.

## Open for the owner

- The bound a join uses.
  The shipped code now counts the host line,
  including the marker and the host indent.
  An earlier round refused that bound,
  and the proposal's question 4 asks for the choice.
  The way not taken is the projection's width with a corrected README,
  which costs a rule that writes lines over the limit it enforces.
- Whether the join should reach a line the author wrote alone.
  Today it reaches only a fragment a break of the same pass cut,
  which is why the wrapper's lone `the` stays.
