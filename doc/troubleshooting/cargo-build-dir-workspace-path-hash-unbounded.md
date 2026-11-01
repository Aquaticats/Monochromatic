# Cargo 1.101.0-nightly `build.build-dir` with `{workspace-path-hash}` grows without bound and is never garbage collected

## Symptom

`~/.cargo` reached 443 GiB on a filesystem with 103 GiB free.
The breakdown:

```text
# du -x -h -d2 ~/.cargo | sort -h | tail
4.0K	/home/user/.cargo/binstall
38M	/home/user/.cargo/bin
806M	/home/user/.cargo/git
4.4G	/home/user/.cargo/registry
438G	/home/user/.cargo/build
443G	/home/user/.cargo
```

`build` was 98.9 percent of the directory.
The registry cache,
which is the part Cargo does
garbage collect,
was 4.4 GiB.

Inside `build` were 171 two-hex-character prefix directories holding 284 workspace trees:

```text
# find ~/.cargo/build -maxdepth 2 -mindepth 2 -type d | wc -l
284
```

Each tree is a complete set of intermediate build artifacts:

```text
# find ~/.cargo/build/2b -maxdepth 3
/home/user/.cargo/build/2b/58e0ca74603674/CACHEDIR.TAG
/home/user/.cargo/build/2b/58e0ca74603674/debug/.cargo-build-lock
/home/user/.cargo/build/2b/58e0ca74603674/debug/incremental
/home/user/.cargo/build/2b/58e0ca74603674/debug/build
/home/user/.cargo/build/2b/58e0ca74603674/.rustc_info.json
/home/user/.cargo/build/2b/d013d29125122f/release/incremental
/home/user/.cargo/build/2b/d013d29125122f/release/build
```

The growth was recent and fast.
`~/.cargo/config.toml` was written at
`2026-09-19 19:27:32`,
the oldest tree dates from `2026-09-23`,
and the directory reached
438 GiB by `2026-10-14`.
That is roughly 20 GiB per day.
No tree was older than 21 days:

```sh
# doc/troubleshooting/cargo-build-dir-workspace-path-hash-unbounded.md
find ~/.cargo/build -maxdepth 2 -mindepth 2 -type d -mtime +21 | wc -l
```

The configuration responsible was two lines:

```toml
# ~/.cargo/config.toml
[build]
build-dir = "{cargo-cache-home}/build/{workspace-path-hash}"
```

Only about 21 git worktrees existed on the system,
against 284 workspace trees.

## Root cause

The [Cargo Book configuration reference][cargo-config] documents `build.build-dir`:

> Type:
> string (path)
> Default:
> Defaults to the value of `build.target-dir`
> Environment:
> `CARGO_BUILD_BUILD_DIR`
>
> The directory where intermediate build artifacts will be stored.
> Intermediate artifacts are
> produced by Rustc/Cargo during the build process.
>
> This option supports path templating.
>
> Available template variables:
>
> - `{workspace-root}` resolves to root of the current workspace.
> - `{cargo-cache-home}` resolves to `CARGO_HOME`
> - `{workspace-path-hash}` resolves to a hash of the manifest path

Because the directory is keyed on a hash of the manifest path,
every distinct workspace path
gets its own complete artifact tree.
Creating a throwaway worktree or scratch checkout
allocates a fresh tree,
and deleting the worktree does not delete the tree,
because the two are
decoupled by path.
Nothing reclaims trees whose workspace path no longer exists.

Cargo's automatic cache cleaning does not cover this.
The same page documents the global cache
behaviour under `[cache]`:

> When running `cargo` commands,
> Cargo will automatically track which files you are using within
> the global cache.
> Periodically,
> Cargo will delete files that have not been used for some
> period of time.
> It will delete files that have to be downloaded from the network if they have
> not been used in 3 months.
> Files that can be generated without network access will be deleted
> if they have not been used in 1 month.

and then excludes build artifacts explicitly:

> **Note**:
> This tracking is currently only implemented for the global cache in Cargo's home
> directory.
> This includes registry indexes and source files downloaded from registries and git
> dependencies.
> Support for tracking build artifacts is not yet implemented,
> and tracked in
> [cargo#13136][cargo-13136].

So `cache.auto-clean-frequency`,
which defaults to `"1 day"`,
runs and does nothing to
`build-dir`.
The 4.4 GiB registry and 806 MiB git directories are the only parts under
tracking,
which matches their small size relative to `build`.

A second consequence follows from `{cargo-cache-home}` resolving to `CARGO_HOME`.
Redirecting
worktrees to another filesystem does not redirect their artifacts.
On this system `~/temp` is a
symlink to `/var/mnt/encrypted/temp`,
so scratch worktree checkouts lived on a 3.6 TiB volume
with 1.6 TiB free,
while every one of their build trees was still written to `~/.cargo/build`
on the 1.86 TiB volume that was nearly full.

## Verification

Version under test:

```text
# cargo --version
cargo 1.101.0-nightly (29c5daa1a 2026-10-07)
```

Documentation read from the live [Cargo Book configuration reference][cargo-config] for the
`build.build-dir` and `[cache]` sections quoted in "Root cause".

Reproduce the accumulation on any system with the setting enabled:

```sh
# doc/troubleshooting/cargo-build-dir-workspace-path-hash-unbounded.md
du -x -h -d2 "${CARGO_HOME:-$HOME/.cargo}" | sort -h | tail -6
find "${CARGO_HOME:-$HOME/.cargo}/build" -maxdepth 2 -mindepth 2 -type d | wc -l
stat -c '%y %n' "${CARGO_HOME:-$HOME/.cargo}/config.toml"
```

Reproduce the path resolution with a throwaway workspace.
This was run to confirm that an
absolute `build-dir` base combined with the template variable is accepted:

```sh
# doc/troubleshooting/cargo-build-dir-workspace-path-hash-unbounded.md
cargo new --vcs none --name probe /tmp/probe
cargo build --manifest-path /tmp/probe/Cargo.toml
find <configured-build-dir> -maxdepth 3
```

With `build-dir = "/var/mnt/encrypted/cargo-build/{workspace-path-hash}"` this produced
`/var/mnt/encrypted/cargo-build/6a/06592ec01d7168/debug`,
confirming that Cargo inserts its own
two-character shard level beneath the configured base and that the build finished normally.

Catalog of settings that stay bounded:

- `build.build-dir` unset.
  Artifacts go to `<workspace>/target`,
  which `git worktree remove`
  deletes along with the worktree.
- `build.build-dir` set without `{workspace-path-hash}`,
  so all workspaces share one tree.

Catalog of settings that grow without bound:

- `build.build-dir` containing `{workspace-path-hash}`,
  on a workflow that creates and discards
  workspace paths.
  Each new path adds a full tree and nothing removes old ones.

## Verified workarounds

Relocate the base of `build-dir` to a filesystem with room,
keeping the template variable.
Applied here and verified by build:

```toml
# ~/.cargo/config.toml
[build]
build-dir = "/var/mnt/encrypted/cargo-build/{workspace-path-hash}"
```

Tradeoffs:
the target filesystem must be mounted whenever a build runs,
and a build started
while it is unmounted writes to the mount point directory on the root filesystem instead of
failing,
silently refilling the disk the move was meant to protect.
On this system the target
is mounted at user login by a `WantedBy=user@1000.service` oneshot unit with retry,
so builds
in a user session see it.

Remove the `build-dir` setting.
Artifacts return to `<workspace>/target` and are deleted with
the worktree,
so growth is self-limiting.
Tradeoff:
no artifact sharing between worktrees,
so
each of the roughly 21 worktrees rebuilds from scratch.
This was the behaviour before
`2026-09-19` and the filesystem did not fill under it.

Delete `~/.cargo/build` outright.
Measured here:
removing it took 58 seconds and,
once the
btrfs snapshots pinning it were also removed,
contributed to available space going from 86 GiB
to 1057 GiB.
Tradeoff:
the next build recompiles every workspace from scratch.

Periodic removal of trees not modified within N days.
Tradeoff:
mtime reflects the last build,
so a workspace that is used but rarely built loses its cache,
and on btrfs a directory mtime is
not a reliable proxy for last use of the artifacts beneath it.

## What does not work

Relying on `cache.auto-clean-frequency` does nothing for `build-dir`.
The Cargo Book states
build-artifact tracking is not implemented,
and the measured sizes agree:
registry stayed at
4.4 GiB while `build` reached 438 GiB under the same `CARGO_HOME`.

`cargo clean` does not reach these trees.
It operates on the workspace's target directory,
and
with `build-dir` redirected the intermediates live outside the workspace,
keyed by a path hash
that a discarded worktree no longer has.

Moving an existing `build-dir` tree to the new location to preserve the cache is unlikely to
work,
and was not adopted.
Generated dep-info records the absolute build directory path.
In the
verification build,
one such file contained the configured path:

```text
/var/mnt/encrypted/cargo-build/6a/06592ec01d7168/debug/build/probe/61fbace85224afae/out/probe.d
```

This is an observation of one file in a hello-world workspace,
and the conclusion that a copied
cache would be invalidated is inference from it,
not a measurement.
Copying 438 GiB to test it
was not worth the I/O,
so deletion was chosen instead.

Redirecting worktree checkouts to another filesystem does not redirect their artifacts while
`{cargo-cache-home}` is the base.
The symlink at `~/temp` moved the checkouts and left every
build tree on the original volume.

## Upstream filing decision

`.out-of-scope/` was checked.
It holds `bun-install.md`,
`cargo-workspace.md`,
`claude-code-upstream-bugs.md`,
`codex-harness.md`,
`jsr.md`,
`lightningcss.md`,
`low-impact-typescript-formatting.md`,
`module-es-monolith.md`,
`pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`,
and `typescript-project-references.md`.
`cargo-workspace.md` concerns the deliberate absence of a repository-root Cargo workspace,
for
profile-ownership reasons,
and does not exempt `build.build-dir` behaviour.
No exemption
applies.

The tracker was searched and a matching open issue exists:
[cargo#13136][cargo-13136],
`Garbage collect whole target/`.
Reading the thread in full,
a maintainer comment already covers
this exact setting:

> #14125 adds a `build-dir` which we will also want to track and clean up.
>
> Once both are stabilized,
> we can consider changing the `workspace-path-hash` to also include
> the Cargo version so that people can clean up caches for old versions of Rust on upgrade.

and a member replied `I'll give it a shot and see if I can implement it and move it forward.`
So `build-dir` tracking,
cleanup,
and the `{workspace-path-hash}` keying are all already in
scope upstream,
with an owner.

The six constraints,
walked:

1. **Is it really upstream's fault?**
   Partly.
   The absence of build-artifact GC is a known
   upstream gap that the Cargo Book itself documents and links to [cargo#13136][cargo-13136].
   The 438 GiB accumulation also required a local choice to enable `build-dir` with
   `{workspace-path-hash}` on a workflow that creates many short-lived workspace paths.
2. **Can upstream fix it?**
   Yes,
   and they intend to.
   The maintainer comment above states the
   plan to track and clean up `build-dir`.
3. **Are they supporting this use case?**
   Yes.
   `build.build-dir` is a documented,
   supported
   configuration key with documented template variables.
4. **Would the repo welcome our contribution?**
   Unevaluated.
   No filing is proposed,
   so no
   `CONTRIBUTING.md` or policy check was run.
5. **Will they likely fix it?**
   Yes.
   The issue is open,
   a maintainer has stated intent,
   and a
   member has volunteered to implement.
6. **Have we prototyped a minimal fix?**
   No. A duplicate with an owner exists,
   so the
   auto-prototype step does not apply.
   Prototyping a GC for Cargo's own cache would also belong
   in the upstream thread rather than here.

Decision:
do not file.
A duplicate exists and is already owned.

Additive content does exist,
but posting to `rust-lang/cargo` is an external mutation of a
repository this project does not own,
so it needs explicit authorization before sending.
The
draft below is kept,
not posted.
Its value over the existing thread is affected-version data
and a relocation workaround the thread does not mention:

~~~md
Affected-version data for the `build-dir` half of this, on a workflow that creates and discards
many short-lived workspace paths (agent worktrees):

- `cargo 1.101.0-nightly (29c5daa1a 2026-10-07)`
- `build-dir = "{cargo-cache-home}/build/{workspace-path-hash}"`
- 284 workspace trees under 171 two-character shard directories
- 438 GiB accumulated between 2026-09-23 and 2026-10-14, roughly 20 GiB per day
- 4.4 GiB in `registry` over the same period, so tracked caching was working while `build-dir`
  was not
- about 21 live worktrees against 284 trees, so the large majority belonged to paths that no
  longer existed

Workaround used, which may be worth documenting until GC lands: keep the template variable but
move the base to another filesystem, `build-dir = "/path/on/other/volume/{workspace-path-hash}"`.
Cargo accepts an absolute base and inserts its own two-character shard level beneath it.

Two caveats for anyone adopting that workaround:

- `{cargo-cache-home}` as the base defeats relocating worktrees themselves. We had scratch
  worktrees on a separate volume via a symlinked home subdirectory, and every build tree still
  landed on the original volume.
- Generated dep-info records the absolute build directory path, for example
  `<build-dir>/<shard>/<hash>/debug/build/<pkg>/<meta>/out/<pkg>.d`. We did not verify whether a
  copied cache is still reusable after relocation, so we deleted rather than migrated. If
  relocation is recommended as a workaround, it would help to state whether an existing cache
  can be moved.
~~~

[cargo-config]: https://doc.rust-lang.org/cargo/reference/config.html
[cargo-13136]: https://github.com/rust-lang/cargo/issues/13136
