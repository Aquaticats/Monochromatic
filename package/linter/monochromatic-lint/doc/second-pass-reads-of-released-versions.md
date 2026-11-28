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
and nothing of them is repeated here.

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
