# Cargo 1.99 nightly: cfg_aliases 0.1.1 and 0.2.1 cause dependency future-incompatibility warnings

Tool:
 Cargo 1.99.0-nightly (`3efb1f477 2026-07-17`) and rustc 1.99.0-nightly
(`0e29c21d9 2026-07-21`).
Surface trigger:
 building `package/music-player/desktop-app` or `package/terminal/app`
with a lockfile resolving `cfg_aliases` 0.1.1 or 0.2.1,
then running `cargo report future-incompatibilities`.
Failure mode:
 Cargo reports `trailing semicolon in macro used in expression position`
from dependency build scripts that invoke the macro.

## Symptom

The report names these packages:

- `glutin-winit` 0.5.0:
  123 warning instances.
- `glutin` 0.32.3:
  135 warning instances.
- `i-slint-backend-selector` 1.17.0:
  8 warning instances.
- `i-slint-backend-testing` 1.17.0:
  15 warning instances.
- `i-slint-backend-winit` 1.17.0:
  95 warning instances.
- `turso_core` 0.6.1:
  21 warning instances.
- `winit` 0.30.13:
  91 warning instances.

Every instance has the same diagnostic:

```text
warning: trailing semicolon in macro used in expression position
= warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
= note: for more information, see issue #79813 <https://github.com/rust-lang/rust/issues/79813>
= note: this warning originates in the macro `$crate::cfg_aliases`
```

The report suggests updating `turso_core` to a 0.7.0 pre-release,
updating `winit` to a 0.31.0 beta,
or notifying each named package's maintainer.
Those suggestions identify the invocation sites,
not the shared defective macro implementation.

This surfaced after rustc PR [#159222][rust-159222] made
`semicolon_in_expressions_from_macros` apply to macros defined in dependency crates.
Rust PR [#159218][rust-159218] proposes completing the transition from a lint to a hard error.

The terminal package showed the same diagnostic after its 0.2 dependency family was updated.
Only `nix` 0.28.0 remained in Cargo's notice:

```text
warning: the following packages contain code that will be rejected by a future version of Rust: nix v0.28.0
```

That warning came from the separate `cfg_aliases` 0.1.1 semver family,
not from the already corrected 0.2 family.

## Root cause

The application lockfile selects the affected macro crate
(`package/music-player/desktop-app/Cargo.lock:1023-1026`):

```toml
[[package]]
name = "cfg_aliases"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "613afe47fcd5fac7ccf1db93babcb082c5994d996f20b8b159f2ad1658eb5724"
```

Each package named by Cargo invokes that macro from its build script.
The source locations from the installed crates are:

```rust
// glutin-winit-0.5.0/build.rs:7
cfg_aliases! {

// glutin-0.32.3/build.rs:5
cfg_aliases! {

// i-slint-backend-selector-1.17.0/build.rs:10
cfg_aliases::cfg_aliases! {

// i-slint-backend-testing-1.17.0/build.rs:6
cfg_aliases::cfg_aliases! {

// i-slint-backend-winit-1.17.0/build.rs:8
cfg_aliases! {

// turso_core-0.6.1/build.rs:7
cfg_aliases! {

// winit-0.30.13/build.rs:8
cfg_aliases! {
```

`cfg_aliases` 0.2.1 parses nested `all(...)` and `any(...)` clauses by recursively invoking itself.
Three recursive expression arms add a semicolon to the nested invocation
(`cfg_aliases-0.2.1/src/lib.rs:255-305`):

```rust
(
    @parser_clause
    $op:ident
    [$({$($grouped:tt)+})*]
    [, $($rest:tt)*]
    $($current:tt)+
) => {
    $crate::cfg_aliases!(@parser_clause $op [
        $(
            {$($grouped)+}
        )*
        {$($current)+}
    ] [
        $($rest)*
    ]);
};

(
    @parser_clause
    $op:ident
    [$({$($grouped:tt)+})*]
    [$tok:tt $($rest:tt)*]
    $($current:tt)*
) => {
    $crate::cfg_aliases!(@parser_clause $op [
        $(
            {$($grouped)+}
        )*
    ] [
        $($rest)*
    ] $($current)* $tok);
};

(
    @parser_clause
    $op:ident
    [$({$($grouped:tt)+})*]
    []
    $($current:tt)+
) => {
    $crate::cfg_aliases!(@parser_emit $op
        $(
            {$($grouped)+}
        )*
        {$($current)+}
    );
};
```

The `all(...)` and `any(...)` parser arms consume those recursive expansions as expressions
(`cfg_aliases-0.2.1/src/lib.rs:309-324`):

```rust
(
    @parser
    all($($tokens:tt)+)
) => {
    $crate::cfg_aliases!(@parser_clause all [] [$($tokens)+])
};

(
    @parser
    any($($tokens:tt)+)
) => {
    $crate::cfg_aliases!(@parser_clause any [] [$($tokens)+])
};
```

Rust previously ignored the trailing semicolon in that position.
The future-incompatible lint preserves the build for dependencies today,
but warns that the token sequence will become invalid.

Upstream PR [cfg_aliases#15][cfg-aliases-15] removed exactly those three semicolons.
The 0.2.1 to 0.2.2 source diff is:

```diff
-        ]);
+        ])
@@
-        ] $($current)* $tok);
+        ] $($current)* $tok)
@@
-        );
+        )
```

The fix is published as `cfg_aliases` 0.2.2,
with crates.io checksum
`f079e83a288787bcd14a6aea84cee5c87a67c5a3e660c30f557a3d24761b3527`.
The seven affected packages request either `cfg_aliases = "0.2.0"` or `"0.2.1"`.
Cargo's default caret requirements admit 0.2.2,
so no top-level package upgrade or manifest override is needed for that semver family.

### Terminal's 0.1 dependency chain

The crates.io `portable-pty` 0.9.0 manifest selects `nix` 0.28
(`portable-pty-0.9.0/Cargo.toml:65-69`):

```toml
[dependencies.nix]
version = "0.28"
features = [
    "term",
    "fs",
]
```

`nix` 0.28.0 in turn requires the unfixed `cfg_aliases` 0.1.1 family
(`nix-0.28.0/Cargo.toml:101-102`):

```toml
[build-dependencies.cfg_aliases]
version = "0.1.1"
```

The 0.1.1 macro contains the same three trailing semicolons in its recursive expression arms
(`cfg_aliases-0.1.1/src/lib.rs:255-300`).
No corrected 0.1 release exists,
and Cargo cannot use 0.2.2 to satisfy a `^0.1.1` requirement.

WezTerm commit `f78f72f2f18bf459561e3681f016365273d3e281` upgrades its workspace to `nix` 0.29
(`Cargo.toml:143`) while `portable-pty` consumes that workspace dependency
(`pty/Cargo.toml:17`):

```toml
# Cargo.toml:143
nix = "0.29"

# pty/Cargo.toml:17
nix = { workspace = true, features = ["term", "fs"] }
```

The default-feature `portable-pty` implementation is unchanged from the 0.9.0 release at that revision.
Its only Rust source differences are `serde_derive` to `serde` import changes behind the disabled
`serde_support` feature in `pty/src/cmdbuilder.rs:4` and `pty/src/lib.rs:45`.

## Verification

The failing catalog is the supplied report,
generated with `cfg_aliases` 0.2.1:

- build scripts with nested `all(...)` clauses warn;
- build scripts with nested `any(...)` clauses warn;
- build scripts combining `not(...)` with a nested clause warn;
- 488 warning instances resolve to the seven invocation sites listed in
  [Symptom](#symptom).

A detached worktree at repository commit
`9db2715d84eaf3290d00f1f28e5b17b361c45ec9`
was used so verification did not change the main worktree.
Cargo performed the targeted update:

```console
$ cargo update \
    --manifest-path package/music-player/desktop-app/Cargo.toml \
    --package cfg_aliases@0.2.1 \
    --precise 0.2.2
    Updating cfg_aliases v0.2.1 -> v0.2.2
```

The generated diff changed only the package version and checksum:

```diff
 name = "cfg_aliases"
-version = "0.2.1"
+version = "0.2.2"
 source = "registry+https://github.com/rust-lang/crates.io-index"
-checksum = "613afe47fcd5fac7ccf1db93babcb082c5994d996f20b8b159f2ad1658eb5724"
+checksum = "f079e83a288787bcd14a6aea84cee5c87a67c5a3e660c30f557a3d24761b3527"
```

The working catalog uses 0.2.2:

```console
$ mise run //package/music-player/desktop-app:lint
Compiling cfg_aliases v0.2.2
Compiling turso_core v0.6.1
Compiling winit v0.30.13
Compiling glutin v0.32.3
Compiling glutin-winit v0.5.0
Compiling i-slint-backend-winit v1.17.0
Compiling i-slint-backend-selector v1.17.0
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

The clean build emitted no `trailing semicolon in macro used in expression position`
or future-incompatibility warning.
The dev-dependency path also compiled cleanly and passed the package tests:

```console
$ mise run //package/music-player/desktop-app:test
Compiling i-slint-backend-testing v1.17.0
Finished `test` profile [unoptimized + debuginfo] target(s)
Summary [3.024s] 94 tests run: 94 passed, 0 skipped
```

Repository-wide remediation advanced every remaining 0.2.1 lock entry to 0.2.2.
The terminal package pins the first post-release `portable-pty` revision with `nix` 0.29;
its generated lockfile contains `nix` 0.29.0 and only `cfg_aliases` 0.2.2
(`package/terminal/app/Cargo.lock:753-756`,
`package/terminal/app/Cargo.lock:2939-2947`).

The terminal Clippy task changed from a notice naming six packages to no future-incompatibility notice.
Its 16 tests pass,
including `pty::tests::spawns_command_and_reads_output` through the upgraded dependency.
The Fedora container verification uses 4 GiB of memory and 4 CPUs.
It independently fetches the Git source,
then passes terminal Clippy,
the release build,
and the same 16 tests.
The truepeak-core 62 tests,
nested-wayland-session 25 tests,
and Rust linter 51 tests also pass.
Android native builds pass for both configured ABIs.
Finally,
the repository-wide 19-package Clippy fanout completed without Cargo's
`packages contain code that will be rejected by a future version of Rust` notice.

## Verified workarounds

For dependencies accepting the 0.2 family,
select released `cfg_aliases` 0.2.2 and commit the package-manager-generated lockfile change.
This advances one transitive build dependency by one patch release without changing application APIs,
Slint,
Turso,
winit,
or glutin versions.
A clean verification build fetches and recompiles the affected build scripts.

The repository applies that update in these standalone lockfiles:

- `package/nested-wayland-session/cli/Cargo.lock`;
- `package/terminal/app/Cargo.lock`;
- `package/linter/rust/Cargo.lock`;
- `package/music-player/android-app/rust/Cargo.lock`;
- `package/music-player/desktop-app/Cargo.lock`;
- `package/music-player/truepeak-core/Cargo.lock`.

For the terminal's incompatible 0.1 family,
pin `portable-pty` to WezTerm revision
`f78f72f2f18bf459561e3681f016365273d3e281`.
That revision keeps the public package version and default-feature implementation at 0.9.0,
but upgrades `nix` to 0.29,
which accepts the corrected 0.2 macro family.

Tradeoffs:
 the terminal now fetches a pinned Git revision of the WezTerm monorepo and its `filedescriptor` path dependency
instead of the crates.io archives.
The generated lockfile also re-resolves flexible Windows dependency edges within their declared version ranges.
The pin is immutable and the terminal's PTY spawn integration test exercises the consumed Linux boundary,
but a future crates.io `portable-pty` release containing `nix` 0.29 or newer should replace the Git source.

## What does not work

- Updating `turso_core` to 0.7.0 pre-release or `winit` to a 0.31.0 beta.
  Those are unrelated top-level migrations for a defect fixed in a shared transitive macro crate.
- Filing reports against all seven named packages.
  The shared fix already shipped in `cfg_aliases` 0.2.2,
  and Slint and winit have already updated their development branches.
- Adding `cfg_aliases` as an application build dependency only to constrain resolution.
  A direct dependency would misstate ownership when the lockfile can select the compatible patch release directly.
- Adding a crates.io `[patch]` to the upstream `cfg_aliases` Git commit.
  That was useful before 0.2.2 was published;
  it now adds a Git source and maintenance work without changing the 0.2-family result.
- Updating only `cfg_aliases` 0.2.1 in the terminal lockfile.
  `nix` 0.28 requires `^0.1.1`,
  so both macro families remain and Cargo still reports `nix`.
- Asking Cargo to select `nix` 0.29 beneath crates.io `portable-pty` 0.9.0.
  The published manifest requires `nix = "0.28"`,
  so Cargo correctly rejects 0.29 as outside that dependency requirement.
- Suppressing `semicolon_in_expressions_from_macros` in application source.
  The invocation sites are dependency build scripts,
  and suppression would not make them valid after rustc turns the construct into a hard error.
- Ignoring the report indefinitely.
  The current compiler transition explicitly targets a future hard error.

## Upstream filing decision

### cfg_aliases 0.2.2

`.out-of-scope/` was checked.
Its Cargo entry covers repository workspace structure,
not dependency future incompatibilities,
so no exemption matches.

Duplicate searches found
[cfg_aliases issue #16][cfg-aliases-16],
which reports the same nightly diagnostic,
and [cfg_aliases PR #15][cfg-aliases-15],
which fixed it and was released as 0.2.2.
There is nothing additive to post.

1. Is it really upstream's fault?
    Yes.
    `cfg_aliases` 0.2.1 emitted trailing semicolons from expression macro arms.
2. Can upstream fix it?
    Yes.
    Removing the three semicolons fixes the token stream.
3. Are they supporting this use case?
    Yes.
    Invoking `cfg_aliases!` from `build.rs` is the crate's documented purpose.
4. Would the repository welcome the contribution?
    Yes.
    No contribution or issue template in the cloned repository prohibits external or assisted contributions,
    and the maintainer merged PR #15.
5. Will they likely fix it?
    Yes,
    and they already did:
    PR #15 was merged and release 0.2.2 was published on 2026-07-16.
6. Has a minimal fix compatible with the architecture been prototyped?
    Yes.
    The released patch changes only the three expression-producing macro arms,
    and the detached-worktree application build verifies the released crate against the observed dependency graph.

Decision:
 do not open an issue and do not comment on the closed duplicate.
Issue #16 already records the symptom,
PR #15 records the minimal fix,
and the 0.2.2 release is available from crates.io.

### portable-pty release

No `.out-of-scope/` entry matches `portable-pty` dependency releases.
Searches across open and closed WezTerm issues and pull requests found no request covering the `nix` 0.29 bump.
Issue [wezterm#3108][wezterm-3108] concerned an older 0.8.0 release and does not cover this warning.

1. Is it really upstream's fault?
    Yes.
    Rust changed the diagnostic,
    but only upstream can publish its existing `nix` 0.29 update for crates.io consumers.
2. Can upstream fix it?
    Yes.
    Publishing the existing package state would let registry consumers leave `cfg_aliases` 0.1.1.
3. Are they supporting this use case?
    Yes.
    `portable-pty` is an independently published crate and documents its crates.io API.
4. Would the repository welcome the contribution?
    Yes.
    `CONTRIBUTING.md` welcomes code and documentation contributions,
    the feature-request template accepts problem and solution proposals,
    and no policy prohibits assisted reports.
5. Will they likely fix it?
    There is no refusal signal.
    The maintainer published portable-pty 0.8.0 in response to issue #3108,
    and the needed dependency update already exists on main.
6. Has a minimal fix compatible with the architecture been prototyped?
    Yes.
    The repository pin uses the exact upstream revision containing the `nix` 0.29 update;
    Clippy emits no future-incompatibility notice and the PTY spawn integration test passes.

Decision:
 keep the following release-request draft for human review rather than posting it automatically.

~~~md
Title: Publish portable-pty with the nix 0.29 dependency update
Labels: enhancement, needs:triage

**Is your feature request related to a problem? Please describe.**

The crates.io `portable-pty` 0.9.0 manifest selects `nix` 0.28 at `pty/Cargo.toml:17` in the release source.
That version of `nix` requires `cfg_aliases` 0.1.1.
Rust 1.99 nightly reports the macro's recursive expression arms at `cfg_aliases/src/lib.rs:255-300` as
`trailing semicolon in macro used in expression position` and warns that this will become a hard error.

A consumer build reports:

```text
warning: the following packages contain code that will be rejected by a future version of Rust: nix v0.28.0
```

**Describe the solution you'd like**

Please publish a `portable-pty` release containing the `nix` 0.29 dependency update already present in commit
`f78f72f2f18bf459561e3681f016365273d3e281` at the workspace `Cargo.toml:143`.

**Describe alternatives you've considered**

Pinning `portable-pty` to that revision removes `cfg_aliases` 0.1.1,
eliminates Cargo's future-incompatibility notice,
and passes our PTY spawn integration test.
Updating only `cfg_aliases` 0.2.1 does not help because `nix` 0.28 requires the incompatible 0.1 family.
Our temporary workaround is the immutable Git revision pin.

**Additional context**

The revision keeps `portable-pty` at version 0.9.0.
Relative to the published default-feature source,
its only Rust changes are optional `serde_support` import updates in `pty/src/cmdbuilder.rs:4` and `pty/src/lib.rs:45`.
~~~

[cfg-aliases-15]: https://github.com/katharostech/cfg_aliases/pull/15
[cfg-aliases-16]: https://github.com/katharostech/cfg_aliases/issues/16
[rust-159218]: https://github.com/rust-lang/rust/pull/159218
[rust-159222]: https://github.com/rust-lang/rust/pull/159222
[wezterm-3108]: https://github.com/wezterm/wezterm/issues/3108
