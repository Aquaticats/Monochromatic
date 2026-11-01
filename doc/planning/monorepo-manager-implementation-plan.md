# meow 0.x implementation plan

## Status

Plan only,
initially written 2026-09-17 after the research queue closed;
no code exists and none is authorized yet (rule `VRB`).
The user accepted resource-aware package maintenance rather than treating development stages as special tasks.
The [software model exploration](monorepo-manager-software-model.md)
records that accepted direction and the distinctions it preserves.
The user requested continued grilling with an implementation-ready 0.x plan as the session's deliverable.
Milestones that assume a task-centric schema or an unsettled lifecycle remain provisional
until the concrete contracts in the decision frontier are resolved.
Design:
[`monorepo-manager-from-scratch-design.md`](monorepo-manager-from-scratch-design.md).
Decisions this plan implements:
[`monorepo-manager-all-rust.md`](../decision/monorepo-manager-all-rust.md),
[`monorepo-manager-cache-key-hash.md`](../decision/monorepo-manager-cache-key-hash.md),
[`monorepo-manager-hcl-front-end.md`](../decision/monorepo-manager-hcl-front-end.md),
and
[`monorepo-manager-per-user-config.md`](../decision/monorepo-manager-per-user-config.md).
Session state:
[`doc/handover/monorepo-manager.md`](../handover/monorepo-manager.md).

Everything here is scoped to meow 0.x;
1.x and later may revisit any of it (user,
 2026-09-17).
The user explicitly deferred the publishing subsystem to 1.x during the non-task-model discussion.
Its future push-to-`main` trigger is recorded in the design,
not an addition to this implementation plan or a 0.x blocker.
No estimate of effort or duration appears anywhere in this plan (rule `QJ1`).

## Active design work

Use the [design workpad](monorepo-manager-design-workpad.md) to work through concrete cases,
transitions,
and counterexamples in writing before pursuing another reasoning branch.
It contains unfinished reasoning,
not merely conclusions recorded afterward.
Accepted decisions stay in this plan and the main design.

## Completion contract and decision frontier

This plan is ready for implementation only when its 0.x requirements have concrete owners,
interfaces,
configuration examples,
state transitions,
and acceptance tests,
with no unresolved implementation-blocking user choice.
The user then confirms alignment.
Product implementation is not part of this design session.

The design tree has these major areas:

- Maintained concerns and authoring:
  native-tool coverage,
  discovery,
  build outputs,
  checks,
  managed-file rules,
  one-off operations,
  labels,
  and HCL selection and inheritance semantics.
- Freshness and lifecycle:
  eligible progression,
  input changes,
  result applicability,
  output ownership,
  cache behavior,
  priorities,
  pause,
  end,
  and retry.
- Commands and environment:
  foreground behavior,
  blocked requests,
  output and errors,
  concurrent clients,
  trust and reload,
  daemon startup and shutdown,
  cgroup prerequisites,
  and doctor.
- Enforcement and migration:
  responsibility coverage,
  parity tests,
  external configuration owners,
  and the distinction between a 0.x deliverable and eventual full-platform cutover.
- Implementation structure:
  reusable package ownership,
  interfaces,
  milestone dependencies,
  and executable acceptance criteria.

First interview round,
accepted:

- Q1:
  finish independent source checks after a failure.
- Q2:
  foreground requests honor check gates.
- Q3:
  cancel superseded ordinary background builds and read-only checks.
- Q4:
  waiting foreground maintenance requests follow the latest relevant inputs.
- Q5:
  `meow end` is forbidden for automatic work in 0.x.
  The user rejected both proposed restart-lifetime choices.
- Q6:
  ordinary-source lint autofixes require explicit intent.

Second interview round:

- Q7 accepted:
  Ctrl+C detaches the foreground request from automatic maintenance,
  which continues.
- Q8 accepted:
  a currently blocked foreground request reports the blocker and returns nonzero.
- Q9 clarified:
  every execution has defined inputs and outputs.
  To obtain re-execution,
  an author can add an intentionally varying input such as `currentDateTime`.
  The per-definition reuse-toggle proposal is rejected;
  ordinary cache identity still governs explicit operations.
- Q10 correction:
  distinct invocations use the ordinary priority queue.
  Concurrency and pausing follow that scheduler,
  not a special target-wide serialization rule.
  The earlier same-target-duplicate wording must not be interpreted as label-based exclusivity.

Q9 is settled:
there is no configurable execution-cache bypass.
Concrete input binding must define volatile-value sampling and preserve a resolved input set during a computation.

The next model prerequisite is native package and concern identity:
what a package-scoped label covers when a package combines ecosystems or native subprojects.
That answer informs authoring,
control selection,
and actual input/output relationships.
The lifecycle work remains incomplete while these prerequisites are resolved.

Native-scope evidence verified directly:

- `package/music-player/android-app/settings.gradle.kts` declares the Gradle `:app` project.
- `package/music-player/android-app/rust/Cargo.toml` declares the standalone `musicplayer-native` crate
  with a `cdylib` product and its own workspace boundary.
- `package/music-player/android-app/mise.toml` names `app/src/main/jniLibs` as the native output destination.
  Its current `lint` selects Android Lint,
  while `lint:clippy` and `lint:rust` separately examine the Rust part.

Q11 A is accepted:
a package-wide maintained concern such as
`meow run //package/music-player/android-app:lint`
covers all applicable native parts owned by that package.
Narrower addresses remain possible.
Ownership inference,
address syntax,
and automatic variant selection are not settled by this answer.
Q12 A is also accepted:
when native declarations do not establish membership for a nested project,
the root configuration must provide the missing connection.
The user accepts the extra authoring effort and clarified that both example groups need separate packages:
the Android Gradle application and Rust engine,
and the nested fixture cases.
Do not preserve their incumbent grouping as a requirement.
Q11 covers owned parts within one package,
not automatic absorption of independent packages or dependencies.
No directory relocation or source migration is authorized here.

The next worked question,
Q13,
is whether a dependency's failing source checks block production in a consumer that reads its source directly.
The [workpad](monorepo-manager-design-workpad.md) separates that gate from an unnecessary dependency build
and from the still-unsettled role of dependency runtime tests.

Exact cache,
output,
and command consequences follow the answers rather than being silently bundled with them.
The authoring and runtime audits returned evidence pointers;
findings must be checked against the current records before becoming decisions.
Their stale report that progression was unaccepted was already resolved before the reports arrived.
Publication is excluded entirely from this frontier.

## Where the work happens

- meow is built in its own git worktree,
   so the TypeScript file-enforcer and meow never enforce the same tree during development.
- The TypeScript file-enforcer keeps running until meow replaces it,
   and the root `mise.toml` stays generated from `mise.no-env.toml` until Mise is removed.
- Mise is removed entirely,
   in one step,
   only when meow supports the full platform matrix including macOS and Windows.
- Rules `MXR` and `RDC` apply to every `.rs` file:
   300 code lines per file,
   rustdoc on every documentable item,
   with tests and fuzz targets exempt.

## What each milestone must prove

A milestone is done when its named evidence exists and is committed,
not when its code compiles (rule `VUB`).
Every milestone carries a parity or conformance target measured against something that exists today.

### M1: the binary and its output contract

- A single Rust binary built for every shipped target:
   x86-64 baseline,
   `x86-64-v2`,
   `x86-64-v3`,
   and `x86-64-v4`,
   plus aarch64,
   each glibc-linked and static musl,
   with aarch64 musl as static-pie from the custom target.
- Every line the daemon writes to standard output or standard error is a JSON object,
   including logs,
   warnings,
   and errors.
  A task's own bytes are not meow's messages:
   `meow run` forwards them as-is to the terminal that asked
   ("User interface" in the design).
- The startup capability check runs before any hashing in builds raised above their target baseline,
   printing a JSON diagnostic and exiting non-zero rather than dying with SIGILL.
- Evidence:
   the matrix probe repeated for every shipped build,
   plus a run on a CPU model lacking the raised build's features that exits with the diagnostic.

### M2: content hashing and the cache key format

- XXH3-128 through `twox-hash` pinned at `=2.1.4`,
   default seed,
   full 128 bits,
   one-shot for in-memory files and streaming for larger inputs.
- The key format fixes the seed and the byte order of the `u128`.
- Evidence:
   golden vectors in meow's own tests,
   including the empty input `99aa06d3014798d86001c324468d497f`;
   a streaming-against-one-shot equality test over many chunkings;
   and a digest comparison across two of the shipped builds.

### M3: the HCL front end

- `hcl-edit` 0.9.7 vendored with the two prototype patches,
   under meow's own evaluator over its tree,
   with byte spans on every diagnostic.
- A nesting-depth pre-scan before parsing,
   because `hcl-edit` aborts the process at depth 5,000 with no knob.
- Evidence:
   the 2,246-file corpus accepted and rejected exactly as the Go reference does;
   the round-trip probe byte-identical on all 2,185 valid files with the patches applied;
   and the evaluator's differential suite against the Go oracle covering the 21 cases where `hcl-rs` diverged.

### M4: the function library and the read set

- The curated OpenTofu-named set plus `meow::` namespaced additions.
- I/O functions record `File`,
   `Absent`,
   `Directory`,
   `Glob`,
   and `Env` reads into a read set that becomes both the cache key input and the watch list.
- `timestamp`,
   `uuid`,
   and `bcrypt` exist,
   mark their evaluation uncacheable,
   and emit a diagnostic naming the function,
   the block,
   and the caching it disabled.
- Evidence:
   every logic unit in the inventory (LI01 to LI22) expressed in HCL and evaluated,
   with the read set for each compared against what the TypeScript implementation reads today.

### M5: configuration discovery, layering, and trust

- Discovery order,
   the `--no-user-config` switch,
   per-attribute declared scope,
   and outside-write proposals matched by per-user acceptances.
- Trust as cli-git does it:
   explicit `trust`,
   `trust --yes`,
   `untrust`,
   and `status`;
   configuration-loading commands blocked until trust exists;
   identity as the filesystem ID paired with the canonical path;
   exact-byte snapshots evaluated in place of the live file;
   the registry under the account home with atomic replacement and private modes.
- Evidence:
   a disposable-fixture suite covering each discovery branch,
   a blocked command whose JSON diagnostic names both recovery commands,
   and a compare-then-swap attempt that the snapshot evaluation defeats.

### M6: file enforcement parity

- The 27 logic units re-expressed and executed,
   with managed edits preserving comments in TOML,
   JSONC,
   and XML through the chosen editors.
- Evidence:
   the differential edit harness's 21 TOML,
   16 JSONC,
   and 9 XML cases passing;
   a full run over this repository whose only output differences are the one-time reviewed reformatting;
   and the corpus round trips for each editor.

### M7: the daemon

- One daemon per canonical repository root,
   socket under `$XDG_RUNTIME_DIR`,
   watching the repository and the per-user configuration's directory.
- The scheduler,
   priorities,
   concurrency,
   task control,
   cgroup sandboxing,
   and the cache with pointer-only outputs,
   failure caching,
   and eviction by pinned bytes plus 30-day age.
- Reload carries a generation number,
   in-flight tasks stay pinned,
   and a malformed configuration keeps the last good snapshot.
- `meow run` against the daemon:
   queued at an integer priority above background work,
   with the newest such task holding the highest priority and pausing others as needed;
   holding the terminal;
   forwarding the task's bytes as-is on a miss;
   replaying the stored result on a hit behind a one-line marker;
   and prompting,
   when no daemon is live,
   to spawn a `meow watch` terminal or start one detached.
- Evidence:
   a watch session that rebuilds affected work on a change;
   a cache hit and a forced miss with their keys recorded,
   the hit replaying without executing anything;
   a task frozen,
   resumed,
   and ended through RPC;
   an eviction run against a filled cache;
   and a `meow run` whose forwarded bytes match the task's own output exactly.

### M7a: the rest of the command surface

Settled during the UX alignment on 2026-09-17
("User interface" in the design),
after this plan was first written:

- `meow status`,
   listing failed,
   paused,
   blocked,
   flaky,
   queued and running work,
   never successes,
   with its layout deliberately unchosen because it is the TUI's design problem.
- `meow stop`,
   beside Ctrl+C in the watch terminal,
   ending running tasks through their cgroups before the daemon exits.
- `meow pause`,
   `meow resume`,
   `meow end`,
   and `meow priority`,
   naming tasks by target label,
   with `priority` taking an absolute integer or a signed bump.
- The trust commands,
   whose review prints every file that would be evaluated,
   unpaged,
   before asking;
   `meow trust --yes` is the non-interactive path an agent in a disposable worktree uses.
- Evidence:
   each command driven against a live daemon,
   plus a blocked command whose JSON diagnostic names both recovery commands.

### M8: the language server and `doctor`

- `lsp-server` 0.10.0 with `gen-lsp-types` behind a hidden `meow lsp` stdio subcommand,
   offering diagnostics,
   completion,
   hover,
   and go-to-definition over the configuration.
- `doctor` reporting the environment problems it can name,
   with the exact change and its reason.
- Evidence:
   an editor session driven end to end against a real configuration,
   and a `doctor` run on a deliberately broken environment.

### M9: cutover

- The `exec` replacement lands with the rewrite:
   `package/vm-builder/cli/src/process.ts` over `nano-spawn`,
   and vm-builder's dependency on file-enforcer removed.
- The `prefer-readonly-parameter-type` read moves to its own fixture package.
- The performance fixture retires with the TypeScript implementation,
   with no speed gate on the rewrite.
- Mise and its configuration leave once the full matrix is supported,
   and `sync:files` goes with them.
- Evidence:
   the Mise removal ledger with every consumed responsibility marked owned and verified,
   and a release containing the shipped build matrix.
  Releasing meow through the existing release infrastructure is distinct from implementing
   the publishing subsystem deferred to 1.x.

## What the language questions still block

The configuration language is not finished.
The built-in-name,
override,
and namespace questionnaire is withdrawn:
it assumed that development concepts are special tasks.
The user emphasized a coherent continuous-maintenance model across build,
correctness,
and publication rather than choosing between state and stages.
The [software model exploration](monorepo-manager-software-model.md)
records the current package-maintenance proposal and the user's correction of the publication detour.
Publication is absent from 0.x;
its future eligibility rules do not block this plan.
The active model needs to explain intrinsic build outputs,
correctness checks,
and managed-file requirements without reducing them to privileged generic tasks.
Ordering remains undecided,
and the unapproved `depends_on` entry stays retracted.

- M4 and M6 remain provisional where configuration schema and file-enforcement scheduling
  assume the unaccepted task model.
- M3's HCL syntax and expression semantics do not depend on selecting a task schema.
- M7 and M7a need the accepted model before defining identity,
  readiness,
  label resolution,
  and ordering.
  Their existing watch,
  cache,
  process-control,
  and command behavior requirements remain constraints,
  not evidence that a task must be the primary domain object.

## Ordering and what blocks what

- M1 blocks everything,
   because the output contract and the build matrix decide how every later diagnostic and test is written.
- M2 blocks M6 and M7,
   since both key their work by content hash.
- M3 blocks M4 and M5;
   M4 and M5 together block M6.
- M7 depends on M6 for the work it schedules,
   and on M2 for its cache.
- M8 depends on M3 for spans and on M5 for what a query may answer before trust.
- M9 depends on M6 and M7 reaching parity,
   and its Mise removal step additionally waits for macOS and Windows support.
- M9's `exec` replacement is the one piece with no dependency on meow;
   the user chose to land it with the rewrite rather than earlier.

## Risks carried into implementation

These come from the design's "Risks" section and the decisions,
and each needs an owner when its milestone starts:

- Every subsystem is repository-built:
   task graph,
   cache,
   watcher,
   scheduler,
   cgroup sandbox,
   RPC,
   and their documentation.
- Whether GitHub runners give a systemd user session for delegated cgroups is unverified.
- Frozen tasks hold locks while timers run,
   and ending a task's cgroup can kill a daemon another task reuses.
- Undeclared reads in repository tasks leave stale cache hits possible.
- The vendored `hcl-edit` patches must be re-applied and re-measured on every upstream release;
   the corpus round-trip probe is that regression test.
- meow owns HCL semantics,
   and the Go oracle harness is what keeps its corner cases honest.
- The rewrite ships without speed evidence,
   because the user chose no speed gate.
- The aarch64 speed evidence behind the hash choice is Apple silicon on Darwin,
   not a release-blocking Linux target.

## Modularity

Stated by the user on 2026-09-17:
"Everything that could possibly be modular should be reusable package/library."

- Each milestone delivers reusable repository packages rather than private modules,
   and `./meow` composes them into the single binary.
- M1 therefore includes the logger package,
   not a logging module inside meow.
- The sink model is the repository logger's:
   verify each backend,
   fan out to every verified sink,
   and refuse to start when none verify.
  0.x ships the terminal stream and a timestamped JSONL file in meow's state directory.

## What this plan does not decide

- The names and boundaries of those packages,
   beyond the subsystem list in the design's "Modularity".
- The wording of user-facing diagnostics,
   which waits on meow's CLI naming.
- Whether the repository ever gains a shared process-execution owner;
   the user answered no for now.
- Anything for 1.x.
