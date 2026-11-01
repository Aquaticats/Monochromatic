# Unified-linter mutation survivor dispositions

## Purpose and scope

This handover dispositions every survivor and timeout from the two recorded cargo-mutants campaigns
over `package/linter/monochromatic-lint`,
then records scoped reruns on the changed source.
Mutation testing is a release acceptance gate for the native linter:
a surviving non-equivalent mutant needs a stronger test,
and an equivalent or unreachable mutant needs evidence.

The work is on branch `test/linter-mutation-survivors`,
created from `main` at `cd54f8b64`.
Inspect the dispositions by file,
the rerun results,
and the open items.
Respond by cherry-picking the branch,
or by naming a disposition that the evidence does not support.

Changes are restricted to the survivor source files,
their sibling test files,
`bin/mutate-container.mjs`,
the package `mise.toml`,
and this document.
No dependency,
lockfile,
lint setting,
or mutant exclusion was changed.

## Source campaigns

Both inventories are in the main checkout's ignored build tree,
`package/linter/monochromatic-lint/target/verification/`.

- `mutation-1vJeuS` is the full-scope campaign on an older snapshot
  (test image `b6577f17bf922e1ada05f8c2831315875fbb8410b21cd020e617bdabdd945d0f`):
  439 mutants,
  367 caught,
  18 missed,
  52 unviable,
  2 timeouts.
- `mutation-p3QH2L` is the Markdown-scoped rerun on a later snapshot
  (test image `3bad17a99adb029730c784df94bc855192f0f5dc527b448f9f02384fe0c1582b`):
  496 mutants,
  442 caught,
  17 missed,
  26 unviable,
  11 timeouts.

The union is 35 missed mutants and 11 distinct timeouts
(the two `MarkdownSource::parent` timeouts appear in both campaigns).
It covers eleven source files,
not nine:
`markdown_definitions.rs` and `markdown_punctuation.rs` each contribute one timeout
and are outside this delegation's ownership.

## Method

Each recorded mutant was mapped from its recorded diff
(`mutants.out/diff/` in the campaign directory)
to the source on this branch.
Four dispositions are used.

- Removed means the mutated expression no longer exists,
  because a redundant guard or hand-stepped index was replaced.
  The nearest remaining mutants and the controls that pin the behavior are named.
- Killed means the same mutation is still generated and a named test fails under it.
- Equivalent means the recorded diff cannot change any output;
  the reasoning is given and the redundant code was removed rather than excluded.
- Detected by timeout means the mutation makes a loop non-terminating,
  so no test that reaches it can finish.
  It is used only for the two loops outside this delegation.

Proof is the scoped rerun,
not a manual edit:
cargo-mutants plants each mutation in turn inside the mount-free container
and its per-mutant log names the tests that failed.
Every "caught" claim in this document resolves to a line of a rerun's `caught.txt`.
The `MarkdownSource::parent` mutants got a scope of their own,
because the Markdown scope can only report two of them as timeouts.

Statements about what a new control would do to a mutant of removed code are derived from the recorded diff.
They were not measured,
because that code no longer exists to mutate.
What was measured is that each new control can fail:
"Controls shown to fail" lists the planted mutants under which each one failed.

## Why the loops were restructured

Nine of the eleven timeouts had one shape.
A `while` loop advanced its own index with `+= 1` or `-= 1`,
and cargo-mutants replaced that step with `*= 1` or `/= 1`.
The index then never changes,
the loop condition never changes,
and every test that enters the loop spins until the 180 second campaign timeout.

A timeout makes the campaign exit with status 3
("Some tests timed out" in the cargo-mutants exit-code documentation),
so the gate cannot pass while such loops exist.
The seven loops in the owned files now walk a slice or call a standard search,
so termination no longer depends on a step the loop body performs.
The same change removed the index guards and offset arithmetic
that produced most of the surviving boundary mutants.

The Markdown scope now generates 424 mutants instead of 496:
`markdown_block_start.rs` went from 87 to 61,
`markdown_break_points.rs` from 50 to 35,
`markdown_commands.rs` from 20 to 16,
`markdown_prose_context.rs` from 35 to 23,
and `markdown_tables.rs` from 35 to 20.
The other eleven Markdown files generate the same counts as before.

## Dispositions by file

Locations are `line:column` as recorded by the named campaign.
Each `caught.txt` reference names the campaign it comes from.

### `src/markdown_block_start.rs`

- `15:62: replace || with && in boundary` was missed in `mutation-p3QH2L`.
  Still generated at `15:62`.
  Killed by `recognizes_the_incumbent_block_start_catalog`.
  Under the mutant LF and CR stop being marker boundaries,
  which its cases ` #\n`,
  ` #\r`,
  ` 12.\r` and ` 9)\n` require.
  Those cases were added in `111e02fec`,
  after the campaign's snapshot.
  Caught in `mutation-BX2JYq` (`caught.txt` line 3).
- `15:44: replace || with && in boundary` was missed in `mutation-p3QH2L`.
  Still generated at `15:44`.
  Killed by the same test.
  Under the mutant a tab stops being a marker boundary,
  which its cases ` #\t heading` and ` 3)\t item` require.
  Caught in `mutation-BX2JYq` (`caught.txt` line 4).
- `36:23: replace || with && in starts_block_construct` was missed in `mutation-p3QH2L`.
  Removed in `111e02fec`.
  Equivalent:
  the mutated line was an early `return false` for a leading LF or CR,
  and a newline byte matches no marker branch and no digit,
  so the function returns `false` without the guard.
  A lone LF and a lone CR remain negative controls in `leaves_non_markers_and_empty_lines_available`.
- `30:15: replace += with *= in starts_block_construct` timed out in `mutation-p3QH2L`.
  Removed.
  The leading-spacing loop is now `trim_start_matches([' ', '\t'])`,
  which has no step to mutate.
- `45:13: replace += with *= in starts_block_construct` timed out in `mutation-p3QH2L`.
  Removed.
  The marker run is counted while walking the slice.
  The counter mutants at `45:15` now terminate and are caught.
  They are recorded in `mutation-BX2JYq` at `caught.txt` lines 18 and 19.
- `69:20: replace += with *= in starts_block_construct` timed out in `mutation-p3QH2L`.
  Removed.
  The rule-line scan iterates `&line[width..]` and has no counter.
- `76:19: replace += with *= in starts_block_construct` timed out in `mutation-p3QH2L`.
  Removed.
  The digit run is counted while walking the slice.
  The counter mutants at `75:16` now terminate and are caught.
  They are recorded in `mutation-BX2JYq` at `caught.txt` lines 53 and 54.

The negative controls ` . next` and ` ) next` were added,
because the digit guard is what keeps a leading delimiter from being read as an ordered marker.

### `src/markdown_break_points.rs`

- `50:59: replace || with && in separator` was missed in `mutation-p3QH2L`.
  Removed in `111e02fec`.
  Equivalent:
  the mutation stopped LF and CR from counting as separators,
  but `needs_break` already returns `false` when the next non-space byte is a line ending,
  so a newline separator could never produce a break.
  `separator` now accepts only space and tab.
- `91:19: replace += with *= in break_offsets` timed out in `mutation-p3QH2L`.
  Removed.
  Closing delimiters are passed with `trim_start_matches(closing)`.
  The offset arithmetic that remains is at `88:38` and `89:40`,
  and all four of its mutants are caught.
  They are recorded in `mutation-BX2JYq` at `caught.txt` lines 85 to 88.
- `93:48: replace < with <= in break_offsets` was missed in `mutation-p3QH2L`.
  Removed.
  The comparison chose between the node's own tail and the paragraph's following source;
  `tail.chars().chain(trailing.chars()).next()` reads the next written character without a comparison.
  The mutant was not equivalent:
  for a node that ends at its break point it ignored the following source,
  so text glued to inline code gained a break.
  `glued_trailing_source_is_not_a_separator`
  and `parsed_inline_code_glued_to_a_sentence_gets_no_break` now pin that case.
- `103:22: replace > with >= in break_offsets` was missed in `mutation-p3QH2L`.
  Removed.
  The guard protected `index - 1` at index zero;
  `slice[..index].ends_with('.')` needs no guard.
  The mutant was not equivalent:
  it subtracted from zero for a text node that begins with a period.
  `leading_break_points_are_not_mistaken_for_ellipses`
  and `parsed_text_nodes_may_begin_with_their_break_point` now pin that case.

### `src/markdown_code.rs`

- `38:23: replace + with - in language_insert_offset` was missed in `mutation-1vJeuS`.
  Removed in `028079a69`.
  The helper added an indentation width to the node start,
  but native fenced-code spans already begin at the marker.
  `fence_spans_preserve_native_marker_boundaries` (added in `294e40db0`) proves that for column-one,
  indented,
  quoted,
  list-contained and Unicode-prefixed fences.
  The helper is now `fence_marker_end`,
  and its remaining sum at `41:23` was caught in `mutation-p3QH2L` (`caught.txt` lines 134 and 135).
  Both are caught again in `mutation-BX2JYq` (`caught.txt` lines 104 and 105).

### `src/markdown_commands.rs`

All three survivors mutated one expression,
`start + opener_end + 1`,
which skipped the fence's opening line before scanning for prompts.
The skip was redundant:
a fenced block's first line begins with its marker and can never match `$ `.
`prompt_edits` now scans the whole node from `start`,
and the expression is gone.

- `80:54: replace + with * in commands_show_output` was missed in `mutation-p3QH2L`.
  Removed.
  Equivalent:
  the scan began at the opening line's own line ending,
  which is an empty segment.
- `80:54: replace + with - in commands_show_output` was missed in `mutation-p3QH2L`.
  Removed.
  Equivalent for an opening line that ends in an ASCII byte.
  For an info string that ends in a multi-byte character the offset split that character.
- `80:41: replace + with * in commands_show_output` was missed in `mutation-p3QH2L`.
  Removed.
  Equivalent only for a fence at byte zero,
  which every earlier fixture used.

`fences_after_other_content_keep_absolute_prompt_offsets` places prose before the fence
and uses a Cyrillic info string in LF,
CRLF and bare CR,
and compares each edit with an offset found by searching the fixture.
It covers both non-equivalent cases.
All 14 viable mutants of `markdown_commands.rs` are caught in `mutation-BX2JYq`
(`caught.txt` lines 109 to 122;
the other 2 are unviable).

### `src/markdown_headings.rs`

- `24:35: replace > with >= in heading_increment` was missed in `mutation-1vJeuS`.
- `24:46: replace + with - in heading_increment` was missed in `mutation-1vJeuS`.
- `24:46: replace + with * in heading_increment` was missed in `mutation-1vJeuS`.

All three are still generated at the same locations.
Killed by `heading_increment_accepts_adjacent_depths_and_equal_siblings`,
added in `1eae5c4bc` after the campaign's snapshot:
the first and third report a legal one-step increase,
the second reports an equal-depth sibling.
They were caught in `mutation-p3QH2L` (`caught.txt` lines 204 to 206).
They are caught again in `mutation-BX2JYq` (`caught.txt` lines 173 to 175).

### `src/markdown_prose_context.rs`

- `38:32: replace && with || in paragraph_for` was missed in `mutation-p3QH2L`.
  Removed.
  Equivalent.
  The mutated guard `paragraph.is_none()` kept only the first paragraph ancestor.
  The mutant could differ in two situations,
  and the parser produces neither.
  The first is a second paragraph ancestor.
  `satteri-pulldown-cmark` 0.6.3 opens `Paragraph` nodes only for block-level paragraph items
  (`arena_build.rs` line 961,
  plus the directive label at line 974,
  which needs `ENABLE_DIRECTIVE` and is not enabled),
  and a paragraph's children are inline content.
  The second is a text node with neither a paragraph nor an excluded ancestor.
  Inline content is tokenized only by `parse_line`,
  which `firstpass.rs` calls for paragraphs (line 1380),
  table cells (line 1208),
  headings (lines 1631,
  3290,
  3294 and 3315)
  and directive labels (lines 642,
  1329 and 2399,
  not enabled).
  Headings and table cells are excluded kinds,
  so every text node has a paragraph or an excluded ancestor.
  `paragraphs_never_nest_inside_paragraphs` checks the first claim through real parses of quotes,
  lists,
  footnotes,
  links and MDX elements.
  The condition is now `kind == MdastNodeType::Paragraph`,
  and `paragraph_lookup_climbs_inline_wrappers_and_respects_exclusions` calls the function directly.
- `71:46: replace - with + in continuation_prefix` was missed in `mutation-p3QH2L`.
- `71:46: replace - with / in continuation_prefix` was missed in `mutation-p3QH2L`.
- `71:80: replace - with + in continuation_prefix` was missed in `mutation-p3QH2L`.
- `71:80: replace - with / in continuation_prefix` was missed in `mutation-p3QH2L`.
- `72:20: replace -= with /= in continuation_prefix` timed out in `mutation-p3QH2L`.

These five mutated one backward `while` loop that looked for the paragraph's line start.
Removed:
the line start now comes from `rfind(['\n', '\r'])`.
The four index mutants were not equivalent.
Each read a neighbouring byte when testing for LF or for CR,
so the copied prefix began at or before the previous line's ending.
They survived because every fixture with a fix had its paragraph on the first line,
where the scan in those fixtures ran to byte zero under each mutant.
`later_line_paragraphs_copy_only_their_own_line_prefix` covers LF,
CRLF and bare CR,
and `nested_later_line_containers_keep_markers_and_blank_list_bullets` covers a quote inside a list.
The remaining arithmetic is `ending + 1` at `75:29`,
and both of its mutants are caught.
They are recorded in `mutation-BX2JYq` at `caught.txt` lines 234 and 235.
The paragraph check that replaced the guard,
`replace == with != in paragraph_for` at `39:17`,
is caught at line 223.

- `80:49: replace || with && in continuation_prefix` was missed in `mutation-p3QH2L`.
  Still generated,
  now at `83:49`.
  Killed:
  a tab in the prefix became a space,
  and `tab_indentation_survives_in_the_continuation_prefix` fails.
  Caught in `mutation-BX2JYq` (`caught.txt` line 238).

### `src/markdown_source.rs`

- `121:33: replace || with && in traversal` was missed in `mutation-1vJeuS`.
- `147:13: replace || with && in traversal` was missed in `mutation-1vJeuS`.
- `148:13: replace || with && in traversal` was missed in `mutation-1vJeuS`.
- `149:13: replace || with && in traversal` was missed in `mutation-1vJeuS`.

Still generated at the same locations.
Killed by the damaged-arena controls in `markdown_traversal_tests.rs`,
added in `5c4c240ba` after the campaign's snapshot.
They were caught in `mutation-p3QH2L` (`caught.txt` lines 334 and 343 to 345).
They are caught again in `mutation-BX2JYq` (`caught.txt` lines 298 and 307 to 309).

- `309:37: replace - with + in MarkdownSource::node_span` was missed in `mutation-1vJeuS`.
  Still generated,
  now at `314:37`.
  Killed by the exact length assertion in `bom_and_astral_source_slices_are_exact`,
  added in `40fb44175`.
  It was caught in `mutation-p3QH2L` (`caught.txt` line 380).
  It is caught again in `mutation-BX2JYq` (`caught.txt` line 344).
- `352:9: replace MarkdownSource::text_nodes -> Vec<u32> with vec![]` was missed in `mutation-1vJeuS`.
- `352:9: replace MarkdownSource::text_nodes -> Vec<u32> with vec![0]` was missed in `mutation-1vJeuS`.
- `352:9: replace MarkdownSource::text_nodes -> Vec<u32> with vec![1]` was missed in `mutation-1vJeuS`.
- `355:35: replace == with != in MarkdownSource::text_nodes` was missed in `mutation-1vJeuS`.

Still generated,
now at `357:9` and `360:35`.
The method had no tested caller in the older snapshot.
Its caller is the heading-punctuation rule,
whose tests were caught killing all four in `mutation-p3QH2L` (`caught.txt` lines 392 to 395).
They are caught again in `mutation-BX2JYq` (`caught.txt` lines 356 to 359).

- `314:9: replace MarkdownSource::parent -> Option<u32> with Some(0)` timed out in `mutation-1vJeuS`.
- `314:9: replace MarkdownSource::parent -> Option<u32> with Some(1)` timed out in `mutation-1vJeuS`.

The same two mutants are recorded at `319:9` in `mutation-p3QH2L`,
and are still generated there.
Killed by the new `parents_mirror_child_edges_and_stop_at_the_root`,
which checks the contract without an ancestor walk:
the root has no parent,
and every child names the node that lists it.
The `--markdown-parent` scope plants the mutants of `parent` and runs only that control.
Both are caught there in under a second
(`mutation-BmlHMt`,
`caught.txt` lines 2 and 3).

The Markdown scope still reports the same two as timeouts,
because other tests in the same binary spin.
`parent` is the only way an ancestor walk ends:
`has_ancestor`,
`paragraph_for` and `delimiter_tail` climb until it returns `None`.
A constant answer makes the root,
or node 1,
its own ancestor,
so each of those walks never finishes,
and one unfinished test keeps the test binary alive until the 180 second limit.
Bounding the walks would add a guard against a cycle
that `traversal` already makes impossible,
so the walks were left alone.
`mutation-BX2JYq` records both as timeouts again.

### `src/markdown_tables.rs`

All three mutated the backslash-run count before a cell's final pipe.
The count is gone:
`cell_content` now flips a flag for each backslash while walking the cell backwards.

- `33:18: replace > with >= in cell_content` was missed in `mutation-p3QH2L`.
  Removed.
  Not equivalent:
  the guard protected `before - 1` at index zero,
  reached when the backslash run starts the cell or the cell is empty.
  No fixture had such a cell.
- `34:16: replace -= with /= in cell_content` timed out in `mutation-p3QH2L`.
  Removed.
  The reversed walk has no step.
- `36:14: replace - with + in cell_content` was missed in `mutation-p3QH2L`.
  Removed.
  Equivalent:
  a sum and a difference of the same two numbers have the same parity,
  and only parity was read.

`escape_runs_reaching_the_cell_start_decide_the_final_pipe` covers runs of one,
two and three backslashes that reach the first byte,
and the empty cell.
`parsed_cells_ending_in_escaped_pipes_keep_their_text` covers the same boundary through parsed rows.
The two remaining mutants at `34:17` and `37:19` are caught.
They are recorded in `mutation-BX2JYq` at `caught.txt` lines 380 and 381.

### `src/rust_inferred_constants.rs`

- `42:5: replace argument_index -> Option<usize> with Some(0)` was missed in `mutation-1vJeuS`.
- `48:81: replace += with -= in argument_index` was missed in `mutation-1vJeuS`.
- `48:81: replace += with *= in argument_index` was missed in `mutation-1vJeuS`.
- `92:15: replace += with -= in is_inferred_const_argument` was missed in `mutation-1vJeuS`.
- `92:15: replace += with *= in is_inferred_const_argument` was missed in `mutation-1vJeuS`.

Still generated at the same locations.
They survived because the older snapshot had no case with a hole in a second slot,
so neither counter was ever incremented before a match.
The conformance case "mixed type and constant slots retain their positions" was added in `0baded620`,
after the campaign's snapshot.
`semantic_conformance_and_source_overlay_controls`,
the one test function that holds every conformance case,
failed under all five in `mutation-yE1SL2`;
its log cannot say which case failed first.

`holes_resolve_against_the_parameter_in_their_own_slot` in the new `rust_inferred_constants_tests.rs`
is a second,
independent control.
The conformance cases compare finding counts,
and that one case covers one slot order.
The new test compares the ordered finding messages for a constant hole after a type argument,
a type hole after a constant argument,
two adjacent holes,
a lifetime slot,
and a method call.

All five are caught in the constant-slot rerun `mutation-yE1SL2`
(`caught.txt` lines 3,
6,
7,
11 and 12).
Both controls failed under each of them,
and under every other viable mutant of the file.

### Files outside this delegation

- `src/markdown_definitions.rs:44:20: replace -= with /= in removal_edit` timed out in `mutation-p3QH2L`.
- `src/markdown_punctuation.rs:64:29: replace -= with /= in suffix_start` timed out in `mutation-p3QH2L`.

Both are still generated and both are detected by timeout;
`mutation-BX2JYq` records both as timeouts again.
They have the hand-stepped shape described in "Why the loops were restructured".
`removal_edit` scans back to the line start exactly as `continuation_prefix` did,
and `suffix_start` counts a backslash run exactly as `cell_content` did.
Neither file was edited,
because both belong to another delegate.
The fixes applied here would apply unchanged:
`rfind(['\n', '\r'])` for the line start,
and a reversed byte walk that flips `escaped` for the backslash run.

## Rerun results

Every run used `MONOCHROMATIC_LINT_IMAGE_TAG=survivors` from the worktree root.
Every container was mount-free and network-disabled,
bounded to 2 GiB of memory,
2 CPUs and 128 processes.
cargo-mutants is 27.1.0,
binary SHA-256 `f985f265ee3ea3e453aa98b04c52134953911f692f8ce8abf137f3873202a2d0`.
Evidence directories are in `package/linter/monochromatic-lint/target/verification/`,
in the worktree and copied to the main checkout.

### Tested snapshot

All three campaigns mutate one test image,
`f1ccc6b562a7aed6507b6d19af275edb33d2da0b727c692d44f5e9cf41c94111`.
It was built from the worktree at commit `1e257034b`,
whose `src` directory is git tree `77a14a737e96d6160c89064fdf3f6df6d7581fdf`
and whose `fixtures` directory is git tree `b709a4fb07297bba4d0826798e4bde8681b00966`.
Later commits on the branch change only the mutation runner,
the package tasks and this document.

`mise run //package/linter/monochromatic-lint:mutation:inferred-constants` ran the gate as its dependency chain.
`test:container` passed 217 tests in 138.78 seconds,
and `lint:container` finished Clippy with `-D warnings` and no finding.
The log of both steps,
including the image ID,
is `gate-survivors-1e257034b.log` beside the evidence directories.
That first chain stopped before mutating,
as "Constant-slot scope" explains.
The campaigns were then started with `mise run --skip-deps`,
so each one mutates the tested image instead of a rebuilt one.

### Constant-slot scope

The command was
`mise run --skip-deps //package/linter/monochromatic-lint:mutation:inferred-constants`.
Evidence is `mutation-yE1SL2`,
exit status 0:
14 mutants,
12 caught,
0 missed,
2 unviable,
0 timeouts.
The unmutated baseline built in 130.4 seconds and ran both test filters in 86.8 seconds.

A first attempt,
`mutation-T789Hq` in the worktree only,
exited 4 before any mutant ran.
cargo-mutants placed both test names before Cargo's own separator,
and Cargo accepts one test name there.
`be43d526f` adds a second separator so that both names reach the test binary.

### Parent-lookup scope

The command was
`mise run --skip-deps //package/linter/monochromatic-lint:mutation:markdown:parent`.
Evidence is `mutation-BmlHMt`,
exit status 0:
4 mutants,
4 caught,
0 missed,
0 unviable,
0 timeouts.
Each mutant's test run took 0.2 to 0.3 seconds
and failed `markdown_source::tests::parents_mirror_child_edges_and_stop_at_the_root`.

### Markdown scope

The command was
`mise run --skip-deps //package/linter/monochromatic-lint:mutation:markdown`.
Evidence is `mutation-BX2JYq`,
exit status 3:
424 mutants,
394 caught,
0 missed,
26 unviable,
4 timeouts.
The unmutated baseline built in 101.4 seconds and ran the Markdown tests in 0.3 seconds.
The campaign ran from 14:54:58 to 15:30:27 UTC.

The four timeouts are exactly the ones this document predicts:
`markdown_definitions.rs:44:20`,
`markdown_punctuation.rs:64:29`,
and the two `Some` replacements at `markdown_source.rs:319:9`.
No mutant of the five restructured files was missed or timed out:
`markdown_block_start.rs` has 61 caught,
`markdown_break_points.rs` 33 caught and 2 unviable,
`markdown_commands.rs` 14 caught and 2 unviable,
`markdown_prose_context.rs` 23 caught,
and `markdown_tables.rs` 17 caught and 3 unviable.

### Controls shown to fail

Each new control failed under at least one planted mutant.
The count is the number of mutants under which the per-mutant log records the control as failed.

- `rust_inferred_constants::tests::holes_resolve_against_the_parameter_in_their_own_slot` failed under 12,
  every viable mutant of `mutation-yE1SL2`.
- `markdown_source::tests::parents_mirror_child_edges_and_stop_at_the_root` failed under 4,
  every mutant of `mutation-BmlHMt`.

The remaining counts come from `mutation-BX2JYq`.

- `markdown_block_start::tests::leaves_non_markers_and_empty_lines_available`
  (extended with ` . next` and ` ) next`) failed under 41.
- `markdown_break_points::tests::leading_break_points_are_not_mistaken_for_ellipses` failed under 23.
- `markdown_break_points::tests::parsed_text_nodes_may_begin_with_their_break_point` failed under 62.
- `markdown_break_points::tests::glued_trailing_source_is_not_a_separator` failed under 21.
- `markdown_break_points::tests::parsed_inline_code_glued_to_a_sentence_gets_no_break` failed under 65.
- `markdown_commands::tests::fences_after_other_content_keep_absolute_prompt_offsets` failed under 47.
- `markdown_prose_context::tests::later_line_paragraphs_copy_only_their_own_line_prefix` failed under 78.
- `markdown_prose_context::tests::nested_later_line_containers_keep_markers_and_blank_list_bullets` failed under 80.
- `markdown_prose_context::tests::tab_indentation_survives_in_the_continuation_prefix` failed under 81.
- `markdown_prose_context::tests::paragraph_lookup_climbs_inline_wrappers_and_respects_exclusions` failed under 33.
- `markdown_prose_context::tests::paragraphs_never_nest_inside_paragraphs` failed under 19.
- `markdown_source::tests::parents_mirror_child_edges_and_stop_at_the_root` failed under 17.
- `markdown_tables::tests::escape_runs_reaching_the_cell_start_decide_the_final_pipe` failed under 4.
- `markdown_tables::tests::parsed_cells_ending_in_escaped_pipes_keep_their_text` failed under 61.

## Open items

### The Markdown scope still exits 3

Four timeouts remain,
so `mutation:markdown` fails after copying its report.
Read the counts from `mutants.out/outcomes.json`,
not from the task's exit status.
Two of the four are the loops in `markdown_definitions.rs` and `markdown_punctuation.rs`;
"Files outside this delegation" names the change that removes each.
The other two are the `MarkdownSource::parent` replacements.

### A possible home for the parent-lookup mutants

This is a proposal,
not something this branch did.
The Markdown scope could exclude exactly the two constant replacements of `parent`
with `--exclude-re 'replace MarkdownSource::parent -> Option<u32> with Some'`,
and the gate could run `mutation:markdown:parent` beside it.
Both mutants would stay under test,
in the scope where they are caught instead of timed out.
With the two loops outside this delegation also fixed,
`mutation:markdown` would then exit 0.
No exclusion was added here,
because the instruction for this work was to exclude nothing without written evidence
and the choice changes what the gate measures.

### Whole-suite duration against the per-mutant limit

`bin/mutate-container.mjs` passes `--timeout 180` to every scope.
The whole suite took 138.78 seconds in the bounded container at this snapshot
(one measurement,
on a host shared with other builds).
An unscoped `mutation` run would therefore have about 41 seconds of margin for every mutant that does not fail early,
and a slower run would be recorded as a timeout.
The scoped tasks are not affected:
the Markdown tests finish in under a second,
and the constant-slot filters took 86.8 seconds unmutated.

The new constant-slot control loads its own Cargo fixture,
which is part of that duration.
Its five cases could move into the fixture that `rust_explicit_types_tests.rs` already loads.
That file is outside this delegation.

### What the reruns do not cover

The unscoped `mutation` task was not run,
as instructed.
The three scopes together cover every file with a recorded survivor or timeout,
at the branch snapshot.

`main` has moved since the branch was created.
At `cba0702fe` it adds 20 `markdown_*` files
(dispatch,
LFS and rule-settings modules with their tests)
that the `src/markdown_*.rs` glob will include after a cherry-pick.
None of them was mutated here.
`git merge-tree --write-tree main` merged this branch into that commit without a conflict.

### Task additions

`test:markdown` and `test:inferred-constants` were added beside `test:rust-style`,
as the focused-slice instruction asked,
although the ownership list names only the `mutation*` tasks.
`mutation:inferred-constants`,
`mutation:list:inferred-constants` and `mutation:markdown:parent` are new mutation tasks,
with `--inferred-constants` and `--markdown-parent` scopes in `bin/mutate-container.mjs`.

### Existing rustdoc findings

`lint:rust` reports 85 `builtin(require-rustdoc)` findings at `3f0cb5b2c`,
and no other finding.
Ten are in files this branch touched
(`markdown_commands.rs`,
`markdown_prose_context.rs` and `markdown_tables.rs`),
and `git blame` attributes each of those lines to the branch's base commit.
Most are `use` lines that share one comment.
None was added or fixed here.

## Commits

The branch holds these commits after `9be97dce4`,
in order,
followed by the commits that add this document.

- `fc703a336` adds the `--inferred-constants` scope and the focused test tasks.
- `87af1a4af` scans block-start markers over slices.
- `5a1bbded9` trims closing delimiters and removes the node-edge and ellipsis index guards.
- `d9dd766d0` scans prompt lines from the fence start.
- `9d5bca2fd` finds continuation line starts by reverse search and drops the first-paragraph guard.
- `2db4109f2` decides escaped closing cell pipes by a parity flag.
- `249b0390d` adds the parent-lookup contract control.
- `0cc406505` adds the constant-slot message controls.
- `83613ca61` applies rustfmt line breaks to the new controls.
- `1e257034b` documents each import of the new controls.
- `5a9a47208` filters the constant-slot scope to the tests that reach the resolver.
- `be43d526f` passes both constant-slot test filters to the test binary.
- `7df60fe63` adds the `--markdown-parent` scope.

The evidence directories `mutation-yE1SL2`,
`mutation-BmlHMt` and `mutation-BX2JYq`,
and `gate-survivors-1e257034b.log`,
are copied to the main checkout's `package/linter/monochromatic-lint/target/verification/`.
