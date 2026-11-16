# A `cargo build` that says `Finished` leaves another checkout's program at `target/debug/`

Cargo 1.101.0-nightly (3960fa69b 2026-10-12) on this host,
 driven the way this repository's `native:build` used to drive it,
 prints `Finished` in a worktree whose sources are newer than the program it leaves at
 `package/git-policy/cli/target/debug/monochromatic-git-policy-cli`,
 because a unit is fresh exactly when none of its sources is newer than the state a build left,
 and Cargo links that state's program over the named path whether the unit was compiled or not.

## Symptom

An agent working in its own worktree ran `mise run //package/git-policy/cli:native:build` on 2026-11-14 at about
 15:41 and read:

```txt
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.34s
```

The program at `target/debug/monochromatic-git-policy-cli` was from 2026-11-11 and lacked a
 string its sources hold.
Touching `src/native/lib.rs` and `src/native/main.rs` made the next build produce the right program,
 and a run of that agent's reproduction through the older program measured the older code.

A person sees three things,
none of which names a cause:

- the build reports the same success a correct build reports,
- the file the task names exists and runs,
  so a control that starts it by path measures something,
- that something is the program of another set of sources.

The failure belongs to any task that hands a person or a control a program to start by path,
 which in this package is `native:build`,
`native:build:release`,
`build` and the candidate arm of
 `test:service-manager`.

## Root cause

Three facts,
each measured on this host or read from Cargo's own source at commit `f93c676e3e`,
 together produce the symptom.

### 1. A worktree of this repository arrives holding another checkout's `target/`

This repository's `git worktree add` is the wrapper's,
 and the wrapper makes a new worktree a btrfs snapshot of the checkout it is added from
 (`package/git-policy/cli/src/native/subvolume/mod.rs`,
 `package/git-policy/cli/src/native/worktree_copying/`),
 so every file the source checkout held arrives in the worktree,
 `target/debug/` and the program the task names included.

Measured 2026-11-16 in a worktree of this repository added the same way (`stat`):

```txt
# the main checkout                                                       # the new worktree
target                 dev=104      ino=28534                             target                 dev=4194455  ino=28534
target/debug           dev=104      ino=28703                             target/debug           dev=4194455  ino=28703
target/debug/monochromatic-git-policy-cli ino=20526056 links=1           ...                    ino=20526056 links=1
   mtime=2026-11-14 19:10:50                                                mtime=2026-11-14 19:10:50
```

Same inode numbers,
 different filesystem identifiers,
 identical modification times:
 the worktree is the source checkout's tree as of one moment,
 and that moment holds a program built from whatever the source checkout's sources were then.
The worktree's own sources keep the source checkout's modification times as well,
 which was measured on the same pair:
 `src/native/lib.rs` is `2026-11-13 18:44:24` and `src/native/main.rs` is `2026-11-15 22:56:49` in both.

### 2. Cargo decides freshness by modification time alone

`src/compiler/fingerprint/mod.rs:2218` of Cargo,
at commit `f93c676e3e`
 ([the line on GitHub][cargo-freshness]):

```rust
                if path_mtime <= reference_mtime {
                    continue;
                }
```

A source file that is not newer than the reference makes nothing stale,
 and the reference is the dep-info file a build writes last
 (`src/compiler/fingerprint/mod.rs:238-241`,
 [the same file, lines 238 to 241][cargo-mtime-doc]):

```txt
//! [`LocalFingerprint::CheckDepInfo`] is used for checking the mtime of
//! packages. It compares the mtime of the input files (the source files) to
//! the mtime of the dep-info file (which is written last after a build is
//! finished).
```

So a build state belongs to whichever sources were in front of the compiler when it was written,
 and it stays fresh for every later checkout whose files are not newer than it.
A second checkout reaches that state through the wrapper's repair,
 which fills a new worktree's build directory from the source checkout's
 (`package/git-policy/cli/src/native/repair/package_repair_prewarm_job.rs`),
 and through cargo-shadow's keyed build folders,
 which are named by toolchain and workspace path
 (`package/cargo-shadow/cli/src/lean_keyed_folder.rs`),
 so a worktree removed and added again at one path finds the build state of its previous life.

### 3. Cargo links the state's program over the named path even when it compiles nothing

`src/compiler/mod.rs:679-681` of Cargo
 ([the link on GitHub][cargo-link-targets]):

```txt
/// Link the compiled target (often of form `foo-{metadata_hash}`) to the
/// final target. This must happen during both "Fresh" and "Compile".
fn link_targets(
```

and the link replaces what is there
 (`crates/cargo-util/src/paths.rs:622-624`,
 [the removal on GitHub][cargo-link-or-copy]):

```rust
    if fs::symlink_metadata(dst).is_ok() {
        remove_file(&dst)?;
    }
```

So the file at the named path is always Cargo's artifact of that unit,
 which means a stale program there is never a leftover the build forgot to overwrite:
 it is the artifact of a fresh unit that was compiled from other sources.

### What the report of 2026-11-14 cannot be shown to be

The agent's worktree
 (`pi-lock-a-killed-git-left-is-cleared-on-a-volume-where-births-are-equal-by-a-record-the-wrapper-wrote`)
 no longer exists,
 so its files cannot be read,
 and the case was not reproduced there.
What the run could read is the tooling's own records and the build folders the path left behind,
 and they bound what the report can be:

- The add's timing record for one 2026-11-16 worktree names `cargoMetadataWorktree` 35 times,
   `cargoMetadataSource` 35 times and `cargoPrewarmHandOver` once
   (`.git/monochromatic-git-policy-cli-worktree-add-timing.jsonl`),
   so the repair does hand a prewarm over.
- The build folder that prewarm published for that worktree
   (`/var/mnt/encrypted/cargo-build/c8/3937b2232ce7f0`,
  written 04:45)
   holds 261 units and none of them `debug/build/monochromatic-git-policy-cli`:
   the clean removed this crate's own units,
   which is the copy's own guarantee and is what the `clean --package` step below keeps.
- The reported worktree's remaining keyed build folders hold program artifacts from `2026-11-14 12:07`,
   `15:07`,
  `17:11` and `20:59` and none from `2026-11-11`
   (`.../toolchain/nightly-2026-11-14/3e/9495fb5e484910/debug/build/monochromatic-git-policy-cli/*/out/`),
   so a `2026-11-11` program at that path was not linked out of them at 15:41.

One reading of the report that these facts leave is a build whose Cargo resolved a build state,
 or a target directory,
other than the one the agent believed;
 another is a `Finished` line from a build that ran in the main checkout.
Neither can now be told from the other,
 and "Verified workarounds" does not depend on which it was.

## Verification

Version under test:
`cargo 1.101.0-nightly (3960fa69b 2026-10-12)`,
 `rustc 1.101.0-nightly (19833af72 2026-10-20)`,
 with the host setting `build.build-dir = "/var/mnt/encrypted/cargo-build/{workspace-path-hash}"`
 from `$CARGO_HOME/config.toml`,
 and the Cargo source read at `rust-lang/cargo` commit `f93c676e3e7ab1aee08dc95d869700f6b99a1a20`.

### A catalog that fails: a build whose state was written after its sources

The harness is a crate of one file,
 a build of it,
 a rewrite of that file,
 and a rewind of the file's modification time to a moment before that build,
 which is the state a snapshot worktree can hand over.
The rewind is `fs.utimes`,
and a person reproduces it without node by writing the file and then
 `touch --date='2026-11-11 19:00:00'`:

```sh
# in a scratch crate of your own, with CARGO_BUILD_BUILD_DIR unset
printf '[package]\nname = "probe"\nversion = "0.1.0"\nedition = "2021"\n\n[workspace]\n' > Cargo.toml
mkdir src
printf 'fn main() { println!("FROM_THE_STATE_AN_EARLIER_BUILD_MADE"); }\n' > src/main.rs
cargo build --offline
printf 'fn main() { println!("FROM_THESE_SOURCES"); }\n' > src/main.rs
touch --date='2026-11-11 19:00:00' src/main.rs
cargo build --offline
./target/debug/probe
```

It prints `Finished` and then:

```txt
FROM_THE_STATE_AN_EARLIER_BUILD_MADE
```

The same harness with this package's build step
 (`node bin/native-build-fresh.ts --directory <that crate> --bin probe`) prints
 `FROM_THESE_SOURCES`,
 because the step removes the crate's units and the program before it builds.

### A catalog that works: sources a build has not seen

The same crate with a source file whose modification time is *nower*
 leaves the unit stale and rebuilds,
 which is why touching `lib.rs` and `main.rs` fixed the report's build:

```sh
printf 'fn main() { println!("FROM_THESE_SOURCES"); }\n' > src/main.rs   # mtime is now
cargo build --offline    # Compiling probe ... Finished
./target/debug/probe     # FROM_THESE_SOURCES
```

### What a fresh build really does to the named path

Two further measurements bound the mechanism:

- Deleting `target/` and rebuilding a fresh unit restores the program from the build directory
   (`Finished` in 0.01 s,
  the restored file the same inode as the artifact,
  `links=2`),
   which is `link_targets` running for a fresh unit.
- Planting a different program at the named path and rebuilding a fresh unit replaces it,
   measured across btrfs subvolume boundaries as well,
  where the link falls back to a copy
   (`links=1` before and after,
  the file's modification time the copy's own).

So a build never leaves a program it did not link,
 and a stale program means a fresh unit compiled from other sources,
 not a missed copy.

## Verified workarounds

This repository's own:
 `package/git-policy/cli/bin/native-build-fresh.ts`,
 which `native:build` and its siblings now run.
Before it builds,
it asks Cargo where the target and build directories are (`cargo metadata`),
 refuses a target directory that is not this package's own (naming `CARGO_TARGET_DIR` when that moved it),
 removes the program at the named path,
 runs `cargo clean --offline --package monochromatic-git-policy-cli`,
 builds,
and fails naming the path,
the target directory and the build directory when no program is there.

Tradeoffs,
measured 2026-11-16 in a worktree of this repository with `mise exec -- cargo`:

- every build of the task recompiles this crate:
   `clean --package` took 0.21 s and the build after it 22.06 s,
   against 1 m 51 s for a build from an empty build directory (dependency builds stay),
- a build of an unchanged checkout no longer answers in under a second,
- the step refuses a caller whose `CARGO_TARGET_DIR` moves the target directory,
   where a bare `cargo build` accepted it and quietly wrote the program elsewhere,
- a release build needs `--release` in the clean as well as in the build:
   measured 2026-11-16, `cargo clean --offline --package monochromatic-git-policy-cli` alone left the release units
   fresh, and `native:build:release` then printed `Finished` in 0.12 s and linked an artifact built from other sources.
   With `--release` it compiled the crate (40.68 s).

A person's own,
when a program's freshness is in question:

```sh
cd package/git-policy/cli
cargo clean --offline --package monochromatic-git-policy-cli
cargo build --locked --offline --bin monochromatic-git-policy-cli
```

## What does not work

- `touch`ing the sources to force a rebuild works and is what the report's agent did,
   but it depends on knowing the program is stale,
  which is the question.
- Deleting `target/` proves nothing:
   Cargo restores the program from the build directory and reports `Finished` without compiling
   (`doc/decision/cargo-build-dir-host-cache.md`,
   "Deleting `target/` does not force a recompile").
- Comparing the program's modification time to the newest source's fails both ways:
   a fresh unit's link may keep the artifact's own modification time (a hard link inside one filesystem)
   or take the copy's (across subvolumes),
   and a program copied from another checkout keeps that checkout's.
- Reading the build directory's layout to decide what is stale is forbidden to every program here,
   and `doc/decision/cargo-build-dir-host-cache.md` decides that.
- `cargo clean` of the whole package directory (`cargo clean` with no `--package`) also works,
   and throws away every dependency build with it,
   which is what a build from an empty build directory costs (1 m 51 s against 22.06 s).

## Upstream filing decision

Nothing is filed.

- Is it really upstream's fault?
   No. Cargo's rule is documented and deliberate:
   freshness by modification time is the design,
  and a link of a fresh unit's artifact is
   what `cargo build` promises.
  The program of another checkout is what a checkout that
   arrives pre-filled produces.
- Can upstream fix it?
   It could,
  in principle,
  hash sources instead of comparing times
   (the fingerprint module discusses size and checksum modes),
   but that is a performance change nobody asked for,
  and the cheap framing here is ours.
- Are they supporting this use case?
   Cargo documents snapshots and shared build directories only as "do not share a target directory",
   and this repository's shape (a btrfs snapshot worktree,
  a toolchain-keyed build directory,
   a repair that fills one from another checkout) is ours to make safe.
- Would the repo welcome the report?
   Not read for this doc,
  because the first three constraints already decide it.
- Is the report new?
   A search of `rust-lang/cargo` was not run for the same reason;
  the decision does not need it.
- Is it safe to publish?
   Nothing is published.

## A note for whoever meets this again

Start from the two decisions in "Root cause" rather than from the files:
 Cargo's `Finished` says "nothing was newer than the state I have",
 and the file at the path is that state's artifact.
Then ask which build directory and which target directory Cargo resolved
 (`cargo metadata --no-deps --format-version 1`),
 because on this host those are not the ones a reader of the package's `target/` would guess:
 `build.build-dir` keys the first by workspace path,
 cargo-shadow keys it by toolchain as well,
 and the prewarm fills the first from another checkout.

[cargo-freshness]: https://github.com/rust-lang/cargo/blob/f93c676e3e7ab1aee08dc95d869700f6b99a1a20/src/compiler/fingerprint/mod.rs#L2218-L2220
[cargo-mtime-doc]: https://github.com/rust-lang/cargo/blob/f93c676e3e7ab1aee08dc95d869700f6b99a1a20/src/compiler/fingerprint/mod.rs#L238-L241
[cargo-link-targets]: https://github.com/rust-lang/cargo/blob/f93c676e3e7ab1aee08dc95d869700f6b99a1a20/src/compiler/mod.rs#L679-L681
[cargo-link-or-copy]: https://github.com/rust-lang/cargo/blob/f93c676e3e7ab1aee08dc95d869700f6b99a1a20/crates/cargo-util/src/paths.rs#L622-L624
