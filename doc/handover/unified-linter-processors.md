# Unified-linter processor handoff

## Purpose and scope

Implement the processor contract in `doc/planning/unified-linter.md`.
The executable integration belongs to the parent agent.
Inspect the interface,
verification evidence,
and commit blocker before integrating it.
Respond with integration results or a concrete contract discrepancy,
not a generic continuation request.

Changes are restricted to new `src/processors*.rs` modules,
`fixtures/processors.json`,
processor module registration in `src/lib.rs`,
owning package tasks,
and this handoff.
No existing rule,
configuration module,
Cargo manifest or lockfile,
scanner,
cli-git source,
fuzz sidecar,
root configuration,
or production executable was changed by this delegation.

## Interface

The deep processor module lives at the orchestration seam:
`package/linter/monochromatic-lint/src/processors.rs`.
It owns native extraction,
doctest preparation,
composed original-host positions,
and safe atomic projection.
Callers do not reconstruct comment prefixes or synthetic positions.

- `extract(filename, source, language)` returns immutable `VirtualSource` inputs.
  `ProcessorLanguage` distinguishes Rust,
  Markdown,
  and MDX.
  Root rule execution remains the caller's responsibility.
- `VirtualSource::filename()` supplies the logical path for existing configuration matching.
  Processors remain enabled even when no rule matches the physical host.
  Match each virtual path separately.
- `source()` supplies prepared Rust or extracted Markdown.
  `language()` chooses the parser.
  `is_rustdoc()` chooses existing Rustdoc-specific Markdown rule behavior.
- `check_rust(settings)` runs the current native syntax rules and returns **already mapped host findings**.
  Do not map those findings again.
  Do not feed prepared Rust into an ordinary file checker that counts or reports synthetic main text.
- For Markdown,
  parse `source()` with `MarkdownSource`,
  run selected rules,
  then call `project_diagnostic(finding)`.
  It returns an authored host finding,
  absence for a purely synthetic ordinary finding,
  or `ProcessorError` for an unsupported operation.
- `project_fix(fix)` maps an entire group to the original host.
  Apply it with the existing `edits::apply_fixes` operation.
  A failed group never returns partial edits.

Re-extract after every changed host snapshot in `fix_loop::SourceChecker`.
Each returned map owns the exact original snapshot for that pass.
No filesystem read or write occurs inside the processor interface.

## Extraction and preparation

- Native Markdown/MDX code nodes discover Rust/rs-prefixed fence info strings,
  including comma-delimited and space-delimited Rustdoc attributes.
  TypeScript and ordinary unlabeled Markdown fences remain unprocessed.
- Rust fence ordinals start at zero for selected Rust fences within each Markdown input.
  Names are `<host>/<ordinal>.rs`.
- Authored Rust comment tokens produce Markdown inputs from adjacent matching `///` or `//!` runs,
  or individual `/** */` and `/*! */` blocks.
  Native classification excludes `////`,
  `/***`,
  `/**/`,
  string lookalikes,
  and `#[doc = ...]` attributes.
- Rustdoc virtual names use the first authored physical line,
  starting at one:
  `<host>/<first line>.md`.
  Common authored margins and consistent block-comment star decoration are stripped into mapped prefixes.
- Unlabeled fences are Rust only inside authored Rustdoc.
  Each embedded Rust fence consumes a nesting level.
  Extraction intentionally stops before a third Rust embedding.
  Authored Markdown comments at the final Rust level still participate in prose checks and mapped fixes.
  Synthetic lines do not affect authored line suffixes in virtual names.
- Preparation strips `# ` and standalone `#` hidden markers,
  handles escaped `##`,
  retains leading `//!` comments outside the wrapper,
  and wraps fragments without a real native top-level `fn main`.
  Both ordinary and raw `r#main` identifier spellings are recognized.
  Empty and doc-only fragments also receive unmapped scaffolding.
- Native syntax recovery remains lintable,
  including `compile_fail` fences.
  Syntax errors are not converted into rule findings.

Discovery is deterministic:
inputs from the current native parse are emitted before their descendant discovery.
Do not infer a global source-offset sort from the returned catalog.

## Mapping and fix safety

Immutable direct-parent layers retain copied byte ranges,
removed prefixes,
physical newline spelling,
and authored container anchors.
Diagnostics intersect authored ranges through each layer,
then resolve the original host's filename and column convention.
Invalid UTF-8 boundaries,
overflowing spans,
and missing source labels are explicit failures.
A processing failure in synthetic text is anchored to the host,
never silently dropped.

Projection validates an entire group,
restores each affected line's newline and continuation prefix,
then reparses the changed native container.
Its extracted bytes must equal the intended virtual rewrite at every layer.
Fence/block closing injections,
hidden-line semantic changes,
CRLF splits,
synthetic-gap edits,
and internal overlap are refused.

Whole-line removal consumes that line's own comment prefix.
Partial line joining consumes the following prefix when required.
Trailing newline replacements do not create an orphan prefix at virtual EOF.
Untouched original bytes stay outside the returned edits.
Conflicting groups are still selected atomically by `edits::apply_fixes`.
An empty virtual document is allowed;
the shared host engine still refuses nonempty real files becoming empty.

## Explicit limitations

- `rust/require-explicit-types` on virtual Rust returns an error-severity
  `core/processing-failure` with `processing_failure = true`.
  This interface supplies no registered Cargo semantic context for virtual Rust.
  There is no method-name heuristic or guessed type resolution.
  Configure the rule off for those virtual paths until executable integration supplies adequate context.
- Empty fences have authored reporting anchors but no proven insertion mapping.
  Insertion fixes into them are explicitly refused.
- A line-doc token following Rust code on the same physical line has no reusable safe prefix.
  Extraction explicitly refuses that layout.
  Inline block docs are supported.
- Native fence normalization that cannot be paired with an exact authored suffix,
  such as partial tab expansion,
  is an explicit extraction failure.
- Projection can conservatively refuse edits that alter the preparation wrapper,
  common margin,
  or extraction identity.
  It does not guess how to recover those mappings.
- The fuzz sidecar remains untouched.
  Extending global fuzzing through this interface is parent-agent work.

## Verification

All processor tasks are package-scoped.
The owned verification image tag is `localhost/monochromatic-lint-processors:development`.
Tests,
artifact consumers,
and mutations are mount-free,
network-disabled,
limited to 2 GiB memory,
2 CPUs,
and 128 PIDs.
Clippy uses the same source snapshot with the existing read-only compiler mount.

Commands exercised:

- `mise run //package/linter/monochromatic-lint:format:processors`.
- `mise run //package/linter/monochromatic-lint:processors:tools`.
- `mise run //package/linter/monochromatic-lint:mutation:list:processors`.
- `mise run //package/linter/monochromatic-lint:lint:processors`.
- `mise run //package/linter/monochromatic-lint:evidence:processors`.
- `mise run //package/linter/monochromatic-lint:mutation:processors`.

The expanded snapshot
`317e12c53bbb9af8c9b14248d65f263f52fc7166938b78cc1c0f542604f6b602`
passed 200 native tests,
the standalone built-library consumer,
and Clippy with warnings denied.
Snapshot
`420856ec9b11550b1e04404cccb12cbb8e11222aaefcf4ed88b9eef87020dc40`
subsequently passed 202 native tests,
the same consumer,
and Clippy.
The final source gate passed 203 native tests,
including 33 processor tests,
the standalone built-library consumer,
and Clippy with warnings denied.
Its exact image is
`62a207e21b353bf4b0b047e67d9424a2720eda1c1d7286b006150ddd9f96188a`.
This gate includes rebased candidate-failure anchors,
empty authored anchors,
Markdown prose at the final Rust depth,
and raw entry-point names.

The last completed source-hash/render gate predates those final corrections.
It compared the processor source,
fixture,
module registration,
Cargo manifest,
and Cargo lockfile with image
`de01eb1499a4f233dc45d032fc1419e1df70af2760e297f065abacbc3d991eba`
and found matching hashes.
Its combined source digest is
`c77a98b90ac4318ea653aec9c0e55ec2f2513e5ef52550e7f66612574db5e25a`.
Per-file historical hashes are in
`package/linter/monochromatic-lint/target/verification/processors-manifest.json`.
The historical rendered handoff is in
`package/linter/monochromatic-lint/target/verification/processors-handover.html`.
These artifacts must not be represented as the final source or final document.

The first runnable scoped campaign caught four mutants but failed its reach assertion:
the regex incorrectly matched `Ok()` instead of the guard replacement `Ok(())`.
Its report is
`package/linter/monochromatic-lint/target/verification/processors-mutation-bklcVi/mutants.out/outcomes.json`.
The regex was corrected and five-control campaigns passed against both the historical and final source images.
The final campaign ran against
`62a207e21b353bf4b0b047e67d9424a2720eda1c1d7286b006150ddd9f96188a`
and caught all five controls,
with zero missed,
unviable,
or timed-out mutants.
Its report is
`package/linter/monochromatic-lint/target/verification/processors-mutation-VrbPEr/mutants.out/outcomes.json`.
It targets five concrete controls:
authored-main detection replaced by constant true/false,
prefix encoding replaced by empty output,
grouped rewrite replaced by empty output,
and container verification replaced by unconditional success.
This is guard-focused mutation evidence,
not a full mutation campaign over all new processor source.
The corrected campaign caught all five selected mutants,
with zero missed,
unviable,
or timed-out controls.
Its report is
`package/linter/monochromatic-lint/target/verification/processors-mutation-fJtiKm/mutants.out/outcomes.json`.
The cargo-mutants executable SHA-256 is
`f985f265ee3ea3e453aa98b04c52134953911f692f8ce8abf137f3873202a2d0`.
An additional artifact-consumer control then exposed a projection-failure metadata defect:
a rejected proposed MDX rewrite reported byte 85 in an original host of 45 bytes.
The original immutable library image reproduced the failure through
`mise run //package/linter/monochromatic-lint:test:processors:anchor-control`.
Container re-extraction failures are now rebased to the original authored container anchor,
while retaining the native failure explanation.
The correction passed the complete native suite,
built-library consumer,
and Clippy.
The original artifact-control failure has therefore been demonstrated to fail before the correction
and pass after rebuilding it.
The final complete gate also passed the empty-authored-anchor control.
Mutation controls passed against the final immutable image.
A zero-width diagnostic in an empty authored inner block doc is also anchored to the original comment,
not classified as synthetic.
That case is covered by the native processor suite.
The depth cap is applied only when extracting another Rust fence,
so authored Markdown prose at the second Rust level is still extracted and safely projected.
Controls verify both that prose path and absence of a third Rust input.

Coverage includes native extraction,
MDX traversal,
Rustdoc attributes and lookalikes,
common margins,
Unicode host columns,
CR/LF/CRLF,
empty/doc-only fragments,
authored and synthetic mains,
closure and documentation checks,
empty-snippet file docs,
hidden lines,
synthetic-only/mixed labels,
grouped diagnostic fixes,
byte overflow,
UTF-8/CRLF boundary refusal,
malformed delimiters,
first/middle/last line deletion,
inline blocks,
nested projection through the second level,
and deliberate rejection of deeper extraction.

The suite includes 84 systematic delimiter/container/newline combinations
and 1,024 seeded fuzz cases with seed `0x91a2_b3c4`.
Fuzz cases exercise real native extraction,
authored host positions,
safe literal fixes,
and accepted/refused adversarial atomic groups.
These are bounded native property controls,
not a claim of exhaustive arbitrary-input fuzz coverage.

## Independent review and corrections

The default Advisor and `hyper/deepseek-v4-pro-0813` attempts timed out without a review.
`synthetic/hf:openai/gpt-oss-120b` subsequently supplied an independent review.
Its UTF-8-splitting concern was already covered by explicit boundary validation and rejection tests.
Its synthetic-newline claim did not match the implementation's separator branch.
Additional controls nevertheless exercise an authored EOF without newline,
an indented block-doc opener,
final block prose-line deletion,
and insertion with the exact original star prefix.
The review's proposed third-level extraction failure was not adopted:
the agreed depth is two,
so deeper discovery is an intentional stop,
now directly tested.

The first processor gate passed 19 tests and failed one fixture catalog expectation.
The fixture's actual block first-line numbers were 14 and 16,
not 15 and 17;
it also generated Markdown from the doctest's authored `//!` comment.
Those expectations were corrected.
A later full native suite passed 190 tests before Clippy rejected three nested conditional shapes
with `clippy::collapsible_if`.
The shapes were rewritten without weakening lint settings.
The first owned-task invocation also failed before execution because mise interpreted a Podman Go template.
The runner now parses JSON image inspection output.

## Verification runner argument correction

`cargo-mutants 27.1.0` rejected the initial processor campaign before any mutation:
`error: the argument '--in-place' cannot be used with '--jobs <JOBS>'`.
The failed invocation used `--in-place --jobs 1`.
The corrected campaign omits `--jobs` and keeps `--jobserver-tasks 2` plus container resource limits.

Read the upstream `v27.1.0` source at commit
`8ab1dc786a1f61a4e370416cc6c68b81a704e917`.
`src/main.rs:496` calls `Cargo::try_parse()` before dispatch.
The declaration at `src/main.rs:183` makes the conflict unconditional on the supplied job count:

```rust,compile_fail
//! cargo-mutants v27.1.0: src/main.rs:183
#[arg(
    long,
    help_heading = "Copying",
    conflicts_with = "jobs",
    conflicts_with = "copy_opts"
)]
in_place: bool,
```

The [in-place documentation](https://mutants.rs/in-place.html) states that parallel jobs require copied trees.
An explicit value of one still supplies the conflicting option.
Removing the option preserves the disposable in-place campaign and compiled cache,
but does not request parallel mutant workspaces.
The corrected invocation accepted the grammar and caught all selected mutants.
No upstream issue is warranted:
this was a caller argument error,
the restriction is documented,
and no upstream patch or filing was attempted.
The scoped ownership boundary permits this handoff,
not an additional repository troubleshooting document.

## Commit blocker

Commit `e431e93b7` records the initial handoff only.
Implementation and final evidence remain local.
The guardrail blocked selective staging of the processor-registration hunk in `src/lib.rs`,
then blocked an explicit-path implementation commit.
Trust approval was rejected because this delegated session has no interactive approval UI.
No further commit retries are authorized from this session.

The parent agent must inspect and commit:

- `doc/handover/unified-linter-processors.md`.
- `package/linter/monochromatic-lint/mise.toml`.
- `package/linter/monochromatic-lint/fixtures/processors.json`.
- Every new `package/linter/monochromatic-lint/src/processors*.rs` file,
  including the standalone consumer and test modules.
- Only the new processor-registration hunk of `src/lib.rs`.

The pre-existing `lib.rs` fix-loop ordering change,
existing rule/fix-loop edits,
fuzz-sidecar changes,
scanner handoff changes,
and cli-git source changes are concurrent work.
They were not restored,
staged,
or committed by this delegation.

## Evidence/render blocker

The guardrail also blocked the final
`mise run //package/linter/monochromatic-lint:evidence:processors` invocation.
It cited snapshot freshness while the final source gate was completing.
That source gate subsequently passed,
but the blocked command cannot be retried from this session.
A trust request was rejected because no interactive approval UI is available.
The parent must approve and run the evidence task for the final image,
then inspect the rendered output.
Earlier versions of this handoff were rendered successfully;
the final document is not yet render-verified.

## Completion and next action

Processor implementation,
complete native tests,
Clippy,
built-library consumer,
bounded seeded fuzz controls,
and guard-focused mutations are complete.
No implementation-side contract decision remains open.
All scoped code remains local because commits were blocked.
The final hash/render task is also blocked.

The parent session must approve and complete the final hash/render task,
commit only the scoped changes while preserving concurrent work,
then integrate the interface into executable orchestration and global fuzzing.
No production executable cutover occurred.
