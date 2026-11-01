# Unified-linter mutation close

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/rust-module/jsonc-edit` is now `package/jsonc-edit/core-rust`.

Note 2026-10-30: the fuzz sidecar scripts `package/linter/monochromatic-lint.fuzz/bin/container.mjs` and
`planted-controls.mjs` are TypeScript files now; read those names here as `container.ts` and `planted-controls.ts`.

## Purpose and how to respond

Mutation testing is a release acceptance gate for the native linter in `package/linter/monochromatic-lint`:
a surviving non-equivalent mutant needs a stronger test,
an equivalent or unreachable one needs evidence,
and a campaign that exits nonzero is not a passing gate.
This handover removes the timeouts that kept the Markdown and processor campaigns exiting 3,
runs the first campaign over the executable's own modules,
and reruns the Markdown and processor campaigns on the final snapshot.
Its second round,
`Gaps round`,
gives every production module a scope,
proves that from listings,
lints every invocation on a thread with an explicit stack,
and checks the fuzz sidecar against the final library.

Inspect `Excluded mutation kinds` first,
because it changes what every campaign measures.
Then inspect `Timeouts removed`,
because it changes production code and adds two error paths,
then `Gaps round`,
because it changes the executable's threads and the scopes every campaign runs,
and read `Defects found` and `Remaining`.
Respond with a veto of a named change or disposition,
or with the next scope to mutate.

## Result

Every production mutant was tried against one test image,
`7526e26a714499174ff012c45175059ac39127333510cc8fa405264c902499f7`,
built from linter source tree `c5f934d97d7579d9eca66741241ecc47291cc877`:
no campaign missed a mutant,
and the one timeout,
in the executable scope,
was caught when its file was rerun once on the same image.
The scopes hold all 1,650 mutants of the unscoped listing with both exclusion patterns,
name by name (`coverage-potTv8`),
and every one of those names is in the outcomes of a campaign below.
Each campaign passes both exclusion patterns;
the semantic scope has a 1,200 second per-mutant limit and every other scope 180 seconds.

- Executable scope,
  `mutation-zRZK4y`:
  239 mutants,
  178 caught,
  60 unviable,
  1 timeout,
  exit status 3.
  The timeout is `src/path_inputs.rs:21:22: replace == with != in has_glob`,
  whose own test failed at once while two unrelated concurrency tests ran past 60 seconds on a loaded host.
  The rerun of that file's 25 mutants on the same image,
  `mutation-m8iNlm`,
  caught 24 and found 1 unviable,
  with no timeout and exit status 0;
  the mutant failed its test in 1.1 seconds there.
- Core scope,
  `mutation-ZFIBGQ` and `mutation-URkAcl` (shards 0/2 and 1/2):
  202 mutants,
  182 caught,
  20 unviable,
  exit status 0 for both.
- Semantic scope,
  `mutation-9mpnyq`,
  `mutation-ZtKdA6` and `mutation-Mq0oAq` (shards 0/3 to 2/3):
  97 mutants,
  75 caught,
  22 unviable,
  exit status 0 for all three.
- Markdown scope,
  `mutation-M41Su2` and `mutation-9AM99n` (shards 0/2 and 1/2):
  742 mutants,
  698 caught,
  44 unviable,
  exit status 0 for both.
- Processor scope,
  `mutation-URvHN4`:
  355 mutants,
  330 caught,
  25 unviable,
  exit status 0.
- Constant-slot scope,
  `mutation-bj1zDp`:
  12 mutants,
  10 caught,
  2 unviable.
- Parent-lookup scope,
  `mutation-Ah0n0W`:
  4 mutants,
  4 caught.
- Anonymous-function scope,
  `mutation-F5vbwg`:
  3 mutants,
  2 caught,
  1 unviable.
- The five planted guard removals of the inline `mutation:processors` task,
  `processors-mutation-NREykG`:
  5 caught,
  exit status 0,
  on processor image `40bb315c9a71364031f0d9b425e14c411943523674fd641d6a068a6c153641dc`,
  which `test:processors` built from the same test image and the same source tree.

No campaign recorded a missed mutant.
`Gaps round` has the gate,
the discovery campaigns,
the survivors they found and how each was killed,
and the fuzz sidecar's run against the final library.
`Final campaigns` keeps the first round,
whose three campaigns passed on image `5b241d8ac5eec41f866c91e433423507a918168bd076ed6c11dae46e1e19da44`.
`Defects found` says what the campaigns and the stack reproduction exposed,
and `Remaining` what is still open.

## Excluded mutation kinds

### Decision

The main session relayed this on 2026-10-05 as the human's decision,
while the first final round was running:
mutation timeouts are resolved by skipping two kinds of mutation in the tool,
and code changes only where that does not cover a timeout.
Every cargo-mutants invocation of this package now passes both patterns,
each as its own `--exclude-re` argument:

- `replace \+= with \*=`
- `replace -= with /=`

They are in `bin/mutate-container.mjs` for every scope,
in the inline `mutation:processors` task,
and in the `mutation:list:*` tasks,
so that a listing shows exactly what a campaign tries.
The fuzz sidecar does not invoke cargo-mutants.

The tradeoff the decision accepts:
mutants of those two kinds are never tried.
`+=` replaced by `-=`,
and `-=` replaced by `+=`,
still test every counter.

### Cause and cost

[`doc/troubleshooting/cargo-mutants-timeout-exit-status.md`](../troubleshooting/cargo-mutants-timeout-exit-status.md)
documents why cargo-mutants 27.1.0 exits 3 whenever any mutant times out,
why these two replacements stall a loop counter,
and what excluding them costs.

### Measured effect

`cargo mutants --list --no-config` at 27.1.0,
with and without the two patterns,
on the source of the gate 5 snapshot.
In every scope the mutants that disappear are exactly those whose names contain one of the two replacements,
and no other mutant appears or disappears.

- Executable scope:
  189 before,
  184 after
  (5 of `+=` to `*=`).
- Markdown scope:
  750 before,
  739 after
  (10 of `+=` to `*=`,
  1 of `-=` to `/=`).
- Processor scope:
  366 before,
  355 after
  (11 of `+=` to `*=`).
- Constant-slot scope:
  14 before,
  12 after
  (2 of `+=` to `*=`).
- Parent-lookup scope and anonymous-function scope:
  4 and 3,
  unchanged.
- Unscoped:
  1,672 before,
  1,640 after
  (31 of `+=` to `*=`,
  1 of `-=` to `/=`).

The `mutation:list:executable` and `mutation:list:markdown` tasks printed 184 and 739 mutants on that source.
The same number of `+=` to `-=` and `-=` to `+=` mutants remains in each scope as was removed:
5,
11,
11 and 2.

The same comparison on the final tree gives the same removals.
Only the Markdown and unscoped totals moved,
by the 3 mutants of the new `MarkdownSource::subtree`:
Markdown 753 before and 742 after,
unscoped 1,675 before and 1,643 after.
The executable,
processor and constant-slot scopes are unchanged at 184,
355 and 12 after exclusion,
which are the totals the final campaigns report.

### What it covers here

The decision covers four of the six recorded timeouts:
`markdown_definitions.rs:44` and `markdown_punctuation.rs:64` (both `-=` to `/=`),
and `processors_docs.rs:222` and `processors_lines.rs:45` (both `+=` to `*=`).
The two constant replacements of `MarkdownSource::parent` are not of those kinds
and are fixed in code,
under `Timeouts removed`.

All four loops had already been restructured,
committed and gated when the decision arrived,
and they stay as they are:
the decision says not to revert committed work.
On this source the exclusion is therefore not what removes those timeouts.
Before it,
every mutant of the two kinds was caught by a failing test:
5 of 5 in the executable scope (`mutation-hQ4LIa`)
and 11 of 11 in the Markdown scope (`mutation-6Cgoi0`).
On the source before the restructuring the same kinds gave 9 caught and 2 timeouts in the Markdown scope
(`mutation-BX2JYq`)
and 15 caught and 2 timeouts in the processor scope (`processor-survivors-mutation-w00nRd`).
What the exclusion buys from here on is that a hand-stepped loop added later cannot turn a campaign into exit 3;
what it costs is that those mutants are no longer tried,
including the ones a test would catch.

## Timeouts removed

Every timeout had one cause:
a loop whose only progress was a step that cargo-mutants can replace.
`+=` replaced by `*=` multiplies a zero counter by one,
and a constant replacement of `MarkdownSource::parent` makes a node its own ancestor,
so the loop never ends and the test binary runs into the 180 second per-mutant limit.
cargo-mutants 27.1.0 exits 3 for any timeout,
so no campaign containing such a loop can pass.
Each loop now walks a slice,
calls a standard search,
or runs over a fixed range,
so termination no longer depends on a statement a mutant can change.
The loop restructuring in this section was done before the decision under `Excluded mutation kinds`;
of its subsections only the bounded ancestor walk is still required by that decision.

### `markdown_definitions.rs`: definition line start

`removal_edit` walked backwards one byte at a time to find the line start
(`replace -= with /= in removal_edit` timed out in `mutation-p3QH2L` and `mutation-BX2JYq`).
It now takes `rfind(['\n', '\r'])` over the source before the definition,
the same shape as `continuation_prefix` in `markdown_prose_context.rs`.
`standalone_definitions_do_not_leave_line_fragments` already deletes an indented definition on a later line
in LF,
CRLF and bare CR,
so both mutants of the remaining `ending + 1` fail it:
the copied prefix would start at the line ending and the definition would no longer count as standalone.

### `markdown_punctuation.rs`: escaped heading punctuation

`suffix_start` counted the backslashes before a trailing period or colon with a hand-stepped index
(`replace -= with /= in suffix_start` timed out in both Markdown campaigns).
It now flips an `escaped` flag while walking the bytes before the punctuation in reverse,
the same shape as `cell_content` in `markdown_tables.rs`.
The new control `escape_runs_reaching_the_text_start_decide_the_suffix` covers runs of one,
two and three backslashes that reach the text's first byte,
two escaped characters in a row,
and a run after a letter.

### `MarkdownSource::parent`: bounded ancestor walks

Both constant replacements of `MarkdownSource::parent` timed out in every Markdown campaign,
because `has_ancestor` and `paragraph_for` climbed until `parent` returned `None`.
The main session chose,
open to veto,
to bound ancestor walks so that a cycle becomes a typed error,
rather than exclude the two mutants.

- `MarkdownSource::ancestors` is the only ancestor walk.
  It returns the ancestors nearest first,
  and runs over the fixed range `0..parents.len()`:
  a node at depth d needs d + 1 passes and d is at most one less than the node count,
  so a tree always finishes inside the range,
  and a walk that does not is a cycle.
  It then returns `MarkdownError` with the starting node's offset.
- `has_ancestor` returns `Result<bool, MarkdownError>`.
  `paragraph_for` and `delimiter_tail` in `markdown_prose_context.rs` now read one ancestor list
  that `semantic_line_breaks` takes once per text node,
  so they cannot fail separately.
- `markdown/semantic-line-breaks` and `markdown/no-emphasis-as-heading`,
  the two rules that walk ancestors,
  report the error as one `core/processing-failure` finding
  whose message starts with the rule code and `could not check this file:`,
  built by the new `ancestry_failure` in `markdown_finding.rs` through `run_failure::processing_failure`.
  That is the same pattern as `resolution_failure` in `rust_type_diagnostic.rs`.
  The public rule signatures are unchanged,
  so dispatch and the fuzz sidecar did not change.
- `markdown_tables.rs` reads one parent and walks nothing,
  so it still calls `parent` directly.

A corrupted arena cannot reach these walks:
`traversal` rejects a cyclic child graph before any parent index exists,
which `child_graph_rejects_duplicate_invalid_and_cyclic_ids` already tests.
The only place a cycle can appear is the derived parent index,
through a later change to `traversal` or a mutated accessor.
`a_parent_index_cycle_is_a_typed_error_and_a_processing_failure` in `markdown_traversal_tests.rs`
plants one there (the paragraph's parent becomes the emphasis inside it),
then requires the exact error message and offset from `ancestors` and `has_ancestor`,
and exactly one processing failure from each rule,
with its code,
severity,
message,
position and no fix.
`ancestor_walks_reach_the_root_from_the_deepest_possible_node` is the positive control for the bound:
in `*__a__*` every node lies on one path,
so the deepest text node needs exactly as many passes as the document has nodes.

### `MarkdownSource::children`: bounded descendant walks

The first final round found a seventh timeout that no earlier campaign had recorded as one:
`src/markdown_source.rs:278:9: replace MarkdownSource::children -> &[u32] with Vec::leak(vec![0])`
(`mutation-CUW7ek`).
With every node's child list replaced by the root,
`text_content` and `text_nodes` popped a node and pushed the same child again without end.

The judgement that this is a real stall and not a slow host
comes from comparing each mutant's test phase with the unmutated baseline of the same run:

- In `mutation-CUW7ek` the baseline's test phase took 0.4 seconds and this mutant's ran the full 180 seconds,
  with two tests reported as running for over 60 seconds.
- The earlier Markdown runs had recorded the same mutant as caught,
  but not by an assertion.
  Each call of the mutated accessor leaks one small list (`Vec::leak`),
  and the test binary was killed with signal 9 after 11.7 seconds in `mutation-6Cgoi0`,
  which fits the container's 2 GiB memory limit;
  the kill reason itself was not read from the container.
  Its sibling `Vec::leak(vec![1])` was killed the same way in both runs,
  after 15.4 and 9.1 seconds.
- Whether the kill or the 180 second limit comes first depends on how fast the host leaks memory.
  The main session measured a load average of 40 to 66 on 16 cores during this round.

A mutant that is caught only when memory runs out first is a timeout waiting for a slower host,
so both are treated as timeouts.
They are function-body replacements,
which the two excluded kinds do not cover,
so the fix is in code and extends the choice made for ancestor walks,
open to the same veto:

- `MarkdownSource::subtree` is the only descendant walk.
  It returns a node followed by its descendants in source order,
  and runs over the fixed range `0..=parents.len()`:
  one pass per node of the document,
  plus the pass that finds nothing waiting.
  A walk that needs more has a cycle in the child index and returns `MarkdownError` with the starting node's offset.
- `text_content` and `text_nodes` read that list and return `Result`.
- `markdown/no-duplicate-heading`,
  `markdown/no-trailing-punctuation` and `markdown/no-emphasis-as-heading`,
  the three rules that read text below a node,
  report the error as one `core/processing-failure` finding
  through the same helper as the ancestry rules,
  renamed from `ancestry_failure` to `structure_failure`.
  The public rule signatures are again unchanged.
- The other callers of `children` read one level and walk nothing.

`a_child_index_cycle_is_a_typed_error_and_a_processing_failure` plants two cycles in a built document
(the heading and the emphasis each become their own first child),
then requires the exact error from `subtree`,
`text_content` and `text_nodes`,
and exactly one processing failure from each of the three rules.
`descendant_walks_cover_the_whole_document_from_the_root` is the positive control for the bound:
the root's subtree is every node of the document,
equal to the validated traversal order,
so the walk needs every pass the range allows.

### `processors_docs.rs`: doc-comment runs

`docs` grouped adjacent line comments with an inner `while` that advanced `index`
(`replace += with *= in docs` timed out in `mutation-exJwfB` and `processor-survivors-mutation-w00nRd`).
Grouping is now a separate step:
`units` walks the fixed range `1..=comments.len()`
and closes a unit wherever the named predicate `joins_run` says the next comment does not continue it.
`joins_run` keeps every condition of the old loop:
the same prefix,
a whitespace-only gap,
exactly one line ending,
and never a block comment.
The refusal for a line comment after code still applies to a run's first comment only;
later members follow a newline-only gap,
so the check could never fire for them.

No test had two adjacent block docs or a changed line prefix on the next line,
so `processors_keep_adjacent_blocks_and_changed_prefixes_in_separate_virtual_files`
compares the exact virtual names and texts for adjacent `/** */` blocks,
`//!` followed by `///`,
and `///` followed by `/** */`,
with a two-line `///` run as the positive control.

### `processors_lines.rs`: physical lines

`physical_lines` advanced a cursor by hand
(`replace += with *= in physical_lines` timed out in the same two processor campaigns).
It is now one `enumerate` pass:
a CR directly followed by LF is skipped,
and the LF that closes a CRLF pair finds its CR with `bytes[start..index].ends_with(b"\r")`,
which needs no index guard.
A bare CR always closes its own line,
so only a CRLF pair can leave a CR at the end of the current line.

### Loops pre-empted in the executable's modules

Three loops in files the executable added had the same shape.
`cargo mutants --list --no-config` on the source at `878f79d48` generates
`replace += with *= in relative_from` at `src/run_paths.rs:119:16`,
`replace += with *= in relative_link` at `src/markdown_lfs_target.rs:217:16`,
and `replace += with *= in sha256_hex` at `src/markdown_lfs_sha256.rs:175:16`.
Each multiplies a counter or offset that starts at zero,
so the loop condition never changes.
They were restructured before any campaign reached them,
so no campaign observed them as timeouts:
the two shared-prefix counts walk zipped component pairs and stop at the first difference,
and the hash splits its input with `as_chunks::<64>()`,
which returns the whole blocks and the remainder together
and removes the `bytes.len() / 64 * 64` arithmetic.
A first version used `chunks_exact(64)`,
which container Clippy denies (`clippy::chunks_exact_to_as_chunks`).

### Gate on the restructured source

Every run used `MONOCHROMATIC_LINT_IMAGE_TAG=mutation-close`,
mount-free and network-disabled containers,
2 GiB of memory,
2 CPUs and 128 processes.
Logs are in `package/linter/monochromatic-lint/target/verification/`.

- `gate-mutation-close-1.log`:
  378 library and 10 executable tests passed,
  then Clippy failed on `chunks_exact` in `markdown_lfs_sha256.rs`.
  The library suite took 183.94 seconds in that run.
- `gate-mutation-close-2.log`,
  after the `as_chunks` change:
  378 library tests passed in 107.20 seconds,
  10 executable tests passed,
  and Clippy with `-D warnings` finished with no finding.
  Test image `f357522ebe2b80d79a5858b5768305f54ba2241629b0d7587184243bcd7633c6`,
  source tree `a3b3491e423bddea5877825ccaacfe573af7248a`.

- `gate-mutation-close-3.log`,
  after the executable controls:
  379 library tests passed,
  and 11 of 12 `binary` tests passed;
  `debug_streams_workspace_progress_and_plain_runs_stay_silent` failed on a byte comparison
  described under `Executable survivors`.
- `gate-mutation-close-4.log`,
  after that fix and the panic-hook refactor:
  380 library tests passed in 142.43 seconds,
  12 `binary` tests passed in 2.52 seconds,
  and Clippy finished with no finding.
  Test image `c184147f62f1ed1afa673cbe53b36cb7c8aeb8fd87e4f537763572551f7b03c8`,
  source tree `fa2efe52e284f9c368a81a08d2720d14579bd3a0`.
  The first Markdown campaign mutated this image.

The `binary` target took 0.30 to 0.65 seconds before the new controls and 1 to 2.5 seconds after them;
the workspace-progress control starts rust-analyzer's sysroot discovery and two `cargo metadata` runs.
Every executable-scope mutant pays that time.

The first two library-suite durations differ by 77 seconds on unchanged tests,
from load on the shared host.
The larger one is over the 180 second per-mutant limit,
so an unscoped campaign could record timeouts that are only a slow host.

`lint:rust` (the incumbent Rust linter) reports no code-line budget finding
and 84 `builtin(require-rustdoc)` findings,
one fewer than the 85 recorded before this work;
none is on an item added here.

## Executable campaign

### Scope

No campaign had mutated the executable's modules.
`mutation:executable`
(`bin/mutate-container.mjs --executable`)
mutates `src/run_*.rs`,
`src/main.rs`,
`src/rust_dispatch.rs` and `src/rust_rule_settings.rs`:
188 mutants at the first snapshot.
`mutation:list:executable` prints the scope without building.
The Markdown modules the executable added
(`markdown_lfs_*`,
`markdown_dispatch.rs` and `markdown_rule_settings.rs`)
are inside the Markdown scope's `src/markdown_*.rs` glob,
so they are dispositioned under `Markdown campaign` instead of being mutated twice.
`rust_dispatch.rs` and `rust_rule_settings.rs` predate the executable
but had never been mutated,
and they are the Rust half of the same dispatch and rule-settings layer.

### Test selection

`main.rs` and the real streams are reached only by the `binary` test target,
and its ten test names share no substring a positive filter could select.
The scope therefore runs every test
and passes libtest `--skip` for the suites that load real Cargo workspaces:
`rust_explicit_types`,
`rust_file_engine`,
`rust_inferred_constants`,
`rust_semantic_session` and `rust_workspace`.
Those suites also call `check_syntax_rules`,
which the orchestration,
processor and Rust-rule tests that stay in reach as well.
The unmutated baseline of the first campaign built in 84 seconds and tested in 1 second,
so the 180 second limit was left unchanged.
Its log shows the selection took effect:
369 library tests ran with 9 filtered out,
then the 10 `binary` tests.
The 9 are every test in `rust_explicit_types_tests.rs` (1),
`rust_file_engine_tests.rs` (2),
`rust_inferred_constants_tests.rs` (1),
`rust_semantic_session_tests.rs` (3) and `rust_workspace_tests.rs` (2).
Five of them load or prepare a Cargo workspace;
`no_semantic_selection_avoids_workspace_initialization` and the three semantic-session tests are quick,
and are skipped only because they share those modules' names.

### Executable survivors

`mutation-hQ4LIa` mutated test image
`f357522ebe2b80d79a5858b5768305f54ba2241629b0d7587184243bcd7633c6`
(`campaign-executable-1.log`):
188 mutants,
129 caught,
6 missed,
53 unviable,
0 timeouts,
exit status 2,
25 minutes.
The 53 unviable mutants are mostly replacements with `Default::default()` for types without `Default`
and `||` in `if let` chains,
which do not compile.
Each missed mutant below names its disposition;
the rerun under `Final campaigns` is the proof for each killing test.

- `src/run_check.rs:126:13`,
  `delete field explicit_types from struct RustRuleSettings expression in HostChecker<'run>::check_rust_root`.
  Equivalent, and the redundant code is removed.
  `check_syntax_rules` reads only `max_lines`,
  `rustdoc` and `no_anonymous_functions`,
  so clearing `explicit_types` in a copy of the settings changed nothing.
  `check_rust_root` now passes the settings through.
- `src/run_finish.rs:49:5: replace debug_progress with ()`.
  Not equivalent:
  `--debug` loses the workspace progress lines.
  No test loaded a workspace with `--debug`.
  The new binary test `debug_streams_workspace_progress_and_plain_runs_stay_silent`
  points `rust/require-explicit-types` at an unparsable `Cargo.toml`:
  rust-analyzer reports `discovering sysroot` before Cargo rejects the manifest,
  so the run stays quick.
  It requires that line on standard error with `--debug`,
  an empty standard error without it,
  and one processing finding with status 2 in both runs.
  The first version also compared both runs' standard output byte for byte
  and failed in `gate-mutation-close-3.log`,
  because the message quotes Cargo's command line,
  which names a fresh temporary lockfile per run;
  the runs are now compared by location and code.
- `src/run_process.rs:70:8: delete ! in parse_and_run`.
  Not equivalent,
  but visible only when a panic happens:
  the silent hook would be installed for `--debug` runs only,
  and a contained panic would print the default message on standard error,
  which carries JSONL in `--stdin --fix` mode.
  No known input makes the executable panic,
  and the process-wide hook cannot be replaced inside a test binary without affecting every other test.
  The decision is now the named function `silences_panics`,
  and `only_debug_runs_keep_the_default_panic_hook` pins both answers.
  What remains untested is that `parse_and_run` calls `set_hook` when it returns true;
  cargo-mutants generates no mutant for that call.
- `src/run_process.rs:111:22: replace || with && in run_process`.
  Not equivalent:
  a failed write to one stream would no longer change the exit status.
  The new binary test `a_failing_output_stream_exits_two` sends standard output,
  then standard error under `--debug`,
  to `/dev/full`,
  and requires status 2 instead of the findings' status 1 each time.
- `src/run_workers.rs:37:37: replace * with +`
  (`WORKER_STACK_BYTES` becomes 8 bytes more than 1 MiB).
  Not equivalent:
  input that needs more than 1 MiB of stack overflows a worker and aborts the process.
  The other three mutants of the constant leave a few kilobytes or less,
  and ordinary test input already overflowed those workers:
  their logs end in `thread '<unknown>' has overflowed its stack`.
- `src/run_workers.rs:254:16: replace <= with > in process_plans`.
  Not equivalent for the same reason:
  with two or more files the plans run on the calling thread instead of on workers,
  and the calling thread of a test has 2 MiB of stack.
  Output order and content are otherwise unchanged.

Both stack mutants are killed by the new
`workers_parse_nesting_deeper_than_a_default_thread_stack_holds`,
which lints two files nested 1,200 parentheses deep at a limit of two.
Under either mutant the test binary aborts with a stack overflow,
so the per-mutant log shows the overflow line rather than a named failing test.
The depth comes from a calibration of the debug executable in the bounded container,
varying the main thread's stack with `ulimit -s`
(scratch harness, not committed;
134 is the abort status, 0 a completed run):

- 1,000 parentheses:
  2,048 KiB aborts,
  4,096 KiB completes.
- 1,200 parentheses:
  3,072 KiB aborts,
  4,096 KiB completes.
- 1,400 parentheses:
  4,096 KiB aborts,
  5,120 KiB completes.
- 1,600 parentheses:
  5,120 KiB aborts,
  6,144 KiB completes.
- 2,000 parentheses:
  4,096 KiB aborts,
  8,192 KiB completes.
- 4,000 parentheses:
  8,192 KiB aborts.

So 1,200 levels need between 3 and 4 MiB:
twice the margin below the 8 MiB worker stack,
and more than the 2 MiB a test thread or a 1 MiB mutant stack holds.
The margin is a property of this toolchain's unoptimized parser frames;
a release build needs less stack,
so the test can only become easier to pass there.
Markdown nesting did not recurse in the same calibration:
block quotes and lists 4,000 levels deep completed at a 256 KiB stack.

## Markdown campaign

`mutation:markdown` now covers 26 production modules,
including the 10 that the executable added
(`markdown_lfs_*`,
`markdown_dispatch.rs` and `markdown_rule_settings.rs`;
20 files with their tests),
which no campaign had mutated:
751 mutants at the gate 4 snapshot.
The first run mutates test image
`c184147f62f1ed1afa673cbe53b36cb7c8aeb8fd87e4f537763572551f7b03c8`.

A first start (`mutation-AuPwZ7`, `campaign-markdown-1.log`) was stopped by removing its container
after the baseline and a few mutants,
because each mutant took 11 seconds to build for 0.1 seconds of tests.
Its baseline ran 136 library tests and filtered out all 12 `binary` tests,
so every selected test is a library test,
yet every mutant also rebuilt the executable and the `binary` target.
The scope now passes `--cargo-arg=--lib`,
which reaches both the build and the test phase and selects the same tests.
The restarted run is `mutation-6Cgoi0` (`campaign-markdown-2.log`),
against the same image.
Its unmutated baseline built in 108 seconds and ran the Markdown tests in under a second.
Result:
751 mutants,
699 caught,
8 missed,
44 unviable,
0 timeouts,
exit status 2.

The four timeouts of `mutation-BX2JYq` are gone,
and the mutants behind them are caught by ordinary failing tests:

- both constant replacements of `MarkdownSource::parent`
  (`Some(0)` and `Some(1)` at `src/markdown_source.rs:319:9`),
  which is the evidence that every ancestor walk is bounded;
- the remaining arithmetic of the restructured loops,
  `ending + 1` at `src/markdown_definitions.rs:55:29`
  and the escape step at `src/markdown_punctuation.rs:79:23`.

All 8 missed mutants are in the modules the executable added;
the 16 modules mutated before have none.

### Markdown survivors

Locations are `line:column` at the gate 4 snapshot.
The rerun under `Final campaigns` is the proof for each killing test.

- `src/markdown_lfs_config.rs:56:28: replace || with && in lfs_endpoints`
  and `src/markdown_lfs_config.rs:56:53: replace || with && in lfs_endpoints`.
  Equivalent, and the redundant code is removed.
  The mutated line skipped blank lines and `#` or `;` comments before anything else read them.
  Without it,
  a blank line has no `=` and declares nothing;
  a comment is never a section header,
  because a header must start with `[`;
  and a comment's key is the trimmed text before its first `=`,
  which starts with the comment character,
  so it can never equal `url` or `lfsurl`,
  the only keys that declare an endpoint.
  `lfs_endpoints` no longer has the skip.
  `commented_out_declarations_declare_nothing` is the control for the behavior the skip appeared to protect:
  commented-out `url` and `lfsurl` lines with either comment character,
  with and without a following space,
  and commented-out section headers,
  with an uncommented declaration after comments as the positive control.
- `src/markdown_lfs_config.rs:113:9`,
  `replace <impl std::fmt::Display for LfsConfigError>::fmt -> std::fmt::Result with Ok(Default::default())`.
  Not equivalent:
  a processing finding built from this error would lose its explanation.
  The only code that renders it is `HostChecker::lfs_context` in `run_check.rs`,
  whose tests are outside the `markdown` filter,
  so the survivor is a property of the scope's test selection, not of the whole suite.
  `optional_reads_distinguish_absence_from_failure` now asserts that the rendered text equals the stored message
  and that the message starts with `Cannot read `.
- `src/markdown_lfs_context.rs:137:72: replace != with == in find_lfs_repo_root`.
  Not equivalent:
  a search that starts below a regular file would fail with an inspection error instead of walking past it,
  and every other inspection error would be skipped silently.
  A permission-denied probe cannot separate the two in the container,
  where tests run as root,
  so `the_nearest_regular_configuration_file_marks_the_root` now starts a search at `r/sub/blocker/deeper`,
  where `blocker` is a regular file:
  the operating system answers "not a directory" for both candidates below it,
  and the search must still return `r/sub`.
- `src/markdown_lfs_sha256.rs:136:47: replace ^ with | in compress`,
  `src/markdown_lfs_sha256.rs:144:49: replace ^ with | in compress`
  and `src/markdown_lfs_sha256.rs:144:71: replace ^ with | in compress`.
  Equivalent, all three.
  Line 136 was the FIPS 180-4 `Ch` spelling `(x & y) ^ (!x & z)`:
  its halves never share a set bit,
  because one needs the `x` bit set and the other needs it clear,
  so exclusive or and inclusive or give the same value.
  Line 144 was the FIPS `Maj` spelling `(x & y) ^ (x & z) ^ (y & z)`:
  for any three input bits either none,
  exactly one,
  or all three of those terms are set,
  and both operators then agree,
  also after the changed operator precedence of the replaced text.
  Nothing here is redundant code that could be deleted,
  and the two excluded mutation kinds do not cover an operator inside an expression,
  so both functions are respelled so that no operator is interchangeable:
  `choose` is `z ^ (x & (y ^ z))` and `majority` is `y ^ ((x ^ y) & (y ^ z))`.
  A scratch truth-table check found that each of the 14 operator replacements cargo-mutants can make
  in the two new bodies changes the result for some input bits.
  `choose_and_majority_match_their_fips_definitions` compares both functions with the FIPS spellings
  on all eight combinations of three input bits (results `0xcacacaca` and `0xe8e8e8e8`),
  and the published and measured digest vectors are unchanged.
- `src/markdown_lfs_target.rs:104:31: replace || with && in apply_segments`.
  Not equivalent:
  empty and `.` segments would stay in the resolved path.
  The one existing assertion compared `PathBuf` values,
  and path equality compares components,
  which drops `.` segments and a trailing separator.
  The operating system does not drop them:
  `shot.png/` and `shot.png/.` name a directory,
  so a tracked image written with either spelling would resolve as missing,
  where the incumbent's `path.resolve` still names the file.
  `lexical_normalization_resolves_dot_components` now compares the exact path spelling for four destinations,
  and `targets_resolve_to_lfs_plain_and_missing` resolves both spellings from disk
  through a repository with an empty cache
  (a cached repository answers from its map and never reaches the file system).

## Defects found

No mutant exposed a defect on unmutated input:
no wrong exit status,
lost finding,
corrupting fix or unbounded work in the released code.
The survivors were test gaps,
one redundant struct update,
one redundant comment skip,
and three interchangeable operators in the hash.

What the campaigns did expose is unbounded work under mutation.
The ancestor and descendant walks trusted their accessors to describe a tree,
so a mutated accessor made them run until the per-mutant limit or the memory limit stopped them.
Both walks are now bounded and report a typed error;
on a document built by `MarkdownSource::new` neither error can occur,
because `traversal` validates the child graph first.

One defect on unmutated input was found by reproduction rather than by a mutant,
and is fixed under `Invocation thread`.
With `--concurrency 1`,
a single file or `--stdin`,
no worker started and the file was linted on the main thread,
whose stack the platform sets.
Under a 1,024 KiB main-thread stack on Linux,
the size of the MSVC linker's documented default reserve,
the debug executable aborted with a stack overflow on 1,200 nested parentheses in all three modes,
while two files at a limit of two completed on 8 MiB workers.
The executable now lints every invocation on a thread with the 8 MiB lint stack.
Neither Windows nor a release build was run.

The gaps round's survivors were test gaps:
inputs and manifests below a regular file,
backend settings no fixture could observe,
and a proc-macro server choice no fixture used.
None of them changed released behavior.

## Final campaigns

A final round is one gate,
which builds the test image,
followed by the three campaigns against that image with `mise run --skip-deps`,
one at a time.
Their three `manifest.json` files must name the same `baseImage`,
and a round counts only if all three exit 0.
In each round the last campaign starts only if the two before it exit 0.
Round 1 does not count and round 2 does;
`Round 2` has the final results.

A processor discovery pass on the gate 4 image (`mutation-dy0x0l`) was started and removed before its baseline,
in favour of running the processor campaign once on the gate 5 image.

### Gate 5

`gate-mutation-close-5.log`,
started at repository head `90044851e`,
linter source tree `d38179d751fa78929968a682224e2c163a5e20e0`:
382 library tests passed in 98.14 seconds,
12 `binary` tests passed,
and Clippy with `-D warnings` finished with no finding.
Test image `6245a831544ee883ab3782207c7820434e9393d32cd1badc1ee1027b31959893`.
Commits after it change the mutation runner,
the package tasks,
the README and this document,
none of which is copied into the test image.

### Executable campaign before the exclusion

`mutation-eGKI9C` (`campaign-executable-final-1.log`) started before the decision under `Excluded mutation kinds`
and ran without the two patterns,
so it tried 5 mutants the scope no longer contains.
Result:
189 mutants,
136 caught,
0 missed,
53 unviable,
0 timeouts,
exit status 0,
39 minutes.
Its per-mutant logs name the test that failed under each mutant the first run missed:

- `replace debug_progress with ()`:
  `debug_streams_workspace_progress_and_plain_runs_stay_silent`.
- `replace || with && in run_process`:
  `a_failing_output_stream_exits_two`, and no other test.
- The three mutants of the new `silences_panics`
  (constant `true`,
  constant `false`,
  and the deleted `!`):
  `run_process::tests::only_debug_runs_keep_the_default_panic_hook`.
- `src/run_workers.rs:37:37: replace * with +`:
  the test binary aborted with `thread '<unknown>' has overflowed its stack`,
  an unnamed worker thread.
- `replace <= with > in process_plans`:
  the test binary aborted with
  `has overflowed its stack` for the thread named
  `run_workers::tests::workers_parse_nesting_deeper_than_a_default_thread_stack_holds`.
- The redundant struct update in `check_rust_root` is gone,
  and its mutant is no longer generated.

This run is evidence for those kills only.
The executable campaign that counts is the one in `Round 2`,
with the committed runner and the same image as the other two.

### Round 1

Against the gate 5 image,
with the two patterns.
The round does not count:
its Markdown campaign timed out on one mutant.

- Processor campaign,
  `mutation-Ij89RQ` (`campaign-processors-files-final-1.log`):
  355 mutants,
  330 caught,
  0 missed,
  25 unviable,
  0 timeouts,
  exit status 0.
  The baseline built in 156 seconds and tested in 1.4 seconds;
  no mutant's test phase reached 20 seconds,
  and none ended by a signal.
  This is the first campaign over the restructured `units`,
  `joins_run` and `physical_lines`,
  and it left no survivor.
- Markdown campaign,
  `mutation-CUW7ek` (`campaign-markdown-final-1.log`):
  739 mutants,
  694 caught,
  0 missed,
  44 unviable,
  1 timeout,
  exit status 3.
  All 8 survivors of the first Markdown run are caught.
  The timeout is the `MarkdownSource::children` replacement
  described under `MarkdownSource::children: bounded descendant walks`.

### Round 2

After the bounded descendant walk:
a new gate,
then the executable and Markdown campaigns,
then the processor campaign if both exit 0,
all against the image that gate builds.
The processor campaign reruns because `processors_fences.rs` parses through `MarkdownSource`,
whose source changed.

Gate 6,
`gate-mutation-close-6.log`,
started at repository head `9e61f66a3`,
linter source tree `ceb495865521beda0f988a536435ffd69cc1d100`:
384 library tests passed in 162.42 seconds,
12 `binary` tests passed in 3.50 seconds,
and Clippy with `-D warnings` finished with no finding.
Test image `5b241d8ac5eec41f866c91e433423507a918168bd076ed6c11dae46e1e19da44`.
`lint:rust` on the same source reports no code-line budget finding
and 83 `builtin(require-rustdoc)` findings,
none on an item added by this work.

- Executable campaign,
  `mutation-Sh3zLV` (`campaign-executable-final-2.log`):
  184 mutants,
  131 caught,
  0 missed,
  53 unviable,
  0 timeouts,
  exit status 0,
  34 minutes.
  The baseline built in 167 seconds and tested in 16.3 seconds,
  eight times the 2.0 seconds of the previous executable run,
  at a host load average of about 55 on 16 cores.
  The longest test phase of any mutant was 41.1 seconds.
  Six mutants ended with the stack-overflow abort of the nesting control (signal 6),
  which does not depend on load;
  none ended with a memory kill.
- Markdown campaign,
  `mutation-RjKWfe` (`campaign-markdown-final-2.log`):
  742 mutants,
  698 caught,
  0 missed,
  44 unviable,
  0 timeouts,
  exit status 0,
  82 minutes.
  The baseline built in 100 seconds and tested in 0.4 seconds.
  No mutant's test phase reached 20 seconds,
  and none ended by a signal.
  All three replacements of `MarkdownSource::children` now end with named failed assertions,
  among them `markdown_basic_tests::collected_heading_text_matches_the_incumbent_helper`,
  and so do the three constant replacements of the new `MarkdownSource::subtree`.
  The per-mutant logs name the tests that fail under the mutants the first Markdown run missed:
  `markdown_lfs_config::tests::optional_reads_distinguish_absence_from_failure` for the `LfsConfigError` rendering,
  `markdown_lfs_context::tests::the_nearest_regular_configuration_file_marks_the_root`
  alone for `137:72` in `find_lfs_repo_root`,
  and both `markdown_lfs_target::tests::lexical_normalization_resolves_dot_components`
  and `markdown_lfs_context::tests::targets_resolve_to_lfs_plain_and_missing` for `apply_segments`.
  All 18 mutants of the respelled `choose` and `majority` are caught.
- Processor campaign,
  `mutation-uVvFQn` (`campaign-processors-files-final-2.log`):
  355 mutants,
  330 caught,
  0 missed,
  25 unviable,
  0 timeouts,
  exit status 0,
  57 minutes.
  The baseline built in 137 seconds and tested in 22.0 seconds,
  again a loaded host:
  the same tests took 1.4 seconds in round 1.
  No mutant's test phase reached 20 seconds,
  and none ended by a signal.
  Its caught and unviable lists are identical,
  mutant for mutant,
  to round 1's,
  which mutated the same processor source.

Round 2 counts.
The three `manifest.json` files name the same `baseImage`,
`5b241d8ac5eec41f866c91e433423507a918168bd076ed6c11dae46e1e19da44`,
the same cargo-mutants executable
(27.1.0, SHA-256 `f985f265ee3ea3e453aa98b04c52134953911f692f8ce8abf137f3873202a2d0`),
both exclusion patterns and the 180 second limit,
and each `exit.json` records status 0.
`git diff` between the gate 6 head `9e61f66a3` and the commit that records this
shows no change under `package/linter/monochromatic-lint`,
`package/rust-module/jsonc-edit` or `clippy.toml`,
the inputs the test image is built from.

### How the rounds were run

Each campaign was started with `mise run --skip-deps //package/linter/monochromatic-lint:mutation:<scope>`
and `MONOCHROMATIC_LINT_IMAGE_TAG=mutation-close`,
after one `lint:container` run with the same tag,
by a scratch driver that ran them one at a time and wrote one log per campaign.
The driver is not committed.
Without `--skip-deps` each mutation task would rebuild the test image through its `lint:container` dependency,
and the three campaigns would not share one image.

Every container was mount-free and network-disabled,
with 2 GiB of memory,
2 CPUs and 128 processes.
The session that ran this work was interrupted more than once by API limits or errors;
each time the state was re-read from the logs,
the evidence directories and `podman ps` before anything was rerun,
and no campaign had to be repeated because of an interruption.
Two campaigns were stopped on purpose by removing their containers,
`mutation-AuPwZ7` and `mutation-dy0x0l`,
for the reasons given where they are named.

## Gaps round

A delegate of the main session closed the items of `Remaining` on 2026-10-06,
with `MONOCHROMATIC_LINT_IMAGE_TAG=mutation-gaps`
and the same container bounds as the first round.
Evidence is in `package/linter/monochromatic-lint/target/verification/`.

### Scopes for every production module

Every scope is now one entry of a table in `bin/mutate-container.mjs`:
the arguments that decide which mutants exist,
the arguments that decide how each mutant is built and tested,
and a per-mutant limit where it differs from 180 seconds.
Any campaign also takes `-- --shard k/n` (zero-based, as in cargo-mutants)
and `-- --re <regex>` to rerun only some of its mutants.
The campaign,
`--list <scope>` and `--coverage` all read that table,
so a listing can no longer drift from the campaign it describes,
and the `mutation:list:*` tasks call `--list`.

- The executable scope also mutates `file_discovery.rs`,
  `path_inputs.rs` and `fix_loop.rs`,
  the input expansion and fix loop it drives:
  233 mutants instead of 184 when the table was committed,
  239 on the final tree.
- The new core scope (`mutation:core`) mutates configuration parsing,
  lookup,
  matching,
  merging and rule-option validation,
  findings,
  grouped edits,
  resolved rule settings,
  and the syntax-only Rust rules with their shared parse:
  202 mutants.
  None of these loads a Cargo workspace,
  so it uses the executable scope's test selection.
- The new semantic scope (`mutation:semantic`) mutates `rust_file_engine.rs`,
  `rust_toolchain.rs`,
  `rust_workspace.rs`,
  the `rust_explicit_*` modules,
  `rust_generic_arguments.rs`,
  `rust_semantic_*` and `rust_type_diagnostic.rs`:
  96 mutants when the table was committed,
  97 on the final tree.
  It runs the whole suite,
  because the Cargo-workspace suites are its tests
  and the rest of the suite costs little next to them.

The four processor tasks that start containers,
and the inline `mutation:processors` task,
now honour `MONOCHROMATIC_LINT_IMAGE_TAG` like the other container tasks;
they used the shared `development` tags before.

### Coverage by listing

`mutation:coverage` lists every scope and the unscoped crate
with `cargo mutants --list --no-config` and both exclusion patterns,
writes each listing to a `coverage-*` evidence directory,
and fails unless the union of the scope listings equals the unscoped listing name by name.
On the committed runner (`ed7c3080d`, linter source tree `ceb495865521beda0f988a536435ffd69cc1d100`)
it reports 1,643 unscoped mutants and a union of 1,643,
with none outside every scope and none unknown (`coverage-WgJPEx`):
rust-style 3,
Markdown 742,
parent lookup 4,
processors 355,
constant slots 12,
executable 233,
core 202 and semantic 96.
The parent-lookup scope is a subset of the Markdown scope,
and the inline `mutation:processors` task a subset of the processor scope.
The positive control is a copy of the runner without the core scope:
it fails with 202 mutants outside every scope (`coverage-99OU43`).
The comparison on the final tree is under `Final round`.

Before this round the union was 347 mutants short,
not only the 72 of the six files the first round listed as never mutated:
configuration,
findings,
edits,
resolved rules,
the syntax-only Rust rules
and the explicit-type modules had been mutated by the full-scope campaign `mutation-1vJeuS` on an older snapshot,
but no current scope held them.

### Workspace tests in the fast scopes

The executable and core scopes skip the tests that load a Cargo workspace by their full names,
instead of five module names.
`no_semantic_selection_avoids_workspace_initialization`
and the three `rust_semantic_session` tests now run in both scopes,
and so does `rust_workspace_tests::cargo_discovery_keeps_its_owner_boundary`:
it rejects a project descriptor before anything is loaded,
and took 0.16 seconds alone,
so only four of the nine tests the scope used to skip load a workspace.

`workspace-tests-cost-1.log` timed each test alone,
on one test thread,
in a bounded container of gate image `1afd09647c5615f9a60e51dd29a3f340c820c0a7e86a1059c416a1f26dc7ac89`
at a host load average of about 85 on 16 cores:

- `rust_workspace_tests::generated_definitions_and_build_failures_are_distinct`:
  139.1 seconds.
- `rust_file_engine::tests::selected_semantics_reuses_the_manifest_session`:
  86.9 seconds.
- `rust_explicit_types_tests::semantic_conformance_and_source_overlay_controls`:
  65.7 seconds.
- `rust_inferred_constants::tests::holes_resolve_against_the_parameter_in_their_own_slot`:
  43.4 seconds.
- `rust_workspace_tests::cargo_discovery_keeps_its_owner_boundary`:
  0.16 seconds.
- The rest of the library suite on two threads:
  14.5 seconds;
  the whole library suite on two threads:
  250.7 seconds.

These four run in `test:container`,
in the semantic scope,
and the constant-slot scope's two filters select two of them.

### Invocation thread

`run_process` now parses the command line,
installs the panic hook,
and runs the whole invocation on a scoped thread built by `lint_thread` in `run_workers.rs`,
the same builder and 8 MiB stack the workers use.
One file,
`--concurrency 1`,
`--stdin` and the semantic plans are therefore linted on a thread whose stack size is explicit.
If the operating system refuses that thread,
the invocation runs on the calling thread as before.

The platform figure behind the change is from Microsoft's documentation,
not recalled:
the MSVC [`/STACK` reference](https://learn.microsoft.com/en-us/cpp/build/reference/stack-stack-allocations)
says "For ARM64, x86, and x64 machines, the default stack size is 1 MB",
and [Thread Stack Size](https://learn.microsoft.com/en-us/windows/win32/procthread/thread-stack-size)
says "The default stack reservation size used by the linker is 1 MB"
and that the main thread's size comes from the executable header.
A GitHub code search of `rust-lang/rust` for `STACK` in `linker.rs` found only `-z noexecstack`,
which suggests that rustc passes no `/STACK` reserve;
that is a search result,
not a Windows build,
and no Windows or release build was run.

The reproduction lints `fn main` with 1,200 nested parentheses,
the input of the worker nesting control,
with the debug executable in the bounded container,
lowering the main thread's stack with `ulimit -s` in `sh -c` before `exec`.
`stack-repro-before.log` is the unchanged tree,
image `5b241d8ac5eec41f866c91e433423507a918168bd076ed6c11dae46e1e19da44`
(the container's default soft limit is 16,384 KiB):

- One file at the default limit:
  exit 0.
- One file at 1,024 KiB:
  exit 134,
  `thread 'main' (4230) has overflowed its stack`.
- Two files with `--concurrency 1` at 1,024 KiB:
  exit 134,
  the same overflow on `main`.
- Two files with `--concurrency 2` at 1,024 KiB:
  exit 0,
  because both files run on 8 MiB workers.
- `--stdin` at 1,024 KiB:
  exit 134,
  the same overflow on `main`.

`stack-repro-after.log` is the same script on the final gate image
`7526e26a714499174ff012c45175059ac39127333510cc8fa405264c902499f7`
(linter source tree `c5f934d97d7579d9eca66741241ecc47291cc877`):
all five runs exit 0 with empty output,
including one file,
`--concurrency 1` and `--stdin` at 1,024 KiB.

`a_small_main_thread_stack_does_not_limit_nesting` in `src/binary_tests.rs` runs the one-file,
`--concurrency 1` and `--stdin` cases under a 1,024 KiB main stack
and requires exit 0 with both streams empty.
With the invocation thread replaced by a direct call on the calling thread,
that test failed on the host:
`the executable was killed by a signal: thread 'main' (532099) has overflowed its stack`
(`hand-mutations-stack-hook.log`).

### Panic hook and stack size controls

The hook installation moved from `parse_and_run` into `run_process_with`,
which takes the parsed options and the runner,
so a test can run one real invocation whose runner panics.
`only_debug_invocations_print_the_default_panic_message` starts its own test binary again
with only the ignored `panic_hook_child` selected,
once plain and once with `--debug`.
The plain child must write exactly the program's internal-error line on standard error and exit 2;
the debug child must write the default hook's message,
naming the `monochromatic-lint` thread and the panic location,
before that line.
Because the hook is process-wide,
the child test is ignored unless selected,
and does nothing unless its environment variable names a mode.

`lint_threads_hold_the_documented_eight_mebibyte_stack` starts a thread from `lint_thread`,
recurses in frames of at least 4 KiB until the frames span 7.5 MiB measured from frame addresses,
and then requires `WORKER_STACK_BYTES` to equal 8 MiB.
The nesting controls alone pass with a 4 MiB stack;
the probe does not.

Hand mutations on the host,
each written back and checked with `git diff --quiet` (`hand-mutations-stack-hook.log`):

- The `set_hook` call deleted:
  the hook control failed,
  because the plain child's standard error began with `thread 'monochromatic-lint' (529092) panicked at`.
- A 7 MiB and a 4 MiB stack:
  the probe thread overflowed and the test binary aborted with signal 6.
- A 4 MiB stack with only the two nesting controls selected:
  both passed,
  which is the gap the probe closes.
- A 16 MiB stack:
  the equality failed (`left: 16777216`, `right: 8388608`).
  A larger stack is not a defect,
  so only the equality distinguishes it.

### Discovery campaigns

Gate `gate-mutation-gaps-1.log`,
at repository head `f430ab536`,
linter source tree `b4f8c609dc6dbc35bdcf99481b1e9cf89529e05d`:
386 library tests passed and one was ignored (the panic-hook child) in 244.71 seconds,
13 `binary` tests passed in 96.00 seconds,
and Clippy with `-D warnings` finished with no finding.
Test image `1afd09647c5615f9a60e51dd29a3f340c820c0a7e86a1059c416a1f26dc7ac89`.
The host load average reached 176 on 16 cores while it ran,
which is why the binary tests took 96 seconds instead of 3.5.
Every discovery campaign mutated this image.

- Executable scope,
  `mutation-t7RNjW` (`campaign-executable-gaps-1.log`):
  239 mutants,
  177 caught,
  2 missed,
  60 unviable,
  0 timeouts,
  exit status 2.
  The baseline built in 145 seconds and tested in 7 seconds;
  the longest test phase of any mutant was 94.0 seconds.
  The scope grew from 233 to 239 mutants because the invocation thread added functions to `run_process.rs`.
- Semantic scope,
  three shards of a 1,200 second limit:
  `mutation-AoaiDq` (shard 0/3,
  32 mutants,
  24 caught,
  8 unviable,
  exit status 0,
  2 hours),
  `mutation-cV2Ad8` (shard 2/3,
  32 mutants,
  16 caught,
  6 missed,
  10 unviable,
  exit status 2,
  67 minutes),
  and `mutation-6G2hzn` (shard 1/3,
  32 mutants,
  28 caught,
  4 unviable,
  exit status 0,
  2 hours).
  The shard baselines tested in 224,
  218 and 188 seconds,
  and no mutant timed out.
- Core scope,
  `mutation-YJ48Vl` (`campaign-core-gaps-1.log`):
  202 mutants,
  182 caught,
  0 missed,
  20 unviable,
  0 timeouts,
  exit status 0,
  2 hours.
  The baseline built in 311 seconds and tested in 42 seconds.
  It still skipped five tests by name,
  because it started before the skip list was reduced to four.

Two campaigns were stopped on purpose by removing their containers.
`mutation-VFaIKp` was the unsharded semantic campaign:
its baseline tested in 251 seconds,
which made 96 mutants a run of about seven hours and set the 1,200 second limit,
and it was stopped after the baseline in favour of three shards.
`mutation-7j4HXK` was the first shard 1/3:
its container ran shard 0/3's command,
because two `podman build` runs started together,
whose Containerfiles differed only in `CMD`,
committed the same image with shard 0's `CMD`,
although the second build printed its own.
The runner now passes each campaign's command to `podman create`
and refuses a container whose configured command differs from it.

### Survivors of the discovery campaigns

Locations are `line:column` on tree `b4f8c609dc6dbc35bdcf99481b1e9cf89529e05d`.
Every disposition below is a kill;
host hand mutations (`hand-mutations-survivors-1.log` and `hand-mutations-survivors-2.log`)
and the bounded-container runs in `workspace-controls-container-2.log` show each new control failing,
and the final round is the proof by cargo-mutants.

- `src/path_inputs.rs:78:68: replace == with != in expand_glob`
  and `src/path_inputs.rs:137:72: replace != with == in collect_inputs`.
  Not equivalent:
  a literal or glob input below a regular file would be refused as unreadable,
  and other inspection errors would be skipped silently.
  `inputs_below_a_regular_file_are_unmatched_not_unreadable` requires `a.md/x.md` and `a.md/sub/*.md`,
  where `a.md` is a regular file,
  to be unmatched inputs:
  empty with `--no-error-on-unmatched-pattern`,
  and the exact unmatched-input message without it.
- `src/rust_workspace.rs:59:72: replace != with == in discover_manifest`.
  Not equivalent,
  the same "not a directory" answer for a manifest below a regular file.
  `manifest_discovery_walks_past_a_regular_file_on_the_path` starts at `blocker/deeper/lib.rs`,
  where `blocker` is a regular file,
  and requires the manifest above it.
- `src/rust_workspace.rs:90:9`,
  `91:9`,
  `92:9` and `93:9`:
  `delete field` `sysroot_src`,
  `metadata_extra_args`,
  `extra_args` and `target_dir_config` from the backend's `CargoConfig`.
  Not equivalent,
  but invisible in the container.
  In `ra_ap_project_model` 0.0.336,
  a sysroot without `sysroot_src` goes through `discover_rust_lib_src_dir` (`sysroot.rs`),
  which reads `RUST_SRC_PATH` before the toolchain's own source;
  `metadata_extra_args` is appended to every `cargo metadata` (`cargo_workspace.rs`),
  and without `--locked` a stale or missing lockfile is resolved again into a temporary copy;
  `extra_args` reaches the build-script `cargo check` (`build_dependencies.rs`).
  The image sets no `RUST_SRC_PATH`,
  sets `CARGO_NET_OFFLINE=true`,
  and every Cargo fixture runs `cargo generate-lockfile --offline` first,
  so no workspace test can see a difference.
  The settings moved into the named `cargo_config`,
  and `cargo_settings_name_the_checked_library_and_keep_cargo_offline_and_locked`
  compares the whole `CargoConfig` it builds with the expected one;
  each hand-deleted field failed it.
- `src/rust_workspace.rs:130:62: replace == with != in load_cargo_workspace`,
  the proc-macro server choice.
  Not equivalent:
  the generated preparation would load no proc-macro server,
  and source-only preparation would start one.
  No fixture had a procedural macro.
  `generated_definitions_and_build_failures_are_distinct` now has a workspace member with a function-like
  procedural macro that defines `macro_generated`,
  called from the checked file.
  Under the mutant the generated preparation reported 2 findings instead of 1 in the container,
  because the unexpanded macro left the call unresolved.

### Final round

Gate `gate-mutation-gaps-2.log`,
at repository head `066b14ec5`,
linter source tree `c5f934d97d7579d9eca66741241ecc47291cc877`
with no uncommitted change under the package:
389 library tests passed and one was ignored in 173.31 seconds,
13 `binary` tests passed in 1.04 seconds,
and Clippy with `-D warnings` finished with no finding.
Test image `7526e26a714499174ff012c45175059ac39127333510cc8fa405264c902499f7`.
Every campaign of the round,
with its manifest's `baseImage`,
mutated this image;
the step logs' closing `test image` line shows the tag at the moment a step ended
and is not the campaign's image.
Commits after the gate change only the runner and this document;
`src` is the same tree at the last commit,
and no package file was edited while the round ran.

`test:processors` (`gate-processors-gaps-final.log`) built processor image
`40bb315c9a71364031f0d9b425e14c411943523674fd641d6a068a6c153641dc` from the test image
and the working tree's `src` and `fixtures`,
at the same head and source tree:
389 library and 13 `binary` tests passed and the consumer program ran.
The inline `mutation:processors` task mutated that image.

The campaigns ran in six chains at once,
three of them the semantic shards,
and later the two core shards and the fuzz smoke run beside them,
each in its own 2-CPU container,
at host load averages of about 45 to 95 on 16 cores.
Per-mutant test phases followed that load:

- Executable scope:
  baseline 43.8 seconds,
  six caught mutants between 120 and 146.1 seconds,
  and the one timeout at 180 seconds.
  The timed-out mutant's log ends with
  `run_workers::tests::every_limit_processes_each_plan_once_in_plan_order` and
  `run_write::tests::concurrent_replacements_do_not_collide` running for over 60 seconds,
  after `relative_absolute_and_literal_inputs_share_one_native_result_set` had already failed;
  the same mutant failed its tests in 2.0 seconds in discovery and in 1.1 seconds in the rerun.
- Core scope:
  baselines 139.8 and 17.9 seconds,
  longest caught mutants 136.2 and 128.8 seconds.
- Semantic scope:
  baselines 242.6,
  258.3 and 365.6 seconds,
  longest mutants 315.8,
  298.4 and 357.8 seconds.
  The 1,200 second limit was 4.8 times the 251 second discovery baseline it was set from,
  and 4.9,
  4.6 and 3.3 times these three;
  cargo-mutants' own automatic limit would be five times the baseline.
- Markdown and processor scopes:
  longest mutants 67.7 and 24.8 seconds.

The Markdown and processor scopes' mutated files,
their listings and the fixtures are unchanged since the first round;
both were rerun so that every scope has a result on this image,
and their caught and unviable counts equal the first round's.

Three things were stopped or lost on purpose or by accident,
none of them a result:

- The first launch of this round,
  at 16:00 UTC,
  died before any campaign container existed:
  all six drivers ended at once with no exit line,
  for a reason not found in the journal.
  They left the empty evidence directories `mutation-OV61jL`,
  `mutation-BV6Pgt`,
  `mutation-JVeXz3`,
  `mutation-pRdJIF` and `mutation-VPYDyQ`.
  The relaunch started the six chains 20 seconds apart from one orchestrator.
- The core scope ran as two shards started beside the executable campaign,
  so the unsharded core step of the executable chain,
  `mutation-vBdCI7`,
  had its container removed as it started;
  its `exit.json` records the removal.
- `mutation-m8iNlm` is the rerun of `src/path_inputs.rs` in the executable scope,
  with `-- --re '^src/path_inputs\.rs:'`,
  the scope's own files and test selection,
  and the 180 second limit;
  its baseline tested in 2.6 seconds.

`mutation:coverage` on the final tree (`coverage-potTv8`) again reports 1,650 unscoped mutants,
a union of 1,650,
none outside every scope and none unknown,
and each scope's listing equals the mutant names in its campaigns' outcomes:
executable 239,
core 202,
semantic 97,
Markdown 742,
processors 355,
constant slots 12,
parent lookup 4 and rust-style 3.
The 7 mutants more than on tree `ceb495865521beda0f988a536435ffd69cc1d100` are the new functions of `run_process.rs`
and `run_workers.rs` (6) and the named `cargo_config` (1).

### Fuzz sidecar against the final library

The sidecar had no planted-defect controls.
`test:planted`,
new in `bin/planted-controls.mjs` of the sidecar and shaped like the one in `package/git-policy/cli.fuzz`,
copies the linter,
the sidecar and their JSONC dependencies to the sidecar's ignored `target/planted` directory,
plants one defect per fuzz target,
and requires a generator control to fail for each;
a plant that does not build counts as not noticed.
The sidecar's container task now tags its images with `MONOCHROMATIC_LINT_IMAGE_TAG`.

Every run below used linter source tree `c5f934d97d7579d9eca66741241ecc47291cc877`,
the tree of the final gate.
Logs are in `package/linter/monochromatic-lint.fuzz/target/verification/`.

- `test` (`fuzz-test-gaps.log`):
  9 generator controls passed on the host toolchain.
- `test:planted` (`fuzz-planted-gaps.log`, evidence `planted-jmjt15`):
  the 9 unplanted controls passed,
  and each of the six plants failed at least one named control:
  a rejected `warn` severity failed `generated_cases_reach_valid_settings` and four orchestration controls;
  arrays replaced instead of concatenated failed `merge_invariant_controls_reach_mixed_and_array_shapes`;
  only the first closure reported failed the Rust-style generator control;
  an initialized binding exempt from annotations failed `explicit_type_generator_reaches_all_controls`;
  a trailing colon no longer heading punctuation failed the Markdown generator control;
  and an unlabeled rustdoc fence no longer a doc test failed `counted_cases_reach_every_processor_layer`
  and `raw_hosts_are_processed_in_every_language`.
  An earlier run on tree `b4f8c609dc6dbc35bdcf99481b1e9cf89529e05d` (`planted-XsM2Y0`) gave the same result.
- `smoke` (`fuzz-smoke-gaps.log`, evidence `campaign-ljLFuJ`),
  which runs the unit controls and Clippy in the build container and then the AddressSanitizer build:
  9 controls passed,
  Clippy finished with no finding,
  the build took 44 minutes 51 seconds,
  and every target exited 0 after 30 seconds with an empty artifact directory.
  Executions:
  `merge_values` 10,679,
  `configuration` 5,519,
  `rust_style` 149,
  `rust_explicit_types` 81,
  `markdown` 3,773
  and `orchestration` 256.
  Build image `c36669b86dc4dfb7195e60f3cc12527f6026288f8ed03cb974cce0867e6794af`,
  run image `33a61e94c5ed1cab6128854faf359b643fb759e3e7de2d3181c846addb21037f`.
  Every count is lower than in the 2026-10-05 smoke run,
  between 0.10 times (`markdown`) and 0.64 times (`configuration`) its count there,
  at a host load average of 70 to 80 with the mutation campaigns running beside it;
  30 seconds per target is a smoke run,
  not a campaign.

The separate `build` task was not run;
`smoke` performs the same AddressSanitizer build first.

## Remaining

### Unscoped campaign

The unscoped `mutation` task runs the whole suite for every mutant under the 180 second limit.
The whole library suite took 250.7 seconds alone in `workspace-tests-cost-1.log`,
so that task would record timeouts on this host;
it was not run.
Every mutant it would try is in a scope,
by the listing comparison under `Coverage by listing`.

### Margin of the fast scopes on a loaded host

The executable and core scopes start child processes and threads,
and their test phases follow host load.
In the final round the executable scope's baseline tested in 43.8 seconds,
six times its 7 seconds in discovery,
six caught mutants took 120 to 146.1 seconds,
and one reached the 180 second limit while two unrelated concurrency tests were still running;
one core shard's baseline took 139.8 seconds and its longest mutant 136.2.
So the 180 second limit holds about 1.2 times the slowest caught mutant on a host loaded like that one,
and a busier host would record timeouts that are only load;
`Final round` has the rerun that disposed of the one it did record.
An inference from the source,
not measured:
under the five `has_glob` mutants that make it true for ordinary text,
`walk_root` stops at the first component,
so the absolute glob in `relative_absolute_and_literal_inputs_share_one_native_result_set` walks from `/`
until the walker meets its first error,
and how long that takes depends on the container's file tree;
that is why `src/path_inputs.rs` is the file most likely to time out again on a slow host,
although the measured cause of this timeout was load.
A timeout in these scopes should be read against the baseline of its own run
and the tests named as still running in the mutant's log
before it is treated as a stall,
and the affected file rerun once with `-- --re '<regex>'`.

### Settings pinned but not observed

The `RUST_SRC_PATH` override and the re-resolution of a stale lockfile,
which `cargo_settings_name_the_checked_library_and_keep_cargo_offline_and_locked` pins through the backend settings,
are not exercised by any fixture.
A behavioral control would load a workspace with `RUST_SRC_PATH` set to another library,
and another workspace whose lockfile is stale,
each costing a workspace load (40 to 140 seconds here) in every semantic mutant's test phase.

### Platforms and builds not run

No Windows build and no release build was run.
The invocation thread uses the same `std::thread::Builder::stack_size` as the workers on every platform;
that it removes the dependence on a 1 MB Windows main thread is an inference from Microsoft's documentation
and the Linux reproduction,
not a Windows measurement.

### Container-build collision

Two `podman build` runs started together for Containerfiles that differed only in `CMD`
committed the same image with the first build's `CMD`,
although the second printed its own (`mutation-7j4HXK`).
The runner no longer depends on an image's `CMD`.
The collision was seen once and not reduced to a minimal reproduction;
a section of `doc/troubleshooting/` for it is proposed,
not written,
because this delegation could change only the linter packages and this document.

### Existing rustdoc findings

`lint:rust` reports 83 `builtin(require-rustdoc)` findings on the final tree and no code-line budget finding,
with the same count in every file as before the gaps round.
85 were recorded before the first round;
the two that went were on `use` lines that round documented.
None is on an item added by either round.

## Commits

On `main`,
in order.
Commits that change only this document are left out;
`git log -- doc/handover/unified-linter-mutation-close.md` lists them.

### Commits for loops and walks

- `b746a2144` finds definition line starts and escaped heading punctuation without hand-stepped loops.
- `42dbf3f58` groups doc comments and splits physical lines over fixed ranges.
- `878f79d48` bounds Markdown ancestor walks by the node count and reports a parent cycle.
- `236f0db1c` counts shared path components and hashes whole blocks without stepped loops.
- `63324540e` applies rustfmt to the parent-cycle control.
- `6f0f3f656` splits hash input with `as_chunks`.
- `9e61f66a3` bounds Markdown descendant walks by the node count and reports a child cycle.

### Commits for the runner, tasks and README

- `cb2d096ae` adds the executable mutation scope and its tasks.
- `34c9b924b` corrects the parent-lookup scope's description.
- `b311f969d` builds only the library tests in the Markdown scope.
- `329e1c473` names the executable and processor mutation tasks in the README.
- `af8431372` passes the two exclusion patterns in every cargo-mutants invocation.
- `46b75dcf7` records the two excluded kinds in the README.

### Commits for executable survivors

- `d10137648` passes Rust settings to syntax dispatch without the redundant copy.
- `a3fccbeba` adds the worker stack and workspace-progress controls.
- `f0403ae7f` adds the failing-stream control.
- `094a375cf` names the panic-hook decision and tests it.
- `2af75268a` compares the two workspace-failure runs by location and code.

### Commits for Markdown survivors

- `33d4b3185` removes the redundant comment skip and adds the LFS configuration,
  error-text and search controls.
- `e7eee1d37` respells the hash's `choose` and `majority` functions.
- `08b561920` compares applied destination segments by exact spelling.

### Commits for the gaps round

- `ed7c3080d` gives every production module a scope in one table,
  adds `--list` and `--coverage`,
  skips the workspace tests by full name,
  and tags the processor tasks' images with `MONOCHROMATIC_LINT_IMAGE_TAG`.
- `f430ab536` lints every invocation on a thread with the 8 MiB lint stack
  and adds the small-main-stack,
  panic-hook and stack-size controls.
- `522fc2a49` shards campaigns,
  starts each from its own image ID,
  and gives the semantic scope its 1,200 second limit.
- `416d00ee1` passes each campaign's command to `podman create` and checks it.
- `a1d285671` adds the sidecar's planted-defect controls and per-session image tags.
- `1944613ed` adds the controls for the discovery survivors,
  names the backend settings `cargo_config`,
  adds the proc-macro member to the generated-definitions fixture,
  and skips only the four workspace-loading tests.
- `5bdb09aaa` states the lint stack in the README.
- `fe00f4a14` lets a campaign rerun only the mutants whose names match `--re`.

### Evidence location

Evidence directories and logs are in the ignored build tree,
`package/linter/monochromatic-lint/target/verification/`:
`mutation-hQ4LIa`,
`mutation-6Cgoi0`,
`mutation-eGKI9C`,
`mutation-Ij89RQ`,
`mutation-CUW7ek`,
`mutation-Sh3zLV`,
`mutation-RjKWfe` and `mutation-uVvFQn`,
with `gate-mutation-close-1.log` to `gate-mutation-close-6.log`
and the `campaign-*.log` files named in the sections that use them.
The gaps round's evidence is in the same directory:
`gate-mutation-gaps-1.log` and `gate-mutation-gaps-2.log`,
`gate-processors-gaps-final.log`,
the `campaign-*-gaps-*.log` files,
the `mutation-*`,
`processors-mutation-*` and `coverage-*` directories named in `Gaps round`,
`stack-repro-before.log`,
`stack-repro-after.log`,
`hand-mutations-stack-hook.log`,
`hand-mutations-survivors-1.log`,
`hand-mutations-survivors-2.log`,
`workspace-tests-cost-1.log`
and `workspace-controls-container-2.log`.
The fuzz sidecar's are in `package/linter/monochromatic-lint.fuzz/target/verification/`.
The scratch drivers that ran the steps are not committed.
The gate and campaign logs record their repository head,
linter source tree and command;
the reproduction and timing logs name the image they ran,
and the hand-mutation logs the exact text each mutation replaced.
