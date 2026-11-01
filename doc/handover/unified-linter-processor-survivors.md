# Unified-linter processor mutation survivors

Note 2026-10-30: `package/linter/monochromatic-lint/bin/test-container.mjs` and `clippy-container.mjs` are
TypeScript files now; read those names here as `test-container.ts` and `clippy-container.ts`.

## Purpose and how to respond

The first unrestricted `cargo-mutants` pass over the embedded processors of
`package/linter/monochromatic-lint` left 52 missed mutants and 2 timeouts.
This handover records one disposition per mutant,
the tests and source simplifications that produced it,
and the rerun that proves it.
Branch `test/linter-processor-survivors` holds the work,
based on commit `9be97dce4`,
the snapshot the first pass mutated.

Inspect the two source simplifications under "Source changes" first,
because they are the only production edits.
Then inspect the seam test named under "`src/processors_spans.rs`",
because it is the only test that does not go through the `VirtualSource` interface.
Respond by cherry-picking the branch,
or by naming the entry whose disposition you reject.

## Result

The rerun on the final source snapshot found 374 mutants:
348 caught,
0 missed,
24 unviable,
and 2 timeouts.
The 2 timeouts are the same two loop-counter mutants as before,
and "Timeouts" explains why a timeout is their only possible detection.

The mutant count fell by 6 because the two simplified lines generate fewer mutants.
A set comparison of both reports finds exactly those 6 removed and none added:

- 2 `line_doc` mutants of the removed `end += 2` statement.
- 4 `fences` mutants of the removed offset arithmetic and its `>` comparison.
  The one mutant of the new `>=` comparison is caught.

Baseline for comparison,
from `target/verification/mutation-exJwfB` on image
`57e310bd9253f1fd6e84437012336fb14c9ecf36fef71091b3fd9fe15beab3f5`:
380 mutants,
302 caught,
52 missed,
24 unviable,
2 timeouts.

Disposition of the 52 missed mutants:

- 46 are killed by new tests that go through `extract`,
  `project_diagnostic`,
  `project_fix`,
  `check_rust`,
  and `edits::apply_fixes` on real Rust and Markdown parses.
- 3 are killed by one direct contract test of the crate-internal `host_range` seam,
  because no `VirtualSource` caller can produce the input that separates them.
- 2 sat on a branch that cannot run;
  the branch is removed.
- 1 differs from the original only on an input the native Markdown parser cannot produce;
  its redundant offset arithmetic is removed and the evidence is recorded here.

Both timeouts remain timeouts,
and the timeout is the only detection possible for them.

No mutant exposed a wrong host position or a corrupting fix in the processor source.
The notable gaps were in the tests,
and "Defects and gaps found" lists them.

## Method

Every killing test asserts exact host byte offsets,
exact virtual text,
exact fixed host output,
or an exact refusal message.
No existing assertion was changed or removed.

Each killable mutant was planted by its exact recorded span and replacement
from `mutation-exJwfB/mutants.out/outcomes.json`,
one at a time,
and the processor test slice was run against it.
Planting used a single container created from `localhost/monochromatic-lint-test:processor-survivors`
with `--network=none --memory=2g --cpus=2 --pids-limit=128` and no mounts.
That image was
`57e310bd9253f1fd6e84437012336fb14c9ecf36fef71091b3fd9fe15beab3f5`,
the same content-addressed image the baseline campaign mutated,
because the branch base has identical crate sources.
Fresh copies of the processor sources and tests,
and then of the one planted file,
were copied in with `podman cp`,
so the worktree was never mutated.
The slice passed before the first plant and again after the last one was reverted.

The scratch driver,
its plan files,
and the per-mutant results are stored beside the rerun report,
in the `hand-plants` directory of the evidence directory named under "Rerun evidence".
`plants-round1.json` holds the 52 original missed mutants against the tests of commit `43737b551`
on unchanged processor source:
49 fail and 3 pass.
`plants-round2.json` holds the mutants of the two simplified source lines,
as committed in `f54b334a9` and `e7e5b24aa`.

## Dispositions by file

Line and column locations are those of commit `9be97dce4`.
Test names are relative to `processors::tests`.
Where several new tests fail under one mutant,
the entry names the most direct one.

### `src/processors_spans.rs`

15 missed mutants.
The existing tests mapped only labels of length one or more that started inside a line,
on hosts whose first authored byte was near offset zero.
The new tests map zero-width labels at a line start,
a line interior,
the boundary between two lines,
and the virtual end,
on a host where every expected offset differs from zero,
from one,
and from the container anchor.

- `20:74`,
  `&&` to `||` in `direct`:
  killed by `spans::processors_map_zero_width_labels_at_line_starts_interiors_and_virtual_end`.
- `20:60`,
  `&&` to `||` in `direct`:
  killed by `spans::processors_map_zero_width_labels_at_line_starts_interiors_and_virtual_end`.
- `20:32`,
  `&&` to `||` in `direct`:
  killed by `spans::processors_map_empty_block_doc_payloads_separately_from_container_anchors`.
- `29:36`,
  `&&` to `||` in `direct`:
  killed by `spans::processors_map_zero_width_labels_at_line_starts_interiors_and_virtual_end`.
- `29:27`,
  `<=` to `>` in `direct`:
  killed by `spans::processors_map_zero_width_labels_at_line_starts_interiors_and_virtual_end`.
- `29:45`,
  `<` to `<=` in `direct`:
  killed by `spans::processors_map_zero_width_labels_at_line_starts_interiors_and_virtual_end`.
- `30:48`,
  `&&` to `||` in `direct`:
  killed by `spans::processors_map_zero_width_labels_at_line_starts_interiors_and_virtual_end`.
- `30:26`,
  `==` to `!=` in `direct`:
  killed by `spans::processors_map_zero_width_labels_at_line_starts_interiors_and_virtual_end`.
- `30:60`,
  `==` to `!=` in `direct`:
  killed by `spans::processors_map_zero_width_labels_at_line_starts_interiors_and_virtual_end`.
- `35:30`,
  `<` to `<=` in `direct`:
  killed by `spans::processors_map_ranges_ending_at_line_boundaries_without_the_next_prefix`.
- `55:9`,
  `||` to `&&` in `host_range`:
  killed by `spans::processors_host_range_seam_refuses_malformed_spans`.
- `54:9`,
  `||` to `&&` in `host_range`:
  killed by `spans::processors_host_range_seam_refuses_malformed_spans`.
- `53:9`,
  `||` to `&&` in `host_range`:
  killed by `spans::processors_host_range_seam_refuses_malformed_spans`.
- `70:5`,
  `anchor` body replaced by `0`:
  killed by `spans::processors_anchor_refusals_at_authored_containers_and_render_them`.
- `70:5`,
  `anchor` body replaced by `1`:
  killed by `spans::processors_anchor_refusals_at_authored_containers_and_render_them`.

#### The `host_range` seam test

The three `host_range` mutants differ from the original only for a reversed span,
or a span with exactly one endpoint inside a UTF-8 character.
No current caller passes such a span:

- `VirtualSource::project_diagnostic` validates the same conditions first and returns an error
  (`src/processors.rs`,
  the check before the `host_range` call).
- `processors_spans::anchor` and `Mapping::error_at` pass a single point,
  so start equals end.

The guard is still the function's documented contract,
and `error_at` forwards offsets from the native Markdown parser,
which `src/markdown_source.rs` does not assume to be character boundaries.
The test therefore extracts a real Rustdoc mapping and calls `crate::processors_spans::host_range` directly
with a reversed span,
a start-only split,
an end-only split,
and an out-of-range end,
after two positive controls.
This is the only new test that bypasses the `VirtualSource` interface.
The three mutants are equivalent on every input reachable through `VirtualSource`;
only the direct seam call separates them.
If the guard is judged unnecessary instead,
deleting it and this test is the alternative.

### `src/processors_docs.rs`

14 missed mutants and 1 timeout.
The existing margin tests used runs whose lines all had the same indentation,
and asserted through a fence that still parses with one to three leftover spaces.
The new tests compare the whole virtual string for runs and blocks whose lines indent differently,
contain an empty comment line,
or contain a line of only non-ASCII whitespace.

- `77:36`,
  `-` to `+` in `line_doc`:
  killed by `docs::processors_keep_non_ascii_whitespace_lines_outside_stripped_margins`.
- `77:36`,
  `-` to `/` in `line_doc`:
  killed by `docs::processors_strip_only_the_common_margin_from_line_doc_runs`.
- `81:13`,
  `+=` to `-=` in `line_doc`:
  unreachable;
  the branch is removed.
- `81:13`,
  `+=` to `*=` in `line_doc`:
  unreachable;
  the branch is removed.
- `92:5`,
  `run_margin` body replaced by `1`:
  killed by `docs::processors_keep_relative_indentation_in_line_doc_runs_without_a_margin`.
- `101:40`,
  `-` to `+` in `run_margin`:
  killed by `docs::processors_keep_relative_indentation_in_line_doc_runs_without_a_margin`.
- `101:40`,
  `-` to `/` in `run_margin`:
  killed by `docs::processors_keep_relative_indentation_in_line_doc_runs_without_a_margin`.
- `123:43`,
  `-` to `+` in `block_margin`:
  killed by `docs::processors_strip_only_the_common_margin_from_undecorated_block_docs`.
- `123:43`,
  `-` to `/` in `block_margin`:
  killed by `docs::processors_strip_only_the_common_margin_from_undecorated_block_docs`.
- `152:30`,
  `&&` to `||` in `block_doc`:
  killed by `docs::processors_strip_one_conventional_space_from_block_doc_opening_lines`.
- `152:25`,
  `==` to `!=` in `block_doc`:
  killed by `docs::processors_strip_one_conventional_space_from_block_doc_opening_lines`.
- `153:21`,
  `+=` to `-=` in `block_doc`:
  killed by `docs::processors_strip_one_conventional_space_from_block_doc_opening_lines`.
- `153:21`,
  `+=` to `*=` in `block_doc`:
  killed by `docs::processors_strip_one_conventional_space_from_block_doc_opening_lines`.
- `155:39`,
  `-` to `+` in `block_doc`:
  killed by `docs::processors_keep_non_ascii_whitespace_lines_outside_stripped_margins`.

The two `-` to `+` mutants are equivalent on every line that is empty,
ASCII-only whitespace,
or has content,
because the result is capped by the common margin.
They differ only on a line of non-ASCII whitespace,
such as U+00A0 or U+3000:
the margin computation skips that line as blank,
while the per-line strip must leave its bytes in the payload.
Under either mutant the strip consumes the character or splits it.

### `src/processors_rewrite.rs`

12 missed mutants.

- `16:5`,
  `edit_key` body replaced by `(0, 0)`:
  killed by `rewrite::processors_project_groups_supplied_in_descending_order`.
- `16:5`,
  `edit_key` body replaced by `(0, 1)`:
  killed by `rewrite::processors_project_groups_supplied_in_descending_order`.
- `16:5`,
  `edit_key` body replaced by `(1, 0)`:
  killed by `rewrite::processors_project_groups_supplied_in_descending_order`.
- `16:5`,
  `edit_key` body replaced by `(1, 1)`:
  killed by `rewrite::processors_project_groups_supplied_in_descending_order`.
- `29:13`,
  `||` to `&&` in `rewrite`:
  killed by `rewrite::processors_refuse_reversed_and_one_sided_split_edit_ranges`.
- `30:13`,
  `||` to `&&` in `rewrite`:
  killed by `rewrite::processors_refuse_reversed_and_one_sided_split_edit_ranges`.
- `31:13`,
  `||` to `&&` in `rewrite`:
  killed by `rewrite::processors_refuse_reversed_and_one_sided_split_edit_ranges`.
- `42:25`,
  `>` to `==` in `rewrite`:
  killed by `rewrite::processors_refuse_edits_between_the_bytes_of_an_authored_crlf`.
- `42:25`,
  `>` to `<` in `rewrite`:
  killed by `rewrite::processors_refuse_edits_between_the_bytes_of_an_authored_crlf`.
- `42:25`,
  `>` to `>=` in `rewrite`:
  killed by `rewrite::processors_project_insertions_before_a_leading_line_feed`.
- `43:58`,
  `==` to `!=` in `rewrite`:
  killed by `rewrite::processors_refuse_edits_between_the_bytes_of_an_authored_crlf`.
- `44:53`,
  `-` to `/` in `rewrite`:
  killed by `rewrite::processors_refuse_edits_between_the_bytes_of_an_authored_crlf`.

### `src/processors.rs`

3 missed mutants.

- `68:9`,
  `VirtualSource::is_rustdoc` body replaced by `true`:
  killed by `prepare::processors_mark_only_comment_derived_markdown_as_rustdoc`.
- `95:17`,
  `||` to `&&` in `VirtualSource::project_diagnostic`:
  killed by `spans::processors_refuse_labels_with_one_endpoint_inside_a_character`.
- `96:17`,
  `||` to `&&` in `VirtualSource::project_diagnostic`:
  killed by `spans::processors_refuse_labels_with_one_endpoint_inside_a_character`.

### `src/processors_prepare.rs`

3 missed mutants.
The existing tests checked substrings of the prepared text,
which hold whether or not a bare marker is stripped or a helper function suppresses the wrapper.
The new tests compare the whole prepared string.

- `30:17`,
  `&&` to `||` in `has_main`:
  killed by `prepare::processors_wrap_fragments_whose_only_functions_are_helpers`.
- `57:21`,
  `+=` to `*=` in `hidden`:
  killed by `prepare::processors_strip_bare_hidden_markers_to_empty_lines`.
- `57:31`,
  `+` to `*` in `hidden`:
  killed by `prepare::processors_strip_bare_hidden_markers_to_empty_lines`.

### `src/processors_projection.rs`

3 missed mutants.

- `69:17`,
  `<` to `>` in `edit_to_parent`:
  killed by `rewrite::processors_name_generated_text_in_refusals_that_reach_synthetic_main`.
- `85:22`,
  `==` to `!=` in `edit_to_parent`:
  killed by `rewrite::processors_project_line_deletions_between_differently_indented_prefixes`.
- `132:64`,
  `&&` to `||` in `verify`:
  killed by `rewrite::processors_refuse_rewrites_whose_container_disappears_or_regenerates`.

The `69:17` mutant still refuses the group,
because container re-extraction rejects it in the next step.
Only the refusal reason changes,
so the test asserts the exact message.

The `85:22` mutant produces identical host bytes whenever the deleted line and the following line
have the same prefix,
which every existing deletion test used.
The new test deletes a line whose comment marker is indented differently from its neighbours.

### `src/processors_fences.rs`

1 missed mutant.

- `73:32`,
  `+` to `*` in `fences`:
  unreachable difference;
  the redundant arithmetic is removed.

"Source changes" holds the evidence.

### `src/processors_model.rs`

1 missed mutant.

- `52:9`,
  `Display::fmt` body replaced by `Ok(Default::default())`:
  killed by `spans::processors_anchor_refusals_at_authored_containers_and_render_them`.

### Timeouts

- `src/processors_docs.rs:223:23`,
  `+=` to `*=` in `docs`,
  at `222:23` after the carriage-return branch was removed:
  the timeout is the detection.
- `src/processors_lines.rs:45:20`,
  `+=` to `*=` in `physical_lines`:
  the timeout is the detection.

Both mutations replace the only state change of a loop with a multiplication by one.
In `docs`,
the mutated statement runs only when the next comment joins the current run,
and then repeats the same comparison forever.
In `physical_lines`,
it runs on the first byte that is not a newline.
An input that never reaches the statement behaves exactly like the original.
An input that reaches it never returns.
No input yields a wrong value that an assertion could observe,
so no fast failing test can exist,
and the campaign timeout of 180 seconds is the only possible detection.

The baseline logs show the mechanism.
Under the `docs` mutant,
`edges::processors_extract_and_project_indented_block_and_line_doc_fences` and
`edges::processors_keep_no_op_groups_byte_identical` were still running after 60 seconds;
both extract a run of two `///` lines or more.
Under the `physical_lines` mutant,
the first two processor tests to start were still running after 60 seconds and no test had finished.

Removing these timeouts would need a loop shape whose mutations cannot spin,
such as a `for` over the remaining comments.
That is a restructuring outside this delegation,
so it is left as an option.

## Source changes

### Unreachable carriage-return branch in `line_doc`

`line_doc` extended a copied line past its comment token with three cases:
a following CRLF,
a following bare carriage return,
or a following line feed.
Only the line feed can occur.
The native lexer ends a line comment at the next line feed or at the end of input:
`line_comment` in `ra-ap-rustc_lexer` 0.165.0 (`src/lib.rs`,
the version pinned in `Cargo.lock`)
calls `eat_until(b'\n')`,
which advances to the first line feed byte or to the end.
A carriage return before that line feed is therefore the last byte of the comment token,
not the byte after it.

CRLF runs still copy their carriage return,
because it is inside the token.
`docs::processors_keep_crlf_pairs_whole_in_line_doc_runs` pins that,
including for an empty comment line.
Both `81:13` mutants passed all 58 processor tests,
old and new,
before the branch was removed.
The remaining `end += 1` was planted as `end -= 1` and `end *= 1`;
both fail.

### Redundant offset in the fence line-count refusal

`fences` refused a code node when `decoded_lines.len() + 1 > lines.len()`.
For unsigned lengths that is `decoded_lines.len() >= lines.len()`,
which is what the code now says.
The refusal and its message are unchanged.

The surviving mutant weakened the refusal to allow equal counts.
Equal counts cannot occur with the pinned parser,
`satteri-pulldown-cmark` 0.6.3:

- `parse_fenced_code_block` (`src/firstpass.rs`) appends one text item per body line,
  through `append_code_text`,
  covering that authored line and nothing else.
- `append_code_text` adds at most three synthesized spaces for a partially consumed tab,
  never a line ending.
- The arena builder (`src/arena_build.rs`) concatenates those verbatim slices into the code value
  and drops one trailing line ending.

Every line ending in the decoded value is therefore an authored line ending of the fence body,
and the authored node also contains the opening fence line.
Authored lines always outnumber decoded lines.

A generated probe supports the reading.
It ran 120,000 random Markdown,
MDX,
and Rustdoc-wrapped hosts built from fence markers,
container markers,
indentation,
tabs,
and the three newline spellings.
The refusal never fired and nothing panicked;
the sibling refusal for partially expanded tabs fired 117 times.
The probe source and log are in the `hand-plants` directory.

The equality boundary itself remains without a test,
because no input reaches it.
A hand-planted `>=` to `>` control confirms that:
it passes all 58 processor tests.
`cargo-mutants` 27.1.0 emits only `<` for `>=`:
the baseline campaign lists exactly one replacement for each existing `>=`,
at `src/processors.rs:195:27` and `src/processors_projection.rs:59:49`.
That mutant of the new comparison fails many tests.
Reverting the one-line change restores a visible survivor,
if a recorded exclusion is preferred over the simplification.

## Defects and gaps found

No processor source defect was found:
no mutant led to an input with a wrong host position or an accepted corrupting fix in the unmutated code.

The gaps were in the tests,
and three are worth knowing about:

- The CRLF refusal in `rewrite` is the only protection against one corrupting fix.
  A probe planted its removal,
  then inserted `X` between the carriage return and line feed of `/// Alpha.\r\n`.
  The group was accepted and the fixed host read `/// Alpha.\rX\n`,
  a bare carriage return followed by a new line,
  because the carriage return stays inside the comment token and the comment re-extracts
  to the intended bytes.
  The unmodified source refuses the same group with this refusal.
  The existing test asserted only that some refusal happened,
  and container verification supplied one for the shape it used.
  The probe and both logs are in `hand-plants` as `processors_probe_crlf_split.rs`,
  `run4-original.log`,
  and `run4-planted.log`.
- Deleting a whole doc line was only tested between lines with identical prefixes,
  where deleting the prefix of the wrong line gives the same bytes.
- No test asserted any value returned by `processors_spans::anchor`,
  so every refusal offset and every rendered `ProcessorError` was unpinned.

## Rerun evidence

The rerun mutated commit `e7e5b24aa`.
Later commits change only this document,
and a hash check inside the tested image found its processor sources,
fixture,
`src/lib.rs`,
`Cargo.toml`,
and `Cargo.lock` byte-identical to commit `f5744ad3c`.

Command,
run from `package/linter/monochromatic-lint` in the worktree:

```sh
# package/linter/monochromatic-lint
MONOCHROMATIC_LINT_IMAGE_TAG=processor-survivors mise run //package/linter/monochromatic-lint:mutation:processors:files
```

The task first ran the full container suite,
which passed 228 tests,
then container Clippy with warnings denied,
which reported no warnings.

- Tested source image,
  `localhost/monochromatic-lint-test:processor-survivors`:
  `48f1112dda8a331248860032ba641dc900154d87f7522225c829826d5e5cb126`.
- Campaign image,
  `localhost/monochromatic-lint-mutation:processor-survivors`:
  `40b37f300a715458f8b86a19d5dd41c245bf59cdc33fa8e9c8ed4042e15997c8`.
- `cargo-mutants` 27.1.0,
  executable SHA-256
  `f985f265ee3ea3e453aa98b04c52134953911f692f8ce8abf137f3873202a2d0`,
  the same executable as the baseline campaign.
- Bounds:
  no network,
  no mounts,
  2 GiB memory,
  2 CPUs,
  128 processes,
  300 second build timeout,
  180 second test timeout.
- Duration:
  97 seconds of baseline build and 28 minutes of mutants,
  1,985 seconds for the whole task including the full suite and Clippy.
- Exit status 3,
  which `cargo-mutants` 27.1.0 reserves for timeouts (`src/exit_code.rs`, `ExitCode::Timeout`).
  Missed mutants would give status 2.
  The runner reports any nonzero status as a failed task,
  so the task fails while these 2 timeouts remain.

The report directory is `target/verification/mutation-w00nRd` in the worktree,
copied to `package/linter/monochromatic-lint/target/verification/processor-survivors-mutation-w00nRd`
in the main checkout.
Its `hand-plants` directory holds the planting driver,
its plans and results,
the reachability probes and their logs,
and the log of the whole task.

One further probe,
`processors_probe_emptied_block_docs.rs`,
checked the accepted path in which a container disappears and the intended virtual text is empty.
Emptying the body of `/**X*/` produces `/***/`,
which the lexer treats as an ordinary comment,
so the item loses its documentation as requested and the host stays valid Rust.
Block delimiters are never deleted by design,
so this is the expected result,
not a defect.

## Operational notes

- The first launch of the rerun task was stopped during its test-image build,
  because the launching tool call had a short timeout,
  and was relaunched unchanged.
  No mutant had started.
- Every test,
  planting,
  probe,
  and campaign container was mount-free.
  The one exception is the source-identity hash check,
  which mounted a single checksum file read-only into the tested image to run `sha256sum --check`.
- Plants and probes used one container at a time,
  never concurrently with the campaign.

## Notes for integration

- New test modules:
  `src/processors_spans_tests.rs`,
  `src/processors_docs_tests.rs`,
  `src/processors_rewrite_tests.rs`,
  and `src/processors_prepare_tests.rs`,
  registered from `src/processors_tests.rs`.
  `src/lib.rs` is unchanged.
- The `format:processors` task lists its files explicitly and does not include the new modules.
  They were formatted with the package `format:rust` task,
  which changed no other file.
- New task `test:processors:slice` runs only the processor tests in the tagged container image.
  It respects `MONOCHROMATIC_LINT_IMAGE_TAG` through `bin/test-container.mjs`.
- No dependency,
  lockfile,
  fixture,
  or `src/lib.rs` change.
