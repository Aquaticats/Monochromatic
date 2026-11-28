# Second-pass reads of released versions

Status:
unapproved and non-final.
Only the owner declares work complete.
This file is the package's record of the reads a later agent made of a released version,
what each found,
what a round changed for it,
and what was left.

A read recorded here is a second pass by another agent,
not an independent review:
the model that wrote it is the vendor of some of this package's authors,
and `AGENTS.md` rule `IRW` holds that only a reviewer from another vendor,
or the owner,
is independent.

## 0.5.2-alpha.24, read on 2026-11-27

The read covered the release commit `40710212ed`,
with the released binaries and fixture trees only:
nothing was built,
no test suite was run,
and no file of this repository was written.
Its counts were one Medium finding and nine Low findings.
It was written at 2026-11-27T23:30:37-05:00 in a scratch folder that expires a day after its last change,
so what this file uses from it is restated here,
reproduced from the release's own source and from runs of it.

Its binaries,
by sha256 of the x86_64 Linux archive's program,
so a later round can tell whether it measured the same code:

- `0.5.2-alpha.24`:
  `0903b914f51cb9ebe68cd8ac2fca53868f9008213f27cc768a1b406f77860994`.
- `0.5.2-alpha.22`:
  `e115b2bf54f8bd97c4c161d9f1e59510468a024db5ed3ed7c1b1b21535bef27b`.

Seven of its findings belonged to the round recorded here,
and the rest belonged to the round that fixes the Markdown line break:
its Lows 5,
7 and 8 and the third part of its Low 9 are that round's,
and they are under "The round on the Markdown line break" in this section.

### What the round changed

#### A dead line's edit is read once, and the list answers the repeat from an index

The read measured a list of dead key lines whose edited text reads as a `uses` line at 1,000,
2,000 and 3,500 lines:
`0.5.2-alpha.24` took 9.3,
23 and 41 times `0.5.2-alpha.22`'s time,
and doubling the lines multiplied the time by about 3.7,
which is the square and not the list.
Two costs made it.

The repeat check looked for the edited line by walking every entry of the list,
and built two identity texts for each of them,
once per dead key line.
It now asks an index of what the list holds,
built once when the list is read,
and a dead key line's own edit is read once:
the sentence about the edit and the check that no other dead line's edit gives the same line read that one answer.

A six-word `file-uses` text asks how long the longest name of the package folder is,
and the reader listed the whole folder for each of a line's two readings:
1,000 such lines over a folder of 5,000 entries cost 175 times `0.5.2-alpha.22`'s time.
The answer now belongs to the package folder a list is read against,
which is listed at most once,
and the reading starts at the first name that folder could hold,
so every name it builds is one it asks about.

Measured on 2026-11-28,
three interleaved rounds on the run's own CPU time,
each figure a ratio to `0.5.2-alpha.22`'s in the same round,
with the byte-identical copy of `0.5.2-alpha.24` as the spread of one binary against itself:

- dead key lines whose edit reads as a `uses` line:
  1,000 lines cost `0.5.2-alpha.24` 7.8 times (range 3.8 to 11.0) and this release 1.01 times (0.49 to 1.34);
  2,000 lines 20.9 (18.4 to 23.2) against 0.99 (0.91 to 1.16);
  3,500 lines 44.4 (42.8 to 45.7) against 1.33 (1.32 to 1.41);
  the copy gives 8.6 (3.8 to 10.7),
  19.2 (18.6 to 20.3) and 46.4 (46.2 to 46.9).
- dead key lines whose edited text is a six-word `file-uses` line,
  over a package folder of 5,000 entries:
  500 lines cost `0.5.2-alpha.24` 124 times (86 to 143) and this release 2.22 times (1.69 to 2.55);
  1,000 lines 201 (163 to 206) against 2.95 (2.84 to 3.05);
  the copy gives 122 (88 to 131) and 191 (177 to 218).

What remains on the six-word shape is one refusal sentence per line,
because the reader that answers whether the text reads is the one that writes the refusal,
so the cost is linear in the list and no longer per entry of the folder.

The controls are `a_list_of_dead_key_lines_that_read_as_uses_scans_no_other_line` and
`a_list_of_six_word_edits_lists_the_package_folder_once` in
`src/layout_allow_list_tests.rs`,
with `src/layout_allow_entries.rs`'s work counters for the names a reading builds and the folders it lists.
Each was shown able to fail by putting the old shape back:
the walk over the entries reads 40,000 of them for 200 lines,
the listing runs 100 times for 50 lines,
and the reading that offers every offset builds 10,000 names for a gap of 10,000 characters,
where the fixed reader reads none,
lists once,
and builds the two names it asks about.

#### A character a reader cannot see is named once, and every sentence says how many

The read's Low 1 found two defects in one sentence:
`any more.With` ran two sentences together,
and a character with no plain name was called `a character a reader cannot see (...)`,
where the sentence then said `where a reader cannot see it` in the same sentence.
Both are gone:
the sentence about what the edit leaves follows the one before it with a space,
and the phrase is the name and the code point alone.

The read's Low 4 found that only the take-out sentence said how many such characters a line holds.
The Delete note,
the help of a line whose folder the package does not hold,
and the note a refusal adds named the first and said nothing of the rest.
Each now says how many the line holds and names the edit for that many.

The read's Low 2 found that the file-name sentence's count covered the whole line,
so a character standing in a real folder's own name was counted,
and following the sentence named a folder that is not there.
The count for that edit is now the file names' characters alone,
and a line whose two file names hold one each says `the file names the rule prints`.

The read's Low 3 found that the note which leaves the take-out advice out,
because the character stands in the folder's own name,
did so only where the folder's text stands in the line as written.
A line that spells the folder's gap with a tab,
a no-break space or a doubled space was told to take the character out of the line.
The note now asks where the words that name the folder stand in the line,
which is the line's own text for every spelling.

The read's Low 6 found two dead key lines whose edits give one line,
each told that the line reads,
where following both is refused as a repeat.
The edited texts are compared with each other as well as with the lines as written,
so neither is told the line reads.

The controls are `a_dead_line_s_message_holds_no_two_words_run_together` (two whole sentences,
one of them asserted in full),
`a_sentence_about_three_characters_a_reader_cannot_see_names_every_one`,
`every_sentence_about_an_unseen_character_says_how_many_the_line_holds`,
`the_count_of_a_file_name_s_edit_covers_the_file_names_characters_only`,
`a_refusal_leaves_the_folder_s_own_character_out_of_its_advice_under_every_spelling_of_its_space`,
and `two_dead_key_lines_whose_edits_give_one_line_are_not_each_told_it_reads`.
Each was shown able to fail by putting the old sentence back,
and each region of `README.md` this round changed was rendered from its Markdown once.

The read's Low 9,
first two parts,
was that three of those controls could not fail:
the cost control's bound left two names of room,
the callback count could not see work done outside the callback,
and the sentence controls built counts of one and two only,
with no file-name test holding a second unseen character on its line.
The bound is now the package folder's own longest name,
the count of names built is compared with the count of names asked about,
the same line at twice the gap must ask about the same names,
and the sentence controls hold three characters and a folder character beside a file-name character.

### What this round did not do

- The read's Markdown findings (its Lows 5,
  7 and 8,
  and the third part of its Low 9) are the round
  `monochromatic-lint-alpha-24-read-the-line-break-fix-settles-in-one-pass-and-keeps-the-line-limit`'s.
- The read's claim 8 and its claim 9 (the list comparison) were argued or measured elsewhere and are not repeated here.
- A defect the round met while building a fixture for the sentence probes is left for a round of its own,
  because it is no part of these findings.
  A package whose `mod` line declares a module file through `#[path]`,
  where the file's own name holds a zero-width space,
  is reported as an unreadable module tree:
  the file is there and readable,
  and the finding reads `cannot read <package>/src/f/o<U+200B>ne.rs: No such file or directory (os error 2)`.
  The same package with a space in that file's name is read,
  and so is one that declares a folder whose name holds the same character.
  Reproduce it with a package holding `src/f/mod.rs` with
  `#[path = "o<U+200B>ne.rs"] pub mod one;`,
  the file itself at `src/f/o<U+200B>ne.rs`,
  and `rust/file-order-in-folder` selected for the package's Rust files.
  The read did not meet it,
  and this round neither fixed it nor could say whether an older release shares it.
- The reader still builds the refusal sentence of a text it only asks about,
  which is the linear cost the six-word shape keeps.
  Dropping it would need a reader that answers whether a text reads without writing what to do about it,
  and no measurement here says the sentence is what a person waits on.

### The round on the Markdown line break

The round
`monochromatic-lint-alpha-24-read-the-line-break-fix-settles-in-one-pass-and-keeps-the-line-limit`
took the join of `markdown/semantic-line-breaks`:
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

#### A join that spends a second pass, or the whole budget (Low 5)

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

#### The joined Rust doc line over the limit (Low 7)

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

#### The quote marker the README said was not written (Low 8)

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

#### Controls that would stay green (the third part of Low 9)

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

#### The three shipped defects of the line-break proposal

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

#### What this round did not change

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

#### Clippy findings that no finding of the read named

`lint:types` runs `cargo check --locked` and then
`cargo clippy --locked --all-targets -- -D warnings`,
and this crate denies `implicit_return`,
`shadow_same` and `shadow_unrelated`.
The round's own code drew five of them:
four closures that returned their tail expression,
and one test that named two results `remaining`.
Four more stood in lines of `4cf7cc74b8` that the read did not mention:
`markdown_prose_context.rs`,
the `all` closure of `continuation_indent`;
`markdown_semantic_breaks.rs`,
the `map` closure that adds each break's separator;
`markdown_fragment_joins.rs`,
a `written` rebound from `written`;
and a `&` before a `repeat` in a `format!` argument of the join tests.
A run of the same command with those four files at `4cf7cc74b8` reports three of them,
all in the lib:
`error: `written` is shadowed`,
and `error: missing `return` statement` twice.
The fourth is in a test file,
and that target did not compile in that run because the rest of the branch stayed in place,
so that one is read and not measured.
The task was red before this round and is green after it.
All nine are fixed,
because a check that stays red for any reason stops showing the next regression.

#### Checks run

Each once,
in this branch's worktree,
in its own capped scope:

- `mise run //package/linter/monochromatic-lint:lint:rustfmt`:
  `cargo fmt --check` printed its own command and nothing else,
  so no file needed formatting.
- `mise run //package/linter/monochromatic-lint:lint:types`:
  exit `0`,
  and both commands finished with no warning and no error.
- `mise run //package/linter/monochromatic-lint:test`:
  exit `0`,
  `1208 tests run: 1208 passed, 3 skipped`,
  and the output names every test this round added.
- `mise run //package/linter/monochromatic-lint:test:host`:
  exit `100`.
  The package defines that task as the closure check's host-only run
  (`cargo nextest run --test closure_binary --run-ignored only`),
  one test named `host_real_run_over_wg_quicker_exempt`,
  so it names no test of this round.
  It fails because the Dylint driver for the active toolchain and
  `dylint_driver` 6.4.0
  are not on this machine:
  "the Dylint driver for toolchain `nightly-x86_64-unknown-linux-gnu` ... is absent
  (cargo-dylint is 6.4.0),
  and `dylint_driver` 6.4.0 with its dependencies is not in the cargo cache,
  so `cargo dylint` would have to download it,
  which this linter does not do".
  The message names the one command that fills those caches,
  and it needs network access this round does not have.
  The closure check reads no line this round changed.
- The installed `monochromatic-lint 0.5.2-alpha.24` over this README,
  this file and the other documents the round changed:
  no finding,
  exit `0`.
- A render of every Markdown passage the round changed,
  through `pandoc --from gfm --to html`,
  read by eye.

#### Open for the owner

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
