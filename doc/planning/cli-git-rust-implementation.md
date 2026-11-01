# cli-git and unified-linter Rust implementation proposal

## Authority and scope

The user confirmed the scope in
[`cli-git-rust-rewrite.md`](cli-git-rust-rewrite.md)
and asked to see the implementation approach before code changes.
The user subsequently assigned this agent ownership of the unified linter as well,
selected JSONC for that tool,
and required container tests,
mutation testing,
and fuzzing.
The language choice is settled.
The expanded cli-git 2.x policy catalog is excluded;
the unified linter's already agreed Rust,
Markdown,
and MDX replacement is included.
Implementation authorized by the user's "Do it" on 2026-10-04.
Execution state is tracked in
[`cli-git-rust-implementation.md`](../handover/cli-git-rust-implementation.md).

The Rust wrapper supports the latest stable Git release only.
Resolve and record that exact release in build and test evidence.
Do not port older-Git compatibility branches or keep an old-version test matrix.
Normal command execution does not query the network to discover a Git release.

## Shape

Build a Rust library and a thin `git` executable under the existing
`package/git-policy/cli` ownership.
Use internal modules rather than introducing a crate for every responsibility.
Keep the TypeScript executable available until the Rust replacement passes the consumer contracts.
Do not introduce a permanent TypeScript bridge,
a daemon,
or a general plugin runtime.

The executable remains a Git wrapper,
not a replacement Git implementation.
Keep native Git responsible for object storage,
refs,
hooks,
signing,
transport,
editors,
filters,
and sequencer behavior.
Preserve Git argument bytes and operating-system path representations;
do not reconstruct commands as shell strings or require every path to be UTF-8.
Port Git-specific token classification rather than handing all Git arguments to an unrelated option grammar.

## Module responsibilities

### Configuration

Read repository-root `cli-git.config.jsonc` through
`package/jsonc-edit/core-rust`.
Validate the document into typed settings for the compiled-in policy registry and existing concurrency controls.
Report unknown keys,
unknown policy IDs,
ambiguous duplicate settings,
and invalid values as configuration errors.

There are no imports,
callbacks,
plugin package names,
command arrays,
or executable-path overrides.
Repository settings can disable policies or change permitted severities without consent.
Do not add a personal policy floor or replace trust prompts with another approval mechanism.

Delete executable-config discovery,
bundling,
stored-code evaluation,
consent,
and trust-registry management.
Keep the command-classification fast path:
operations that need no policy configuration must not initialize the policy engines.
Keep transaction and worktree recovery even where their present implementation imports helpers from `trust/`.
Move retained helpers according to responsibility,
not their old directory name.

### Git operations and candidate content

Port real-Git resolution,
wrapper-self exclusion,
platform-specific executable lookup,
child environment,
signals,
and exit handling into Rust.
Do not call the TypeScript resolver from the Rust executable.
The TypeScript resolver used by other consumers needs corresponding native-wrapper recognition at cutover.

Expose immutable candidate versions and lazy,
batched Git facts internally.
Read committed or staged candidate bytes,
not whatever happens to be in the live worktree.
Reuse object reads and path metadata within one candidate version;
invalidate affected facts after a fix or replay.
Avoid a Git process per file or per historical blob.

### Built-in policies and scanner integration

Use a fixed typed registry of shipped policies with stable ordering,
validated options,
structured findings,
and bounded fixing passes.
Preserve `check`,
`fix`,
manual-push checks,
JSONL events,
stream routing,
and exit-code semantics.

Initial coverage is the existing required behavior:

- root,
  linked-worktree,
  branch-worktree,
  explicit-add,
  and final-newline policies;
- fixed atomic-push,
  commit-only,
  and status-hint transforms;
- forbidden-strings;
- forbidden-root-context and dependent-version propagation;
- current commit-time Markdown normalization.

Link the existing forbidden-strings Rust library into cli-git.
Add a library interface that accepts candidate identity,
logical pathname,
and exact bytes,
and returns structured redacted findings.
Route the standalone scanner CLI through the same core;
do not fork the rule engine or change its exit contract.

Preserve pathname scanning,
content scanning,
rule identities,
redaction,
file-selection semantics,
fail-closed errors,
and the engine-panic guard.
Load the rules once per invocation when scanning is enabled,
and reuse them across candidates and valid repeated policy passes.
Retain the existing validated runtime-rule cache and built-in precompiled rules.
A replay that changes a declared input invalidates the corresponding policy result.

This removes the current scanner integration's temporary content files,
scanner child process,
and parsing of rendered stderr.
It is a concrete execution-path change,
not a quantified speedup claim.

### Unified linter and Markdown integration

This agent owns both tools.
Implement the full accepted
[`unified-linter.md`](unified-linter.md)
design in `package/linter/monochromatic-lint`,
including Rust,
Markdown,
MDX,
processors,
fix mapping,
and JSONL output.
Do not implement a second Markdown rule engine inside cli-git.

Use `package/jsonc-edit/core-rust` for both tools,
with separate typed schemas:
cli-git's policy settings and the linter's ordered configuration blocks are different data models.
The linter's filename is `monochromatic-lint.config.jsonc`.
Preserve its accepted nearest-config lookup,
block ordering,
per-rule defaults,
and deepmerge semantics.
The user subsequently authorized a simple handwritten merge module for now.
Implement the agreed JSONC merge semantics inside the linter;
do not build the separate deepmerge port or wait for the fork.

Keep the already accepted linter CLI integration rather than introducing a new public library interface
or `--rule` flag.
Cli-git's built-in Markdown policy selects the coordinated installation's owned native linter,
creates a one-rule temporary JSONC configuration,
and uses `--config`,
`--stdin`,
`--stdin-filename`,
and `--fix` with the existing fixed-source/JSONL stream contract.
Repository JSONC cannot choose the executable or inject leading arguments.
First-party linter selection is installation wiring,
not a generic custom-policy facility.

Preserve the LFS-image rule's findings and localized edits,
including astral Unicode and BOM fixtures.
Do not carry over the known double offset correction.
The earlier linter design deferred a production parse-time budget;
this assignment does not silently reverse that choice.
Container and fuzz runs still bound malformed-input work and retain hanging inputs as evidence.

### Commit transactions and recovery

Port the transaction state machine and durable protocol explicitly.
Typed states represent capture,
private preparation,
policy convergence,
landing,
replay,
and post-landing completion.

Preserve:

- private indexes and shadow repositories;
- hook,
  editor,
  signing,
  and branch identity behavior;
- compare-and-swap landing,
  replay,
  policy read sets,
  capture ordering,
  and starvation reservations;
- protection of pending objects and post-landing automatic maintenance;
- real index and worktree isolation;
- process-birth-aware locks,
  foreign-lock evidence,
  and cancellation;
- durable recovery after interruption;
- current auto-push completion and failure reporting.

Keep the existing journal and lock formats where practical during the first release,
with cross-version fixture verification.
Do not silently discard old transaction state or invent fresh empty state when recovery is required.
Do not intentionally switch wrapper versions during a live transaction.
If a state cannot be read safely,
fail with an actionable recovery diagnostic rather than guessing or deleting it.

Use the capabilities of the supported latest Git release directly.
Report missing required Git behavior as an unsupported-environment failure,
not an invitation to fall back to an older transaction algorithm.

### Linked-worktree state

Port worktree-copy as its own lifecycle module,
including ignored-state selection,
copy-on-write requests,
exact existing-entry checks,
journals,
containment validation,
and crash recovery.
Preserve the main-worktree bypass and the distinction between worktree settlement locks and commit landing locks.
The trust subsystem's removal does not remove these locks or journals.

## Implementation sequence and verification

1.  Freeze a behavior ledger from `package/git-policy/cli/SPEC.md` and the accepted platform and concurrency decisions.
    Mark executable-config and plugin-authoring behavior as intentionally retired.
    Every retained responsibility receives a Rust owner and a consumer-level test.
2.  Add the Rust crate,
    direct executable entry,
    JSONC loader,
    Git resolver/forwarding,
    diagnostics,
    and package-scoped build/lint/test tasks.
    Verify exact forwarding and self-recursion prevention before mutating Git operations.
3.  Expose and test the scanner's structured library interface.
    Add the shipped policy registry and the existing non-Markdown policies.
    Verify planted findings,
    clean controls,
    candidate-byte isolation,
    redaction,
    fixing convergence,
    and direct `check`/`fix`.
4.  Implement the unified linter according to its accepted build order,
    using JSONC instead of HCL.
    Port the existing Rust and Markdown rules,
    processors,
    fix mapping,
    and CLI contracts.
    Verify rule findings and byte-exact fixes before wiring cli-git's built-in Markdown policy to it.
5.  Port transaction,
    locking,
    hook dispatch,
    replay,
    worktree-copy,
    recovery,
    and push behavior.
    Exercise crash points and concurrent actors,
    not only successful sequential commits.
6.  Complete the container,
    mutation,
    and fuzz gates for both tools and their integration.
    Reuse the existing end-to-end scenarios against the Rust executable on the latest Git,
    adapting the driver rather than reproducing its logic in a new harness.
    Use equivalent disposable repositories for semantic comparisons with the incumbent and native Git.
    This verifies behavior,
    not whether to rewrite.
7.  Package and verify the installed native executables,
    then perform the coordinated cutover.
    Delete retired implementations and authoring surfaces only after their consumers have moved.
    The unified linter's first-publication approval remains a separate requirement.

Replace the old multi-version Git fixture with the exact latest stable release selected for the run.
Keep the existing Linux,
macOS,
and Windows consumer coverage.
Tests cover argument syntax,
non-UTF-8 and adversarial names,
partial staging,
policy failures,
hooks,
signed commits,
amend/sequencer conclusions,
concurrent races,
interruption,
recovery,
and worktree state.
Do not blindly preserve an incumbent bug merely because differential output matches it;
the accepted behavior and named regressions are the oracle.

Measure the release Rust artifact at the real command interface as an acceptance check.
Verify benchmark reach with positive controls,
report local processing separately from remote push latency,
and never substitute earlier shell return for completing required work.
Do not restart the canceled incumbent profiling campaign.

## Verification gates

### Container tests

Exercise the installed Rust binaries in disposable repositories,
with inputs baked into the image,
no real home or credentials,
and local bare remotes.
Use bounded memory,
CPU,
process counts,
and workload sizes;
network access is disabled during execution.
Run build and fixture setup separately from the measured operations.
Native macOS and Windows jobs remain necessary for their filesystem and process contracts.

Cover cli-git commits,
partial staging,
hooks and signing,
concurrent landings,
process termination and restart,
recovery,
worktree copies,
and push failures.
Cover linter configuration discovery,
merging,
walking,
stdin fixing,
processor positions,
atomic file writes,
and the combined commit-time Markdown path.
Capture stderr and require orderly shutdown without bare cleanup errors.

### Mutation testing

Apply mutations to repository-owned implementation code,
not only a selected happy-path module.
Target policy enabling and severity,
JSONC validation,
configuration precedence,
rule predicates,
fix overlap and atomicity,
virtual-to-host mapping,
transaction state transitions,
lock ownership,
recovery,
and scanner fail-closed behavior.

An unmutated baseline must pass.
Planted guard removals must make the corresponding test fail before trusting the mutation harness.
Inspect survivors:
strengthen tests for non-equivalent mutants,
and document equivalent,
unreachable,
or excluded mutants with evidence.
Do not equate a mutation-tool exit code with adequate coverage.

### Fuzzing

Add sibling fuzz packages for cli-git and the unified linter,
following the existing `jsonc-edit.fuzz` and `forbidden-strings.fuzz` conventions.
Keep and extend those existing dependency fuzz suites;
they do not replace fuzzing the integration boundaries.

Targets include:

- Git argument classification,
  separators,
  byte-valued paths,
  and JSONC schema handling;
- ordered configuration merge values and per-rule settings;
- scanner candidate identities,
  pathname/content distinction,
  rule loading,
  and redacted findings;
- Markdown/MDX,
  Rust snippets,
  comments,
  nested processors,
  Unicode,
  BOMs,
  and byte-to-host positions;
- edit sets,
  overlap rejection,
  all-or-nothing fixes,
  and convergence;
- serialized journals and bounded stateful transaction sequences,
  including interruption points and concurrent actors.

Run fuzzing and stateful process tests under explicit resource and input-size limits.
Retain seeds and corpora,
replay known failures in CI,
and convert minimized counterexamples into deterministic regression tests.
A fuzz crash,
hang,
or suspicious surviving mutation is investigated rather than filtered away.

## Cutover

Build and verify the native artifact before changing command resolution.
Use an explicit native path in disposable tests until the shadowing install is verified.
The installed `git` launcher resolves directly to the native executable,
not to a Node process that starts it.
Keep standalone forbidden-strings available for its other consumers.

Update together:

- root cli-git configuration;
- package tasks and installed-bin wiring;
- real-Git resolver self-exclusion,
  including its surviving TypeScript consumers;
- file-enforcer-owned scanner/build integration where the ownership changes;
- CI,
  README,
  and the surviving management-command documentation.

Legacy `.ts` or `.mjs` configuration gets an explicit migration diagnostic,
not execution or silent ignoring.
Translate this repository's inspected config directly;
do not build a general program-to-JSONC converter.
Retired trust commands explain that JSONC no longer requires code-execution approval.
Old trust records remain inert rather than being deleted automatically.
Keep the previous executable available for rollback until native installation and recovery checks pass.
