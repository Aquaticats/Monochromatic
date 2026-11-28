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
decision 3,
records that an earlier round refused a host-aware bound and asks the owner for a veto;
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
  and the proposal's decision 3 asks for the veto.
  The way not taken is the projection's width with a corrected README,
  which costs a rule that writes lines over the limit it enforces.
- Whether the join should reach a line the author wrote alone.
  Today it reaches only a fragment a break of the same pass cut,
  which is why the wrapper's lone `the` stays.

## 0.5.2-alpha.25, read on 2026-11-28

The read covered release commit `6f52a414fc`
with the released binaries and fixture trees only:
nothing was built,
no test suite was run,
and no file of this repository was written.
Its counts were no High finding,
no Medium finding,
and eight Low ones.
Three of those findings and one change the read found no claim for belong to this round;
the other five findings and two controls belong to the round that answers the allow-list reader,
and the closure rule of issue 6667 belongs to neither.

The read was written at `2026-11-28T06:22:20-05:00`
in a scratch folder that expires a day after its last change,
so what this section uses from it is restated here
from the release's own source and from runs of both released binaries.

Its binary,
by sha256 of the x86_64 Linux archive's program,
so a later round can tell whether it measured the same code:

- `0.5.2-alpha.25`:
  `34ca8f26d618be2cbfce78bfd209aec0ae2d1c2faf7f00f4144374df37de485a`.
- `0.5.2-alpha.24`,
  the comparison arm every reproduction of the Markdown part below was run against:
  `0903b914f51cb9ebe68cd8ac2fca53868f9008213f27cc768a1b406f77860994`.

Special characters are written `<U+XXXX>` in this section and `<BOM>` for the byte order mark,
and the linter's own spelling in a sentence it prints is `\u{...}`.
`git diff --stat 6f52a414fc main -- package/linter/monochromatic-lint/src` was empty when the allow-list round began,
so the released binary is the tree that round started from.

### The Markdown line-break findings

#### A delimiter that closes nothing is not the rule's line end (the read's Low 3)

The read's finding,
quoted:

> The test for a fragment that "ends at a break point" trims characters of a text
> (`*`,
> `_`,
> `~`,
> brackets,
> quotes) and not delimiters of a tree,
> so it declines seven joins that `0.5.2-alpha.24` made and that nothing would break again.

Reproduced on both released binaries,
one file per shape,
each `One two, <fragment>` followed by `four five.`:

- `(*held,*)`,
  `"*held,*"`,
  `held,*`,
  `held,__`,
  `snake_case,_`,
  `x*,*` and `**held**,**`:
  `0.5.2-alpha.24` wrote the fragment onto the following line in one changing pass,
  and `0.5.2-alpha.25` kept the author's line ending,
  also in one pass.
- `*held,*`,
  `**held,**`,
  `_held,_`,
  `~~held,~~`,
  `***held,***`,
  `**_held,_**`,
  `*held;*`,
  `*held.*` and `~held,~`:
  both releases wrote the same text,
  `0.5.2-alpha.24` in two changing passes and `0.5.2-alpha.25` in one.
- A second `--fix` changed no byte in any of the sixteen shapes on either release.

Two mechanisms in the source,
read at `6f52a414fc`:

- `src/markdown_break_points.rs`,
  `ends_at_break_point`,
  trimmed `*`,
  `_`,
  `~`,
  the brackets and the quotes from the end of a fragment
  and called the fragment an ending whatever stood there,
  so a star that closes nothing read as an emphasis closer.
- `src/markdown_break_points.rs`,
  `break_offsets`,
  and `src/markdown_prose_context.rs`,
  `delimiter_tail`,
  decide that question with the parsed tree:
  the break moves past the brackets and quotes its own text node ends at
  and past an emphasis,
  a strong emphasis or a strikethrough that ends there,
  and then needs a space or a tab at that position.

Changed,
in the code:

- `ends_at_break_point` takes the parsed source,
  the paragraph and the fragment's byte range,
  finds the last break point of that range
  and asks `break_position` where the rule's own break lands:
  the text node's trailing bracket and quote run,
  then the delimiter chain that ends there.
  It answers yes only when that position reaches the fragment's end.
- `fragment_joins` and `fragment_join` take the parsed source and the paragraph
  instead of raw text and a host-prefix number,
  because the tree answers where the rule breaks
  and the width is the source's own prefix.
- The `*`,
  `_` and `~` trim disappears with no replacement character list:
  a delimiter counts only where the tree closes a node.

The two lists are held at both levels:

- `src/markdown_fragment_joins_tests.rs`,
  `an_emphasis_closed_break_point_ends_its_line` holds the nine shapes that stay,
  and `a_delimiter_that_closes_nothing_leaves_the_fragment_joinable` holds the seven that join.
  The fixtures are parsed now,
  so the answer under test is the rule's own tree answer.
- `src/markdown_semantic_breaks_tests.rs`,
  `a_fragment_ending_inside_closing_emphasis_stays_and_the_fix_settles` holds the nine
  with one pass and no finding left,
  and `a_delimiter_that_closes_nothing_joins_in_one_pass` requires exactly one changing pass
  and no finding left on each of the seven.

Each control was shown able to fail by putting the old code back:

- The `0.5.2-alpha.25` character trim makes both join-list controls fail on `(*held,*)`:
  `"One two, (*held,*)\nfour five.\n"` against `"One two, (*held,*) four five.\n"`,
  and `"One two,\n(*held,*)\nfour five.\n"` against `"One two,\n(*held,*) four five.\n"`.
- A body answering `false` for every fragment drops the keep list:
  `an_emphasis_closed_break_point_ends_its_line` fails on `*held,*`,
  and `a_fragment_ending_inside_closing_emphasis_stays_and_the_fix_settles` fails at its own check.

After the change a joined fragment keeps the author's line ending
until the join removes it,
and the rule reports nothing on the joined line,
which is why one pass settles.

#### The line limit bounds joins, and a break may write a longer line (the read's Low 4)

The read's finding,
quoted:

> A break the fix writes inside a deep quote or list can be 120 characters or more,
> because the continuation prefix is written before a tail that was a lazy line;
> only a join bounds a width,
> and the release note says the fix keeps the line limit.

Reproduced on both released binaries,
which write the same bytes in all three shapes,
each in one changing pass with a second `--fix` changing nothing:

- `> > > Start of it` followed by `c,` and 116 characters:
  the fix writes `> > > ` before that tail,
  a line of **122** characters.
- a list three levels deep,
  `- a`,
  `  - b`,
  `    - c start`,
  then `d,` and 116 characters:
  the fix writes six spaces before the tail,
  a line of **122**.
- an ordered item numbered `100.`,
  `100. start` then `c,` and 115 characters:
  the fix writes five spaces,
  a line of **120**.

The code is right and the release note was the sentence that overreached:
`src/markdown_semantic_breaks.rs` writes the paragraph's continuation prefix before the rest of
the author's line,
and only `src/markdown_fragment_joins.rs`,
`fragment_join`,
compares a width with the limit.
Changed,
in `README.md`:

- The join paragraph says the limit bounds joins,
  and says what a break may write,
  with these three shapes as its example:
  a third-level quote writes `> > > ` before a tail of 119 characters,
  a list item two levels deep writes six spaces before that tail,
  and an item numbered `100.` writes five spaces before a tail of 118.
- It says the tail is the author's own clause,
  so no count shortens it,
  and the rule reports nothing on the line it wrote.

Whether such a tail could be written as a lazy line,
with no prefix at all,
was measured rather than argued:
`pandoc --from gfm --to html` renders each marked output of the three shapes and its lazy variant
to byte-identical HTML,
and each marked output renders like its input,
so laziness changes nothing a reader sees today.
What it would take is a `continuation_prefix` that writes nothing for a break whose tail was the
author's own lazy line,
and it would cost the raw structure:
a blank line planted before the tail stays inside the quotation and the list item in the marked
form,
and moves the tail to a top-level paragraph in the lazy one.
That fragility is what `src/markdown_prose_context.rs`,
`continuation_prefix`,
and the README's own paragraph about laziness already name,
so this is a suggestion to weigh and no change of this round.

#### Which prefix of a comment run a join counts (a change no claim accounted for)

The read's section "Changes that no claim accounts for",
quoted:

> The width of a join counts the widest prefix of the doc comment run (`processors.rs` lines 96 to 102),
> so a run that holds lines of different indent declines joins the line's own prefix would allow.
> The README says only that the marker and indentation count.

Reproduced on both released binaries with a rustdoc run of four lines:
`/// One two, so we`,
then `/// ` and 109 characters,
then `    /// Third four, so we`,
then `    /// ` and 87 characters,
then `pub struct A;`.
A break is planned after `two,`,
and the fragment's own line carries four characters of prefix
while another line of the same run carries seven:

- `0.5.2-alpha.24` joins the column-zero fragment,
  and the joined line is **119** characters.
- `0.5.2-alpha.25` declines it,
  leaving `/// so we` alone,
  and joins the deeper fragment instead,
  whose joined line is 101 characters.

So the shipped code counts the widest prefix of the run,
which is a deliberate bound:
`src/processors.rs`,
`host_prefix_chars`,
whose doc comment says the widest prefix is a bound no written line can pass.
The README was the record that was incomplete.
Changed,
in `README.md`:

- The width counts the widest prefix the host writes before the lines of the comment run,
  not the joined line's own.
- A run whose lines sit at two depths declines a join its own line's prefix would allow,
  and the line that absorbs a fragment keeps the prefix it was written with.

What counting the joined line's own prefix would take,
and what it would cost:
`MarkdownSource` carries one number,
so it would have to carry the projection's per-line prefix widths,
which `src/processors_docs.rs`,
`line_doc`,
already keeps as the line's `prefix`,
and answer them by line offset.
It cannot write a joined line over the limit,
because the sum counted is the host line's own width:
the run's margin only decides how much of a line's indent sits in the prefix and how much in the
text,
so prefix plus text is the same number either way,
and the absorbing line keeps its own prefix.
Its cost is a bound that differs between lines of one run,
so the same words join or stay depending on which line of the run absorbs them.
The widest prefix was kept because one number per projected source is what the round before this
one built,
and no record asks for a per-line bound.

#### The host-width controls hold a mixed run and a `///` marker (the third control of the read's Low 5)

The read's third control,
quoted:

> every fixture line has the same prefix and the marker is `//!` only,
> so `VirtualSource::host_prefix_chars` written as the first line's prefix,
> the last line's,
> or the shortest stays green.

That was true of the tests this round started from:
`src/run_command_whitespace_fix_tests.rs`,
`a_joined_doc_line_stays_under_the_limit_in_its_host_file`,
ran one prefix per fixture,
`//! ` at three depths,
and the read's own plant showed such a fixture stays green under a first-line prefix.
Changed:

- That test now runs `/// ` and `    /// ` as well as `//! ` and the deeper `//! `,
  so both markers stand at the binary's surface.
- `the_widest_prefix_of_a_comment_run_bounds_the_join` is new
  and runs through the same entry point.
  Its run holds two lines at column zero,
  a middle line four spaces deeper,
  and the word line at column zero,
  so the widest prefix stands neither first nor last
  and the shortest reading gives four characters.
  A word that fills the joined line to 119 characters counted at the widest prefix is written,
  the same word at 120 is declined,
  and the same text with the middle line at column zero joins at that length,
  so the decline is the widest prefix and not the word's length.

Shown able to fail:
`VirtualSource::host_prefix_chars` written as the first line's prefix makes
`the_widest_prefix_of_a_comment_run_bounds_the_join` fail,
while `a_joined_doc_line_stays_under_the_limit_in_its_host_file` stays green,
which is the gap the read named.

#### What the Markdown round did not do

- The read's Lows 1,
  2,
  6,
  7 and 8,
  and the first two controls of its Low 5,
  are the allow-list reader's and belong to the round
  `monochromatic-lint-alpha-25-read-a-spaced-folder-is-listed-once-and-each-unseen-character-gets-its-own-advice`.
- Issue 6667 and the closure rule belong to the round
  `monochromatic-lint-a-package-the-closure-rule-did-not-check-never-ends-as-clean`.
- No option of the line-break proposal was decided,
  and no mutation campaign was run:
  the mutants this round's lines would draw are named under "Checks run".
- A fragment ending in an abbreviation (`e.g.`) still keeps its line,
  although the rule does not break there.
  That predates this round and no finding names it.

#### Checks run

Each once,
in this branch's worktree,
in its own capped scope:

- `mise run //package/linter/monochromatic-lint:lint:rustfmt`:
  `cargo fmt --check` printed its own command and nothing else.
- `mise run //package/linter/monochromatic-lint:test`:
  exit `0` with `1219 tests run: 1219 passed, 3 skipped` at commit `69bc666b56`,
  and the output names every test this round added.
- `mise run //package/linter/monochromatic-lint:lint:types`:
  exit `0`,
  both `cargo check --locked` and `cargo clippy --locked --all-targets -- -D warnings`.
  The first run was red on `clippy::shadow_unrelated`,
  for a second `text_start` this round had put beside the existing one in `fragment_join`;
  the binding is now `text_from`,
  and the task is green.
- `mise run //package/linter/monochromatic-lint:test:host`:
  exit `100`.
  The package defines that task as the closure check's host-only run
  (`cargo nextest run --test closure_binary --run-ignored only`),
  so it names one test,
  `host_real_run_over_wg_quicker_exempt`,
  and no test of this round.
  It fails because the Dylint driver for the active toolchain and `dylint_driver` 6.4.0
  are not on this machine:
  "the Dylint driver for toolchain `nightly-x86_64-unknown-linux-gnu` ... is absent
  (cargo-dylint is 6.4.0),
  and `dylint_driver` 6.4.0 with its dependencies is not in the cargo cache,
  so `cargo dylint` would have to download it,
  which this linter does not do".
  The message names the one command that fills those caches,
  and it needs network access this round does not have.
  The closure check reads no line this round changed.
- The installed `monochromatic-lint 0.5.2-alpha.25` over this README and this file:
  exit `0`,
  no finding.
  Its first run over the same two files was red:
  the join paragraph had commas inside a line,
  and the new section had commas and a colon inside a line,
  which `markdown/semantic-line-breaks` reports at three lines of the README and at 14 lines of
  the section,
  and every one of those lines was rewritten to end at its clause.
- A render of every Markdown passage the round changed,
  through `pandoc --from gfm --to html`,
  read by eye.
- The two released binaries for every reproduction above,
  each in its own capped scope,
  over fixture files in a scratch folder only.

No mutation run was made.
A run of the package's Markdown and processors scopes would be expected to report,
among the lines this round changed:
`break_position` returning the text node's own end instead of the delimiter chain's end,
which the keep list catches on `*held,*`,
`ends_at_break_point` replaced by `true` or by `false`,
which the two lists catch between them,
the `>=` in `position >= end` written `>`,
which the keep list catches on the same shape,
and the `context.host_prefix_chars` term of the width dropped,
which `a_joined_doc_line_stays_under_the_limit_in_its_host_file` catches.

### The allow-list reader's findings

Five of its findings and two controls of a sixth belonged to the allow-list round:
its Low 1,
Low 2,
Low 6,
Low 7,
Low 8,
the first two controls of its Low 5,
and the figure the alpha.24 record gave for the walk over the entries.
Its Lows 3 and 4 and the third control of its Low 5 are the Markdown line-break round's,
and issue 6667 (the closure rule) is another round's.

#### A refusal names the edit for the unseen characters by where each one stands

The read's Low 2,
quoted:

> alpha.25 says `... holds a zero-width space at character 16 of the line, ..., and 1 more character a reader cannot see: the reader does not treat them as whitespace, so take each of them out, and write a whitespace character where one stood if it stood where one belongs.`
> The edit taken (every unseen character out,
> as the sentence says) gives `file-uses src/c dx one.rs -> two.rs`,
> and the next run says `A use between two files is written ..., which is five words; this line is 6 words.` with no folder named:
> a dead end.
> Taking out only the character the message places (the first) gives the refusal that names the folder and says renaming it takes its character out:
> right.

Reproduced on the released binary,
with `src/c d<U+200C>x` a real folder and no module file in it,
for the line `file-uses src/c<U+200B> d<U+200C>x one.rs -> two.rs`:

- the sentence read
  `... and 1 more character a reader cannot see: the reader does not treat them as whitespace, so take each of them out, and write a whitespace character where one stood if it stood where one belongs.`
- the second run over the line that edit leaves,
  `file-uses src/c dx one.rs -> two.rs`,
  read ``A use between two files is written `file-uses <folder> <file> -> <file>`, which is five words; this line is 6 words.``
  and named no folder,
  which is the dead end the read describes.
- the same line with only the placed character out,
  `file-uses src/c d<U+200C>x one.rs -> two.rs`,
  read the refusal that names the folder and says renaming it takes its character out.

Two mechanisms made the sentence:
the question whether a character stands in the folder's own name was asked of the first character only,
so a refusal whose first character stood in the name said nothing about the characters after it;
and a refusal that names no folder said to take each character out,
though it cannot know which one stands in a real folder's name.

Changed,
in `src/layout_allow_entries.rs`:

- `note_of_an_unseen_character` asks the question of every character of the line:
  `unseen_characters_in_span` counts the characters inside the folder's words,
  the sentence names the rename for those,
  and it names the take-out edit for those outside the name.
  The characters the rename takes out are named as a group (`one of them`,
  `2 of them`,
  `every one of them`),
  because a `that character` after `1 more character` reads as the wrong one.
- A refusal that names no folder names the edit for the one character it placed,
  says how many stand beside it,
  and says that a character standing in a real folder's name is taken out by renaming that folder:
  it cannot tell the two apart,
  and the next run names the folder.
- The one-character sentence says `where it stood if it stood`,
  where the release said `where it stands if it stood`,
  a change no record listed.

Before and after,
for the line `file-uses src/c d<U+200C>x on<U+200B>e.rs -> two.rs`
(the folder's character first,
a file name's second),
whose refusal names the folder:

- before:
  ``The line also holds a zero-width non-joiner (`\u{200c}`) at character 18 of the line, where a reader cannot see it, and 1 more character a reader cannot see, and that character stands in the name of the folder above: renaming that folder is the edit that takes it out.``
- after:
  ``The line also holds a zero-width non-joiner (`\u{200c}`) at character 18 of the line, where a reader cannot see it, and 1 more character a reader cannot see, and one of them stands in the name of the folder above, so renaming that folder takes it out: take out the one outside the folder's name, and write a whitespace character where it stood if it stood where one belongs.``

Before and after,
for the line whose folder the reader does not find
(`file-uses src/c<U+200B> d<U+200C>x one.rs -> two.rs`):

- before:
  `... and 1 more character a reader cannot see: the reader does not treat them as whitespace, so take each of them out, and write a whitespace character where one stood if it stood where one belongs.`
- after:
  `... and 1 more character a reader cannot see: the reader does not treat them as whitespace, so take that character out, and write a whitespace character where it stood if it stood where one belongs. A character that stands in the name of a real folder is part of that name, and renaming that folder is the edit that takes it out.`

The controls are
`a_refusal_names_the_edit_for_each_character_by_where_it_stands`
(whole sentences,
for the folder's character second,
for it in the middle of three,
for it first with a file name's after it,
for one character outside the folder's name,
and for a refusal that names no folder,
with the all-in-folder sentence as its control)
and the strengthened
`a_refusal_leaves_the_folder_s_own_character_out_of_its_advice`.
Each was shown able to fail by putting the release's shape back:
with the question asked of no character the new test read
`... and 1 more character a reader cannot see: the reader does not treat them as whitespace, so take each of them out, ...`,
and with the release's folder-less sentence back it read
`... so take each of them out, and write a whitespace character where one stood if it stood where one belongs.`
The same test asserted both sentences whole.

The way not taken is to teach the reader a reading that folds an unseen character into the folder's own whitespace,
so that the refusal names the folder in the line the read used.
That costs a new reading in `spaced_file_use_folder`
and a membership test that is a set of places rather than a span,
and it changes which folder a refusal names,
where this round kept the folder each refusal names exactly as it was.
The sentence now says what is true without it:
the placed character's edit,
and the rename for a character in a real folder's name.

#### A folder a dead key line names is listed once for a list

The read's Low 1,
quoted:

> a dead key line whose edited text names a real folder with a space and many files costs lines times entries;
> the record and the README say the cost is linear in the list

Reproduced on the released binary,
with `src/docs notes` a real folder holding 1,000,
5,000 and 20,000 files and no `.rs` file,
and M lines `<BOM>file-uses src/docs notes f<i>.rs -> g<i>.rs`:
100,
200 and 400 lines cost 0.050,
0.092 and 0.177 CPU seconds over 1,000 files,
0.215,
0.400 and 0.814 over 5,000,
and 0.830,
1.635 and 3.075 over 20,000,
where the same lines naming a folder of 5,000 files whose name holds no space cost 0.007 to 0.009 s
and the edit is a five-word line,
so no refusal is built.
The cost doubled with the files and with the lines,
and the source of the release made it plain:
`entry_of` called `holds_a_module_file`,
which reads the folder,
every time a refusal named such a folder,
and `dead_entry_findings` reads some lines twice.

Changed:
`PackageFolder` keeps one answer per folder for the module-file question,
as it already kept the longest name,
and `entry_of` asks it instead of the free function.
With the release's per-line call put back,
`a_list_of_six_word_edits_lists_the_package_folder_once` read
`50 six-word edits listed the folder they name 100 times for a module file`
(twice per line,
the removal reading and then the space reading);
with the answer kept per folder it reads 1.
That control now creates files in the folder it names,
which is what made the old control green:
it counted the longest-name listing only,
on a folder created empty.

Measured on 2026-11-28,
three interleaved rounds on the run's own CPU time,
each figure a ratio to `0.5.2-alpha.25`'s in the same round,
with a byte-identical copy of this release's build as the spread of every round:

- 100 lines:
  0.157 over 1,000 files (range 0.145 to 0.159),
  0.047 over 5,000 (0.046 to 0.050),
  and 0.022 over 20,000 (0.021 to 0.025).
- 200 lines:
  0.099 (0.085 to 0.119),
  0.025 (0.023 to 0.027) and 0.012 (0.012 to 0.015).
- 400 lines:
  0.058 (0.053 to 0.062),
  0.015 (0.015 to 0.016) and 0.007 (0.006 to 0.008).
- the control,
  200 lines naming a folder whose name holds no space over 5,000 files:
  1.018 (0.925 to 1.086).
- the copy over the fixed build:
  0.830 to 1.144 over the nine shapes.

So the remaining cost is the refusal sentence per line,
which is linear in the list,
and one listing of each folder the lines name,
where the release listed it once per line.
The README's two sentences now say the listing is read once for the list,
and carry these figures.

#### The count of a file name's edit covers the characters that edit changes

The read's Low 6,
quoted:

> The count in a file-name sentence is the edit's count,
> not the line's,
> and it includes a real module's name that the advice function itself leaves alone

Reproduced on the released binary:
for `file-uses src/f o<U+200B>ne.rs -> tw<U+200D>o.rs`,
where `src/f/o<U+200B>ne.rs` is a real module of the folder,
the sentence read
`... and 1 more character a reader cannot see: ... Take both such characters out of the line, then write the file names the rule prints`,
which asks for a change the same sentence undoes:
the next sentence writes the name the rule prints,
which holds that character.
For `file-uses src/g o<U+200B>n<U+200D>e.rs -> two.rs`
the sentence said `the file names` where one name holds both characters.

The session's pick,
with the veto open:
the count and the edit cover the characters the advice changes and no others,
a real module's own name left out as a folder's is,
and the noun follows the number of names.
`unseen_characters_the_file_name_edit_changes` now skips a file name a module of the folder holds
and answers both the number of characters and the number of names,
and `file_names_the_rule_prints` follows the names.
Before and after for the line above:

- before:
  `... and 1 more character a reader cannot see: ... Take both such characters out of the line, then write the file names the rule prints; ...`
- after:
  `... where a reader cannot see it: ... Take that character out of the line, then write the file name the rule prints; ...`

The control is the same test,
`the_count_of_a_file_name_s_edit_covers_the_file_names_characters_only`,
which now asserts the whole sentence for a real module first,
the whole sentence for one second,
and the one-name plural;
with the release's count back it read
`... and 1 more character a reader cannot see: ... Take both such characters out of the line, then write the file names the rule prints; ...`.

The other count,
which this round did not take,
covers every character of the line and names the edit for the file names separately:
it counts characters the sentence does not ask to change,
and a reader who counts the invisible characters in an editor still finds one the sentence never mentions.

#### Two dead key lines whose edits give one line say which lines share the edit

The read's Low 8,
quoted:

> Two or more dead key lines whose edits give one line are each told only to delete,
> and nothing says that one of them could be kept.

The release,
for `\u{feff}uses src/a -> src/b` and `\u{200b}uses src/a -> src/b`,
said of each
`The allow-list entry ... tolerates nothing any more. Delete the line so the list keeps shrinking. The line also holds a byte order mark ... at character 1 of the line, where a reader cannot see it.`
and nothing of the other line.

The package's record,
its README and `two_dead_key_lines_whose_edits_give_one_line_are_not_each_told_it_reads`
say that no such line is told it reads,
because the list refuses the second of the two edits as a repeat.
This round keeps that,
and adds to each such line's sentence which other lines share its edit and that one of them may be kept:

- before:
  ``The allow-list entry ... tolerates nothing any more. Delete the line so the list keeps shrinking. The line also holds a byte order mark or zero-width no-break space (`\u{feff}`) at character 1 of the line, where a reader cannot see it.``
- after:
  the same sentence and then
  ` The line's own edit, taking that character out, gives the same line as line 2: making that edit on one of them and deleting the rest keeps one of them.`

The edit each line names is its own:
a line edited by removing the character says `taking that character out`,
and one edited by writing a space where it stands says `writing a whitespace character where that character stands`.
The record gave no reason against naming the edit,
so the sentence stands.
The control is the same test,
which now asserts the whole message for both lines,
naming two other lines for a group of three,
naming each line's own edit where the two edits differ,
and requiring that a line alone says no such sentence;
with the sentence removed it read the release's message without the added sentence.

#### Two controls that stayed green, and the figure the alpha.24 record gave for the walk

The read's Low 5,
first two controls,
quoted:

> `a_list_of_dead_key_lines_that_read_as_uses_scans_no_other_line` stays green if the old scan is put back,
> because the counter it reads is not on that path;
> and `a_list_of_six_word_edits_lists_the_package_folder_once` stays green under the cost of point 2.

Both reproduced from the source:
the counter was incremented in `entry_holds_key` and `covers_file_use` only,
while the release's repeat question read no entry through either,
and the second control created its folder empty and counted the longest-name listing alone.

Changed:
`AllowList::entries` counts the entries it hands out (`note_entries`),
which is the path every walk over the entries takes,
and `holds_a_module_file` counts its listings,
which is the question the second control now asks of a folder that holds files.
With the release's scan put back,
the first control read
`200 dead key lines read 40600 entries of the list, where the reporting hands out 3 lists`:
40,000 for the walk (200 entries per line over 200 lines)
and 600 for the three times the reporting asks for the entries (its length and its two passes).
With the counter handed no entries it read
`200 dead key lines read 0 entries of the list, where the reporting hands out 3 lists`,
so a counter that counts nothing fails the control as a scan does.
With the release's per-line call put back,
the second read 100 listings for 50 lines.

The alpha.24 record's sentence
`the walk over the entries reads 40,000 of them for 200 lines`
rests on the alpha.24 round's own record commit `23ebfdb868` (2026-11-28T03:19:55-05:00),
which wrote it;
no committed counter stood on that path then,
so the earlier read could not find what produced it.
The figure is the walk's own count (200 entries per line over 200 lines)
and is now what the committed counter reads with the scan in place,
which is why the sentence is kept.
The same paragraph's `the listing runs 100 times for 50 lines`
is what the release's per-line call reads,
two readings per line over 50 lines.

#### What the allow-list round did not do

- The read's Lows 3 and 4 and the third control of its Low 5,
  which are the Markdown line-break round's.
- Issue 6667 (the closure rule).
- The way not taken for the folder-less refusal,
  described above.
- A folder name whose own space the line spells with an unseen character is still no folder the reader finds;
  the refusal says what is true without it and the next run names the folder.

#### Open for the owner

- The count a file-name sentence speaks of.
  It now covers the characters the file-name edit changes,
  a real module's own name left out as a folder's is,
  and the noun follows the number of names.
  The other count covers every character of the line;
  it names characters the sentence does not ask to change.
- Whether a group of dead key lines whose edits give one line should name the edit at all.
  The record and the tests say no line is told it reads;
  each line now also names the lines that share its edit and says one of them may be kept.
