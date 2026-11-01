# Bazel route design

## Status

- Status:
   not taken.
  The user chose the from-scratch all-Rust route on 2026-09-16
   ([`doc/decision/monorepo-manager-all-rust.md`](../decision/monorepo-manager-all-rust.md)),
   whose design is in
   [`monorepo-manager-from-scratch-design.md`](monorepo-manager-from-scratch-design.md).
  This document stays as the evidence for why Bazel was rejected.
- Route comparison:
   [`monorepo-manager-build-routes.md`](monorepo-manager-build-routes.md).
- Evidence:
   research appendices in [`monorepo-manager-route-research/`](monorepo-manager-route-research/),
   which label every claim verified,
   measured,
   or unverified.
  Nothing in this design was built or run with Bazel.
- Scope:
   design only.
   The user asked for this design expecting it to reveal any disqualifying problem.

## Shape of the route

- Bazel 9 with Bzlmod owns the build and test graph.
- A repository-built daemon owns watching,
   the inspection and control RPC,
   priorities,
   pausing,
   and cgroup placement,
   because Bazel provides none of those as documented user features.
- File-enforcer runs inside the daemon,
   not as Bazel actions:
   `BUILD` globs cannot reach into other packages,
   Bazel actions cannot write outside the workspace,
   and generated `Cargo.toml` and `package.json` files feed Bazel's own dependency fetching
   (`bazel-daemon-platforms.md`).
- Meta Package Manager owns OS packages,
   as in the from-scratch route.
- Cargo stays for crates.io publishing and Rust fuzzing,
   which rules_rust and rules_fuzzing do not cover
   (`bazel-native-languages.md`).

## Components

### TypeScript

Details and citations:
`bazel-typescript.md`.

- rules_js `npm_translate_lock` reads the single-document `lockfileVersion 9.0` lockfile:
   156 importers and 772 packages,
   up to 156 `BUILD` files,
   78 `:pkg` targets,
   and 1,544 external repositories.
- Workspace `/ts` source imports would be modelled by putting `.ts` sources in `js_library` `types`,
   an undocumented path;
   rules_ts documents `.d.ts` dependencies.
- rules_ts pins TypeScript 7.0.2,
   matching the repository.
- oxlint needs a custom lint aspect because rules_lint has none
   (`aspect-build/rules_lint` issue #445).
- tsdown and rolldown run through `js_run_binary` wrappers.
- rules_nodejs publishes no Node 26 toolchain;
   the newest is 24.21.0.

### Rust

Details and citations:
`bazel-native-languages.md`.

- All 19 `Cargo.toml` files declare `[workspace]`,
   by the deliberate no-workspace architecture in `doc/planning/cargo-toml-file-enforcer.md`.
- rules_rust `crate_universe` refuses manifests from more than one workspace root
   (`crate_universe/src/splicing/splicer.rs:58-60`),
   so the design needs 19 dependency repositories,
   3,616 lockfile entries for 1,061 unique crate versions,
   or a Bazel-only resolution that diverges from the Cargo lockfiles.
- rules_rust requires dated nightly toolchains.
- `rust_test` runs a crate's tests in one process,
   unlike cargo-nextest;
   tests that change the working directory or panic hook may interfere.
- rules_rust drops `rustc-link-arg-bin` with a warning,
   which the terminal's `build.rs` needs for its `libghostty-vt.so.0` rpath.

### Android and Kotlin

Details and citations:
`bazel-native-languages.md`.

- rules_android describes itself as an incomplete development preview.
- No `android_instrumentation_test` rule exists in Bazel 9 or rules_android.
- rules_android has no Android Lint support.
- The repository edits Android through Gradle in IntelliJ IDEA;
   the JetBrains Bazel plugin covers Java and Kotlin only.
- rules_kotlin supports Kotlin 2.3 and 2.4;
   the app uses 2.2.10.
- rules_android signs with its own debug keystore unless `debug_key` points at the Gradle keystore;
   a key change forces an uninstall that drops storage access grants.

### Zig

- The repository has no Zig source.
  Zig is reached only through `libghostty-vt-sys`,
   whose build script clones Ghostty and runs `zig build` with network package fetches.
  No Bazel rule builds that kind of `build.zig` project.

### Watch daemon

Details and citations:
`bazel-daemon-platforms.md`.

- WR1:
   the daemon watches the repository,
   computes affected targets,
   and runs `bazel build` and `bazel test`.
- Affected-target queries while a build runs need a second output base,
   because one Bazel server runs one command at a time.
  That second server is a second JVM;
   the development machine had about 18 GiB of memory available with 20 GiB of swap in use,
   and Bazel sets no default heap cap.
- WR2:
   the daemon receives build events through `--bes_backend`.
  The build event stream has no action-started event,
   so the current action-level activity is available only as unstructured progress text.
- WR3:
   the daemon owns each `bazel` client process and cancels with an interrupt;
   the client-to-server `Run` and `Cancel` protocol is documented only as internal code layout.
- Pausing freezes the cgroup that contains both the Bazel server and its actions.
  Test timeouts use wall-clock alarms,
   so a test frozen past its timeout fails after thaw,
   and the cancel request's 10-second deadline fails while frozen.

### Sandboxing

- Bazel creates per-action cgroups only for `linux-sandbox` actions with
   `--experimental_sandbox_limits` or `--experimental_sandbox_memory_limit_mb`,
   covering CPU and memory.
- When cgroups cannot be created,
   Bazel logs and continues without them;
   upstream issue `bazelbuild/bazel#26062` is open,
   and a maintainer stated builds should not fail over it.
- `bazel run` targets,
   `local` or `no-sandbox` actions,
   and persistent workers such as Kotlin get neither sandbox nor per-action cgroups.
- The daemon places Bazel under `--experimental_cgroup_parent` inside its own delegated cgroup;
   a delegated user cgroup on the development machine supported nested cgroups,
   memory and PID limits,
   and freezing.
- On Ubuntu 24 runners,
   AppArmor blocks `linux-sandbox`
   (`bazelbuild/bazel#24081`,
   fix pull request #26434 open).

### Toolchains

- rules_rust needs dated nightlies and `rules_multitool` needs a sha256 per platform,
   which conflicts with the repository's `latest` tool requests.
- The Android SDK still comes from `ANDROID_HOME` unless `hermetic_android_toolchains` replaces it.

## Disqualifying problems

Ranked most severe first.

1. Rust dependency resolution conflicts with the recorded no-workspace architecture;
   `crate_universe` refuses it,
   verified in source.
2. The design depends on behavior Bazel documents only in source or one-line flag help:
   cancellation,
   build event upload failure,
   cgroup fallback,
   and freezing.
   Bazel's documentation already failed the user's confusion rule.
3. Android parity is lost:
   no instrumentation tests,
   no Android Lint,
   no Bazel Android support in the IntelliJ IDEA setup the repository uses,
   and preview-quality rules.
4. Sandboxed lint actions discard the `prefer-readonly-parameter-type` cache,
   so every lint action starts cold;
   repository records put cold whole-repository sweeps at 261 to 305 seconds.
5. Cgroup sandboxing is experimental,
   limit-dependent,
   and fails open,
   against the user's requirement that cgroup sandboxing is a must.
6. Host system libraries such as GTK,
   PipeWire,
   Wayland,
   and Qt are untracked inputs,
   so system upgrades can leave stale outputs.
7. Cargo remains for publishing and fuzzing,
   so Rust keeps two build definitions.
8. pnpm `overrides` shims and `.pnpmfile.mjs` stubs are expected
   to reference package-store targets rules_js never creates;
   this is from source reading and needs a prototype.

## Result

The Bazel route is not viable without giving up at least one of:
the no-workspace Rust architecture,
the user's documentation rule,
Android testing and lint parity,
or cgroup sandboxing that fails closed.
Most watch,
inspection,
control,
file enforcement,
and sandbox policy would still be repository-built.
