# Cargo refuses to build a fixture crate written under a container gate's build folder: "current package believes it's in a workspace when it's not"

## Symptom

The wrapper's container gate ran 7954 tests on `3b94069dbf` and two failed.
One of them proved nothing about the code it covers:

```text
FAIL [   0.125s] (7498/7954) monochromatic-git-policy-cli::native_binary package_repair::a_shared_target_a_link_carries_out_of_the_worktree_is_left_built

thread 'package_repair::a_shared_target_a_link_carries_out_of_the_worktree_is_left_built' (187140) panicked at src/native/binary/binary_package_repair_tests.rs:344:5:
cargo ["build", "--offline", "--quiet"] failed in /work/package/git-policy/cli/target/tmp/native-binary-187134-repair-shared-target/repo: error: current package believes it's in a workspace when it's not:
current:   /work/package/git-policy/cli/target/tmp/native-binary-187134-repair-shared-target/repo/Cargo.toml
workspace: /work/package/git-policy/cli/Cargo.toml

this may be fixable by ensuring that this crate is depended on by the workspace root: /work/package/git-policy/cli/Cargo.toml
Alternatively, to keep it out of the workspace, add the package to the `workspace.exclude` array, or add an empty `[workspace]` table to the package's manifest.
```

The panic line is `run_tool`'s `assert!`
(`package/git-policy/cli/src/native/binary/binary_package_repair_tests.rs`),
reached from the build `shared_target_repository` makes for its fixture.
No test assertion ran:
the fixture's own crate never built.

The same test passes on this host alone:

```text
Summary [  13.161s] 1 test run: 1 passed, 1030 skipped
```

## Root cause

Two locations of one directory decide the outcome.

Every fixture is created below the fixture root,
which is Cargo's per-target scratch directory:

```rust
// package/git-policy/cli/src/native/binary/binary_support_fixture_directories.rs
pub fn fixture_root() -> PathBuf {
    let root: PathBuf = std::env::var_os(FIXTURE_ROOT_VARIABLE)
        .map_or_else(|| { return PathBuf::from(env!("CARGO_TARGET_TMPDIR")); }, PathBuf::from);
```

Cargo sets `CARGO_TARGET_TMPDIR` to the build folder's `tmp` directory when it compiles an integration
test target.
The build folder is in a different place in the two environments:

- this host:
  `cargo-shadow` keys Cargo's build folder by workspace path outside the worktree,
  and the built `native_binary` binary carries that value,
  `/var/mnt/encrypted/cargo-build/toolchain/nightly-2026-11-11/d9/f32e1ff1564338/tmp`,
  which is inside no workspace;
- the gate container:
  the build folder is `/work/package/git-policy/cli/target`,
  which is inside the wrapper crate.

That difference is host configuration,
not repository configuration.
`doc/decision/cargo-build-dir-host-cache.md` records the setting this host carries
(`[build] build-dir = "{cargo-cache-home}/build/{workspace-path-hash}"` in `~/.cargo/config.toml`)
and says of it:
"It does not travel with a clone.
A fresh machine,
 a new contributor,
 or a container build needs the same two lines,
 or intermediates go back into `<package>/target/`."
The container gate is that container build,
so its fixture root is inside the wrapper crate.

The wrapper crate is a workspace root of its own
(`[workspace]` in `package/git-policy/cli/Cargo.toml`).
Cargo therefore walks up from a manifest that declares no workspace and no `package.workspace`
pointer,
and finds the wrapper crate above the fixture crate.

The walk,
in Cargo at `8f71b6c11636d6895a003e605b892831eb26d948`
(`src/workspace/workspace.rs`):

1. `get_ws_root` (`src/workspace/workspace.rs:178`) answers `None` for
   `WorkspaceConfig::Member { .. }`,
   which is what a manifest without a `[workspace]` table is,
   so `find_root` (`src/workspace/workspace.rs:821`) keeps looking upward.
2. `find_root_iter` (`src/workspace/workspace.rs:2243`) walks the ancestors and stops short of
   `target/package` only:

   ```rust
   .take_while(|path| !path.curr.ends_with("target/package"))
   ```

3. The full load then calls `error_if_manifest_not_in_members`
   (`src/workspace/workspace.rs:1023`,
   defined at `src/workspace/workspace.rs:1114`),
   which writes the message above,
   because the wrapper crate's `[workspace]` lists no `members`
   (`has_members_list`,
   `src/workspace/workspace.rs:2025`).

Both environments agree on every other real-Cargo fixture of the same file,
which is why the gate log shows them passing:
`cargo_repository` writes a `[workspace]` root with a members list
(`binary_package_repair_tests.rs:435`),
so Cargo stops there.
The shared-target fixture wrote a lone package manifest,
and so became the first fixture whose build depended on where the build folder is.

## Verification

Versions under test:
`cargo 1.102.0-nightly (c07ac2c4d 2026-11-06)`,
`rustc 1.102.0-nightly (9c5648833 2026-11-10)`,
`cargo-shadow 0.1.5-alpha.9`.
Cargo's own behavior was read at `rust-lang/cargo` commit
`8f71b6c11636d6895a003e605b892831eb26d948`
([`src/workspace/workspace.rs`](https://github.com/rust-lang/cargo/blob/8f71b6c11636d6895a003e605b892831eb26d948/src/workspace/workspace.rs#L1114-L1173)).

Reproduce the gate's layout on this host without running the gate,
from the worktree root,
with a fixture root inside the wrapper crate as the container has it:

```sh
# doc/troubleshooting/cargo-fixture-crate-joins-the-wrapper-workspace.md
env CLI_GIT_TEST_ROOT="$PWD/package/git-policy/cli/target/tmp" mise run \
  //package/git-policy/cli:native:test:binary:host -- \
  'package_repair::a_shared_target_a_link_carries_out_of_the_worktree_is_left_built'
```

That run fails before the fix,
with the gate's words pointed at the worktree instead of `/work`:

```text
cargo ["build", "--offline", "--quiet"] failed in /var/mnt/encrypted/repository/Monochromatic/.claude/worktrees/pi-package-repair-shared-target-fixture-builds-after-pass-12/package/git-policy/cli/target/tmp/native-binary-3411735-repair-shared-target/repo: error: current package believes it's in a workspace when it's not:
current:   .../package/git-policy/cli/target/tmp/native-binary-3411735-repair-shared-target/repo/Cargo.toml
workspace: .../package/git-policy/cli/Cargo.toml
```

The smallest harness,
a workspace root of one package and a lone crate two directories below it,
answers the same way and shows both remedies:

```sh
# doc/troubleshooting/cargo-fixture-crate-joins-the-wrapper-workspace.md
# root: a scratch directory under ${HOME}/temp/agent
# root/Cargo.toml holds [package] and [workspace]; root/src/main.rs is the package's source
# lone crate: root/target/tmp/x/repo, its Cargo.toml holding [package] only
cd -- "${HOME}/temp/agent/probe-workspace-exclude/ws/target/tmp/x/repo"
cargo build --offline --quiet   # the error above: patterns that fail
```

Adding either remedy to that harness builds:

- `[workspace]` in the lone crate's manifest:
  `cargo build` succeeds and writes
  `${HOME}/temp/agent/probe-workspace-exclude/ws/target/tmp/x/repo/target/debug/lone`;
- `exclude = ["target"]` in the root's `[workspace]`:
  the same build succeeds and the lone crate's manifest stays as it is.

## Fix

`shared_target_manifest` in `binary_package_repair_tests.rs` gives the fixture crate a `[workspace]`
table of its own.
A lone package crate is then a workspace root wherever it is placed,
which is Cargo's own second remedy and the same shape `cargo_repository` already used.
The repair's plan for the fixture is unchanged:
`cargo metadata --no-deps` still names the fixture as its workspace root with one member.

The test itself is unchanged and is the control:
it failed before the fix under the gate's layout,
and after it the whole `package_repair` module passes with the fixture root inside the wrapper crate:

```text
Summary [  79.140s] 12 tests run: 12 passed, 1019 skipped
```

`exclude = ["target"]` on the wrapper crate's `[workspace]` was measured and not taken.
It makes the default fixture root inert for Cargo,
but it hides a fixture's dependence on where it sits,
and it covers only the root inside the build folder:
a person pointing `CLI_GIT_TEST_ROOT` anywhere else inside the wrapper crate
(or a suite that moves the fixture root again) would meet the same failure.

## What does not work

- Moving the fixture root out of the build folder:
  `CARGO_TARGET_TMPDIR` is decided when the test binary is compiled,
  and the system temporary directory is refused for it on purpose
  (a shared tmpfs with a per-user quota filled up and broke every command of a session,
  issue 1809).
- Reading the fixture root from the environment in every run:
  `CLI_GIT_TEST_ROOT` exists for the btrfs controls and for probes like the one above,
  so a default that avoids every workspace cannot be assumed.
- A fixture crate that lets the ambient folder above it decide its workspace:
  the failure is silent on a host whose build folder sits outside the worktree,
  which is this repository's normal development layout.

## Upstream filing decision

Nothing to file:
this is not an upstream defect.

1. Is it really upstream's fault?
   No.
   Cargo documents and reports this behavior,
   and its message names both remedies.
   The fixture wrote a package manifest below another workspace root,
   and expected that root's absence to protect it.
2. Can upstream fix it?
   Not applicable after item 1.
3. Are they supporting this use case?
   Yes:
   a lone package crate inside someone else's workspace is deliberately refused,
   and `[workspace]` is the supported way to opt out.
4. Would the repo welcome our contribution?
   Not assessed,
   because there is nothing to contribute.
5. Will they likely fix it?
   Not applicable.
6. Have we prototyped a minimal fix compatible with their architecture?
   Not applicable;
   the fix is in this repository's fixture.
