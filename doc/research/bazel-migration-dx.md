# Would a Bazel migration drastically improve developer experience?

## Research status

- Status:
   complete for the question asked;
   inference from measured repository shape plus ruleset source,
   not a measured before-and-after pilot
- Date:
   2026-09-16
- Method:
   `mise tasks --all --hidden --json`,
   `gh run list`,
   `git shortlog`,
   existing timing records in `doc/planning/`,
   shallow clones under `~/temp/agent/<repo>-2026-09-16`
   (rules_js `3f5022d`,
   rules_ts `bfbf401`,
   rules_lint `4cb8c45`,
   rules_rust `dd2b107`,
   rules_kotlin `02f96a1`,
   rules_android `7bd7cbb`)
- Product changes:
   none
- Scope limit:
   this is not a `choosing-technology` selection.
   No discovery saturation ran and no build tool is recommended for adoption.

## Answer

No drastic improvement is expected.
Bazel would close one real gap,
the absence of incremental task skipping,
but the measured slow paths are not graph-shaped,
the repository has one active author and no repo-wide CI build,
and several load-bearing tools have no maintained Bazel integration.

## Measured repository facts

- `mise tasks --all --hidden --json` reports 1,825 tasks:
   0 declare `sources`,
   0 declare `outputs`,
   41 declare `depends`.
  Every invocation re-executes;
   nothing is skipped as fresh.
- Root fanout in `mise.no-env.toml` is repository-owned Node code.
  The reason given for it,
   that the native Mise monorepo glob aborted aggregates mid-flight,
   has no primary evidence:
   `mise.toml:308-312` attributes the observation to `doc/handover/lint-fix-2026-06.md`,
   which does not contain it
   (checked 2026-09-17,
   `rg 'glob'` over its 199 lines returns nothing),
   and `doc/decision/desktop-app-podman-build.md:84-87` still tells readers to use the native glob.
  Tracked in issue #555.
- `git shortlog --since=2025-09-16` attributes 8,699 commits to one author,
   36 to a test fixture identity,
   and 6 to `github-actions[bot]`.
- The 200 most recent non-skipped,
   non-cancelled GitHub Actions runs have per-workflow median durations from 23 to 303 seconds.
  No workflow builds or tests the whole repository.
- Workspace packages import each other's TypeScript source through the `/ts` subpath
   (`doc/decision/workspace-ts-source-imports.md`),
   so there are few built-output edges between packages.
- Recorded slow paths:
  - warm whole-repo `lint:oxlint` reduced from 3m04.7s to about 59s,
     (`doc/planning/oxlint-warm-sweep-attribution.md`);
     an earlier 184-second warm sweep spent 171 seconds inside `prefer-readonly-parameter-types`
     (`doc/planning/issue-486-workspace-ts-source-imports.md`);
  - one Rust crate's `cargo check` at 503 seconds
     (`doc/handover/forbidden-strings-runtime-precompilation.md`).
- Migration surface:
   178 Mise configuration files with 9,422 lines,
   176 `package.json` files,
   19 `Cargo.toml` files,
   Gradle Kotlin DSL builds for the Android app and Kotlin linter,
   plus agent guardrails,
   output filters,
   `AGENTS.md` rules,
   and file-enforcer generation built around `mise run`
   (`doc/planning/mise-removal-coverage.md`).

## Bazel ecosystem evidence

- Bazel 9 is the active LTS and supports Bzlmod only
   (<https://blog.bazel.build/2026/01/20/bazel-9.html>).
- rules_js accepts `lockfileVersion: '9.0'`,
   catalogs through resolved importers,
   and `workspace:*` as `link:` entries
   (`npm/private/pnpm.bzl:124-125`,
   `:295-305`).
  Native pnpm 12 support is on `main` but unreleased as of v3.4.1.
- rules_ts expects dependencies to supply `.d.ts` files
   (`docs/troubleshooting.md:96`);
   no documented path covers depending on `.ts` source through `exports`.
- rules_lint ships no oxlint linter
   (`lint/` lists eslint and stylelint for JavaScript);
   oxlint JS plugins and type-aware mode would need a custom lint aspect.
- No Bazel Central Registry ruleset exists for tsdown or rolldown;
   each needs a `js_run_binary` wrapper.
  dprint fits only as a generic formatter binary.
- rules_rust keeps `Cargo.toml` as the dependency source through `crate.from_cargo`,
   supports dated nightly toolchains,
   and has no cargo-nextest integration.
  rules_fuzzing covers C++ and Java only.
- rules_android `README.md:5-6`:
   "development preview of the Starlark implementation...
   incomplete and may not function as-is".
  No Gradle interop is documented,
   so the Android app would be a full port.
- JetBrains Bazel plugin 2026.2.2.1 describes itself as supporting Java and Kotlin;
   the legacy Google plugin is marked unmaintained.
  rules_js tells TypeScript editor users to run `pnpm install` outside Bazel.

## Why the gains stay small

- Content-addressed caching skips unchanged actions but does not shorten an action.
  The dominant lint cost sits inside one oxlint rule's type analysis.
- Source-level `/ts` imports widen invalidation:
   editing a shared module invalidates every dependent's lint and type-check action anyway.
- A shared remote cache pays off across many developers and CI machines;
   neither exists here at a scale that would amortize it.
- Starlark would add a configuration language.
  This was first recorded as conflicting with a draft TypeScript-canonical direction;
   the user later stated that monorepo manager configuration does not have to be TypeScript,
   so it is no longer a conflict.

## Lighter paths to the same gap

- Mise tasks can declare `sources` and `outputs` for modification-time skipping,
   and Mise has an experimental content-hash task cache
   (mise `docs/tasks/task-configuration.md:444-452`,
   `docs/tasks/caching.md`).
- pnpm 12.4 `pnpm pipeline` runs affected projects with cached outputs restored
   (pnpm.io `docs/cli/pipeline.md`);
   it is already the task-orchestration candidate in `doc/planning/mise-removal-coverage.md`.
  The repository resolves pnpm 12.3.4.

Neither path has been piloted here.

## Relation to Mise removal

The user stated on 2026-09-16 that Mise must be replaced because it is not documented well enough,
with no ETA.
That moves the comparison baseline from Mise to its eventual replacement,
but does not change the answer:

- Bazel core documentation is extensive,
   but the parts this repository would lean on are either self-described as incomplete
   (rules_android)
   or would be repository-owned wrappers with no upstream documentation
   (oxlint,
   tsdown,
   rolldown,
   cargo-nextest).
- The `doc/planning/mise-removal-coverage.md` ledger entries for secrets,
   shell environment,
   editors,
   VM images,
   and the full tool-provisioning set were not evaluated against Bazel.
  Bazel would need to own them or share ownership with other tools before it could be a full Mise replacement.

## Follow-up questions, 2026-09-16

### Watch mode and live inspection

Clones:
 `~/temp/agent/bazel-2026-09-16` at `8e90a0d`,
 `~/temp/agent/bazel-watcher-2026-09-16` at `ed00d96`,
 `~/temp/agent/aspect-cli-2026-09-16`.

- Bazel core has no watch-and-rebuild command.
  `--watchfs` only swaps per-file scanning for the operating system's file-watch service when the next command
   checks for changes
   (`src/main/java/com/google/devtools/build/lib/skyframe/LocalDiffAwareness.java:52-62`).
- `ibazel` from `bazelbuild/bazel-watcher`,
   v0.33.0 released 2026-08-26,
   reruns `build`,
   `test`,
   or `run` on source changes.
  Its outward interfaces are one-directional:
   `ibazel_notify_changes` and `ibazel_notify_changes_v1` send build results to the running target's stdin
   (`README.md:68-86`),
   `--profile_dev` appends one JSON event per line to a file,
   a LiveReload server pushes reloads to browsers,
   and the profiler HTTP server only serves `profiler.js` and accepts browser events
   (`internal/ibazel/profiler/profiler.go:220-240`).
  No endpoint answers a status query.
- The Bazel server's gRPC `CommandServer` exposes only `Run`,
   `Cancel`,
   `UpdateTerminalSize`,
   and `Ping`
   (`src/main/protobuf/command_server.proto:266-280`).
- One output base runs one command at a time;
   the server makes a second command wait,
   printing "Another command (...) is running. Waiting for it to complete on the server"
   (`src/main/java/com/google/devtools/build/lib/runtime/BlazeCommandDispatcher.java:229-265`),
   so a separate `bazel query` cannot inspect a build in progress on the same output base.
  An earlier version cited the client-side lock message in `src/main/cpp/blaze_util_posix.cc`;
   the client releases that lock and the server-side wait is what blocks the second command.
- The supported observation channel is the Build Event Protocol.
  `--build_event_json_file` writes and periodically flushes events during the invocation
   (`src/main/java/com/google/devtools/build/lib/buildeventstream/transports/FileTransport.java:130-150`);
   `--bes_backend` streams the same events to a gRPC Build Event Service
   (`BuildEventServiceOptions.java:35-46`).
  Under `ibazel`,
   each rebuild is a separate invocation with its own event stream.
- The Aspect CLI source contains no watch command;
   its `watch` matches are watchdogs and unrelated flags.

### Community size

The user raised Bazel's community as an advantage over Mise.

- GitHub contributors,
   counted from the last page of `contributors?per_page=1&anon=true`:
   Bazel 1,437,
   Mise 705.
- Stack Overflow tag question counts from `api.stackexchange.com`:
   `bazel` 3,460,
   `mise` 12.
- GitHub stars:
   Mise 33,992,
   Bazel 25,857.
- The advantage thins at this repository's integration points:
   `aspect-build/rules_lint` issue #445,
   "[FR]: oxlint",
   has been open with zero comments since 2024-12-05,
   and rules_android describes itself as an incomplete preview.

### Never running a warm whole-repo oxlint again

The user expects a good monorepo manager to remove the need for warm whole-repo oxlint sweeps.
Any correct affected-only cache must key each package's lint on its own sources,
 the TypeScript sources it imports through `/ts`,
 the shared TypeScript and oxlint configuration,
 and the oxlint plugin sources.

- Direct workspace dependents,
   a lower bound on invalidation fan-out:
   `config-typescript` 146,
   `module-test` 119,
   `config-rolldown` 111,
   `module-logger` 66.
- Of 2,328 commits since 2026-06-18 that touched `.ts` or `.tsx` files,
   320 changed non-test TypeScript under `package/oxlint-plugin/` or `package/config/oxlint/`.
  A correct cache relints every package after each of those commits.
- Most other edits would relint only the changed package and its dependents,
   so affected-only lint would remove most sweeps but not all of them.
- Bazel's sandbox enforces declared inputs,
   which guards against stale cache hits.
  Mise's documented `sources` check compares only the declared files,
   so an undeclared input can produce a wrong skip.
  Whether `pnpm pipeline` enforces inputs was not checked,
   and no cache in either tool was exercised here.

### CI

The user reported that CI is being added.
A CI workflow sharing a remote cache with local runs raises the value of a mature remote cache,
 which Bazel has.
The Mise content-hash cache is experimental;
 remote cache support in `pnpm pipeline` was not checked.

### Hard requirements stated afterwards

The user then set requirements for any monorepo manager:
 full file-enforcer functionality,
 and a watch mode with an RPC or IPC inspection channel.
They are recorded in `doc/planning/mise-removal-coverage.md`,
 section "Monorepo manager hard requirements".
Bazel core meets neither natively:
 it has no watch command and no status query,
 and file generation would come from rulesets or repository-owned rules.

### Bending Bazel to the requirements

Asked after `doc/audit/tech-monorepo-manager-vet-2026-09-16.md` found no qualifying tool.
Bazel exited that vet on a documentation confusion trigger:
`https://bazel.build/docs/configurable-attributes` links "Configurable Build Attributes" to two different URLs.
Clones:
`~/temp/agent/bazel-2026-09-16` at `8e90a0d`,
`~/temp/agent/bazel-lib-2026-09-16` at `7ca5f6d`.

- Inspection:
   the Build Event Protocol is the documented interface
   that "allows third-party programs to gain insight into a Bazel invocation"
   (`docs/remote/bep.mdx:8-10`).
  `--bes_backend=grpc://HOST:PORT` streams those events to any server implementing the public
   `google/devtools/build/v1/publish_build_event.proto` service
   (`docs/remote/bep.mdx:92-100`).
  A repository-owned watch daemon could be that server and answer status queries.
- Control:
   one Bazel server handles at most one invocation;
   concurrent invocations block or fail fast
   (`docs/run/client-server.mdx:12-14`).
  Exit code 8 documents an interrupted build with orderly shutdown
   (`docs/run/scripts.mdx:53`),
   so a daemon that owns the `bazel` client process can cancel it.
  The client-to-server gRPC protocol with `Run` and `Cancel`
   is mentioned only in contributor docs as internal code layout
   (`docs/contribute/codebase.mdx:87-91`),
   not documented for users.
- Queries during a build:
   the scripting guide says a separate `--output_base` avoids contending for the lock used by interactive commands
   (`docs/run/scripts.mdx:12-31`),
   at the cost of a second server.
- Watch:
   Bazel has none;
   `ibazel` reruns commands but reports outward only,
   so the daemon would replace it.
- File generation into the repository:
   bazel-lib `write_source_files` updates checked-in files with `bazel run`,
   and its generated `diff_test` fails when a file is out of date
   (`lib/write_source_files.bzl`,
   module docstring).
  That detects drift;
   reverting an external edit would still be the daemon's job.
  File-enforcer logic would run as declared-input Bazel actions instead of reading arbitrary files at runtime.

Parts that would be repository-built:
the watch daemon,
its documented inspection and control RPC,
the local Build Event Service endpoint,
file-enforcer actions,
protected-destination reverts,
and the ruleset gaps already recorded:
oxlint,
tsdown,
rolldown,
cargo-nextest,
the Android app,
and `.ts` source imports.

## Evidence limits

- No Bazel build of any package was attempted.
- Unverified:
   whether rules_js can run `.ts` files resolved through `exports` in the sandbox,
   and whether oxlint JS plugins or type-aware mode work there.
- The single-author and CI measurements describe the last year,
   not future team growth.
