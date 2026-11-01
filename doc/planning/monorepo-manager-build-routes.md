# Monorepo manager build routes

## Status and scope

- Status:
   decided.
  Route B was taken:
   the user accepted an all-Rust from-scratch tool on 2026-09-16
   ([`doc/decision/monorepo-manager-all-rust.md`](../decision/monorepo-manager-all-rust.md)),
   and its design is in
   [`monorepo-manager-from-scratch-design.md`](monorepo-manager-from-scratch-design.md).
  Three further decision records followed on 2026-09-17:
   the cache key hash,
   the HCL front end,
   and the per-user configuration.
- Requested 2026-09-16:
   compare extending Bazel against building a replacement from scratch.
- Scope:
   documentation only;
   no code,
   no installs,
   and no effort or duration estimates.
- Evidence sources:
   [`tech-monorepo-manager-vet-2026-09-16.md`](../audit/tech-monorepo-manager-vet-2026-09-16.md),
   [`bazel-migration-dx.md`](../research/bazel-migration-dx.md),
   [`mise-removal-coverage.md`](mise-removal-coverage.md),
   and repository package READMEs.

## Why building

The vet screened 732 entries and re-screened 21 after the user counted external wrappers as plugging in.
No candidate passed.
Candidates that documented pieces of inspection or control,
such as Tilt,
pitchfork,
devtooie,
and Rush through a plugin,
exited on documentation confusion triggers,
as did every candidate with complete documentation coverage.
The user's remaining fallback is building a Bazel replacement;
the user then asked whether Bazel itself can be bent to the requirements.

## Requirements recap

The authoritative checklist is the vet report,
sections "Requirement checklist" and "Frozen hard constraints".

- File-enforcer functionality,
   FE01 to FE24:
   FE14 and FE15 are pluggable by definition,
   FE18 and FE19 may be delegated to Meta Package Manager,
   and the configuration does not have to be TypeScript.
- WR1:
   a persistent watch process keeps task outputs up to date.
- WR2 and WR3:
   another process inspects and controls the running watch process through a documented RPC or IPC channel;
   control covers at least starting,
   rerunning,
   and cancelling work.
- Documentation readable without confusion,
   without JavaScript,
   and without strict bot blocking.
- Tasks for TypeScript,
   Rust,
   Kotlin with Gradle,
   and Zig packages.

## Existing repository assets

Both routes can reuse these;
their READMEs define current scope.

- `@monochromatic-dev/file-enforcer-core`:
   the incumbent for FE01 to FE24,
   including watch mode,
   protected destinations,
   the persistent staleness manifest,
   and the tracking API.
- `@monochromatic-dev/watch-restart-cli`:
   watches source files and restarts one long-running child process;
   it replaced `watchexec` in one dev loop.
- `@monochromatic-dev/monorepo-task-core-exec`:
   task helpers,
   including a `tsc` wrapper and an `oxlint` wrapper that preserves findings and exit status.
- `@monochromatic-dev/mcp-stdio`:
   JSON-RPC 2.0 over newline-delimited stdin and stdout.
  A watch daemon that other processes connect to needs a different transport;
   this package covers message framing only for stdio.
- Meta Package Manager `v7.6.1`:
   adapters for every package manager FE18 dispatches to.

## Route A: Bazel extension layer

### Reused from Bazel

- Declared-input actions with sandboxing:
   Bazel's sandbox "prevents the action from accidentally using any input files that are not declared"
   (`docs/docs/sandboxing.mdx:70-75` in the Bazel source),
   but the same page says actions can still reach files "unless they know the absolute paths to them"
   (`docs/docs/sandboxing.mdx:16`),
   so undeclared reads by absolute path remain possible.
- Live build events:
   the Build Event Protocol is documented for third-party insight into an invocation,
   and `--bes_backend` streams events to any server implementing the public
   `google/devtools/build/v1/publish_build_event.proto` service
   (`docs/remote/bep.mdx`).
- Orderly cancellation:
   exit code 8 documents an interrupted build with orderly shutdown
   (`docs/run/scripts.mdx:53`).
- Generated files in the source tree:
   bazel-lib `write_source_files` updates checked-in files and generates a `diff_test` that fails on drift.
- Dependency sources:
   rules_js reads `lockfileVersion: '9.0'`;
   rules_rust keeps `Cargo.toml` as the dependency source;
   rules_zig supports Zig 0.15.2.

### Built by the repository

- A watch daemon replacing `ibazel`,
   which reports outward only.
- The documented inspection and control RPC,
   backed by a local Build Event Service endpoint.
  Bazel's own client-to-server `Run` and `Cancel` protocol appears only in contributor docs
   (`docs/contribute/codebase.mdx:87-91`),
   so the daemon owns the `bazel` client process instead.
- A second `--output_base` for queries during builds,
   because one Bazel server handles one invocation at a time
   (`docs/run/client-server.mdx:12-14`).
- File-enforcer logic rewritten as Bazel actions with declared inputs,
   plus protected-destination reverts,
   which `write_source_files` does not perform.
- Rule wrappers for tools without Bazel support:
   oxlint including JS plugins and type-aware mode,
   tsdown,
   rolldown,
   and cargo-nextest.

### Inherited gaps

- Bazel exited the vet on the confusion rule:
   `https://bazel.build/docs/configurable-attributes` links "Configurable Build Attributes" to two different URLs.
- rules_android describes itself as an incomplete development preview,
   so the Gradle Android app becomes a port.
- rules_ts expects dependencies to provide `.d.ts` files,
   which conflicts with `/ts` source imports.
- The JetBrains Bazel plugin covers Java and Kotlin only.
- Native pnpm 12 support in rules_js is unreleased as of v3.4.1.

### Pros

- Sandboxing blocks accidental undeclared reads through relative paths,
   which makes affected-only caching more resistant to stale hits,
   though absolute-path reads and untracked host libraries still escape it.
- A mature local and remote cache is available as CI is added.
- Live build events are a documented,
   public protocol.

### Cons

- Every Bazel-facing interaction uses documentation the user's rule rejected.
- The repository still builds the watch daemon,
   the RPC,
   and file-enforcer actions.
- Tool coverage requires new wrappers,
   an Android port,
   and a change to `.ts` source imports.

## Route B: From-scratch replacement

### Reused from the repository

- File-enforcer as-is for FE01 to FE24,
   including its watch loop and staleness manifest.
  Superseded on 2026-09-16:
   the tool must ship as one file,
   a file carrying Node is too big,
   so file-enforcer is rewritten in Rust
   (`monorepo-manager-from-scratch-design.md`,
   "Current stack direction").
- Native tool invocation as Mise tasks use today:
   Cargo,
   Gradle,
   Zig,
   tsdown,
   and oxlint through `task-util`,
   with no rulesets.
- `watch-restart` for long-running child processes,
   as a task the daemon runs;
   TypeScript packages do not ship inside the single file.
- Meta Package Manager for FE18 and FE19.

### Built by the repository

- A workspace and project model with a task graph,
   replacing Mise task configuration and the Node fanout scripts in `mise.no-env.toml`.
- A content-hash task cache with declared inputs.
- A watch daemon that reruns affected tasks.
- The documented inspection and control RPC and its transport.
- User-facing documentation meeting the confusion,
   no-JavaScript,
   and no-bot-blocking rules.

### Pros

- Documentation quality is under repository control.
- No Bazel ruleset gaps:
   tools keep their native build systems,
   including Gradle for Android.
- `/ts` source imports stay unchanged.
  File-enforcer's model was also listed as unchanged until the Rust rewrite superseded that.

### Cons

- Nothing enforces declared inputs unless the repository builds enforcement,
   so stale cache hits from undeclared inputs remain possible.
- Remote caching and execution for CI do not exist unless built.
- The task graph,
   scheduling,
   and caching that Bazel already provides are rebuilt.

## Ranking

Route B over Route A.
The two requirements that eliminated every market tool,
readable documentation and watch with inspection and control,
are repository-built in both routes.
What remains of Route A's advantage is Bazel's task graph,
sandboxed cache,
and build events;
against that,
Route A also inherits documentation the user rejected,
an Android port,
missing oxlint,
tsdown,
rolldown,
and cargo-nextest rules,
and a conflict with `/ts` source imports.

The strongest counterpoint was cache correctness:
sandbox-enforced inputs or a mature remote cache for CI would have given Route A a capability Route B must build.
The user's answers,
recorded in "Answered questions",
accept lint-level input enforcement and a local cache,
so the ranking holds.

## Answered questions

Answered by the user on 2026-09-16;
no answer flips the ranking.

- Input enforcement:
   OS-level sandbox enforcement is not required.
  The user noted that file-enforcer does not force every read through `cat`,
   and had considered an oxlint rule forbidding `readFile` and similar direct reads in `file-enforcer.config.ts`,
   an idea not previously documented.
  Lint-level enforcement of that kind is "okay-ish",
   so static enforcement meets the bar.
- Remote cache:
   a local cache is enough for now.
- Sandboxing:
   later the same day the user made sandboxing with at least cgroups a must,
   and made every 0.x capability Linux only.
  Cgroup v2 provides resource control and freezing but no filesystem access control,
   so this does not replace the input-enforcement answer.
  Route B must build cgroup sandboxing;
   Bazel exposes experimental cgroup startup options such as `experimental_cgroup_parent` and
   `experimental_run_in_user_cgroup`.
  The Bazel design found per-action cgroups only for `linux-sandbox` actions with limit flags,
   and a silent fallback when cgroups cannot be created
   ([`monorepo-manager-bazel-route-design.md`](monorepo-manager-bazel-route-design.md),
   "Sandboxing").
- Platforms:
   everything in 0.x is Linux only,
   per the user on 2026-09-16.
  The vet's HC5 still lists macOS and Windows CI runners,
   a conflict recorded under "Risks" in the from-scratch design.

## Questions resolved by the designs

- RPC transport:
   the from-scratch design proposes a filesystem Unix socket under `$XDG_RUNTIME_DIR`
   with newline-delimited JSON-RPC 2.0;
   HTTP on loopback is not proposed.
- Mise removal ledger:
   the from-scratch design keeps tool provisioning,
   environment,
   and secrets as separate owners per `mise-removal-coverage.md`.

## Design findings so far

Research appendices live in [`monorepo-manager-route-research/`](monorepo-manager-route-research/).

- Bazel route,
   Rust dependencies,
   verified 2026-09-16:
   all 19 `Cargo.toml` files carry an explicit `[workspace]` table,
   and `doc/planning/cargo-toml-file-enforcer.md` records the no-workspace architecture as deliberate.
  rules_rust `crate_universe` refuses to splice manifests from more than one workspace root
   (`crate_universe/src/splicing/splicer.rs:58-60` at the rules_rust clone).
  Bazel would need one dependency resolution per crate
   or a Bazel-only resolution that diverges from the Cargo lockfiles Cargo keeps using.
- Bazel route,
   other native-language findings in `bazel-native-languages.md`:
   no Android instrumentation test rule,
   no Android Lint support,
   no Bazel Android support in the IntelliJ IDEA setup the repo uses,
   no build rule for the Ghostty Zig build,
   crates.io publishing and Rust fuzzing staying on Cargo,
   host system libraries outside Bazel's input tracking,
   and a silently dropped `rustc-link-arg-bin` rpath.
- Bazel route,
   TypeScript lint cache,
   verified 2026-09-16:
   `prefer-readonly-parameter-type` stores its effect cache under the nearest `pnpm-lock.yaml` directory's
   `node_modules/.cache/prefer-readonly-parameter-type`,
   with an override only as a function argument used by tests
   (`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/effect-summary-cache-identity.ts:223-275`).
  Inside a sandboxed lint action that cache does not persist,
   so every lint action starts cold;
   repository records put cold whole-repo sweeps at 261 to 305 seconds.
- Bazel route,
   other TypeScript findings in `bazel-typescript.md`,
   from source reading without running Bazel:
   pnpm `overrides` shims and `.pnpmfile.mjs` stubs are expected to make rules_js
   reference package-store targets it never creates;
   tsgo resolves real paths natively while Node is kept inside the sandbox;
   234 test files import bundled `dist` output,
   so shared-module edits rebuild and retest bundling packages;
   aquati.cat reads `git log`,
   keeps a source-tree cache,
   and rewrites `dist` in place.

## Design result

- Bazel route:
   [`monorepo-manager-bazel-route-design.md`](monorepo-manager-bazel-route-design.md)
   concludes the route is not viable without giving up the no-workspace Rust architecture,
   the user's documentation rule,
   Android testing and lint parity,
   or cgroup sandboxing that fails closed.
- From-scratch route:
   [`monorepo-manager-from-scratch-design.md`](monorepo-manager-from-scratch-design.md)
   records the user's daemon requirements,
   a design,
   ranked stack options,
   risks,
   and open questions.
- Ranking after design:
   Route B over Route A,
   now because Route A hits the disqualifying problems in its design document,
   not only on documentation and tooling gaps.

## Next action

On 2026-09-16 the user asked for design documents for both routes,
expecting a concrete Bazel design to surface any disqualifying problem;
both documents now exist,
and the Bazel design records disqualifying problems.
Current work and the next action are tracked in
[`doc/handover/monorepo-manager.md`](../handover/monorepo-manager.md).
No code is written before a route is chosen.
