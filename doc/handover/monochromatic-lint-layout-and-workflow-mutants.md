# Linter layout, workflow-shell and semantic mutation campaigns

## Purpose and scope

This record holds the first runs of the three mutation scopes of `package/linter/monochromatic-lint` that had never been run:
`--layout` (`src/layout_*.rs` against the `layout_` tests),
`--workflow-shell` (`src/workflow_*.rs` and `src/shell_*.rs` against the `workflow_` and `shell_` tests),
and the two files that `--semantic` gained (`src/rust_manifest.rs` and `src/rust_file_engine_syntax.rs`).
It dispositions every survivor and timeout,
names the tests that kill them or the code that makes them equivalent,
and lists what changed in the package to make the campaigns possible.

The owner's words that bound the work:
"Principle 1: Lint issues need to be taken care of asap."
and "Mutation runs don't block publish and release."
Nothing here releases the package.

Inspect the survivor dispositions below,
the units and logs,
and the open items at the end.
Respond by naming a disposition that the evidence does not support.

The work is on branch `test/monochromatic-lint-layout-and-workflow-mutants`,
created from `main` at `17da6bdbe8`.

## What stopped the campaigns before they started

Every mutation task depends on `lint:container`,
which runs the whole library and binary suite.
The suite was red on the released pre-release version:
a stale-binary hint compared the running version with a lockfile pin,
and `numeric_triple` in `src/config_pin.rs` refused any version with a suffix.
At `0.5.2-alpha.1`,
`config_pin::tests::the_hint_names_a_measured_newer_pin_and_nothing_else`,
`config_pin::tests::unmeasured_pins_and_farther_lockfiles_produce_no_hint`
and `binary::a_newer_repository_pin_adds_the_stale_binary_hint` failed.

Measured:

- A container run on this branch at `17da6bdbe8` stopped at the first of them (140 of 869 tests run).
- A container run on a build of `main` at `6720c6e7dc` (version `0.5.2-alpha.2`) failed the first two (log `measure-main.log`).
- A container run on this branch at version `0.5.2-alpha.3` failed the built-executable one
  (log `pin-red-binary.log`:
  the executable printed no hint for a pin of `1.0.0`).

The cause is the manifest version, not a live value:
the running version comes from `env!("CARGO_PKG_VERSION")`,
and every alpha made the comparison give up.
Two commits first made the controls independent of the manifest version
(a fixed running version passed into `stale_binary_hint_with`).
The coordinator then asked for semantic-versioning order,
which was done test-first and merged into `main` on its own:
merge `1b1aa901b3`,
branch `fix/monochromatic-lint-stale-hint-prerelease-order`
(its container run on its own: 886 of 886 passed).

Known limit that stays:
the hint still says nothing when either version has an empty pre-release or an empty identifier
(`1.0.0-`, `1.0.0-alpha..1`),
and a numeric identifier with a leading zero is read as a number.

## Campaign units, caps and logs

All logs are in `/home/user/temp/agent/lint-mutation/`.
Every unit ran `mise run //package/mutation-test/cli-cargo-mutants:campaign:gate-slot -- mise run <package task>`
with the launcher's own unit caps,
`MemoryMax=12G`,
`CPUQuota=500%`,
`TasksMax=256`
and `IOWeight=10`,
and the task's container defaults (2 GiB, 2 CPUs, 512 processes).
The first minutes of the first units measured a memory peak of 1.0 to 1.4 GB and 54 tasks on the host side,
so the defaults were kept.
The host load average was 32 to 58 on 28 threads for the whole run.

The linter's adapter does not take the launcher's `--timeout`,
so the launcher's `campaign:parallel` command was not used;
the units were started with `systemd-run --user` by hand,
each with its own image tag (`MONOCHROMATIC_LINT_IMAGE_TAG`).
Two campaigns ran at once from the start
(the brief named an order;
the units shared the limiter's six slots),
and the semantic scope started a few minutes later.

- Layout: unit `lint-mutation-layout`,
  log `layout-run2.log` (the first attempt, `layout.log`, stopped at the failing pin test above),
  evidence `package/linter/monochromatic-lint/target/verification/mutation-03e5a166d284`.
- Workflow-shell, sequential run: unit `lint-mutation-workflow-shell`,
  log `workflow-shell-run2.log`,
  evidence `mutation-a8d16d57fbc0`.
  Its single unit would have needed about seven hours,
  so it was stopped by `systemctl --user stop` when it reached the first POSIX scanner mutant,
  after it had scored the dialect,
  rule settings,
  rules,
  scan dispatch and `cmd` scanner files.
- Workflow-shell, the rest of the scope as three more campaigns of the same task with `--re`:
  `lint-mutation-workflow-files` (`--re 'src/workflow_'`, log `workflow-shell-part-workflow-files.log`, evidence `mutation-0be5eee05be0`),
  `lint-mutation-shell-powershell` (`--re 'src/shell_scan_powershell.rs'`, log `workflow-shell-part-powershell.log`, evidence `mutation-6d7e31f75682`),
  `lint-mutation-shell-posix` (`--re 'src/shell_scan_posix.rs'`, log `workflow-shell-part-posix.log`, evidence `mutation-f2d95e99588f`).
  Each mutant was scored once:
  the union of the four runs names 740 mutants of the 741 listed,
  without a conflict between runs,
  and the 741st is the one unclassified failure below.
- Semantic, four shards of one campaign:
  units `lint-mutation-semantic-shard0` to `lint-mutation-semantic-shard3`,
  logs `semantic-shard0.log`,
  `semantic-shard1.log`,
  `semantic-shard2.log`,
  `semantic-shard3-run2.log`
  (`semantic-shard3.log` is the first attempt of shard 3,
  which died before scoring anything with podman's `database is locked` at the container start,
  exit 125;
  the rerun is the resume the launcher documents),
  evidence `mutation-468941331fe9` (0/4),
  `mutation-00042aa98dfb` (1/4),
  `mutation-7ae75cbd432f` (2/4),
  `mutation-a0492939e32b` (3/4).
  The other three shards ran with `mise run --skip-deps` after shard 0 built the image.
- Single-mutant reruns after the tests:
  `red-*.log`,
  `red2-*.log`,
  `red3-*.log`,
  `red4-workflow.log`,
  `red5-workflow.log` and `red-syntax-2.log`,
  with `mise run --skip-deps <task> -- --re <regex of the survivors>`
  (`-- --re` is the adapter's single-mutant route;
  the launcher's own `--examine` flag is the same idea for its `campaign:parallel` command).

Every unit has ended or been stopped,
and none was left running.

## Results per scope

Layout (`--layout`, 415 mutants, 137 `layout_` tests before the new ones):
326 caught,
39 missed,
1 timeout,
49 unviable.
Survivors and the timeout: 40.

Workflow-shell (`--workflow-shell`, 741 mutants):
532 caught,
115 missed,
67 timeouts,
26 unviable,
and 1 unclassified failure
(`src/workflow_walk.rs:234:33: replace + with - in Walker<'_>::on_key`:
cargo-mutants wrote `Unclassified failure` and put it in none of the four lists;
the mutant makes the walk revisit a line forever).
Survivors, timeouts and the failure: 183.

Semantic (`--semantic`, 103 mutants over eleven files,
the 11 new ones among them):
77 caught,
4 missed,
0 timeouts,
22 unviable.
All four survivors are in `src/rust_file_engine_syntax.rs`,
which is compiled only without the `semantic` feature
and so was never built by a campaign that uses the default features.

Syntax-only (`--syntax-only`, a new scope for that one file, 4 mutants):
2 caught,
1 missed,
1 unviable.

## The new scope for the syntax-only router

`src/rust_file_engine_syntax.rs` is behind `#[cfg(not(feature = "semantic"))]`.
No gate compiled it:
`cargo check`,
Clippy,
the container suite and the semantic campaign all use the default features.
Its four mutants survived the semantic campaign for that reason,
not because tests were weak.

What was added:

- The test target `syntax_only_engine` (`src/syntax_only_engine_tests.rs`),
  empty with the feature and four controls without it,
  run by the new task `test:syntax-only`.
- The scope `--syntax-only` (tasks `mutation:syntax-only` and `mutation:list:syntax-only`),
  which builds with `--no-default-features` and builds and runs only that target
  (the library's own tests do not compile without the feature:
  `run_workers_tests` and `run_directive_tests` import semantic-only modules unconditionally).
  The file left `--semantic`.
  The first run of this scope failed its baseline for that reason;
  the second is the one counted.
- One mutant of this scope cannot be killed:
  `replace RustFileEngine::workspace_count -> usize with 0`,
  because the function already returns `0`.
  A campaign of that scope therefore always exits with the missed-mutant status.
  Equivalent;
  open to the owner's veto
  (the alternative is an exclusion that `mutation:coverage` would have to learn).

The other three: `workspace_count -> 1` and `check -> Ok(vec![])` are caught,
`check -> Ok(vec![Default::default()])` is unviable (`Diagnostic` has no `Default`).

## How hanging mutants are killed

Many survivors were loops that stop advancing (`index += 1` replaced by `index -= 1`,
a step by `index * 1`,
a function replaced by a constant index).
They never fail an assertion,
and cargo-mutants scores them as a timeout.
Two things were needed:

- `src/test_deadline.rs` (`finished_within_deadline`) runs a probe on its own thread
  and fails the test when it panics or does not finish in thirty seconds
  (the slowest scan test measured under 0.02 s,
  the whole container suite 105 to 145 s).
- Every test that can reach the code ran under it,
  because nextest lets tests that already run finish after the first failure,
  so one unbounded test beside a failing one kept the run until the 180 second limit.
  The single-mutant rerun of `loads_itself -> false` showed it:
  the new test failed after 33 s,
  and the older test beside it ran until the limit.
  The tree builder of the layout tests (`tree_of`),
  every scanner test,
  the workflow walk tests,
  the rule,
  extraction and end-to-end workflow tests now use the deadline.

## Layout survivors and their dispositions

Killed by a test (24).
Each test is in the sidecar of its module and was shown red against the mutant by a single-mutant rerun
(`red-layout-shard0.log`,
`red-layout-shard1.log`,
`red2-layout.log`,
`red3-layout.log`):

- `layout_crate_roots.rs` `rust_files_in` (`&&` to `||`):
  `layout_only_regular_rust_files_of_the_bin_folder_are_roots`.
- `layout_file_order.rs` `uses_between` (`<` to `==`, `>`):
  `layout_the_smallest_proof_of_a_use_between_two_files_wins_in_any_order`.
- `layout_finding.rs` `shown` (guard to `true`):
  `layout_the_package_folder_itself_is_shown_whole` (new sidecar `layout_finding_tests.rs`).
- `layout_folder_links.rs` `glob_reexports` (`||` to `&&`):
  `layout_two_globs_on_one_line_are_one_finding_and_a_self_glob_is_none`.
- `layout_folder_uses.rs` `record_use` (`<` to `==`, `>`):
  `layout_the_smallest_proof_of_a_folder_use_is_kept_in_any_order` (new sidecar `layout_folder_uses_tests.rs`).
- `layout_group_summary_links.rs` `summaries` (`<` to `==`, `>`, `<=`):
  `layout_the_missing_summary_is_reported_on_the_smallest_proof_of_two_children`,
  `layout_the_smallest_proof_wins_among_the_folders_one_child_uses_in_a_group`,
  `layout_one_reference_that_reaches_two_folders_of_a_group_names_the_first`
  (the last one shows that `<=` is not equivalent here:
  one reference through a re-exporting folder gives two uses with an equal proof).
- `layout_group_summary_links.rs` `extra_link` (guard to `true`):
  `layout_a_summary_link_to_a_folder_of_its_own_group_is_an_unneeded_link`.
- `layout_link_entries.rs` `read_links` (guard to `true`):
  `layout_a_uses_path_that_cannot_be_read_is_reported_and_not_skipped`.
- `layout_module_tree.rs` `loads_itself -> false` (the timeout):
  `layout_a_file_that_loads_itself_ends_the_tree_build`, with `tree_of` under the deadline.
- `layout_path_resolution.rs` `root_of -> 0`:
  `layout_a_crate_path_in_a_second_root_names_that_root`.
- `layout_reference_paths.rs` `read_group` guard and `+`,
  `read_entry` (the lone-`self` test, the glob branch, the segment branch, the glob step),
  `read_references` re-export range end:
  `layout_entries_after_a_nested_group_are_still_read`,
  `layout_a_group_that_is_never_closed_ends_with_the_source`,
  `layout_group_entries_of_several_segments_and_globs_are_read_whole`,
  `layout_a_self_path_inside_a_group_keeps_its_self_segment`,
  `layout_a_path_right_after_a_public_use_is_not_a_reexport`.
- `layout_step.rs` `line_span` (`==` to `!=`):
  `layout_the_span_of_a_line_starts_at_that_line`.

Equivalent (16), each with its reason in the code:

- `layout_file_order.rs:113` `uses_between` and `layout_folder_uses.rs:78` `record_use`, `<` to `<=`:
  the value is replaced only when the two proofs are equal,
  and `Proof` derives `Eq`,
  so the map is written with an identical value.
- `layout_group_summary_links.rs:91` `summaries`, `<` to `<=` is not in this list:
  it is killed (see above).
- `layout_file_order.rs:275` `list_findings`, `<` to `<=`:
  equal places need equal names,
  `listed` is built from a `BTreeSet` of the files of one folder (`files_by_folder`),
  so its names are distinct,
  and `uses_between` skips an edge from a file to itself (`if from == to`).
- `layout_reference_paths.rs:264` (`delete match arm TokenKind::Comma`) and `layout_source_facts.rs:105` (`delete match arm SyntaxKind::COMMA`):
  a comma falls to `read_entry`'s stray-token branch,
  which returns `next: start + 1`,
  and `read_group` takes `entry.next.max(index + 1)`,
  which is the step the comma arm made.
- `layout_reference_paths.rs:275` (`index + 1` to `index - 1`, `index * 1`),
  `:303` and `:310` (the `next` of the glob entry and of a stray token),
  `:380` (the `next` after a glob) and `:431` (`read_references`' floor):
  every `next` is at least the start of its read plus one,
  every caller floors it with `.max(index + 1)`,
  and a pointer left on `*` or `::` cannot start a path
  (`starts_path` needs `is_segment`).
- `layout_reference_paths.rs:413` `<` to `<=`:
  at `index == tokens.len()`,
  `kind_at` answers `Other`,
  `starts_path` is false and the loop ends after one more step.

## Workflow-shell survivors and their dispositions

Killed by a test (179 of the 183).
The killing tests are named after their file;
every one was shown green in the container and red against the mutant by the single-mutant reruns
(`red-workflow-shard*.log`,
`red2-workflow-shard*.log`,
`red3-workflow*.log`,
`red4-workflow.log`):

- `shell_dialect.rs` `Construct::token`:
  `shell_a_construct_finding_quotes_the_token_as_written`.
- `shell_rule_settings.rs` `is_empty`:
  `shell_settings_with_only_one_kind_of_rule_are_not_empty`.
- `shell_rules.rs` `is_comment_line`:
  `shell_a_bare_rem_line_is_a_cmd_comment_and_other_lines_count`.
- `shell_scan.rs` `expression_end` (two timeouts, bound and brace arithmetic):
  `shell_an_expression_ends_after_its_closing_braces_or_at_the_end_of_the_body`,
  `an_expression_ends_after_its_own_closing_braces_whatever_surrounds_it`.
- `shell_scan_cmd.rs` (byte ranges, start-of-body and mid-line comment words, the redirect, the caret, three timeouts):
  `operators_are_reported_over_their_own_bytes`,
  `an_ampersand_at_the_start_is_an_operator_and_one_after_a_redirect_is_not`,
  `a_caret_at_the_start_escapes_the_next_byte`,
  `a_comment_word_is_a_comment_only_at_the_start_of_a_line`.
- `shell_scan_posix.rs` (here-documents, quoting resumption, arithmetic, substitutions, parentheses, input redirections, 25 timeouts):
  `heredoc_openers_skip_exactly_their_own_bodies`,
  `text_that_is_not_a_heredoc_opener_leaves_the_next_lines_as_code`,
  `quoting_resumes_scanning_at_the_byte_after_the_quote`,
  `arithmetic_and_substitutions_end_where_their_parentheses_do`,
  `a_substitution_ends_at_its_own_closing_parenthesis`,
  `arithmetic_ends_at_its_own_closing_parentheses`,
  `heredoc_words_that_are_empty_or_one_character_long_end_their_own_body`,
  `an_input_redirection_opens_no_heredoc`.
- `shell_scan_powershell.rs` (quotes, here-strings and their openers, comments, parentheses, 14 timeouts):
  `operators_are_reported_over_their_own_bytes`,
  `single_quotes_resume_scanning_at_the_byte_after_them`,
  `double_quotes_resume_scanning_at_the_byte_after_them`,
  `here_strings_end_at_their_closer_and_report_every_live_substitution`,
  `an_at_sign_that_opens_no_here_string_is_text`,
  `comments_end_where_they_end`,
  `a_sub_expression_ends_at_its_own_closing_parenthesis`,
  `a_less_than_sign_is_not_a_block_comment`,
  `here_strings_ignore_the_quotes_of_their_body`,
  `a_live_here_string_reports_a_substitution_right_after_another`,
  `an_empty_single_quoted_string_ends_at_its_second_quote`,
  `a_hash_comments_only_after_a_separator_and_a_block_comment_may_be_empty`,
  `a_parenthesis_in_double_quotes_is_text_unless_a_dollar_precedes_it`.
- `workflow_lines.rs`: `a_tab_is_refused_at_its_own_byte_after_deep_indentation`,
  `a_hash_starts_a_comment_only_after_a_space`.
- `workflow_run.rs` `block_lines`:
  `blank_lines_inside_a_block_keep_their_payload_inside_their_own_text`.
- `workflow_walk.rs` (20 timeouts and the unclassified failure, killed by the deadline;
  `skip_value`, `odd_key_matters`, `open_dashes`, the refusal offset):
  `an_empty_value_and_a_one_line_flow_collection_end_on_their_own_line`,
  `a_flow_collection_runs_to_the_line_where_its_brackets_balance`,
  `a_scalar_continues_over_deeper_lines_and_skippable_lines`,
  `a_nested_scalar_stops_at_its_own_sibling`,
  `an_odd_key_is_judged_by_the_keys_above_its_column`,
  `a_step_is_anchored_at_its_dash`,
  `a_refused_structural_value_is_reported_at_its_own_byte`,
  and every walk control now runs under the deadline.

Equivalent (4):

- `shell_scan_posix.rs:126` `heredoc_opener`, `||` to `&&` (between the `<<<` guard and the `<<(` guard):
  a `<<<` or `<<(` no longer returns false at the guard,
  and the word check that follows returns false for both,
  because `<` and `(` are in the set that ends a word,
  so `cursor == word_start`.
- `shell_scan_posix.rs:310` `in_code`, guard `!self.heredocs.is_empty()` to `true`:
  with no pending here-document,
  `skip_heredoc_bodies` loops over nothing and sets `index` to `position`,
  which is the `index + 1` the plain advance makes.
- `workflow_walk.rs:182` `on_key`, `index + 1` to `index - 1`, `index * 1`:
  that return is the `else` of `let Content::Key { .. } = &entry.content`,
  and the only caller (`step`) reaches `on_key` through the `Content::Key { .. }` arm.

The mutant `shell_scan_posix.rs:125` (`||` to `&&` between the `ahead(1) != '<'` test and the `ahead(2) == '<'` test)
is not equivalent:
a plain `<` then opened a here-document.
It is killed by `an_input_redirection_opens_no_heredoc` (`red5-workflow.log`).

## Other changes in the package

- `src/config_pin.rs`: semantic-versioning order (merged separately as `1b1aa901b3`).
- `src/closure_cargo.rs`: rustdoc on the two fields of `ProbeFailure` and a named status predicate instead of a capture-free closure,
  both reported by the repository linter (`rust/require-rustdoc`, `rust/avoidable-closure`);
  behavior unchanged.
- `src/lib.rs`: the module lines of `markdown_tables` and `markdown_trim_space` in rustfmt's order,
  and the new test modules.
- Four test files got semantic line breaks in their docs for the repository linter.
- The package `README.md` names `mutation:syntax-only` and its one mutant no test can kill.

## Gates

- `mise run //package/linter/monochromatic-lint:test` on the host: 930 passed at the middle of the work;
  the final run is in the report that handed this record over.
- `lint`,
  `lint:types` and `lint:rustfmt`: pass.
- `mise run lint:monochromatic-lint -- package/linter/monochromatic-lint`: exit 0 with the installed `0.5.2-alpha.5`.
- Container suite: 941 of 941 passed on the last image.

## Open items

- The semantic campaign ran before the file left `--semantic`,
  so its four missed mutants are of the old scope;
  the new scope is what a rerun of `mutation:semantic` will list.
- A campaign of `--syntax-only` always exits nonzero (the equivalent `workspace_count` mutant);
  an owner decision on an exclusion would change `mutation:coverage` too.
- The deadline is thirty seconds in `src/test_deadline.rs`;
  its comment cites the measurement.
